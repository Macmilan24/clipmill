import { describe, expect, it } from 'vitest';

import { rememberTour, shouldAutoStartTour, TOUR_KEY } from '../src/onboarding/state.js';
import { MEMORY_KEY } from '../src/shell/memory.js';

function store(initial: Record<string, string> = {}) {
  const data = new Map(Object.entries(initial));
  return {
    getItem: (key: string) => data.get(key) ?? null,
    setItem: (key: string, value: string) => {
      data.set(key, value);
    },
  };
}

describe('first-run guide', () => {
  it('opens only for a new profile and remembers a completed or skipped tour', () => {
    const fresh = store();
    expect(shouldAutoStartTour(fresh)).toBe(true);
    rememberTour(fresh, 'skipped');
    expect(fresh.getItem(TOUR_KEY)).toBe('skipped');
    expect(shouldAutoStartTour(fresh)).toBe(false);
  });

  it('does not interrupt someone who already used ClipMill before this guide existed', () => {
    expect(shouldAutoStartTour(store({ [MEMORY_KEY]: '{"route":"library"}' }))).toBe(false);
    expect(shouldAutoStartTour(null)).toBe(false);
  });
});
