/**
 * The pipeline, as ten rows.
 *
 * A job's tasks are not the rows. Ingest is eight tasks and one row; the fan-in
 * that publishes the manifest is a task and no row at all; and a stage the
 * planner decided not to run for this recording has no task, which is a state of
 * its own rather than an absence to be rendered as "waiting" forever.
 *
 * All of that is a reading, so it lives here where it can be tested against a
 * job record and nothing else.
 */
import { TaskState } from '@clipmill/contracts';

import type { Job, Progress, Task } from '../daemon/client.js';
import { ANALYSIS_STAGES, type AnalysisStage } from '../pipeline/stages.js';
import { formatClock, formatProgress } from '../library/model.js';

export type StageState =
  | 'done'
  | 'running'
  | 'waiting'
  | 'failed'
  | 'cancelled'
  /** No task for it: the planner decided this recording does not need it. */
  | 'skipped';

export interface StageRow {
  readonly stage: AnalysisStage;
  readonly state: StageState;
  /** What the stage measured, when it is measuring something. */
  readonly progress: Progress | null;
  /**
   * Set only where a stage rolls several tasks up. Ingest's eight derivatives
   * finish at eight different times, and "3 of 8 derivatives" is the only honest
   * thing to say while they are in flight.
   */
  readonly rollup: { readonly done: number; readonly total: number } | null;
  /** The address it published, once it has. */
  readonly artifactId: string | null;
}

/**
 * The state of a group of tasks, taken from the worst thing that happened.
 *
 * Order matters and is deliberate: one failure makes the stage failed even if
 * seven siblings succeeded, because a stage that lost a derivative did not
 * finish. Running beats waiting so the row moves as soon as any of its work
 * starts.
 */
function combine(tasks: readonly Task[]): StageState {
  if (tasks.some((task) => task.state === TaskState.FAILED)) {
    return 'failed';
  }
  if (tasks.some((task) => task.state === TaskState.CANCELLED)) {
    return 'cancelled';
  }
  if (tasks.some((task) => task.state === TaskState.RUNNING)) {
    return 'running';
  }
  return tasks.every((task) => task.state === TaskState.SUCCEEDED) ? 'done' : 'waiting';
}

/** Every stage of the pipeline, in order, against what this job actually has. */
export function stageRows(job: Job | null): readonly StageRow[] {
  return ANALYSIS_STAGES.map((stage) => {
    const kinds = new Set([stage.kind, ...(stage.covers ?? [])]);
    const tasks = job?.tasks.filter((task) => kinds.has(task.outputKind)) ?? [];
    if (tasks.length === 0) {
      return { stage, state: 'skipped', progress: null, rollup: null, artifactId: null };
    }
    const running = tasks.find((task) => task.state === TaskState.RUNNING);
    const published = tasks.find(
      (task) => task.outputKind === stage.kind && task.outputArtifactId !== '',
    );
    return {
      stage,
      state: combine(tasks),
      progress: running?.progress ?? null,
      rollup:
        tasks.length > 1
          ? {
              done: tasks.filter((task) => task.state === TaskState.SUCCEEDED).length,
              total: tasks.length,
            }
          : null,
      artifactId: published?.outputArtifactId ?? null,
    };
  });
}

/** The right-hand column of a stage row: what it is doing, or what it did. */
export function describeStageRow(row: StageRow): string {
  if (row.state === 'running') {
    const measured = formatProgress(row.progress);
    if (measured !== null) {
      return measured;
    }
    return row.rollup === null
      ? 'running'
      : `${row.rollup.done} of ${row.rollup.total} derivatives`;
  }
  if (row.state === 'done' && row.rollup !== null) {
    return `${row.rollup.total} derivatives`;
  }
  return row.state === 'skipped' ? 'not needed' : row.state;
}

/**
 * How far the run has come, as stages finished over stages planned.
 *
 * A real fraction with a real denominator, which is why it may be drawn as a
 * bar. It is emphatically not a time estimate: stages differ in cost by two
 * orders of magnitude, so eight of ten finished does not mean eighty percent of
 * the wait is over, and the label beside it counts rather than claiming a
 * percentage.
 */
export function stageCounts(rows: readonly StageRow[]): {
  readonly done: number;
  readonly planned: number;
} {
  const planned = rows.filter((row) => row.state !== 'skipped');
  return { done: planned.filter((row) => row.state === 'done').length, planned: planned.length };
}

/** The stage a viewer would say the run is on. */
export function currentStage(rows: readonly StageRow[]): StageRow | null {
  return (
    rows.find((row) => row.state === 'running') ??
    rows.find((row) => row.state === 'failed') ??
    null
  );
}

/** `12:48`, counting up. Elapsed is measured; remaining would be a guess. */
export function formatElapsed(millis: number): string {
  return formatClock(millis / 1000);
}

/** Middle-truncated, so the file name survives and the volume is still visible. */
export function shortenPath(path: string, budget = 44): string {
  if (path.length <= budget) {
    return path;
  }
  const separator = path.includes('\\') ? '\\' : '/';
  const name = path.slice(path.lastIndexOf(separator) + 1);
  const head = Math.max(0, budget - name.length - 2);
  return head === 0 ? `…${separator}${name}` : `${path.slice(0, head)}…${separator}${name}`;
}
