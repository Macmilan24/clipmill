/**
 * Renderer access to daemon commands through the Rust host.
 * Outside Tauri, report the bridge as unavailable so the shell can still render.
 * Use generated enums for wire states and generated schema types for canonical
 * JSON documents.
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
  /**
   * The file is not where it was registered: moved, renamed, replaced or
   * deleted. Editing still works from the preview copies; exporting needs the
   * original, which "Locate recording…" relinks. Absent from older daemons.
   */
  readonly missing?: boolean;
}

export interface SourceDetails {
  readonly source: Source;
  readonly sourceMapJson: string;
}

export type YoutubeImportState =
  | 'queued'
  | 'downloading'
  | 'processing'
  | 'registering'
  | 'completed'
  | 'failed'
  | 'cancelled'
  | 'interrupted';

/** Durable download state. A missing total is unknown, not zero percent. */
export interface YoutubeImport {
  readonly importId: string;
  readonly projectId: string;
  readonly canonicalUrl: string;
  readonly videoId: string;
  readonly title: string;
  readonly channel: string;
  /** Requested maximum download height; absent/zero in legacy records means 1080. */
  readonly maxHeight?: number;
  readonly state: YoutubeImportState;
  readonly attempt: number;
  readonly downloadedBytes: number;
  readonly totalBytes?: number;
  readonly sourceId: string;
  readonly errorCode: string;
  readonly error: string;
  readonly createdUnixMillis: number;
  readonly updatedUnixMillis: number;
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
  /** When it first ran and when it succeeded; zero, or absent, until it has. */
  readonly startedUnixMillis?: number;
  readonly finishedUnixMillis?: number;
}

export interface Job {
  readonly contentProfile?: string;
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
  /**
   * The recording the job ran over, or empty for a job not about one.
   *
   * A project holds any number of recordings, and an analysis is of exactly
   * one; a screen that could not tell which fell back to the newest job and
   * showed one recording's clips under another's name.
   */
  readonly sourceId: string;
  /**
   * What an export job is delivering, off its own payload; absent for every
   * other kind. An export is durable daemon state, and this is how the export
   * screen finds a document's export again after it was left or the
   * application relaunched — rather than in renderer state a remount lost.
   */
  readonly export?: ExportSummary;
  /** What an analysis was asked for, off its own payload; absent for other kinds. */
  readonly analysis?: AnalysisSettings;
}

/** The choices an analysis was started with, as its job reports them. */
export interface AnalysisSettings {
  readonly language: string;
  /** Zero where the run left the daemon's default. */
  readonly minTicks: number;
  readonly maxTicks: number;
  readonly count: number;
  readonly localEditorial: boolean;
}

/** The identity of an export, as its job carries it. */
export interface ExportSummary {
  readonly docId: string;
  /** The revision that was rendered. */
  readonly revision: number;
  /** The immutable edit.ir.v1 snapshot it was frozen as. */
  readonly irArtifactId: string;
  /** The folder, resolved, the files land in. */
  readonly destinationDir: string;
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
 * Categories separate decisions: artifacts can be collected, weights should
 * not be re-downloaded, and state and imported originals belong to projects.
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
  /**
   * What cleaning up unused generated files would free now. Absent when the
   * estimate could not be made; the clean-up re-checks everything anyway.
   */
  readonly reclaimableBytes?: number;
  readonly reclaimableItems?: number;
}

export interface StorageCategory {
  /** `artifacts`, `models`, `state`, `imports`, `backups` or `temporary`. */
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

/** Give a project a new name; the daemon answers with the project as it now stands. */
export async function renameProject(projectId: string, name: string): Promise<Project> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<Project>('rename_project', { projectId, name });
}

/** Delete a project with its sources, runs and edits. */
export async function deleteProject(projectId: string): Promise<void> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  await invoke<void>('delete_project', { projectId });
}

/** Stop a run. Tasks not started never start; running ones are asked to stop. */
export async function cancelJob(jobId: string): Promise<Job> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<Job>('cancel_job', { jobId });
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
  readonly localEditorial?: boolean;
  readonly contentProfile?: 'interview' | 'scripted';
  readonly cloudEditorial?: {
    readonly transcriptConsent: boolean;
    readonly budgetMicroUsd: number;
    readonly model: string;
  };
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

/** Reconnect a moved recording only when the daemon verifies its fingerprint. */
export async function relinkSource(
  projectId: string,
  sourceId: string,
  absolutePath: string,
): Promise<RegisteredSource> {
  if (!isTauri()) throw new Error(NOT_IN_SHELL.reason);
  const { invoke } = await core();
  return invoke<RegisteredSource>('relink_source', { projectId, sourceId, absolutePath });
}

export async function getSource(sourceId: string): Promise<SourceDetails> {
  if (!isTauri()) throw new Error(NOT_IN_SHELL.reason);
  const { invoke } = await core();
  return invoke<SourceDetails>('get_source', { sourceId });
}

export async function startYoutubeImport(
  projectId: string,
  url: string,
  rightsConfirmed: boolean,
  maxHeight = 1080,
): Promise<YoutubeImport> {
  if (!isTauri()) throw new Error(NOT_IN_SHELL.reason);
  const { invoke } = await core();
  return invoke<YoutubeImport>('start_youtube_import', {
    projectId,
    url,
    rightsConfirmed,
    maxHeight,
  });
}

export async function getYoutubeImport(importId: string): Promise<YoutubeImport> {
  if (!isTauri()) throw new Error(NOT_IN_SHELL.reason);
  const { invoke } = await core();
  return invoke<YoutubeImport>('get_youtube_import', { importId });
}

export async function listYoutubeImports(projectId = ''): Promise<readonly YoutubeImport[]> {
  if (!isTauri()) throw new Error(NOT_IN_SHELL.reason);
  const { invoke } = await core();
  return invoke<YoutubeImport[]>('list_youtube_imports', { projectId });
}

export async function updateYoutubeImport(
  importId: string,
  action: 'cancel' | 'retry',
): Promise<YoutubeImport> {
  if (!isTauri()) throw new Error(NOT_IN_SHELL.reason);
  const { invoke } = await core();
  return invoke<YoutubeImport>('update_youtube_import', { importId, action });
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

/** One of the person's own pictures or sounds, kept by its content hash. */
export interface Asset {
  readonly hash: string;
  readonly kind: 'image' | 'audio';
  /** The file name it was brought in from. */
  readonly name: string;
  readonly mediaType: string;
  readonly bytes: number;
  /** A picture's size; zero for a sound. */
  readonly width: number;
  readonly height: number;
  /** A sound's length; zero for a picture. */
  readonly durationTicks: number;
  readonly license: AssetLicense;
  readonly addedUnixMillis: number;
}

export type AssetLicense = 'own_content' | 'licensed' | 'royalty_free' | 'public_domain';

/**
 * Bring a picture or a sound in. The host opens the picker; the page names
 * no path. `null` when the person closed the picker.
 */
export async function importAsset(
  kind: Asset['kind'],
  license: AssetLicense,
): Promise<Asset | null> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<Asset | null>('import_asset', { kind, license });
}

/** Every asset of a kind, newest first. */
export async function listAssets(kind: Asset['kind']): Promise<readonly Asset[]> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<readonly Asset[]>('list_assets', { kind });
}

/** Where the page loads an asset from, by its hash. */
export function assetUrl(hash: string): string {
  const path = `assets/${encodeURIComponent(hash.replace(/^sha256:/, ''))}`;
  return navigator.userAgent.includes('Windows')
    ? `http://clipmill-media.localhost/${path}`
    : `clipmill-media://localhost/${path}`;
}

/** Where the editor loads a pinned emoji's picture from, by its code. */
export function emojiUrl(code: string): string {
  const path = `emoji/${encodeURIComponent(code)}.png`;
  return navigator.userAgent.includes('Windows')
    ? `http://clipmill-media.localhost/${path}`
    : `clipmill-media://localhost/${path}`;
}

/** Where the player loads a pinned caption font from. */
export function captionFontUrl(file: string): string {
  const path = `fonts/${encodeURIComponent(file)}`;
  return navigator.userAgent.includes('Windows')
    ? `http://clipmill-media.localhost/${path}`
    : `clipmill-media://localhost/${path}`;
}

/** Captions drawn under a look that has not been chosen yet. */
export interface CaptionPreview {
  readonly ass: string;
  readonly revision: number;
}

/**
 * The burned-in captions a document would have under another look, computed
 * by the render code and not saved: trying a look is not an edit.
 */
export async function previewCaptions(
  docId: string,
  styleRef: string,
  optionsJson: string,
): Promise<CaptionPreview> {
  if (!isTauri()) throw new Error(NOT_IN_SHELL.reason);
  const { invoke } = await core();
  return invoke<CaptionPreview>('preview_captions', { docId, styleRef, optionsJson });
}

/** One logged edit, with the command that undoes it. */
export interface EditHistoryEntry {
  readonly revision: number;
  readonly commandJson: string;
  readonly inverseJson: string;
  readonly appliedUnixMillis: number;
}

/** Every command a document has had, oldest first. */
export async function listEditHistory(docId: string): Promise<readonly EditHistoryEntry[]> {
  if (!isTauri()) throw new Error(NOT_IN_SHELL.reason);
  const { invoke } = await core();
  return invoke<readonly EditHistoryEntry[]>('list_edit_history', { docId });
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
  readonly highlightSpokenWord?: boolean;
  /** The caption options a saved style starts the clip with, as JSON. */
  readonly captionOptionsJson?: string;
  /** The brand a saved kit starts the clip with, as JSON. */
  readonly brandJson?: string;
  /** The frame the clip is framed for; absent is vertical. */
  readonly shape?: 'vertical' | 'portrait' | 'square' | 'landscape';
  /**
   * Read only for `exact`. Any edge between two words is kept as sent; one
   * that falls inside a word is moved out to keep the whole word (R63).
   */
  readonly startTicks?: number;
  readonly endTicks?: number;
  /**
   * Build a second document beside the one this candidate already has.
   *
   * Left off, the daemon reopens the existing document — edits and all —
   * which is what approving a clip twice should mean. Taking a different cut
   * of a clip that already has an edit is a variation, and says so.
   */
  readonly variation?: boolean;
  /**
   * Record the approval in the same write as the document, so a clip is
   * never approved without an edit to open or edited without being approved.
   */
  readonly approve?: boolean;
  readonly allowDeclined?: boolean;
  readonly manualSpan?: boolean;
  /**
   * The analysis run the candidate belongs to. The director reads every stage
   * from this run, as one snapshot; a re-analysis that renumbered the
   * candidates, or one still half published, cannot be mixed into a clip
   * chosen from another. Left off, the newest run over the source.
   */
  readonly jobId?: string;
}

export interface DirectedClip {
  readonly docId: string;
  readonly projectId: string;
  readonly sourceId: string;
  readonly candidateId: string;
  /** The run the document was cut from; empty when it was not recorded. */
  readonly jobId: string;
  readonly revision: number;
  readonly documentJson: string;
  /**
   * Where the cut landed, which is not always where it was asked for: a
   * hand-set edge inside a word is moved out to keep the word.
   * For a reopened document, where its segment stands now, trims included.
   */
  readonly startTicks: number;
  readonly endTicks: number;
  readonly decisions: readonly string[];
  /** True when the document already existed and came back as it stands. */
  readonly reopened: boolean;
}

export async function directClip(request: DirectClipInput): Promise<DirectedClip> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<DirectedClip>('direct_clip', { request });
}

/**
 * Record what somebody decided about a clip, or take it back.
 *
 * `null` is the taking back: the clip is undecided again. An edit that an
 * approval made is left where it is — undoing a verdict is not deleting work.
 */
export async function setClipDecision(
  projectId: string,
  sourceId: string,
  candidateId: string,
  decision: ClipDecision | null,
): Promise<ClipDecisionRecord> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<ClipDecisionRecord>('set_clip_decision', {
    projectId,
    sourceId,
    candidateId,
    decision: decision ?? 'unspecified',
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
  /** The face followed, when one was. Older hosts leave it out. */
  readonly trackId?: number | null;
  /** The lower portrait's path, for a two-person solve; empty otherwise. */
  readonly secondaryKeyframes?: readonly CropKeyframe[];
}

/** What else a solve may be asked for. */
export interface SolveOptions {
  /** Follow this face rather than the one the solver would choose. */
  readonly trackId?: number;
  /** Solve both portraits of a two-person layout. */
  readonly twoUp?: boolean;
  /** The clip's frame, whose shape the camera is fitted to. Absent is 9:16. */
  readonly aspect?: { readonly width: number; readonly height: number };
}

/** One face in one sampled frame, as shares of the source's display frame. */
export interface FaceSighting {
  readonly trackId: number;
  readonly tTicks: number;
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

/**
 * Where the camera would point at each moment, as the crop's centre across the
 * source frame (0..1): the middle where it would fit the whole frame.
 */
export async function thumbnailFraming(
  projectId: string,
  faceTrackArtifactId: string,
  moments: readonly number[],
): Promise<readonly number[]> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<number[]>('thumbnail_framing', { projectId, faceTrackArtifactId, moments });
}

/** The faces seen over a span, for picking who the camera follows. */
export async function listFaces(
  projectId: string,
  faceTrackArtifactId: string,
  startTicks: number,
  endTicks: number,
): Promise<readonly FaceSighting[]> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<FaceSighting[]>('list_faces', {
    projectId,
    faceTrackArtifactId,
    startTicks,
    endTicks,
  });
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
  options: SolveOptions = {},
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
    trackId: options.trackId ?? null,
    twoUp: options.twoUp ?? false,
    aspectWidth: options.aspect?.width ?? null,
    aspectHeight: options.aspect?.height ?? null,
  });
}

/** One word of a caption, with how long it holds the highlight. */
export interface PreviewWord {
  readonly text: string;
  readonly holdCentis: number;
  /**
   * The word's identity, shared with the same word in the reading cues. A
   * correction is addressed to this, so it lands in both presentations.
   * Empty only for a document that predates word identities.
   */
  readonly wordId: string;
}

/**
 * One segment of the program, and where in the source it plays.
 *
 * The numbers every seek, scrub and trim go through. A clip cut from ten
 * minutes into a recording starts at program frame zero and source tick
 * 54,000,000; a player that handed the media element program seconds seeked
 * to the recording's opening instead.
 */
export interface PreviewSegment {
  readonly segmentId: string;
  readonly sourceFingerprint: string;
  /** Source ticks the segment plays, half-open. */
  readonly inTicks: number;
  readonly outTicks: number;
  readonly programStartTicks: number;
  readonly hasTwoUpPaths?: boolean;
  readonly framingWarning?: string;
  /** Program frames the segment occupies, half-open. */
  readonly firstFrame: number;
  readonly endFrame: number;
  /** How the section is drawn. Older hosts leave it out. */
  readonly layout?: 'fit' | 'speaker_fill' | 'two_up' | 'picture_in_picture' | '';
  /** Two viewports: the upper one's height in output pixels. */
  readonly upperHeight?: number;
  /** Picture in picture: the inset's `[x, y, side]` in output pixels. */
  readonly inset?: readonly [number, number, number] | null;
  /** A fitted picture's fill: absent for the picture blurred. */
  readonly backgroundColour?: string | null;
  /** A fitted picture's zoom past fitting, in percent. */
  readonly zoomPercent?: number;
}

/** A source the program draws from: the frame the crops are measured in. */
export interface PreviewSource {
  readonly sourceFingerprint: string;
  readonly sourceId: string;
  readonly displayWidth: number;
  readonly displayHeight: number;
}

/** The proxy a source is previewed from. Proxy second zero is `coverageStartTicks`. */
export interface PreviewProxy {
  readonly sourceFingerprint: string;
  readonly artifactId: string;
  readonly file: string;
  readonly coverageStartTicks: number;
  readonly coverageEndTicks: number;
  readonly width: number;
  readonly height: number;
  readonly rateNum: number;
  readonly rateDen: number;
}

export interface PreviewCue {
  readonly cueId: string;
  /** `[x, y]` in thousandths of the frame, when the cue was placed by hand. */
  readonly position?: readonly [number, number] | null;
  /** Exact display bounds. Older hosts do not expose timing edits. */
  readonly startTicks?: number;
  readonly endTicks?: number;
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
export interface PreviewCaptionStyle {
  readonly styleRef: string;
  readonly fontFamily: string;
  readonly fontSize: number;
  readonly spoken: string;
  readonly unspoken: string;
  readonly outline: string;
  readonly shadow: string;
  readonly outlineWidth: number;
  readonly shadowDepth: number;
  readonly bold: boolean;
  readonly boxed: boolean;
  readonly marginHorizontal: number;
  readonly marginVertical: number;
  /** The colour key words are set in. Absent from older daemons. */
  readonly accent?: string;
  /** How the spoken word is marked: fill, word, box, pop or underline. */
  readonly highlight?: string;
}

/** One caption typeface, and whether this installation has its pinned file. */
export interface CaptionFont {
  readonly family: string;
  readonly label: string;
  readonly file: string;
  readonly installed: boolean;
}

/** A saved soft cut, allocated on the render's program frame grid. */
export interface PreviewTransition {
  readonly incomingSegmentId: string;
  /** The outgoing picture held while the incoming shot appears. */
  readonly outgoingFrame: number;
  /** Program frames affected by the blend, half-open. */
  readonly firstFrame: number;
  readonly endFrame: number;
}

/**
 * A text laid over the program, for the editor to show, select and move. Its
 * pixels come from the plan's script, which draws it as the render will.
 */
/** B-roll as the render lays it over the program's own picture. */
export interface PreviewCutaway {
  readonly cutawayId: string;
  readonly startTicks: number;
  readonly endTicks: number;
  readonly firstFrame: number;
  readonly endFrame: number;
  readonly fit: 'fill' | 'fit';
  readonly kind: 'picture' | 'footage';
  /** A picture's hash. */
  readonly asset?: string | null;
  /** Whether a picture moves closer, eight per cent by its last frame. */
  readonly pushIn: boolean;
  /** Footage's recording; its proxy is listed with the sections'. */
  readonly sourceFingerprint?: string | null;
  /** Where in its recording footage starts. */
  readonly inTicks: number;
}

export interface PreviewOverlay {
  readonly overlayId: string;
  /** Absent from plans made before emoji, which held texts only. */
  readonly kind?: 'text' | 'emoji';
  /** An emoji's code, as its picture is named. */
  readonly emoji?: string | null;
  readonly startTicks: number;
  readonly endTicks: number;
  readonly firstFrame: number;
  readonly endFrame: number;
  /** Empty for an emoji. */
  readonly text: string;
  readonly role: 'hook' | 'label';
  /** Its centre, per mille of the frame's width and height. */
  readonly x: number;
  readonly y: number;
  /**
   * A text's size at the 1920-pixel design height; an emoji's side, per mille
   * of the frame's short side.
   */
  readonly size: number;
  readonly colour: string;
  /** The plate behind it; absent draws an outline. */
  readonly plate?: string | null;
}

export interface PreviewPlan {
  readonly captionStyle?: PreviewCaptionStyle;
  /** B-roll over the program's own picture. Absent from older hosts. */
  readonly cutaways?: readonly PreviewCutaway[];
  /** Requested clip-wide blend duration; absent in older plans means off. */
  readonly transitionTicks?: number;
  /** Actual blends, shortened to fit each neighboring shot by the renderer. */
  readonly transitions?: readonly PreviewTransition[];
  readonly revision: number;
  readonly rateNum: number;
  readonly rateDen: number;
  readonly frameCount: number;
  /** `[x, y, width, height]` per frame, or null where the layout is fit. */
  readonly crops: readonly (readonly [number, number, number, number] | null)[];
  /** Lower viewport of a two-person composition, indexed like crops. */
  readonly secondaryCrops?: readonly (readonly [number, number, number, number] | null)[];
  readonly cues: readonly PreviewCue[];
  readonly readingCues?: readonly PreviewCue[];
  readonly readingMinDurationTicks?: number;
  readonly readingMinGapTicks?: number;
  /**
   * The burned-in captions exactly as the export writes them. A player that
   * runs libass draws the export's pixels from this. Absent from older daemons.
   */
  readonly ass?: string;
  /** Every caption typeface, with whether this installation has it. */
  readonly fonts?: readonly CaptionFont[];
  readonly gain: readonly PreviewGain[];
  readonly width: number;
  readonly height: number;
  /** The program's segments, in order, each mapped to its source. */
  readonly segments: readonly PreviewSegment[];
  /** Every source the segments name. */
  readonly sources: readonly PreviewSource[];
  /** The proxy for each source that has one. */
  readonly proxies: readonly PreviewProxy[];
  /**
   * Which of the document's cue lists `cues` came from. A cue-scoped command
   * names the list it means, because each list numbers its own cues.
   */
  readonly presentation: 'reading' | 'burn_in';
  /** Why the director built the clip as it did. Only a dry run carries it. */
  readonly decisions?: readonly string[];
  /** Titles and labels over the program, bottom first. Absent from older hosts. */
  readonly overlays?: readonly PreviewOverlay[];
  /** The music, and its level in decibels at frames, as the render mixes it. */
  readonly music?: {
    readonly asset: string;
    readonly offsetTicks: number;
    readonly levels: readonly { readonly frame: number; readonly gainDb: number }[];
  } | null;
  /** The logo where the render puts it, in output pixels. */
  readonly logo?: {
    readonly asset: string;
    readonly corner: 'top_left' | 'top_right' | 'bottom_left' | 'bottom_right';
    readonly side: number;
    readonly insetX: number;
    readonly insetY: number;
    readonly opacity: number;
  } | null;
  /** The progress bar, when the clip has one: thickness in output pixels. */
  readonly progress?: {
    readonly colour: string;
    readonly edge: 'top' | 'bottom';
    readonly thickness: number;
  } | null;
}

/**
 * Ask for attention after a long run, when the window is behind others: the
 * Dock icon bounces once. Nothing outside the shell.
 */
export async function requestAttention(): Promise<void> {
  if (!isTauri()) return;
  const { invoke } = await core();
  await invoke('request_attention');
}

/**
 * The clip approving would build, drawn as the Editor draws it — built by the
 * director and not saved. Nothing is written and nothing is decided.
 */
export async function previewDirect(request: DirectClipInput): Promise<PreviewPlan> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<PreviewPlan>('preview_direct', { request });
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
  /**
   * The source it was cut from and the candidate it was built for. Both empty
   * for a document handed in whole rather than directed from a clip.
   */
  readonly sourceId: string;
  readonly candidateId: string;
  /** The run it was cut from; empty when it was not recorded. */
  readonly jobId: string;
  readonly revision: number;
  readonly createdUnixMillis: number;
  readonly updatedUnixMillis: number;
  /** What somebody named the clip, when they did. */
  readonly title?: string | null;
}

export async function listEditDocs(projectId: string): Promise<readonly EditDocSummary[]> {
  if (!isTauri()) {
    return [];
  }
  const { invoke } = await core();
  return invoke<EditDocSummary[]>('list_edit_docs', { projectId });
}

export interface EditDocDetail {
  readonly docId: string;
  readonly revision: number;
  readonly documentJson: string;
}

/** The live document, used to show saved keyframes and caption options. */
export async function getEditDoc(docId: string): Promise<EditDocDetail> {
  if (!isTauri()) throw new Error(NOT_IN_SHELL.reason);
  const { invoke } = await core();
  return invoke<EditDocDetail>('get_edit_doc', { docId });
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
  /**
   * The revision the person reviewed. The daemon refuses to export any
   * other, so an edit that landed between the review and the click — another
   * window, a late command — is a conflict to re-check rather than a clip
   * nobody looked at.
   */
  readonly expectedRevision?: number;
  /** Frame rate and size of the delivered picture. Absent is the recording's rate at 1080p. */
  readonly format?: OutputFormat;
}

/**
 * The delivered picture. A zero numerator keeps the recording's own frame
 * rate, which is the default: converting 23.976 or 25 to 29.97 repeats frames.
 */
export interface OutputFormat {
  readonly frameRateNum: number;
  readonly frameRateDen: number;
  /** 1920 (1080p), 2560 (1440p) or 3840 (4K); zero is 1920. */
  readonly height: number;
}

export interface ExportFinding {
  readonly code: string;
  readonly severity: 'blocking' | 'advisory';
  readonly detail: string;
  /** The caption cue a `captions.*` finding is about. Absent for other checks. */
  readonly cueId?: string;
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
  /** The revision this plan was computed over — what is being reviewed. */
  readonly revision: number;
}

/** What an export froze when it was queued. */
export interface QueuedExport {
  /** The job to watch: its two tasks are the render and the delivery. */
  readonly jobId: string;
  /** The revision rendered, and the immutable snapshot it was frozen as. */
  readonly revision: number;
  readonly irArtifactId: string;
  /** The folder, resolved, the files land in. */
  readonly destinationDir: string;
}

export interface ArchiveResult {
  readonly path: string;
  readonly sha256: string;
  readonly bytes: number;
  readonly entryCount: number;
}

/** One stage an analysis plans, and whether what it needs is here. */
export interface StageReadiness {
  /** The task kind, e.g. `speech-asr`; what a job's task calls itself. */
  readonly stage: string;
  readonly capability: string;
  readonly implementation: string;
  readonly model: string;
  readonly backend: string;
  readonly modelPresent: boolean;
  readonly missingFiles: readonly string[];
  readonly workerPresent: boolean;
  readonly ready: boolean;
  /** What to do about it, when not ready: one sentence naming the command. */
  readonly remedy: string;
}

export interface WorkerPresence {
  readonly workerId: string;
  readonly family: string;
  readonly capabilities: readonly string[];
  readonly backend: string;
  readonly sinceUnixMillis: number;
}

/**
 * Whether an analysis could run right now.
 *
 * Asked before a run is submitted and again while a stage waits, because a
 * missing weight file or a worker fleet nobody started used to show as a
 * stage sitting planned forever with nothing to say.
 */
export interface Readiness {
  readonly ready: boolean;
  readonly decoderPresent: boolean;
  readonly decoderPath: string;
  readonly stages: readonly StageReadiness[];
  readonly workers: readonly WorkerPresence[];
}

export async function fetchReadiness(): Promise<Readiness> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<Readiness>('readiness');
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
export async function exportClip(request: ExportRequest): Promise<QueuedExport> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  return invoke<QueuedExport>('export_clip', { request });
}

/**
 * Show a delivered file in the operating system's file manager.
 *
 * The host checks the path names an existing file before anything is
 * spawned; nothing here can read, write or run it.
 */
export async function revealPath(path: string): Promise<void> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  await invoke<void>('reveal_path', { path });
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

/** Durable export intents; queued jobs retain their own execution state. */
export interface ExportBatchItem {
  readonly index: number;
  readonly projectId: string;
  readonly request: ExportRequest;
  readonly state: 'pending' | 'queued' | 'failed' | 'cancelled';
  readonly attempt: number;
  readonly queued?: QueuedExport;
  readonly error: string;
}
export interface ExportBatch {
  readonly batchId: string;
  readonly createdUnixMillis: number;
  readonly items: readonly ExportBatchItem[];
}
export async function submitExportBatch(requests: readonly ExportRequest[]): Promise<ExportBatch> {
  if (!isTauri()) throw new Error(NOT_IN_SHELL.reason);
  const { invoke } = await core();
  return invoke<ExportBatch>('submit_export_batch', { requests });
}
export async function listExportBatches(): Promise<readonly ExportBatch[]> {
  if (!isTauri()) throw new Error(NOT_IN_SHELL.reason);
  const { invoke } = await core();
  return invoke<ExportBatch[]>('list_export_batches');
}
export async function updateExportBatchItem(
  batchId: string,
  index: number,
  action: 'retry' | 'cancel',
): Promise<ExportBatch> {
  if (!isTauri()) throw new Error(NOT_IN_SHELL.reason);
  const { invoke } = await core();
  return invoke<ExportBatch>('update_export_batch_item', { batchId, index, action });
}
