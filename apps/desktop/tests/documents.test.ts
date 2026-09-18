/**
 * The list a clip-less Editor or Export offers, as a join.
 *
 * Every entry must be openable — project, document, source — and named by
 * where it came from, because this list is how an edit is found again rather
 * than how a clip is chosen.
 */
import { describe, expect, it } from 'vitest';

import { documentEntries } from '../src/editor/documents.js';
import { CANDIDATE, NEW, NEW_DOC, OLD, OLD_DOC, document } from './support/clips.js';
import { project, source } from './support/library.js';

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
