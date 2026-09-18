/** Local UI fixture only; writes nothing and never enters the production bundle. */
import { JobState, TaskState } from '@clipmill/contracts';
import { daemonApi, type ShellApi } from '../src/daemon/api.js';
import type { ExportBatch } from '../src/daemon/client.js';
import { plan } from './fixtures.js';

const names = [
  'A better question',
  'Leave room for a different answer',
  'Why slowing down helps',
] as const;
const previews = names.map((name, index) => ({
  docId: `preview-edit-${index}`,
  projectId: 'preview',
  sourceId: 'preview-source',
  candidateId: `cand_${index}`,
  jobId: 'preview-analysis',
  revision: 4 + index,
  createdUnixMillis: Date.now(),
  updatedUnixMillis: Date.now() - index * 1000,
  name,
}));
let batches: readonly ExportBatch[] = [
  {
    batchId: 'preview-collection',
    createdUnixMillis: Date.now() - 3600000,
    items: [
      {
        index: 1,
        projectId: 'preview',
        request: {
          docId: 'preview-edit-0',
          index: 1,
          title: names[0]!,
          destinationDir: '/Users/demo/Movies/ClipMill',
          expectedRevision: 4,
        },
        attempt: 1,
        state: 'queued',
        error: '',
        queued: {
          jobId: 'preview-render',
          revision: 4,
          irArtifactId: 'preview-frozen',
          destinationDir: '/Users/demo/Movies/ClipMill',
        },
      },
      {
        index: 2,
        projectId: 'preview',
        request: {
          docId: 'preview-edit-1',
          index: 2,
          title: names[1]!,
          destinationDir: '/Users/demo/Movies/ClipMill',
          expectedRevision: 5,
        },
        attempt: 1,
        state: 'failed',
        error: 'The external drive was disconnected. Reconnect it and retry this clip.',
      },
    ],
  },
];
export const batchApi: ShellApi = {
  ...daemonApi,
  listProjects: async () => [
    {
      projectId: 'preview',
      name: 'The creative process · Episode 12',
      createdUnixMillis: Date.now(),
    },
  ],
  listSources: async () => [
    {
      sourceId: 'preview-source',
      projectId: 'preview',
      absolutePath: '/Preview/creative-process.mp4',
      byteSize: 10000000,
      sourceFingerprint: 'preview-source',
      sourceMapArtifactId: 'preview-map',
      createdUnixMillis: Date.now(),
    },
  ],
  listEditDocs: async () => previews,
  previewPlan: async (_project, docId) => {
    const selected = previews.find((item) => item.docId === docId)!;
    return {
      ...plan,
      revision: selected.revision,
      cues: plan.cues.slice(0, 1).map((cue) => ({
        ...cue,
        lines: [selected.name.split(' ').map((text) => ({ text, holdCentis: 20, wordId: text }))],
      })),
    };
  },
  planExport: async (request) => ({
    passes: !!request.sourceAttestation,
    revision: request.expectedRevision ?? 4,
    findings: request.sourceAttestation
      ? []
      : [
          {
            code: 'rights.required',
            severity: 'blocking',
            detail: 'Choose the permission you hold for this footage.',
          },
        ],
    stem: `${String(request.index).padStart(2, '0')}-${request.title?.toLowerCase().replaceAll(' ', '-')}`,
    fileNames: ['clip.mp4', 'clip.srt', 'clip.vtt'],
    estimatedBytes: 42000000,
    availableBytes: 75000000000,
  }),
  chooseExportFolder: async () => '/Users/demo/Movies/ClipMill',
  listExportBatches: async () => batches,
  submitExportBatch: async (requests) => {
    const batch: ExportBatch = {
      batchId: `preview-${Date.now()}`,
      createdUnixMillis: Date.now(),
      items: requests.map((request) => ({
        index: request.index!,
        projectId: 'preview',
        request,
        state: 'failed',
        attempt: 1,
        error: 'UI preview only. No file was rendered.',
      })),
    };
    batches = [batch, ...batches];
    return batch;
  },
  updateExportBatchItem: async (batchId, index, action) => {
    batches = batches.map((batch) =>
      batch.batchId === batchId
        ? {
            ...batch,
            items: batch.items.map((item) =>
              item.index === index
                ? {
                    ...item,
                    state: action === 'cancel' ? 'cancelled' : 'failed',
                    attempt: item.attempt + (action === 'retry' ? 1 : 0),
                    error: action === 'retry' ? 'UI preview only. No file was rendered.' : '',
                  }
                : item,
            ),
          }
        : batch,
    );
    return batches.find((batch) => batch.batchId === batchId)!;
  },
  fetchJob: async (jobId) => ({
    jobId,
    projectId: 'preview',
    kind: 'export-clip',
    state: JobState.RUNNING,
    createdUnixMillis: Date.now(),
    updatedUnixMillis: Date.now(),
    sourceId: 'preview-source',
    failureClass: 0,
    failureDetail: '',
    outputArtifactIds: [],
    tasks: [
      {
        taskId: 'preview-task',
        kind: 'render-clip',
        outputKind: 'render.clip.v1',
        state: TaskState.RUNNING,
        attempt: 1,
        maxAttempts: 3,
        outputArtifactId: '',
        waitReason: '',
        progress: { unit: 'frames', done: 720, total: 1260 },
      },
    ],
  }),
  revealPath: async () => {},
};
