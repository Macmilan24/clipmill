/**
 * Which analysis the board reads, against a daemon that answers from memory.
 *
 * A project can hold two recordings, each analyzed. The loader used to take
 * the project's newest job that published a ranking and then discover, by the
 * ranking's fingerprint, that it was the other recording's — and report the
 * selected one as not analyzed. Now a job says which source it ran over.
 */
import { JobState, TaskState } from '@clipmill/contracts';
import { describe, expect, it } from 'vitest';

import { ResultsLoader } from '../src/results/loader.js';
import { CANDIDATE, OLD, OLD_JOB, OLD_SOURCE, twoProjects } from './support/clips.js';
import { fakeApi, job, source, sourceMap, sourceMapDocument, task } from './support/library.js';

const OTHER_SOURCE = 'src_other';
const OTHER_JOB = 'job-other';
const OTHER_FINGERPRINT = `sha256:${'cc'.repeat(32)}`;

/**
 * The older project with a second recording, analyzed more recently.
 *
 * Its ranking names its own fingerprint, so the fingerprint check cannot
 * mistake it for the first recording's — which is exactly what made the old
 * loader report "not analyzed" for a recording that was.
 */
function twoRecordings() {
  const world = twoProjects();
  const later = job(OLD, JobState.SUCCEEDED, [
    task('discovery.candidates.v1', TaskState.SUCCEEDED, {
      outputArtifactId: 'sha256:candidates-other',
    }),
    task('ranking.set.v1', TaskState.SUCCEEDED, { outputArtifactId: 'sha256:ranking-other' }),
  ]);
  const ranking = JSON.parse(world.documents[`sha256:ranking-${OLD}`]!.json) as {
    source_fingerprint: string;
  };
  return {
    ...world,
    sources: {
      ...world.sources,
      [OLD]: [
        source(OLD, { sourceId: OTHER_SOURCE, sourceFingerprint: OTHER_FINGERPRINT }),
        ...world.sources[OLD]!,
      ],
    },
    jobs: {
      ...world.jobs,
      // Newest first, as the daemon lists them.
      [OLD]: [
        { ...later, jobId: OTHER_JOB, sourceId: OTHER_SOURCE, createdUnixMillis: 9_000_000 },
        ...world.jobs[OLD]!,
      ],
    },
    documents: {
      ...world.documents,
      'sha256:ranking-other': {
        artifactId: 'sha256:ranking-other',
        kind: 'ranking.set.v1',
        json: JSON.stringify({ ...ranking, source_fingerprint: OTHER_FINGERPRINT }),
      },
      'sha256:candidates-other': world.documents[`sha256:candidates-${OLD}`]!,
    },
  };
}

describe('which analysis the board reads', () => {
  it('is the newest run over the selected recording, not the newest run of the project', async () => {
    const loader = new ResultsLoader(fakeApi(twoRecordings()));
    const snapshot = await loader.load(OLD, OLD_SOURCE);
    expect(snapshot.problem).toBeNull();
    expect(snapshot.run?.jobId).toBe(OLD_JOB);
    expect(snapshot.rows.map((row) => row.candidateId)).toContain(CANDIDATE);
  });

  it('is the run the route named, even when a newer one over the same recording exists', async () => {
    const world = twoRecordings();
    const rerun = {
      ...world.jobs[OLD]![1]!,
      jobId: 'job-rerun',
      createdUnixMillis: 9_500_000,
    };
    world.jobs[OLD] = [rerun, ...world.jobs[OLD]!];
    const loader = new ResultsLoader(fakeApi(world));
    expect((await loader.load(OLD, OLD_SOURCE, OLD_JOB)).run?.jobId).toBe(OLD_JOB);
    expect((await loader.load(OLD, OLD_SOURCE)).run?.jobId).toBe('job-rerun');
  });

  it('reports a run the route named that no longer exists as not analyzed, not as another run', async () => {
    const loader = new ResultsLoader(fakeApi(twoRecordings()));
    const snapshot = await loader.load(OLD, OLD_SOURCE, 'job-gone');
    expect(snapshot.problem).toEqual({ kind: 'not-analyzed' });
    expect(snapshot.source?.sourceId).toBe(OLD_SOURCE);
  });

  it('marks the rows that have an edit, and only for this recording', async () => {
    const world = twoRecordings();
    world.editDocs = [
      {
        docId: 'edt_here',
        projectId: OLD,
        sourceId: OLD_SOURCE,
        candidateId: CANDIDATE,
        jobId: OLD_JOB,
        revision: 1,
        createdUnixMillis: 1,
        updatedUnixMillis: 1,
      },
      {
        docId: 'edt_elsewhere',
        projectId: OLD,
        sourceId: OTHER_SOURCE,
        candidateId: CANDIDATE,
        jobId: OTHER_JOB,
        revision: 1,
        createdUnixMillis: 2,
        updatedUnixMillis: 2,
      },
    ];
    const snapshot = await new ResultsLoader(fakeApi(world)).load(OLD, OLD_SOURCE);
    const row = snapshot.rows.find((candidate) => candidate.candidateId === CANDIDATE);
    // The other recording's document is for a candidate with the same id in
    // a different run; it is not this clip's edit.
    expect(row?.docId).toBe('edt_here');
    expect(row?.docJobId).toBe(OLD_JOB);
  });
});

describe('manual span source bounds', () => {
  it.each([true, false])(
    'accepts duration only from this source fingerprint (matching=%s)',
    async (matching) => {
      const world = twoProjects();
      const chosen = world.sources[OLD]![0]!;
      const map = sourceMap({
        source_fingerprint: matching ? chosen.sourceFingerprint : OTHER_FINGERPRINT,
      });
      const withMap = {
        ...world,
        documents: {
          ...world.documents,
          [chosen.sourceMapArtifactId]: sourceMapDocument(chosen.sourceMapArtifactId, map),
        },
      };
      const snapshot = await new ResultsLoader(fakeApi(withMap)).load(OLD, OLD_SOURCE, OLD_JOB);
      expect(snapshot.problem).toBeNull();
      expect(snapshot.sourceDurationTicks).toBe(matching ? map.container.duration_ticks : null);
      expect(snapshot.rows).toHaveLength(2);
    },
  );
  it('keeps clips available if source timing cannot be read and never guesses from their spans', async () => {
    const snapshot = await new ResultsLoader(fakeApi(twoProjects())).load(OLD, OLD_SOURCE, OLD_JOB);
    expect(snapshot.sourceDurationTicks).toBeNull();
    expect(snapshot.rows).toHaveLength(2);
    expect(snapshot.problem).toBeNull();
  });
});

describe('filmstrip identity', () => {
  it.each([true, false])(
    'uses source timing only from the same recording (matching=%s)',
    async (matching) => {
      const world = twoProjects();
      const chosen = world.sources[OLD]![0]!;
      const stripId = 'sha256:filmstrip';
      const analysis = world.jobs[OLD]![0]!;
      const snapshot = await new ResultsLoader(
        fakeApi({
          ...world,
          jobs: {
            ...world.jobs,
            [OLD]: [
              {
                ...analysis,
                tasks: [
                  ...analysis.tasks,
                  task('media.filmstrip.v1', TaskState.SUCCEEDED, { outputArtifactId: stripId }),
                ],
              },
            ],
          },
          documents: {
            ...world.documents,
            [stripId]: {
              artifactId: stripId,
              kind: 'media.filmstrip.v1',
              json: JSON.stringify({
                schema_version: 'clipmill.media.filmstrip.v1',
                source_fingerprint: matching ? chosen.sourceFingerprint : OTHER_FINGERPRINT,
                tiles: [{ file: 'tile_00124.jpg', t_ticks: 600 * 90_000 }],
              }),
            },
          },
        }),
      ).load(OLD, OLD_SOURCE, OLD_JOB);
      expect(snapshot.filmstrip).toEqual(
        matching
          ? {
              artifactId: stripId,
              tiles: [{ file: 'tile_00124.jpg', tTicks: 600 * 90_000 }],
            }
          : null,
      );
      expect(snapshot.problem).toBeNull();
    },
  );
});
