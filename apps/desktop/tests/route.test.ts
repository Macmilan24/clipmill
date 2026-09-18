/**
 * Where a route places the shell, and what it remembers between launches.
 *
 * The clip routes are the reason both exist. A section id has nowhere to put
 * "this document of this project", and a shell that forgot the clip on
 * relaunch reopened whichever document the daemon wrote last — which is the
 * substitution every screen in this batch exists to remove.
 */
import { describe, expect, it } from 'vitest';

import { MEMORY_KEY, recall, remember } from '../src/shell/memory.js';
import {
  type ClipRef,
  DEFAULT_ROUTE,
  clipOf,
  editorRoute,
  exportRoute,
  inspectorRoute,
  placementOf,
  sectionRoute,
  resultsRouteFor,
} from '../src/shell/route.js';

const CLIP: ClipRef = {
  projectId: 'p_old',
  docId: 'edt_0000000000000000000000OLD1',
  sourceId: 'src_p_old',
  candidateId: 'cand_0000000000000001',
  jobId: 'job-p_old',
  labels: { project: 'CUDA kernels', clip: 'Clip 01' },
};

describe('placing a clip route', () => {
  it('lights the Editor row and names the project and the clip', () => {
    const { section, trail } = placementOf(editorRoute(CLIP));
    expect(section.id).toBe('editor');
    expect(trail).toEqual(['Editor', 'CUDA kernels', 'Clip 01']);
  });

  it('lights the Export row for the same clip', () => {
    const { section, trail } = placementOf(exportRoute(CLIP));
    expect(section.id).toBe('export');
    expect(trail).toEqual(['Export', 'CUDA kernels', 'Clip 01']);
  });

  it('still reads when the opener knew no names', () => {
    const { labels: _none, ...unnamed } = CLIP;
    const { trail } = placementOf(editorRoute(unnamed));
    expect(trail).toEqual(['Editor', 'Clip']);
  });

  it('carries the run into the inspector route', () => {
    const route = inspectorRoute('p_old', 'src_p_old', 'cand_1', undefined, 'job-p_old');
    expect(route).toEqual({
      kind: 'inspector',
      projectId: 'p_old',
      sourceId: 'src_p_old',
      candidateId: 'cand_1',
      jobId: 'job-p_old',
    });
  });

  it('answers which clip a route is about, and null for the rest', () => {
    expect(clipOf(editorRoute(CLIP))).toEqual(CLIP);
    expect(clipOf(exportRoute(CLIP))).toEqual(CLIP);
    expect(clipOf(sectionRoute('editor'))).toBeNull();
    expect(clipOf(DEFAULT_ROUTE)).toBeNull();
  });
});

/** A `Storage` that lives in a map, so a test owns what was written. */
function store(initial: Record<string, string> = {}) {
  const held = new Map(Object.entries(initial));
  return {
    getItem: (key: string) => held.get(key) ?? null,
    setItem: (key: string, value: string) => {
      held.set(key, value);
    },
    held,
  };
}

describe('what the shell remembers', () => {
  it('puts the route and the clip back as they were', () => {
    const memory = store();
    remember(memory, { route: exportRoute(CLIP), clip: CLIP });
    expect(recall(store({ [MEMORY_KEY]: memory.held.get(MEMORY_KEY)! }))).toEqual({
      route: exportRoute(CLIP),
      clip: CLIP,
    });
  });

  it('round-trips every kind of route', () => {
    for (const route of [
      sectionRoute('library'),
      sectionRoute('results', 'p_old'),
      { kind: 'analysis', projectId: 'p_old', jobId: 'job-p_old', from: 'library' } as const,
      inspectorRoute('p_old', 'src_p_old', 'cand_1', { project: 'CUDA kernels' }, 'job-p_old'),
      editorRoute(CLIP),
    ]) {
      const memory = store();
      remember(memory, { route, clip: null });
      expect(recall(memory).route).toEqual(route);
    }
  });

  it('falls back to the defaults for nothing, garbage, or a route missing an id', () => {
    const fallback = { route: DEFAULT_ROUTE, clip: null };
    expect(recall(null)).toEqual(fallback);
    expect(recall(store())).toEqual(fallback);
    expect(recall(store({ [MEMORY_KEY]: 'not json' }))).toEqual(fallback);
    expect(recall(store({ [MEMORY_KEY]: '[]' }))).toEqual(fallback);
    expect(
      recall(store({ [MEMORY_KEY]: JSON.stringify({ route: { kind: 'editor', clip: {} } }) })),
    ).toEqual(fallback);
    expect(
      recall(
        store({
          [MEMORY_KEY]: JSON.stringify({
            route: { kind: 'inspector', projectId: 'p', sourceId: 's' },
          }),
        }),
      ),
    ).toEqual(fallback);
  });

  it('keeps a clip whose route was unusable, and drops a clip missing its document', () => {
    const kept = recall(
      store({ [MEMORY_KEY]: JSON.stringify({ route: { kind: 'nowhere' }, clip: CLIP }) }),
    );
    expect(kept).toEqual({ route: DEFAULT_ROUTE, clip: CLIP });
    const dropped = recall(
      store({
        [MEMORY_KEY]: JSON.stringify({
          route: sectionRoute('library'),
          clip: { projectId: 'p_old', sourceId: 'src_p_old' },
        }),
      }),
    );
    expect(dropped).toEqual({ route: sectionRoute('library'), clip: null });
  });

  it('survives a store that refuses to write', () => {
    const refusing = {
      getItem: () => null,
      setItem: () => {
        throw new Error('quota');
      },
    };
    expect(() => remember(refusing, { route: DEFAULT_ROUTE, clip: null })).not.toThrow();
  });
});

describe('returning to the board', () => {
  it('preserves the project, recording and run through inspection, editing, export and relaunch', () => {
    for (const route of [
      inspectorRoute(CLIP.projectId, CLIP.sourceId, CLIP.candidateId!, CLIP.labels, CLIP.jobId),
      editorRoute(CLIP),
      exportRoute(CLIP),
    ]) {
      const back = resultsRouteFor(route);
      expect(back).toEqual({
        kind: 'section',
        sectionId: 'results',
        projectId: CLIP.projectId,
        sourceId: CLIP.sourceId,
        jobId: CLIP.jobId,
      });
      const memory = store();
      remember(memory, { route: back, clip: CLIP });
      expect(recall(memory).route).toEqual(back);
    }
  });
});
