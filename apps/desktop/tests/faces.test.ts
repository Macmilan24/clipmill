import { describe, expect, it } from 'vitest';

import type { FaceSighting } from '../src/daemon/client.js';
import { byTrack, facesAt, fittedFrame } from '../src/editor/faces.js';

const seen = (trackId: number, tTicks: number, x: number): FaceSighting => ({
  trackId,
  tTicks,
  x,
  y: 0.3,
  width: 0.1,
  height: 0.2,
});

describe('facesAt', () => {
  const tracks = byTrack([
    seen(1, 15_000, 0.7),
    seen(0, 0, 0.2),
    seen(0, 15_000, 0.3),
    seen(1, 0, 0.7),
    seen(0, 300_000, 0.4),
  ]);

  it('places a face between its two samples, left to right', () => {
    const now = facesAt(tracks, 7_500);
    expect(now.map((face) => face.trackId)).toEqual([0, 1]);
    expect(now[0]!.x).toBeCloseTo(0.25);
    expect(now[1]!.x).toBeCloseTo(0.7);
  });

  it('holds the nearest sample only while it is close', () => {
    expect(facesAt(tracks, 30_000).map((face) => face.trackId)).toEqual([0, 1]);
    expect(facesAt(tracks, 100_000)).toEqual([]);
    expect(facesAt(tracks, 290_000).map((face) => face.x)).toEqual([0.4]);
  });

  it('never bridges a long gap between samples', () => {
    const now = facesAt(tracks, 160_000);
    expect(now).toEqual([]);
  });
});

describe('fittedFrame', () => {
  it('letterboxes a wide recording on a tall stage', () => {
    const frame = fittedFrame(
      { displayWidth: 1920, displayHeight: 1080 },
      { width: 1080, height: 1920 },
    );
    expect(frame.width).toBe(1);
    expect(frame.height).toBeCloseTo((1080 / 1920) * (1080 / 1920));
    expect(frame.top + frame.height / 2).toBeCloseTo(0.5);
  });

  it('pillarboxes a recording taller than the stage', () => {
    const frame = fittedFrame(
      { displayWidth: 1080, displayHeight: 2400 },
      { width: 1080, height: 1920 },
    );
    expect(frame.height).toBe(1);
    expect(frame.left + frame.width / 2).toBeCloseTo(0.5);
  });
});
