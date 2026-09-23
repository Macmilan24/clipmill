/**
 * A model library that answers from memory.
 *
 * It records what a screen asked for and applies only the immediate effects
 * the daemon documents for each request — a queued download, a removed
 * model's missing weights, a job's recorded choice. Whether a download
 * verifies, which model a job falls back to and what a clean-up frees are the
 * daemon's decisions; a test that needs one states it through `next`.
 */
import type { StorageStats } from '../../src/daemon/client.js';
import type {
  AddCustomModel,
  CleanAction,
  CleanResult,
  HubInspection,
  LibraryModel,
  ModelJob,
  ModelLibrary,
  ModelLibraryApi,
} from '../../src/daemon/models.js';

const GIB = 1024 ** 3;

export function libraryModel(name: string, overrides: Partial<LibraryModel> = {}): LibraryModel {
  return {
    name,
    title: name,
    summary: `${name} does its job.`,
    capability: 'asr',
    runtime: 'whisper.cpp',
    backend: 'cpu',
    quantization: 'ggml-f16',
    licenseSpdx: 'MIT',
    sourceRepo: 'ggerganov/whisper.cpp',
    sourceRevision: '5359861c739e955e79d9a303bcbc70fb988958b1',
    downloadBytes: 150 * 1024 ** 2,
    memoryBytes: 400 * 1024 ** 2,
    recommended: false,
    supported: true,
    unsupportedReason: '',
    memoryFit: 'fits',
    custom: false,
    installState: 'missing',
    installedBytes: 0,
    inUse: false,
    ...overrides,
  };
}

export function modelJob(capability: string, overrides: Partial<ModelJob> = {}): ModelJob {
  return {
    capability,
    title: capability,
    summary: `The ${capability} job.`,
    model: '',
    selectedBy: 'default',
    choice: '',
    models: [],
    ...overrides,
  };
}

/** The bundled catalog, as a fresh install on an Apple silicon Mac sees it. */
export function freshLibrary(overrides: Partial<ModelLibrary> = {}): ModelLibrary {
  const models = [
    libraryModel('qwen3-5-editorial-mlx', {
      title: 'Qwen3.5 9B',
      summary: 'Finds complete moments and reviews them in context.',
      capability: 'editorial',
      runtime: 'mlx',
      backend: 'mlx',
      quantization: 'int4',
      licenseSpdx: 'Apache-2.0',
      sourceRepo: 'mlx-community/Qwen3.5-9B-4bit',
      downloadBytes: 5_977_071_067,
      memoryBytes: 5_977_071_067 + 2 * GIB,
      recommended: true,
      inUse: true,
    }),
    libraryModel('whisper-base', {
      title: 'Whisper Base',
      summary: 'Fast, compact transcription that runs on any computer.',
      downloadBytes: 147_951_465,
      memoryBytes: 147_951_465 + 256 * 1024 ** 2,
      recommended: true,
      inUse: true,
    }),
    libraryModel('whisper-large-v3-turbo', {
      title: 'Whisper Large v3 Turbo',
      summary: 'More accurate transcription for difficult audio.',
      downloadBytes: 1_624_555_275,
      memoryBytes: 1_624_555_275 + GIB,
    }),
    libraryModel('silero-vad', {
      title: 'Silero VAD',
      summary: 'Finds where people speak.',
      capability: 'vad',
      runtime: 'onnxruntime',
      backend: 'onnx-cpu',
      downloadBytes: 2_243_022,
      memoryBytes: 2_243_022 + 64 * 1024 ** 2,
      recommended: true,
      inUse: true,
    }),
  ];
  const jobs = [
    modelJob('editorial', {
      title: 'Editorial AI',
      summary: 'Finds complete moments, reviews them in context and drafts titles.',
      model: 'qwen3-5-editorial-mlx',
      models: ['qwen3-5-editorial-mlx'],
    }),
    modelJob('asr', {
      title: 'Transcription',
      summary: 'Turns speech into words.',
      model: 'whisper-base',
      selectedBy: 'portable',
      models: ['whisper-base', 'whisper-large-v3-turbo'],
    }),
    modelJob('vad', {
      title: 'Speech detection',
      summary: 'Finds where people speak.',
      model: 'silero-vad',
      models: ['silero-vad'],
    }),
  ];
  const missing = models.filter((model) => model.recommended);
  return {
    models,
    jobs,
    installPath: '/Users/demo/Library/Application Support/dev.clipmill.ClipMill/models',
    availableBytes: 120 * GIB,
    memoryTotalBytes: 24 * GIB,
    memoryBudgetBytes: 18 * GIB,
    recommendedMissingBytes: missing.reduce((total, model) => total + model.downloadBytes, 0),
    recommendedMissing: missing.map((model) => model.name),
    ...overrides,
  };
}

/** The same catalog with every recommended model installed. */
export function installedLibrary(overrides: Partial<ModelLibrary> = {}): ModelLibrary {
  const fresh = freshLibrary();
  return {
    ...fresh,
    models: fresh.models.map((model) =>
      model.recommended
        ? { ...model, installState: 'installed', installedBytes: model.downloadBytes }
        : model,
    ),
    recommendedMissing: [],
    recommendedMissingBytes: 0,
    ...overrides,
  };
}

type Call =
  | readonly ['list']
  | readonly ['download', readonly string[]]
  | readonly ['cancel', string]
  | readonly ['remove', string]
  | readonly ['verify', string]
  | readonly ['choose', string, string]
  | readonly ['inspect', string, string, string]
  | readonly ['add', AddCustomModel]
  | readonly ['forget', string]
  | readonly ['clean', CleanAction]
  | readonly ['open', string];

export class FakeModelLibrary {
  library: ModelLibrary;
  readonly calls: Call[] = [];
  /** A refusal to answer the next request of a kind with, once. */
  readonly refusals = new Map<Call[0], string>();
  inspection: HubInspection | null = null;
  cleanResult: CleanResult = { freedBytes: 0, removedItems: 0 };
  /** Replace the library a request answers with — the daemon's own decision. */
  next: ((library: ModelLibrary, call: Call) => ModelLibrary) | null = null;

  constructor(library: ModelLibrary = freshLibrary()) {
    this.library = library;
  }

  api(): ModelLibraryApi {
    return {
      listModels: () => this.answer(['list'], (library) => library),
      downloadModels: (names) =>
        this.answer(['download', [...names]], (library) =>
          this.update(library, names, () => ({
            download: {
              state: 'queued',
              receivedBytes: 0,
              totalBytes: 0,
              currentFile: '',
              error: '',
              updatedUnixMillis: 0,
            },
          })),
        ),
      cancelModelDownload: (name) =>
        this.answer(['cancel', name], (library) =>
          this.update(library, [name], (model) =>
            model.download ? { download: { ...model.download, state: 'cancelled' } } : {},
          ),
        ),
      removeModel: (name) =>
        this.answer(['remove', name], (library) => ({
          ...library,
          models: library.models.map((model) => {
            if (model.name !== name) return model;
            const { download: _dropped, ...rest } = model;
            return { ...rest, installState: 'missing' as const, installedBytes: 0 };
          }),
          jobs: library.jobs.map((job) =>
            job.choice === name ? { ...job, choice: '', selectedBy: 'default' } : job,
          ),
        })),
      verifyModel: (name) =>
        this.answer(['verify', name], (library) =>
          this.update(library, [name], () => ({
            download: {
              state: 'queued',
              receivedBytes: 0,
              totalBytes: 0,
              currentFile: '',
              error: '',
              updatedUnixMillis: 0,
            },
          })),
        ),
      setModelChoice: (capability, model) =>
        this.answer(['choose', capability, model], (library) => ({
          ...library,
          jobs: library.jobs.map((job) =>
            job.capability === capability
              ? {
                  ...job,
                  choice: model,
                  model: model || job.model,
                  selectedBy: model ? 'chosen' : 'default',
                }
              : job,
          ),
        })),
      inspectHubModel: (repo, revision, capability) => {
        this.calls.push(['inspect', repo, revision, capability]);
        const refusal = this.takeRefusal('inspect');
        if (refusal) return Promise.reject(new Error(refusal));
        return this.inspection
          ? Promise.resolve(this.inspection)
          : Promise.reject(new Error('no inspection configured'));
      },
      addCustomModel: (request) =>
        this.answer(['add', request], (library) => ({
          ...library,
          // The daemon lists a pinned model under its job at once.
          jobs: library.jobs.map((job) =>
            job.capability === request.capability
              ? { ...job, models: [...job.models, request.name] }
              : job,
          ),
          models: [
            ...library.models,
            libraryModel(request.name, {
              title: request.title,
              capability: request.capability,
              custom: true,
              sourceRepo: request.repo,
              sourceRevision: request.commit,
              download: {
                state: 'queued',
                receivedBytes: 0,
                totalBytes: 0,
                currentFile: '',
                error: '',
                updatedUnixMillis: 0,
              },
            }),
          ],
        })),
      forgetModel: (name) =>
        this.answer(['forget', name], (library) => ({
          ...library,
          models: library.models.filter((model) => model.name !== name),
        })),
      cleanStorage: (action) => {
        this.calls.push(['clean', action]);
        const refusal = this.takeRefusal('clean');
        return refusal ? Promise.reject(new Error(refusal)) : Promise.resolve(this.cleanResult);
      },
      openStorageLocation: (key) => {
        this.calls.push(['open', key]);
        const refusal = this.takeRefusal('open');
        return refusal ? Promise.reject(new Error(refusal)) : Promise.resolve();
      },
    };
  }

  /** The calls of one kind, in order. */
  asked(kind: Call[0]): readonly Call[] {
    return this.calls.filter((call) => call[0] === kind);
  }

  private answer(call: Call, effect: (library: ModelLibrary) => ModelLibrary) {
    this.calls.push(call);
    const refusal = this.takeRefusal(call[0]);
    if (refusal) return Promise.reject(new Error(refusal));
    const affected = effect(this.library);
    this.library = this.next ? this.next(affected, call) : affected;
    return Promise.resolve(this.library);
  }

  private takeRefusal(kind: Call[0]): string | undefined {
    const refusal = this.refusals.get(kind);
    this.refusals.delete(kind);
    return refusal;
  }

  private update(
    library: ModelLibrary,
    names: readonly string[],
    change: (model: LibraryModel) => Partial<LibraryModel>,
  ): ModelLibrary {
    return {
      ...library,
      models: library.models.map((model) =>
        names.includes(model.name) ? { ...model, ...change(model) } : model,
      ),
    };
  }
}

/** A storage report with every category present and an estimate. */
export function storageWithCleanUp(overrides: Partial<StorageStats> = {}): StorageStats {
  const base = '/Users/demo/Library/Application Support/dev.clipmill.ClipMill';
  return {
    categories: [
      { key: 'artifacts', bytes: 19 * GIB, items: 412, path: `${base}/artifacts` },
      { key: 'models', bytes: 8 * GIB, items: 23, path: `${base}/models` },
      { key: 'state', bytes: 95 * 1024 ** 2, items: 12, path: `${base}/state` },
      { key: 'imports', bytes: 840 * 1024 ** 2, items: 3, path: `${base}/imports` },
      { key: 'backups', bytes: 387 * 1024 ** 2, items: 5, path: `${base}/state/backups` },
      { key: 'temporary', bytes: 12 * 1024 ** 2, items: 4, path: `${base}/state` },
    ],
    availableBytes: 120 * GIB,
    retentionGraceSeconds: 604_800,
    reclaimableBytes: 3 * GIB,
    reclaimableItems: 41,
    ...overrides,
  };
}
