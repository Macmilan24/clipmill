/**
 * The review station's arithmetic: positions as a reviewer reads them, the
 * crop a path puts on a frame, repeats, the next clip to review, and a ruler
 * that follows the zoom.
 */
import { describe, expect, it } from 'vitest';

import type { CropPath } from '../src/daemon/client.js';
import {
  clockTenths,
  cropRect,
  lengthLabel,
  nextUndecided,
  overlapsOf,
  progress,
  rulerStep,
  timecode,
} from '../src/inspector/review.js';
import type { ClipRow } from '../src/results/model.js';

const SECOND = 90_000;

function row(id: string, start: number, end: number, decision: ClipRow['decision'] = null) {
  return {
    candidateId: id,
    rank: Number(id.replace(/\D/g, '')) || 1,
    startTicks: start * SECOND,
    endTicks: end * SECOND,
    decision,
  } as ClipRow;
}

describe('positions', () => {
  it('reads as a timecode at the render rate', () => {
    expect(timecode(0)).toBe('00:00:00;00');
    expect(timecode(3_723 * SECOND + 3 * 3_003)).toBe('01:02:03;03');
  });

  it('reads to a tenth where a frame count would be noise', () => {
    expect(clockTenths(0)).toBe('0:00.0');
    expect(clockTenths(754.26 * SECOND)).toBe('12:34.3');
    expect(clockTenths(3_725.5 * SECOND)).toBe('1:02:05.5');
  });

  it('gives a length in seconds under a minute and as a clock past it', () => {
    expect(lengthLabel(26.66 * SECOND)).toBe('26.7 s');
    expect(lengthLabel(72.4 * SECOND)).toBe('1:12.4');
  });
});

describe('the crop a path puts on a frame', () => {
  const frame = { width: 1920, height: 1080 };
  const path: CropPath = {
    fit: false,
    fitReason: '',
    containment: 1,
    keyframes: [
      { tTicks: 0, centerX: 0.25, centerY: 0.5, scale: 1 },
      { tTicks: 10 * SECOND, centerX: 0.75, centerY: 0.5, scale: 0.5 },
    ],
  };

  it('is a 9:16 rectangle as tall as the scale, interpolated between keyframes', () => {
    const halfway = cropRect(path, 5 * SECOND, frame)!;
    expect(halfway.height).toBeCloseTo(1080 * 0.75);
    expect(halfway.width).toBeCloseTo((1080 * 0.75 * 9) / 16);
    expect(halfway.x + halfway.width / 2).toBeCloseTo(0.5 * 1920);
  });

  it('holds the first and last keyframes outside the path', () => {
    const before = cropRect(path, -SECOND, frame)!;
    const after = cropRect(path, 60 * SECOND, frame)!;
    expect(before.height).toBeCloseTo(1080);
    expect(after.height).toBeCloseTo(540);
  });

  it('keeps the rectangle inside the frame', () => {
    const edge: CropPath = {
      ...path,
      keyframes: [{ tTicks: 0, centerX: 0.99, centerY: 0.02, scale: 1 }],
    };
    const rect = cropRect(edge, 0, frame)!;
    expect(rect.x + rect.width).toBeCloseTo(1920);
    expect(rect.y).toBe(0);
  });

  it('is no rectangle at all for a fitted frame', () => {
    expect(cropRect({ ...path, fit: true }, 0, frame)).toBeNull();
    expect(cropRect(null, 0, frame)).toBeNull();
    expect(cropRect({ ...path, keyframes: [] }, 0, frame)).toBeNull();
  });
});

describe('repeats', () => {
  it('names the clips that cover the same ground, by share of the shorter one', () => {
    const rows = [row('c1', 0, 40), row('c2', 30, 50), row('c3', 10, 20), row('c4', 39, 90)];
    const found = overlapsOf(rows, rows[0]!);
    // c3 lies wholly inside c1; c2 shares half its length; c4 shares a sliver.
    expect(found.map((overlap) => overlap.candidateId)).toEqual(['c3', 'c2']);
    expect(found[0]!.share).toBe(1);
    expect(found[1]!.share).toBeCloseTo(0.5);
  });
});

describe('moving through a review', () => {
  it('goes to the next undecided clip, wrapping round and never back to this one', () => {
    const rows = [
      row('c1', 0, 1, 'approved'),
      row('c2', 2, 3),
      row('c3', 4, 5, 'rejected'),
      row('c4', 6, 7),
    ];
    expect(nextUndecided(rows, 'c2')).toBe('c4');
    expect(nextUndecided(rows, 'c4')).toBe('c2');
    expect(nextUndecided([row('c1', 0, 1)], 'c1')).toBeNull();
  });

  it('counts what has been decided, and how', () => {
    expect(
      progress([
        row('c1', 0, 1, 'approved'),
        row('c2', 2, 3, 'kept'),
        row('c3', 4, 5),
        row('c4', 6, 7, 'rejected'),
      ]),
    ).toEqual({ decided: 3, approved: 1, kept: 1, rejected: 1, total: 4 });
  });
});

describe('the ruler', () => {
  it('uses the finest step that keeps its labels apart', () => {
    // Sixty seconds across 600 pixels: ten pixels a second, so five-second
    // labels would sit 50 px apart and ten-second ones 100 px apart.
    expect(rulerStep(60 * SECOND, 600)).toBe(10 * SECOND);
    // Zoomed right in, tenths of a second.
    expect(rulerStep(4 * SECOND, 3_000)).toBe(0.1 * SECOND);
  });
});
