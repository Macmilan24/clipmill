/**
 * Gathering a project, against a daemon that answers from memory.
 *
 * What is worth pinning here is not the happy path but the refusals: the daemon
 * legitimately declines artifacts — collected, unverifiable, belonging to another
 * project — and a Library that lost a whole page because one thumbnail would not
 * open would be broken for the wrong reason.
 */
import { JobState, TaskState } from '@clipmill/contracts';
import { describe, expect, it } from 'vitest';

import { LibraryLoader } from '../src/library/loader.js';
import {
  type FakeWorld,
  emptyWorld,
  fakeApi,
  filmstrip,
  job,
  project,
  source,
  sourceMapDocument,
  task,
} from './support/library.js';

const FILMSTRIP_ID = 'sha256:art-media.filmstrip.v1';

function analyzedWorld(overrides: Partial<FakeWorld> = {}): FakeWorld {
  const finished = job('p1', JobState.SUCCEEDED, [
    task('evidence.source_map.v1', TaskState.SUCCEEDED),
    task('media.filmstrip.v1', TaskState.SUCCEEDED),
  ]);
  return {
    ...emptyWorld(),
    projects: [project('p1', 'Episode 41')],
    jobs: { p1: [finished] },
    sources: { p1: [source('p1')] },
    documents: { 'sha256:map-p1': sourceMapDocument('sha256:map-p1') },
    media: { [FILMSTRIP_ID]: filmstrip(FILMSTRIP_ID, 9) },
    storage: {
      categories: [{ key: 'artifacts', bytes: 1024, items: 3, path: '/data/artifacts' }],
      availableBytes: 4096,
      retentionGraceSeconds: 604_800,
    },
    ...overrides,
  };
}

describe('gathering the library', () => {
  it('assembles a project from its jobs, its source, and its probe', async () => {
    const snapshot = await new LibraryLoader(fakeApi(analyzedWorld())).load();
    const [entry] = snapshot.projects;

    expect(snapshot.projects).toHaveLength(1);
    expect(entry?.status.kind).toBe('analyzed');
    expect(entry?.sourceMap?.container.duration_ticks).toBe(6127 * 90_000);
    expect(entry?.source?.byteSize).toBe(8_400_000_000);
    expect(snapshot.storage?.availableBytes).toBe(4096);
  });

  /**
   * The middle tile, because the first frame of a recording is so often black or
   * a slate — a grid of black squares is technically correct and useless.
   */
  it('takes a thumbnail from the middle of the filmstrip', async () => {
    const snapshot = await new LibraryLoader(fakeApi(analyzedWorld())).load();
    expect(snapshot.projects[0]?.thumbnail).toBe(
      `clipmill-media://localhost/p1/${FILMSTRIP_ID}/strip_00004.jpg`,
    );
  });

  it('has no thumbnail before the run has published a filmstrip', async () => {
    const world = analyzedWorld({
      jobs: { p1: [job('p1', JobState.RUNNING, [task('media.filmstrip.v1', TaskState.RUNNING)])] },
    });
    const snapshot = await new LibraryLoader(fakeApi(world)).load();
    expect(snapshot.projects[0]?.thumbnail).toBeNull();
    expect(snapshot.projects[0]?.status.kind).toBe('analyzing');
  });

  it('keeps the project when the daemon refuses one of its artifacts', async () => {
    const world = analyzedWorld({ documents: {}, media: {} });
    const snapshot = await new LibraryLoader(fakeApi(world)).load();

    expect(snapshot.projects).toHaveLength(1);
    expect(snapshot.projects[0]?.sourceMap).toBeNull();
    expect(snapshot.projects[0]?.thumbnail).toBeNull();
    // The status came from the job, which was readable, so it is still right.
    expect(snapshot.projects[0]?.status.kind).toBe('analyzed');
  });

  /**
   * The daemon echoes the kind it served. Parsing whatever arrived as a source
   * map would turn a daemon-side mix-up into a card full of wrong numbers rather
   * than an absent one.
   */
  it('refuses a document that is not the kind it asked for', async () => {
    const world = analyzedWorld({
      documents: {
        'sha256:map-p1': { artifactId: 'sha256:map-p1', kind: 'ranking.set.v1', json: '{}' },
      },
    });
    const snapshot = await new LibraryLoader(fakeApi(world)).load();
    expect(snapshot.projects[0]?.sourceMap).toBeNull();
  });

  it('reports no storage rather than failing when the daemon will not measure', async () => {
    const snapshot = await new LibraryLoader(fakeApi(analyzedWorld({ storage: null }))).load();
    expect(snapshot.storage).toBeNull();
    expect(snapshot.projects).toHaveLength(1);
  });

  it('lists a project that has never been analyzed', async () => {
    const world = { ...emptyWorld(), projects: [project('p2', 'Untouched')] };
    const snapshot = await new LibraryLoader(fakeApi(world)).load();
    expect(snapshot.projects[0]?.status.kind).toBe('none');
    expect(snapshot.projects[0]?.job).toBeNull();
  });
});
