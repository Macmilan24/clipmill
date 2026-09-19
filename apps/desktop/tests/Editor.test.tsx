/**
 * The player, against a plan cut from ten minutes into a recording.
 *
 * The media element plays the proxy and the plan is in program frames, and
 * the two clocks meet in exactly one place. These check that place from the
 * outside: where the element is told to be when the clip opens, when the
 * playhead moves, and — the case the audit reproduced — when a trim changes
 * the mapping under a playhead that did not move.
 */
import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

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

function show(initial: PreviewPlan) {
  const onApply = vi.fn();
  // The URL map is what the hook derives from the plan's proxies.
  const props = (current: PreviewPlan) => ({
    plan: current,
    proxyUrls: new Map(current.proxies.map((proxy) => [proxy.sourceFingerprint, PROXY_URL])),
    docId: 'edt_A',
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
    replan: (next: PreviewPlan) =>
      view.rerender(
        <TooltipProvider>
          <Editor {...props(next)} />
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
