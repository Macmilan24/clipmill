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
import { afterEach, describe, expect, it, vi } from 'vitest';

import { TooltipProvider } from '../src/components/ui/tooltip.js';
import type { PreviewPlan } from '../src/daemon/client.js';
import { Editor } from '../src/screens/Editor.js';
import { plan } from './support/clips.js';
import { TICKS, mapping } from './support/plan.js';

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

function show(initial: PreviewPlan, urls?: ReadonlyMap<string, string>) {
  const onApply = vi.fn();
  // The URL map is what the hook derives from the plan's proxies.
  const props = (current: PreviewPlan, docId = 'edt_A') => ({
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
    fireEvent.change(screen.getByRole('slider', { name: /scrub/i }), {
      target: { value: '450' },
    });
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
    fireEvent.change(screen.getByRole('slider', { name: /scrub/i }), {
      target: { value: '750' },
    });
    expect(screen.getByTestId('timecode').textContent).toContain('frame 750');
    // The tail is trimmed to ten seconds: frame 750 no longer exists.
    replan(program(300, 600));
    expect(screen.getByTestId('timecode').textContent).toContain('frame 299 of 300');
    // And the picture is still there, at the program's last frame.
    expect(video().currentTime).toBeCloseTo(600 + 299 / 30, 6);
    expect(screen.queryByText(/nothing to play/i)).toBeNull();
  });

  it('sends a trim in the segment’s own source ticks', () => {
    const { onApply } = show(program(900, 600));
    fireEvent.change(screen.getByRole('slider', { name: /scrub/i }), {
      target: { value: '150' },
    });
    fireEvent.mouseDown(screen.getByRole('tab', { name: /clip/i }));
    fireEvent.click(screen.getByRole('button', { name: /trim start here/i }));
    expect(onApply).toHaveBeenCalledWith({
      op: 'trim',
      segment_id: 'seg_1',
      in_ticks: 605 * TICKS,
      out_ticks: 630 * TICKS,
    });
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

describe('decoded playback across shots and documents', () => {
  let control: ReturnType<typeof playback>;
  afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
    control?.restore();
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
    fireEvent.change(screen.getByRole('slider', { name: /scrub/i }), {
      target: { value: '16' },
    });
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
  it('leaves arrow keys in caption fields and native scrubbers alone', () => {
    const { video } = show({ ...program(900, 600), cues: plan().cues });
    fireEvent.click(screen.getByRole('button', { name: 'Charging' }));
    const field = screen.getByRole('textbox', { name: /correct this word/i });
    fireEvent.keyDown(field, { key: 'ArrowRight' });
    expect(video().currentTime).toBe(600);
    fireEvent.keyDown(screen.getByRole('slider', { name: /scrub/i }), { key: 'ArrowRight' });
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

it('presents two portraits from one media element and edits the selected lower crop', () => {
  const initial = program(30, 600);
  const two: PreviewPlan = {
    ...initial,
    crops: Array.from({ length: 30 }, () => [0, 140, 900, 800] as const),
    secondaryCrops: Array.from({ length: 30 }, () => [1000, 140, 900, 800] as const),
  };
  const { onApply } = show(two);
  expect(document.querySelectorAll('video')).toHaveLength(1);
  expect(screen.getByRole('img', { name: /two synchronized portraits/i })).toBeTruthy();
  fireEvent.mouseDown(screen.getByRole('tab', { name: /reframe/i }));
  fireEvent.click(screen.getByRole('button', { name: 'Lower' }));
  fireEvent.click(screen.getByRole('button', { name: /move crop left/i }));
  expect(onApply).toHaveBeenLastCalledWith(
    expect.objectContaining({
      op: 'set_secondary_crop_keyframe',
      rect: { x: 984, y: 140, width: 900, height: 800 },
    }),
  );
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
  expect(screen.getByText(/Fast proxy preview with your gain edits/)).toBeTruthy();
});
