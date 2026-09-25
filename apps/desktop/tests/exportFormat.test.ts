/**
 * The export format a creator picks, and what the daemon is asked for.
 */
import { afterEach, describe, expect, it } from 'vitest';

import {
  DEFAULT_FORMAT,
  formatSummary,
  outputFormat,
  rateLabel,
  recallFormat,
  rememberFormat,
  sourceFpsOf,
} from '../src/export/format.js';

afterEach(() => localStorage.clear());

describe('the export format', () => {
  it('keeps the recording rate unless a rate is chosen', () => {
    expect(outputFormat(DEFAULT_FORMAT)).toEqual({
      frameRateNum: 0,
      frameRateDen: 0,
      height: 1920,
    });
    expect(outputFormat({ rate: '60', height: 3840 })).toEqual({
      frameRateNum: 60,
      frameRateDen: 1,
      height: 3840,
    });
  });

  it('remembers the last choice and ignores anything else in its place', () => {
    expect(recallFormat()).toEqual(DEFAULT_FORMAT);
    rememberFormat({ rate: '30', height: 2560 });
    expect(recallFormat()).toEqual({ rate: '30', height: 2560 });
    localStorage.setItem('clipmill.export.format', '{"rate":"120","height":1}');
    expect(recallFormat()).toEqual(DEFAULT_FORMAT);
  });

  it('reads the recording rate from its probe', () => {
    const map = JSON.stringify({
      streams: [{ kind: 'audio' }, { kind: 'video', video: { frame_rate: { num: 25, den: 1 } } }],
    });
    expect(sourceFpsOf(map)).toBe(25);
    expect(sourceFpsOf('not json')).toBeNull();
    expect(rateLabel('source', 25)).toBe('Match recording (25 fps)');
    expect(formatSummary(DEFAULT_FORMAT, 25)).toBe('1080 × 1920 · 25 fps · MP4');
    expect(formatSummary({ rate: '30', height: 3840 }, null)).toBe('2160 × 3840 · 30 fps · MP4');
  });
});
