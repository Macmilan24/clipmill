/**
 * The model library and storage clean-up, as the daemon answers them.
 *
 * Every mutation answers with the whole library as it then stands, so a screen
 * replaces what it holds rather than guessing what its click did. Downloads
 * and checks continue in the background; the next listing says how far they
 * got. Nothing here decides anything: which model runs, whether a file is safe
 * to delete and whether a download matched its pin are all the daemon's.
 */
import { isTauri, type StorageStats } from './client.js';

export type ModelCapability =
  'editorial' | 'asr' | 'forced-align' | 'vad' | 'speaker-embed' | 'detect-faces';
export type InstallState = 'installed' | 'partial' | 'missing';
export type DownloadState = 'queued' | 'downloading' | 'verifying' | 'failed' | 'cancelled';
/** How the memory a model needs compares with this device. A warning, never a gate. */
export type MemoryFit = 'fits' | 'tight' | 'too_large' | 'unknown';
/** Why a job runs the model it runs. */
export type SelectedBy =
  'chosen' | 'measured' | 'portable' | 'installed_fallback' | 'default' | 'unavailable';

export interface ModelDownload {
  readonly state: DownloadState;
  readonly receivedBytes: number;
  readonly totalBytes: number;
  /** The file in flight, relative to the model's folder. */
  readonly currentFile: string;
  /** One sentence, when the attempt failed. */
  readonly error: string;
  readonly updatedUnixMillis: number;
}

export interface LibraryModel {
  /** Registry name: the identity analyses are keyed against. */
  readonly name: string;
  readonly title: string;
  readonly summary: string;
  readonly capability: string;
  readonly runtime: string;
  readonly backend: string;
  readonly quantization: string;
  readonly licenseSpdx: string;
  readonly sourceRepo: string;
  readonly sourceRevision: string;
  /** What a download costs. */
  readonly downloadBytes: number;
  /** Weights plus the runtime allowance: what running it needs. */
  readonly memoryBytes: number;
  readonly recommended: boolean;
  /** This platform can load the model's runtime at all. */
  readonly supported: boolean;
  readonly unsupportedReason: string;
  readonly memoryFit: MemoryFit;
  /** Pinned by the person rather than bundled with ClipMill. */
  readonly custom: boolean;
  readonly installState: InstallState;
  readonly installedBytes: number;
  readonly download?: ModelDownload;
  /** The next analysis plans this model for its job. */
  readonly inUse: boolean;
  /** The worker family that runs it: only that family is handed its tasks. */
  readonly worker: string;
  /** What a person calls that worker, such as "MLX speech worker". */
  readonly workerTitle: string;
  /** A worker of that family is connected now. */
  readonly workerConnected: boolean;
}

export interface ModelJob {
  readonly capability: string;
  readonly title: string;
  readonly summary: string;
  /** What the next analysis plans for the job. Empty when nothing is registered. */
  readonly model: string;
  readonly selectedBy: SelectedBy;
  /** The person's explicit choice; empty for automatic. */
  readonly choice: string;
  /** Every model registered for the job, in display order. */
  readonly models: readonly string[];
  /**
   * A model more accurate than the planned one that this computer can run
   * and hold in memory; empty or absent when there is none.
   */
  readonly moreAccurate?: string;
}

export interface ModelLibrary {
  readonly models: readonly LibraryModel[];
  readonly jobs: readonly ModelJob[];
  readonly installPath: string;
  /** Absent when the filesystem would not say — not the same as a full disk. */
  readonly availableBytes?: number;
  readonly memoryTotalBytes?: number;
  readonly memoryBudgetBytes?: number;
  readonly recommendedMissingBytes: number;
  readonly recommendedMissing: readonly string[];
}

export interface HubFile {
  readonly path: string;
  readonly bytes: number;
  readonly suggestedName: string;
  readonly suggestedTitle: string;
}

/** What pinning a Hugging Face repository would pin, or why it cannot be. */
export interface HubInspection {
  readonly repo: string;
  /** The commit the revision resolved to; the only one ever pinned. */
  readonly commit: string;
  readonly capability: string;
  readonly license: string;
  readonly licenseSpdx: string;
  readonly licenseAllowed: boolean;
  /** Empty when the repository can be pinned. */
  readonly problem: string;
  /** whisper.cpp: the GGML files to choose from. */
  readonly weightChoices: readonly HubFile[];
  /** MLX: every file the model folder needs, pinned together. */
  readonly files: readonly HubFile[];
  readonly suggestedName: string;
  readonly suggestedTitle: string;
}

export interface AddCustomModel {
  readonly repo: string;
  readonly commit: string;
  readonly capability: string;
  readonly weightsFile: string;
  readonly name: string;
  readonly title: string;
}

export type CleanAction = 'unused_files' | 'temporary' | 'backups';

export interface CleanResult {
  readonly freedBytes: number;
  readonly removedItems: number;
  readonly storage?: StorageStats;
}

export interface ModelLibraryApi {
  listModels(): Promise<ModelLibrary>;
  /** A network operation: the caller has already said so on screen. */
  downloadModels(names: readonly string[]): Promise<ModelLibrary>;
  cancelModelDownload(name: string): Promise<ModelLibrary>;
  removeModel(name: string): Promise<ModelLibrary>;
  verifyModel(name: string): Promise<ModelLibrary>;
  /** An empty model returns the job to automatic selection. */
  setModelChoice(capability: string, model: string): Promise<ModelLibrary>;
  /** Reads a Hugging Face repository: a network operation. */
  inspectHubModel(repo: string, revision: string, capability: string): Promise<HubInspection>;
  addCustomModel(request: AddCustomModel): Promise<ModelLibrary>;
  forgetModel(name: string): Promise<ModelLibrary>;
  cleanStorage(action: CleanAction): Promise<CleanResult>;
  /** Show a storage category's folder. The host resolves the key to a path. */
  openStorageLocation(key: string): Promise<void>;
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) throw new Error('Models are managed in the desktop app.');
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(command, args);
}

export const modelLibraryApi: ModelLibraryApi = {
  listModels: () => call('list_models'),
  downloadModels: (names) => call('download_models', { names: [...names] }),
  cancelModelDownload: (name) => call('cancel_model_download', { name }),
  removeModel: (name) => call('remove_model', { name }),
  verifyModel: (name) => call('verify_model', { name }),
  setModelChoice: (capability, model) => call('set_model_choice', { capability, model }),
  inspectHubModel: (repo, revision, capability) =>
    call('inspect_hub_model', { repo, revision, capability }),
  addCustomModel: (request) => call('add_custom_model', { ...request }),
  forgetModel: (name) => call('forget_model', { name }),
  cleanStorage: (action) => call('clean_storage', { action }),
  openStorageLocation: (key) => call('open_storage_location', { key }),
};
