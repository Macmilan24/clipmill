/**
 * The renderer's only way to reach the daemon.
 *
 * There is no socket, no fetch and no filesystem here — only commands the Rust
 * host exposes. Outside a Tauri window (a plain `pnpm dev` browser tab, or a
 * test) the bridge reports itself unavailable instead of throwing, so the shell
 * still renders and says why it has no data.
 *
 * States cross as the integers the contract defines and are read here through
 * the generated enums, so no screen depends on a string this file made up.
 * Documents cross as the canonical JSON the daemon published and are parsed with
 * the generated schema types, which keeps the JSON Schema the only contract
 * between the two ends.
 */
import type { DeviceProfile } from '@clipmill/contracts';
import type { FailureClass, JobState, TaskState } from '@clipmill/contracts';

export type ConnectionState =
  | { readonly status: 'connecting' }
  | {
      readonly status: 'connected';
      readonly daemonVersion: string;
      readonly localLock: boolean;
      readonly startedUnixMillis: number;
    }
  | { readonly status: 'disconnected'; readonly reason: string };

export interface DeviceProfileResult {
  readonly artifactId: string;
  readonly profile: DeviceProfile;
}

/** Emitted by the host on every connection transition. */
const STATE_EVENT = 'daemon://state';
/** Emitted by the host for every task transition in the daemon's durable log. */
const TASK_EVENT = 'daemon://task-events';

export interface Project {
  readonly projectId: string;
  readonly name: string;
  readonly createdUnixMillis: number;
}

export interface Source {
  readonly sourceId: string;
  readonly projectId: string;
  readonly absolutePath: string;
  readonly byteSize: number;
  readonly sourceFingerprint: string;
  readonly sourceMapArtifactId: string;
  readonly createdUnixMillis: number;
}

/**
 * What a stage has done so far, in the unit it measured.
 *
 * `total` of zero means the stage knows how far it has come and not how far
 * there is to go. A bar drawn from that would be inventing the denominator, so
 * a caller must check it before dividing.
 */
export interface Progress {
  readonly unit: string;
  readonly done: number;
  readonly total: number;
}

export interface Task {
  readonly taskId: string;
  /** What the daemon calls the work, e.g. `ingest-filmstrip`. */
  readonly kind: string;
  /**
   * What the work publishes, e.g. `media.filmstrip.v1`.
   *
   * This is the one a screen wants. Kinds are contract names a renderer already
   * knows — they are what the read allowlist is written in — whereas the work's
   * own name is the daemon's business.
   */
  readonly outputKind: string;
  readonly state: TaskState;
  readonly attempt: number;
  readonly maxAttempts: number;
  readonly waitReason: string;
  /** Empty until the task publishes. */
  readonly outputArtifactId: string;
  readonly progress?: Progress;
}

export interface Job {
  readonly jobId: string;
  readonly projectId: string;
  readonly kind: string;
  readonly state: JobState;
  readonly createdUnixMillis: number;
  readonly updatedUnixMillis: number;
  readonly tasks: readonly Task[];
  readonly outputArtifactIds: readonly string[];
  readonly failureClass: FailureClass;
  readonly failureDetail: string;
}

/** One transition, as it happened. */
export interface TaskEvent {
  readonly eventId: number;
  readonly jobId: string;
  readonly taskId: string;
  readonly state: TaskState;
  readonly attempt: number;
  readonly waitReason: string;
  readonly failureClass: FailureClass;
  readonly atUnixMillis: number;
  readonly progress?: Progress;
}

/**
 * What this installation is using on disk.
 *
 * Three categories rather than one total, because the three are different
 * decisions: artifacts can be collected, weights should not be re-downloaded,
 * and state must be left alone.
 */
export interface StorageStats {
  readonly categories: readonly StorageCategory[];
  /**
   * Absent when the filesystem would not say — which is not the same as zero.
   * A screen must not render "0 B free" for a question that went unanswered.
   */
  readonly availableBytes?: number;
}

export interface StorageCategory {
  /** `artifacts`, `models`, or `state`. */
  readonly key: string;
  readonly bytes: number;
  readonly items: number;
}

/** One published document, still as text. */
export interface Document {
  readonly artifactId: string;
  readonly kind: string;
  readonly json: string;
}

interface TauriCore {
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
}

interface TauriEvent {
  listen<T>(event: string, handler: (payload: { payload: T }) => void): Promise<() => void>;
}

export function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

async function core(): Promise<TauriCore> {
  return (await import('@tauri-apps/api/core')) as unknown as TauriCore;
}

async function events(): Promise<TauriEvent> {
  return (await import('@tauri-apps/api/event')) as unknown as TauriEvent;
}

const NOT_IN_SHELL = {
  status: 'disconnected',
  reason: 'Not running inside the ClipMill desktop shell.',
} as const satisfies ConnectionState;

export async function fetchDaemonState(): Promise<ConnectionState> {
  if (!isTauri()) {
    return NOT_IN_SHELL;
  }
  const { invoke } = await core();
  return invoke<ConnectionState>('daemon_state');
}

export async function reconnectDaemon(): Promise<ConnectionState> {
  if (!isTauri()) {
    return NOT_IN_SHELL;
  }
  const { invoke } = await core();
  return invoke<ConnectionState>('reconnect_daemon');
}

/**
 * The host hands back the profile document verbatim; parsing it here keeps the
 * JSON Schema the only contract between daemon and renderer.
 */
export async function fetchDeviceProfile(remeasure = false): Promise<DeviceProfileResult> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  const raw = await invoke<{ artifactId: string; profileJson: string }>('device_profile', {
    remeasure,
  });
  return {
    artifactId: raw.artifactId,
    profile: JSON.parse(raw.profileJson) as DeviceProfile,
  };
}

export async function subscribeDaemonState(
  handler: (state: ConnectionState) => void,
): Promise<() => void> {
  if (!isTauri()) {
    return () => undefined;
  }
  const { listen } = await events();
  return listen<ConnectionState>(STATE_EVENT, (event) => {
    handler(event.payload);
  });
}

export async function listProjects(): Promise<readonly Project[]> {
  if (!isTauri()) {
    return [];
  }
  const { invoke } = await core();
  return invoke<Project[]>('list_projects');
}

export async function createProject(name: string): Promise<string> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<string>('create_project', { name });
}

export async function listSources(projectId: string): Promise<readonly Source[]> {
  if (!isTauri()) {
    return [];
  }
  const { invoke } = await core();
  return invoke<Source[]>('list_sources', { projectId });
}

export async function listJobs(projectId: string): Promise<readonly Job[]> {
  if (!isTauri()) {
    return [];
  }
  const { invoke } = await core();
  return invoke<Job[]>('list_jobs', { projectId });
}

export async function fetchJob(jobId: string): Promise<Job> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<Job>('get_job', { jobId });
}

export async function fetchStorageStats(): Promise<StorageStats> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<StorageStats>('storage_stats');
}

/**
 * One published document, whole.
 *
 * The kind comes back beside the text so a caller can refuse a document it did
 * not ask for instead of parsing it and finding out. Parsing is the caller's:
 * it knows which generated schema type it wants.
 */
export async function readDocument(projectId: string, artifactId: string): Promise<Document> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<Document>('read_document', { projectId, artifactId });
}

/**
 * A URL the WebView can load media from.
 *
 * Built here rather than by each screen so the shape of the scheme lives in one
 * place. Nothing is fetched: the host answers this URL, and the daemon decides
 * whether it may.
 */
export function mediaUrl(projectId: string, artifactId: string, file: string): string {
  const path = `${encodeURIComponent(projectId)}/${encodeURIComponent(artifactId)}/${file
    .split('/')
    .map(encodeURIComponent)
    .join('/')}`;
  // Windows serves custom schemes over a localhost origin; the other platforms
  // use the scheme directly. Both are in the CSP.
  return navigator.userAgent.includes('Windows')
    ? `http://clipmill-media.localhost/${path}`
    : `clipmill-media://localhost/${path}`;
}

/**
 * Follow task transitions.
 *
 * The host owns the subscription and its replay cursor, so a screen that mounts
 * late sees events from that moment on and asks the daemon for current job state
 * separately. It never has to reason about reconnects.
 */
export async function subscribeTaskEvents(
  handler: (event: TaskEvent) => void,
): Promise<() => void> {
  if (!isTauri()) {
    return () => undefined;
  }
  const { listen } = await events();
  return listen<TaskEvent>(TASK_EVENT, (event) => {
    handler(event.payload);
  });
}
