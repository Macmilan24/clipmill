/**
 * The Library's data, kept current without polling.
 *
 * The daemon already streams every task transition, so a list that re-fetched on
 * a timer would be both slower to react and busier at rest. An event says which
 * job moved; only that project is re-read.
 *
 * Two things make that work. Transitions arrive in bursts — a fan-out of eight
 * starts eight tasks at once — so ids are collected and flushed together, or one
 * ingest would fire eight identical refreshes. And the subscription reads the
 * current projects through a ref rather than depending on them, because an
 * effect that listed them would tear down and re-establish the subscription
 * every time one of them changed, which is every time an event arrives.
 */
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import { type ConnectionState, type StorageStats, subscribeTaskEvents } from '../daemon/client.js';
import { LibraryLoader } from './loader.js';
import type { LibraryProject } from './model.js';

/** How long transitions are gathered before the affected projects are re-read. */
const COALESCE_MILLIS = 300;

export interface LibraryData {
  readonly loading: boolean;
  readonly projects: readonly LibraryProject[];
  readonly storage: StorageStats | null;
  readonly error: string | null;
  readonly reload: () => void;
}

export function useLibrary(state: ConnectionState, provided?: LibraryLoader): LibraryData {
  // Held across renders on purpose. A loader constructed inline would be a new
  // object every render, and the effects below list it as a dependency — the
  // result would be a fetch loop that never settles.
  const loader = useMemo(() => provided ?? new LibraryLoader(), [provided]);
  const [projects, setProjects] = useState<readonly LibraryProject[]>([]);
  const [storage, setStorage] = useState<StorageStats | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [generation, setGeneration] = useState(0);

  const connected = state.status === 'connected';
  const reload = useCallback(() => {
    setGeneration((value) => value + 1);
  }, []);

  // What the subscription reads, so it never has to depend on the list itself.
  const latest = useRef<readonly LibraryProject[]>([]);
  useEffect(() => {
    latest.current = projects;
  }, [projects]);

  // The full gather: on connect, and whenever something asks for it again.
  useEffect(() => {
    if (!connected) {
      setLoading(false);
      return undefined;
    }
    let live = true;
    setLoading(true);
    void loader
      .load()
      .then((snapshot) => {
        if (live) {
          setProjects(snapshot.projects);
          setStorage(snapshot.storage);
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
  }, [connected, generation, loader]);

  useEffect(() => {
    if (!connected) {
      return undefined;
    }
    let live = true;
    const pending = new Set<string>();
    let timer: ReturnType<typeof setTimeout> | null = null;

    const refresh = (projectId: string): void => {
      const existing = latest.current.find((entry) => entry.project.projectId === projectId);
      if (existing === undefined) {
        return;
      }
      void loader.loadProject(existing.project).then((refreshed) => {
        if (live) {
          setProjects((current) =>
            current.map((entry) => (entry.project.projectId === projectId ? refreshed : entry)),
          );
        }
      });
    };

    const flush = (): void => {
      timer = null;
      const ids = [...pending];
      pending.clear();
      ids.forEach(refresh);
    };

    const pendingUnlisten = subscribeTaskEvents((event) => {
      const owner = latest.current.find((entry) => entry.job?.jobId === event.jobId);
      if (owner === undefined) {
        // A job this list has never seen: a project created elsewhere, or the
        // first job of one that had none. Only a full gather can place it.
        reload();
        return;
      }
      pending.add(owner.project.projectId);
      timer ??= setTimeout(flush, COALESCE_MILLIS);
    });

    return () => {
      live = false;
      if (timer !== null) {
        clearTimeout(timer);
      }
      void pendingUnlisten.then((unlisten) => {
        unlisten();
      });
    };
  }, [connected, loader, reload]);

  return { loading, projects, storage, error, reload };
}
