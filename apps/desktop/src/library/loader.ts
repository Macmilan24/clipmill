/**
 * Load each project's jobs, sources, probe, and filmstrip inventory through
 * `ShellApi`. Assemble the data here; derive presentation values in `model.ts`.
 */
import type { SourceMap } from '@clipmill/contracts';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import type { Job, Project, Source, StorageStats } from '../daemon/client.js';
import { type LibraryProject, newestAnalysis, publishedArtifact, readStatus } from './model.js';

export const FILMSTRIP_KIND = 'media.filmstrip.v1';
export const SOURCE_MAP_KIND = 'evidence.source_map.v1';

export interface LibrarySnapshot {
  readonly projects: readonly LibraryProject[];
  /** Null when the daemon would not say, which the strip reports as such. */
  readonly storage: StorageStats | null;
}

export class LibraryLoader {
  constructor(private readonly api: ShellApi = daemonApi) {}

  /**
   * Every project, gathered concurrently.
   *
   * One project failing does not lose the others: the daemon can refuse a single
   * artifact — a store that was garbage-collected, an object that failed
   * verification — and a Library that showed nothing because one thumbnail was
   * unreadable would be useless for the wrong reason.
   */
  async load(): Promise<LibrarySnapshot> {
    const [projects, storage] = await Promise.all([
      this.api.listProjects(),
      this.api.fetchStorageStats().catch(() => null),
    ]);
    const gathered = await Promise.all(projects.map((project) => this.loadProject(project)));
    return { projects: gathered, storage };
  }

  rename(projectId: string, name: string): Promise<Project> {
    return this.api.renameProject(projectId, name);
  }

  remove(projectId: string): Promise<void> {
    return this.api.deleteProject(projectId);
  }

  reveal(path: string): Promise<void> {
    return this.api.revealPath(path);
  }

  async relink(projectId: string, sourceId: string): Promise<boolean> {
    const chosen = await this.api.chooseSourceFile();
    if (chosen === null) return false;
    await this.api.relinkSource(projectId, sourceId, chosen);
    return true;
  }

  /** One project, whole. Used for the first load and for every refresh. */
  async loadProject(project: Project): Promise<LibraryProject> {
    const [jobs, sources] = await Promise.all([
      this.api.listJobs(project.projectId).catch(() => [] as readonly Job[]),
      this.api.listSources(project.projectId).catch(() => [] as readonly Source[]),
    ]);
    const job = newestAnalysis(jobs);
    const source = sources[0] ?? null;
    const [sourceMap, thumbnail] = await Promise.all([
      this.readSourceMap(project.projectId, source),
      this.readThumbnail(project.projectId, job),
    ]);
    return { project, job, source, sourceMap, thumbnail, status: readStatus(job) };
  }

  /**
   * The probe, which is where duration and resolution come from.
   *
   * The kind is checked rather than trusted. The daemon echoes what it served,
   * and parsing whatever arrived as a source map would turn a daemon-side
   * mix-up into a screen full of wrong numbers instead of an absent one.
   */
  async readSourceMap(projectId: string, source: Source | null): Promise<SourceMap | null> {
    if (source === null || source.sourceMapArtifactId === '') {
      return null;
    }
    try {
      const document = await this.api.readDocument(projectId, source.sourceMapArtifactId);
      return document.kind === SOURCE_MAP_KIND ? (JSON.parse(document.json) as SourceMap) : null;
    } catch {
      return null;
    }
  }

  /**
   * A frame from the middle of the filmstrip.
   *
   * The middle rather than the first, because the first frame of a recording is
   * so often black, a slate, or a fade-in — a thumbnail grid of black squares is
   * technically correct and completely useless.
   */
  async readThumbnail(projectId: string, job: Job | null): Promise<string | null> {
    const artifactId = publishedArtifact(job, FILMSTRIP_KIND);
    if (artifactId === null) {
      return null;
    }
    try {
      const media = await this.api.resolveMedia(projectId, artifactId);
      const tiles = media.files.filter((file) => file.mediaType.startsWith('image/'));
      const tile = tiles[Math.floor(tiles.length / 2)];
      return tile === undefined ? null : this.api.mediaUrl(projectId, artifactId, tile.path);
    } catch {
      return null;
    }
  }
}
