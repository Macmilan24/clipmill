/**
 * The readings the Library performs on what the daemon returned.
 *
 * None of this touches a socket or a DOM, which is the reason it is a separate
 * module: a project's status is an interpretation, interpretations are where the
 * mistakes are, and they should be cheap to pin down.
 */
import { JobState, TaskState } from '@clipmill/contracts';
import { describe, expect, it } from 'vitest';

import {
  EM_DASH,
  type LibraryProject,
  applyFilter,
  availableFilters,
  completedStages,
  describeActivity,
  describeStatus,
  formatProgress,
  formatRelative,
  formatTimecode,
  formatVideoSpec,
  matchesQuery,
  newestAnalysis,
  publishedArtifact,
  readStatus,
  sortProjects,
} from '../src/library/model.js';
import { NOW, job, progress, project, sourceMap, task } from './support/library.js';

function entry(name: string, status: LibraryProject['status'], createdAgo = 0): LibraryProject {
  return {
    project: project(name, name, createdAgo),
    status,
    job: null,
    source: null,
    sourceMap: null,
    thumbnail: null,
  };
}

describe('reading a project status', () => {
  it('says nothing has been analyzed when no analyze job exists', () => {
    expect(readStatus(null).kind).toBe('none');
  });

  it('reads each job state as the status a user recognises', () => {
    const states: readonly (readonly [JobState, string])[] = [
      [JobState.SUCCEEDED, 'analyzed'],
      [JobState.FAILED, 'failed'],
      [JobState.CANCELLED, 'cancelled'],
      [JobState.CANCEL_REQUESTED, 'cancelled'],
      [JobState.PLANNED, 'queued'],
    ];
    for (const [state, expected] of states) {
      expect(readStatus(job('p', state)).kind).toBe(expected);
    }
  });

  /**
   * The running task is what the row should name. The alternative — naming the
   * job — would leave a nine-stage run saying "analyzing" for twenty minutes.
   */
  it('names the stage a running job is actually working on', () => {
    const running = job('p', JobState.RUNNING, [
      task('evidence.source_map.v1', TaskState.SUCCEEDED),
      task('ranking.set.v1', TaskState.RUNNING, {
        progress: progress('candidates', 11, 18),
      }),
    ]);
    const status = readStatus(running);
    expect(status.kind).toBe('analyzing');
    expect(describeActivity(status)).toBe('Rank candidates · 11 of 18 candidates');
  });

  /** Ingest's eight derivatives are one stage, and each of them says so. */
  it('rolls an ingest derivative up into the ingest stage', () => {
    const running = job('p', JobState.RUNNING, [
      task('media.filmstrip.v1', TaskState.RUNNING, { progress: progress('tiles', 40, 0) }),
    ]);
    expect(describeActivity(readStatus(running))).toBe('Ingest · 40 tiles');
  });

  it('says a job is running even when no task has been leased yet', () => {
    const status = readStatus(
      job('p', JobState.RUNNING, [task('speech.vad.v1', TaskState.PLANNED)]),
    );
    expect(status.kind).toBe('analyzing');
    expect(describeActivity(status)).toBe('Working');
  });

  it('keeps the failure detail rather than replacing it with a word', () => {
    const failed = job('p', JobState.FAILED);
    const status = readStatus({ ...failed, failureDetail: 'ffprobe found no streams' });
    expect(describeActivity(status)).toBe('ffprobe found no streams');
  });

  it('uses one of the reserved colours and never invents a sixth', () => {
    const tones = new Set(
      (['analyzed', 'analyzing', 'queued', 'failed', 'cancelled', 'none'] as const).map(
        (kind) => describeStatus({ kind } as LibraryProject['status']).tone,
      ),
    );
    expect([...tones].toSorted()).toEqual(['danger', 'neutral', 'progress', 'success']);
  });
});

describe('finding the analysis a project is represented by', () => {
  it('takes the newest analyze job and ignores every other kind', () => {
    const older = { ...job('p', JobState.FAILED), jobId: 'old', createdUnixMillis: 1 };
    const newer = { ...job('p', JobState.RUNNING), jobId: 'new', createdUnixMillis: 2 };
    const other = { ...job('p', JobState.SUCCEEDED), jobId: 'render', kind: 'render-clip' };
    expect(newestAnalysis([older, other, newer])?.jobId).toBe('new');
    expect(newestAnalysis([other])).toBeNull();
  });

  /**
   * The reason a task reports its output kind: without it, finding the filmstrip
   * meant reading the analysis manifest, then the ingest manifest inside it, to
   * arrive at an address that was on the job the whole time.
   */
  it('finds a published artifact by the kind that produced it', () => {
    const finished = job('p', JobState.SUCCEEDED, [
      task('media.proxy.v1', TaskState.SUCCEEDED),
      task('media.filmstrip.v1', TaskState.SUCCEEDED),
    ]);
    expect(publishedArtifact(finished, 'media.filmstrip.v1')).toBe('sha256:art-media.filmstrip.v1');
    expect(publishedArtifact(finished, 'ranking.set.v1')).toBeNull();
  });

  it('does not offer an address for a task that has not published one', () => {
    const running = job('p', JobState.RUNNING, [task('media.filmstrip.v1', TaskState.RUNNING)]);
    expect(publishedArtifact(running, 'media.filmstrip.v1')).toBeNull();
  });

  it('counts finished stages against the stages the job declared', () => {
    const running = job('p', JobState.RUNNING, [
      task('evidence.source_map.v1', TaskState.SUCCEEDED),
      task('media.ingest_manifest.v1', TaskState.RUNNING),
    ]);
    expect(completedStages(running)).toEqual({ done: 1, total: 2 });
    expect(completedStages(null)).toEqual({ done: 0, total: 0 });
  });
});

describe('formatting what was measured', () => {
  it('writes a timecode with hours only when there are hours', () => {
    expect(formatTimecode(6127 * 90_000)).toBe('1:42:07');
    expect(formatTimecode(271 * 90_000)).toBe('4:31');
    expect(formatTimecode(undefined)).toBe(EM_DASH);
    expect(formatTimecode(-1)).toBe(EM_DASH);
  });

  /** A percentage would need a denominator the stage explicitly did not give. */
  it('counts progress and never converts it to a percentage', () => {
    expect(formatProgress(progress('candidates', 11, 18))).toBe('11 of 18 candidates');
    expect(formatProgress(progress('frames', 240, 0))).toBe('240 frames');
    expect(formatProgress(null)).toBeNull();
    expect(formatProgress(progress('', 3, 4))).toBeNull();
  });

  it('reads resolution and frame rate from the probe', () => {
    expect(formatVideoSpec(sourceMap())).toBe('1920×1080 · 29.97 fps');
  });

  it('says a recording has no video rather than inventing a resolution', () => {
    const audioOnly = sourceMap({
      streams: [{ index: 0, kind: 'audio', codec: 'aac', timebase: { num: 1, den: 90_000 } }],
    } as never);
    expect(formatVideoSpec(audioOnly)).toBe('audio only');
    expect(formatVideoSpec(null)).toBe(EM_DASH);
  });

  it('writes elapsed time in the unit that fits', () => {
    expect(formatRelative(NOW - 3 * 60 * 60_000, NOW)).toMatch(/3/);
    expect(formatRelative(NOW - 5_000, NOW)).toMatch(/5/);
  });
});

describe('narrowing the list', () => {
  const entries = [
    entry('alpha', { kind: 'analyzed' }, 3_000),
    entry('beta', { kind: 'analyzing', stage: null, progress: null }, 2_000),
    entry('gamma', { kind: 'analyzed' }, 1_000),
  ];

  /**
   * A chip for a state nothing is in is a control that does nothing. The
   * design's fixed chips assume a category nobody records; counting what is
   * there means the row can never produce an empty result.
   */
  it('offers only the filters that would match something', () => {
    expect(availableFilters(entries)).toEqual([
      { filter: 'all', label: 'All', count: 3 },
      { filter: 'analyzed', label: 'Analyzed', count: 2 },
      { filter: 'analyzing', label: 'Analyzing', count: 1 },
    ]);
  });

  it('filters by status and passes everything through for all', () => {
    expect(applyFilter(entries, 'analyzed')).toHaveLength(2);
    expect(applyFilter(entries, 'all')).toHaveLength(3);
  });

  it('matches titles case-insensitively and matches everything on an empty query', () => {
    expect(matchesQuery(entries[0]!, 'ALP')).toBe(true);
    expect(matchesQuery(entries[0]!, '  ')).toBe(true);
    expect(matchesQuery(entries[0]!, 'zeta')).toBe(false);
  });

  it('sorts newest first by default and alphabetically by name', () => {
    expect(sortProjects(entries, 'created').map((item) => item.project.name)).toEqual([
      'gamma',
      'beta',
      'alpha',
    ]);
    expect(sortProjects(entries, 'name').map((item) => item.project.name)).toEqual([
      'alpha',
      'beta',
      'gamma',
    ]);
  });
});
