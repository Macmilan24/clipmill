/**
 * "A new version is out."
 *
 * Once a day, while the person leaves it on, the app asks the engine which
 * ClipMill release is newest; the engine asks GitHub, and the Local Lock
 * counts that as a network operation. The first question waits a day after
 * this installation first ran, since what was just installed is new. A
 * newer release shows as a notice that opens its page, and "not now" hides
 * that version. Nothing is downloaded or installed.
 */
import { useCallback, useEffect, useState, useSyncExternalStore } from 'react';

import { isTauri } from '../daemon/client.js';

export interface UpdateCheck {
  readonly currentVersion: string;
  readonly latestVersion: string;
  readonly newer: boolean;
}

export interface UpdatesApi {
  checkForUpdate(): Promise<UpdateCheck>;
  /** Opens that version's release page in the browser. */
  openReleasePage(version: string): Promise<void>;
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) throw new Error('Updates are checked in the desktop app.');
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(command, args);
}

export const updatesApi: UpdatesApi = {
  checkForUpdate: () => call('check_for_update'),
  openReleasePage: (version) => call('open_release_page', { version }),
};

export const CHECK_EVERY_MS = 24 * 60 * 60 * 1000;
const ENABLED_KEY = 'clipmill.updates.notify';
const MEMORY_KEY = 'clipmill.updates';
const listeners = new Set<() => void>();

/** What the last question found, kept across launches. */
export interface UpdateMemory {
  /** When the engine was last asked, or when this installation first ran. */
  readonly checkedAt: number;
  /** The newest release the last answer named. */
  readonly latest: string | null;
  /** The version the person said "not now" to. */
  readonly dismissed: string | null;
}

export function updatesEnabled(): boolean {
  try {
    return localStorage.getItem(ENABLED_KEY) !== 'off';
  } catch {
    return true;
  }
}

export function setUpdatesEnabled(enabled: boolean): void {
  try {
    localStorage.setItem(ENABLED_KEY, enabled ? 'on' : 'off');
  } catch {
    // This session keeps it through the listeners below.
  }
  for (const listener of listeners) listener();
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

/** Whether the notice is on, followed as Settings changes it. */
export function useUpdatesEnabled(): boolean {
  return useSyncExternalStore(subscribe, updatesEnabled, () => true);
}

export function recallUpdates(): UpdateMemory | null {
  try {
    const kept = JSON.parse(localStorage.getItem(MEMORY_KEY) ?? 'null') as UpdateMemory | null;
    return typeof kept?.checkedAt === 'number' ? kept : null;
  } catch {
    return null;
  }
}

function rememberUpdates(memory: UpdateMemory): void {
  try {
    localStorage.setItem(MEMORY_KEY, JSON.stringify(memory));
  } catch {
    // Asked again next launch, which is still at most once a session.
  }
}

const versionParts = (text: string) => text.split('.').map(Number);

/** `a` is a later release than `b`; both `major.minor.patch`. */
export function isNewer(a: string, b: string): boolean {
  const [left, right] = [versionParts(a), versionParts(b)];
  if (left.length !== 3 || right.length !== 3 || [...left, ...right].some(Number.isNaN)) {
    return false;
  }
  for (let index = 0; index < 3; index += 1) {
    if (left[index] !== right[index]) return left[index]! > right[index]!;
  }
  return false;
}

/** The release to tell the person about, if any. */
export function releaseToShow(memory: UpdateMemory | null, running: string): string | null {
  const latest = memory?.latest ?? null;
  return latest !== null && latest !== memory?.dismissed && isNewer(latest, running)
    ? latest
    : null;
}

export interface UpdateNotice {
  readonly version: string;
  readonly open: () => void;
  readonly dismiss: () => void;
}

/**
 * The newer release to show, asking at most once a day while `running`, the
 * connected engine's version, is known and the notice is on.
 */
export function useUpdateNotice(
  running: string | null,
  api: UpdatesApi = updatesApi,
  now: () => number = Date.now,
): UpdateNotice | null {
  const enabled = useUpdatesEnabled();
  const [memory, setMemory] = useState(recallUpdates);
  const keep = useCallback((next: UpdateMemory) => {
    rememberUpdates(next);
    setMemory(next);
  }, []);

  useEffect(() => {
    if (!enabled || running === null) return;
    const askIfDue = () => {
      const kept = recallUpdates();
      if (kept === null) {
        keep({ checkedAt: now(), latest: null, dismissed: null });
        return;
      }
      if (now() - kept.checkedAt < CHECK_EVERY_MS) return;
      // Asked, so written down, whatever comes back: a day before the next.
      const asked = { ...kept, checkedAt: now() };
      keep(asked);
      void api.checkForUpdate().then(
        (answer) => keep({ ...asked, latest: answer.latestVersion }),
        () => {},
      );
    };
    askIfDue();
    const hourly = window.setInterval(askIfDue, 60 * 60 * 1000);
    return () => window.clearInterval(hourly);
  }, [api, enabled, keep, now, running]);

  const version = enabled && running !== null ? releaseToShow(memory, running) : null;
  if (version === null) return null;
  return {
    version,
    open: () => {
      void api.openReleasePage(version).catch(() => {});
    },
    dismiss: () => keep({ ...memory!, dismissed: version }),
  };
}
