import { MEMORY_KEY, type KeyValueStore } from '../shell/memory.js';

/** A new version can introduce a new walkthrough without erasing shell memory. */
export const TOUR_KEY = 'clipmill.guided-tour.v1';

export type TourOutcome = 'finished' | 'skipped';

/** Existing installations should not be interrupted by a new first-run tour. */
export function shouldAutoStartTour(store: KeyValueStore | null): boolean {
  if (!store) return false;
  try {
    return store.getItem(MEMORY_KEY) === null && store.getItem(TOUR_KEY) === null;
  } catch {
    return false;
  }
}

export function rememberTour(store: KeyValueStore | null, outcome: TourOutcome): void {
  if (!store) return;
  try {
    store.setItem(TOUR_KEY, outcome);
  } catch {
    // A restricted WebView can still use the guide for this launch.
  }
}
