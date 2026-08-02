/**
 * Applying a preview plan. Nothing here derives anything.
 *
 * Every function in this file is a lookup. The crop at a frame is read out of
 * an array the daemon filled; the caption at a frame is the cue whose window
 * contains it; the highlighted word is found by walking holds that were already
 * measured in centiseconds. If any of this computed instead of looked up, it
 * would be a second implementation of the render's arithmetic — and the whole
 * point of the plan is that there is only one.
 *
 * The one thing this file *does* compute is the mapping from a media element's
 * `currentTime` to a frame index, and that is unavoidable: a browser reports
 * seconds. It is done by one function so there is one place to look when a
 * frame is off by one.
 */
import type { PreviewCue, PreviewPlan } from '../daemon/client.js';

/** The frame a playhead in seconds is inside. */
export function frameAt(plan: PreviewPlan, seconds: number): number {
  if (plan.rateNum <= 0 || plan.rateDen <= 0) {
    return 0;
  }
  const frame = Math.floor((seconds * plan.rateNum) / plan.rateDen);
  return Math.max(0, Math.min(plan.frameCount - 1, frame));
}

/** Where a frame starts, in seconds. The inverse of `frameAt`. */
export function secondsAt(plan: PreviewPlan, frame: number): number {
  if (plan.rateNum <= 0) {
    return 0;
  }
  return (frame * plan.rateDen) / plan.rateNum;
}

/** The crop the encoder will apply at a frame, or null where it fits. */
export function cropAt(
  plan: PreviewPlan,
  frame: number,
): {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
} | null {
  const found = plan.crops[Math.max(0, Math.min(plan.crops.length - 1, frame))];
  if (!found) {
    return null;
  }
  const [x, y, width, height] = found;
  return { x, y, width, height };
}

/** The cue on screen at a frame, or nothing. */
export function cueAt(plan: PreviewPlan, frame: number): PreviewCue | null {
  return plan.cues.find((cue) => frame >= cue.firstFrame && frame < cue.endFrame) ?? null;
}

/**
 * Which word of a cue carries the highlight at a frame.
 *
 * Walks the holds the plan already measured rather than re-deriving them from
 * word timings, which is the same reason the burned-in track and this agree:
 * both are reading one sweep. Returns -1 before the first word is sung.
 */
export function highlightedWord(plan: PreviewPlan, cue: PreviewCue, frame: number): number {
  if (!cue.karaoke) {
    return -1;
  }
  const elapsedCentis = (secondsAt(plan, frame) - secondsAt(plan, cue.firstFrame)) * 100;
  let at = cue.leadInCentis;
  if (elapsedCentis < at) {
    return -1;
  }
  let index = 0;
  for (const line of cue.lines) {
    for (const word of line) {
      at += word.holdCentis;
      if (elapsedCentis < at) {
        return index;
      }
      index += 1;
    }
  }
  return index - 1;
}

/** A cue's text as a reader sees it, lines kept apart. */
export function cueLines(cue: PreviewCue): readonly string[] {
  return cue.lines.map((line) => line.map((word) => word.text).join(' '));
}

/** Where a frame sits along a lane, as a percentage. */
export function lanePosition(plan: PreviewPlan, frame: number): number {
  if (plan.frameCount <= 1) {
    return 0;
  }
  return (frame / (plan.frameCount - 1)) * 100;
}

/** The gain in decibels at a frame, held from the last point before it. */
export function gainAt(plan: PreviewPlan, frame: number): number {
  let value = 0;
  for (const point of plan.gain) {
    if (point.frame > frame) {
      break;
    }
    value = point.gainDb;
  }
  return value;
}

/** A frame as `m:ss.ff`, which is how an editor reads a transport. */
export function timecode(plan: PreviewPlan, frame: number): string {
  const seconds = secondsAt(plan, frame);
  const whole = Math.floor(seconds);
  const frames = Math.max(0, frame - frameAt(plan, whole));
  return `${Math.floor(whole / 60)}:${String(whole % 60).padStart(2, '0')}.${String(frames).padStart(2, '0')}`;
}
