/**
 * Settings' container: two reads, neither of which changes anything.
 *
 * They are fetched together and reported apart. A daemon that measures no
 * storage still has a policy, and a screen that treated one failure as both
 * would hide the half that answered.
 */
import { useEffect, useState } from 'react';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import type { LocalLock, StorageStats } from '../daemon/client.js';
import { Settings } from './Settings.js';

export interface SettingsScreenProps {
  readonly api?: ShellApi;
}

export function SettingsScreen({ api = daemonApi }: SettingsScreenProps) {
  const [storage, setStorage] = useState<StorageStats | null>(null);
  const [lock, setLock] = useState<LocalLock | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let live = true;
    void (async () => {
      const [measured, policy] = await Promise.allSettled([
        api.fetchStorageStats(),
        api.fetchLocalLock(),
      ]);
      if (!live) {
        return;
      }
      if (measured.status === 'fulfilled') {
        setStorage(measured.value);
      }
      if (policy.status === 'fulfilled') {
        setLock(policy.value);
      }
      // Only a failure of both is worth a banner. One of the two answering is
      // a screen that is half useful, which is better than a screen that says
      // nothing worked when something did.
      if (measured.status === 'rejected' && policy.status === 'rejected') {
        setError(reasonOf(measured.reason));
      }
      setLoading(false);
    })();
    return () => {
      live = false;
    };
  }, [api]);

  return <Settings storage={storage} lock={lock} loading={loading} error={error} />;
}

function reasonOf(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}
