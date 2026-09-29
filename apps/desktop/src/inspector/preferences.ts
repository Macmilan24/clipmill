/**
 * How clip review behaves on this machine: whether a decision moves on to the
 * next clip, and the speed clips play at. The Inspector changes them in
 * place and Settings changes them too; every screen follows the latest.
 */
import { useSyncExternalStore } from 'react';

import { SPEEDS } from './playback.js';

export type ReviewSpeed = (typeof SPEEDS)[number];

const ADVANCE_KEY = 'clipmill.review.advance';
const SPEED_KEY = 'clipmill.review.speed';
const listeners = new Set<() => void>();

function read(key: string): string | null {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

function write(key: string, value: string): void {
  try {
    localStorage.setItem(key, value);
  } catch {
    // Kept for as long as the page keeps it.
  }
  for (const listener of listeners) listener();
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

/** Whether a decision moves on to the next undecided clip. On until turned off. */
export function autoAdvance(): boolean {
  return read(ADVANCE_KEY) !== 'off';
}

export function setAutoAdvance(on: boolean): void {
  write(ADVANCE_KEY, on ? 'on' : 'off');
}

export function useAutoAdvance(): boolean {
  return useSyncExternalStore(subscribe, autoAdvance, () => true);
}

/** The speed review plays a clip at, from its first frame. */
export function reviewSpeed(): ReviewSpeed {
  const stored = Number(read(SPEED_KEY));
  return SPEEDS.find((speed) => speed === stored) ?? 1;
}

export function setReviewSpeed(speed: ReviewSpeed): void {
  write(SPEED_KEY, String(speed));
}

export function useReviewSpeed(): ReviewSpeed {
  return useSyncExternalStore(subscribe, reviewSpeed, () => 1);
}
