/**
 * What the player is allowed to do with a plan.
 *
 * Every one of these is really the same assertion: the answer came out of the
 * plan rather than out of this side. A player that derived a crop, a cue window
 * or a highlight would be a second implementation of the render's arithmetic,
 * and the whole workstream turns on there being one.
 */
import { describe, expect, it } from 'vitest';

import type { PreviewPlan } from '../src/daemon/client.js';
import {
  cropAt,
  cueAt,
  cueLines,
  frameAt,
  gainAt,
  highlightedWord,
  lanePosition,
  secondsAt,
  timecode,
} from '../src/editor/player.js';

/** Thirty frames, a crop that moves, one karaoke cue, one gain step. */
function plan(): PreviewPlan {
  return {
    revision: 3,
    rateNum: 30_000,
    rateDen: 1_001,
    frameCount: 30,
    crops: Array.from({ length: 30 }, (_unused, frame) =>
      frame < 10 ? null : ([frame * 10, 0, 608, 1080] as const),
    ),
    cues: [
      {
        cueId: 'hot_1',
        firstFrame: 5,
        endFrame: 20,
        region: 'lower_safe',
        karaoke: true,
        leadInCentis: 10,
        lines: [
          [
            { text: 'the', holdCentis: 20 },
            { text: 'whole', holdCentis: 20 },
          ],
          [{ text: 'point', holdCentis: 10 }],
        ],
      },
    ],
    gain: [
      { frame: 0, gainDb: 0 },
      { frame: 15, gainDb: -6 },
    ],
    width: 1080,
    height: 1920,
  };
}

describe('reading a plan', () => {
  it('finds the crop the daemon put at a frame, and null where it fits', () => {
    expect(cropAt(plan(), 0)).toBeNull();
    expect(cropAt(plan(), 12)).toEqual({ x: 120, y: 0, width: 608, height: 1080 });
  });

  it('never reads past the end of the array it was given', () => {
    expect(cropAt(plan(), 9_999)).toEqual({ x: 290, y: 0, width: 608, height: 1080 });
    expect(cropAt(plan(), -5)).toBeNull();
  });

  it('shows a cue only inside the window the plan states', () => {
    expect(cueAt(plan(), 4)).toBeNull();
    expect(cueAt(plan(), 5)?.cueId).toBe('hot_1');
    expect(cueAt(plan(), 19)?.cueId).toBe('hot_1');
    // The window is half open, exactly as the render's is.
    expect(cueAt(plan(), 20)).toBeNull();
  });

  it('keeps the line breaks the caption engine decided', () => {
    const cue = cueAt(plan(), 10);
    expect(cue).not.toBeNull();
    expect(cueLines(cue!)).toEqual(['the whole', 'point']);
  });
});

describe('the highlight', () => {
  it('waits out the lead-in before any word is lit', () => {
    const at = plan();
    const cue = at.cues[0]!;
    // The cue starts at frame 5 and leads in for 10 centiseconds, which is
    // about three frames at this rate.
    expect(highlightedWord(at, cue, 5)).toBe(-1);
  });

  it('advances word by word using the holds the sweep measured', () => {
    const at = plan();
    const cue = at.cues[0]!;
    const lit = [8, 14, 19].map((frame) => highlightedWord(at, cue, frame));
    // Monotonic, and never past the last word.
    expect(lit[0]).toBeLessThanOrEqual(lit[1]!);
    expect(lit[1]).toBeLessThanOrEqual(lit[2]!);
    expect(lit[2]).toBeLessThanOrEqual(2);
  });

  it('lights nothing on a cue with no animation', () => {
    const at = plan();
    const still = { ...at.cues[0]!, karaoke: false };
    expect(highlightedWord(at, still, 15)).toBe(-1);
  });
});

describe('frames and seconds', () => {
  it('round-trip through the rate the plan states', () => {
    const at = plan();
    for (const frame of [0, 1, 7, 29]) {
      expect(frameAt(at, secondsAt(at, frame))).toBe(frame);
    }
  });

  it('clamp to the program rather than running off it', () => {
    const at = plan();
    expect(frameAt(at, -10)).toBe(0);
    expect(frameAt(at, 9_999)).toBe(at.frameCount - 1);
  });

  it('read as a timecode an editor recognises', () => {
    const at = plan();
    expect(timecode(at, 0)).toBe('0:00.00');
    expect(timecode(at, 29)).toMatch(/^0:00\.\d\d$/);
  });
});

describe('the lanes', () => {
  it('put the first and last frame at the ends', () => {
    const at = plan();
    expect(lanePosition(at, 0)).toBe(0);
    expect(lanePosition(at, at.frameCount - 1)).toBe(100);
  });

  it('hold the gain from the last point before a frame', () => {
    const at = plan();
    expect(gainAt(at, 0)).toBe(0);
    expect(gainAt(at, 14)).toBe(0);
    expect(gainAt(at, 15)).toBe(-6);
    expect(gainAt(at, 29)).toBe(-6);
  });
});
