import { useEffect, useRef } from 'react';
import type { RefObject } from 'react';
import type { PreviewSource } from '../daemon/client.js';
import type { cropAt } from './player.js';

type Crop = ReturnType<typeof cropAt>;
interface Drawing {
  readonly crop: Crop;
  readonly secondary: Crop;
  readonly source: Pick<PreviewSource, 'displayWidth' | 'displayHeight'> | null;
  readonly width: number;
  readonly height: number;
}

/** Both portraits are sampled from this one decoded frame; they cannot drift. */
export function drawComposition(
  context: CanvasRenderingContext2D,
  video: HTMLVideoElement,
  drawing: Drawing,
) {
  const { width, height, crop, secondary, source } = drawing;
  if (!video.videoWidth || !video.videoHeight) return;
  context.clearRect(0, 0, width, height);
  const portrait = (rect: NonNullable<Crop>, top: number, viewportHeight: number) => {
    const sx = video.videoWidth / (source?.displayWidth ?? video.videoWidth);
    const sy = video.videoHeight / (source?.displayHeight ?? video.videoHeight);
    context.drawImage(
      video,
      rect.x * sx,
      rect.y * sy,
      rect.width * sx,
      rect.height * sy,
      0,
      top,
      width,
      viewportHeight,
    );
  };
  if (crop) {
    portrait(crop, 0, secondary ? height / 2 : height);
    if (secondary) portrait(secondary, height / 2, height / 2);
    return;
  }
  const fill = Math.max(width / video.videoWidth, height / video.videoHeight);
  context.save();
  context.filter = `blur(${(40 * height) / 1920}px)`;
  context.drawImage(
    video,
    (width - video.videoWidth * fill) / 2,
    (height - video.videoHeight * fill) / 2,
    video.videoWidth * fill,
    video.videoHeight * fill,
  );
  context.restore();
  const fit = Math.min(width / video.videoWidth, height / video.videoHeight);
  context.drawImage(
    video,
    (width - video.videoWidth * fit) / 2,
    (height - video.videoHeight * fit) / 2,
    video.videoWidth * fit,
    video.videoHeight * fit,
  );
}

export function CompositionCanvas({
  video,
  ...drawing
}: Drawing & { readonly video: RefObject<HTMLVideoElement | null> }) {
  const canvas = useRef<HTMLCanvasElement>(null);
  const latest = useRef(drawing);
  latest.current = drawing;
  useEffect(() => {
    const media = video.current;
    if (!media) return;
    let request = 0;
    const draw = () => {
      if (media.readyState < 2) return;
      const context = canvas.current?.getContext('2d');
      if (context) drawComposition(context, media, latest.current);
    };
    const tick = () => {
      draw();
      request = requestAnimationFrame(tick);
    };
    for (const event of ['loadeddata', 'seeked', 'timeupdate']) media.addEventListener(event, draw);
    request = requestAnimationFrame(tick);
    return () => {
      cancelAnimationFrame(request);
      for (const event of ['loadeddata', 'seeked', 'timeupdate'])
        media.removeEventListener(event, draw);
    };
  }, [video]);
  return (
    <canvas
      ref={canvas}
      width={drawing.width}
      height={drawing.height}
      className="absolute inset-0 h-full w-full"
      data-testid="composition"
      role="img"
      aria-label={
        drawing.secondary
          ? 'Draft composition with two synchronized portraits'
          : 'Draft clip composition'
      }
    />
  );
}
