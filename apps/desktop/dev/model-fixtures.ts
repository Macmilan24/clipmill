/**
 * A model library for browser styling checks: the bundled catalog as a fresh
 * install on an Apple silicon Mac sees it, with downloads that visibly move.
 * Never imported by the app entrypoint, and nothing here reaches a network.
 */
import type { StorageStats } from '../src/daemon/client.js';
import type {
  CleanAction,
  CleanResult,
  HubInspection,
  LibraryModel,
  ModelJob,
  ModelLibrary,
  ModelLibraryApi,
} from '../src/daemon/models.js';

const GIB = 1024 ** 3;
const MIB = 1024 ** 2;

function model(name: string, fields: Partial<LibraryModel>): LibraryModel {
  return {
    name,
    title: name,
    summary: '',
    capability: 'asr',
    runtime: 'whisper.cpp',
    backend: 'cpu',
    quantization: 'ggml-f16',
    licenseSpdx: 'MIT',
    sourceRepo: 'ggerganov/whisper.cpp',
    sourceRevision: '5359861c739e955e79d9a303bcbc70fb988958b1',
    downloadBytes: 0,
    memoryBytes: 0,
    recommended: false,
    supported: true,
    unsupportedReason: '',
    memoryFit: 'fits',
    custom: false,
    installState: 'missing',
    installedBytes: 0,
    inUse: false,
    ...fields,
  };
}

const CATALOG: readonly LibraryModel[] = [
  model('qwen3-5-editorial-mlx', {
    title: 'Qwen3.5 9B',
    summary:
      'A 9-billion-parameter vision-language model, quantized to 4 bits, that reads the transcript and looks at frames. Runs on the Apple silicon GPU.',
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
  model('whisper-base', {
    title: 'Whisper Base',
    summary:
      'Fast and compact. Runs on any computer, and is used until something else is chosen or measured faster.',
    downloadBytes: 147_951_465,
    memoryBytes: 147_951_465 + 256 * MIB,
    recommended: true,
    inUse: true,
  }),
  model('whisper-large-v3-turbo', {
    title: 'Whisper Large v3 Turbo',
    summary:
      'More accurate on accents, crosstalk and noisy rooms. Slower than Whisper Base and needs more memory.',
    downloadBytes: 1_624_555_275,
    memoryBytes: 1_624_555_275 + GIB,
  }),
  model('qwen3-asr-mlx', {
    title: 'Qwen3-ASR 1.7B',
    summary: 'Faster transcription on the Apple silicon GPU.',
    runtime: 'mlx',
    backend: 'mlx',
    quantization: 'int4',
    licenseSpdx: 'Apache-2.0',
    sourceRepo: 'mlx-community/Qwen3-ASR-1.7B-4bit',
    downloadBytes: 1_607_551_611,
    memoryBytes: 1_607_551_611 + 2 * GIB,
  }),
  model('wav2vec2-ctc-en', {
    title: 'Wav2Vec2 word timing (English)',
    summary: 'English only. Runs on any computer.',
    capability: 'forced-align',
    runtime: 'onnxruntime',
    backend: 'onnx-cpu',
    licenseSpdx: 'Apache-2.0',
    downloadBytes: 377_912_464,
    memoryBytes: 377_912_464 + 768 * MIB,
    recommended: true,
    inUse: true,
  }),
  model('qwen3-aligner-mlx', {
    title: 'Qwen3 Forced Aligner 0.6B',
    summary: 'Faster word timing on the Apple silicon GPU.',
    capability: 'forced-align',
    runtime: 'mlx',
    backend: 'mlx',
    quantization: 'int4',
    licenseSpdx: 'Apache-2.0',
    downloadBytes: 975_775_725,
    memoryBytes: 975_775_725 + GIB,
  }),
  model('silero-vad', {
    title: 'Silero VAD',
    summary: 'Tiny and fast. Runs on any computer.',
    capability: 'vad',
    runtime: 'onnxruntime',
    backend: 'onnx-cpu',
    downloadBytes: 2_243_022,
    memoryBytes: 2_243_022 + 64 * MIB,
    recommended: true,
    inUse: true,
  }),
  model('yunet-face', {
    title: 'YuNet face detection',
    summary: 'Tiny and fast. Runs on any computer.',
    capability: 'detect-faces',
    runtime: 'onnxruntime',
    backend: 'onnx-cpu',
    downloadBytes: 232_589,
    memoryBytes: 232_589 + 64 * MIB,
    recommended: true,
    inUse: true,
  }),
];

const JOBS: readonly ModelJob[] = [
  {
    capability: 'editorial',
    title: 'Editorial AI',
    summary:
      'Finds complete moments, reviews them in context, checks visual references and drafts titles.',
    model: 'qwen3-5-editorial-mlx',
    selectedBy: 'default',
    choice: '',
    models: ['qwen3-5-editorial-mlx'],
  },
  {
    capability: 'asr',
    title: 'Transcription',
    summary: 'Turns speech into words.',
    model: 'whisper-base',
    selectedBy: 'portable',
    choice: '',
    models: ['whisper-base', 'whisper-large-v3-turbo', 'qwen3-asr-mlx'],
  },
  {
    capability: 'forced-align',
    title: 'Word timing',
    summary: 'Times each word against the audio, so captions land exactly.',
    model: 'wav2vec2-ctc-en',
    selectedBy: 'portable',
    choice: '',
    models: ['wav2vec2-ctc-en', 'qwen3-aligner-mlx'],
  },
  {
    capability: 'vad',
    title: 'Speech detection',
    summary: 'Finds where people speak, so nothing else is transcribed.',
    model: 'silero-vad',
    selectedBy: 'default',
    choice: '',
    models: ['silero-vad'],
  },
  {
    capability: 'detect-faces',
    title: 'Face tracking',
    summary: 'Finds faces, so vertical crops follow the speaker.',
    model: 'yunet-face',
    selectedBy: 'default',
    choice: '',
    models: ['yunet-face'],
  },
];

const INSPECTION: HubInspection = {
  repo: 'ggerganov/whisper.cpp',
  commit: '5359861c739e955e79d9a303bcbc70fb988958b1',
  capability: 'asr',
  license: 'mit',
  licenseSpdx: 'MIT',
  licenseAllowed: true,
  problem: '',
  weightChoices: [
    {
      path: 'ggml-small.bin',
      bytes: 487_601_967,
      suggestedName: 'whisper-small',
      suggestedTitle: 'Whisper small',
    },
    {
      path: 'ggml-small.en.bin',
      bytes: 487_614_201,
      suggestedName: 'whisper-small.en',
      suggestedTitle: 'Whisper small.en',
    },
    {
      path: 'ggml-medium.bin',
      bytes: 1_533_763_059,
      suggestedName: 'whisper-medium',
      suggestedTitle: 'Whisper medium',
    },
  ],
  files: [],
  suggestedName: 'whisper-small',
  suggestedTitle: 'Whisper small',
};

type Scenario = 'fresh' | 'installed';

/** An in-memory library whose downloads advance each time it is read. */
export function previewModelApi(scenario: Scenario): ModelLibraryApi {
  let models: LibraryModel[] = CATALOG.map((entry) =>
    scenario === 'installed' && entry.recommended
      ? { ...entry, installState: 'installed', installedBytes: entry.downloadBytes }
      : entry,
  );
  let jobs: ModelJob[] = [...JOBS];
  const snapshot = (): ModelLibrary => {
    const missing = models.filter(
      (entry) => entry.recommended && !entry.custom && entry.installState !== 'installed',
    );
    return {
      models,
      jobs,
      installPath: '/Users/demo/Library/Application Support/dev.clipmill.ClipMill/models',
      availableBytes: 312 * GIB,
      memoryTotalBytes: 24 * GIB,
      memoryBudgetBytes: 18 * GIB,
      recommendedMissing: missing.map((entry) => entry.name),
      recommendedMissingBytes: missing.reduce(
        (total, entry) => total + entry.downloadBytes - entry.installedBytes,
        0,
      ),
    };
  };
  const change = (names: readonly string[], edit: (entry: LibraryModel) => LibraryModel) => {
    models = models.map((entry) => (names.includes(entry.name) ? edit(entry) : entry));
  };
  // Advance the first active download: one moves at a time, as in the daemon.
  const advance = () => {
    const active = models.find(
      (entry) => entry.download?.state === 'downloading' || entry.download?.state === 'queued',
    );
    if (!active?.download) return;
    const step = Math.max(active.downloadBytes / 9, 40 * MIB);
    const received = Math.min(active.downloadBytes, active.download.receivedBytes + step);
    if (received >= active.downloadBytes) {
      const { download: _done, ...rest } = active;
      change([active.name], () => ({
        ...rest,
        installState: 'installed',
        installedBytes: active.downloadBytes,
      }));
      return;
    }
    change([active.name], (entry) => ({
      ...entry,
      installState: 'partial',
      download: {
        state: 'downloading',
        receivedBytes: received,
        totalBytes: entry.downloadBytes,
        currentFile:
          entry.backend === 'mlx' ? 'model-00001-of-00002.safetensors' : `${entry.name}.bin`,
        error: '',
        updatedUnixMillis: Date.now(),
      },
    }));
  };
  const queue = (names: readonly string[]) =>
    change(names, (entry) => ({
      ...entry,
      download: {
        state: 'queued',
        receivedBytes: entry.download?.receivedBytes ?? 0,
        totalBytes: entry.downloadBytes,
        currentFile: '',
        error: '',
        updatedUnixMillis: Date.now(),
      },
    }));
  return {
    listModels: async () => {
      advance();
      return snapshot();
    },
    downloadModels: async (names) => {
      queue(names);
      return snapshot();
    },
    cancelModelDownload: async (name) => {
      change([name], (entry) =>
        entry.download ? { ...entry, download: { ...entry.download, state: 'cancelled' } } : entry,
      );
      return snapshot();
    },
    removeModel: async (name) => {
      change([name], (entry) => {
        const { download: _gone, ...rest } = entry;
        return { ...rest, installState: 'missing', installedBytes: 0 };
      });
      jobs = jobs.map((job) => (job.choice === name ? { ...job, choice: '' } : job));
      return snapshot();
    },
    verifyModel: async () => snapshot(),
    setModelChoice: async (capability, chosen) => {
      jobs = jobs.map((job) =>
        job.capability === capability
          ? {
              ...job,
              choice: chosen,
              model: chosen || JOBS.find((entry) => entry.capability === capability)!.model,
              selectedBy: chosen ? 'chosen' : 'default',
            }
          : job,
      );
      models = models.map((entry) => ({
        ...entry,
        inUse: jobs.some((job) => job.model === entry.name),
      }));
      return snapshot();
    },
    inspectHubModel: async (repo) => ({ ...INSPECTION, repo: repo || INSPECTION.repo }),
    addCustomModel: async (request) => {
      models = [
        ...models,
        model(request.name, {
          title: request.title,
          summary: `Added from ${request.repo} at ${request.commit.slice(0, 7)}.`,
          capability: request.capability,
          custom: true,
          downloadBytes:
            INSPECTION.weightChoices.find((file) => file.path === request.weightsFile)?.bytes ?? 0,
          memoryBytes: 900 * MIB,
        }),
      ];
      jobs = jobs.map((job) =>
        job.capability === request.capability
          ? { ...job, models: [...job.models, request.name] }
          : job,
      );
      queue([request.name]);
      return snapshot();
    },
    forgetModel: async (name) => {
      models = models.filter((entry) => entry.name !== name);
      jobs = jobs.map((job) => ({ ...job, models: job.models.filter((entry) => entry !== name) }));
      return snapshot();
    },
    cleanStorage: async (): Promise<CleanResult> => ({ freedBytes: 0, removedItems: 0 }),
    openStorageLocation: async () => undefined,
  };
}

/** Every storage category, with a clean-up estimate, for the Settings page. */
export const storageWithCleanUp: StorageStats = {
  categories: [
    {
      key: 'artifacts',
      bytes: 20_078_895_104,
      items: 412,
      path: '/Users/demo/Library/Application Support/dev.clipmill.ClipMill/artifacts',
    },
    {
      key: 'models',
      bytes: 8_589_934_592,
      items: 23,
      path: '/Users/demo/Library/Application Support/dev.clipmill.ClipMill/models',
    },
    {
      key: 'state',
      bytes: 94_371_840,
      items: 12,
      path: '/Users/demo/Library/Application Support/dev.clipmill.ClipMill/state',
    },
    {
      key: 'imports',
      bytes: 881_852_416,
      items: 3,
      path: '/Users/demo/Library/Application Support/dev.clipmill.ClipMill/imports',
    },
    {
      key: 'backups',
      bytes: 405_798_912,
      items: 5,
      path: '/Users/demo/Library/Application Support/dev.clipmill.ClipMill/state/backups',
    },
    {
      key: 'temporary',
      bytes: 12_582_912,
      items: 4,
      path: '/Users/demo/Library/Application Support/dev.clipmill.ClipMill/state',
    },
  ],
  availableBytes: 335_007_449_088,
  retentionGraceSeconds: 604_800,
  reclaimableBytes: 3_221_225_472,
  reclaimableItems: 41,
};

/** A clean-up that reports what a real one would, for the preview only. */
export async function previewClean(action: CleanAction): Promise<CleanResult> {
  const freed: Record<CleanAction, CleanResult> = {
    unused_files: { freedBytes: 3_221_225_472, removedItems: 41 },
    backups: { freedBytes: 324_639_130, removedItems: 4 },
    temporary: { freedBytes: 12_582_912, removedItems: 4 },
  };
  return freed[action];
}
