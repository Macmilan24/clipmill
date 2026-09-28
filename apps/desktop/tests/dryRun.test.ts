import { describe, expect, it } from 'vitest';

import type { PreviewPlan } from '../src/daemon/client.js';
import { framingNote, programAt, scaledSegment } from '../src/inspector/dryRun.js';

const SECOND = 90_000;
const segment = (id: string, inS: number, outS: number, first: number, layout: string) => ({
  segmentId: id,
  sourceFingerprint: 'source',
  inTicks: inS * SECOND,
  outTicks: outS * SECOND,
  programStartTicks: first * 3_000,
  firstFrame: first,
  endFrame: first + (outS - inS) * 30,
  layout: layout as 'fit',
});
const plan = {
  rateNum: 30,
  rateDen: 1,
  crops: Array.from({ length: 150 }, (_, frame) => (frame < 60 ? [0, 0, 608, 1080] : null)),
  segments: [segment('a', 10, 12, 0, 'speaker_fill'), segment('b', 12, 15, 60, 'fit')],
} as unknown as PreviewPlan;

describe('placing the recording in the program an approval builds', () => {
  it('finds the frame that plays a moment inside the cut', () => {
    expect(programAt(plan, 11 * SECOND)).toMatchObject({ frame: 30, inside: true });
    expect(programAt(plan, 13 * SECOND)).toMatchObject({ frame: 90, inside: true });
  });

  it('holds the nearest edge before and after the cut', () => {
    expect(programAt(plan, 9 * SECOND)).toMatchObject({ frame: 0, inside: false });
    expect(programAt(plan, 20 * SECOND)).toMatchObject({ frame: 149, inside: false });
  });

  it('names the framing and the section', () => {
    expect(framingNote(plan, programAt(plan, 11 * SECOND))).toBe(
      'Following the speaker · Section 1 of 2',
    );
    expect(framingNote(plan, programAt(plan, 13 * SECOND))).toBe('Whole frame · Section 2 of 2');
  });

  it('scales a section’s geometry to the canvas', () => {
    const scaled = scaledSegment(
      { ...plan.segments[0]!, upperHeight: 606, inset: [42, 988, 432] },
      0.5,
    );
    expect(scaled.upperHeight).toBe(303);
    expect(scaled.inset).toEqual([21, 494, 216]);
  });
});
