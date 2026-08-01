/**
 * Every way a screen reaches the daemon, as one interface.
 *
 * The client module is a set of functions bound to the Tauri bridge, which is
 * exactly what a screen wants at runtime and exactly what a test cannot supply.
 * Naming the surface makes the seam: a loader takes this, the real one is the
 * bridge, and a test passes a daemon that answers from memory.
 *
 * It is one interface rather than one per screen because the screens overlap —
 * the Library and an analysis in progress ask most of the same questions — and
 * two interfaces that drift apart would mean two fakes to keep honest.
 */
import {
  type AnalyzeRequest,
  type ClipDecision,
  type ClipDecisionRecord,
  type CropPath,
  type DirectClipInput,
  type DirectedClip,
  type Document,
  type Job,
  type MediaArtifact,
  type Project,
  type RegisteredSource,
  type Source,
  type StorageStats,
  chooseSourceFile,
  createProject,
  fetchJob,
  fetchStorageStats,
  listJobs,
  listProjects,
  listSources,
  mediaUrl,
  directClip,
  listClipDecisions,
  solveCropPath,
  readDocument,
  registerSource,
  setClipDecision,
  resolveMedia,
  submitAnalyze,
} from './client.js';

export interface ShellApi {
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
  chooseSourceFile(): Promise<string | null>;
  registerSource(projectId: string, absolutePath: string): Promise<RegisteredSource>;
  submitAnalyze(projectId: string, request: AnalyzeRequest): Promise<Job>;
  directClip(request: DirectClipInput): Promise<DirectedClip>;
  solveCropPath(
    projectId: string,
    faceTrackArtifactId: string,
    startTicks: number,
    endTicks: number,
  ): Promise<CropPath>;
  setClipDecision(
    projectId: string,
    sourceId: string,
    candidateId: string,
    decision: ClipDecision,
  ): Promise<ClipDecisionRecord>;
  listClipDecisions(projectId: string, sourceId: string): Promise<readonly ClipDecisionRecord[]>;
}

export const daemonApi: ShellApi = {
  listProjects,
  listJobs,
  fetchJob,
  listSources,
  readDocument,
  resolveMedia,
  mediaUrl,
  fetchStorageStats,
  createProject,
  chooseSourceFile,
  registerSource,
  submitAnalyze,
  directClip,
  solveCropPath,
  setClipDecision,
  listClipDecisions,
};
