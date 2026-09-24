/**
 * Program-time arithmetic for the Editor's timeline: how far it reaches past the
 * clip into source that can be pulled in, zoom and pan inside that reach, and
 * the edit points the arrow keys walk between.
 */
import type { PreviewPlan, PreviewSegment } from '../daemon/client.js';
import { type Cut, clamp } from '../inspector/review.js';
import { frameAt, proxyOf } from './player.js';

export const TICKS = 90_000;

/** The most source shown past either end of the clip, ready to pull in. */
const REACH_TICKS = 6 * TICKS;

/** The narrowest view the timeline zooms to. */
const MIN_SPAN_TICKS = TICKS;

/** The program's length: its sections laid end to end. */
export function programTicks(plan: PreviewPlan): number {
  return plan.segments.reduce((sum, part) => sum + part.outTicks - part.inTicks, 0);
}

/** The frame a program tick is in. */
export function frameOfTicks(plan: PreviewPlan, ticks: number): number {
  return frameAt(plan, ticks / TICKS);
}

/** Where a frame starts, in program ticks. */
export function ticksOfFrame(plan: PreviewPlan, frame: number): number {
  return Math.round((frame * plan.rateDen * TICKS) / Math.max(1, plan.rateNum));
}

/**
 * How much source lies before the clip's first section and after its last,
 * up to a few seconds each way: what a pulled edge can reach without leaving
 * the proxy the preview plays.
 */
export function reach(plan: PreviewPlan): { readonly before: number; readonly after: number } {
  const first = plan.segments[0];
  const last = plan.segments.at(-1);
  if (!first || !last) return { before: 0, after: 0 };
  const head = proxyOf(plan, first);
  const tail = proxyOf(plan, last);
  return {
    before: head ? clamp(first.inTicks - head.coverageStartTicks, 0, REACH_TICKS) : 0,
    after: tail ? clamp(tail.coverageEndTicks - last.outTicks, 0, REACH_TICKS) : 0,
  };
}

/** The whole timeline: the clip, and the reachable source either side of it. */
export function extentOf(plan: PreviewPlan): Cut {
  const { before, after } = reach(plan);
  return { startTicks: -before, endTicks: programTicks(plan) + after };
}

/**
 * The source tick under a program tick, past either end of the clip too, so
 * the reach can be drawn from the recording's own stills and sound.
 */
export function sourceAt(
  plan: PreviewPlan,
  ticks: number,
): { readonly segment: PreviewSegment; readonly ticks: number } | null {
  const first = plan.segments[0];
  const last = plan.segments.at(-1);
  if (!first || !last) return null;
  if (ticks < 0) return { segment: first, ticks: first.inTicks + ticks };
  for (const segment of plan.segments) {
    const length = segment.outTicks - segment.inTicks;
    if (ticks < segment.programStartTicks + length) {
      return { segment, ticks: segment.inTicks + ticks - segment.programStartTicks };
    }
  }
  return {
    segment: last,
    ticks: last.outTicks + ticks - (last.programStartTicks + last.outTicks - last.inTicks),
  };
}

/** A view zoomed by `factor` about `anchor`, kept inside the extent. */
export function zoomSpan(view: Cut, factor: number, anchor: number, extent: Cut): Cut {
  const span = view.endTicks - view.startTicks;
  const whole = extent.endTicks - extent.startTicks;
  const next = clamp(span * factor, Math.min(MIN_SPAN_TICKS, whole), whole);
  const share = span <= 0 ? 0.5 : (anchor - view.startTicks) / span;
  const start = clamp(anchor - share * next, extent.startTicks, extent.endTicks - next);
  return { startTicks: start, endTicks: start + next };
}

/** A view slid by `ticks`, kept inside the extent. */
export function panSpan(view: Cut, ticks: number, extent: Cut): Cut {
  const span = view.endTicks - view.startTicks;
  const start = clamp(view.startTicks + ticks, extent.startTicks, extent.endTicks - span);
  return { startTicks: start, endTicks: start + span };
}

/** The view that shows `ticks`, moved as little as possible. */
export function revealSpan(view: Cut, ticks: number, extent: Cut): Cut {
  const span = view.endTicks - view.startTicks;
  if (ticks >= view.startTicks && ticks <= view.endTicks) return view;
  return panSpan(view, ticks - (view.startTicks + span / 2), extent);
}

/**
 * Where the arrow keys stop: the start of every section and caption, and the
 * end of the clip, in frames, sorted and without repeats.
 */
export function editPoints(plan: PreviewPlan): number[] {
  const points = new Set<number>([0, Math.max(0, plan.frameCount - 1)]);
  for (const part of plan.segments) points.add(part.firstFrame);
  for (const cue of plan.cues) points.add(cue.firstFrame);
  return [...points].toSorted((a, b) => a - b);
}

/** A section id that is not in the document yet, derived so replay repeats it. */
export function freshSegmentId(existing: Iterable<string>, base: string, atTicks: number): string {
  const taken = new Set(existing);
  let id = `${base}_cut_${atTicks}`;
  for (let suffix = 2; taken.has(id); suffix += 1) id = `${base}_cut_${atTicks}_${suffix}`;
  return id;
}

/** A cue id that is not in the list yet, derived from the cue it came out of. */
export function freshCueId(existing: Iterable<string>, base: string): string {
  const taken = new Set(existing);
  let id = `${base}_b`;
  for (let suffix = 2; taken.has(id); suffix += 1) id = `${base}_b${suffix}`;
  return id;
}
