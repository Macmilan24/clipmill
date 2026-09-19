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
  frameAtProxySeconds,
  gainAt,
  highlightedWord,
  lanePosition,
  proxySecondsAt,
  resolvePlaybackFrame,
  secondsAt,
  segmentAt,
  sourceTicksAt,
  stageTransform,
  timecode,
} from '../src/editor/player.js';
import { TICKS, mapping } from './support/plan.js';

/** Thirty frames, a crop that moves, one karaoke cue, one gain step. */
function plan(): PreviewPlan {
  const program = { frameCount: 30, rateNum: 30_000, rateDen: 1_001 };
  return {
    ...mapping(program),
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
            { text: 'the', holdCentis: 20, wordId: 'w1' },
            { text: 'whole', holdCentis: 20, wordId: 'w2' },
          ],
          [{ text: 'point', holdCentis: 10, wordId: 'w3' }],
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

  it('interpolates gain in decibels and holds beyond the last point', () => {
    const at = plan();
    expect(gainAt(at, 0)).toBe(0);
    expect(gainAt(at, 14)).toBeCloseTo(-5.6);
    expect(gainAt(at, 15)).toBe(-6);
    expect(gainAt(at, 29)).toBe(-6);
  });
});

/**
 * The two clocks, kept apart.
 *
 * The plan is in program frames; the media element is in proxy seconds; the
 * clip is cut from ten minutes into the recording. Every conversion below is
 * the one a seek, a scrub or a trim makes, and every expected number is the
 * document's own `program_to_source` worked by hand.
 */
describe('mapping the program onto the recording', () => {
  it('finds the source tick a frame plays, through its segment', () => {
    expect(sourceTicksAt(plan(), 0)).toBe(600 * TICKS);
    // Frame 15 at 30000/1001 is 15 × 1001 / 30000 s = 0.5005 s = 45,045 ticks.
    expect(sourceTicksAt(plan(), 15)).toBe(600 * TICKS + 45_045);
    expect(segmentAt(plan(), 999)).toBeNull();
    expect(sourceTicksAt(plan(), 999)).toBeNull();
  });

  it('seeks the proxy to the recording\u2019s time, less where the proxy begins', () => {
    expect(proxySecondsAt(plan(), 0)).toBeCloseTo(600, 6);
    expect(proxySecondsAt(plan(), 15)).toBeCloseTo(600.5005, 6);
    // A proxy that covers the recording from ten seconds in starts its own
    // clock there.
    const offset: PreviewPlan = {
      ...plan(),
      proxies: [{ ...plan().proxies[0]!, coverageStartTicks: 10 * TICKS }],
    };
    expect(proxySecondsAt(offset, 0)).toBeCloseTo(590, 6);
    // No proxy is no answer, not second zero.
    expect(proxySecondsAt({ ...plan(), proxies: [] }, 0)).toBeNull();
  });

  it('reads the media element\u2019s time back as a program frame, clamped to the segment', () => {
    const segment = plan().segments[0]!;
    expect(frameAtProxySeconds(plan(), segment, 600)).toBe(0);
    expect(frameAtProxySeconds(plan(), segment, 600.5)).toBe(14);
    // Past the segment's end is its last frame; the player decides what
    // comes next.
    expect(frameAtProxySeconds(plan(), segment, 700)).toBe(29);
    expect(frameAtProxySeconds(plan(), segment, 0)).toBe(0);
  });

  it('builds the crop transform against the source frame, not the output', () => {
    // A 9:16 crop of a 1920×1080 source, 200 px from the left: full height,
    // so no scale, and the crop's centre (504) moved to the frame's (960).
    expect(
      stageTransform(
        { x: 200, y: 0, width: 608, height: 1080 },
        { displayWidth: 1920, displayHeight: 1080 },
      ),
    ).toBe('scale(1) translate(23.75%, 0%)');
    // Half the height: scaled up twice, centred on the crop.
    const halved = stageTransform(
      { x: 656, y: 270, width: 304, height: 540 },
      { displayWidth: 1920, displayHeight: 1080 },
    );
    const [, scale, dx, dy] = /scale\((.+)\) translate\((.+)%, (.+)%\)/.exec(halved ?? '') ?? [];
    expect(Number(scale)).toBe(2);
    expect(Number(dx)).toBeCloseTo(7.9167, 3);
    expect(Number(dy)).toBe(0);
    expect(
      stageTransform(
        { x: 0, y: 0, width: 0, height: 0 },
        { displayWidth: 1920, displayHeight: 1080 },
      ),
    ).toBeUndefined();
  });
});

describe('duration labels at the program boundary', () => {
  it('does not clamp the end to the last playable frame', () => {
    const program = { frameCount: 1260, rateNum: 30, rateDen: 1 } as PreviewPlan;
    expect(timecode(program, 1260)).toBe('0:42.00');
  });
});

/** Source intervals in ticks relative to a recording's ten-minute mark. */
function cutPlan(
  intervals: readonly (readonly [number, number])[],
  rateNum = 30_000,
  rateDen = 1_001,
): PreviewPlan {
  const base = plan();
  let programStartTicks = 0;
  const segments = intervals.map(([start, end], index) => {
    const segment = {
      ...base.segments[0]!,
      segmentId: `cut_${index}`,
      inTicks: 600 * TICKS + start,
      outTicks: 600 * TICKS + end,
      programStartTicks,
      firstFrame: Math.ceil((programStartTicks * rateNum) / (TICKS * rateDen)),
      endFrame: Math.ceil(((programStartTicks + end - start) * rateNum) / (TICKS * rateDen)),
    };
    programStartTicks += end - start;
    return segment;
  });
  return {
    ...base,
    segments,
    rateNum,
    rateDen,
    frameCount: Math.ceil((programStartTicks * rateNum) / (TICKS * rateDen)),
  };
}

describe('advancing a playing proxy', () => {
  it('crosses an exact camera cut without seeking', () => {
    const at = cutPlan(
      [
        [0, TICKS / 2],
        [TICKS / 2, TICKS],
      ],
      30,
      1,
    );
    expect(resolvePlaybackFrame(at, 14, 600.5 - 1 / TICKS)).toEqual({
      frame: 14,
      seek: false,
      ended: false,
    });
    expect(resolvePlaybackFrame(at, 14, 600.5)).toEqual({ frame: 15, seek: false, ended: false });
  });

  it('crosses several short shots when a decoded-frame callback arrives late', () => {
    const at = cutPlan(
      [
        [0, 18_000],
        [18_000, 36_000],
        [36_000, 54_000],
        [54_000, 90_000],
      ],
      30,
      1,
    );
    expect(resolvePlaybackFrame(at, 2, 600.75)).toEqual({ frame: 22, seek: false, ended: false });
  });

  it.each([
    ['one-tick source gap', 45_001, 90_001],
    ['one-tick source overlap', 44_999, 89_999],
    ['omitted source footage', 90_000, 135_000],
    ['repeated source footage', 0, 45_000],
    ['backward source edit', -45_000, 0],
  ] as const)('seeks for a %s, even with adjacent timeline frames', (_label, start, end) => {
    const at = cutPlan(
      [
        [0, 45_000],
        [start, end],
      ],
      30,
      1,
    );
    expect(resolvePlaybackFrame(at, 14, 600.5)).toEqual({ frame: 15, seek: true, ended: false });
  });

  it('does not consume a delayed callback past a genuine edit after continuous cuts', () => {
    const at = cutPlan(
      [
        [0, 18_000],
        [18_000, 36_000],
        [90_000, 135_000],
      ],
      30,
      1,
    );
    expect(resolvePlaybackFrame(at, 0, 610)).toEqual({ frame: 12, seek: true, ended: false });
  });

  it('seeks when identical source times refer to another source', () => {
    const base = cutPlan([
      [0, 45_000],
      [45_000, 90_000],
    ]);
    const at = {
      ...base,
      segments: [base.segments[0]!, { ...base.segments[1]!, sourceFingerprint: 'another-source' }],
    };
    expect(resolvePlaybackFrame(at, 14, 600.5)).toEqual({ frame: 15, seek: true, ended: false });
  });

  it.each([
    ['tick timeline gap', { programStartTicks: 45_001 }],
    ['tick timeline overlap', { programStartTicks: 44_999 }],
    ['frame timeline gap', { firstFrame: 16 }],
    ['frame timeline overlap', { firstFrame: 14 }],
  ])('does not infer continuity across a %s', (_label, change) => {
    const base = cutPlan(
      [
        [0, 45_000],
        [45_000, 90_000],
      ],
      30,
      1,
    );
    const following = { ...base.segments[1]!, ...change };
    const at = { ...base, segments: [base.segments[0]!, following] };
    expect(resolvePlaybackFrame(at, 13, 600.5)).toEqual({
      frame: following.firstFrame,
      seek: true,
      ended: false,
    });
  });

  it('honors the proxy coverage offset across continuous shots', () => {
    const base = cutPlan([
      [0, 45_000],
      [45_000, 90_000],
    ]);
    const at = {
      ...base,
      proxies: [{ ...base.proxies[0]!, coverageStartTicks: 100 * TICKS }],
    };
    expect(resolvePlaybackFrame(at, 0, 500.75)).toEqual({ frame: 22, seek: false, ended: false });
  });

  it.each([
    [24, 1],
    [30_000, 1_001],
  ])('holds the picture when %i/%i decoded and program shot boundaries disagree', (num, den) => {
    const at = cutPlan(
      [
        [0, 46_800],
        [46_800, 93_600],
      ],
      num,
      den,
    );
    const nextFrame = at.segments[1]!.firstFrame;
    expect(resolvePlaybackFrame(at, 0, 600.52 - 1 / TICKS)).toEqual({
      frame: nextFrame - 1,
      seek: false,
      ended: false,
    });
    expect(resolvePlaybackFrame(at, 0, 600.52)).toEqual({
      frame: nextFrame - 1,
      seek: false,
      ended: false,
      hold: true,
    });
    const nextTime = 600 + secondsAt(at, nextFrame);
    expect(resolvePlaybackFrame(at, nextFrame - 1, nextTime - 1 / TICKS)).toEqual({
      frame: nextFrame - 1,
      seek: false,
      ended: false,
      hold: true,
    });
    expect(resolvePlaybackFrame(at, nextFrame - 1, nextTime)).toEqual({
      frame: nextFrame,
      seek: false,
      ended: false,
    });
  });

  it('does not flash the outgoing crop at a shot cut minutes into the recording', () => {
    // The saved edit starts just before a source frame. Its first camera cut
    // lands at program frame 38.016, while the incoming layout starts at 39.
    const sourceStart = 204_852_600;
    const cut = 204_966_762;
    const nextCut = 205_131_927;
    const base = cutPlan([
      [0, cut - sourceStart],
      [cut - sourceStart, nextCut - sourceStart],
    ]);
    const at = {
      ...base,
      segments: base.segments.map((segment) => ({
        ...segment,
        inTicks: segment.inTicks - 600 * TICKS + sourceStart,
        outTicks: segment.outTicks - 600 * TICKS + sourceStart,
      })),
    };
    const resolved = resolvePlaybackFrame(at, 37, cut / TICKS);
    expect(resolved).toEqual({ frame: 38, seek: false, ended: false, hold: true });
    const aligned = resolvePlaybackFrame(at, resolved.frame, (cut + 3003) / TICKS);
    expect(aligned).toEqual({ frame: 39, seek: false, ended: false });
    expect(segmentAt(at, aligned.frame)).toBe(at.segments[1]);
  });

  it('seeks at the exact source boundary for a fractional-frame discontinuity', () => {
    const at = cutPlan([
      [0, 46_800],
      [90_000, 136_800],
    ]);
    expect(resolvePlaybackFrame(at, 15, 600.52)).toEqual({ frame: 16, seek: true, ended: false });
  });

  it('stops at the actual final source boundary, including a partial final frame', () => {
    const at = cutPlan([
      [0, 46_800],
      [46_800, 93_600],
    ]);
    expect(at.frameCount).toBe(32);
    expect(resolvePlaybackFrame(at, 31, 601.04 - 1 / TICKS)).toEqual({
      frame: 31,
      seek: false,
      ended: false,
    });
    expect(resolvePlaybackFrame(at, 31, 601.04)).toEqual({ frame: 31, seek: false, ended: true });
    // The callback can also overshoot the entire chain while React still has
    // the first shot's frame. The result must not seek backward through cuts.
    expect(resolvePlaybackFrame(at, 0, 610)).toEqual({ frame: 31, seek: false, ended: true });
  });

  it('does not move into earlier segments for a stale time before the current source span', () => {
    const at = cutPlan(
      [
        [0, 45_000],
        [90_000, 135_000],
      ],
      30,
      1,
    );
    expect(resolvePlaybackFrame(at, 15, 600.5)).toEqual({ frame: 15, seek: false, ended: false });
  });

  it('holds its frame when the source proxy is unavailable', () => {
    expect(resolvePlaybackFrame({ ...plan(), proxies: [] }, 14, 601)).toEqual({
      frame: 14,
      seek: false,
      ended: false,
    });
  });
});
