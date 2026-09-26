import { describe, expect, it, vi } from 'vitest';
import { drawComposition } from '../src/editor/CompositionCanvas.js';

const canvasContext = () =>
  ({
    clearRect: vi.fn(),
    drawImage: vi.fn(),
    fillRect: vi.fn(),
    save: vi.fn(),
    restore: vi.fn(),
  }) as unknown as CanvasRenderingContext2D & {
    drawImage: ReturnType<typeof vi.fn>;
    fillRect: ReturnType<typeof vi.fn>;
  };

describe('one decoded frame drives the composition', () => {
  it('draws both portraits from the same video frame in their own viewport', () => {
    const context = {
      clearRect: vi.fn(),
      drawImage: vi.fn(),
    } as unknown as CanvasRenderingContext2D;
    const video = { videoWidth: 960, videoHeight: 540 } as HTMLVideoElement;
    drawComposition(context, video, {
      crop: { x: 0, y: 140, width: 900, height: 800 },
      secondary: { x: 1000, y: 140, width: 900, height: 800 },
      source: { displayWidth: 1920, displayHeight: 1080 },
      width: 1080,
      height: 1920,
    });
    expect(context.drawImage).toHaveBeenNthCalledWith(1, video, 0, 70, 450, 400, 0, 0, 1080, 960);
    expect(context.drawImage).toHaveBeenNthCalledWith(
      2,
      video,
      500,
      70,
      450,
      400,
      0,
      960,
      1080,
      960,
    );
  });

  const video = { videoWidth: 1920, videoHeight: 1080 } as HTMLVideoElement;
  const source = { displayWidth: 1920, displayHeight: 1080 };

  it('stacks two viewports at the split of the section, as the render does', () => {
    const drawn = canvasContext();
    drawComposition(drawn, video, {
      crop: { x: 0, y: 2, width: 1918, height: 1076 },
      secondary: { x: 1000, y: 0, width: 888, height: 1080 },
      source,
      width: 1080,
      height: 1920,
      segment: { layout: 'two_up', upperHeight: 606 },
    });
    expect(drawn.drawImage.mock.calls[0]!.slice(5)).toEqual([0, 0, 1080, 606]);
    expect(drawn.drawImage.mock.calls[1]!.slice(5)).toEqual([0, 606, 1080, 1314]);
  });

  it('fills around a fitted picture with its colour and zooms it about the centre', () => {
    const drawn = canvasContext();
    drawComposition(drawn, video, {
      crop: null,
      secondary: null,
      source,
      width: 1080,
      height: 1920,
      segment: { layout: 'fit', backgroundColour: '#00ff00', zoomPercent: 150 },
    });
    expect(drawn.fillStyle).toBe('#00ff00');
    expect(drawn.fillRect).toHaveBeenCalledWith(0, 0, 1080, 1920);
    // One draw only, no blurred fill: 1620x911.25 about the frame's centre.
    expect(drawn.drawImage).toHaveBeenCalledTimes(1);
    const [, left, top, width, height] = drawn.drawImage.mock.calls[0]!;
    expect(width).toBeCloseTo(1620);
    expect(height).toBeCloseTo(911.25);
    expect(left).toBeCloseTo(-270);
    expect(top).toBeCloseTo((1920 - 911.25) / 2);
  });

  it('insets the second crop in its square over the whole frame', () => {
    const drawn = canvasContext();
    drawComposition(drawn, video, {
      crop: null,
      secondary: { x: 1100, y: 100, width: 800, height: 800 },
      source,
      width: 1080,
      height: 1920,
      segment: { layout: 'picture_in_picture', inset: [42, 988, 432] },
    });
    const last = drawn.drawImage.mock.calls.at(-1)!;
    expect(last.slice(1)).toEqual([1100, 100, 800, 800, 42, 988, 432, 432]);
  });
});
