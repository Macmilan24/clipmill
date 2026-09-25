/**
 * Shared daemon interface for screen loaders.
 * The runtime implementation uses the Tauri bridge; tests can provide an
 * in-memory implementation through the same interface.
 */
import { modelLibraryApi, type ModelLibraryApi } from './models.js';
import { publishingApi, type PublishingApi } from './publishing.js';
import {
  type AnalyzeRequest,
  type ClipDecision,
  type ClipDecisionRecord,
  type AppliedCommand,
  type CropPath,
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
  solveCropPath,
  readDocument,
  registerSource,
  relinkSource,
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
  registerSource(projectId: string, absolutePath: string): Promise<RegisteredSource>;
  relinkSource(
    projectId: string,
    sourceId: string,
    absolutePath: string,
  ): Promise<RegisteredSource>;
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
  solveCropPath(
    projectId: string,
    faceTrackArtifactId: string,
    startTicks: number,
    endTicks: number,
  ): Promise<CropPath>;
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
  registerSource,
  relinkSource,
  getSource,
  startYoutubeImport,
  getYoutubeImport,
  listYoutubeImports,
  updateYoutubeImport,
  submitAnalyze,
  directClip,
  solveCropPath,
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
