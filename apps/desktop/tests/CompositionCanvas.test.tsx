import { act, render } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { CompositionCanvas, observeVideoFrames } from '../src/editor/CompositionCanvas.js';
import { plan } from './support/clips.js';

function transport(decoded = true) {
  const media = document.createElement('video');
  const state = { paused: false, seeking: false, readyState: 4, currentTime: 600 };
  for (const field of Object.keys(state) as (keyof typeof state)[])
    Object.defineProperty(media, field, { configurable: true, get: () => state[field] });
  Object.defineProperties(media, {
    videoWidth: { value: 960 },
    videoHeight: { value: 540 },
  });
  const callbacks = new Map<number, VideoFrameRequestCallback>();
  let id = 0;
  if (decoded) {
    media.requestVideoFrameCallback = vi.fn((callback) => {
      callbacks.set(++id, callback);
      return id;
    });
    media.cancelVideoFrameCallback = vi.fn((handle) => callbacks.delete(handle));
  }
  media.pause = vi.fn(() => {
    state.paused = true;
    media.dispatchEvent(new Event('pause'));
  });
  const present = (mediaTime: number) => {
    const [handle, callback] = [...callbacks.entries()][0]!;
    callbacks.delete(handle);
    act(() => callback(0, { mediaTime } as VideoFrameCallbackMetadata));
  };
  return { media, state, callbacks, present };
}

afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

describe('decoded-frame clock', () => {
  it('uses the presented frame timestamp and ignores competing timeupdate events', () => {
    const { media, state, present } = transport();
    const draw = vi.fn();
    const clock = observeVideoFrames(media, draw);
    draw.mockClear();
    state.currentTime = 601;
    present(600.5);
    media.dispatchEvent(new Event('timeupdate'));
    expect(draw.mock.calls).toEqual([[600.5]]);
    clock.dispose();
  });

  it('holds the last picture while seeking and rejects callbacks queued before the seek', () => {
    const { media, state, callbacks } = transport();
    const draw = vi.fn();
    const clock = observeVideoFrames(media, draw);
    const stale = [...callbacks.values()][0]!;
    draw.mockClear();
    state.seeking = true;
    media.dispatchEvent(new Event('seeking'));
    clock.redraw();
    stale(0, { mediaTime: 600.1 } as VideoFrameCallbackMetadata);
    expect(draw).not.toHaveBeenCalled();
    state.seeking = false;
    state.currentTime = 615;
    media.dispatchEvent(new Event('seeked'));
    stale(0, { mediaTime: 600.2 } as VideoFrameCallbackMetadata);
    expect(draw.mock.calls).toEqual([[615]]);
    clock.dispose();
    expect(callbacks.size).toBe(0);
  });

  it('receives the first decoded frame and seek result even while paused', () => {
    const { media, state, present, callbacks } = transport();
    state.paused = true;
    state.readyState = 1;
    const draw = vi.fn();
    const clock = observeVideoFrames(media, draw);
    expect(draw).not.toHaveBeenCalled();
    expect(callbacks.size).toBe(1);
    state.readyState = 4;
    present(600);
    expect(draw).toHaveBeenCalledExactlyOnceWith(600);
    expect(callbacks.size).toBe(1);
    media.dispatchEvent(new Event('pause'));
    expect(callbacks.size).toBe(1);
    clock.dispose();
    expect(callbacks.size).toBe(0);
  });

  it('does not keep an animation loop alive while paused without decoded-frame support', () => {
    const { media, state } = transport(false);
    const callbacks = new Map<number, FrameRequestCallback>();
    let id = 0;
    vi.stubGlobal(
      'requestAnimationFrame',
      vi.fn((callback: FrameRequestCallback) => {
        callbacks.set(++id, callback);
        return id;
      }),
    );
    vi.stubGlobal(
      'cancelAnimationFrame',
      vi.fn((handle: number) => callbacks.delete(handle)),
    );
    const draw = vi.fn();
    const clock = observeVideoFrames(media, draw);
    expect(callbacks.size).toBe(1);
    state.currentTime = 600.5;
    const [handle, callback] = [...callbacks.entries()][0]!;
    callbacks.delete(handle);
    callback(0);
    expect(draw).toHaveBeenLastCalledWith(600.5);
    media.pause();
    expect(callbacks.size).toBe(0);
    clock.redraw();
    expect(callbacks.size).toBe(0);
    clock.dispose();
  });
});

function context() {
  return {
    clearRect: vi.fn(),
    drawImage: vi.fn(),
    save: vi.fn(),
    restore: vi.fn(),
    filter: 'none',
    globalCompositeOperation: 'source-over',
  };
}

function canvasHarness() {
  const buffer = context();
  const destination = context();
  vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockImplementation(function (
    this: HTMLCanvasElement,
  ) {
    return (this.dataset['testid'] === 'composition'
      ? destination
      : buffer) as unknown as CanvasRenderingContext2D;
  });
  const video = transport();
  const initial = {
    ...plan(),
    crops: [
      [0, 0, 900, 800],
      [100, 40, 800, 600],
    ] as const,
  };
  const onFrame = vi.fn((seconds: number) => Math.round((seconds - 600) * 30));
  const props = {
    video: { current: video.media },
    mediaKey: 'proxy-a',
    plan: initial,
    frame: 0,
    onFrame,
  };
  const view = render(<CompositionCanvas {...props} />);
  return { ...video, buffer, destination, onFrame, view, props };
}

describe('atomic preview composition', () => {
  it('draws the new frame with its own crop before React updates the frame prop', () => {
    const { buffer, state, present, props } = canvasHarness();
    buffer.drawImage.mockClear();
    state.currentTime = 609;
    present(600 + 1 / 30);
    expect(props.frame).toBe(0);
    expect(buffer.drawImage).toHaveBeenCalledWith(
      props.video.current,
      50,
      20,
      400,
      300,
      0,
      0,
      1080,
      1920,
    );
  });

  it('replaces the complete bitmap and leaves the previous one intact on decode failure', () => {
    const { buffer, destination, present } = canvasHarness();
    expect(destination.clearRect).not.toHaveBeenCalled();
    const modes: string[] = [];
    destination.drawImage.mockImplementation(() =>
      modes.push(destination.globalCompositeOperation),
    );
    present(600 + 1 / 30);
    expect(modes).toEqual(['copy']);
    const published = destination.drawImage.mock.calls.length;
    buffer.drawImage.mockImplementation(() => {
      throw new DOMException('No decoded frame', 'InvalidStateError');
    });
    present(600 + 2 / 30);
    expect(destination.drawImage).toHaveBeenCalledTimes(published);
  });

  it('redraws paused crop edits and stops the old transport on replacement/unmount', () => {
    const { media, state, buffer, props, view, callbacks } = canvasHarness();
    state.paused = true;
    act(() => media.dispatchEvent(new Event('pause')));
    buffer.drawImage.mockClear();
    view.rerender(
      <CompositionCanvas {...props} plan={{ ...props.plan, crops: [[200, 0, 800, 600]] }} />,
    );
    expect(buffer.drawImage).toHaveBeenCalledWith(media, 100, 0, 400, 300, 0, 0, 1080, 1920);
    state.paused = false;
    act(() => media.dispatchEvent(new Event('playing')));
    const stale = [...callbacks.values()][0]!;
    const count = buffer.drawImage.mock.calls.length;
    view.unmount();
    expect(media.pause).toHaveBeenCalled();
    stale(0, { mediaTime: 601 } as VideoFrameCallbackMetadata);
    expect(buffer.drawImage).toHaveBeenCalledTimes(count);
  });
});
