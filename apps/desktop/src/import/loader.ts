/**
 * Source registration and analysis submission through `ShellApi`.
 * Registration requires a project, created from the chosen file's name and reused
 * if the selection changes. Registration returns probe data inline because the
 * analysis DAG has not yet published the probe artifact.
 */
import type { SourceMap } from '@clipmill/contracts';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import { type Job, type Source, type YoutubeImport, isTauri } from '../daemon/client.js';
import { rememberLook } from '../results/captionLook.js';
import { type ImportSettings, languageSubtag, projectNameFor, secondsToTicks } from './model.js';

/**
 * The probe as the daemon canonicalized it.
 *
 * A document that will not parse becomes an absent one rather than an exception:
 * the file is still importable, and the screen already knows how to show a
 * duration it does not have.
 */
function parseProbe(json: string): SourceMap | null {
  if (json === '') {
    return null;
  }
  try {
    return JSON.parse(json) as SourceMap;
  } catch {
    return null;
  }
}

export interface ChosenSource {
  readonly projectId: string;
  readonly source: Source;
  /** The probe. Null when the daemon served a document that would not parse. */
  readonly sourceMap: SourceMap | null;
  /** True when an unchanged file avoided a second probe. */
  readonly cached: boolean;
  /** Actual remote metadata, when the source was imported from YouTube. */
  readonly title?: string;
}

/** A file dragged over the window, or let go on it. */
export interface FileDrop {
  readonly kind: 'over' | 'leave' | 'drop';
  readonly paths: readonly string[];
}

/** The recordings New Project accepts, by extension. */
export const VIDEO_EXTENSIONS = ['mp4', 'mov', 'mkv', 'webm', 'm4v', 'avi'] as const;

export function isVideoPath(path: string): boolean {
  const extension = path.split('.').pop()?.toLowerCase() ?? '';
  return (VIDEO_EXTENSIONS as readonly string[]).includes(extension);
}

export class ImportLoader {
  constructor(readonly api: ShellApi = daemonApi) {}

  /** The native dialog. `null` when it was closed without choosing. */
  choose(): Promise<string | null> {
    return this.api.chooseSourceFile();
  }

  /**
   * Register a chosen file, which probes it.
   *
   * `existingProjectId` is passed back on a second choice so a person changing
   * their mind does not leave a project behind for every file they looked at.
   */
  async register(absolutePath: string, existingProjectId: string | null): Promise<ChosenSource> {
    const projectId =
      existingProjectId ?? (await this.api.createProject(projectNameFor(absolutePath)));
    const registered = await this.api.registerSource(projectId, absolutePath);
    return {
      projectId,
      source: registered.source,
      sourceMap: parseProbe(registered.sourceMapJson),
      cached: registered.observationCacheHit,
    };
  }

  /** Open the exact registered source of a completed import, including after relaunch. */
  async imported(record: YoutubeImport): Promise<ChosenSource> {
    if (record.state !== 'completed' || !record.sourceId) {
      throw new Error('This video has not finished importing.');
    }
    const detail = await this.api.getSource(record.sourceId);
    if (
      detail.source.projectId !== record.projectId ||
      detail.source.sourceId !== record.sourceId
    ) {
      throw new Error('The imported source did not match this project.');
    }
    return {
      projectId: record.projectId,
      source: detail.source,
      sourceMap: parseProbe(detail.sourceMapJson),
      cached: false,
      ...(record.title ? { title: record.title } : {}),
    };
  }

  /**
   * Hear files dropped on the window, as paths. Only the desktop shell can say
   * where a dropped file lives; elsewhere nothing is ever heard.
   */
  async watchDrops(listener: (drop: FileDrop) => void): Promise<() => void> {
    if (!isTauri()) return () => {};
    const { getCurrentWebview } = await import('@tauri-apps/api/webview');
    return getCurrentWebview().onDragDropEvent((event) => {
      const payload = event.payload;
      if (payload.type === 'enter') listener({ kind: 'over', paths: payload.paths });
      else if (payload.type === 'over') listener({ kind: 'over', paths: [] });
      else if (payload.type === 'leave') listener({ kind: 'leave', paths: [] });
      else listener({ kind: 'drop', paths: payload.paths });
    });
  }

  /** Start the analysis, in the units the contract keeps. */
  async start(chosen: ChosenSource, settings: ImportSettings): Promise<Job> {
    const job = await this.submit(chosen, settings);
    if (settings.captionLook) rememberLook(chosen.projectId, settings.captionLook);
    return job;
  }

  private submit(chosen: ChosenSource, settings: ImportSettings): Promise<Job> {
    return this.api.submitAnalyze(chosen.projectId, {
      sourceId: chosen.source.sourceId,
      language: languageSubtag(settings),
      minTicks: secondsToTicks(settings.minSeconds),
      maxTicks: secondsToTicks(settings.maxSeconds),
      count: settings.count,
      contentProfile: settings.contentProfile ?? 'interview',
      localEditorial: (settings.editorialRoute ?? 'local') === 'local',
      ...(settings.editorialRoute === 'cloud'
        ? {
            cloudEditorial: {
              transcriptConsent: settings.cloudConsent === true,
              budgetMicroUsd: Math.round((settings.cloudBudgetUsd ?? 2) * 1_000_000),
              model: 'claude-sonnet-4-6',
            },
          }
        : {}),
    });
  }
}
