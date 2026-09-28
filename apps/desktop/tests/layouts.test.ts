import { describe, expect, it } from 'vitest';

import {
  FRAME_SHAPES,
  frameOfShape,
  layoutCommands,
  recordingSplit,
  refit,
  shapeCommand,
  shapeOfFrame,
  splitCommands,
  splitsAcross,
  viewportHeights,
  viewports,
} from '../src/editor/layouts.js';
import { plan } from './support/clips.js';

const frame = { displayWidth: 1920, displayHeight: 1080 };
const output = { width: 1080, height: 1920 };
const still = (x: number, y: number, width: number, height: number) => [
  { t_ticks: 0, rect: { x, y, width, height } },
];
type Replace = { op: string; path: { rect: { width: number; height: number; x: number } }[] };
const paths = (commands: readonly unknown[]) =>
  (commands as Replace[]).filter((command) => command.op.startsWith('replace_'));
const aspect = (rect: { width: number; height: number }) => rect.width / rect.height;

describe('refit', () => {
  it('keeps the centre and the height, and takes the new shape', () => {
    const [moved] = refit(still(500, 140, 900, 800), { width: 1080, height: 1920 }, frame);
    expect(moved!.rect.height).toBe(800);
    expect(moved!.rect.width).toBe(450);
    expect(moved!.rect.x + moved!.rect.width / 2).toBe(950);
  });

  it('narrows to the frame when the shape would leave it', () => {
    const [wide] = refit(still(0, 0, 1080, 1080), { width: 1080, height: 606 }, frame);
    expect(wide!.rect.width).toBeLessThanOrEqual(1920);
    expect(Math.abs(aspect(wide!.rect) - 1080 / 606)).toBeLessThan(0.01);
    expect(wide!.rect.x).toBe(0);
  });
});

describe('choosing a layout', () => {
  const twoUp = {
    state: 'two_up' as const,
    crop_path: still(0, 140, 900, 800),
    secondary_crop_path: still(1000, 140, 900, 800),
  };

  it('reshapes the portrait when two speakers become one', () => {
    const commands = layoutCommands('seg', 'speaker_fill', twoUp, output, frame);
    expect(commands[0]).toEqual({ op: 'set_layout', segment_id: 'seg', state: 'speaker_fill' });
    const [primary] = paths(commands);
    expect(Math.abs(aspect(primary!.path[0]!.rect) - 9 / 16)).toBeLessThan(0.01);
  });

  it('puts the whole recording on top for a screen and a face', () => {
    expect(recordingSplit(frame, output)).toBe(316);
    const commands = layoutCommands('seg', 'screen_and_face', twoUp, output, frame);
    expect(commands).toContainEqual({ op: 'set_layout_style', segment_id: 'seg', split: 316 });
    const [upper, lower] = paths(commands);
    const [top, bottom] = viewportHeights(316, 1920);
    expect(top).toBe(606);
    expect(upper!.path[0]!.rect.width).toBeGreaterThan(1900);
    expect(Math.abs(aspect(upper!.path[0]!.rect) - 1080 / top)).toBeLessThan(0.01);
    expect(Math.abs(aspect(lower!.path[0]!.rect) - 1080 / bottom)).toBeLessThan(0.01);
  });

  it('insets the second person square, over the first', () => {
    const commands = layoutCommands('seg', 'picture_in_picture', twoUp, output, frame);
    expect(commands).toContainEqual({
      op: 'set_layout_style',
      segment_id: 'seg',
      inset: { corner: 'top_right', size: 360 },
    });
    const [main, inset] = paths(commands);
    expect(Math.abs(aspect(main!.path[0]!.rect) - 9 / 16)).toBeLessThan(0.01);
    expect(inset!.path[0]!.rect.width).toBe(inset!.path[0]!.rect.height);
  });

  it('insets the middle of a fitted frame when there is nobody to inset', () => {
    const commands = layoutCommands('seg', 'picture_in_picture', { state: 'fit' }, output, frame);
    const [main, inset] = paths(commands);
    expect(main!.path).toEqual([]);
    expect(inset!.path[0]!.rect.width).toBe(inset!.path[0]!.rect.height);
  });
});

describe('moving the split', () => {
  it('reshapes both crops for the viewports the split makes', () => {
    const commands = splitCommands(
      'seg',
      { split: 600 },
      {
        state: 'two_up',
        crop_path: still(0, 140, 900, 800),
        secondary_crop_path: still(1000, 140, 900, 800),
      },
      output,
      frame,
    );
    const [upper, lower] = paths(commands);
    const [top, bottom] = viewportHeights(600, 1920);
    expect(Math.abs(aspect(upper!.path[0]!.rect) - 1080 / top)).toBeLessThan(0.01);
    expect(Math.abs(aspect(lower!.path[0]!.rect) - 1080 / bottom)).toBeLessThan(0.01);
    expect(splitCommands('seg', { split: 500 }, { state: 'two_up' }, output, frame)[0]).toEqual({
      op: 'set_layout_style',
      segment_id: 'seg',
    });
  });
});

describe('switching the two people', () => {
  it('trades places at an even split and reshapes at any other', async () => {
    const { switchCommands } = await import('../src/editor/layouts.js');
    const saved = {
      state: 'two_up' as const,
      crop_path: still(0, 140, 900, 800),
      secondary_crop_path: still(1000, 140, 900, 800),
    };
    expect(switchCommands('seg', saved, output, frame)).toEqual([
      { op: 'swap_portraits', segment_id: 'seg' },
    ]);
    const [upper, lower] = paths(switchCommands('seg', { ...saved, split: 316 }, output, frame));
    const [top, bottom] = viewportHeights(316, 1920);
    // The lower person moves up, reshaped for the wider upper viewport: the
    // widened crop still holds them, shifted in to stay inside the frame.
    const moved = upper!.path[0]!.rect;
    expect(moved.x + moved.width).toBe(1920);
    expect(moved.x).toBeLessThan(1450);
    expect(Math.abs(aspect(upper!.path[0]!.rect) - 1080 / top)).toBeLessThan(0.01);
    expect(Math.abs(aspect(lower!.path[0]!.rect) - 1080 / bottom)).toBeLessThan(0.01);
  });
});

describe('the clip in another shape', () => {
  it('splits side by side across a landscape frame, and stacks in any other', () => {
    const landscape = { width: 1920, height: 1080 };
    expect(splitsAcross(landscape)).toBe(true);
    expect(splitsAcross({ width: 1080, height: 1080 })).toBe(false);
    expect(viewports(400, landscape)).toEqual([
      { width: 768, height: 1080 },
      { width: 1152, height: 1080 },
    ]);
    expect(viewports(400, output)).toEqual([
      { width: 1080, height: 768 },
      { width: 1080, height: 1152 },
    ]);
    // A 16:9 recording on the left of a 16:9 frame would take all of it.
    expect(recordingSplit(frame, landscape)).toBe(750);
  });

  it('names each shape and sizes its frame as the render does', () => {
    expect(FRAME_SHAPES.map((item) => item.ratio)).toEqual(['9:16', '4:5', '1:1', '16:9']);
    expect(frameOfShape('portrait')).toEqual({ width: 1080, height: 1350 });
    expect(shapeOfFrame({ width: 1920, height: 1080 })).toBe('landscape');
    expect(shapeOfFrame({ width: 540, height: 960 })).toBe('vertical');
  });

  it('changes shape in one step, every crop reshaped for its new viewport', () => {
    const base = plan();
    const part = base.segments[0]!;
    const second = { ...part, segmentId: 'seg_2' };
    const third = { ...part, segmentId: 'seg_3' };
    const document = {
      video: {
        segments: [
          {
            segment_id: part.segmentId,
            source_fingerprint: part.sourceFingerprint,
            in_ticks: 0,
            out_ticks: 1,
            layout: { state: 'speaker_fill' as const, crop_path: still(700, 0, 608, 1080) },
          },
          {
            segment_id: 'seg_2',
            source_fingerprint: part.sourceFingerprint,
            in_ticks: 1,
            out_ticks: 2,
            layout: {
              state: 'two_up' as const,
              crop_path: still(0, 140, 900, 800),
              secondary_crop_path: still(1000, 140, 900, 800),
            },
          },
          {
            segment_id: 'seg_3',
            source_fingerprint: part.sourceFingerprint,
            in_ticks: 2,
            out_ticks: 3,
            layout: { state: 'fit' as const },
          },
        ],
      },
    };
    const command = shapeCommand('landscape', document, {
      ...base,
      segments: [part, second, third],
    }) as { op: string; commands: Replace[] };
    expect(command.op).toBe('batch');
    expect(command.commands[0]).toEqual({ op: 'set_frame_shape', shape: 'landscape' });
    const [follow, left, right, ...rest] = paths(command.commands);
    expect(rest).toEqual([]);
    // The followed crop is as close as it was, now 16:9 and within the frame.
    expect(Math.abs(aspect(follow!.path[0]!.rect) - 16 / 9)).toBeLessThan(0.01);
    expect(follow!.path[0]!.rect.height).toBe(1080);
    // Two people sit side by side, each in half of the landscape frame.
    expect(Math.abs(aspect(left!.path[0]!.rect) - 960 / 1080)).toBeLessThan(0.01);
    expect(Math.abs(aspect(right!.path[0]!.rect) - 960 / 1080)).toBeLessThan(0.01);
    expect(left!.path[0]!.rect.height).toBe(800);
  });
});
