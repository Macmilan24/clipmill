/**
 * How long analysis takes on this machine, learned from its own runs.
 *
 * Every finished run leaves how many seconds each stage took per second of
 * recording, and the whole run too. The median of the last eight is the
 * estimate: one unusually slow night does not set the next. A stage this
 * machine has never finished has no estimate rather than a guess, and the
 * route matters — a run with the local editorial model is a different run
 * from one without — so the whole-run pace is kept per route.
 */
import { JobState, TaskState } from '@clipmill/contracts';

import type { Job } from '../daemon/client.js';
import { ANALYSIS_STAGES } from '../pipeline/stages.js';
import type { StageRow } from './model.js';

const KEY = 'clipmill.analysisPace';
const KEEP = 8;

interface Pace {
  /** Seconds per second of recording, by stage kind and by `run:<route>`. */
  readonly rates: Readonly<Record<string, readonly number[]>>;
  /** Runs already learned from, so a screen shown twice counts once. */
  readonly learned: readonly string[];
}

function read(): Pace {
  try {
    const stored = JSON.parse(localStorage.getItem(KEY) ?? '{}') as Partial<Pace>;
    return {
      rates: stored.rates && typeof stored.rates === 'object' ? stored.rates : {},
      learned: Array.isArray(stored.learned) ? stored.learned : [],
    };
  } catch {
    return { rates: {}, learned: [] };
  }
}

function write(pace: Pace): void {
  try {
    localStorage.setItem(KEY, JSON.stringify(pace));
  } catch {
    // An unavailable store only removes the estimate.
  }
}

/** Remember how long a finished run and each of its stages took. */
export function learnFrom(job: Job, mediaSeconds: number, route: string): void {
  if (job.state !== JobState.SUCCEEDED || mediaSeconds <= 0) return;
  const pace = read();
  if (pace.learned.includes(job.jobId)) return;
  const rates: Record<string, readonly number[]> = { ...pace.rates };
  const add = (key: string, seconds: number) => {
    if (seconds <= 0) return;
    rates[key] = [...(rates[key] ?? []), seconds / mediaSeconds].slice(-KEEP);
  };
  for (const stage of ANALYSIS_STAGES) {
    const kinds = new Set([stage.kind, ...(stage.covers ?? [])]);
    const tasks = job.tasks.filter((task) => kinds.has(task.outputKind));
    if (tasks.length === 0 || tasks.some((task) => task.state !== TaskState.SUCCEEDED)) continue;
    const started = Math.min(...tasks.map((task) => task.startedUnixMillis ?? 0));
    const finished = Math.max(...tasks.map((task) => task.finishedUnixMillis ?? 0));
    // A stage served from the cache never ran; it says nothing about pace.
    if (started <= 0 || finished <= started) continue;
    add(stage.kind, (finished - started) / 1000);
  }
  add(`run:${route}`, (job.updatedUnixMillis - job.createdUnixMillis) / 1000);
  write({ rates, learned: [...pace.learned, job.jobId].slice(-4 * KEEP) });
}

function median(values: readonly number[] | undefined): number | null {
  if (!values || values.length === 0) return null;
  const sorted = values.toSorted((a, b) => a - b);
  return sorted[Math.floor(sorted.length / 2)] ?? null;
}

/** How long a stage usually takes here for this much recording, in seconds. */
export function stageEstimate(kind: string, mediaSeconds: number): number | null {
  const rate = median(read().rates[kind]);
  return rate === null || mediaSeconds <= 0 ? null : rate * mediaSeconds;
}

/** How long a whole run usually takes here for this much recording, in seconds. */
export function runEstimate(mediaSeconds: number, route: string): number | null {
  const rate = median(read().rates[`run:${route}`]);
  return rate === null || mediaSeconds <= 0 ? null : rate * mediaSeconds;
}

/**
 * What is left of a run: each stage not yet done at its usual pace, a running
 * one less what it has measured of itself. Null when any stage still to come
 * has never been timed here — a sum with a hole in it is not an estimate.
 */
export function remainingEstimate(rows: readonly StageRow[], mediaSeconds: number): number | null {
  let total = 0;
  for (const row of rows) {
    if (row.state === 'done' || row.state === 'skipped') continue;
    const usual = stageEstimate(row.stage.kind, mediaSeconds);
    if (usual === null) return null;
    const progress = row.progress;
    const share =
      row.state === 'running' && progress && progress.total > 0
        ? Math.min(1, progress.done / progress.total)
        : 0;
    total += usual * (1 - share);
  }
  return total;
}

/** `about 12 min`, `about 1 h 5 min`, or `under a minute`. */
export function formatEstimate(seconds: number): string {
  const minutes = Math.round(seconds / 60);
  if (minutes < 1) return 'under a minute';
  if (minutes < 60) return `about ${minutes} min`;
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  return rest === 0 ? `about ${hours} h` : `about ${hours} h ${rest} min`;
}
