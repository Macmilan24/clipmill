/**
 * Words for the model library: what a model is, whether this device can run
 * it, and why a job runs the model it runs. Pure, so each sentence is tested
 * where it is written rather than found by reading a screen.
 */
import { formatBytes } from '../deviceProfile.js';
import type { LibraryModel, ModelDownload, ModelJob, ModelLibrary } from '../daemon/models.js';

export const BUSY_STATES = ['queued', 'downloading', 'verifying'] as const;

export function isDownloading(model: LibraryModel): boolean {
  return (
    model.download !== undefined &&
    (BUSY_STATES as readonly string[]).includes(model.download.state)
  );
}

/** Where a model runs, in words rather than a backend identifier. */
export function runsOn(model: LibraryModel): string {
  switch (model.backend) {
    case 'mlx':
      return 'Apple silicon GPU';
    case 'cpu':
    case 'onnx-cpu':
      return 'CPU';
    default:
      return model.backend;
  }
}

/** One line of facts: size, memory, where it runs and its licence. */
export function modelFacts(model: LibraryModel): string {
  return [
    `${formatBytes(model.downloadBytes)} download`,
    `needs about ${formatBytes(model.memoryBytes)} of memory`,
    runsOn(model),
    model.licenseSpdx,
  ]
    .filter((part) => part.length > 0)
    .join(' · ');
}

export interface FitWarning {
  readonly tone: 'warning' | 'danger';
  readonly text: string;
}

/**
 * How the memory a model needs compares with this device. A warning, never a
 * refusal: the library lists every model on every machine, and a person may
 * download one for a machine they will move to.
 */
export function fitWarning(model: LibraryModel, library: ModelLibrary): FitWarning | null {
  const needs = formatBytes(model.memoryBytes);
  const total = library.memoryTotalBytes;
  const budget = library.memoryBudgetBytes;
  switch (model.memoryFit) {
    case 'tight':
      return {
        tone: 'warning',
        text:
          budget !== undefined
            ? `Needs about ${needs} of memory; analysis can use ${formatBytes(budget)} on this computer. It may run slowly or stop.`
            : `Needs about ${needs} of memory, close to what this computer has. It may run slowly or stop.`,
      };
    case 'too_large':
      return {
        tone: 'danger',
        text:
          total !== undefined
            ? `Needs about ${needs} of memory; this computer has ${formatBytes(total)}. It is unlikely to run here.`
            : `Needs about ${needs} of memory, more than this computer has. It is unlikely to run here.`,
      };
    default:
      return null;
  }
}

/**
 * Said beside another model for a job, before it is chosen, when the worker
 * that runs it is not connected but the current model's is. Only a worker of
 * a model's own family is handed its tasks, so choosing it would leave the job
 * waiting. The model a job already uses has its problem said once, under the
 * job, from readiness.
 */
export function workerWarning(
  model: LibraryModel,
  job: ModelJob,
  current: LibraryModel | undefined,
): string | null {
  if (model.name === job.model || model.worker === '' || model.workerConnected) return null;
  if (current !== undefined && !current.workerConnected) return null;
  const worker = model.workerTitle === '' ? 'worker that runs it' : model.workerTitle;
  return `Runs in the ${worker}, which isn't running, so ${job.title.toLowerCase()} would wait for it. Restart the workers before choosing it.`;
}

/** Why a job runs the model it runs, as a badge. */
export function jobReason(job: ModelJob): string {
  switch (job.selectedBy) {
    case 'chosen':
      return 'Your choice';
    case 'measured':
      return 'Measured here';
    case 'installed_fallback':
      return 'Stand-in';
    case 'unavailable':
      return 'No model';
    default:
      return 'Automatic';
  }
}

/** A sentence for the job header, naming the model the next analysis uses. */
export function jobSentence(job: ModelJob, planned: LibraryModel | undefined): string {
  if (planned === undefined) {
    return 'No model is registered for this job.';
  }
  const installed = planned.installState === 'installed';
  switch (job.selectedBy) {
    case 'chosen':
      return `Analyses use ${planned.title}, which you chose.`;
    case 'installed_fallback':
      return `Analyses use ${planned.title}, because the default is not installed.`;
    case 'measured':
      return `Analyses use ${planned.title}, chosen by measuring this computer: the most accurate that keeps up.`;
    default:
      return installed
        ? `Analyses use ${planned.title}.`
        : `Analyses need ${planned.title}. Download it to run this job.`;
  }
}

/**
 * Where this computer could do better than the model a job plans: a more
 * accurate one it can run and hold in memory. Said, and offered; the choice
 * stays with the person, so nothing is switched or downloaded for them.
 */
export function upgradeNote(job: ModelJob, library: ModelLibrary): string | null {
  if (!job.moreAccurate) return null;
  const better = library.models.find((model) => model.name === job.moreAccurate);
  if (better === undefined) return null;
  const work = job.title.toLowerCase();
  return better.installState === 'installed'
    ? `${better.title} is more accurate and this computer can run it. Choose it below to use it for ${work}.`
    : `${better.title} is more accurate and this computer can run it. Download it below, then choose it for ${work}.`;
}

/** Percent done, or undefined while nothing about the total is known. */
export function percentOf(download: ModelDownload): number | undefined {
  if (download.totalBytes <= 0) return undefined;
  return Math.min(100, Math.floor((download.receivedBytes / download.totalBytes) * 100));
}

/** What a download or check is doing, in a few words. */
export function progressText(download: ModelDownload): string {
  const amount =
    download.totalBytes > 0
      ? `${formatBytes(download.receivedBytes)} of ${formatBytes(download.totalBytes)}`
      : '';
  switch (download.state) {
    case 'queued':
      return 'Waiting to start';
    case 'verifying':
      return download.currentFile ? `Checking ${download.currentFile}` : 'Checking files';
    case 'downloading':
      return [download.currentFile ? `Downloading ${download.currentFile}` : 'Downloading', amount]
        .filter(Boolean)
        .join(' · ');
    case 'cancelled':
      return amount ? `Paused at ${amount}` : 'Paused';
    case 'failed':
      return download.error || 'The last attempt failed.';
    default:
      return '';
  }
}

/** Every model the library would download to complete the recommended set. */
export function recommendedTitles(library: ModelLibrary): string {
  const titles = library.recommendedMissing
    .map((name) => library.models.find((model) => model.name === name)?.title ?? name)
    .filter(Boolean);
  if (titles.length <= 1) return titles.join('');
  return `${titles.slice(0, -1).join(', ')} and ${titles.at(-1)}`;
}
