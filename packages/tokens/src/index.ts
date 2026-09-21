/** Shared tokens and local workspace appearance preferences. */
import tokens from './tokens.json' with { type: 'json' };
import catalog from './workspace-themes.json' with { type: 'json' };

export { tokens };

export type Theme = 'dark' | 'light';

export const THEMES: readonly Theme[] = ['dark', 'light'];

/** Matches the :root block in tokens.css, so first paint needs no correction. */
export const DEFAULT_THEME: Theme = 'dark';

export type WorkspaceTheme = keyof typeof catalog.themes;
export const DEFAULT_WORKSPACE_THEME = catalog.default as WorkspaceTheme;
export const WORKSPACE_THEMES = Object.entries(catalog.themes).map(([id, definition]) =>
  Object.assign({ id: id as WorkspaceTheme }, definition),
);

export function isWorkspaceTheme(value: unknown): value is WorkspaceTheme {
  return typeof value === 'string' && Object.hasOwn(catalog.themes, value);
}

export function workspacePalette(workspace: WorkspaceTheme, appearance: Theme) {
  return { ...tokens.themes[appearance], ...catalog.themes[workspace][appearance] };
}

const THEME_ATTRIBUTE = 'data-theme';
const STORAGE_KEY = 'clipmill.theme';
const WORKSPACE_STORAGE_KEY = 'clipmill.workspace-theme';

function readPreference(storage: ThemeStorage | null, key: string): string | null {
  try {
    return storage?.getItem(key) ?? null;
  } catch {
    return null;
  }
}

export function isTheme(value: unknown): value is Theme {
  return value === 'dark' || value === 'light';
}

/** The document surface ThemeController needs, narrowed so tests can fake it. */
export interface ThemeTarget {
  getAttribute(name: string): string | null;
  setAttribute(name: string, value: string): void;
}

export interface ThemeStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

/** Theme and light/dark choices are independent and never affect project data. */
export class ThemeController {
  readonly #root: ThemeTarget;
  readonly #storage: ThemeStorage | null;

  constructor(root: ThemeTarget, storage: ThemeStorage | null = null) {
    this.#root = root;
    this.#storage = storage;
  }

  /** Stored choice, else the OS preference, else the stylesheet default. */
  static resolveInitial(storage: ThemeStorage | null, prefersLight: boolean): Theme {
    const stored = readPreference(storage, STORAGE_KEY);
    if (isTheme(stored)) {
      return stored;
    }
    return prefersLight ? 'light' : DEFAULT_THEME;
  }

  static resolveWorkspace(storage: ThemeStorage | null): WorkspaceTheme {
    const stored = readPreference(storage, WORKSPACE_STORAGE_KEY);
    return isWorkspaceTheme(stored) ? stored : DEFAULT_WORKSPACE_THEME;
  }

  current(): Theme {
    const attribute = this.#root.getAttribute(THEME_ATTRIBUTE);
    return isTheme(attribute) ? attribute : DEFAULT_THEME;
  }

  apply(theme: Theme): Theme {
    this.#root.setAttribute(THEME_ATTRIBUTE, theme);
    this.#remember(STORAGE_KEY, theme);
    return theme;
  }

  applyWorkspace(theme: WorkspaceTheme): WorkspaceTheme {
    this.#root.setAttribute('data-workspace-theme', theme);
    this.#root.setAttribute('data-theme-chrome', catalog.themes[theme].chrome);
    this.#remember(WORKSPACE_STORAGE_KEY, theme);
    return theme;
  }

  #remember(key: string, value: string): void {
    try {
      this.#storage?.setItem(key, value);
    } catch {
      // The current session remains usable when preference storage is unavailable.
    }
  }

  toggle(): Theme {
    return this.apply(this.current() === 'dark' ? 'light' : 'dark');
  }
}
