import { act, renderHook } from '@testing-library/react';
import { JobState, TaskState } from '@clipmill/contracts';
import { afterEach, expect, it, vi } from 'vitest';

import type { ConnectionState, Job, TaskEvent } from '../src/daemon/client.js';
import { useAnalysisActivity } from '../src/shell/useAnalysisActivity.js';

const connected: ConnectionState = {
  status: 'connected',
  daemonVersion: 'test',
  startedUnixMillis: 1,
  localLock: true,
};

function job(jobId: string, state: JobState, kind = 'analyze-source'): Job {
  return {
    jobId,
    projectId: 'project',
    sourceId: 'source',
    kind,
    state,
    createdUnixMillis: 1,
    updatedUnixMillis: Date.now(),
    tasks: [],
    outputArtifactIds: [],
    failureClass: 0,
    failureDetail: '',
  };
}

afterEach(() => {
  vi.useRealTimers();
});

it('follows durable analysis jobs from relaunch through completion across screens', async () => {
  vi.useFakeTimers();
  let onEvent: ((event: TaskEvent) => void) | undefined;
  const current = new Map<string, Job>([['first', job('first', JobState.RUNNING)]]);
  const api = {
    listProjects: vi.fn(async () => [{ projectId: 'project', name: 'Test', createdUnixMillis: 1 }]),
    listJobs: vi.fn(async () => [...current.values()]),
    fetchJob: vi.fn(async (id: string) => current.get(id)!),
  };
  const subscribe = vi.fn(async (handler: (event: TaskEvent) => void) => {
    onEvent = handler;
    return () => {
      onEvent = undefined;
    };
  });
  const { result, rerender, unmount } = renderHook(
    ({ state }: { state: ConnectionState }) => useAnalysisActivity(state, api, subscribe),
    { initialProps: { state: connected as ConnectionState } },
  );

  await act(async () => {
    await Promise.resolve();
    await Promise.resolve();
  });
  expect(result.current).toBe(true);

  const emit = async (id: string) => {
    onEvent?.({
      eventId: 1,
      jobId: id,
      taskId: 'task',
      state: TaskState.SUCCEEDED,
      attempt: 1,
      waitReason: '',
      failureClass: 0,
      atUnixMillis: Date.now(),
    });
    await act(async () => {
      vi.advanceTimersByTime(210);
      await Promise.resolve();
    });
  };

  current.set('first', job('first', JobState.SUCCEEDED));
  await emit('first');
  expect(result.current).toBe(false);

  current.set('export', job('export', JobState.RUNNING, 'render-export'));
  await emit('export');
  expect(result.current).toBe(false);

  current.set('second', job('second', JobState.PLANNED));
  await emit('second');
  expect(result.current).toBe(true);

  rerender({ state: { status: 'disconnected', reason: 'test' } });
  expect(result.current).toBe(false);
  await act(async () => {
    unmount();
    await Promise.resolve();
  });
  expect(onEvent).toBeUndefined();
});
