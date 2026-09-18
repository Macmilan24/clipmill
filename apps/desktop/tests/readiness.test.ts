/**
 * What blocks a run, what only makes it wait, and which tasks the wait is
 * pinned to.
 */
import { JobState, TaskState } from '@clipmill/contracts';
import { describe, expect, it } from 'vitest';

import {
  missingModels,
  missingWorkers,
  submissionBlocker,
  waitingReason,
  waitingReasons,
} from '../src/analysis/readiness.js';
import { job, readiness, stageReadiness, task } from './support/library.js';

describe('what readiness blocks and what it only names', () => {
  it('knows nothing until the report has been read', () => {
    expect(submissionBlocker(null)).toBeNull();
    expect(missingModels(null)).toEqual([]);
    expect(missingWorkers(null)).toEqual([]);
  });

  it('blocks on the decoder before anything else', () => {
    const report = readiness(
      [stageReadiness('speech-asr', { modelPresent: false, missingFiles: ['a', 'b'] })],
      { decoderPresent: false, decoderPath: '/x/ffmpeg' },
    );
    expect(submissionBlocker(report)).toMatch(/decoder is not at \/x\/ffmpeg/);
    expect(submissionBlocker(report)).toMatch(/just setup/);
  });

  it('blocks on a missing model, naming every model that is missing', () => {
    const report = readiness([
      stageReadiness('speech-asr', { modelPresent: false, missingFiles: ['w'] }),
      stageReadiness('speech-vad', { modelPresent: false, missingFiles: ['w'] }),
      stageReadiness('detect-shots', { workerPresent: false }),
    ]);
    expect(submissionBlocker(report)).toBe(
      'Models are not installed (speech-asr-weights, speech-vad-weights); run `tools/fetch-models.sh` to fetch the pinned weights.',
    );
    expect(missingModels(report).map((stage) => stage.stage)).toEqual(['speech-asr', 'speech-vad']);
    // A stage whose model is missing is not also counted as short a worker;
    // one remedy at a time.
    expect(missingWorkers(report).map((stage) => stage.stage)).toEqual(['detect-shots']);
  });

  it('does not block on a worker that has not connected yet', () => {
    const report = readiness([stageReadiness('detect-shots', { workerPresent: false })]);
    expect(submissionBlocker(report)).toBeNull();
    expect(report.ready).toBe(false);
  });
});

describe('the reason a task is waiting', () => {
  const report = readiness([
    stageReadiness('speech.asr', { workerPresent: false }),
    stageReadiness('speech.vad'),
  ]);

  it('is the stage remedy for a planned or admitted task whose stage is not ready', () => {
    expect(waitingReason(task('speech.asr.v1', TaskState.PLANNED), report)).toMatch(/just workers/);
    expect(waitingReason(task('speech.asr.v1', TaskState.ADMITTED), report)).toMatch(
      /just workers/,
    );
  });

  it('is nothing for a task that is running, done, or waiting on a dependency', () => {
    expect(waitingReason(task('speech.asr.v1', TaskState.RUNNING), report)).toBeNull();
    expect(waitingReason(task('speech.asr.v1', TaskState.SUCCEEDED), report)).toBeNull();
    expect(waitingReason(task('speech.vad.v1', TaskState.PLANNED), report)).toBeNull();
    expect(waitingReason(task('index.transcript.v1', TaskState.PLANNED), report)).toBeNull();
    expect(waitingReason(task('speech.asr.v1', TaskState.PLANNED), null)).toBeNull();
  });

  it('keys the reasons by what each task publishes, as the progress rows are', () => {
    const current = job('p1', JobState.RUNNING, [
      task('speech.vad.v1', TaskState.RUNNING),
      task('speech.asr.v1', TaskState.PLANNED),
    ]);
    const reasons = waitingReasons(current, report);
    expect([...reasons.keys()]).toEqual(['speech.asr.v1']);
    expect(waitingReasons(null, report).size).toBe(0);
  });
});
