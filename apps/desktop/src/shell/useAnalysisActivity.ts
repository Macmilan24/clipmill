import { useCallback, useEffect, useRef, useState } from 'react';

import { JobState } from '@clipmill/contracts';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import { type ConnectionState, type Job, subscribeTaskEvents } from '../daemon/client.js';
import { ANALYZE_KIND } from '../library/model.js';

type ActivityApi = Pick<ShellApi, 'listProjects' | 'listJobs' | 'fetchJob'>;
type Subscribe = typeof subscribeTaskEvents;

function isProcessing(job: Job): boolean {
  return (
    job.kind === ANALYZE_KIND && (job.state === JobState.PLANNED || job.state === JobState.RUNNING)
  );
}

/** Follow analysis across every project, including runs resumed after relaunch. */
export function useAnalysisActivity(
  connection: ConnectionState,
  api: ActivityApi = daemonApi,
  subscribe: Subscribe = subscribeTaskEvents,
): { active: boolean; markStarted: (jobId: string) => void } {
  const [active, setActive] = useState(false);
  const markStartedRef = useRef<(jobId: string) => void>(() => undefined);
  const markStarted = useCallback((jobId: string) => markStartedRef.current(jobId), []);
  const daemonId =
    connection.status === 'connected'
      ? `${connection.daemonVersion}:${connection.startedUnixMillis}`
      : null;

  useEffect(() => {
    setActive(false);
    markStartedRef.current = () => undefined;
    if (daemonId === null) return;

    let live = true;
    let timer: ReturnType<typeof setTimeout> | null = null;
    const pending = new Set<string>();
    const justStarted = new Set<string>();
    const jobs = new Map<string, Job>();
    const unrelated = new Set<string>();

    const record = (job: Job) => {
      if (!live) return;
      if (job.kind !== ANALYZE_KIND) {
        unrelated.add(job.jobId);
        return;
      }
      const previous = jobs.get(job.jobId);
      if (previous && previous.updatedUnixMillis > job.updatedUnixMillis) return;
      if (
        previous &&
        previous.updatedUnixMillis === job.updatedUnixMillis &&
        !isProcessing(previous) &&
        isProcessing(job)
      )
        return;
      justStarted.delete(job.jobId);
      jobs.set(job.jobId, job);
      setActive(justStarted.size > 0 || [...jobs.values()].some(isProcessing));
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
      for (const job of jobs.values()) if (isProcessing(job)) ids.add(job.jobId);
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

  return { active, markStarted };
}
