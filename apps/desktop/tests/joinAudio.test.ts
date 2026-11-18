import { afterEach, describe, expect, it, vi } from 'vitest';
import type { PreviewPlan } from '../src/daemon/client.js';
import { RoomSamples, scheduleFades, waveFormat } from '../src/editor/joinAudio.js';

afterEach(() => vi.unstubAllGlobals());

function waveHeader() {
  const bytes = new ArrayBuffer(44);
  const view = new DataView(bytes);
  const name = (at: number, s: string) =>
    new Uint8Array(bytes, at, s.length).set(new TextEncoder().encode(s));
  name(0, 'RIFF');
  name(8, 'WAVE');
  name(12, 'fmt ');
  name(36, 'data');
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true);
  view.setUint16(22, 2, true);
  view.setUint32(24, 48_000, true);
  view.setUint16(34, 16, true);
  return bytes;
}

describe('audio at a programme join', () => {
  it('schedules sub-frame quarter-sine ramps from a source seek and honors playback speed', () => {
    const setValueAtTime = vi.fn();
    const setValueCurveAtTime = vi.fn();
    const param = {
      setValueAtTime,
      setValueCurveAtTime,
      cancelScheduledValues: vi.fn(),
    } as unknown as AudioParam;
    scheduleFades(param, 5, 2, 0.025, 1, 0.05, 0.05);
    expect(setValueAtTime.mock.calls[0]![0]).toBeCloseTo(Math.SQRT1_2, 8);
    const [rising, when, remaining] = setValueCurveAtTime.mock.calls[0]!;
    expect(rising[0]).toBeCloseTo(Math.SQRT1_2);
    expect(rising[64]).toBe(1);
    expect(when).toBe(5);
    expect(remaining).toBe(0.0125);
    const [falling, cut] = setValueCurveAtTime.mock.calls[1]!;
    expect(falling[64]).toBe(0);
    expect(cut).toBeCloseTo(5 + (0.95 - 0.025) / 2);
  });

  it('reads only the verified stereo samples, reuses its read, and leaves no abrupt edge', async () => {
    expect(waveFormat(waveHeader())).toEqual({ rate: 48_000, channels: 2, offset: 44 });
    const fetch = vi.fn(async (_: string, options: RequestInit) => {
      const range = (options.headers as Record<string, string>).Range;
      if (range === 'bytes=0-4095') return new Response(waveHeader(), { status: 206 });
      expect(range).toBe('bytes=192044-211243');
      return new Response(new Int16Array(4800 * 2).fill(4096).buffer, { status: 206 });
    });
    vi.stubGlobal('fetch', fetch);
    const plan = {
      segments: [
        { sourceFingerprint: 'rec', fadeOutTicks: 4500 },
        { sourceFingerprint: 'rec', fadeInTicks: 4500, roomToneStartTicks: 135000 },
      ],
      proxies: [{ sourceFingerprint: 'rec', audioStartTicks: 45_000 }],
    } as unknown as PreviewPlan;
    const reader = new RoomSamples(plan, new Map([['rec', 'media:source']]));
    const [left, right] = (await reader.get(1))!;
    expect(left).toEqual(right);
    expect(left![0]).toBe(0);
    expect(left![1200]).toBeCloseTo(0.125 * Math.SQRT1_2, 5);
    expect(left![2400]).toBe(0.125);
    expect(left![3600]).toBeCloseTo(0.125 * Math.SQRT1_2, 5);
    await reader.get(1);
    expect(fetch).toHaveBeenCalledTimes(2);
  });
});
