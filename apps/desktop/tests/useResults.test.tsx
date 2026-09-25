import { act, renderHook, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { CropPath } from '../src/daemon/client.js';
import { rememberLook } from '../src/results/captionLook.js';
import { EMPTY_SNAPSHOT, ResultsLoader, type ResultsSnapshot } from '../src/results/loader.js';
import { useResults } from '../src/results/useResults.js';
import { emptyWorld, fakeApi, source } from './support/library.js';
import {
  CANDIDATE,
  OTHER_CANDIDATE,
  OLD,
  OLD_DOC,
  OLD_JOB,
  OLD_SOURCE,
  document,
  twoProjects,
} from './support/clips.js';
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

  it.each(['approval', 'variation', 'batch', 'manual'] as const)(
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
            ? hook.result.current.approve(CANDIDATE, null)
            : operation === 'variation'
              ? hook.result.current.approve(CANDIDATE, {
                  startTicks: 601 * 90_000,
                  endTicks: 630 * 90_000,
                })
              : operation === 'manual'
                ? hook.result.current.manual(600 * 90_000, 630 * 90_000)
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

it('creates a manual edit from the named run without pretending it was approved by the model', async () => {
  const api = fakeApi(twoProjects());
  const direct = vi.spyOn(api, 'directClip');
  const hook = renderHook(() => useResults(OLD, OLD_SOURCE, OLD_JOB, api));
  await waitFor(() => expect(hook.result.current.snapshot.run?.jobId).toBe(OLD_JOB));
  await act(async () => {
    await hook.result.current.manual(600 * 90_000, 630 * 90_000);
  });
  expect(direct).toHaveBeenCalledWith({
    projectId: OLD,
    sourceId: OLD_SOURCE,
    jobId: OLD_JOB,
    candidateId: '',
    cut: 'exact',
    startTicks: 600 * 90_000,
    endTicks: 630 * 90_000,
    manualSpan: true,
    approve: false,
    highlightSpokenWord: true,
  });
});

describe('declined edit intent', () => {
  it('requires an individual edit action and never overrides declines in a bulk approval', async () => {
    const api = fakeApi(twoProjects());
    const snapshot = await new ResultsLoader(api).load(OLD, null, null);
    vi.spyOn(ResultsLoader.prototype, 'load').mockResolvedValue({
      ...snapshot,
      rows: snapshot.rows.map((row) =>
        row.candidateId === CANDIDATE
          ? Object.assign({}, row, {
              review: {
                status: 'rejected' as const,
                route: 'local' as const,
                reasons: ['The payoff is incomplete.'],
              },
            })
          : row,
      ),
    });
    const direct = vi.spyOn(api, 'directClip');
    const hook = renderHook(() => useResults(OLD, null, null, api));
    await waitFor(() => expect(hook.result.current.snapshot.rows).toHaveLength(2));
    await act(async () => hook.result.current.approveMany([CANDIDATE]));
    expect(direct).not.toHaveBeenCalled();
    expect(hook.result.current.notice).toContain('Inspect declined moments individually');
    await act(async () => {
      await hook.result.current.approve(CANDIDATE, null);
    });
    expect(direct).toHaveBeenCalledWith(
      expect.objectContaining({ candidateId: CANDIDATE, approve: true, allowDeclined: true }),
    );
  });
});

describe('approving the cut on screen', () => {
  it('names the runner-up as itself and any other moved cut as exact', async () => {
    const api = fakeApi(twoProjects());
    const direct = vi.spyOn(api, 'directClip');
    const hook = renderHook(() => useResults(OLD, OLD_SOURCE, OLD_JOB, api));
    await waitFor(() => expect(hook.result.current.snapshot.rows).toHaveLength(2));
    await act(async () => {
      await hook.result.current.approve(CANDIDATE, {
        startTicks: 601 * 90_000,
        endTicks: 630 * 90_000,
      });
    });
    await act(async () => {
      await hook.result.current.approve(CANDIDATE, {
        startTicks: 598 * 90_000,
        endTicks: 633 * 90_000,
      });
    });
    expect(direct.mock.calls[0]?.[0]).toMatchObject({ cut: 'alternative', approve: true });
    expect(direct.mock.calls[0]?.[0]).not.toHaveProperty('startTicks');
    expect(direct.mock.calls[1]?.[0]).toMatchObject({
      cut: 'exact',
      startTicks: 598 * 90_000,
      endTicks: 633 * 90_000,
      approve: true,
    });
  });

  it('builds the edit in the caption look chosen for the project', async () => {
    rememberLook(OLD, 'clipmill.captions.minimal.v1');
    const api = fakeApi(twoProjects());
    const direct = vi.spyOn(api, 'directClip');
    const hook = renderHook(() => useResults(OLD, OLD_SOURCE, OLD_JOB, api));
    await waitFor(() => expect(hook.result.current.snapshot.rows).toHaveLength(2));
    await act(async () => {
      await hook.result.current.approve(CANDIDATE, null);
    });
    expect(direct.mock.calls[0]?.[0]).toMatchObject({
      styleRef: 'clipmill.captions.minimal.v1',
    });
    localStorage.removeItem(`clipmill.captionLook.${OLD}`);
  });

  it('asks for a second edit only when the clip already has one and the cut moved', async () => {
    const world = twoProjects({ editDocs: [document(OLD, OLD_DOC, CANDIDATE)] });
    const api = fakeApi(world);
    const direct = vi.spyOn(api, 'directClip');
    const hook = renderHook(() => useResults(OLD, OLD_SOURCE, OLD_JOB, api));
    await waitFor(() =>
      expect(hook.result.current.snapshot.rows.find((row) => row.docId)).toBeTruthy(),
    );
    await act(async () => {
      await hook.result.current.approve(CANDIDATE, null);
    });
    await act(async () => {
      await hook.result.current.approve(CANDIDATE, {
        startTicks: 598 * 90_000,
        endTicks: 630 * 90_000,
      });
    });
    expect(direct.mock.calls[0]?.[0]).not.toHaveProperty('variation');
    expect(direct.mock.calls[1]?.[0]).toMatchObject({ cut: 'exact', variation: true });
  });

  it('marks the decision on its row at once, without blanking the board to reload', async () => {
    const api = fakeApi(twoProjects());
    const hook = renderHook(() => useResults(OLD, OLD_SOURCE, OLD_JOB, api));
    await waitFor(() => expect(hook.result.current.snapshot.rows).toHaveLength(2));
    const loading: boolean[] = [];
    await act(async () => {
      const pending = hook.result.current.decide(CANDIDATE, 'kept');
      loading.push(hook.result.current.loading);
      await pending;
    });
    loading.push(hook.result.current.loading);
    expect(loading).toEqual([false, false]);
    expect(
      hook.result.current.snapshot.rows.find((row) => row.candidateId === CANDIDATE)?.decision,
    ).toBe('kept');
    expect(hook.result.current.notice).toBe('Kept for later.');
  });

  it('takes a decision back with null', async () => {
    const world = twoProjects();
    const api = fakeApi(world);
    const hook = renderHook(() => useResults(OLD, OLD_SOURCE, OLD_JOB, api));
    await waitFor(() => expect(hook.result.current.snapshot.rows).toHaveLength(2));
    await act(async () => {
      await hook.result.current.decide(CANDIDATE, 'rejected');
    });
    await act(async () => {
      await hook.result.current.decide(CANDIDATE, null);
    });
    expect(world.decisions.has(CANDIDATE)).toBe(false);
    expect(
      hook.result.current.snapshot.rows.find((row) => row.candidateId === CANDIDATE)?.decision,
    ).toBeNull();
    expect(hook.result.current.notice).toBe('Decision cleared.');
  });
});
