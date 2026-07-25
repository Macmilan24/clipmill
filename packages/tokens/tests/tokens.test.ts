import { describe, expect, it } from 'vitest';

import {
  DEFAULT_THEME,
  ThemeController,
  type ThemeStorage,
  type ThemeTarget,
  isTheme,
  tokens,
} from '../src/index.js';

class FakeRoot implements ThemeTarget {
  #attributes = new Map<string, string>();

  getAttribute(name: string): string | null {
    return this.#attributes.get(name) ?? null;
  }

  setAttribute(name: string, value: string): void {
    this.#attributes.set(name, value);
  }
}

class FakeStorage implements ThemeStorage {
  #entries = new Map<string, string>();

  getItem(key: string): string | null {
    return this.#entries.get(key) ?? null;
  }

  setItem(key: string, value: string): void {
    this.#entries.set(key, value);
  }
}

describe('token document', () => {
  it('defines the same variable names in both themes', () => {
    // A themed value added to one side only would silently keep the other
    // theme's stale colour, which is invisible until someone toggles.
    const dark = Object.keys(tokens.themes.dark).toSorted();
    const light = Object.keys(tokens.themes.light).toSorted();
    expect(light).toEqual(dark);
  });

  it('keeps the accent identical across themes', () => {
    // One product accent is a hard design rule, so it must not be themed.
    expect(tokens.accent.default).toBe('#5E6AD2');
    expect(Object.keys(tokens.themes.dark)).not.toContain('accent');
  });

  it('carries the reserved outbound-network colour', () => {
    // Reserved exclusively for actions that send data off-device.
    expect(tokens.semantic.outbound).toBe('#D9756B');
  });
});

describe('ThemeController', () => {
  it('reports the stylesheet default before anything is applied', () => {
    expect(new ThemeController(new FakeRoot()).current()).toBe(DEFAULT_THEME);
  });

  it('toggles between the two themes and persists the choice', () => {
    const storage = new FakeStorage();
    const controller = new ThemeController(new FakeRoot(), storage);

    expect(controller.toggle()).toBe('light');
    expect(controller.current()).toBe('light');
    expect(controller.toggle()).toBe('dark');
    expect(storage.getItem('clipmill.theme')).toBe('dark');
  });

  it('prefers a stored theme over the OS preference', () => {
    const storage = new FakeStorage();
    storage.setItem('clipmill.theme', 'dark');
    expect(ThemeController.resolveInitial(storage, true)).toBe('dark');
  });

  it('falls back to the OS preference when nothing is stored', () => {
    expect(ThemeController.resolveInitial(new FakeStorage(), true)).toBe('light');
    expect(ThemeController.resolveInitial(new FakeStorage(), false)).toBe('dark');
  });

  it('ignores a corrupted stored value', () => {
    const storage = new FakeStorage();
    storage.setItem('clipmill.theme', 'chartreuse');
    expect(ThemeController.resolveInitial(storage, false)).toBe(DEFAULT_THEME);
    expect(isTheme('chartreuse')).toBe(false);
  });
});
