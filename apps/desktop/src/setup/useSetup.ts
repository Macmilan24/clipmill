/**
 * First-run setup, read from the daemon and kept current while it works.
 *
 * A packaged ClipMill arrives without its components (the workers that run
 * the models) and without model weights. Setup is both, started together by
 * one click: the daemon installs the components and downloads the
 * recommended models side by side, and this reads how far each has got,
 * every second while anything is moving and not at all once nothing is.
 * A development checkout reports its components as unmanaged, and then there
 * is nothing to set up here.
 */
import { useCallback, useEffect, useState } from 'react';

import {
  type Component,
  type Components,
  type ComponentsApi,
  componentDownloadBytes,
  componentsNeeded,
  installingComponents,
} from '../daemon/components.js';
import type { LibraryModel, ModelLibrary, ModelLibraryApi } from '../daemon/models.js';
import { isBusy } from '../models/useModelLibrary.js';

/** How often setup is read while something is installing or downloading. */
const POLL_MILLIS = 1_000;

export interface SetupState {
  readonly components: Components | null;
  readonly library: ModelLibrary | null;
  /** Why setup could not be read, when it could not. */
  readonly problem: string | null;
  /** Why the last action was refused, in the daemon's words. */
  readonly error: string | null;
  /** An action is on its way to the daemon. */
  readonly pending: boolean;
  /** Something is installing or downloading now. */
  readonly busy: boolean;
  /** This is a packaged app with something left to install or download. */
  readonly needed: boolean;
  /** Components that still need installing, updating or another try. */
  readonly parts: readonly Component[];
  /** Recommended models not yet here. */
  readonly models: readonly LibraryModel[];
  /** What finishing setup downloads, as far as it is known. */
  readonly downloadBytes: number;
  readonly start: () => void;
  readonly cancel: () => void;
  readonly refresh: () => void;
}

export function useSetup(api: ComponentsApi & ModelLibraryApi): SetupState {
  const [components, setComponents] = useState<Components | null>(null);
  const [library, setLibrary] = useState<ModelLibrary | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [generation, setGeneration] = useState(0);
  const refresh = useCallback(() => setGeneration((value) => value + 1), []);

  useEffect(() => {
    let live = true;
    void (async () => {
      try {
        const next = await api.listComponents();
        // The library matters only where setup does.
        const models = next.managed ? await api.listModels() : null;
        if (!live) return;
        setComponents(next);
        setLibrary(models);
        setProblem(null);
      } catch (cause) {
        if (live) setProblem(reasonOf(cause));
      }
    })();
    return () => {
      live = false;
    };
  }, [api, generation]);

  const busy = installingComponents(components) || isBusy(library);
  useEffect(() => {
    if (!busy) return undefined;
    const timer = setInterval(refresh, POLL_MILLIS);
    return () => clearInterval(timer);
  }, [busy, refresh]);

  const parts = componentsNeeded(components);
  const models = recommendedMissing(library);
  const modelBytes = models.reduce((total, model) => total + model.downloadBytes, 0);
  const managed = components?.managed === true;

  const act = useCallback(
    async (action: () => Promise<void>) => {
      setPending(true);
      setError(null);
      try {
        await action();
      } catch (cause) {
        setError(reasonOf(cause));
      } finally {
        setPending(false);
        refresh();
      }
    },
    [refresh],
  );

  const start = useCallback(() => {
    void act(async () => {
      // Side by side: the daemon runs component installs and model downloads
      // in separate queues, so neither waits for the other.
      if (parts.length > 0) setComponents(await api.installComponents([]));
      const names = models
        .filter(
          (model) =>
            model.download === undefined ||
            !['queued', 'downloading', 'verifying'].includes(model.download.state),
        )
        .map((model) => model.name);
      if (names.length > 0) setLibrary(await api.downloadModels(names));
    });
  }, [act, api, models, parts.length]);

  const cancel = useCallback(() => {
    void act(async () => {
      setComponents(await api.cancelComponentInstall());
      for (const model of models) {
        if (
          model.download !== undefined &&
          ['queued', 'downloading', 'verifying'].includes(model.download.state)
        ) {
          setLibrary(await api.cancelModelDownload(model.name));
        }
      }
    });
  }, [act, api, models]);

  return {
    components,
    library,
    problem,
    error,
    pending,
    busy,
    needed: managed && (parts.length > 0 || models.length > 0 || busy),
    parts,
    models,
    downloadBytes: componentDownloadBytes(parts) + modelBytes,
    start,
    cancel,
    refresh,
  };
}

/** The recommended models this computer does not have yet. */
function recommendedMissing(library: ModelLibrary | null): readonly LibraryModel[] {
  if (library === null) return [];
  return library.recommendedMissing
    .map((name) => library.models.find((model) => model.name === name))
    .filter((model): model is LibraryModel => model !== undefined);
}

function reasonOf(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}
