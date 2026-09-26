import { act, renderHook } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { useDraftAudio } from '../src/editor/useDraftAudio.js';

afterEach(() => vi.unstubAllGlobals());

function stubContext() {
  const setValueAtTime = vi.fn();
  const destination = {};
  const gain = { gain: { setValueAtTime }, connect: vi.fn(() => destination) };
  const source = { connect: vi.fn(() => gain) };
  const createSource = vi.fn(() => source);
  const close = vi.fn(() => Promise.resolve());
  class Context {
    readonly currentTime = 7;
    readonly baseLatency = 0.01;
    readonly outputLatency = 0.04;
    readonly destination = destination;
    readonly createGain = () => gain;
    readonly createMediaElementSource = createSource;
    readonly resume = () => Promise.resolve();
    readonly close = close;
  }
  vi.stubGlobal('AudioContext', Context);
  return { setValueAtTime, createSource, close };
}

describe('draft audio gain', () => {
  it('turns the level down on the element itself, leaving sound and picture in sync', () => {
    const { createSource } = stubContext();
    const video = { current: document.createElement('video') };
    const { result, rerender } = renderHook(({ db }) => useDraftAudio(video, db, false), {
      initialProps: { db: -6 },
    });
    act(() => result.current.connect());
    expect(createSource).not.toHaveBeenCalled();
    expect(video.current.volume).toBeCloseTo(10 ** (-6 / 20), 6);
    expect(result.current.lag).toBe(0);
    rerender({ db: -12 });
    expect(video.current.volume).toBeCloseTo(10 ** (-12 / 20), 6);
  });

  it('routes a boost through Web Audio once, and reports the latency it adds', () => {
    const { setValueAtTime, createSource, close } = stubContext();
    const video = { current: document.createElement('video') };
    const { result, rerender, unmount } = renderHook(({ db }) => useDraftAudio(video, db, true), {
      initialProps: { db: -6 },
    });
    act(() => result.current.connect());
    expect(createSource).toHaveBeenCalledExactlyOnceWith(video.current);
    expect(video.current.volume).toBe(1);
    expect(setValueAtTime).toHaveBeenLastCalledWith(10 ** (-6 / 20), 7);
    expect(result.current.lag).toBeCloseTo(0.05, 6);
    rerender({ db: 3 });
    expect(setValueAtTime).toHaveBeenLastCalledWith(10 ** (3 / 20), 7);
    act(() => result.current.connect());
    expect(createSource).toHaveBeenCalledTimes(1);
    unmount();
    expect(close).toHaveBeenCalledTimes(1);
  });

  it('says a boost cannot be heard where Web Audio is missing, and still plays', () => {
    const video = { current: document.createElement('video') };
    vi.stubGlobal('AudioContext', undefined);
    const { result } = renderHook(() => useDraftAudio(video, 4, true));
    act(() => result.current.connect());
    expect(video.current.volume).toBe(1);
    expect(result.current.problem).toMatch(/cannot play a boost/i);
  });
});
