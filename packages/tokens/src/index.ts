/**
 * Pro-Studio Precision design tokens.
 *
 * tokens.json is the hand-transcribed source of truth; tokens.css and
 * tailwind-preset.css are generated from it and committed (decision R2).
 * Import the CSS once at the app entry point, then read values through the
 * custom properties rather than re-declaring literals in components.
 */
import tokens from './tokens.json' with { type: 'json' };

export { tokens };

export type Theme = 'dark' | 'light';

export const THEMES: readonly Theme[] = ['dark', 'light'];

/** Matches the :root block in tokens.css, so first paint needs no correction. */
export const DEFAULT_THEME: Theme = 'dark';

const THEME_ATTRIBUTE = 'data-theme';
const STORAGE_KEY = 'clipmill.theme';

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

/**
 * The one runtime theme switch. The design ships two artboards per screen, but
 * the shell must not fork components per theme: every themed value resolves
 * through a custom property, and this controller flips the single attribute
 * those properties key off. The attribute always wins over the OS preference,
 * which is only consulted to pick the very first value.
 */
export class ThemeController {
  readonly #root: ThemeTarget;
  readonly #storage: ThemeStorage | null;

  constructor(root: ThemeTarget, storage: ThemeStorage | null = null) {
    this.#root = root;
    this.#storage = storage;
  }

  /** Stored choice, else the OS preference, else the stylesheet default. */
  static resolveInitial(storage: ThemeStorage | null, prefersLight: boolean): Theme {
    const stored = storage?.getItem(STORAGE_KEY);
    if (isTheme(stored)) {
      return stored;
    }
    return prefersLight ? 'light' : DEFAULT_THEME;
  }

  current(): Theme {
    const attribute = this.#root.getAttribute(THEME_ATTRIBUTE);
    return isTheme(attribute) ? attribute : DEFAULT_THEME;
  }

  apply(theme: Theme): Theme {
    this.#root.setAttribute(THEME_ATTRIBUTE, theme);
    this.#storage?.setItem(STORAGE_KEY, theme);
    return theme;
  }

  toggle(): Theme {
    return this.apply(this.current() === 'dark' ? 'light' : 'dark');
  }
}
