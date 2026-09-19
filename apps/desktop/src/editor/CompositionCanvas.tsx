import { useEffect, useRef } from 'react';
import type { RefObject } from 'react';
import type { PreviewPlan, PreviewSource } from '../daemon/client.js';
import { cropAt, proxySecondsAt, segmentAt, sourceOf } from './player.js';

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
  if (!video.videoWidth || !video.videoHeight) return false;
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
    return true;
  }
  const fill = Math.max(width / video.videoWidth, height / video.videoHeight);
  context.save();
  context.filter = `blur(${(40 * height) / 1920}px)`;
  try {
    context.drawImage(
      video,
      (width - video.videoWidth * fill) / 2,
      (height - video.videoHeight * fill) / 2,
      video.videoWidth * fill,
      video.videoHeight * fill,
    );
  } finally {
    context.restore();
  }
  const fit = Math.min(width / video.videoWidth, height / video.videoHeight);
  context.drawImage(
    video,
    (width - video.videoWidth * fit) / 2,
    (height - video.videoHeight * fit) / 2,
    video.videoWidth * fit,
    video.videoHeight * fit,
  );
  return true;
}

/** One clock for decoded pixels, crops and the editor playhead. */
export function observeVideoFrames(media: HTMLVideoElement, draw: (seconds: number) => void) {
  const decodedFrames = typeof media.requestVideoFrameCallback === 'function';
  let request: number | null = null;
  let generation = 0;
  let disposed = false;
  let lastTime: number | null = null;
  const cancel = () => {
    generation += 1;
    if (request !== null) {
      if (decodedFrames) media.cancelVideoFrameCallback(request);
      else cancelAnimationFrame(request);
      request = null;
    }
  };
  const present = (seconds: number) => {
    if (disposed || media.seeking || media.readyState < 2) return;
    lastTime = seconds;
    draw(seconds);
  };
  // A paused video can still present its first decoded frame or a seek result.
  // One pending rVFC catches that without polling or repainting idle frames.
  const schedule = () => {
    if (disposed || request !== null || media.seeking || (!decodedFrames && media.paused)) return;
    const scheduledGeneration = generation;
    if (decodedFrames) {
      request = media.requestVideoFrameCallback((_now, metadata) => {
        if (disposed || generation !== scheduledGeneration) return;
        request = null;
        present(metadata.mediaTime);
        schedule();
      });
    } else {
      request = requestAnimationFrame(() => {
        if (disposed || generation !== scheduledGeneration) return;
        request = null;
        if (lastTime !== media.currentTime) present(media.currentTime);
        schedule();
      });
    }
  };
  const ready = () => {
    present(media.currentTime);
    schedule();
  };
  const seeking = () => {
    cancel();
    lastTime = null;
  };
  const pause = () => {
    cancel();
    present(lastTime ?? media.currentTime);
    schedule();
  };
  // timeupdate must not race the decoded-frame clock. It is only a fallback
  // for webviews without rVFC, and also handles paused native seeks there.
  const timeupdate = () => {
    if (!decodedFrames) ready();
  };
  const events = {
    loadeddata: ready,
    seeked: ready,
    playing: schedule,
    seeking,
    pause,
    timeupdate,
  };
  for (const [event, listener] of Object.entries(events)) media.addEventListener(event, listener);
  ready();
  return {
    redraw: () => present(lastTime ?? media.currentTime),
    dispose: () => {
      disposed = true;
      cancel();
      for (const [event, listener] of Object.entries(events))
        media.removeEventListener(event, listener);
    },
  };
}

export function CompositionCanvas({
  video,
  mediaKey,
  plan,
  frame,
  onFrame,
  proxyUrls,
  onError,
  onBuffering,
}: {
  readonly video: RefObject<HTMLVideoElement | null>;
  readonly mediaKey: string;
  readonly plan: PreviewPlan;
  readonly frame: number;
  readonly onFrame: (seconds: number) => number | null;
  readonly proxyUrls?: ReadonlyMap<string, string>;
  readonly onError?: () => void;
  readonly onBuffering?: (waiting: boolean) => void;
}) {
  const canvas = useRef<HTMLCanvasElement>(null);
  const latest = useRef({ plan, onFrame, onError, onBuffering });
  latest.current = { plan, onFrame, onError, onBuffering };
  const observer = useRef<ReturnType<typeof observeVideoFrames> | null>(null);
  const referenceVideo = useRef<HTMLVideoElement>(null);
  const referenceFailed = useRef(false);
  const reference = useRef<{ plan: PreviewPlan; frame: number; canvas: HTMLCanvasElement } | null>(
    null,
  );
  // Prepare the upcoming boundary in advance. A separate muted, paused decoder
  // also makes a direct scrub into a blend deterministic: playback history is
  // never substituted for the outgoing frame named by the saved plan.
  const upcoming = plan.transitions?.find((transition) => frame < transition.endFrame);
  const outgoing = upcoming ? segmentAt(plan, upcoming.outgoingFrame) : null;
  const referenceUrl = outgoing ? proxyUrls?.get(outgoing.sourceFingerprint) : undefined;
  const referenceFrame = upcoming?.outgoingFrame;
  useEffect(() => {
    reference.current = null;
    referenceFailed.current = false;
    const media = referenceVideo.current;
    if (referenceFrame !== undefined && !referenceUrl) {
      referenceFailed.current = true;
      latest.current.onError?.();
      return;
    }
    if (!media || !referenceUrl || referenceFrame === undefined) return;
    const seconds = proxySecondsAt(plan, referenceFrame);
    const segment = segmentAt(plan, referenceFrame);
    if (seconds === null || !segment) return;
    const bitmap = document.createElement('canvas');
    bitmap.width = plan.width;
    bitmap.height = plan.height;
    const position = () => {
      if (Math.abs(media.currentTime - seconds) > 1 / 90_000) media.currentTime = seconds;
    };
    const capture = () => {
      if (Math.abs(media.currentTime - seconds) > 1 / 90_000) return;
      const context = bitmap.getContext('2d');
      if (!context) return;
      try {
        if (
          !drawComposition(context, media, {
            crop: cropAt(plan, referenceFrame),
            secondary: cropAt(plan, referenceFrame, true),
            source: sourceOf(plan, segment),
            width: plan.width,
            height: plan.height,
          })
        )
          return;
        reference.current = { plan, frame: referenceFrame, canvas: bitmap };
        observer.current?.redraw();
      } catch (error) {
        if (!(error instanceof DOMException && error.name === 'InvalidStateError')) throw error;
      }
    };
    media.addEventListener('loadedmetadata', position);
    position();
    const prepared = observeVideoFrames(media, capture);
    return () => {
      prepared.dispose();
      media.removeEventListener('loadedmetadata', position);
      if (!media.paused) media.pause();
    };
  }, [plan, referenceUrl, referenceFrame]);
  useEffect(() => {
    const media = video.current;
    const output = canvas.current;
    if (!media || !output) return;
    // Compose offscreen and publish only a complete frame. A decoder becoming
    // unavailable midway through a seek must not clear the visible picture.
    const buffer = document.createElement('canvas');
    const draw = (seconds: number) => {
      if (!media.videoWidth || !media.videoHeight) return;
      const current = latest.current;
      const programFrame = current.onFrame(seconds);
      if (programFrame === null || media.seeking) return;
      const drawingPlan = current.plan;
      const segment = segmentAt(drawingPlan, programFrame);
      const transition = drawingPlan.transitions?.find(
        (item) => programFrame >= item.firstFrame && programFrame < item.endFrame,
      );
      const held = reference.current;
      if (
        transition &&
        (!held || held.plan !== drawingPlan || held.frame !== transition.outgoingFrame)
      ) {
        if (referenceFailed.current) current.onError?.();
        else current.onBuffering?.(true);
        // Preserve the last complete picture until a seek-only reference is
        // decoded. Publishing an unblended frame here would flash at the cut.
        return;
      }
      const context = buffer.getContext('2d');
      const destination = output.getContext('2d');
      if (!context || !destination) return;
      if (buffer.width !== drawingPlan.width) buffer.width = drawingPlan.width;
      if (buffer.height !== drawingPlan.height) buffer.height = drawingPlan.height;
      try {
        const drawn = drawComposition(context, media, {
          crop: cropAt(drawingPlan, programFrame),
          secondary: cropAt(drawingPlan, programFrame, true),
          source: segment ? sourceOf(drawingPlan, segment) : null,
          width: drawingPlan.width,
          height: drawingPlan.height,
        });
        if (drawn) {
          if (transition && held) {
            context.save();
            try {
              context.globalAlpha =
                (transition.endFrame - programFrame) /
                (transition.endFrame - transition.firstFrame);
              context.drawImage(held.canvas, 0, 0);
            } finally {
              context.restore();
            }
          }
          destination.save();
          try {
            // Replace even translucent blur edges; source-over would blend
            // them with the previous frame and leave motion trails.
            destination.globalCompositeOperation = 'copy';
            destination.drawImage(buffer, 0, 0);
          } finally {
            destination.restore();
          }
          current.onBuffering?.(false);
        }
      } catch (error) {
        // drawImage may lose its decoded frame during a media transition.
        // Other errors are programming errors and must stay visible.
        if (!(error instanceof DOMException && error.name === 'InvalidStateError')) throw error;
      }
    };
    const active = observeVideoFrames(media, draw);
    observer.current = active;
    return () => {
      active.dispose();
      if (!media.paused) media.pause();
      observer.current = null;
    };
  }, [video, mediaKey]);
  useEffect(() => {
    // Paused crop/layout edits have no new video frame to trigger a callback.
    if (video.current?.paused) observer.current?.redraw();
  }, [video, plan, frame]);
  return (
    <>
      {referenceUrl && (
        // No audio or captions belong to the held outgoing picture.
        <video
          ref={referenceVideo}
          src={referenceUrl}
          muted
          playsInline
          preload="auto"
          crossOrigin="anonymous"
          aria-hidden="true"
          tabIndex={-1}
          className="pointer-events-none absolute h-full w-full opacity-0"
          data-testid="transition-reference"
          onError={() => {
            referenceFailed.current = true;
            reference.current = null;
            onError?.();
          }}
        />
      )}
      <canvas
        ref={canvas}
        width={plan.width}
        height={plan.height}
        className="absolute inset-0 h-full w-full"
        data-testid="composition"
        role="img"
        aria-label={
          cropAt(plan, frame, true)
            ? 'Draft composition with two synchronized portraits'
            : 'Draft clip composition'
        }
      />
    </>
  );
}
