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
import { fakeApi, job, source, task } from './support/library.js';

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
        revision: 1,
        createdUnixMillis: 1,
        updatedUnixMillis: 1,
      },
      {
        docId: 'edt_elsewhere',
        projectId: OLD,
        sourceId: OTHER_SOURCE,
        candidateId: CANDIDATE,
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
  });
});
