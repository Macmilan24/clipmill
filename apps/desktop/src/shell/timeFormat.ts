/**
 * How the app writes a position: one way, everywhere.
 *
 * Clip time reads as `m:ss.t` by default, the form a creator says a moment
 * in. An editor who counts frames can switch to `m:ss:ff`, counted at the
 * clip's own frame rate rather than a broadcast one. The choice lives with
 * this installation and every screen that shows a time follows it.
 */
import { useSyncExternalStore } from 'react';

import { clockTenths } from '../inspector/review.js';

export type TimeFormat = 'clock' | 'frames';

const KEY = 'clipmill.timeFormat';
const TICKS_PER_SECOND = 90_000;
const listeners = new Set<() => void>();

export function timeFormat(): TimeFormat {
  try {
    return localStorage.getItem(KEY) === 'frames' ? 'frames' : 'clock';
  } catch {
    return 'clock';
  }
}

export function setTimeFormat(format: TimeFormat): void {
  try {
    localStorage.setItem(KEY, format);
  } catch {
    // This session keeps it through the listeners below.
  }
  for (const listener of listeners) listener();
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  const fromOtherWindow = (event: StorageEvent) => {
    if (event.key === KEY) listener();
  };
  window.addEventListener('storage', fromOtherWindow);
  return () => {
    listeners.delete(listener);
    window.removeEventListener('storage', fromOtherWindow);
  };
}

/** The format now, followed as it changes. */
export function useTimeFormat(): TimeFormat {
  return useSyncExternalStore(subscribe, timeFormat, () => 'clock');
}

const pad = (value: number) => String(value).padStart(2, '0');

/**
 * A position, in the chosen format. `fps` is the clip's frame rate, for the
 * frame count; it is not used by the clock.
 */
export function formatTime(ticks: number, format: TimeFormat, fps: number): string {
  if (format === 'clock' || !(fps > 0)) return clockTenths(ticks);
  const seconds = Math.max(0, ticks) / TICKS_PER_SECOND;
  const whole = Math.floor(seconds);
  const frames = Math.min(Math.ceil(fps) - 1, Math.floor((seconds - whole) * fps + 1e-6));
  const hours = Math.floor(whole / 3600);
  const minutes = Math.floor(whole / 60) % 60;
  const rest = `${pad(whole % 60)}:${pad(frames)}`;
  return hours > 0 ? `${hours}:${pad(minutes)}:${rest}` : `${minutes}:${rest}`;
}
