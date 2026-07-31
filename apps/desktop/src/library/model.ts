/**
 * What the Library knows about a project, derived rather than fetched.
 *
 * The daemon has no notion of a "project status": it has jobs, and jobs have
 * states. A status is a reading of those, and readings belong somewhere they can
 * be tested without a socket, a window, or a render. So everything here is a
 * pure function of what the daemon returned.
 *
 * The design's card carries a score ring, a speaker count, and a clip count.
 * None of those exist yet — a project-level score is not a thing the ranker
 * produces, and diarization is not implemented — so they are absent rather than
 * invented. What is here was measured.
 */
import type { SourceMap } from '@clipmill/contracts';
import { JobState, TaskState } from '@clipmill/contracts';

import type { Job, Progress, Project, Source, Task } from '../daemon/client.js';
import { type AnalysisStage, stageFor } from '../pipeline/stages.js';

/** The job kind that produces an analysis. */
export const ANALYZE_KIND = 'analyze-source';

const TICKS_PER_SECOND = 90_000;

export type ProjectStatus =
  | { readonly kind: 'analyzed' }
  | {
      readonly kind: 'analyzing';
      readonly stage: AnalysisStage | null;
      readonly progress: Progress | null;
    }
  | { readonly kind: 'queued' }
  | { readonly kind: 'failed'; readonly detail: string }
  | { readonly kind: 'cancelled' }
  /** No analyze job has ever been submitted for this project. */
  | { readonly kind: 'none' };

/** One row or card, with everything it renders and nothing it does not. */
export interface LibraryProject {
  readonly project: Project;
  readonly status: ProjectStatus;
  /** The newest analyze job, which is what the status was read from. */
  readonly job: Job | null;
  readonly source: Source | null;
  /** Parsed from the source's probe, when it has been read. */
  readonly sourceMap: SourceMap | null;
  /** A filmstrip tile, when the run got far enough to publish one. */
  readonly thumbnail: string | null;
}

/**
 * The newest analyze job for a project.
 *
 * Newest by creation, not by update: a job that failed and was resubmitted
 * should be represented by the resubmission even while the older one is still
 * being cancelled.
 */
export function newestAnalysis(jobs: readonly Job[]): Job | null {
  let newest: Job | null = null;
  for (const job of jobs) {
    if (job.kind !== ANALYZE_KIND) {
      continue;
    }
    if (newest === null || job.createdUnixMillis > newest.createdUnixMillis) {
      newest = job;
    }
  }
  return newest;
}

/**
 * The task a running job is currently working on.
 *
 * Several can be running at once — ingest fans out into eight — so this takes
 * the first in DAG order, which is the one whose name best describes where the
 * run has got to. A job whose tasks are all waiting has nothing running, and
 * saying so is better than naming a task that is not started.
 */
function runningTask(job: Job): Task | null {
  return job.tasks.find((task) => task.state === TaskState.RUNNING) ?? null;
}

export function readStatus(job: Job | null): ProjectStatus {
  if (job === null) {
    return { kind: 'none' };
  }
  switch (job.state) {
    case JobState.SUCCEEDED:
      return { kind: 'analyzed' };
    case JobState.FAILED:
      return { kind: 'failed', detail: job.failureDetail };
    case JobState.CANCELLED:
    case JobState.CANCEL_REQUESTED:
      return { kind: 'cancelled' };
    case JobState.RUNNING: {
      const task = runningTask(job);
      return {
        kind: 'analyzing',
        stage: task === null ? null : (stageFor(task.outputKind) ?? null),
        progress: task?.progress ?? null,
      };
    }
    default:
      return { kind: 'queued' };
  }
}

/**
 * The artifact a task of this job published for a kind, if it has.
 *
 * This is why a task reports its output kind: the alternative was reading the
 * analysis manifest, then the ingest manifest inside it, to find one address
 * that was on the job all along.
 */
export function publishedArtifact(job: Job | null, outputKind: string): string | null {
  const task = job?.tasks.find(
    (candidate) => candidate.outputKind === outputKind && candidate.outputArtifactId !== '',
  );
  return task?.outputArtifactId ?? null;
}

/** How far a job has got, as completed stages over the stages it declared. */
export function completedStages(job: Job | null): { readonly done: number; readonly total: number } {
  if (job === null) {
    return { done: 0, total: 0 };
  }
  const done = job.tasks.filter((task) => task.state === TaskState.SUCCEEDED).length;
  return { done, total: job.tasks.length };
}

// ---- Formatting. Every one of these has a shape for "not known". ----

export const EM_DASH = '—';

function pad(value: number): string {
  return value.toString().padStart(2, '0');
}

/**
 * `11 of 18 candidates`, or `240 frames` when the stage has no total.
 *
 * Never a percentage. A `total` of zero means the stage knows how far it has
 * come and not how far there is to go, and a figure drawn from that would be
 * inventing the denominator. Nothing is gained by guessing: the count is the
 * honest reading, and it is the one an editor can act on.
 */
export function formatProgress(progress: Progress | null): string | null {
  if (progress === null || progress.unit === '') {
    return null;
  }
  return progress.total > 0
    ? `${progress.done} of ${progress.total} ${progress.unit}`
    : `${progress.done} ${progress.unit}`;
}

/** What a status says on a badge, and in which of the reserved colours. */
export function describeStatus(status: ProjectStatus): {
  readonly label: string;
  readonly tone: 'success' | 'progress' | 'danger' | 'neutral';
} {
  switch (status.kind) {
    case 'analyzed':
      return { label: 'Analyzed', tone: 'success' };
    case 'analyzing':
      return { label: 'Analyzing', tone: 'progress' };
    case 'failed':
      return { label: 'Failed', tone: 'danger' };
    case 'cancelled':
      return { label: 'Cancelled', tone: 'neutral' };
    case 'queued':
      return { label: 'Queued', tone: 'neutral' };
    default:
      return { label: 'Not analyzed', tone: 'neutral' };
  }
}

/** The line under a title: where the run has got to, or why it stopped. */
export function describeActivity(status: ProjectStatus): string | null {
  switch (status.kind) {
    case 'analyzing': {
      const stage = status.stage?.label ?? 'Working';
      const progress = formatProgress(status.progress);
      return progress === null ? stage : `${stage} · ${progress}`;
    }
    case 'failed':
      return status.detail === '' ? 'The run failed' : status.detail;
    case 'queued':
      return 'Waiting for a worker';
    default:
      return null;
  }
}

/**
 * Seconds as `1:42:07`, dropping the hours when there are none.
 *
 * Shared by every clock this shell writes — a source duration, a run's elapsed
 * time — so the two can never drift into different shapes.
 */
export function formatClock(totalSeconds: number): string {
  const total = Math.max(0, Math.floor(totalSeconds));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor(total / 60) % 60;
  return hours > 0
    ? `${hours}:${pad(minutes)}:${pad(total % 60)}`
    : `${minutes}:${pad(total % 60)}`;
}

/** `1:42:07`, or `4:31` for anything under an hour. */
export function formatTimecode(ticks: number | undefined): string {
  if (ticks === undefined || !Number.isFinite(ticks) || ticks < 0) {
    return EM_DASH;
  }
  return formatClock(ticks / TICKS_PER_SECOND);
}

const RELATIVE = new Intl.RelativeTimeFormat(undefined, { numeric: 'auto', style: 'narrow' });
const UNITS: readonly (readonly [Intl.RelativeTimeFormatUnit, number])[] = [
  ['year', 365 * 24 * 60 * 60_000],
  ['month', 30 * 24 * 60 * 60_000],
  ['day', 24 * 60 * 60_000],
  ['hour', 60 * 60_000],
  ['minute', 60_000],
];

/**
 * "3h ago", in the viewer's locale.
 *
 * `Intl` rather than a hand-rolled ladder: this is a solved problem, and the
 * hand-rolled version is the one that says "1 minutes ago".
 */
export function formatRelative(unixMillis: number, now = Date.now()): string {
  const elapsed = now - unixMillis;
  if (!Number.isFinite(elapsed)) {
    return EM_DASH;
  }
  for (const [unit, span] of UNITS) {
    if (Math.abs(elapsed) >= span) {
      return RELATIVE.format(-Math.round(elapsed / span), unit);
    }
  }
  return RELATIVE.format(-Math.round(elapsed / 1000), 'second');
}

/** The first video stream, which is the one a viewer would call "the video". */
function videoStream(map: SourceMap | null): SourceMap['streams'][number] | null {
  return map?.streams.find((stream) => stream.kind === 'video') ?? null;
}

export function formatDuration(map: SourceMap | null): string {
  return formatTimecode(map?.container.duration_ticks);
}

/** `1920×1080 · 29.97 fps`, dropping whichever half the probe did not report. */
export function formatVideoSpec(map: SourceMap | null): string {
  const stream = videoStream(map);
  const video = stream?.video;
  if (video === undefined) {
    return map === null ? EM_DASH : 'audio only';
  }
  const parts = [`${video.display_width}×${video.display_height}`];
  const rate = video.frame_rate;
  if (rate !== undefined && rate.den > 0) {
    const fps = rate.num / rate.den;
    parts.push(`${Number.isInteger(fps) ? fps : fps.toFixed(2)} fps`);
  }
  return parts.join(' · ');
}

export type SortKey = 'created' | 'updated' | 'name';

export const SORTS: readonly { readonly key: SortKey; readonly label: string }[] = [
  { key: 'created', label: 'Recently created' },
  { key: 'updated', label: 'Recently analyzed' },
  { key: 'name', label: 'Name' },
];

/** When a project last saw work, which is its newest job's last transition. */
function lastActivity(entry: LibraryProject): number {
  return entry.job?.updatedUnixMillis ?? entry.project.createdUnixMillis;
}

export function sortProjects(
  entries: readonly LibraryProject[],
  key: SortKey,
): readonly LibraryProject[] {
  const sorted = [...entries];
  switch (key) {
    case 'name':
      sorted.sort((left, right) =>
        left.project.name.localeCompare(right.project.name, undefined, { sensitivity: 'base' }),
      );
      break;
    case 'updated':
      sorted.sort((left, right) => lastActivity(right) - lastActivity(left));
      break;
    default:
      sorted.sort((left, right) => right.project.createdUnixMillis - left.project.createdUnixMillis);
  }
  return sorted;
}

/**
 * Title search only, and the field says so.
 *
 * The design's placeholder offers transcripts and speakers. Neither is
 * searchable from here: a transcript search is the Discovery screen's job and
 * needs an index this screen does not read, and speakers are not detected at
 * all. Offering it and quietly matching titles would be worse than not offering
 * it.
 */
export function matchesQuery(entry: LibraryProject, query: string): boolean {
  const trimmed = query.trim().toLowerCase();
  return trimmed === '' || entry.project.name.toLowerCase().includes(trimmed);
}

export type StatusFilter = ProjectStatus['kind'] | 'all';

export const FILTER_LABELS: Readonly<Record<ProjectStatus['kind'], string>> = {
  analyzed: 'Analyzed',
  analyzing: 'Analyzing',
  queued: 'Queued',
  failed: 'Failed',
  cancelled: 'Cancelled',
  none: 'Not analyzed',
};

/**
 * The filters worth showing, which are the ones that would match something.
 *
 * A chip for a state no project is in is a control that does nothing, and the
 * design's fixed chip row — Podcasts, Tutorials, Gaming — assumes a category
 * nobody records. Counting what is actually there keeps the row honest and keeps
 * it from ever showing an empty result.
 */
export function availableFilters(
  entries: readonly LibraryProject[],
): readonly { readonly filter: StatusFilter; readonly label: string; readonly count: number }[] {
  const counts = new Map<ProjectStatus['kind'], number>();
  for (const entry of entries) {
    counts.set(entry.status.kind, (counts.get(entry.status.kind) ?? 0) + 1);
  }
  const present = (Object.keys(FILTER_LABELS) as ProjectStatus['kind'][])
    .filter((kind) => (counts.get(kind) ?? 0) > 0)
    .map((kind) => ({
      filter: kind satisfies StatusFilter,
      label: FILTER_LABELS[kind],
      count: counts.get(kind) ?? 0,
    }));
  return [{ filter: 'all', label: 'All', count: entries.length }, ...present];
}

export function applyFilter(
  entries: readonly LibraryProject[],
  filter: StatusFilter,
): readonly LibraryProject[] {
  return filter === 'all' ? entries : entries.filter((entry) => entry.status.kind === filter);
}
