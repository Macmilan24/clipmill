import { useCallback, useEffect, useState, type ReactNode } from 'react';
import type { Theme } from '@clipmill/tokens';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import type { LocalLock, StorageStats } from '../daemon/client.js';
import { Settings } from './Settings.js';

export interface SettingsScreenProps {
  readonly api?: ShellApi;
  readonly theme?: Theme;
  readonly onThemeChange?: (theme: Theme) => void;
  readonly integrations?: ReactNode;
}

/** Refresh independently: a failed storage read must not hide the privacy answer. */
export function SettingsScreen({ api = daemonApi, ...preferences }: SettingsScreenProps) {
  const [storage, setStorage] = useState<StorageStats | null>(null);
  const [lock, setLock] = useState<LocalLock | null>(null);
  const [loading, setLoading] = useState(true);
  const [storageError, setStorageError] = useState<string | null>(null);
  const [lockError, setLockError] = useState<string | null>(null);
  const [generation, setGeneration] = useState(0);
  const refresh = useCallback(() => setGeneration((value) => value + 1), []);

  useEffect(() => {
    let live = true;
    setLoading(true);
    void (async () => {
      const [measured, policy] = await Promise.allSettled([
        api.fetchStorageStats(),
        api.fetchLocalLock(),
      ]);
      if (!live) return;
      if (measured.status === 'fulfilled') {
        setStorage(measured.value);
        setStorageError(null);
      } else setStorageError(reasonOf(measured.reason));
      if (policy.status === 'fulfilled') {
        setLock(policy.value);
        setLockError(null);
      } else setLockError(reasonOf(policy.reason));
      setLoading(false);
    })();
    return () => {
      live = false;
    };
  }, [api, generation]);

  return (
    <Settings
      {...preferences}
      storage={storage}
      lock={lock}
      loading={loading}
      error={null}
      storageError={storageError}
      lockError={lockError}
      onRefresh={refresh}
    />
  );
}

function reasonOf(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}
