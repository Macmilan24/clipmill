/**
 * The list a clip-less Editor or Export offers, as a join.
 *
 * Every entry must be openable — project, document, source — and named by
 * where it came from, because this list is how an edit is found again rather
 * than how a clip is chosen.
 */
import { describe, expect, it } from 'vitest';

import { documentEntries, withClipNames } from '../src/editor/documents.js';
import { CANDIDATE, NEW, NEW_DOC, OLD, OLD_DOC, document, twoProjects } from './support/clips.js';
import { fakeApi, project, source } from './support/library.js';

const PROJECTS = [project(NEW, 'Dogfood episode'), project(OLD, 'CUDA kernels', 3_600_000)];
const SOURCES = new Map([
  [NEW, [source(NEW, { absolutePath: '/Volumes/Media/dogfood.mp4' })]],
  [OLD, [source(OLD, { absolutePath: '/Volumes/Media/cuda.mov' })]],
]);

describe('the document list', () => {
  it('names every edit by its recording and project, newest edited first', () => {
    const entries = documentEntries(
      PROJECTS,
      SOURCES,
      new Map([
        [NEW, [document(NEW, NEW_DOC, CANDIDATE, { updatedUnixMillis: 5_000 })]],
        [OLD, [document(OLD, OLD_DOC, CANDIDATE, { updatedUnixMillis: 9_000, revision: 2 })]],
      ]),
    );
    expect(entries.map((entry) => entry.clip.docId)).toEqual([OLD_DOC, NEW_DOC]);
    expect(entries[0]).toMatchObject({
      projectName: 'CUDA kernels',
      sourceName: 'cuda.mov',
      revision: 2,
      clip: {
        projectId: OLD,
        docId: OLD_DOC,
        sourceId: `src_${OLD}`,
        candidateId: CANDIDATE,
        labels: { project: 'CUDA kernels', clip: 'cuda.mov · 000000' },
      },
    });
  });

  it('leaves out a document that names no source, since nothing could play it', () => {
    const entries = documentEntries(
      PROJECTS,
      SOURCES,
      new Map([[OLD, [document(OLD, OLD_DOC, '', { sourceId: '' })]]]),
    );
    expect(entries).toEqual([]);
  });

  it('still lists a document whose recording is no longer registered', () => {
    const entries = documentEntries(
      PROJECTS,
      new Map(),
      new Map([[OLD, [document(OLD, OLD_DOC, '')]]]),
    );
    expect(entries).toHaveLength(1);
    expect(entries[0]?.sourceName).toBeNull();
    expect(entries[0]?.clip.candidateId).toBeUndefined();
    expect(entries[0]?.clip.labels?.clip).toBe('Clip');
  });
});

describe('naming clips by what they are', () => {
  it('uses the title the analysis gave each clip, read once per run', async () => {
    const world = twoProjects({
      editDocs: [document(OLD, OLD_DOC, CANDIDATE)],
    });
    const ranking = JSON.parse(world.documents[`sha256:ranking-${OLD}`]!.json) as {
      cohort: { candidate_id: string; title?: string }[];
    };
    for (const item of ranking.cohort) {
      if (item.candidate_id === CANDIDATE) item.title = 'Why pricing mistakes compound';
    }
    const named = {
      ...world,
      documents: {
        ...world.documents,
        [`sha256:ranking-${OLD}`]: {
          ...world.documents[`sha256:ranking-${OLD}`]!,
          json: JSON.stringify(ranking),
        },
      },
    };
    const api = fakeApi(named);
    const listed = documentEntries(
      named.projects,
      new Map(Object.entries(named.sources)),
      new Map([[OLD, named.editDocs!]]),
    );
    expect(listed[0]!.title).toBeNull();
    const [entry] = await withClipNames(listed, api);
    expect(entry!.title).toBe('Why pricing mistakes compound');
    expect(entry!.clip.labels?.clip).toBe('Why pricing mistakes compound');
  });

  it('keeps the file name when the run gave the clip no title', async () => {
    const world = twoProjects({ editDocs: [document(OLD, OLD_DOC, CANDIDATE)] });
    const listed = documentEntries(
      world.projects,
      new Map(Object.entries(world.sources)),
      new Map([[OLD, world.editDocs!]]),
    );
    const [entry] = await withClipNames(listed, fakeApi(world));
    expect(entry!.clip.labels?.clip).toBe(listed[0]!.clip.labels?.clip);
  });
});
