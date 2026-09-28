/**
 * ClipMill's components, as the daemon answers them: the model workers a
 * packaged app installs on first run and keeps running.
 *
 * Every request answers with the components as they then stand, so a screen
 * replaces what it holds rather than guessing what its click did. Installing
 * downloads Python and the workers' packages, so the page says so before it
 * asks; which parts need installing and whether an install worked are the
 * daemon's decisions.
 */
import { isTauri } from './client.js';

/** Where one part's install stands. Outdated keeps running until replaced. */
export type ComponentState =
  'missing' | 'queued' | 'installing' | 'installed' | 'outdated' | 'failed';
/** What its worker process is doing. Waiting: it stopped and restarts itself. */
export type ComponentProcess = 'stopped' | 'starting' | 'running' | 'waiting';

export interface Component {
  readonly name: string;
  /** What it does, for a person: "Speech detection". */
  readonly title: string;
  /** The worker family its tasks are leased to. */
  readonly family: string;
  readonly state: ComponentState;
  /** The step running now, or one sentence on why the last attempt failed. */
  readonly detail: string;
  readonly installedBytes: number;
  /** What installing it downloads, measured when the app was built; 0 when not. */
  readonly downloadBytes: number;
  readonly process: ComponentProcess;
  /** How often its process stopped on its own since the engine started. */
  readonly restarts: number;
  readonly logPath: string;
}

export interface Components {
  /** False in a development checkout, whose workers are started by hand. */
  readonly managed: boolean;
  readonly parts: readonly Component[];
  readonly pythonVersion: string;
  /** Why they cannot be installed here at all, when they cannot. */
  readonly unavailable: string;
}

export interface ComponentsApi {
  listComponents(): Promise<Components>;
  /**
   * A network operation: the caller has already said so on screen. Empty
   * installs every part that is missing, out of date or failed.
   */
  installComponents(parts: readonly string[]): Promise<Components>;
  cancelComponentInstall(): Promise<Components>;
}

/** What a browser preview, which has no daemon, reads. */
export const UNMANAGED: Components = {
  managed: false,
  parts: [],
  pythonVersion: '',
  unavailable: '',
};

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) throw new Error('Components are managed in the desktop app.');
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(command, args);
}

export const componentsApi: ComponentsApi = {
  listComponents: () => (isTauri() ? call('list_components') : Promise.resolve(UNMANAGED)),
  installComponents: (parts) => call('install_components', { parts: [...parts] }),
  cancelComponentInstall: () => call('cancel_component_install'),
};

/** Parts that still need installing, updating or another try. */
export function componentsNeeded(components: Components | null): readonly Component[] {
  if (components === null || !components.managed) return [];
  return components.parts.filter((part) => ['missing', 'outdated', 'failed'].includes(part.state));
}

/** Whether an install is queued or running. */
export function installingComponents(components: Components | null): boolean {
  return components?.parts.some((part) => ['queued', 'installing'].includes(part.state)) ?? false;
}

/** What installing `parts` downloads, as measured when the app was built. */
export function componentDownloadBytes(parts: readonly Component[]): number {
  return parts.reduce((total, part) => total + part.downloadBytes, 0);
}
