/**
 * What a person has already been shown: the welcome, once per installation,
 * and each workspace's tips, once each. A new version of either gets a new
 * key rather than erasing what was remembered.
 */
import { createContext, useContext } from 'react';

import { MEMORY_KEY, type KeyValueStore } from '../shell/memory.js';

export const WELCOME_KEY = 'clipmill.welcome.v1';

export type CoachPlace = 'inspector' | 'editor';

const coachKey = (place: CoachPlace) => `clipmill.coach.${place}.v1`;

export type WelcomeOutcome = 'started' | 'closed';

function store(): KeyValueStore | null {
  try {
    return typeof localStorage === 'undefined' ? null : localStorage;
  } catch {
    return null;
  }
}

/**
 * Greet a new installation only: one with no remembered place in the app and
 * no welcome on record. Someone who has used ClipMill before is not
 * interrupted by a welcome that arrived with an update.
 */
export function shouldWelcome(memory: KeyValueStore | null = store()): boolean {
  if (!memory) return false;
  try {
    return memory.getItem(MEMORY_KEY) === null && memory.getItem(WELCOME_KEY) === null;
  } catch {
    return false;
  }
}

export function rememberWelcome(outcome: WelcomeOutcome, memory = store()): void {
  try {
    memory?.setItem(WELCOME_KEY, outcome);
  } catch {
    // A restricted webview shows the welcome again next time, no worse.
  }
}

export function coachSeen(place: CoachPlace, memory = store()): boolean {
  try {
    return memory?.getItem(coachKey(place)) != null;
  } catch {
    return true;
  }
}

export function rememberCoach(place: CoachPlace, memory = store()): void {
  try {
    memory?.setItem(coachKey(place), 'seen');
  } catch {
    // Shown again next time, no worse.
  }
}

/** Forget the welcome and every tip, from Settings. */
export function forgetOnboarding(memory = store()): void {
  try {
    if (!memory || !('removeItem' in memory)) return;
    const removable = memory as KeyValueStore & { removeItem(key: string): void };
    removable.removeItem(WELCOME_KEY);
    for (const place of ['inspector', 'editor'] as const) removable.removeItem(coachKey(place));
  } catch {
    // Nothing remembered, nothing to forget.
  }
}

/** Asks the app to show the welcome again, from Settings. */
export const OPEN_WELCOME_EVENT = 'clipmill:welcome';

export function openWelcome(): void {
  window.dispatchEvent(new Event(OPEN_WELCOME_EVENT));
}

/**
 * Whether tips may show. The app turns them on; a screen rendered on its own
 * — in a test, or in the UI preview — shows none.
 */
export const CoachEnabled = createContext(false);

export function useCoachEnabled(): boolean {
  return useContext(CoachEnabled);
}
