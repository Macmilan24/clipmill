import { act, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
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
  beforeEach(() => {
    vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockImplementation(
      () => context() as unknown as CanvasRenderingContext2D,
    );
  });

  it('uses the presented frame timestamp and ignores competing timeupdate events', () => {
    const { media, state, present } = transport();
    const draw = vi.fn();
    const clock = observeVideoFrames(media, draw);
    draw.mockClear();
    state.currentTime = 601;
    present(600.5);
    media.dispatchEvent(new Event('timeupdate'));
    expect(draw.mock.calls.map(([seconds]) => seconds)).toEqual([600.5]);
    clock.dispose();
  });

  it('holds the last picture while seeking and rejects callbacks queued before the seek', () => {
    const { media, state, callbacks, present } = transport();
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
    expect(draw).not.toHaveBeenCalled();
    present(615);
    expect(draw.mock.calls.map(([seconds]) => seconds)).toEqual([615]);
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
    expect(draw.mock.calls.map(([seconds]) => seconds)).toEqual([600]);
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
    expect(draw.mock.lastCall?.[0]).toBe(600.5);
    media.pause();
    expect(callbacks.size).toBe(0);
    clock.redraw();
    expect(callbacks.size).toBe(0);
    clock.dispose();
  });

  it('keeps decoded pixels paired with their PTS when a redraw or pause races the live decoder', () => {
    const { media, state, present } = transport();
    let livePicture = 'outgoing';
    const pictures = new WeakMap<HTMLCanvasElement, string>();
    vi.mocked(HTMLCanvasElement.prototype.getContext).mockImplementation(function (
      this: HTMLCanvasElement,
    ) {
      return {
        drawImage: () => pictures.set(this, livePicture),
      } as unknown as CanvasRenderingContext2D;
    });
    const published: { seconds: number; picture: string | undefined }[] = [];
    const clock = observeVideoFrames(media, (seconds, picture) =>
      published.push({ seconds, picture: pictures.get(picture) }),
    );
    present(600);
    livePicture = 'incoming';
    state.currentTime = 600.04;
    clock.redraw(); // e.g. the separately decoded soft-cut reference arrives
    media.pause();
    media.dispatchEvent(new Event('loadeddata'));
    media.dispatchEvent(new Event('seeked'));
    expect(published).toHaveLength(5);
    expect(published.every((frame) => frame.seconds === 600 && frame.picture === 'outgoing')).toBe(
      true,
    );
    present(600.04);
    expect(published.at(-1)).toEqual({ seconds: 600.04, picture: 'incoming' });
    clock.dispose();
  });

  it('does not repaint an old decoded picture after seeking until the new decoded callback', () => {
    const { media, state, present } = transport();
    const draw = vi.fn();
    const clock = observeVideoFrames(media, draw);
    present(600);
    draw.mockClear();
    state.seeking = true;
    media.dispatchEvent(new Event('seeking'));
    state.currentTime = 900;
    state.seeking = false;
    media.dispatchEvent(new Event('seeked'));
    clock.redraw();
    expect(draw).not.toHaveBeenCalled();
    present(900);
    expect(draw.mock.calls.map(([seconds]) => seconds)).toEqual([900]);
    clock.dispose();
  });

  it('captures a paused seek result whose decoded callback arrives before seeked', () => {
    const { media, state, present, callbacks } = transport();
    state.paused = true;
    const draw = vi.fn();
    const clock = observeVideoFrames(media, draw);
    present(600);
    draw.mockClear();
    state.seeking = true;
    media.dispatchEvent(new Event('seeking'));
    expect(callbacks.size).toBe(1);
    state.currentTime = 900;
    present(900);
    expect(draw).not.toHaveBeenCalled();
    state.seeking = false;
    media.dispatchEvent(new Event('seeked'));
    expect(draw.mock.calls.map(([seconds]) => seconds)).toEqual([900]);
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
  const raw = context();
  const destination = context();
  vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockImplementation(function (
    this: HTMLCanvasElement,
  ) {
    return (this.dataset['testid'] === 'composition'
      ? destination
      : this.width === 960 && this.height === 540
        ? raw
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
  video.present(video.state.currentTime);
  return { ...video, raw, buffer, destination, onFrame, view, props };
}

describe('atomic preview composition', () => {
  it('draws the new frame with its own crop before React updates the frame prop', () => {
    const { buffer, state, present, props } = canvasHarness();
    buffer.drawImage.mockClear();
    state.currentTime = 609;
    present(600 + 1 / 30);
    expect(props.frame).toBe(0);
    expect(buffer.drawImage).toHaveBeenCalledWith(
      expect.any(HTMLCanvasElement),
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
    expect(buffer.drawImage).toHaveBeenCalledWith(
      expect.any(HTMLCanvasElement),
      100,
      0,
      400,
      300,
      0,
      0,
      1080,
      1920,
    );
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
    const { raw, buffer, destination, media } = canvasHarness(softCutPlan(), true, 31);
    // Starting here has no previously played picture to reuse.
    expect(destination.drawImage).not.toHaveBeenCalled();
    const draws: { source: unknown; alpha: number }[] = [];
    buffer.drawImage.mockImplementation((source) =>
      draws.push({ source, alpha: buffer.globalAlpha }),
    );
    const reference = readyReference();
    expect(reference.muted).toBe(true);
    expect(reference.currentTime).toBeCloseTo(600 + 29 / 30, 8);
    expect(raw.drawImage).toHaveBeenCalledWith(reference, 0, 0);
    expect(raw.drawImage).toHaveBeenCalledWith(media, 0, 0);
    expect(draws.every((draw) => draw.source instanceof HTMLCanvasElement)).toBe(true);
    expect(draws.at(-1)?.source).toBeInstanceOf(HTMLCanvasElement);
    expect(draws.at(-1)?.alpha).toBe(0.75);
    expect(destination.drawImage).toHaveBeenCalledOnce();
  });

  it('preloads the outgoing frame and blends only inside the planned frame interval', () => {
    const { buffer, destination, present } = canvasHarness(softCutPlan());
    readyReference();
    const blends: number[] = [];
    buffer.drawImage.mockImplementation((...args) => {
      if (args.length === 3) blends.push(buffer.globalAlpha);
    });
    destination.drawImage.mockClear();
    present(601);
    present(601 + 2 / 30);
    present(601 + 4 / 30);
    expect(blends).toEqual([1, 0.5]);
    expect(destination.drawImage).toHaveBeenCalledTimes(3);
    expect(buffer.drawImage.mock.calls.at(-1)).toHaveLength(9);
  });

  it('invalidates a prepared reference when a new revision changes the crop', () => {
    const { raw, buffer, view, props } = canvasHarness(softCutPlan(), true, 31);
    readyReference();
    buffer.drawImage.mockClear();
    raw.drawImage.mockClear();
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
    // Both incoming and held outgoing crops are rebuilt from r2, not r1's cache.
    const rawPictures = new Set(
      buffer.drawImage.mock.calls.filter((call) => call[1] === 100).map((call) => call[0]),
    );
    expect(rawPictures.size).toBe(2);
    expect(raw.drawImage).not.toHaveBeenCalled();
  });

  it('reuses a decoded reference when a trim changes its program frame but not its source time', () => {
    const prototype = HTMLVideoElement.prototype;
    const requestDescriptor = Object.getOwnPropertyDescriptor(
      prototype,
      'requestVideoFrameCallback',
    );
    const cancelDescriptor = Object.getOwnPropertyDescriptor(prototype, 'cancelVideoFrameCallback');
    const callbacks = new Map<number, VideoFrameRequestCallback>();
    let id = 0;
    Object.defineProperty(prototype, 'requestVideoFrameCallback', {
      configurable: true,
      writable: true,
      value: (callback: VideoFrameRequestCallback) => {
        callbacks.set(++id, callback);
        return id;
      },
    });
    Object.defineProperty(prototype, 'cancelVideoFrameCallback', {
      configurable: true,
      writable: true,
      value: (handle: number) => callbacks.delete(handle),
    });
    try {
      const { raw, buffer, destination, view, props } = canvasHarness(softCutPlan(), true, 31);
      const reference = readyReference();
      expect(destination.drawImage).not.toHaveBeenCalled();
      const [handle, callback] = [...callbacks.entries()][0]!;
      callbacks.delete(handle);
      act(() => callback(0, { mediaTime: reference.currentTime } as VideoFrameCallbackMetadata));
      expect(destination.drawImage).toHaveBeenCalledOnce();
      raw.drawImage.mockClear();
      buffer.drawImage.mockClear();
      destination.drawImage.mockClear();

      const rate = { frameCount: 45, rateNum: 30, rateDen: 1 };
      const trimmed: PreviewPlan = {
        ...props.plan,
        ...rate,
        ...mapping(rate, 600.5),
        revision: 2,
        crops: props.plan.crops.slice(15),
        transitions: [
          { incomingSegmentId: 'seg_2', outgoingFrame: 14, firstFrame: 15, endFrame: 19 },
        ],
      };
      view.rerender(<CompositionCanvas {...props} plan={trimmed} frame={16} onFrame={() => 16} />);
      // No new reference decode is due at the unchanged source position.
      expect(reference.currentTime).toBeCloseTo(600 + 29 / 30, 8);
      expect(raw.drawImage).not.toHaveBeenCalled();
      expect(destination.drawImage).toHaveBeenCalled();
      expect(buffer.drawImage.mock.calls.some((call) => call.length === 3)).toBe(true);
      view.unmount();
    } finally {
      for (const [name, descriptor] of [
        ['requestVideoFrameCallback', requestDescriptor],
        ['cancelVideoFrameCallback', cancelDescriptor],
      ] as const) {
        if (descriptor) Object.defineProperty(prototype, name, descriptor);
        else Reflect.deleteProperty(prototype, name);
      }
    }
  });
});
