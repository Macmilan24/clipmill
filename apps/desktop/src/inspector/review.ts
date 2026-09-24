/**
 * Review station arithmetic in source ticks: timecodes, the crop a solved path
 * puts on a frame, overlapping candidates, the next clip to review, ruler steps
 * and the platform safe area.
 */
import type { CropPath } from '../daemon/client.js';
import { type ClipRow, TICKS_PER_SECOND } from '../results/model.js';

/** One frame at the render rate, 30000/1001. */
export const FRAME_TICKS = 3_003;

/** A window of the recording, in source ticks. */
export interface Cut {
  readonly startTicks: number;
  readonly endTicks: number;
}

export function clamp(value: number, low: number, high: number): number {
  return Math.max(low, Math.min(high, value));
}

const pad = (value: number) => String(value).padStart(2, '0');

/** `hh:mm:ss;ff` at the render rate, which is how an editor reads a position. */
export function timecode(ticks: number): string {
  const safe = Math.max(0, ticks);
  const totalSeconds = Math.floor(safe / TICKS_PER_SECOND);
  const frames = Math.floor((safe % TICKS_PER_SECOND) / FRAME_TICKS);
  return `${pad(Math.floor(totalSeconds / 3600))}:${pad(Math.floor(totalSeconds / 60) % 60)}:${pad(
    totalSeconds % 60,
  )};${pad(frames)}`;
}

/**
 * A position as `m:ss.t`, or `h:mm:ss.t` past an hour: the reading form, for
 * places where a frame count would be noise.
 */
export function clockTenths(ticks: number): string {
  const tenths = Math.max(0, Math.round(ticks / (TICKS_PER_SECOND / 10)));
  const seconds = Math.floor(tenths / 10);
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor(seconds / 60) % 60;
  const rest = `${pad(seconds % 60)}.${tenths % 10}`;
  return hours > 0 ? `${hours}:${pad(minutes)}:${rest}` : `${minutes}:${rest}`;
}

/** A length as `42.1 s`, or `1:12.4` past a minute. */
export function lengthLabel(ticks: number): string {
  const seconds = Math.max(0, ticks) / TICKS_PER_SECOND;
  if (seconds < 60) return `${seconds.toFixed(1)} s`;
  return clockTenths(ticks);
}

/** The same cut, by value. */
export function sameCut(left: Cut, right: Cut): boolean {
  return left.startTicks === right.startTicks && left.endTicks === right.endTicks;
}

/**
 * The crop a path puts on a frame at a moment, in that frame's pixels.
 *
 * The path is the solver's proposal — centres and a scale, normalized against
 * the source — interpolated the way a player does between its keyframes. The
 * rectangle is 9:16, as tall as the scale says, and kept inside the frame: the
 * same conversion the director makes when it writes the path into an edit, so
 * the review shows the framing the edit will start from. Null is a fitted
 * frame, which is drawn whole over its own blurred fill.
 */
export function cropRect(
  path: CropPath | null,
  atTicks: number,
  frame: { readonly width: number; readonly height: number },
): { x: number; y: number; width: number; height: number } | null {
  const keyframes = path?.keyframes ?? [];
  if (!path || path.fit || keyframes.length === 0 || frame.width <= 0 || frame.height <= 0) {
    return null;
  }
  const first = keyframes[0]!;
  const last = keyframes.at(-1)!;
  let point = { x: first.centerX, y: first.centerY, scale: first.scale };
  if (atTicks >= last.tTicks) {
    point = { x: last.centerX, y: last.centerY, scale: last.scale };
  } else if (atTicks > first.tTicks) {
    const after = keyframes.findIndex((keyframe) => keyframe.tTicks > atTicks);
    const before = keyframes[after - 1]!;
    const next = keyframes[after]!;
    const span = next.tTicks - before.tTicks;
    const ratio = span <= 0 ? 0 : (atTicks - before.tTicks) / span;
    point = {
      x: before.centerX + (next.centerX - before.centerX) * ratio,
      y: before.centerY + (next.centerY - before.centerY) * ratio,
      scale: before.scale + (next.scale - before.scale) * ratio,
    };
  }
  let height = clamp(point.scale, 0.01, 1) * frame.height;
  let width = (height * 9) / 16;
  if (width > frame.width) {
    width = frame.width;
    height = (width * 16) / 9;
  }
  return {
    x: clamp(point.x * frame.width - width / 2, 0, frame.width - width),
    y: clamp(point.y * frame.height - height / 2, 0, frame.height - height),
    width,
    height,
  };
}

/** Another candidate that covers the same ground, and how much of it. */
export interface Overlap {
  readonly candidateId: string;
  readonly rank: number;
  /** The shared span as a share of the shorter clip, 0–1. */
  readonly share: number;
}

/**
 * The candidates that repeat this one.
 *
 * Measured against the shorter of the two, because a ten-second clip wholly
 * inside a minute-long one is a duplicate however little of the minute it
 * covers. A sliver of shared context is not a repeat, so a share under a
 * quarter is left out.
 */
export function overlapsOf(rows: readonly ClipRow[], row: ClipRow, minimum = 0.25): Overlap[] {
  const found: Overlap[] = [];
  for (const other of rows) {
    if (other.candidateId === row.candidateId) continue;
    const shared =
      Math.min(row.endTicks, other.endTicks) - Math.max(row.startTicks, other.startTicks);
    const shorter = Math.min(row.endTicks - row.startTicks, other.endTicks - other.startTicks);
    if (shared <= 0 || shorter <= 0) continue;
    const share = shared / shorter;
    if (share >= minimum) {
      found.push({ candidateId: other.candidateId, rank: other.rank, share });
    }
  }
  return found.toSorted((left, right) => right.share - left.share || left.rank - right.rank);
}

/**
 * The clip to review next: the first undecided one after this, wrapping
 * round, and never this one. Null when nothing is left to decide.
 */
export function nextUndecided(rows: readonly ClipRow[], candidateId: string): string | null {
  const at = rows.findIndex((row) => row.candidateId === candidateId);
  for (let step = 1; step <= rows.length; step += 1) {
    const row = rows[(at + step + rows.length) % rows.length];
    if (row && row.candidateId !== candidateId && row.decision === null) {
      return row.candidateId;
    }
  }
  return null;
}

/** How far through a review is: decided, and of which kind. */
export function progress(rows: readonly ClipRow[]): {
  readonly decided: number;
  readonly approved: number;
  readonly kept: number;
  readonly rejected: number;
  readonly total: number;
} {
  const count = (decision: ClipRow['decision']) =>
    rows.filter((row) => row.decision === decision).length;
  const approved = count('approved');
  const kept = count('kept');
  const rejected = count('rejected');
  return { decided: approved + kept + rejected, approved, kept, rejected, total: rows.length };
}

/** Steps a ruler may count in, in seconds, from a tenth to ten minutes. */
const RULER_STEPS = [0.1, 0.2, 0.5, 1, 2, 5, 10, 15, 30, 60, 120, 300, 600];

/**
 * The finest ruler step that still leaves `minimumPixels` between labels.
 *
 * A ruler that labels every frame is a grey smear and one that labels twice
 * is a guess, so the step follows the zoom.
 */
export function rulerStep(spanTicks: number, widthPixels: number, minimumPixels = 72): number {
  if (spanTicks <= 0 || widthPixels <= 0) return TICKS_PER_SECOND;
  const perPixel = spanTicks / widthPixels;
  for (const seconds of RULER_STEPS) {
    const ticks = seconds * TICKS_PER_SECOND;
    if (ticks / perPixel >= minimumPixels) return ticks;
  }
  return RULER_STEPS.at(-1)! * TICKS_PER_SECOND;
}

/**
 * Where the three apps draw their own buttons and captions over a 1080×1920
 * frame, as shares of it, taken at the most cautious of TikTok, Reels and
 * Shorts: the top bar, the caption and sound block along the bottom, and the
 * like-and-share column down the right. The apps move these, so the overlay is
 * a guide for a reviewer's eye, never a check anything is refused by.
 */
export const SAFE_AREA = {
  top: 130 / 1920,
  bottom: 320 / 1920,
  left: 60 / 1080,
  right: 120 / 1080,
} as const;
