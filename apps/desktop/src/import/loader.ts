/**
 * Choosing a file, and starting the run it becomes.
 *
 * Two sequences, both behind the same seam every other screen uses, so the whole
 * import can be exercised without a window or a daemon.
 *
 * The ordering is forced by the daemon's model rather than chosen: probing is
 * registering, registering needs a project, and a project needs a name — which
 * is why the name comes from the file rather than from a field. A project is
 * therefore created the moment a file is chosen, and reused if the choice
 * changes, so abandoning the screen leaves at most one empty project rather than
 * one per attempt. The Library lists it honestly as not analyzed.
 */
import type { SourceMap } from '@clipmill/contracts';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import type { Job, Source } from '../daemon/client.js';
import { LibraryLoader } from '../library/loader.js';
import { type ImportSettings, languageSubtag, projectNameFor, secondsToTicks } from './model.js';

export interface ChosenSource {
  readonly projectId: string;
  readonly source: Source;
  /** The probe. Null when the daemon served a document that would not parse. */
  readonly sourceMap: SourceMap | null;
  /** True when an unchanged file avoided a second probe. */
  readonly cached: boolean;
}

export class ImportLoader {
  private readonly library: LibraryLoader;

  constructor(private readonly api: ShellApi = daemonApi) {
    this.library = new LibraryLoader(api);
  }

  /** The native dialog. `null` when it was closed without choosing. */
  choose(): Promise<string | null> {
    return this.api.chooseSourceFile();
  }

  /**
   * Register a chosen file, which probes it.
   *
   * `existingProjectId` is passed back on a second choice so a person changing
   * their mind does not leave a project behind for every file they looked at.
   */
  async register(absolutePath: string, existingProjectId: string | null): Promise<ChosenSource> {
    const projectId =
      existingProjectId ?? (await this.api.createProject(projectNameFor(absolutePath)));
    const registered = await this.api.registerSource(projectId, absolutePath);
    const sourceMap = await this.library.readSourceMap(projectId, registered.source);
    return {
      projectId,
      source: registered.source,
      sourceMap,
      cached: registered.observationCacheHit,
    };
  }

  /** Start the analysis, in the units the contract keeps. */
  start(chosen: ChosenSource, settings: ImportSettings): Promise<Job> {
    return this.api.submitAnalyze(chosen.projectId, {
      sourceId: chosen.source.sourceId,
      language: languageSubtag(settings),
      minTicks: secondsToTicks(settings.minSeconds),
      maxTicks: secondsToTicks(settings.maxSeconds),
      count: settings.count,
    });
  }
}
