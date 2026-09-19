/**
 * Choosing a file, and starting the run it becomes.
 *
 * Two sequences, both behind the same seam every other screen uses, so the whole
 * import can be exercised without a window or a daemon.
 *
 * The ordering is forced by the daemon's model rather than chosen: probing is
 * registering, registering needs a project, and a project needs a name — which
 * is why the name comes from the file rather than from a field. A project is
 * therefore created the moment a file is chosen, and reused if the choice
 * changes, so abandoning the screen leaves at most one empty project rather than
 * one per attempt. The Library lists it honestly as not analyzed.
 *
 * The probe arrives inline with the registration rather than being read by
 * address, because the artifact that carries it is published by the analysis
 * DAG's first task — which has not run yet, and cannot be made to run just so a
 * screen can print a duration.
 */
import type { SourceMap } from '@clipmill/contracts';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import type { Job, Source, YoutubeImport } from '../daemon/client.js';
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

  /** Start the analysis, in the units the contract keeps. */
  start(chosen: ChosenSource, settings: ImportSettings): Promise<Job> {
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
