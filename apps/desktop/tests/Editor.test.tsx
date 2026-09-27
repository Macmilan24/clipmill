/**
 * The player, against a plan cut from ten minutes into a recording.
 *
 * The media element plays the proxy and the plan is in program frames, and
 * the two clocks meet in exactly one place. These check that place from the
 * outside: where the element is told to be when the clip opens, when the
 * playhead moves, and — the case the audit reproduced — when a trim changes
 * the mapping under a playhead that did not move.
 */
import { act, cleanup, fireEvent, render, screen } from '@testing-library/react';
import type { ComponentProps } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { TooltipProvider } from '../src/components/ui/tooltip.js';
import type { PreviewPlan } from '../src/daemon/client.js';
import { Editor } from '../src/screens/Editor.js';
import { plan } from './support/clips.js';
import { TICKS, mapping } from './support/plan.js';
import { box, drag, measure, measureTrack, seekTo, xOf } from './support/timeline.js';

const PROXY_URL = 'clipmill-media://localhost/p_old/sha256:proxy-old/proxy.mp4';

/** `frames` frames at 30 fps, cut from `inSeconds` into the recording. */
function program(frames: number, inSeconds: number): PreviewPlan {
  const rate = { frameCount: frames, rateNum: 30, rateDen: 1 };
  return {
    ...plan(),
    ...mapping(rate, inSeconds),
    frameCount: frames,
    crops: Array.from({ length: frames }, () => null),
    cues: [],
  };
}

const video = () => screen.getByTestId('proxy') as HTMLVideoElement;

function canvasContext() {
  return {
    clearRect: vi.fn(),
    drawImage: vi.fn(),
    save: vi.fn(),
    restore: vi.fn(),
  };
}

function show(
  initial: PreviewPlan,
  urls?: ReadonlyMap<string, string>,
  extra: Partial<ComponentProps<typeof Editor>> = {},
) {
  const onApply = vi.fn();
  // The URL map is what the hook derives from the plan's proxies.
  const props = (current: PreviewPlan, docId = 'edt_A') => ({
    ...extra,
    plan: current,
    proxyUrls:
      urls ?? new Map(current.proxies.map((proxy) => [proxy.sourceFingerprint, PROXY_URL])),
    docId,
    labels: { project: 'CUDA kernels', clip: 'Clip 01' },
    loading: false,
    problem: null,
    busy: false,
    canUndo: false,
    canRedo: false,
    resolving: false,
    resolveRefusal: null,
    picker: null,
    onOpenResults: () => {},
    onExport: null,
    onApply,
    onUndo: () => {},
    onRedo: () => {},
    onResolve: () => {},
  });
  const view = render(
    <TooltipProvider>
      <Editor {...props(initial)} />
    </TooltipProvider>,
  );
  return {
    onApply,
    video,
    replan: (next: PreviewPlan, docId = 'edt_A') =>
      view.rerender(
        <TooltipProvider>
          <Editor {...props(next, docId)} />
        </TooltipProvider>,
      ),
  };
}

describe('the player and the recording’s clock', () => {
  it('opens the clip where it begins in the proxy, not at the proxy’s start', () => {
    const { video } = show(program(900, 600));
    expect(video().currentTime).toBe(600);
    expect(video().dataset['startSeconds']).toBe('600');
  });

  it('scrubs and steps through the same mapping', () => {
    const { video } = show(program(900, 600));
    seekTo(program(900, 600), 450);
    expect(video().currentTime).toBe(615);
    expect(screen.getByTestId('timecode').textContent).toContain('frame 450');
    fireEvent.click(screen.getByRole('button', { name: /next frame/i }));
    expect(screen.getByTestId('timecode').textContent).toContain('frame 451');
    expect(video().currentTime).toBeCloseTo(615 + 1 / 30, 6);
  });

  it('moves the media element when a trim moves the segment under the playhead', () => {
    // The head is trimmed five seconds later. Program frame zero now plays
    // source second 605, and the element must be told so — the document id
    // and the proxy are unchanged, so nothing else would tell it.
    const { video, replan } = show(program(900, 600));
    expect(video().currentTime).toBe(600);
    replan(program(750, 605));
    expect(video().currentTime).toBe(605);
  });

  it('brings a playhead past the new end back onto the program', () => {
    const { video, replan } = show(program(900, 600));
    seekTo(program(900, 600), 750);
    expect(screen.getByTestId('timecode').textContent).toContain('frame 750');
    // The tail is trimmed to ten seconds: frame 750 no longer exists.
    replan(program(300, 600));
    expect(screen.getByTestId('timecode').textContent).toContain('frame 299 of 300');
    // And the picture is still there, at the program's last frame.
    expect(video().currentTime).toBeCloseTo(600 + 299 / 30, 6);
    expect(screen.queryByText(/nothing to play/i)).toBeNull();
  });

  it('sends a trim in the segment’s own source ticks', () => {
    const current = program(900, 600);
    const { onApply } = show(current);
    const track = measureTrack();
    drag(
      screen.getByRole('button', { name: 'Start of the clip' }),
      { x: xOf(current, 0) },
      { x: xOf(current, 5 * TICKS) },
    );
    expect(onApply).toHaveBeenCalledWith({
      op: 'trim',
      segment_id: 'seg_1',
      in_ticks: 605 * TICKS,
      out_ticks: 630 * TICKS,
    });
    track.mockRestore();
  });

  it('says there is nothing to play when the recording has no proxy', () => {
    show({ ...program(900, 600), proxies: [] });
    expect(screen.getByText(/no proxy/i)).toBeTruthy();
  });
});

/** Browser media state with explicitly delivered decoded-frame callbacks. */
function playback() {
  type MediaState = {
    time: number;
    paused: boolean;
    seeking: boolean;
    ended: boolean;
    ready: number;
    callbacks: Map<number, VideoFrameRequestCallback>;
  };
  const mediaStates = new WeakMap<HTMLMediaElement, MediaState>();
  const state = (media: HTMLMediaElement) => {
    let found = mediaStates.get(media);
    if (!found) {
      found = {
        time: 0,
        paused: true,
        seeking: false,
        ended: false,
        ready: 0,
        callbacks: new Map(),
      };
      mediaStates.set(media, found);
    }
    return found;
  };
  const media = HTMLMediaElement.prototype;
  const timeDescriptor = Object.getOwnPropertyDescriptor(media, 'currentTime')!;
  vi.spyOn(media, 'currentTime', 'get').mockImplementation(function (this: HTMLMediaElement) {
    return state(this).time;
  });
  const seeks = vi.spyOn(media, 'currentTime', 'set').mockImplementation(function (
    this: HTMLMediaElement,
    time,
  ) {
    state(this).time = time;
    state(this).seeking = true;
    state(this).ended = false;
  });
  vi.spyOn(media, 'paused', 'get').mockImplementation(function (this: HTMLMediaElement) {
    return state(this).paused;
  });
  vi.spyOn(media, 'seeking', 'get').mockImplementation(function (this: HTMLMediaElement) {
    return state(this).seeking;
  });
  vi.spyOn(media, 'ended', 'get').mockImplementation(function (this: HTMLMediaElement) {
    return state(this).ended;
  });
  vi.spyOn(media, 'readyState', 'get').mockImplementation(function (this: HTMLMediaElement) {
    return state(this).ready;
  });
  const play = vi.spyOn(media, 'play').mockImplementation(function (this: HTMLMediaElement) {
    // Native play() at EOF restarts the whole media resource unless the editor
    // first seeks back to the selected clip's source start.
    if (state(this).ended) {
      state(this).time = 0;
      state(this).ended = false;
    }
    state(this).paused = false;
    fireEvent.play(this);
    fireEvent.playing(this);
    return Promise.resolve();
  });
  const pause = vi.spyOn(media, 'pause').mockImplementation(function (this: HTMLMediaElement) {
    if (state(this).paused) return;
    state(this).paused = true;
    fireEvent.pause(this);
  });
  vi.spyOn(HTMLVideoElement.prototype, 'videoWidth', 'get').mockReturnValue(1280);
  vi.spyOn(HTMLVideoElement.prototype, 'videoHeight', 'get').mockReturnValue(720);
  vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue({
    clearRect: vi.fn(),
    drawImage: vi.fn(),
    save: vi.fn(),
    restore: vi.fn(),
  } as unknown as CanvasRenderingContext2D);
  let nextId = 0;
  const requestDescriptor = Object.getOwnPropertyDescriptor(
    HTMLVideoElement.prototype,
    'requestVideoFrameCallback',
  );
  const cancelDescriptor = Object.getOwnPropertyDescriptor(
    HTMLVideoElement.prototype,
    'cancelVideoFrameCallback',
  );
  Object.defineProperty(HTMLVideoElement.prototype, 'requestVideoFrameCallback', {
    configurable: true,
    value(this: HTMLVideoElement, callback: VideoFrameRequestCallback) {
      const id = ++nextId;
      state(this).callbacks.set(id, callback);
      return id;
    },
  });
  Object.defineProperty(HTMLVideoElement.prototype, 'cancelVideoFrameCallback', {
    configurable: true,
    value(this: HTMLVideoElement, id: number) {
      state(this).callbacks.delete(id);
    },
  });
  const restore = () => {
    // Getter and setter spies wrap the same descriptor; restore both together.
    Object.defineProperty(media, 'currentTime', timeDescriptor);
    for (const [name, descriptor] of [
      ['requestVideoFrameCallback', requestDescriptor],
      ['cancelVideoFrameCallback', cancelDescriptor],
    ] as const) {
      if (descriptor) Object.defineProperty(HTMLVideoElement.prototype, name, descriptor);
      else Reflect.deleteProperty(HTMLVideoElement.prototype, name);
    }
  };
  return {
    state,
    seeks,
    play,
    pause,
    restore,
    ready(element: HTMLVideoElement) {
      state(element).ready = 4;
      state(element).ended = false;
      fireEvent.loadedMetadata(element);
      state(element).seeking = false;
      fireEvent.loadedData(element);
      fireEvent.seeked(element);
      // Readiness events expose the requested clock; only rVFC confirms pixels.
      const next = state(element).callbacks.entries().next().value;
      expect(next, 'ready media must be subscribed before its decoded frame').toBeDefined();
      state(element).callbacks.delete(next![0]);
      act(() => next![1](0, { mediaTime: state(element).time } as VideoFrameCallbackMetadata));
    },
    finishSeek(element: HTMLVideoElement) {
      state(element).seeking = false;
      fireEvent.seeked(element);
    },
    decode(element: HTMLVideoElement, seconds: number) {
      const next = state(element).callbacks.entries().next().value;
      expect(next, 'playing video must have a pending decoded-frame callback').toBeDefined();
      state(element).time = seconds;
      state(element).callbacks.delete(next![0]);
      act(() => next![1](0, { mediaTime: seconds } as VideoFrameCallbackMetadata));
      return next![1];
    },
    end(element: HTMLVideoElement, seconds: number) {
      state(element).time = seconds;
      state(element).paused = true;
      state(element).ended = true;
      fireEvent.ended(element);
    },
  };
}

/** Four half-second shots, continuous unless the last shot starts elsewhere. */
function shots(lastStart = 601.5): PreviewPlan {
  const base = program(60, 600);
  return {
    ...base,
    segments: Array.from({ length: 4 }, (_unused, index) => {
      const start = index === 3 ? lastStart : 600 + index / 2;
      return {
        ...base.segments[0]!,
        segmentId: `shot_${index}`,
        inTicks: start * TICKS,
        outTicks: (start + 0.5) * TICKS,
        programStartTicks: index * 0.5 * TICKS,
        firstFrame: index * 15,
        endFrame: (index + 1) * 15,
      };
    }),
  };
}

function endingAtProxyEof(): PreviewPlan {
  const base = program(60, 600.01);
  return {
    ...base,
    segments: [{ ...base.segments[0]!, outTicks: 602 * TICKS }],
    proxies: [{ ...base.proxies[0]!, coverageEndTicks: 602 * TICKS, rateNum: 24, rateDen: 1 }],
  };
}

function softShots(): PreviewPlan {
  return {
    ...shots(),
    transitionTicks: 10_800,
    transitions: [{ incomingSegmentId: 'shot_1', outgoingFrame: 14, firstFrame: 15, endFrame: 19 }],
  };
}

describe('decoded playback across shots and documents', () => {
  let control: ReturnType<typeof playback>;
  afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
    control?.restore();
  });

  it('stops the playing transport when a soft-cut reference fails to load', () => {
    control = playback();
    show(softShots());
    const element = video();
    control.ready(element);
    fireEvent.click(screen.getByRole('button', { name: /^play$/i }));
    expect(control.state(element).paused).toBe(false);
    control.pause.mockClear();

    fireEvent.error(screen.getByTestId('transition-reference'));

    expect(control.state(element).paused).toBe(true);
    expect(control.pause).toHaveBeenCalledOnce();
    expect(screen.getByRole('button', { name: /^play$/i })).toBeTruthy();
    expect(screen.getByRole('alert').textContent).toContain('The preview could not be loaded');
  });

  it('buffers a slow soft-cut reference at the requested frame, then resumes when it is ready', () => {
    control = playback();
    show(softShots());
    const element = video();
    control.ready(element);
    fireEvent.click(screen.getByRole('button', { name: /^play$/i }));
    control.play.mockClear();

    const stale = control.decode(element, 600 + 16 / 30);
    expect(control.state(element).paused).toBe(true);
    expect(element.currentTime).toBeCloseTo(600 + 16 / 30, 8);
    expect(screen.getByTestId('timecode').textContent).toContain('frame 16 of 60');
    expect(screen.getByText(/preparing preview/i)).toBeTruthy();
    fireEvent.seeking(element);
    control.finishSeek(element);
    control.decode(element, 600 + 16 / 30);
    act(() => stale(0, { mediaTime: 600 + 22 / 30 } as VideoFrameCallbackMetadata));
    expect(screen.getByTestId('timecode').textContent).toContain('frame 16 of 60');
    expect(control.play).not.toHaveBeenCalled();

    control.ready(screen.getByTestId('transition-reference') as HTMLVideoElement);
    expect(control.play).toHaveBeenCalledOnce();
    expect(control.state(element).paused).toBe(false);
    expect(screen.queryByText(/preparing preview/i)).toBeNull();
    control.decode(element, 600 + 17 / 30);
    expect(screen.getByTestId('timecode').textContent).toContain('frame 17 of 60');
  });

  it('keeps the captions on the sound while a boost plays through Web Audio', () => {
    class Context {
      readonly currentTime = 0;
      readonly baseLatency = 0.02;
      readonly outputLatency = 0.08;
      readonly destination = {};
      readonly createGain = () => ({
        gain: { setValueAtTime: vi.fn() },
        connect: (next: unknown) => next,
      });
      readonly createMediaElementSource = () => ({ connect: (next: unknown) => next });
      readonly resume = () => Promise.resolve();
      readonly close = () => Promise.resolve();
    }
    vi.stubGlobal('AudioContext', Context);
    try {
      control = playback();
      const first = plan().cues[0]!;
      show({
        ...program(900, 600),
        gain: [{ frame: 0, gainDb: 3 }],
        cues: [
          { ...first, firstFrame: 0, endFrame: 30 },
          {
            ...first,
            cueId: 'cue_2',
            firstFrame: 30,
            endFrame: 60,
            lines: [[{ text: 'Afterwards', holdCentis: 100, wordId: 'w3' }]],
          },
        ],
      });
      const element = video();
      control.ready(element);
      fireEvent.click(screen.getByRole('button', { name: /^play$/i }));
      control.decode(element, 600 + 31 / 30);
      expect(screen.getByTestId('timecode').textContent).toContain('frame 31 of 900');
      // A tenth of a second behind at 30 fps: what is heard is frame 28's.
      expect(document.querySelector('.edit-caption')!.textContent).toContain('Charging');

      fireEvent.click(screen.getByRole('button', { name: /^pause$/i }));
      expect(document.querySelector('.edit-caption')!.textContent).toContain('Afterwards');
    } finally {
      vi.unstubAllGlobals();
    }
  });

  it('plays the music with the picture, at the render’s level for each frame', () => {
    control = playback();
    const hash = `sha256:${'3'.repeat(64)}`;
    const scored: PreviewPlan = {
      ...program(900, 600),
      music: {
        asset: hash,
        offsetTicks: 90_000,
        levels: [
          { frame: 0, gainDb: -20 },
          { frame: 60, gainDb: -20 },
          { frame: 90, gainDb: -40 },
        ],
      },
    };
    render(
      <TooltipProvider>
        <Editor
          plan={scored}
          proxyUrls={new Map(scored.proxies.map((proxy) => [proxy.sourceFingerprint, PROXY_URL]))}
          docId="edt_A"
          labels={null}
          loading={false}
          problem={null}
          busy={false}
          canUndo={false}
          canRedo={false}
          resolving={false}
          resolveRefusal={null}
          picker={null}
          onOpenResults={() => {}}
          onExport={null}
          onApply={() => {}}
          onUndo={() => {}}
          onRedo={() => {}}
          onResolve={() => {}}
          assets={{
            list: vi.fn().mockResolvedValue([]),
            bring: vi.fn().mockResolvedValue(null),
            url: (asset) => `asset://${asset}`,
          }}
        />
      </TooltipProvider>,
    );
    const music = screen.getByTestId('music') as HTMLAudioElement;
    expect(music.getAttribute('src')).toBe(`asset://${hash}`);
    expect(music.volume).toBeCloseTo(0.1, 6);
    const element = video();
    control.ready(element);
    fireEvent.click(screen.getByRole('button', { name: /^play$/i }));
    expect(control.state(music).paused).toBe(false);
    control.decode(element, 600 + 75 / 30);
    // A second in the music, and two and a half more of the program.
    expect(music.currentTime).toBeCloseTo(1 + 75 / 30, 3);
    expect(music.volume).toBeCloseTo(10 ** (-30 / 20), 6);
    fireEvent.click(screen.getByRole('button', { name: /^pause$/i }));
    expect(control.state(music).paused).toBe(true);
  });

  it('honors Pause during buffering instead of resuming when the reference arrives', () => {
    control = playback();
    show(softShots());
    const element = video();
    control.ready(element);
    fireEvent.click(screen.getByRole('button', { name: /^play$/i }));
    control.decode(element, 600 + 16 / 30);
    expect(control.state(element).paused).toBe(true);
    fireEvent.click(screen.getByRole('button', { name: /^pause$/i }));
    control.play.mockClear();
    fireEvent.seeking(element);
    control.finishSeek(element);
    control.decode(element, 600 + 16 / 30);

    control.ready(screen.getByTestId('transition-reference') as HTMLVideoElement);
    expect(control.play).not.toHaveBeenCalled();
    expect(control.state(element).paused).toBe(true);
    expect(screen.getByRole('button', { name: /^play$/i })).toBeTruthy();
    expect(screen.getByTestId('timecode').textContent).toContain('frame 16 of 60');
  });

  it('waits for the main seek pixels when the soft-cut reference finishes first', () => {
    control = playback();
    show(softShots());
    const element = video();
    control.ready(element);
    fireEvent.click(screen.getByRole('button', { name: /^play$/i }));
    control.decode(element, 600 + 16 / 30);
    fireEvent.seeking(element);
    control.play.mockClear();
    control.ready(screen.getByTestId('transition-reference') as HTMLVideoElement);
    expect(control.play).not.toHaveBeenCalled();
    control.finishSeek(element);
    expect(control.play).not.toHaveBeenCalled();
    expect(screen.getByText(/preparing preview/i)).toBeTruthy();
    control.decode(element, 600 + 16 / 30);
    expect(control.play).toHaveBeenCalledOnce();
    expect(control.state(element).paused).toBe(false);
    expect(screen.queryByText(/preparing preview/i)).toBeNull();
  });

  it('clears automatic resume after a buffered reference fails', () => {
    control = playback();
    show(softShots());
    const element = video();
    control.ready(element);
    fireEvent.click(screen.getByRole('button', { name: /^play$/i }));
    control.decode(element, 600 + 16 / 30);
    expect(control.state(element).paused).toBe(true);
    control.play.mockClear();
    const reference = screen.getByTestId('transition-reference') as HTMLVideoElement;
    fireEvent.error(reference);
    expect(screen.getByRole('alert').textContent).toContain('The preview could not be loaded');
    fireEvent.seeking(element);
    control.finishSeek(element);
    control.decode(element, 600 + 16 / 30);
    control.ready(reference);
    expect(control.play).not.toHaveBeenCalled();
    expect(control.state(element).paused).toBe(true);
    expect(screen.getByRole('button', { name: /^play$/i })).toBeTruthy();
  });

  it('keeps continuous shots playing without seeking, including delayed multi-cut callbacks', () => {
    control = playback();
    show(shots());
    const element = video();
    control.ready(element);
    fireEvent.click(screen.getByRole('button', { name: /^play$/i }));
    control.seeks.mockClear();

    control.decode(element, 600.5);
    expect(screen.getByTestId('timecode').textContent).toContain('frame 15 of 60');
    control.decode(element, 601.75);
    expect(screen.getByTestId('timecode').textContent).toContain('frame 52 of 60');
    expect(control.seeks).not.toHaveBeenCalled();
    expect(screen.getByRole('button', { name: /^pause$/i })).toBeTruthy();
  });

  it('holds the previous picture through a fractional shot boundary, including a Pause redraw', () => {
    control = playback();
    const visible = canvasContext();
    const offscreen = canvasContext();
    vi.mocked(HTMLCanvasElement.prototype.getContext).mockImplementation(function (
      this: HTMLCanvasElement,
    ) {
      return (this.dataset['testid'] === 'composition'
        ? visible
        : offscreen) as unknown as CanvasRenderingContext2D;
    });
    const base = program(30, 600);
    const segment = base.segments[0]!;
    show({
      ...base,
      segments: [
        { ...segment, outTicks: 600.52 * TICKS, endFrame: 16 },
        {
          ...segment,
          segmentId: 'incoming',
          inTicks: 600.52 * TICKS,
          programStartTicks: 0.52 * TICKS,
          firstFrame: 16,
        },
      ],
      crops: Array.from({ length: 30 }, (_, frame) => [frame < 16 ? 0 : 300, 0, 600, 1080]),
    });
    const element = video();
    control.ready(element);
    fireEvent.click(screen.getByRole('button', { name: /^play$/i }));
    control.decode(element, 600.5);
    visible.drawImage.mockClear();
    offscreen.drawImage.mockClear();

    control.decode(element, 600.52); // incoming pixels, but output frame 15 is still outgoing
    expect(visible.drawImage).not.toHaveBeenCalled();
    expect(screen.getByTestId('timecode').textContent).toContain('frame 15 of 30');
    fireEvent.click(screen.getByRole('button', { name: /^pause$/i }));
    expect(visible.drawImage).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: /^play$/i }));
    control.decode(element, 600.55);
    expect(visible.drawImage).toHaveBeenCalledOnce();
    expect(offscreen.drawImage).toHaveBeenCalledWith(
      expect.any(HTMLCanvasElement),
      200,
      0,
      400,
      720,
      0,
      0,
      1080,
      1920,
    );
    expect(screen.getByTestId('timecode').textContent).toContain('frame 16 of 30');
  });

  it('seeks a genuine source gap once and ignores seeking events and stale decoded callbacks', () => {
    control = playback();
    show(shots(610));
    const element = video();
    control.ready(element);
    fireEvent.click(screen.getByRole('button', { name: /^play$/i }));
    control.seeks.mockClear();

    const oldCallback = control.decode(element, 601.75);
    expect(control.seeks).toHaveBeenCalledExactlyOnceWith(610);
    expect(screen.getByTestId('timecode').textContent).toContain('frame 45 of 60');
    fireEvent.seeking(element);
    act(() => oldCallback(0, { mediaTime: 601.9 } as VideoFrameCallbackMetadata));
    fireEvent.timeUpdate(element);
    expect(control.seeks).toHaveBeenCalledTimes(1);
    expect(screen.getByTestId('timecode').textContent).toContain('frame 45 of 60');

    control.state(element).seeking = false;
    fireEvent.seeked(element);
    control.decode(element, 610.1);
    expect(screen.getByTestId('timecode').textContent).toContain('frame 48 of 60');
    expect(control.seeks).toHaveBeenCalledTimes(1);
  });

  it('loads a different source before seeking and resuming playback', () => {
    control = playback();
    const base = shots(120);
    const nextSource = 'sha256:another-source';
    const nextUrl = 'clipmill-media://localhost/p_old/sha256:proxy-next/proxy.mp4';
    const next: PreviewPlan = {
      ...base,
      segments: base.segments.map((segment, index) =>
        index === 3 ? { ...segment, sourceFingerprint: nextSource } : segment,
      ),
      sources: [...base.sources, { ...base.sources[0]!, sourceFingerprint: nextSource }],
      proxies: [...base.proxies, { ...base.proxies[0]!, sourceFingerprint: nextSource }],
    };
    show(
      next,
      new Map([
        [base.segments[0]!.sourceFingerprint, PROXY_URL],
        [nextSource, nextUrl],
      ]),
    );
    const element = video();
    control.ready(element);
    fireEvent.click(screen.getByRole('button', { name: /^play$/i }));
    control.seeks.mockClear();
    control.play.mockClear();

    control.decode(element, 601.5);
    expect(video().getAttribute('src')).toBe(nextUrl);
    expect(control.seeks).not.toHaveBeenCalled();
    expect(control.state(element).paused).toBe(true);
    expect(control.play).not.toHaveBeenCalled();
    control.ready(element);
    expect(control.seeks).toHaveBeenCalledExactlyOnceWith(120);
    expect(control.play).toHaveBeenCalledTimes(1);
    expect(screen.getByRole('button', { name: /^pause$/i })).toBeTruthy();
    control.decode(element, 120.1);
    expect(screen.getByTestId('timecode').textContent).toContain('frame 48 of 60');
  });

  it('repositions a playing proxy when a new revision changes its source mapping', () => {
    control = playback();
    const view = show(shots());
    const element = video();
    control.ready(element);
    fireEvent.click(screen.getByRole('button', { name: /^play$/i }));
    const oldCallback = control.decode(element, 600.5);
    control.seeks.mockClear();

    view.replan({ ...program(45, 605), revision: 1 });
    expect(control.seeks).toHaveBeenCalledExactlyOnceWith(605.5);
    fireEvent.seeking(element);
    act(() => oldCallback(0, { mediaTime: 601 } as VideoFrameCallbackMetadata));
    expect(screen.getByTestId('timecode').textContent).toContain('frame 15 of 45');
    control.state(element).seeking = false;
    fireEvent.seeked(element);
    control.decode(element, 605.6);
    expect(screen.getByTestId('timecode').textContent).toContain('frame 18 of 45');
    expect(control.seeks).toHaveBeenCalledTimes(1);
  });

  it('holds a paused requested frame when the lower-rate proxy decodes an earlier picture', () => {
    control = playback();
    show(shots());
    const element = video();
    control.ready(element);
    seekTo(shots(), 16);
    control.seeks.mockClear();
    control.state(element).time = 600.5;
    control.state(element).seeking = false;
    fireEvent.seeked(element);
    expect(screen.getByTestId('timecode').textContent).toContain('frame 16 of 60');
    expect(control.seeks).not.toHaveBeenCalled();
    expect(screen.getByRole('button', { name: /^play$/i })).toBeTruthy();
  });

  it('finishes at native EOF and replays the clip even when no final program frame was decoded', () => {
    control = playback();
    show(endingAtProxyEof());
    const element = video();
    control.ready(element);
    fireEvent.click(screen.getByRole('button', { name: /^play$/i }));
    control.decode(element, 602 - 1 / 24);
    expect(screen.getByTestId('timecode').textContent).toContain('frame 58 of 60');

    control.end(element, 602);
    expect(screen.getByTestId('timecode').textContent).toContain('frame 59 of 60');
    expect(screen.getByRole('button', { name: /^play$/i })).toBeTruthy();
    control.seeks.mockClear();
    fireEvent.click(screen.getByRole('button', { name: /^play$/i }));
    expect(control.seeks).toHaveBeenCalledExactlyOnceWith(600.01);
    expect(element.currentTime).toBe(600.01);
    expect(screen.getByTestId('timecode').textContent).toContain('frame 0 of 60');
  });

  it('continues into another source when the previous proxy ends between program frames', () => {
    control = playback();
    const base = endingAtProxyEof();
    const nextSource = 'sha256:source-after-eof';
    const nextUrl = 'clipmill-media://localhost/p_old/sha256:after-eof/proxy.mp4';
    const at: PreviewPlan = {
      ...base,
      frameCount: 90,
      crops: Array.from({ length: 90 }, () => null),
      segments: [
        ...base.segments,
        {
          ...base.segments[0]!,
          segmentId: 'after_eof',
          sourceFingerprint: nextSource,
          inTicks: 120 * TICKS,
          outTicks: 121 * TICKS,
          programStartTicks: 179_100,
          firstFrame: 60,
          endFrame: 90,
        },
      ],
      sources: [...base.sources, { ...base.sources[0]!, sourceFingerprint: nextSource }],
      proxies: [...base.proxies, { ...base.proxies[0]!, sourceFingerprint: nextSource }],
    };
    show(
      at,
      new Map([
        [base.segments[0]!.sourceFingerprint, PROXY_URL],
        [nextSource, nextUrl],
      ]),
    );
    const element = video();
    control.ready(element);
    fireEvent.click(screen.getByRole('button', { name: /^play$/i }));
    control.decode(element, 602 - 1 / 24);
    expect(screen.getByTestId('timecode').textContent).toContain('frame 58 of 90');
    control.seeks.mockClear();
    control.play.mockClear();

    control.end(element, 602);
    expect(video().getAttribute('src')).toBe(nextUrl);
    expect(screen.getByTestId('timecode').textContent).toContain('frame 60 of 90');
    expect(control.seeks).not.toHaveBeenCalled();
    expect(control.play).not.toHaveBeenCalled();
    control.ready(element);
    // Program frame60 starts 0.01s after this segment's fractional boundary.
    expect(control.seeks).toHaveBeenCalledExactlyOnceWith(120.01);
    expect(control.play).toHaveBeenCalledTimes(1);
    expect(screen.getByRole('button', { name: /^pause$/i })).toBeTruthy();
    control.decode(element, 120.11);
    expect(screen.getByTestId('timecode').textContent).toContain('frame 63 of 90');
  });

  it('stops the old video and its decoded callbacks when another document shares the source', async () => {
    control = playback();
    const view = show(shots());
    const oldElement = video();
    control.ready(oldElement);
    fireEvent.click(screen.getByRole('button', { name: /^play$/i }));
    const staleCallback = control.decode(oldElement, 600.5);
    control.pause.mockClear();

    await act(async () => view.replan(program(90, 610), 'edt_B'));
    expect(video()).not.toBe(oldElement);
    expect(control.state(oldElement).paused).toBe(true);
    expect(control.pause).toHaveBeenCalledTimes(1);
    expect(control.state(oldElement).callbacks.size).toBe(0);
    expect(video().currentTime).toBe(610);
    expect(screen.getByRole('button', { name: /^play$/i })).toBeTruthy();
    act(() => staleCallback(0, { mediaTime: 601.5 } as VideoFrameCallbackMetadata));
    expect(screen.getByTestId('timecode').textContent).toContain('frame 0 of 90');
  });
});

describe('editor interaction boundaries', () => {
  it('seeks on an audio lane click and adds a volume point only on a double-click', () => {
    const current = program(60, 600);
    const { onApply } = show(current);
    const track = measureTrack(120);
    const lane = measure(document.querySelector('.edit-audio'), box(120, 48));

    fireEvent.pointerDown(document.querySelector('.edit-audio')!, { clientX: 60, clientY: 24 });
    fireEvent.pointerUp(window, { clientX: 60, clientY: 24 });
    expect(onApply).not.toHaveBeenCalled();
    fireEvent.doubleClick(document.querySelector('.edit-audio')!, {
      clientX: xOf(current, 90_000, 120),
      clientY: 24,
    });
    expect(onApply).toHaveBeenCalledWith({ op: 'set_gain', t_ticks: 90_000, gain_db: 0 });
    track.mockRestore();
    lane.mockRestore();
  });

  it('moves a volume point as one undoable edit, and a click only selects it', () => {
    const current = { ...program(60, 600), gain: [{ frame: 15, gainDb: 2 }] };
    const { onApply } = show(current);
    const track = measureTrack(120);
    const lane = measure(document.querySelector('.edit-audio'), box(120, 48));
    const point = screen.getByRole('button', { name: /volume \+2\.0 db/i });

    fireEvent.pointerDown(point, { clientX: xOf(current, 45_000, 120), clientY: 20, button: 0 });
    fireEvent.pointerUp(window, { clientX: xOf(current, 45_000, 120), clientY: 20 });
    expect(onApply).not.toHaveBeenCalled();

    drag(point, { x: xOf(current, 45_000, 120), y: 20 }, { x: xOf(current, 117_000, 120), y: 12 });
    expect(onApply).toHaveBeenCalledWith({
      op: 'batch',
      commands: [
        { op: 'remove_gain_point', t_ticks: 45_000 },
        { op: 'set_gain', t_ticks: 117_000, gain_db: 6 },
      ],
    });
    track.mockRestore();
    lane.mockRestore();
  });

  it('leaves arrow keys in caption fields and sliders alone', () => {
    const { video } = show({ ...program(900, 600), cues: plan().cues });
    fireEvent.pointerDown(document.querySelector('.edit-cue')!, { button: 0 });
    fireEvent.click(screen.getByRole('button', { name: 'Charging' }));
    const field = screen.getByRole('textbox', { name: /correct this word/i });
    fireEvent.keyDown(field, { key: 'ArrowRight' });
    expect(video().currentTime).toBe(600);
    fireEvent.keyDown(screen.getByRole('slider', { name: 'Caption size' }), { key: 'ArrowRight' });
    expect(video().currentTime).toBe(600);
    fireEvent.keyDown(window, { key: 'ArrowRight' });
    expect(video().currentTime).toBeCloseTo(600 + 1 / 30, 6);
  });

  it('reports playback rejection without claiming the video is playing', async () => {
    const { video } = show(program(900, 600));
    const play = vi.spyOn(video(), 'play').mockRejectedValue(new Error('decode failed'));
    fireEvent.click(screen.getByRole('button', { name: /^play$/i }));
    expect((await screen.findByRole('alert')).textContent).toContain('Playback could not start');
    expect(screen.getByRole('button', { name: /^play$/i })).toBeTruthy();
    play.mockRestore();
  });

  it('shows an edit failure regardless of the active properties tab', () => {
    const initial = program(900, 600);
    const view = render(
      <TooltipProvider>
        <Editor
          plan={initial}
          proxyUrls={new Map()}
          docId="edit"
          labels={null}
          loading={false}
          problem="Could not save the edit"
          busy={false}
          canUndo={false}
          canRedo={false}
          resolving={false}
          resolveRefusal={null}
          picker={null}
          onOpenResults={() => {}}
          onExport={null}
          onApply={() => {}}
          onUndo={() => {}}
          onRedo={() => {}}
          onResolve={() => {}}
        />
      </TooltipProvider>,
    );
    expect(screen.getByRole('alert').textContent).toContain('Could not save the edit');
    view.unmount();
  });
});

it('presents two portraits from one media element and reframes the lower one by dragging it', () => {
  const initial = program(30, 600);
  const two: PreviewPlan = {
    ...initial,
    crops: Array.from({ length: 30 }, () => [0, 140, 900, 800] as const),
    secondaryCrops: Array.from({ length: 30 }, () => [1000, 140, 900, 800] as const),
  };
  const { onApply } = show(two);
  expect(document.querySelectorAll('video')).toHaveLength(1);
  expect(screen.getByRole('img', { name: /two synchronized portraits/i })).toBeTruthy();
  const grab = document.querySelector('.edit-frame-grab')!;
  const bounds = measure(grab, box(180, 320));
  drag(grab, { x: 90, y: 240 }, { x: 108, y: 240 });
  expect(onApply).toHaveBeenLastCalledWith(
    expect.objectContaining({
      op: 'set_secondary_crop_keyframe',
      rect: { x: 910, y: 140, width: 900, height: 800 },
    }),
  );
  bounds.mockRestore();
});

it('shows a stored framing problem over the draft regardless of the active tab', () => {
  const base = program(30, 600);
  show({
    ...base,
    segments: base.segments.map((segment) => ({
      ...segment,
      framingWarning: 'This shot has no saved crop path. Choose Fit.',
    })),
  });
  expect(screen.getByRole('alert').textContent).toContain('no saved crop path');
});

it('uses the renderer’s caption style and placement while labelling proxy audio as draft', () => {
  const base = plan();
  show({
    ...base,
    cues: [{ ...base.cues[0]!, region: 'upper_safe' }],
    captionStyle: {
      styleRef: 'clipmill.captions.minimal.v1',
      fontFamily: 'Inter',
      fontSize: 72,
      spoken: '#ffffffff',
      unspoken: '#ffffffff',
      outline: '#000000ff',
      shadow: '#000000ff',
      outlineWidth: 3,
      shadowDepth: 1,
      bold: false,
      boxed: false,
      marginHorizontal: 96,
      marginVertical: 240,
    },
  });
  const caption = screen.getByTestId('caption');
  expect(caption.style.fontWeight).toBe('400');
  expect(caption.style.top).toBe('12.5%');
  fireEvent.mouseDown(screen.getByRole('tab', { name: /brand/i }));
  expect(screen.getByText(/drawn from the render plan/i)).toBeTruthy();
});

it('keeps the current media and playhead when focusing the preview and returning with Escape', () => {
  show(program(900, 600));
  const originalVideo = video();
  seekTo(program(900, 600), 450);
  fireEvent.click(screen.getByRole('button', { name: 'Focus preview' }));
  expect(
    screen.getByRole('button', { name: 'Restore editing panels' }).getAttribute('aria-pressed'),
  ).toBe('true');
  expect(video()).toBe(originalVideo);
  expect(video().currentTime).toBe(615);

  fireEvent.keyDown(window, { key: 'Escape' });
  expect(screen.getByRole('button', { name: 'Focus preview' }).getAttribute('aria-pressed')).toBe(
    'false',
  );
  expect(video()).toBe(originalVideo);
  expect(screen.getByTestId('timecode').textContent).toContain('frame 450');
});

describe('captions made before a better transcript', () => {
  it('refreshes them from the transcript as one saved step', () => {
    const { onApply } = show(program(900, 600));
    fireEvent.mouseDown(screen.getByRole('tab', { name: /captions/i }));
    fireEvent.click(screen.getByRole('button', { name: 'Refresh captions' }));
    expect(onApply).toHaveBeenCalledWith({ op: 'refresh_captions' });
  });
});

describe('b-roll over the clip', () => {
  const PICTURE = `sha256:${'3'.repeat(64)}`;
  const RECORDING = `sha256:${'2'.repeat(64)}`;
  const assets = {
    list: async (kind: 'image' | 'audio') =>
      kind === 'image'
        ? [
            {
              hash: PICTURE,
              kind: 'image' as const,
              name: 'chart.png',
              mediaType: 'image/png',
              bytes: 10,
              width: 800,
              height: 600,
              durationTicks: 0,
              license: 'licensed' as const,
              addedUnixMillis: 1,
            },
          ]
        : [],
    bring: async () => null,
    url: (hash: string) => `media://assets/${hash}`,
  };
  const recordings = {
    list: async () => [{ fingerprint: RECORDING, name: 'interview.mp4' }],
  };

  it('puts a picture over the moment at the playhead, with its licence', async () => {
    const document = {
      version: 'ir/1',
      timebase: { num: 1, den: 90_000 },
      video: { segments: [] },
      captions: { style_ref: 'clean' },
      audio: { target_lufs: -14, true_peak_dbtp: -1 },
    } as never;
    const { onApply } = show(program(900, 600), undefined, { assets, recordings, document });
    seekTo(program(900, 600), 90);
    fireEvent.mouseDown(screen.getByRole('tab', { name: 'Framing' }));
    fireEvent.click(screen.getByRole('button', { name: 'Add a picture' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Show chart.png' }));
    expect(onApply).toHaveBeenLastCalledWith({
      op: 'set_cutaways',
      cutaways: [
        {
          cutaway_id: 'cut_1',
          start_ticks: 270_000,
          end_ticks: 495_000,
          content: { kind: 'picture', asset: PICTURE, push_in: true },
        },
      ],
      assets: [{ hash: PICTURE, license: 'licensed' }],
    });
  });

  it('cuts to footage from one of the project’s recordings', async () => {
    const { onApply } = show(program(900, 600), undefined, { assets, recordings });
    fireEvent.mouseDown(screen.getByRole('tab', { name: 'Framing' }));
    fireEvent.click(screen.getByRole('button', { name: 'Add footage' }));
    fireEvent.click(await screen.findByRole('button', { name: /interview\.mp4/ }));
    expect(onApply).toHaveBeenLastCalledWith({
      op: 'set_cutaways',
      cutaways: [
        {
          cutaway_id: 'cut_1',
          start_ticks: 0,
          end_ticks: 225_000,
          content: { kind: 'footage', source_fingerprint: RECORDING, in_ticks: 0 },
        },
      ],
    });
  });

  it('shows a cutaway over the picture, picks it there, changes it and deletes it', () => {
    const { onApply } = show(
      {
        ...program(900, 600),
        cutaways: [
          {
            cutawayId: 'cut_1',
            startTicks: 0,
            endTicks: 90_000,
            firstFrame: 0,
            endFrame: 30,
            fit: 'fill',
            kind: 'picture',
            asset: PICTURE,
            pushIn: true,
            inTicks: 0,
          },
        ],
      },
      undefined,
      { assets, recordings },
    );
    const cutaway = screen.getByTestId('cutaway');
    expect(cutaway.querySelector('img')?.getAttribute('src')).toBe(`media://assets/${PICTURE}`);
    fireEvent.pointerDown(cutaway, { button: 0 });
    expect(screen.getByRole('tab', { name: 'Framing' }).getAttribute('aria-selected')).toBe('true');
    expect(screen.getByRole('heading', { name: 'This cutaway' })).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Whole' }));
    expect(onApply).toHaveBeenLastCalledWith({
      op: 'set_cutaways',
      cutaways: [
        {
          cutaway_id: 'cut_1',
          start_ticks: 0,
          end_ticks: 90_000,
          fit: 'fit',
          content: { kind: 'picture', asset: PICTURE, push_in: true },
        },
      ],
    });
    fireEvent.click(screen.getByRole('button', { name: 'Still' }));
    expect(onApply).toHaveBeenLastCalledWith({
      op: 'set_cutaways',
      cutaways: [
        {
          cutaway_id: 'cut_1',
          start_ticks: 0,
          end_ticks: 90_000,
          content: { kind: 'picture', asset: PICTURE },
        },
      ],
    });
    fireEvent.keyDown(window, { key: 'Delete' });
    expect(onApply).toHaveBeenLastCalledWith({ op: 'set_cutaways', cutaways: [] });
  });
});

describe('text over the clip', () => {
  const hooked = (): PreviewPlan => ({
    ...program(900, 600),
    overlays: [
      {
        overlayId: 'ovl_1',
        startTicks: 0,
        endTicks: 270_000,
        firstFrame: 0,
        endFrame: 90,
        text: 'Charging less',
        role: 'hook',
        x: 500,
        y: 140,
        size: 88,
        colour: '#111111',
        plate: '#FFFFFF',
      },
    ],
  });

  it('adds a hook title named after the clip, as one saved step', () => {
    const { onApply } = show(program(900, 600));
    fireEvent.mouseDown(screen.getByRole('tab', { name: 'Text' }));
    fireEvent.click(screen.getByRole('button', { name: /add a hook title/i }));
    expect(onApply).toHaveBeenCalledWith({
      op: 'add_overlay',
      overlay: {
        overlay_id: 'ovl_1',
        start_ticks: 0,
        end_ticks: 270_000,
        content: {
          kind: 'text',
          text: 'Clip 01',
          role: 'hook',
          x: 500,
          y: 140,
          size: 88,
          colour: '#111111',
          plate: '#FFFFFF',
        },
      },
    });
  });

  it('shows the hook on the picture, picks it there, restyles and removes it', () => {
    const { onApply } = show(hooked());
    const text = screen.getByTestId('overlay-text');
    expect(text.textContent).toBe('Charging less');
    fireEvent.pointerDown(text, { button: 0, clientX: 10, clientY: 10 });
    fireEvent.pointerUp(window, { button: 0, clientX: 10, clientY: 10 });
    expect(screen.getByRole('tab', { name: 'Text' }).getAttribute('aria-selected')).toBe('true');
    expect(screen.getByRole('textbox', { name: 'Text words' })).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Red plate' }));
    expect(onApply).toHaveBeenLastCalledWith({
      op: 'set_overlay',
      overlay: {
        overlay_id: 'ovl_1',
        start_ticks: 0,
        end_ticks: 270_000,
        content: {
          kind: 'text',
          text: 'Charging less',
          role: 'hook',
          x: 500,
          y: 140,
          size: 88,
          colour: '#FFFFFF',
          plate: '#E0245E',
        },
      },
    });
    fireEvent.keyDown(window, { key: 'Delete' });
    expect(onApply).toHaveBeenLastCalledWith({ op: 'remove_overlay', overlay_id: 'ovl_1' });
  });

  it('puts an emoji at the playhead, drawn from its pinned picture', () => {
    const { onApply } = show(program(900, 600), undefined, {
      emojiUrl: (code) => `media://emoji/${code}.png`,
    });
    seekTo(program(900, 600), 90);
    fireEvent.mouseDown(screen.getByRole('tab', { name: 'Text' }));
    const fire = screen.getByRole('button', { name: 'Add Fire' });
    expect(fire.querySelector('img')?.getAttribute('src')).toBe('media://emoji/1f525.png');
    fireEvent.click(fire);
    expect(onApply).toHaveBeenLastCalledWith({
      op: 'add_overlay',
      overlay: {
        overlay_id: 'ovl_1',
        start_ticks: 270_000,
        end_ticks: 378_000,
        content: { kind: 'emoji', emoji: '1f525', x: 500, y: 640, size: 180 },
      },
    });
    // One row until the rest are asked for.
    expect(screen.queryByRole('button', { name: 'Add Skull' })).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'All 40' }));
    expect(screen.getByRole('button', { name: 'Add Skull' })).toBeTruthy();
  });

  it('puts emoji on the words that call for one, as one step', () => {
    const inTicks = 600 * 90_000;
    const said = (text: string, seconds: number) => ({
      text,
      startTicks: inTicks + seconds * 90_000,
      endTicks: inTicks + (seconds + 0.4) * 90_000,
    });
    const { onApply } = show(program(900, 600), undefined, {
      transcript: {
        words: [said('We', 0.5), said('made', 0.8), said('money', 1), said('then', 6)],
        sentences: [],
      },
    });
    fireEvent.mouseDown(screen.getByRole('tab', { name: 'Text' }));
    fireEvent.click(screen.getByRole('button', { name: /on key words/i }));
    expect(onApply).toHaveBeenLastCalledWith({
      op: 'batch',
      commands: [
        {
          op: 'add_overlay',
          overlay: {
            overlay_id: 'ovl_1',
            start_ticks: 90_000,
            end_ticks: 198_000,
            content: { kind: 'emoji', emoji: '1f4b0', x: 500, y: 640, size: 180 },
          },
        },
      ],
    });
    expect(screen.getByText(/added 1 where words call for one/i)).toBeTruthy();
  });

  it('shows an emoji on the picture, moves it there and changes it here', () => {
    const { onApply } = show({
      ...program(900, 600),
      overlays: [
        {
          overlayId: 'ovl_4',
          kind: 'emoji',
          emoji: '1f525',
          startTicks: 0,
          endTicks: 108_000,
          firstFrame: 0,
          endFrame: 36,
          text: '',
          role: 'label',
          x: 500,
          y: 640,
          size: 180,
          colour: '',
        },
      ],
    });
    const shown = screen.getByTestId('overlay-emoji');
    expect(screen.queryByTestId('overlay-text')).toBeNull();
    // Without a picture to load, the character stands in.
    expect(shown.textContent).toBe('🔥');
    fireEvent.pointerDown(shown, { button: 0, clientX: 10, clientY: 10 });
    fireEvent.pointerUp(window, { button: 0, clientX: 10, clientY: 10 });
    expect(screen.getByRole('tab', { name: 'Text' }).getAttribute('aria-selected')).toBe('true');
    expect(screen.getByRole('heading', { name: 'This emoji' })).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Top' }));
    expect(onApply).toHaveBeenLastCalledWith({
      op: 'set_overlay',
      overlay: {
        overlay_id: 'ovl_4',
        start_ticks: 0,
        end_ticks: 108_000,
        content: { kind: 'emoji', emoji: '1f525', x: 500, y: 140, size: 180 },
      },
    });
    fireEvent.click(screen.getByRole('button', { name: /remove all 1/i }));
    expect(onApply).toHaveBeenLastCalledWith({
      op: 'batch',
      commands: [{ op: 'remove_overlay', overlay_id: 'ovl_4' }],
    });
  });

  it('keeps the words a person types, with their line breaks', () => {
    const { onApply } = show(hooked());
    fireEvent.mouseDown(screen.getByRole('tab', { name: 'Text' }));
    fireEvent.click(screen.getByRole('button', { name: /charging less/i }));
    const words = screen.getByRole('textbox', { name: 'Text words' });
    fireEvent.change(words, { target: { value: 'Charge\nless' } });
    fireEvent.blur(words);
    expect(onApply).toHaveBeenLastCalledWith(
      expect.objectContaining({
        op: 'set_overlay',
        overlay: expect.objectContaining({
          content: expect.objectContaining({ text: 'Charge\nless' }),
        }),
      }),
    );
  });
});
