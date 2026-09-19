import { JobState, TaskState } from '@clipmill/contracts';
import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import type { ShellApi } from '../src/daemon/api.js';
import type { ExportBatch, ExportRequest } from '../src/daemon/client.js';
import { BatchExportScreen } from '../src/screens/BatchExportScreen.js';
import { collectionPattern } from '../src/export/batchPreparation.js';
import {
  CANDIDATE,
  document,
  NEW,
  NEW_DOC,
  OLD,
  OLD_DOC,
  plan,
  twoProjects,
} from './support/clips.js';
import { fakeApi, job, NOW, task } from './support/library.js';
Element.prototype.scrollIntoView ??= () => {};
function fixture(overrides: Partial<ShellApi> = {}) {
  const world = twoProjects({
    editDocs: [document(OLD, OLD_DOC, CANDIDATE), document(NEW, NEW_DOC, CANDIDATE)],
  });
  const api: ShellApi = {
    ...fakeApi(world),
    listExportBatches: vi.fn<ShellApi['listExportBatches']>(async () => []),
    previewPlan: vi.fn<ShellApi['previewPlan']>(async (_projectId, docId) => ({
      ...plan(),
      revision: docId === OLD_DOC ? 4 : 8,
    })),
    planExport: vi.fn<ShellApi['planExport']>(async (request) => ({
      passes: !!request.sourceAttestation,
      revision: request.expectedRevision ?? 0,
      findings: request.sourceAttestation
        ? []
        : [
            {
              code: 'rights.missing',
              severity: 'blocking' as const,
              detail: 'Choose source rights.',
            },
          ],
      stem: `${String(request.index).padStart(2, '0')}-${request.title}`,
      fileNames: ['clip.mp4'],
      estimatedBytes: 12000,
    })),
    submitExportBatch: vi.fn<ShellApi['submitExportBatch']>(async (requests) => ({
      batchId: 'batch-saved',
      createdUnixMillis: NOW,
      items: requests.map((request) => ({
        index: request.index!,
        projectId: request.docId === OLD_DOC ? OLD : NEW,
        request,
        state: 'failed' as const,
        attempt: 1,
        error: 'Fixture refuses render',
      })),
    })),
    updateExportBatchItem: vi.fn<ShellApi['updateExportBatchItem']>(async () => {
      throw new Error('No fixture action');
    }),
    ...overrides,
  };
  return { api, world };
}
async function selectClips(count = 2) {
  const controls = await screen.findAllByRole('checkbox', { name: /^Select / });
  for (const control of controls.slice(0, count)) fireEvent.click(control);
  for (const control of screen.getAllByRole('combobox', { name: 'Source rights' })) {
    fireEvent.click(control);
    fireEvent.click(await screen.findByRole('option', { name: 'I own this footage' }));
  }
  fireEvent.change(screen.getByLabelText('Local folder'), { target: { value: '/tmp/collection' } });
}
async function ready(count: number) {
  const name = `Export ${count} ${count === 1 ? 'clip' : 'clips'}`;
  await waitFor(() =>
    expect(screen.getByRole('button', { name })).toHaveProperty('disabled', false),
  );
  return screen.getByRole('button', { name });
}
it('checks each selected project and submits reviewed revisions with distinct one-based names', async () => {
  const { api } = fixture();
  render(<BatchExportScreen api={api} />);
  await selectClips();
  fireEvent.change(screen.getByLabelText('File naming'), { target: { value: 'episode' } });
  const button = await ready(2);
  fireEvent.click(button);
  fireEvent.click(button);
  await waitFor(() => expect(api.submitExportBatch).toHaveBeenCalledTimes(1));
  const requests = vi.mocked(api.submitExportBatch).mock.calls[0]![0];
  expect(requests.map((request) => request.index)).toEqual([1, 2]);
  expect(new Set(requests.map((request) => request.docId))).toEqual(new Set([OLD_DOC, NEW_DOC]));
  for (const request of requests) {
    expect(request).toMatchObject({
      destinationDir: '/tmp/collection',
      namingPattern: 'episode-{index}',
      title: 'Charging less',
      sourceAttestation: 'own_content',
    });
    expect(request.expectedRevision).toBe(request.docId === OLD_DOC ? 4 : 8);
  }
  expect(api.previewPlan).toHaveBeenCalledWith(OLD, OLD_DOC);
  expect(api.previewPlan).toHaveBeenCalledWith(NEW, NEW_DOC);
  expect(await screen.findByText('Saved')).toBeTruthy();
});
it('requires an explicit rights choice instead of silently assuming ownership', async () => {
  const { api } = fixture();
  render(<BatchExportScreen api={api} />);
  fireEvent.click((await screen.findAllByRole('checkbox', { name: /^Select / }))[0]!);
  fireEvent.change(screen.getByLabelText('Local folder'), { target: { value: '/tmp/collection' } });
  expect(await screen.findByText('Choose source rights.')).toBeTruthy();
  expect(screen.getByRole('button', { name: 'Export 1 clip' })).toHaveProperty('disabled', true);
  expect(api.submitExportBatch).not.toHaveBeenCalled();
});
it('scopes fast-caption and duration confirmations to the revision actually read', async () => {
  let revision = 4;
  const { api } = fixture({
    previewPlan: vi.fn<ShellApi['previewPlan']>(async () => {
      const base = plan();
      return {
        ...base,
        revision,
        segments: base.segments.map((segment) => ({
          ...segment,
          outTicks: segment.inTicks + 70 * 90000,
        })),
      };
    }),
    planExport: vi.fn<ShellApi['planExport']>(async (request) => {
      const confirmed = request.gatesPassed?.includes('captions_reading_rate');
      return {
        revision,
        passes:
          !!request.sourceAttestation &&
          !!confirmed &&
          !!request.gatesPassed?.includes('duration_60s'),
        findings: [
          {
            code: 'captions.reading_rate',
            severity: confirmed ? ('advisory' as const) : ('blocking' as const),
            detail: `A caption runs at 24 characters a second${confirmed ? ' — confirmed as read.' : ''}`,
          },
        ],
        stem: '01-clip',
        fileNames: ['01-clip.mp4'],
        estimatedBytes: 12000,
      };
    }),
  });
  render(<BatchExportScreen api={api} />);
  await selectClips(1);
  fireEvent.click(await screen.findByRole('checkbox', { name: /reviewed this clip’s duration/ }));
  await waitFor(() =>
    expect(screen.getByRole('checkbox', { name: /reviewed the 1 fast caption/ })).toHaveProperty(
      'disabled',
      false,
    ),
  );
  fireEvent.click(screen.getByRole('checkbox', { name: /reviewed the 1 fast caption/ }));
  await ready(1);
  revision = 5;
  fireEvent.change(screen.getByLabelText('File naming'), { target: { value: 'revised-{index}' } });
  await waitFor(() =>
    expect(vi.mocked(api.planExport).mock.calls.at(-1)?.[0].expectedRevision).toBe(5),
  );
  expect(vi.mocked(api.planExport).mock.calls.at(-1)?.[0].gatesPassed).toEqual([]);
  await waitFor(() =>
    expect(screen.getByRole('button', { name: 'Export 1 clip' })).toHaveProperty('disabled', true),
  );
  expect(
    screen
      .getByRole('checkbox', { name: /reviewed the 1 fast caption/ })
      .getAttribute('aria-checked'),
  ).toBe('false');
  expect(api.submitExportBatch).not.toHaveBeenCalled();
});
it('keeps the other selection after one clip fails preflight and exports when it is removed', async () => {
  const { api } = fixture({
    previewPlan: vi.fn<ShellApi['previewPlan']>(async (_projectId, docId) => {
      if (docId === NEW_DOC) throw new Error('Source file is offline');
      return { ...plan(), revision: 4 };
    }),
  });
  render(<BatchExportScreen api={api} />);
  await selectClips();
  expect(await screen.findByText('Source file is offline')).toBeTruthy();
  expect(screen.getByRole('button', { name: 'Export 2 clips' })).toHaveProperty('disabled', true);
  fireEvent.click(screen.getByRole('checkbox', { name: /^Select p_new/ }));
  fireEvent.click(await ready(1));
  await waitFor(() => expect(api.submitExportBatch).toHaveBeenCalled());
  const requests = vi.mocked(api.submitExportBatch).mock.calls[0]![0];
  expect(requests).toHaveLength(1);
  expect(requests[0]?.docId).toBe(OLD_DOC);
});
it('restores independent outcomes and retries only the failed frozen item', async () => {
  const request = (index: number): ExportRequest => ({
    docId: index === 1 ? OLD_DOC : NEW_DOC,
    index,
    title: `Clip ${index}`,
    destinationDir: '/tmp/old-export',
    expectedRevision: 3,
  });
  const batch: ExportBatch = {
    batchId: 'saved-batch',
    createdUnixMillis: NOW,
    items: [
      {
        index: 1,
        projectId: OLD,
        request: request(1),
        attempt: 1,
        state: 'queued',
        error: '',
        queued: {
          jobId: 'delivered-job',
          revision: 3,
          irArtifactId: 'sha256:frozen1',
          destinationDir: '/tmp/old-export',
        },
      },
      {
        index: 2,
        projectId: NEW,
        request: request(2),
        attempt: 1,
        state: 'failed',
        error: 'Destination became unavailable',
      },
    ],
  };
  const { api, world } = fixture({
    listExportBatches: vi.fn<ShellApi['listExportBatches']>(async () => [batch]),
    fetchJob: vi.fn<ShellApi['fetchJob']>(async () => ({
      ...job(OLD, JobState.SUCCEEDED, [task('export.package.v1', TaskState.SUCCEEDED)]),
      jobId: 'delivered-job',
    })),
    readDocument: vi.fn<ShellApi['readDocument']>(async () => ({
      artifactId: 'package',
      kind: 'export.package.v1',
      json: JSON.stringify({ files: [{ name: 'clip1.mp4', role: 'video', bytes: 200 }] }),
    })),
    updateExportBatchItem: vi.fn<ShellApi['updateExportBatchItem']>(async (_id, index, action) => ({
      ...batch,
      items: batch.items.map((item) =>
        item.index === index
          ? {
              ...item,
              state: action === 'cancel' ? 'cancelled' : 'failed',
              attempt: item.attempt + 1,
              error: '',
            }
          : item,
      ),
    })),
  });
  render(<BatchExportScreen api={api} />);
  expect(await screen.findByText('Delivered')).toBeTruthy();
  expect(await screen.findByText('Destination became unavailable')).toBeTruthy();
  const delivered = screen.getByLabelText('Export 1: Clip 1');
  fireEvent.click(within(delivered).getByText(/1 delivered files/));
  fireEvent.click(within(delivered).getByRole('button', { name: 'Reveal' }));
  await waitFor(() => expect(world.revealed).toEqual(['/tmp/old-export/clip1.mp4']));
  fireEvent.click(screen.getByRole('button', { name: 'Retry clip 2' }));
  await waitFor(() =>
    expect(api.updateExportBatchItem).toHaveBeenCalledWith('saved-batch', 2, 'retry'),
  );
  expect(api.submitExportBatch).not.toHaveBeenCalled();
  expect(api.previewPlan).not.toHaveBeenCalled();
});
it('cancels only the requested saved item', async () => {
  const batch: ExportBatch = {
    batchId: 'cancel-batch',
    createdUnixMillis: NOW,
    items: [1, 2].map((index) => ({
      index,
      projectId: OLD,
      request: {
        docId: OLD_DOC,
        index,
        title: `Clip ${index}`,
        destinationDir: '/tmp/clips',
        expectedRevision: 4,
      },
      attempt: 1,
      state: 'pending',
      error: '',
    })),
  };
  const { api } = fixture({
    listExportBatches: vi.fn<ShellApi['listExportBatches']>(async () => [batch]),
    updateExportBatchItem: vi.fn<ShellApi['updateExportBatchItem']>(async (_id, index) => ({
      ...batch,
      items: batch.items.map((item) =>
        item.index === index ? { ...item, state: 'cancelled' } : item,
      ),
    })),
  });
  render(<BatchExportScreen api={api} />);
  fireEvent.click(await screen.findByRole('button', { name: 'Cancel clip 2' }));
  await waitFor(() =>
    expect(api.updateExportBatchItem).toHaveBeenCalledWith('cancel-batch', 2, 'cancel'),
  );
  expect(within(screen.getByLabelText('Export 2: Clip 2')).getByText('Cancelled')).toBeTruthy();
  expect(within(screen.getByLabelText('Export 1: Clip 1')).getByText('Preparing')).toBeTruthy();
});
it('always adds a collection index even when a title token already exists', () => {
  expect(collectionPattern('{clip}')).toBe('{clip}-{index}');
  expect(collectionPattern('')).toBe('{index}-{clip}');
  expect(collectionPattern('{index}-{date}-{clip}')).toBe('{index}-{date}-{clip}');
});

it('follows a retry persisted as pending until the new job is queued', async () => {
  const failed: ExportBatch = {
    batchId: 'restart-poll',
    createdUnixMillis: NOW,
    items: [
      {
        index: 1,
        projectId: OLD,
        request: {
          docId: OLD_DOC,
          title: 'Retry me',
          destinationDir: '/tmp/clips',
          expectedRevision: 4,
          index: 1,
        },
        state: 'failed',
        attempt: 1,
        error: 'Render interrupted',
        queued: {
          jobId: 'previous-failed-job',
          revision: 4,
          irArtifactId: 'same-frozen-ir',
          destinationDir: '/tmp/clips',
        },
      },
    ],
  };
  const pending: ExportBatch = {
    ...failed,
    items: failed.items.map((item) => ({ ...item, state: 'pending', attempt: 2, error: '' })),
  };
  const queued: ExportBatch = {
    ...pending,
    items: pending.items.map((item) => ({
      ...item,
      state: 'queued',
      queued: {
        jobId: 'retry-job',
        revision: 4,
        irArtifactId: 'same-frozen-ir',
        destinationDir: '/tmp/clips',
      },
    })),
  };
  const { api } = fixture({
    listExportBatches: vi
      .fn<ShellApi['listExportBatches']>()
      .mockResolvedValueOnce([failed])
      .mockResolvedValueOnce([pending])
      .mockResolvedValue([queued]),
    updateExportBatchItem: vi.fn<ShellApi['updateExportBatchItem']>(async () => pending),
    fetchJob: vi.fn<ShellApi['fetchJob']>(async () => ({
      ...job(OLD, JobState.RUNNING, [
        task('render.clip.v1', TaskState.RUNNING, {
          progress: { unit: 'frames', done: 10, total: 30 },
        }),
      ]),
      jobId: 'retry-job',
    })),
  });
  render(<BatchExportScreen api={api} />);
  fireEvent.click(await screen.findByRole('button', { name: 'Retry clip 1' }));
  expect(await screen.findByText('Preparing')).toBeTruthy();
  expect(api.fetchJob).not.toHaveBeenCalledWith('previous-failed-job');
  await waitFor(() => expect(api.fetchJob).toHaveBeenCalledWith('retry-job'), { timeout: 3000 });
  expect(await screen.findByText('10 of 30 frames')).toBeTruthy();
  expect(api.listExportBatches).toHaveBeenCalledTimes(3);
});

it('shows rendered media time without the downstream delivery wait, but keeps a real queue wait', async () => {
  const batch: ExportBatch = {
    batchId: 'progress-batch',
    createdUnixMillis: NOW,
    items: [
      {
        index: 1,
        projectId: OLD,
        request: { docId: OLD_DOC, title: 'Native progress', destinationDir: '/tmp/clips' },
        state: 'queued',
        attempt: 1,
        error: '',
        queued: {
          jobId: 'progress-job',
          revision: 4,
          irArtifactId: 'frozen-ir',
          destinationDir: '/tmp/clips',
        },
      },
    ],
  };
  let rendering = true;
  const { api } = fixture({
    listExportBatches: vi.fn<ShellApi['listExportBatches']>(async () => [batch]),
    fetchJob: vi.fn<ShellApi['fetchJob']>(async () => ({
      ...job(OLD, JobState.RUNNING, [
        task('render.clip.v1', rendering ? TaskState.RUNNING : TaskState.PLANNED, {
          ...(rendering ? { progress: { unit: 'media_millis', done: 6336, total: 15900 } } : {}),
          waitReason: rendering ? '' : 'waiting: admission',
        }),
        task('export.package.v1', TaskState.PLANNED, { waitReason: 'waiting: dependencies' }),
      ]),
      jobId: 'progress-job',
    })),
  });
  render(<BatchExportScreen api={api} />);
  expect(await screen.findByText('0:06.3 of 0:15.9 rendered')).toBeTruthy();
  expect(screen.queryByText(/media_millis|dependencies|After rendering/)).toBeNull();
  rendering = false;
  expect(await screen.findByText('Waiting to start')).toBeTruthy();
  expect(screen.queryByText(/waiting:|dependencies/)).toBeNull();
});
