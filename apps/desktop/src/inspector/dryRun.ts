/**
 * The Inspector's picture, from the clip an approval would build.
 *
 * The Inspector plays the recording — around the cut as well as through it —
 * while a preview plan counts the program's frames. So a moment of the
 * recording is placed in the program here: inside the cut, the frame that
 * plays it; before or after, the nearest edge's frame, whose framing stands
 * in while no caption shows.
 */
import type { PreviewPlan, PreviewSegment } from '../daemon/client.js';

const TICKS_PER_SECOND = 90_000;

export interface ProgramMoment {
  readonly frame: number;
  readonly segment: PreviewSegment;
  /** False before the cut starts or after it ends. */
  readonly inside: boolean;
}

/** Where a moment of the recording falls in the program, or the nearest edge. */
export function programAt(plan: PreviewPlan, ticks: number): ProgramMoment | null {
  let nearest: ProgramMoment | null = null;
  let distance = Number.POSITIVE_INFINITY;
  for (const segment of plan.segments) {
    if (segment.endFrame <= segment.firstFrame) continue;
    if (ticks >= segment.inTicks && ticks < segment.outTicks) {
      const into = Math.floor(
        ((ticks - segment.inTicks) * plan.rateNum) / (plan.rateDen * TICKS_PER_SECOND),
      );
      return {
        frame: Math.min(segment.endFrame - 1, segment.firstFrame + into),
        segment,
        inside: true,
      };
    }
    const before = segment.inTicks - ticks;
    if (before > 0 && before < distance) {
      distance = before;
      nearest = { frame: segment.firstFrame, segment, inside: false };
    }
    const after = ticks - segment.outTicks;
    if (after >= 0 && after < distance) {
      distance = after;
      nearest = { frame: segment.endFrame - 1, segment, inside: false };
    }
  }
  return nearest;
}

/** A section's layout geometry for a canvas `scale` times the plan's size. */
export function scaledSegment(segment: PreviewSegment, scale: number): PreviewSegment {
  const inset = segment.inset;
  return {
    ...segment,
    ...(segment.upperHeight ? { upperHeight: segment.upperHeight * scale } : {}),
    ...(inset ? { inset: [inset[0] * scale, inset[1] * scale, inset[2] * scale] as const } : {}),
  };
}

const NAMES: Record<string, string> = {
  fit: 'Whole frame',
  speaker_fill: 'Following the speaker',
  two_up: 'Two speakers',
  picture_in_picture: 'Picture in picture',
};

/** What the monitor bar says about the framing at a moment of a dry run. */
export function framingNote(plan: PreviewPlan, moment: ProgramMoment | null): string {
  if (!moment) return 'Whole frame';
  const layout = moment.segment.layout;
  const name =
    (layout && NAMES[layout]) ??
    ((plan.secondaryCrops?.[moment.frame] ?? null) !== null
      ? NAMES['two_up']!
      : plan.crops[moment.frame]
        ? NAMES['speaker_fill']!
        : NAMES['fit']!);
  const count = plan.segments.length;
  return count > 1
    ? `${name} · Section ${plan.segments.indexOf(moment.segment) + 1} of ${count}`
    : name;
}
