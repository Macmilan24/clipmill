import { describe, expect, it, vi } from 'vitest';
import { drawComposition } from '../src/editor/CompositionCanvas.js';

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
});
