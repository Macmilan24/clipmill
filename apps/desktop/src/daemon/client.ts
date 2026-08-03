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
 * What a media artifact holds.
 *
 * A screen needs the names before it can build a URL: a filmstrip's tiles are
 * named by whatever produced them, and guessing at the pattern would be the
 * renderer reimplementing a producer's convention. Nothing is fetched here — the
 * bytes arrive over `clipmill-media://`, and the daemon has already decided
 * whether this project may see them.
 */
export interface MediaArtifact {
  readonly artifactId: string;
  readonly kind: string;
  readonly files: readonly MediaFile[];
}

export interface MediaFile {
  /** Name inside the artifact, e.g. `proxy.mp4`. Never a filesystem path. */
  readonly path: string;
  readonly bytes: number;
  readonly mediaType: string;
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
  /** How long an unreferenced artifact is kept. Shown, not adjustable. */
  readonly retentionGraceSeconds: number;
}

export interface StorageCategory {
  /** `artifacts`, `models`, or `state`. */
  readonly key: string;
  readonly bytes: number;
  readonly items: number;
  /** Where it lives. A size a user cannot go and look at is not actionable. */
  readonly path: string;
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

/** A source the daemon registered, and whether it had to probe it again. */
export interface RegisteredSource {
  readonly source: Source;
  readonly observationCacheHit: boolean;
  /**
   * The probe, inline.
   *
   * Registering is what probes a file; the artifact carrying the result is not
   * published until the analysis runs. So there is nothing to read by address
   * between choosing a file and starting a run, and this is how a screen shows
   * a duration before asking anyone to commit to one.
   */
  readonly sourceMapJson: string;
}

/**
 * What starting an analysis asks for.
 *
 * Durations are ticks, which is the contract's unit. The screen that offers
 * "15 to 60 seconds" converts once, here, rather than leaving two ends to
 * convert separately and eventually differently.
 */
export interface AnalyzeRequest {
  readonly sourceId: string;
  /** BCP 47 primary subtag, or empty to let the recognizer decide. */
  readonly language: string;
  readonly minTicks: number;
  readonly maxTicks: number;
  /** Zero leaves the daemon's default. */
  readonly count: number;
}

/**
 * Ask the host to open a native file dialog.
 *
 * Nothing about this happens in the page. The WebView has no filesystem
 * capability and no permission to reach the dialog plugin; it can only ask the
 * host, and what comes back is one path a person chose in an operating-system
 * window. `null` means they closed it.
 */
export async function chooseSourceFile(): Promise<string | null> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<string | null>('choose_source_file');
}

/**
 * Register a local file as a project's source.
 *
 * This is also what probes it: the daemon reads the container and publishes a
 * source map, which is the only way to learn a file's duration and streams.
 */
export async function registerSource(
  projectId: string,
  absolutePath: string,
): Promise<RegisteredSource> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<RegisteredSource>('register_source', { projectId, absolutePath });
}

/** Start the analysis. The reply is the job, so a screen can watch it. */
export async function submitAnalyze(projectId: string, request: AnalyzeRequest): Promise<Job> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<Job>('submit_analyze', { projectId, request });
}

export async function resolveMedia(projectId: string, artifactId: string): Promise<MediaArtifact> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<MediaArtifact>('resolve_media', { projectId, artifactId });
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

/** What somebody decided about a clip. Three answers, and no fourth. */
export type ClipDecision = 'rejected' | 'kept' | 'approved';

export interface ClipDecisionRecord {
  readonly candidateId: string;
  readonly decision: ClipDecision | 'unspecified';
  readonly decidedUnixMillis: number;
}

/** Which cut the director should build from. */
export type ClipCut = 'chosen' | 'alternative' | 'exact';

export interface DirectClipInput {
  readonly projectId: string;
  readonly sourceId: string;
  readonly candidateId: string;
  readonly cut: ClipCut;
  readonly styleRef?: string;
  /** Read only for `exact`, and snapped to the lattice by the daemon. */
  readonly startTicks?: number;
  readonly endTicks?: number;
}

export interface DirectedClip {
  readonly docId: string;
  readonly revision: number;
  readonly documentJson: string;
  /**
   * Where the cut landed, which is not always where it was asked for: a
   * hand-set boundary is moved onto the lattice before anything is built.
   */
  readonly startTicks: number;
  readonly endTicks: number;
  readonly decisions: readonly string[];
}

export async function directClip(request: DirectClipInput): Promise<DirectedClip> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<DirectedClip>('direct_clip', { request });
}

export async function setClipDecision(
  projectId: string,
  sourceId: string,
  candidateId: string,
  decision: ClipDecision,
): Promise<ClipDecisionRecord> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<ClipDecisionRecord>('set_clip_decision', {
    projectId,
    sourceId,
    candidateId,
    decision,
  });
}

export async function listClipDecisions(
  projectId: string,
  sourceId: string,
): Promise<readonly ClipDecisionRecord[]> {
  if (!isTauri()) {
    return [];
  }
  const { invoke } = await core();
  return invoke<ClipDecisionRecord[]>('list_clip_decisions', { projectId, sourceId });
}

/** One point of a proposed crop path, normalized against the source frame. */
export interface CropKeyframe {
  readonly tTicks: number;
  readonly centerX: number;
  readonly centerY: number;
  /** Height of the crop as a share of the source height. */
  readonly scale: number;
}

export interface CropPath {
  readonly keyframes: readonly CropKeyframe[];
  /** True when nobody earned the frame and this is the fitted rectangle. */
  readonly fit: boolean;
  readonly fitReason: string;
  readonly containment: number;
}

/**
 * Where the camera should point over a span.
 *
 * A proposal: nothing is written, which is what makes it safe to ask again
 * every time somebody moves a boundary.
 */
export async function solveCropPath(
  projectId: string,
  faceTrackArtifactId: string,
  startTicks: number,
  endTicks: number,
): Promise<CropPath> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<CropPath>('solve_crop_path', {
    projectId,
    faceTrackArtifactId,
    startTicks,
    endTicks,
  });
}

/** One word of a caption, with how long it holds the highlight. */
export interface PreviewWord {
  readonly text: string;
  readonly holdCentis: number;
}

export interface PreviewCue {
  readonly cueId: string;
  readonly firstFrame: number;
  readonly endFrame: number;
  readonly region: string;
  readonly karaoke: boolean;
  readonly leadInCentis: number;
  /** Already broken. The player must not re-wrap. */
  readonly lines: readonly (readonly PreviewWord[])[];
}

export interface PreviewGain {
  readonly frame: number;
  readonly gainDb: number;
}

/**
 * What the player draws, computed by the code that renders.
 *
 * Nothing here is derived on this side. A crop is an integer rectangle for a
 * frame, a cue window is already in frames, lines are already broken and holds
 * are already in centiseconds — because a preview that worked any of that out
 * for itself would be a second implementation of the render's arithmetic.
 */
export interface PreviewPlan {
  readonly revision: number;
  readonly rateNum: number;
  readonly rateDen: number;
  readonly frameCount: number;
  /** `[x, y, width, height]` per frame, or null where the layout is fit. */
  readonly crops: readonly (readonly [number, number, number, number] | null)[];
  readonly cues: readonly PreviewCue[];
  readonly gain: readonly PreviewGain[];
  readonly width: number;
  readonly height: number;
}

export async function previewPlan(projectId: string, docId: string): Promise<PreviewPlan> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<PreviewPlan>('preview_plan', { projectId, docId });
}

/** One edit document, as a list shows it. */
export interface EditDocSummary {
  readonly docId: string;
  readonly projectId: string;
  readonly revision: number;
  readonly createdUnixMillis: number;
  readonly updatedUnixMillis: number;
}

export async function listEditDocs(projectId: string): Promise<readonly EditDocSummary[]> {
  if (!isTauri()) {
    return [];
  }
  const { invoke } = await core();
  return invoke<EditDocSummary[]>('list_edit_docs', { projectId });
}

/**
 * One edit command, in the shape the Edit IR deserializes.
 *
 * Typed loosely on purpose: the authority on what a command is lives in the
 * Rust crate, and a duplicate of that enum here would be a second definition
 * to keep in step. What this side guarantees is the tag, which is what the
 * daemon dispatches on.
 */
export interface EditCommandJson {
  readonly op: string;
  readonly [field: string]: unknown;
}

export interface AppliedCommand {
  readonly docId: string;
  readonly revision: number;
  /** The command that undoes this one. The daemon keeps no undo stack. */
  readonly inverseCommandJson: string;
}

export async function applyEditCommand(
  docId: string,
  expectedRevision: number,
  command: EditCommandJson,
): Promise<AppliedCommand> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<AppliedCommand>('apply_edit_command', {
    docId,
    expectedRevision,
    commandJson: JSON.stringify(command),
  });
}

/** What an export is being asked to do. */
export interface ExportRequest {
  readonly docId: string;
  readonly destinationDir: string;
  /** Tokens in braces. Empty takes the default. */
  readonly namingPattern?: string;
  readonly sourceAttestation?: string;
  readonly gatesPassed?: readonly string[];
  readonly aiAssistance?: readonly string[];
  /** One-based ordinal within this export, for the {index} token. */
  readonly index?: number;
  /** YYYY-MM-DD. Supplied here because the daemon's naming reads no clock. */
  readonly date?: string;
  readonly title?: string;
}

export interface ExportFinding {
  readonly code: string;
  readonly severity: 'blocking' | 'advisory';
  readonly detail: string;
}

/** What an export would do, answered without doing it. */
export interface ExportPlan {
  readonly passes: boolean;
  readonly findings: readonly ExportFinding[];
  /** The resolved filename stem — the naming preview, computed by the daemon. */
  readonly stem: string;
  readonly fileNames: readonly string[];
  readonly estimatedBytes: number;
  readonly availableBytes?: number;
}

export interface ArchiveResult {
  readonly path: string;
  readonly sha256: string;
  readonly bytes: number;
  readonly entryCount: number;
}

/** Whether this installation is offline, and the evidence for it. */
export interface LocalLock {
  readonly engaged: boolean;
  readonly stages: number;
  readonly networkAllowedStages: number;
  readonly egressAttempts: number;
}

/**
 * What an export would produce, without producing it.
 *
 * The stem and the file names come back from the daemon rather than being
 * computed here, because the code that answers is the code that names the
 * files. A preview assembled in the renderer would be a second implementation
 * of the pattern, and the two would drift.
 */
export async function planExport(request: ExportRequest): Promise<ExportPlan> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<ExportPlan>('plan_export', { request });
}

/** Perform an export. Answers with the job to watch, not the finished files. */
export async function exportClip(request: ExportRequest): Promise<string> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<string>('export_clip', { request });
}

/** Pack a project's work into a zip that outlives this application. */
export async function exportArchive(
  projectId: string,
  destinationDir: string,
): Promise<ArchiveResult> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<ArchiveResult>('export_archive', { projectId, destinationDir });
}

export async function fetchLocalLock(): Promise<LocalLock> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<LocalLock>('local_lock');
}

/** Ask the user where exports should land. Native dialog, host-side. */
export async function chooseExportFolder(): Promise<string | null> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<string | null>('choose_export_folder');
}
