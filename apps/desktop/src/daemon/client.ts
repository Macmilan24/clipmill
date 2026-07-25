/**
 * The renderer's only way to reach the daemon.
 *
 * There is no socket, no fetch and no filesystem here — just three commands the
 * Rust host exposes. Outside a Tauri window (a plain `pnpm dev` browser tab, or
 * a test) the bridge reports itself unavailable instead of throwing, so the
 * shell still renders and says why it has no data.
 */
import type { DeviceProfile } from '@clipmill/contracts';

export type ConnectionState =
  | { readonly status: 'connecting' }
  | {
      readonly status: 'connected';
      readonly daemonVersion: string;
      readonly localLock: boolean;
      readonly startedUnixMillis: number;
    }
  | { readonly status: 'disconnected'; readonly reason: string };

export interface DeviceProfileResult {
  readonly artifactId: string;
  readonly profile: DeviceProfile;
}

/** Emitted by the host on every connection transition. */
const STATE_EVENT = 'daemon://state';

interface TauriCore {
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
}

interface TauriEvent {
  listen<T>(event: string, handler: (payload: { payload: T }) => void): Promise<() => void>;
}

export function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

async function core(): Promise<TauriCore> {
  return (await import('@tauri-apps/api/core')) as unknown as TauriCore;
}

async function events(): Promise<TauriEvent> {
  return (await import('@tauri-apps/api/event')) as unknown as TauriEvent;
}

const NOT_IN_SHELL = {
  status: 'disconnected',
  reason: 'Not running inside the ClipMill desktop shell.',
} as const satisfies ConnectionState;

export async function fetchDaemonState(): Promise<ConnectionState> {
  if (!isTauri()) {
    return NOT_IN_SHELL;
  }
  const { invoke } = await core();
  return invoke<ConnectionState>('daemon_state');
}

export async function reconnectDaemon(): Promise<ConnectionState> {
  if (!isTauri()) {
    return NOT_IN_SHELL;
  }
  const { invoke } = await core();
  return invoke<ConnectionState>('reconnect_daemon');
}

/**
 * The host hands back the profile document verbatim; parsing it here keeps the
 * JSON Schema the only contract between daemon and renderer.
 */
export async function fetchDeviceProfile(remeasure = false): Promise<DeviceProfileResult> {
  if (!isTauri()) {
    throw new Error(NOT_IN_SHELL.reason);
  }
  const { invoke } = await core();
  const raw = await invoke<{ artifactId: string; profileJson: string }>('device_profile', {
    remeasure,
  });
  return {
    artifactId: raw.artifactId,
    profile: JSON.parse(raw.profileJson) as DeviceProfile,
  };
}

export async function subscribeDaemonState(
  handler: (state: ConnectionState) => void,
): Promise<() => void> {
  if (!isTauri()) {
    return () => undefined;
  }
  const { listen } = await events();
  return listen<ConnectionState>(STATE_EVENT, (event) => {
    handler(event.payload);
  });
}
