/**
 * Shared daemon interface for screen loaders.
 * The runtime implementation uses the Tauri bridge; tests can provide an
 * in-memory implementation through the same interface.
 */
import { modelLibraryApi, type ModelLibraryApi } from './models.js';
import { publishingApi, type PublishingApi } from './publishing.js';
import {
  type AnalyzeRequest,
  type Asset,
  type AssetLicense,
  type CaptionPreview,
  type EditHistoryEntry,
  type ClipDecision,
  type ClipDecisionRecord,
  type AppliedCommand,
  type CropPath,
  type FaceSighting,
  type SolveOptions,
  type EditCommandJson,
  type EditDocSummary,
  type PreviewPlan,
  type DirectClipInput,
  type DirectedClip,
  type Document,
  type Job,
  type MediaArtifact,
  type Project,
  type RegisteredSource,
  type Source,
  type SourceDetails,
  type YoutubeImport,
  type StorageStats,
  type ArchiveResult,
  type ExportPlan,
  type ExportBatch,
  type ExportRequest,
  type QueuedExport,
  type LocalLock,
  type Readiness,
  chooseExportFolder,
  chooseSourceFile,
  importAsset,
  listAssets,
  assetUrl,
  cancelJob,
  createProject,
  deleteProject,
  fetchJob,
  fetchStorageStats,
  listJobs,
  listProjects,
  listSources,
  mediaUrl,
  directClip,
  applyEditCommand,
  listClipDecisions,
  listEditDocs,
  getEditDoc,
  previewPlan,
  previewDirect,
  solveCropPath,
  listFaces,
  thumbnailFraming,
  readDocument,
  registerSource,
  relinkSource,
  previewCaptions,
  listEditHistory,
  captionFontUrl,
  emojiUrl,
  getSource,
  startYoutubeImport,
  getYoutubeImport,
  listYoutubeImports,
  updateYoutubeImport,
  setClipDecision,
  resolveMedia,
  submitAnalyze,
  planExport,
  exportClip,
  submitExportBatch,
  listExportBatches,
  updateExportBatchItem,
  exportArchive,
  fetchLocalLock,
  fetchReadiness,
  renameProject,
  revealPath,
} from './client.js';

export interface ShellApi extends PublishingApi, ModelLibraryApi {
  listProjects(): Promise<readonly Project[]>;
  listJobs(projectId: string): Promise<readonly Job[]>;
  fetchJob(jobId: string): Promise<Job>;
  listSources(projectId: string): Promise<readonly Source[]>;
  readDocument(projectId: string, artifactId: string): Promise<Document>;
  resolveMedia(projectId: string, artifactId: string): Promise<MediaArtifact>;
  /** Not a call: the URL the media protocol answers. */
  mediaUrl(projectId: string, artifactId: string, file: string): string;
  fetchStorageStats(): Promise<StorageStats>;
  createProject(name: string): Promise<string>;
  renameProject(projectId: string, name: string): Promise<Project>;
  deleteProject(projectId: string): Promise<void>;
  cancelJob(jobId: string): Promise<Job>;
  chooseSourceFile(): Promise<string | null>;
  /** Bring a picture or a sound in. Absent from shells without assets. */
  importAsset?(kind: Asset['kind'], license: AssetLicense): Promise<Asset | null>;
  /** Every asset of a kind. Absent from shells without assets. */
  listAssets?(kind: Asset['kind']): Promise<readonly Asset[]>;
  /** Not a call: the URL an asset loads from. */
  assetUrl?(hash: string): string;
  registerSource(projectId: string, absolutePath: string): Promise<RegisteredSource>;
  relinkSource(
    projectId: string,
    sourceId: string,
    absolutePath: string,
  ): Promise<RegisteredSource>;
  /** Captions under a look not chosen yet. Absent from shells without it. */
  previewCaptions?(docId: string, styleRef: string, optionsJson: string): Promise<CaptionPreview>;
  /** A document's whole edit history. Absent from shells without it. */
  listEditHistory?(docId: string): Promise<readonly EditHistoryEntry[]>;
  /** Where a pinned caption font is served from. Absent where none are. */
  captionFontUrl?(file: string): string;
  /** Where a pinned emoji's picture is served from. Absent where none are. */
  emojiUrl?(code: string): string;
  getSource(sourceId: string): Promise<SourceDetails>;
  startYoutubeImport(
    projectId: string,
    url: string,
    rightsConfirmed: boolean,
    maxHeight?: number,
  ): Promise<YoutubeImport>;
  getYoutubeImport(importId: string): Promise<YoutubeImport>;
  listYoutubeImports(projectId?: string): Promise<readonly YoutubeImport[]>;
  updateYoutubeImport(importId: string, action: 'cancel' | 'retry'): Promise<YoutubeImport>;
  submitAnalyze(projectId: string, request: AnalyzeRequest): Promise<Job>;
  directClip(request: DirectClipInput): Promise<DirectedClip>;
  /** The clip approving would build, not saved. Absent from shells without it. */
  previewDirect?(request: DirectClipInput): Promise<PreviewPlan>;
  solveCropPath(
    projectId: string,
    faceTrackArtifactId: string,
    startTicks: number,
    endTicks: number,
    options?: SolveOptions,
  ): Promise<CropPath>;
  /** Where the camera would point at each moment. Absent from shells without it. */
  thumbnailFraming?(
    projectId: string,
    faceTrackArtifactId: string,
    moments: readonly number[],
  ): Promise<readonly number[]>;
  /** The faces seen over a span. Absent from shells without it. */
  listFaces?(
    projectId: string,
    faceTrackArtifactId: string,
    startTicks: number,
    endTicks: number,
  ): Promise<readonly FaceSighting[]>;
  previewPlan(projectId: string, docId: string): Promise<PreviewPlan>;
  listEditDocs(projectId: string): Promise<readonly EditDocSummary[]>;
  getEditDoc?(docId: string): Promise<import('./client.js').EditDocDetail>;
  applyEditCommand(
    docId: string,
    expectedRevision: number,
    command: EditCommandJson,
  ): Promise<AppliedCommand>;
  /** `null` takes a decision back, leaving the clip undecided. */
  setClipDecision(
    projectId: string,
    sourceId: string,
    candidateId: string,
    decision: ClipDecision | null,
  ): Promise<ClipDecisionRecord>;
  listClipDecisions(projectId: string, sourceId: string): Promise<readonly ClipDecisionRecord[]>;
  planExport(request: ExportRequest): Promise<ExportPlan>;
  exportClip(request: ExportRequest): Promise<QueuedExport>;
  submitExportBatch(requests: readonly ExportRequest[]): Promise<ExportBatch>;
  listExportBatches(): Promise<readonly ExportBatch[]>;
  updateExportBatchItem(
    batchId: string,
    index: number,
    action: 'retry' | 'cancel',
  ): Promise<ExportBatch>;
  /** Show a delivered file in the file manager. Host-side, checked there. */
  revealPath(path: string): Promise<void>;
  exportArchive(projectId: string, destinationDir: string): Promise<ArchiveResult>;
  fetchLocalLock(): Promise<LocalLock>;
  /** Whether an analysis could run right now, stage by stage. */
  fetchReadiness(): Promise<Readiness>;
  chooseExportFolder(): Promise<string | null>;
}

export const daemonApi: ShellApi = {
  ...publishingApi,
  ...modelLibraryApi,
  listProjects,
  listJobs,
  fetchJob,
  listSources,
  readDocument,
  resolveMedia,
  mediaUrl,
  fetchStorageStats,
  createProject,
  renameProject,
  deleteProject,
  cancelJob,
  chooseSourceFile,
  importAsset,
  listAssets,
  assetUrl,
  registerSource,
  relinkSource,
  previewCaptions,
  listEditHistory,
  captionFontUrl,
  emojiUrl,
  getSource,
  startYoutubeImport,
  getYoutubeImport,
  listYoutubeImports,
  updateYoutubeImport,
  submitAnalyze,
  directClip,
  previewDirect,
  solveCropPath,
  listFaces,
  thumbnailFraming,
  previewPlan,
  listEditDocs,
  getEditDoc,
  applyEditCommand,
  setClipDecision,
  listClipDecisions,
  planExport,
  exportClip,
  submitExportBatch,
  listExportBatches,
  updateExportBatchItem,
  exportArchive,
  fetchLocalLock,
  fetchReadiness,
  chooseExportFolder,
  revealPath,
};
