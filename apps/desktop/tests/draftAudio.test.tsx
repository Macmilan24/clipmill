import { act, renderHook } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { useDraftAudio } from '../src/editor/useDraftAudio.js';

afterEach(() => vi.unstubAllGlobals());
describe('draft audio gain', () => {
  it('connects once and applies the current decibel curve to real audio gain', () => {
    const setValueAtTime = vi.fn();
    const destination = {};
    const gain = { gain: { setValueAtTime }, connect: vi.fn(() => destination) };
    const source = { connect: vi.fn(() => gain) };
    const createSource = vi.fn(() => source);
    const close = vi.fn(() => Promise.resolve());
    class Context {
      readonly currentTime = 7;
      readonly destination = destination;
      readonly createGain = () => gain;
      readonly createMediaElementSource = createSource;
      readonly resume = () => Promise.resolve();
      readonly close = close;
    }
    vi.stubGlobal('AudioContext', Context);
    const video = { current: document.createElement('video') };
    const { result, rerender, unmount } = renderHook(({ db }) => useDraftAudio(video, db), {
      initialProps: { db: -6 },
    });
    act(() => result.current.connect());
    expect(createSource).toHaveBeenCalledExactlyOnceWith(video.current);
    expect(setValueAtTime).toHaveBeenLastCalledWith(10 ** (-6 / 20), 7);
    rerender({ db: 3 });
    expect(setValueAtTime).toHaveBeenLastCalledWith(10 ** (3 / 20), 7);
    act(() => result.current.connect());
    expect(createSource).toHaveBeenCalledTimes(1);
    unmount();
    expect(close).toHaveBeenCalledTimes(1);
  });
});
