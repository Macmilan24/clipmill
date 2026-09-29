import { useCallback, useEffect, useRef, useState } from 'react';

import { JobState, TaskState } from '@clipmill/contracts';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import { type ConnectionState, type Job, subscribeTaskEvents } from '../daemon/client.js';
import { EXPORT_JOB_KIND } from '../export/delivery.js';
import { ANALYZE_KIND } from '../library/model.js';
import { announceFinished } from './announce.js';

type ActivityApi = Pick<ShellApi, 'listProjects' | 'listJobs' | 'fetchJob'>;
type Subscribe = typeof subscribeTaskEvents;

/** The runs followed: analyses, and exports, which can take as long. */
const FOLLOWED = new Set([ANALYZE_KIND, EXPORT_JOB_KIND]);

function isProcessing(job: Job): boolean {
  return (
    job.kind === ANALYZE_KIND &&
    (job.state === JobState.PLANNED || job.state === JobState.RUNNING) &&
    job.tasks.some((task) => task.state === TaskState.ADMITTED || task.state === TaskState.RUNNING)
  );
}

function isUnfinished(job: Job): boolean {
  return job.state === JobState.PLANNED || job.state === JobState.RUNNING;
}

function isUnfinishedAnalysis(job: Job): boolean {
  return job.kind === ANALYZE_KIND && isUnfinished(job);
}

/** What a finished run says in the window title, or nothing to say. */
function finishedTitle(job: Job, project: string): string | null {
  const succeeded = job.state === JobState.SUCCEEDED;
  if (!succeeded && job.state !== JobState.FAILED) return null;
  if (job.kind === ANALYZE_KIND) {
    return succeeded ? `Clips ready · ${project}` : `Analysis stopped · ${project}`;
  }
  return succeeded ? `Export ready · ${project}` : `Export stopped · ${project}`;
}

export interface RunActivity {
  /** An analysis is doing work right now, for the brand mark. */
  readonly active: boolean;
  readonly markStarted: (jobId: string) => void;
  /** Analyses and exports not finished yet, oldest first. */
  readonly runs: readonly Job[];
  /** Each project's name, by its id. */
  readonly projectNames: ReadonlyMap<string, string>;
}

/**
 * Follow analyses and exports across every project, including runs resumed
 * after relaunch, and say so when one that was followed finishes.
 */
export function useAnalysisActivity(
  connection: ConnectionState,
  api: ActivityApi = daemonApi,
  subscribe: Subscribe = subscribeTaskEvents,
): RunActivity {
  const [active, setActive] = useState(false);
  const [runs, setRuns] = useState<readonly Job[]>([]);
  const [projectNames, setProjectNames] = useState<ReadonlyMap<string, string>>(new Map());
  const markStartedRef = useRef<(jobId: string) => void>(() => undefined);
  const markStarted = useCallback((jobId: string) => markStartedRef.current(jobId), []);
  const daemonId =
    connection.status === 'connected'
      ? `${connection.daemonVersion}:${connection.startedUnixMillis}`
      : null;

  useEffect(() => {
    setActive(false);
    setRuns([]);
    markStartedRef.current = () => undefined;
    if (daemonId === null) return;

    let live = true;
    let timer: ReturnType<typeof setTimeout> | null = null;
    const pending = new Set<string>();
    const justStarted = new Set<string>();
    const jobs = new Map<string, Job>();
    const unrelated = new Set<string>();
    const names = new Map<string, string>();

    const publish = () => {
      setActive(justStarted.size > 0 || [...jobs.values()].some(isProcessing));
      setRuns(
        [...jobs.values()]
          .filter(isUnfinished)
          .toSorted((a, b) => a.createdUnixMillis - b.createdUnixMillis),
      );
    };

    // Names for projects made since the first listing, when a run meets one.
    const soughtNames = new Set<string>();
    let naming = false;
    const learnNames = () => {
      if (naming) return;
      naming = true;
      void api
        .listProjects()
        .then((projects) => {
          for (const project of projects) names.set(project.projectId, project.name);
          if (live) setProjectNames(new Map(names));
        })
        .catch(() => undefined)
        .finally(() => {
          naming = false;
        });
    };

    const record = (job: Job) => {
      if (!live) return;
      if (!FOLLOWED.has(job.kind)) {
        unrelated.add(job.jobId);
        return;
      }
      if (!names.has(job.projectId) && !soughtNames.has(job.projectId)) {
        soughtNames.add(job.projectId);
        learnNames();
      }
      const previous = jobs.get(job.jobId);
      if (previous && previous.updatedUnixMillis > job.updatedUnixMillis) return;
      if (
        previous &&
        previous.updatedUnixMillis === job.updatedUnixMillis &&
        !isUnfinishedAnalysis(previous) &&
        isUnfinishedAnalysis(job)
      )
        return;
      if (!isUnfinished(job)) {
        justStarted.delete(job.jobId);
        if (previous && isUnfinished(previous)) {
          const title = finishedTitle(job, names.get(job.projectId) ?? 'ClipMill');
          if (title !== null) announceFinished(job.jobId, title);
        }
      }
      jobs.set(job.jobId, job);
      publish();
    };

    const refresh = (jobIds: readonly string[]) => {
      void Promise.all(
        jobIds.map((id) =>
          api
            .fetchJob(id)
            .then(record)
            .catch(() => undefined),
        ),
      );
    };
    markStartedRef.current = (jobId) => {
      if (!live) return;
      justStarted.add(jobId);
      setActive(true);
      refresh([jobId]);
    };
    const poll = setInterval(() => {
      const ids = new Set(justStarted);
      for (const job of jobs.values()) {
        if (isProcessing(job) || (job.kind === EXPORT_JOB_KIND && isUnfinished(job))) {
          ids.add(job.jobId);
        }
      }
      if (ids.size > 0) refresh([...ids]);
    }, 1500);
    const flush = () => {
      timer = null;
      const ids = [...pending];
      pending.clear();
      refresh(ids);
    };

    const pendingUnlisten = subscribe((event) => {
      if (!live || unrelated.has(event.jobId)) return;
      pending.add(event.jobId);
      timer ??= setTimeout(flush, 200);
    }).catch(() => () => undefined);

    void api
      .listProjects()
      .then(async (projects) => {
        for (const project of projects) names.set(project.projectId, project.name);
        if (live) setProjectNames(new Map(names));
        const lists = await Promise.all(
          projects.map((project) =>
            api.listJobs(project.projectId).catch(() => [] as readonly Job[]),
          ),
        );
        lists.flat().forEach(record);
      })
      .catch(() => undefined);

    return () => {
      live = false;
      markStartedRef.current = () => undefined;
      clearInterval(poll);
      if (timer !== null) clearTimeout(timer);
      void pendingUnlisten
        .then((unlisten) => {
          unlisten();
        })
        .catch(() => undefined);
    };
  }, [api, daemonId, subscribe]);

  return { active, markStarted, runs, projectNames };
}
