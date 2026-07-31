/**
 * Turning a job's tasks into ten rows.
 *
 * The mapping is not one-to-one and that is the whole reason it is worth
 * testing: ingest is eight tasks and one row, the fan-in is a task and no row,
 * and a stage the planner left out is a state rather than an absence.
 */
import { JobState, TaskState } from '@clipmill/contracts';
import { describe, expect, it } from 'vitest';

import {
  currentStage,
  describeStageRow,
  formatElapsed,
  shortenPath,
  stageCounts,
  stageRows,
} from '../src/analysis/model.js';
import { ANALYSIS_STAGES } from '../src/pipeline/stages.js';
import { job, progress, task } from './support/library.js';

const INGEST_DERIVATIVES = [
  'media.proxy.v1',
  'media.audio_16k.v1',
  'media.audio_48k.v1',
  'media.loudness_envelope.v1',
  'media.reference_index.v1',
  'media.filmstrip.v1',
  'media.audio_peaks.v1',
  'media.frames.v1',
];

describe('reading a job as pipeline rows', () => {
  it('always renders every stage, in the order the daemon runs them', () => {
    const rows = stageRows(null);
    expect(rows.map((row) => row.stage.kind)).toEqual(ANALYSIS_STAGES.map((stage) => stage.kind));
  });

  /**
   * A recording with no video gets no shot detection, and the planner simply
   * omits the task. Rendering that as "waiting" would leave a row that never
   * moves and a progress count that never completes.
   */
  it('calls a stage with no task skipped rather than waiting forever', () => {
    const rows = stageRows(job('p', JobState.RUNNING, [task('speech.vad.v1', TaskState.RUNNING)]));
    const shots = rows.find((row) => row.stage.kind === 'evidence.shots.v1');
    expect(shots?.state).toBe('skipped');
    expect(describeStageRow(shots!)).toBe('not needed');
  });

  it('never gives the fan-in manifest a row of its own', () => {
    const rows = stageRows(
      job('p', JobState.SUCCEEDED, [task('analysis.manifest.v1', TaskState.SUCCEEDED)]),
    );
    expect(rows.some((row) => row.stage.kind === 'analysis.manifest.v1')).toBe(false);
    // And it is not silently attributed to some other stage either.
    expect(rows.every((row) => row.state === 'skipped')).toBe(true);
  });

  it('rolls the eight ingest derivatives into one row that counts them', () => {
    const tasks = INGEST_DERIVATIVES.map((kind, index) =>
      task(kind, index < 3 ? TaskState.SUCCEEDED : TaskState.RUNNING),
    );
    const ingest = stageRows(job('p', JobState.RUNNING, tasks)).find(
      (row) => row.stage.kind === 'media.ingest_manifest.v1',
    );

    expect(ingest?.state).toBe('running');
    expect(ingest?.rollup).toEqual({ done: 3, total: 8 });
    expect(describeStageRow(ingest!)).toBe('3 of 8 derivatives');
  });

  /** One lost derivative means the stage did not finish, whatever the rest did. */
  it('reports a group as failed when any member failed', () => {
    const tasks = [
      task('media.proxy.v1', TaskState.SUCCEEDED),
      task('media.frames.v1', TaskState.FAILED),
    ];
    const ingest = stageRows(job('p', JobState.FAILED, tasks)).find(
      (row) => row.stage.kind === 'media.ingest_manifest.v1',
    );
    expect(ingest?.state).toBe('failed');
  });

  it('prefers the stage measurement over the derivative count', () => {
    const rows = stageRows(
      job('p', JobState.RUNNING, [
        task('ranking.set.v1', TaskState.RUNNING, { progress: progress('candidates', 4, 18) }),
      ]),
    );
    expect(describeStageRow(rows.find((row) => row.stage.kind === 'ranking.set.v1')!)).toBe(
      '4 of 18 candidates',
    );
  });

  it('offers the address a finished stage published', () => {
    const rows = stageRows(
      job('p', JobState.SUCCEEDED, [task('speech.transcript.v1', TaskState.SUCCEEDED)]),
    );
    const transcript = rows.find((row) => row.stage.kind === 'speech.transcript.v1');
    expect(transcript?.artifactId).toBe('sha256:art-speech.transcript.v1');
  });
});

describe('how far the run has come', () => {
  /**
   * Counted against the stages actually planned. Including skipped ones would
   * mean an audio-only recording could never reach the end of its own pipeline.
   */
  it('counts finished stages against planned stages, not against all ten', () => {
    const rows = stageRows(
      job('p', JobState.RUNNING, [
        task('evidence.source_map.v1', TaskState.SUCCEEDED),
        task('speech.vad.v1', TaskState.RUNNING),
      ]),
    );
    expect(stageCounts(rows)).toEqual({ done: 1, planned: 2 });
  });

  it('names the running stage, and the failed one when the run stopped', () => {
    const running = stageRows(
      job('p', JobState.RUNNING, [task('index.transcript.v1', TaskState.RUNNING)]),
    );
    expect(currentStage(running)?.stage.label).toBe('Index transcript');

    const failed = stageRows(job('p', JobState.FAILED, [task('speech.asr.v1', TaskState.FAILED)]));
    expect(currentStage(failed)?.stage.label).toBe('Recognise speech');

    expect(currentStage(stageRows(null))).toBeNull();
  });

  it('counts elapsed time up, and never counts below zero', () => {
    expect(formatElapsed(768_000)).toBe('12:48');
    expect(formatElapsed(3_600_000 + 61_000)).toBe('1:01:01');
    expect(formatElapsed(-5000)).toBe('0:00');
  });
});

describe('showing a path without losing the file name', () => {
  it('truncates in the middle so the name survives', () => {
    const path = '/Volumes/Creator/Podcast/Episode 41/pricing-mistakes-episode-41.mp4';
    const short = shortenPath(path);
    expect(short.endsWith('pricing-mistakes-episode-41.mp4')).toBe(true);
    expect(short.startsWith('/Volumes')).toBe(true);
    expect(short.length).toBeLessThan(path.length);
  });

  it('leaves a path that already fits alone', () => {
    expect(shortenPath('/tmp/a.mp4')).toBe('/tmp/a.mp4');
  });

  it('keeps the separator the path was written with', () => {
    const windows = String.raw`C:\Users\sami\Videos\Long recordings\episode-forty-one.mp4`;
    expect(shortenPath(windows)).toContain('\\episode-forty-one.mp4');
  });
});
