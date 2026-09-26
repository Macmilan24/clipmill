import { JobState, TaskState } from '@clipmill/contracts';
import { beforeEach, describe, expect, it } from 'vitest';

import {
  formatEstimate,
  learnFrom,
  remainingEstimate,
  runEstimate,
  stageEstimate,
} from '../src/analysis/estimates.js';
import { stageRows } from '../src/analysis/model.js';
import type { Job, Task } from '../src/daemon/client.js';

const MINUTE = 60_000;
const task = (outputKind: string, from: number, to: number, state = TaskState.SUCCEEDED) =>
  ({
    taskId: `tsk_${outputKind}`,
    kind: outputKind,
    outputKind,
    state,
    attempt: 1,
    maxAttempts: 3,
    waitReason: '',
    outputArtifactId: '',
    startedUnixMillis: from,
    finishedUnixMillis: to,
  }) as Task;
const job = (id: string, tasks: readonly Task[], minutes: number, state = JobState.SUCCEEDED) =>
  ({
    jobId: id,
    projectId: 'p',
    kind: 'analyze-source',
    state,
    createdUnixMillis: 0,
    updatedUnixMillis: minutes * MINUTE,
    tasks,
    outputArtifactIds: [],
    failureClass: 0,
    failureDetail: '',
  }) as unknown as Job;

beforeEach(() => localStorage.clear());

describe('learning this machine’s pace', () => {
  it('estimates a stage and a run from runs it finished, per minute of recording', () => {
    // An hour of recording: transcription took 12 minutes, the run 30.
    learnFrom(job('a', [task('speech.asr.v1', 1 * MINUTE, 13 * MINUTE)], 30), 3_600, 'local');
    expect(stageEstimate('speech.asr.v1', 1_800)).toBeCloseTo(360);
    expect(runEstimate(1_800, 'local')).toBeCloseTo(900);
    expect(runEstimate(1_800, 'cloud')).toBeNull();
    // The same run shown twice counts once.
    learnFrom(job('a', [task('speech.asr.v1', 0, 60 * MINUTE)], 90), 3_600, 'local');
    expect(stageEstimate('speech.asr.v1', 3_600)).toBeCloseTo(720);
  });

  it('learns nothing from a stage served from the cache, or a failed run', () => {
    learnFrom(job('b', [task('speech.asr.v1', 0, 0)], 1), 3_600, 'local');
    expect(stageEstimate('speech.asr.v1', 3_600)).toBeNull();
    learnFrom(job('c', [], 5, JobState.FAILED), 3_600, 'local');
    expect(runEstimate(3_600, 'local')).toBeCloseTo(60);
  });

  it('says what is left only when every stage still to come has been timed here', () => {
    learnFrom(job('d', [task('speech.asr.v1', MINUTE, 13 * MINUTE)], 20), 3_600, 'local');
    const live = job(
      'e',
      [
        task('speech.asr.v1', 0, 0, TaskState.RUNNING),
        task('speech.vad.v1', 0, 0, TaskState.SUCCEEDED),
      ],
      1,
      JobState.RUNNING,
    );
    const rows = stageRows(live);
    expect(remainingEstimate(rows, 3_600)).toBeCloseTo(720);
    const untimed = job(
      'f',
      [task('speech.transcript.v1', 0, 0, TaskState.PLANNED)],
      1,
      JobState.RUNNING,
    );
    expect(remainingEstimate(stageRows(untimed), 3_600)).toBeNull();
  });

  it('reads as a person would say it', () => {
    expect(formatEstimate(20)).toBe('under a minute');
    expect(formatEstimate(12 * 60)).toBe('about 12 min');
    expect(formatEstimate(65 * 60)).toBe('about 1 h 5 min');
  });
});
