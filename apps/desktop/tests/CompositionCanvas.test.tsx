import { act, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { CompositionCanvas, observeVideoFrames } from '../src/editor/CompositionCanvas.js';
import { plan } from './support/clips.js';
import type { PreviewPlan } from '../src/daemon/client.js';
import { mapping } from './support/plan.js';

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
    globalAlpha: 1,
  };
}

function canvasHarness(custom: Partial<PreviewPlan> = {}, paused = false, frame = 0) {
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
  video.state.paused = paused;
  video.state.currentTime = 600 + frame / 30;
  const initial: PreviewPlan = {
    ...plan(),
    crops: [
      [0, 0, 900, 800],
      [100, 40, 800, 600],
    ] as const,
    ...custom,
  };
  const onFrame = vi.fn((seconds: number) => Math.round((seconds - 600) * 30));
  const props = {
    video: { current: video.media },
    mediaKey: 'proxy-a',
    plan: initial,
    frame,
    onFrame,
    proxyUrls: new Map(initial.proxies.map((item) => [item.sourceFingerprint, 'proxy-a'])),
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

function softCutPlan(): Partial<PreviewPlan> {
  const rate = { frameCount: 60, rateNum: 30, rateDen: 1 };
  return {
    ...rate,
    ...mapping(rate),
    transitionTicks: 10800,
    transitions: [{ incomingSegmentId: 'seg_2', outgoingFrame: 29, firstFrame: 30, endFrame: 34 }],
    crops: Array.from({ length: 60 }, (_, frame) => [frame < 30 ? 0 : 100, 0, 540, 960] as const),
  };
}

function readyReference() {
  const reference = screen.getByTestId('transition-reference') as HTMLVideoElement;
  Object.defineProperties(reference, {
    videoWidth: { value: 960 },
    videoHeight: { value: 540 },
    readyState: { configurable: true, value: 4 },
  });
  fireEvent.loadedData(reference);
  return reference;
}

describe('soft cuts use an exact outgoing picture', () => {
  it('scrubs directly into a blend, holds until its reference loads, and applies the planned alpha', () => {
    const { buffer, destination, media } = canvasHarness(softCutPlan(), true, 31);
    // Starting here has no previously played picture to reuse.
    expect(destination.drawImage).not.toHaveBeenCalled();
    const draws: { source: unknown; alpha: number }[] = [];
    buffer.drawImage.mockImplementation((source) =>
      draws.push({ source, alpha: buffer.globalAlpha }),
    );
    const reference = readyReference();
    expect(reference.muted).toBe(true);
    expect(reference.currentTime).toBeCloseTo(600 + 29 / 30, 8);
    expect(draws.some((draw) => draw.source === reference)).toBe(true);
    expect(draws.some((draw) => draw.source === media)).toBe(true);
    expect(draws.at(-1)?.source).toBeInstanceOf(HTMLCanvasElement);
    expect(draws.at(-1)?.alpha).toBe(0.75);
    expect(destination.drawImage).toHaveBeenCalledOnce();
  });

  it('preloads the outgoing frame and blends only inside the planned frame interval', () => {
    const { media, buffer, destination, present } = canvasHarness(softCutPlan());
    readyReference();
    const blends: number[] = [];
    buffer.drawImage.mockImplementation((source) => {
      if (source instanceof HTMLCanvasElement) blends.push(buffer.globalAlpha);
    });
    destination.drawImage.mockClear();
    present(601);
    present(601 + 2 / 30);
    present(601 + 4 / 30);
    expect(blends).toEqual([1, 0.5]);
    expect(destination.drawImage).toHaveBeenCalledTimes(3);
    expect(buffer.drawImage.mock.calls.at(-1)?.[0]).toBe(media);
  });

  it('invalidates a prepared reference when a new revision changes the crop', () => {
    const { buffer, view, props } = canvasHarness(softCutPlan(), true, 31);
    readyReference();
    buffer.drawImage.mockClear();
    view.rerender(
      <CompositionCanvas
        {...props}
        plan={{
          ...props.plan,
          revision: 2,
          crops: props.plan.crops.map(() => [200, 0, 540, 960] as const),
        }}
      />,
    );
    const reference = screen.getByTestId('transition-reference');
    // Both incoming and held outgoing crops are rebuilt from r2, not r1's cache.
    expect(
      buffer.drawImage.mock.calls.some((call) => call[0] === reference && call[1] === 100),
    ).toBe(true);
  });
});
