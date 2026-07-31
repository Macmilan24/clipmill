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
  type Document,
  type Job,
  type MediaArtifact,
  type Project,
  type Source,
  type StorageStats,
  fetchJob,
  fetchStorageStats,
  listJobs,
  listProjects,
  listSources,
  mediaUrl,
  readDocument,
  resolveMedia,
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
};
