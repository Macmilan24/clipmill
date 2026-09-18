import { act, renderHook, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { CropPath } from '../src/daemon/client.js';
import { EMPTY_SNAPSHOT, ResultsLoader, type ResultsSnapshot } from '../src/results/loader.js';
import { useResults } from '../src/results/useResults.js';
import { emptyWorld, fakeApi, source } from './support/library.js';
import { CANDIDATE, OTHER_CANDIDATE, OLD, twoProjects } from './support/clips.js';
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}
afterEach(() => vi.restoreAllMocks());
describe('results selection races', () => {
  it('discards a project load that finishes after the user switches projects', async () => {
    const old = deferred<ResultsSnapshot>();
    const current = { ...EMPTY_SNAPSHOT, source: source('current') };
    vi.spyOn(ResultsLoader.prototype, 'load').mockImplementation((id) =>
      id === 'old' ? old.promise : Promise.resolve(current),
    );
    const api = fakeApi(emptyWorld());
    const hook = renderHook(({ id }) => useResults(id, null, null, api), {
      initialProps: { id: 'old' },
    });
    hook.rerender({ id: 'current' });
    await waitFor(() => expect(hook.result.current.snapshot.source?.projectId).toBe('current'));
    await act(async () => old.resolve({ ...EMPTY_SNAPSHOT, source: source('old') }));
    expect(hook.result.current.snapshot.source?.projectId).toBe('current');
  });

  it('discards a crop answer from the previously inspected clip', async () => {
    const api = fakeApi(twoProjects());
    const snapshot = await new ResultsLoader(api).load(OLD, null, null);
    vi.spyOn(ResultsLoader.prototype, 'load').mockResolvedValue({
      ...snapshot,
      faceTrackArtifactId: 'faces',
    });
    const old = deferred<CropPath>();
    const current: CropPath = {
      fit: true,
      fitReason: 'No face in this clip',
      containment: 0,
      keyframes: [],
    };
    vi.spyOn(api, 'solveCropPath')
      .mockImplementationOnce(() => old.promise)
      .mockResolvedValueOnce(current);
    const hook = renderHook(() => useResults(OLD, null, null, api));
    await waitFor(() => expect(hook.result.current.snapshot.rows.length).toBe(2));
    act(() => hook.result.current.solveFor(CANDIDATE));
    act(() => hook.result.current.solveFor(OTHER_CANDIDATE));
    await waitFor(() => expect(hook.result.current.crop).toEqual(current));
    await act(async () =>
      old.resolve({ fit: false, fitReason: '', containment: 1, keyframes: [] }),
    );
    expect(hook.result.current.crop).toEqual(current);
  });

  it.each(['approval', 'variation', 'batch'] as const)(
    'does not reload the previous project when a delayed %s completes',
    async (operation) => {
      const api = fakeApi(twoProjects());
      const finish = deferred<void>();
      const original = api.directClip;
      vi.spyOn(api, 'directClip').mockImplementation(async (request) => {
        await finish.promise;
        return original(request);
      });
      const loads = vi.spyOn(ResultsLoader.prototype, 'load');
      const hook = renderHook(({ id }) => useResults(id, null, null, api), {
        initialProps: { id: OLD },
      });
      await waitFor(() => expect(hook.result.current.snapshot.source?.projectId).toBe(OLD));
      let pending!: Promise<unknown>;
      act(() => {
        pending =
          operation === 'approval'
            ? hook.result.current.decide(CANDIDATE, 'approved')
            : operation === 'variation'
              ? hook.result.current.direct(CANDIDATE, 'alternative')
              : hook.result.current.approveMany([CANDIDATE]);
      });
      hook.rerender({ id: 'p_new' });
      await waitFor(() => expect(hook.result.current.snapshot.source?.projectId).toBe('p_new'));
      await act(async () => {
        finish.resolve();
        await pending;
      });
      expect(hook.result.current.snapshot.source?.projectId).toBe('p_new');
      expect(hook.result.current.notice).toBeNull();
      expect(hook.result.current.busy).toBe(false);
      expect(loads.mock.calls.filter(([id]) => id === OLD)).toHaveLength(1);
    },
  );
});
