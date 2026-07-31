/**
 * One run, followed.
 *
 * The job is fetched once and then re-fetched whenever a transition for it
 * arrives. The events themselves are also kept, because they are the live log:
 * the daemon's own record of what moved and when, which is a truer log than
 * anything this screen could compose.
 *
 * The log starts when the screen opens. The host holds one subscription for the
 * whole application and replays from its own cursor across reconnects, so what
 * reaches a screen mounted later is the transitions from that moment on. Saying
 * "this session" is honest; pretending to have the history would not be.
 */
import { useCallback, useEffect, useMemo, useState } from 'react';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import type { Job, Source, TaskEvent } from '../daemon/client.js';
import { subscribeTaskEvents } from '../daemon/client.js';
import type { SourceMap } from '@clipmill/contracts';
import { LibraryLoader } from '../library/loader.js';

/** How many transitions the log keeps. Older ones scroll out of usefulness. */
const LOG_LIMIT = 200;

export interface AnalysisData {
  readonly job: Job | null;
  readonly projectName: string;
  readonly source: Source | null;
  readonly sourceMap: SourceMap | null;
  readonly thumbnail: string | null;
  readonly events: readonly TaskEvent[];
  readonly error: string | null;
  readonly loading: boolean;
}

/**
 * Everything the Analysis screen needs, gathered once.
 *
 * The source, its probe and its filmstrip are exactly what the Library gathers
 * for a card, so the same loader answers for them rather than a second copy of
 * the same four calls.
 */
export class AnalysisLoader {
  private readonly library: LibraryLoader;

  constructor(private readonly api: ShellApi = daemonApi) {
    this.library = new LibraryLoader(api);
  }

  async load(
    projectId: string,
    jobId: string,
  ): Promise<{
    job: Job;
    projectName: string;
    source: Source | null;
    sourceMap: SourceMap | null;
    thumbnail: string | null;
  }> {
    const [job, projects, sources] = await Promise.all([
      this.api.fetchJob(jobId),
      this.api.listProjects().catch(() => []),
      this.api.listSources(projectId).catch(() => []),
    ]);
    const source = sources[0] ?? null;
    const [sourceMap, thumbnail] = await Promise.all([
      this.library.readSourceMap(projectId, source),
      this.library.readThumbnail(projectId, job),
    ]);
    return {
      job,
      projectName: projects.find((entry) => entry.projectId === projectId)?.name ?? 'this project',
      source,
      sourceMap,
      thumbnail,
    };
  }

  refresh(jobId: string): Promise<Job> {
    return this.api.fetchJob(jobId);
  }
}

export function useAnalysis(
  projectId: string,
  jobId: string,
  provided?: AnalysisLoader,
): AnalysisData {
  // Held across renders: the effects below list it as a dependency, and a loader
  // rebuilt every render would fetch forever.
  const loader = useMemo(() => provided ?? new AnalysisLoader(), [provided]);

  const [job, setJob] = useState<Job | null>(null);
  const [projectName, setProjectName] = useState('this project');
  const [source, setSource] = useState<Source | null>(null);
  const [sourceMap, setSourceMap] = useState<SourceMap | null>(null);
  const [thumbnail, setThumbnail] = useState<string | null>(null);
  const [events, setEvents] = useState<readonly TaskEvent[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let live = true;
    setLoading(true);
    void loader
      .load(projectId, jobId)
      .then((snapshot) => {
        if (live) {
          setJob(snapshot.job);
          setProjectName(snapshot.projectName);
          setSource(snapshot.source);
          setSourceMap(snapshot.sourceMap);
          setThumbnail(snapshot.thumbnail);
          setError(null);
        }
      })
      .catch((cause: unknown) => {
        if (live) {
          setError(cause instanceof Error ? cause.message : String(cause));
        }
      })
      .finally(() => {
        if (live) {
          setLoading(false);
        }
      });
    return () => {
      live = false;
    };
  }, [loader, projectId, jobId]);

  const refresh = useCallback(() => {
    void loader
      .refresh(jobId)
      .then(setJob)
      .catch(() => undefined);
  }, [loader, jobId]);

  useEffect(() => {
    let live = true;
    const pendingUnlisten = subscribeTaskEvents((event) => {
      if (!live || event.jobId !== jobId) {
        return;
      }
      setEvents((current) => [...current, event].slice(-LOG_LIMIT));
      refresh();
    });
    return () => {
      live = false;
      void pendingUnlisten.then((unlisten) => {
        unlisten();
      });
    };
  }, [jobId, refresh]);

  return { job, projectName, source, sourceMap, thumbnail, events, error, loading };
}
