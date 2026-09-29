/**
 * The activity tray's rows: every long piece of work running now, whichever
 * screen started it. Analyses and exports come from the run tracker; model
 * downloads, component installs, YouTube imports and uploads are read from
 * the engine, often while something runs and seldom otherwise.
 */
import { useEffect, useState } from 'react';

import { TaskState } from '@clipmill/contracts';

import { currentStage, stageCounts, stageRows } from '../analysis/model.js';
import type { ShellApi } from '../daemon/api.js';
import type { Job, YoutubeImport } from '../daemon/client.js';
import type { Components } from '../daemon/components.js';
import type { ModelLibrary } from '../daemon/models.js';
import type { YoutubeUpload } from '../daemon/publishing.js';
import { formatBytes } from '../deviceProfile.js';
import { ANALYZE_KIND, formatProgress } from '../library/model.js';
import { isDownloading, percentOf, progressText } from '../models/describe.js';

export type ActivityKind = 'analysis' | 'export' | 'download' | 'install' | 'import' | 'upload';

/** Where a row leads: a run's own progress screen, or the section it belongs to. */
export type ActivityTarget =
  | { readonly kind: 'analysis'; readonly projectId: string; readonly jobId: string }
  | { readonly kind: 'section'; readonly sectionId: string };

export interface ActivityItem {
  readonly id: string;
  readonly kind: ActivityKind;
  readonly title: string;
  readonly detail: string;
  /** How far along, from 0 to 1, when the work measures it; null when it does not. */
  readonly fraction: number | null;
  readonly target: ActivityTarget;
}

const share = (done: number, total: number) => (total > 0 ? Math.min(1, done / total) : null);

/** Analyses and exports still running, from the run tracker. */
export function runItems(
  runs: readonly Job[],
  projectNames: ReadonlyMap<string, string>,
): readonly ActivityItem[] {
  return runs.map((job) => {
    const project = projectNames.get(job.projectId) ?? 'a project';
    if (job.kind === ANALYZE_KIND) {
      const rows = stageRows(job);
      const counts = stageCounts(rows);
      const stage = currentStage(rows);
      return {
        id: `job:${job.jobId}`,
        kind: 'analysis',
        title: `Analysing ${project}`,
        detail:
          stage === null
            ? 'Waiting to start'
            : `${stage.stage.label} · ${counts.done} of ${counts.planned} steps`,
        fraction: share(counts.done, counts.planned),
        target: { kind: 'analysis', projectId: job.projectId, jobId: job.jobId },
      };
    }
    const running = job.tasks.find((task) => task.state === TaskState.RUNNING);
    const progress = running?.progress ?? null;
    return {
      id: `job:${job.jobId}`,
      kind: 'export',
      title: `Exporting from ${project}`,
      detail: running ? (formatProgress(progress) ?? 'Rendering') : 'Waiting to start',
      fraction: progress ? share(progress.done, progress.total) : null,
      target: { kind: 'section', sectionId: 'export' },
    };
  });
}

/** What the engine reports running outside the job queue. */
export interface EngineWork {
  readonly models: ModelLibrary | null;
  readonly components: Components | null;
  readonly imports: readonly YoutubeImport[];
  readonly uploads: readonly YoutubeUpload[];
}

const IMPORTING = new Set(['queued', 'downloading', 'processing', 'registering']);
const UPLOADING = new Set([
  'queued',
  'verifying',
  'starting',
  'uploading',
  'reconciling',
  'publishing',
]);

export function engineItems(work: EngineWork): readonly ActivityItem[] {
  const downloads = (work.models?.models ?? []).filter(isDownloading).map((model): ActivityItem => {
    const percent = model.download ? percentOf(model.download) : undefined;
    return {
      id: `model:${model.name}`,
      kind: 'download',
      title: model.title,
      detail: model.download ? progressText(model.download) : 'Downloading',
      fraction: percent === undefined ? null : percent / 100,
      target: { kind: 'section', sectionId: 'models' },
    };
  });
  const installs = (work.components?.parts ?? [])
    .filter((part) => part.state === 'queued' || part.state === 'installing')
    .map((part): ActivityItem => ({
      id: `component:${part.name}`,
      kind: 'install',
      title: `Installing ${part.title}`,
      detail: part.state === 'queued' ? 'Waiting its turn' : part.detail || 'Installing',
      fraction: null,
      target: { kind: 'section', sectionId: 'models' },
    }));
  const imports = work.imports
    .filter((item) => IMPORTING.has(item.state))
    .map((item): ActivityItem => ({
      id: `import:${item.importId}`,
      kind: 'import',
      title: item.title || 'A YouTube video',
      detail:
        item.state === 'downloading' && item.totalBytes
          ? `Downloading · ${formatBytes(item.downloadedBytes)} of ${formatBytes(item.totalBytes)}`
          : item.state === 'queued'
            ? 'Waiting to start'
            : 'Preparing the recording',
      fraction:
        item.state === 'downloading' && item.totalBytes
          ? share(item.downloadedBytes, item.totalBytes)
          : null,
      target: { kind: 'section', sectionId: 'new-project' },
    }));
  const uploads = work.uploads
    .filter((upload) => UPLOADING.has(upload.state))
    .map((upload): ActivityItem => ({
      id: `upload:${upload.uploadId}`,
      kind: 'upload',
      title: upload.metadata.title || 'A clip',
      detail:
        upload.state === 'uploading' && upload.totalBytes > 0
          ? `Uploading to ${upload.channelTitle || 'YouTube'} · ${formatBytes(upload.acknowledgedBytes)} of ${formatBytes(upload.totalBytes)}`
          : `Uploading to ${upload.channelTitle || 'YouTube'}`,
      fraction:
        upload.state === 'uploading' ? share(upload.acknowledgedBytes, upload.totalBytes) : null,
      target: { kind: 'section', sectionId: 'settings' },
    }));
  return [...downloads, ...installs, ...imports, ...uploads];
}

type EngineApi = Pick<
  ShellApi,
  'listModels' | 'listComponents' | 'listYoutubeImports' | 'listYoutubeUploads'
>;

const EMPTY: EngineWork = { models: null, components: null, imports: [], uploads: [] };
/** Read often while something runs or the tray is open, seldom otherwise. */
const BUSY_EVERY_MS = 3_000;
const IDLE_EVERY_MS = 15_000;

/** The engine's own work, kept current while `connected`. */
export function useEngineWork(connected: boolean, looking: boolean, api: EngineApi): EngineWork {
  const [work, setWork] = useState<EngineWork>(EMPTY);
  const busy = engineItems(work).length > 0;
  useEffect(() => {
    if (!connected) {
      setWork(EMPTY);
      return;
    }
    let live = true;
    const read = () => {
      void Promise.all([
        api.listModels().catch(() => null),
        api.listComponents().catch(() => null),
        api.listYoutubeImports().catch(() => [] as readonly YoutubeImport[]),
        api.listYoutubeUploads().catch(() => [] as readonly YoutubeUpload[]),
      ]).then(([models, components, imports, uploads]) => {
        if (live) setWork({ models, components, imports, uploads });
      });
    };
    read();
    const timer = window.setInterval(read, busy || looking ? BUSY_EVERY_MS : IDLE_EVERY_MS);
    return () => {
      live = false;
      window.clearInterval(timer);
    };
  }, [api, busy, connected, looking]);
  return work;
}
