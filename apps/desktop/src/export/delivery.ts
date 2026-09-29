/**
 * Follow export render and delivery tasks until completion or failure.
 * Delivered filenames, sizes, and digests come from the published export package,
 * which records the files actually written rather than the plan's predictions.
 */
import { JobState, TaskState } from '@clipmill/contracts';
import type { ExportPackage } from '@clipmill/contracts';
import { useEffect, useState } from 'react';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import type { Job, QueuedExport, Task } from '../daemon/client.js';

const PACKAGE_KIND = 'export.package.v1';
const RENDER_KIND = 'render.clip.v1';
const EXPORT_JOB_KIND = 'export-clip';
/** How often the job is re-read while it runs. A local socket; cheap. */
const POLL_MILLIS = 750;
/** How long to wait before asking again after a read that failed. */
const RETRY_MILLIS = 2_000;
const RECENT_EXPORTS_KEY = 'clipmill.export.recent-rates';

interface ExportRate {
  readonly jobId: string;
  readonly secondsPerMediaSecond: number;
}

function recentExportRates(): ExportRate[] {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(RECENT_EXPORTS_KEY) ?? '[]');
    if (!Array.isArray(parsed)) return [];
    return parsed
      .filter(
        (entry): entry is ExportRate =>
          typeof entry === 'object' &&
          entry !== null &&
          typeof (entry as ExportRate).jobId === 'string' &&
          Number.isFinite((entry as ExportRate).secondsPerMediaSecond) &&
          (entry as ExportRate).secondsPerMediaSecond > 0,
      )
      .slice(-8);
  } catch {
    return [];
  }
}

/** Remember actual wall time on this machine, including export delivery. */
export function rememberExportRate(
  jobId: string,
  mediaSeconds: number,
  startedMillis: number,
  endedMillis: number,
): void {
  if (mediaSeconds <= 0 || endedMillis <= startedMillis) return;
  const samples = recentExportRates();
  if (samples.some((sample) => sample.jobId === jobId)) return;
  const secondsPerMediaSecond = (endedMillis - startedMillis) / 1000 / mediaSeconds;
  try {
    localStorage.setItem(
      RECENT_EXPORTS_KEY,
      JSON.stringify([...samples, { jobId, secondsPerMediaSecond }].slice(-8)),
    );
  } catch {
    // An unavailable local store only removes the estimate.
  }
}

/** Median avoids one unusually slow render dominating future estimates. */
export function estimatedExportSeconds(mediaSeconds: number, fractionDone: number): number | null {
  const rates = recentExportRates()
    .map((sample) => sample.secondsPerMediaSecond)
    .sort((a, b) => a - b);
  if (rates.length === 0 || mediaSeconds <= 0) return null;
  const rate = rates[Math.floor(rates.length / 2)] ?? 0;
  return Math.max(0, Math.ceil(rate * mediaSeconds * (1 - Math.max(0, Math.min(1, fractionDone)))));
}

/**
 * Find a document's newest export in durable daemon jobs so delivery state
 * can be restored after navigation or application restart.
 */
export function latestExportOf(jobs: readonly Job[], docId: string): QueuedExport | null {
  const mine = jobs
    .filter((job) => job.kind === EXPORT_JOB_KIND && job.export?.docId === docId)
    .toSorted((left, right) => right.createdUnixMillis - left.createdUnixMillis);
  const job = mine[0];
  if (!job?.export) {
    return null;
  }
  return {
    jobId: job.jobId,
    revision: job.export.revision,
    irArtifactId: job.export.irArtifactId,
    destinationDir: job.export.destinationDir,
  };
}

export type StageState = 'waiting' | 'running' | 'done' | 'failed' | 'cancelled';

export interface DeliveryStage {
  /** `render` or `deliver`. */
  readonly kind: 'render' | 'deliver';
  readonly label: string;
  readonly state: StageState;
  /** What the stage has done so far, when it says. */
  readonly progress: {
    readonly unit: string;
    readonly done: number;
    readonly total: number;
  } | null;
  /** Why the stage is waiting, when the daemon says. */
  readonly waitReason: string;
}

/** Worker counters are media time, not elapsed wall time or an ETA. */
export function deliveryProgressText(progress: NonNullable<DeliveryStage['progress']>): string {
  if (progress.unit.startsWith('export.')) {
    const label =
      {
        'export.measuring': 'Measuring sound',
        'export.rendering': 'Rendering picture',
        'export.checking': 'Checking result',
      }[progress.unit] ?? 'Exporting';
    return `${label} · ${Math.floor((100 * progress.done) / Math.max(1, progress.total))}%`;
  }
  if (progress.unit === 'media_millis') {
    const clock = (millis: number) => {
      const tenths = Math.max(0, Math.floor(millis / 100));
      const seconds = Math.floor(tenths / 10);
      return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}.${tenths % 10}`;
    };
    return `${clock(progress.done)} of ${clock(progress.total)} rendered`;
  }
  if (progress.unit === 'frames' || progress.unit === 'files')
    return `${progress.done.toLocaleString()} of ${progress.total.toLocaleString()} ${progress.unit}`;
  return `${Math.floor((100 * progress.done) / progress.total)}% complete`;
}

/** Translate durable scheduling reasons without exposing their machine codes. */
export function deliveryWaitText(stage: DeliveryStage): string {
  switch (stage.waitReason) {
    case 'waiting: admission':
      return 'Waiting to start';
    case 'waiting: dependencies':
      return stage.kind === 'deliver' ? 'After rendering' : 'Waiting for preparation';
    case 'retry: daemon restart':
      return 'Resuming after restart';
    case 'retry: transient failure':
      return 'Trying this step again';
    case 'retry: lease expired':
      return 'Restarting an interrupted step';
    case 'waiting: identical output':
      return 'Waiting for the same step in another run';
    default:
      return stage.waitReason || 'waiting';
  }
}

/** One file the delivery wrote, with the path it is at. */
export interface DeliveredPath {
  readonly name: string;
  readonly path: string;
  readonly role: string;
  readonly bytes: number;
}

export interface Delivery {
  readonly renderArtifactId?: string | undefined;
  readonly revision: number;
  readonly destinationDir: string;
  readonly createdUnixMillis?: number | undefined;
  readonly updatedUnixMillis?: number | undefined;
  readonly stages: readonly DeliveryStage[];
  /** True once the job has settled, delivered or not. */
  readonly settled: boolean;
  /** The files, from the package the delivery wrote. Null until it has. */
  readonly files: readonly DeliveredPath[] | null;
  /** The daemon's account of what went wrong, when something did. */
  readonly failure: string | null;
  /**
   * Why the last read of the job failed, while it is still being followed.
   * Not an outcome: the job is durable and goes on, and the read is tried
   * again until the job itself says how it ended.
   */
  readonly interruption: string | null;
}

function stageState(task: Task | undefined): StageState {
  switch (task?.state) {
    case TaskState.RUNNING:
      return 'running';
    case TaskState.SUCCEEDED:
      return 'done';
    case TaskState.FAILED:
      return 'failed';
    case TaskState.CANCELLED:
      return 'cancelled';
    default:
      return 'waiting';
  }
}

/** The two stages of an export job, by what they publish. */
export function deliveryStages(job: Job | null): readonly DeliveryStage[] {
  const render = job?.tasks.find((task) => task.outputKind === RENDER_KIND);
  const deliver = job?.tasks.find((task) => task.outputKind === PACKAGE_KIND);
  const stage = (kind: 'render' | 'deliver', label: string, task: Task | undefined) => ({
    kind,
    label,
    state: stageState(task),
    progress: task?.progress && task.progress.total > 0 ? task.progress : null,
    waitReason: task?.waitReason ?? '',
  });
  return [stage('render', 'Render', render), stage('deliver', 'Deliver', deliver)];
}

/** Whether a job has stopped, one way or another. */
export function settled(job: Job | null): boolean {
  return (
    job !== null &&
    (job.state === JobState.SUCCEEDED ||
      job.state === JobState.FAILED ||
      job.state === JobState.CANCELLED)
  );
}

/** The files a package names, as paths under the folder they were written to. */
export function deliveredPaths(
  pkg: ExportPackage,
  destinationDir: string,
): readonly DeliveredPath[] {
  const folder = destinationDir.endsWith('/') ? destinationDir.slice(0, -1) : destinationDir;
  return pkg.files.map((file) => ({
    name: file.name,
    path: `${folder}/${file.name}`,
    role: file.role,
    bytes: file.bytes,
  }));
}

/** The daemon's reason, or a sentence when it gave none. */
export function failureOf(job: Job | null): string | null {
  if (!job || job.state === JobState.SUCCEEDED) {
    return null;
  }
  if (job.state === JobState.CANCELLED) {
    return 'The export was cancelled before it delivered anything.';
  }
  if (job.state === JobState.FAILED) {
    return job.failureDetail === ''
      ? 'The export failed and the daemon gave no reason.'
      : job.failureDetail;
  }
  return null;
}

/**
 * Poll the two-task export job until it settles or the selected export changes.
 * Retry failed reads: a lost socket or daemon restart does not determine the
 * durable job's outcome.
 */
export function useDelivery(
  projectId: string | null,
  queued: QueuedExport | null,
  api: ShellApi = daemonApi,
): Delivery | null {
  const [job, setJob] = useState<Job | null>(null);
  const [files, setFiles] = useState<{ jobId: string; value: readonly DeliveredPath[] } | null>(
    null,
  );
  const [problem, setProblem] = useState<string | null>(null);

  useEffect(() => {
    setJob(null);
    setFiles(null);
    setProblem(null);
    if (!queued || !projectId) {
      return undefined;
    }
    let live = true;
    let timer: ReturnType<typeof setTimeout> | null = null;
    const read = async () => {
      try {
        const current = await api.fetchJob(queued.jobId);
        if (!live) {
          return;
        }
        setJob(current);
        setProblem(null);
        if (!settled(current)) {
          timer = setTimeout(() => void read(), POLL_MILLIS);
          return;
        }
        const deliver = current.tasks.find(
          (task) => task.outputKind === PACKAGE_KIND && task.outputArtifactId !== '',
        );
        if (current.state === JobState.SUCCEEDED && deliver) {
          const document = await api.readDocument(projectId, deliver.outputArtifactId);
          if (live) {
            setFiles({
              jobId: queued.jobId,
              value: deliveredPaths(
                JSON.parse(document.json) as ExportPackage,
                queued.destinationDir,
              ),
            });
          }
        }
      } catch (error) {
        if (live) {
          setProblem((error as Error).message);
          timer = setTimeout(() => void read(), RETRY_MILLIS);
        }
      }
    };
    void read();
    return () => {
      live = false;
      if (timer !== null) {
        clearTimeout(timer);
      }
    };
  }, [api, projectId, queued]);

  if (!queued) {
    return null;
  }
  const current = job?.jobId === queued.jobId ? job : null;
  return {
    renderArtifactId: current?.tasks.find(
      (task) => task.outputKind === RENDER_KIND && task.state === TaskState.SUCCEEDED,
    )?.outputArtifactId,
    revision: queued.revision,
    destinationDir: queued.destinationDir,
    createdUnixMillis: current?.createdUnixMillis,
    updatedUnixMillis: current?.updatedUnixMillis,
    stages: deliveryStages(current),
    settled: settled(current),
    files: files?.jobId === queued.jobId ? files.value : null,
    failure: failureOf(current),
    interruption: settled(current) ? null : problem,
  };
}
