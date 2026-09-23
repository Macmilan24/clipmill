/**
 * The model library, read from the daemon and kept current while it works.
 *
 * Progress is the daemon's, so it is read rather than estimated: while any
 * download or check is in flight the library is asked again every second, and
 * once nothing is moving it is left alone. Every action answers with the whole
 * library, which replaces what is held — a click never has to be reconciled
 * with a guess about what it did.
 */
import { useCallback, useEffect, useState } from 'react';

import type { ModelLibrary, ModelLibraryApi } from '../daemon/models.js';

/** How often the library is read while something is downloading. */
const POLL_MILLIS = 1_000;

export interface ModelLibraryState {
  readonly library: ModelLibrary | null;
  /** Why the library could not be read, when it could not. */
  readonly problem: string | null;
  /** The action in flight, as the key it was started with. */
  readonly pending: string | null;
  /** Why the last action was refused, in the daemon's words. */
  readonly error: string | null;
  readonly refresh: () => void;
  readonly dismissError: () => void;
  /** Run one action; resolves false when the daemon refused it. */
  readonly run: (key: string, action: () => Promise<ModelLibrary>) => Promise<boolean>;
}

export function isBusy(library: ModelLibrary | null): boolean {
  return (
    library?.models.some(
      (model) =>
        model.download !== undefined &&
        ['queued', 'downloading', 'verifying'].includes(model.download.state),
    ) ?? false
  );
}

export function useModelLibrary(api: ModelLibraryApi): ModelLibraryState {
  const [library, setLibrary] = useState<ModelLibrary | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  const [pending, setPending] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [generation, setGeneration] = useState(0);
  const refresh = useCallback(() => setGeneration((value) => value + 1), []);
  const busy = isBusy(library);

  useEffect(() => {
    let live = true;
    void (async () => {
      try {
        const next = await api.listModels();
        if (live) {
          setLibrary(next);
          setProblem(null);
        }
      } catch (cause) {
        if (live) setProblem(reasonOf(cause));
      }
    })();
    return () => {
      live = false;
    };
  }, [api, generation]);

  useEffect(() => {
    if (!busy) return undefined;
    const timer = setInterval(refresh, POLL_MILLIS);
    return () => clearInterval(timer);
  }, [busy, refresh]);

  const run = useCallback(async (key: string, action: () => Promise<ModelLibrary>) => {
    setPending(key);
    setError(null);
    try {
      setLibrary(await action());
      setProblem(null);
      return true;
    } catch (cause) {
      setError(reasonOf(cause));
      return false;
    } finally {
      setPending(null);
    }
  }, []);

  return {
    library,
    problem,
    pending,
    error,
    refresh,
    dismissError: () => setError(null),
    run,
  };
}

export function reasonOf(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}
