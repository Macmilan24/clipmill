/**
 * What the editor does with an answer that arrives late.
 *
 * A mutation is in flight when the person opens another clip. Its answer is
 * about the document that was open when it was sent — a plan of A's footage,
 * an inverse for A's undo stack — and applied to B it showed B under A's
 * source interval and put A's inverse on B's history. The audit reproduced
 * exactly that; this holds the repair: after every await a mutation checks it
 * is still on the document it started on, and drops its answer whole if not.
 */
import { act, renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import type { ShellApi } from '../src/daemon/api.js';
import type { AppliedCommand, PreviewPlan } from '../src/daemon/client.js';
import { useEditor } from '../src/editor/useEditor.js';
import type { ClipRef } from '../src/shell/route.js';
import { OLD, OLD_JOB, OLD_SOURCE, plan, twoProjects } from './support/clips.js';
import { fakeApi } from './support/library.js';
import { TICKS, mapping } from './support/plan.js';

const A: ClipRef = { projectId: OLD, docId: 'edt_A', sourceId: OLD_SOURCE, jobId: OLD_JOB };
const B: ClipRef = { projectId: OLD, docId: 'edt_B', sourceId: OLD_SOURCE, jobId: OLD_JOB };

/** A plan whose segment starts at `inSeconds`, so two documents can be told apart. */
function planAt(inSeconds: number): PreviewPlan {
  const program = { frameCount: 30, rateNum: 30, rateDen: 1 };
  return { ...plan(), ...mapping(program, inSeconds) };
}

/** A promise the test resolves by hand. */
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((settle) => {
    resolve = settle;
  });
  return { promise, resolve };
}

describe('a mutation that lands after the document changed', () => {
  it('leaves the new document’s plan, revision and history untouched', async () => {
    const applied = deferred<AppliedCommand>();
    const world = twoProjects();
    const api: ShellApi = {
      ...fakeApi(world),
      previewPlan: (_projectId, docId) =>
        Promise.resolve(docId === 'edt_A' ? planAt(600) : planAt(900)),
      applyEditCommand: () => applied.promise,
    };

    const { result, rerender } = renderHook(({ clip }) => useEditor(clip, api), {
      initialProps: { clip: A },
    });
    await waitFor(() => {
      expect(result.current.plan?.segments[0]?.inTicks).toBe(600 * TICKS);
    });

    // An edit to A goes out and does not come back yet.
    let pending: Promise<void> | null = null;
    act(() => {
      pending = result.current.apply({ op: 'set_gain', t_ticks: 0, gain_db: -3 });
    });
    expect(result.current.busy).toBe(true);

    // B opens meanwhile.
    rerender({ clip: B });
    await waitFor(() => {
      expect(result.current.plan?.segments[0]?.inTicks).toBe(900 * TICKS);
    });
    expect(result.current.busy).toBe(false);

    // A's answer arrives: a new revision and an inverse, both A's.
    await act(async () => {
      applied.resolve({
        docId: 'edt_A',
        revision: 7,
        inverseCommandJson: JSON.stringify({ op: 'remove_gain_point', t_ticks: 0 }),
      });
      await pending;
    });

    // B is still B, at B's revision, with nothing to undo.
    expect(result.current.docId).toBe('edt_B');
    expect(result.current.plan?.segments[0]?.inTicks).toBe(900 * TICKS);
    expect(result.current.revision).toBe(0);
    expect(result.current.canUndo).toBe(false);
    expect(result.current.busy).toBe(false);
    expect(result.current.problem).toBeNull();
  });

  it('keeps the newer plan when an older plan answer arrives afterwards', async () => {
    // Two applies to one document, answered out of order: the second's plan
    // (revision 2) arrives before the first's (revision 1). The picture must
    // be revision 2's.
    const first = deferred<PreviewPlan>();
    const second = deferred<PreviewPlan>();
    let planRequests = 0;
    let applies = 0;
    const world = twoProjects();
    const api: ShellApi = {
      ...fakeApi(world),
      previewPlan: () => {
        planRequests += 1;
        return planRequests === 1
          ? Promise.resolve(planAt(600))
          : planRequests === 2
            ? first.promise
            : second.promise;
      },
      applyEditCommand: () => {
        applies += 1;
        return Promise.resolve({
          docId: 'edt_A',
          revision: applies,
          inverseCommandJson: JSON.stringify({ op: 'remove_gain_point', t_ticks: 0 }),
        });
      },
    };
    const { result } = renderHook(() => useEditor(A, api));
    await waitFor(() => {
      expect(result.current.plan).not.toBeNull();
    });

    let one: Promise<void> | null = null;
    let two: Promise<void> | null = null;
    await act(async () => {
      one = result.current.apply({ op: 'set_gain', t_ticks: 0, gain_db: -3 });
      await Promise.resolve();
    });
    await act(async () => {
      two = result.current.apply({ op: 'set_gain', t_ticks: 90_000, gain_db: -6 });
      await Promise.resolve();
    });
    await act(async () => {
      second.resolve({ ...planAt(605), revision: 2 });
      await two;
      first.resolve({ ...planAt(600), revision: 1 });
      await one;
    });
    expect(result.current.revision).toBe(2);
    expect(result.current.plan?.segments[0]?.inTicks).toBe(605 * TICKS);
  });
});
