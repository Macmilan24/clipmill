import { useEffect, useRef } from 'react';
import type { RefObject } from 'react';
import type { PreviewPlan, PreviewSource } from '../daemon/client.js';
import { cropAt, proxySecondsAt, segmentAt, sourceOf } from './player.js';

type Crop = ReturnType<typeof cropAt>;
type Picture = HTMLVideoElement | HTMLCanvasElement;
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
  video: Picture,
  drawing: Drawing,
) {
  const { width, height, crop, secondary, source } = drawing;
  const pixelWidth = 'videoWidth' in video ? video.videoWidth : video.width;
  const pixelHeight = 'videoHeight' in video ? video.videoHeight : video.height;
  if (!pixelWidth || !pixelHeight) return false;
  context.clearRect(0, 0, width, height);
  const portrait = (rect: NonNullable<Crop>, top: number, viewportHeight: number) => {
    const sx = pixelWidth / (source?.displayWidth ?? pixelWidth);
    const sy = pixelHeight / (source?.displayHeight ?? pixelHeight);
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
  const fill = Math.max(width / pixelWidth, height / pixelHeight);
  context.save();
  context.filter = `blur(${(40 * height) / 1920}px)`;
  try {
    context.drawImage(
      video,
      (width - pixelWidth * fill) / 2,
      (height - pixelHeight * fill) / 2,
      pixelWidth * fill,
      pixelHeight * fill,
    );
  } finally {
    context.restore();
  }
  const fit = Math.min(width / pixelWidth, height / pixelHeight);
  context.drawImage(
    video,
    (width - pixelWidth * fit) / 2,
    (height - pixelHeight * fit) / 2,
    pixelWidth * fit,
    pixelHeight * fit,
  );
  return true;
}

/** One clock for decoded pixels, crops and the editor playhead. */
export function observeVideoFrames(
  media: HTMLVideoElement,
  draw: (seconds: number, picture: HTMLCanvasElement) => void,
) {
  const decodedFrames = typeof media.requestVideoFrameCallback === 'function';
  let request: number | null = null;
  let generation = 0;
  let disposed = false;
  let cached: { seconds: number; picture: HTMLCanvasElement } | null = null;
  let scratch = document.createElement('canvas');
  const cancel = () => {
    generation += 1;
    if (request !== null) {
      if (decodedFrames) media.cancelVideoFrameCallback(request);
      else cancelAnimationFrame(request);
      request = null;
    }
  };
  const redraw = () => {
    if (disposed || media.seeking || media.readyState < 2) return;
    if (cached) draw(cached.seconds, cached.picture);
  };
  const present = (seconds: number) => {
    if (disposed || (!decodedFrames && (media.seeking || media.readyState < 2))) return;
    if (!media.videoWidth || !media.videoHeight) return;
    if (scratch.width !== media.videoWidth) scratch.width = media.videoWidth;
    if (scratch.height !== media.videoHeight) scratch.height = media.videoHeight;
    const context = scratch.getContext('2d');
    if (!context) return;
    try {
      context.globalCompositeOperation = 'copy';
      context.drawImage(media, 0, 0);
    } catch (error) {
      if (!(error instanceof DOMException && error.name === 'InvalidStateError')) throw error;
      return;
    }
    // Keep decoded pixels with their own PTS. A reference finishing its seek,
    // or a paused crop edit, must never sample a newer live video frame under
    // an older timestamp. Double buffering also preserves the last good pair
    // if a decoder becomes unavailable during the next copy.
    const previous = cached?.picture;
    cached = { seconds, picture: scratch };
    scratch = previous ?? document.createElement('canvas');
    redraw();
  };
  // A paused video can still present its first decoded frame or a seek result.
  // One pending rVFC catches that without polling or repainting idle frames.
  const schedule = () => {
    if (disposed || request !== null || (!decodedFrames && (media.seeking || media.paused))) return;
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
        if (cached?.seconds !== media.currentTime) present(media.currentTime);
        schedule();
      });
    }
  };
  const ready = () => {
    // currentTime is the playback/seek clock, not a decoded frame's PTS.
    // With rVFC, wait for the matching pixels even after loadeddata/seeked.
    if (decodedFrames) redraw();
    else present(media.currentTime);
    schedule();
  };
  const seeking = () => {
    cancel();
    cached = null;
    // Register before the seek result is presented, including paused seeks.
    // If its callback precedes seeked, snapshot now and publish at seeked.
    schedule();
  };
  const pause = () => {
    cancel();
    redraw();
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
    redraw,
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
  const referenceObserver = useRef<ReturnType<typeof observeVideoFrames> | null>(null);
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
  const referenceSeconds =
    referenceFrame === undefined ? null : proxySecondsAt(plan, referenceFrame);
  const referenceTarget = useRef({ frame: referenceFrame, seconds: referenceSeconds });
  referenceTarget.current = { frame: referenceFrame, seconds: referenceSeconds };
  useEffect(() => {
    reference.current = null;
    referenceFailed.current = false;
    const media = referenceVideo.current;
    if (referenceSeconds !== null && !referenceUrl) {
      referenceFailed.current = true;
      latest.current.onError?.();
      return;
    }
    if (!media || !referenceUrl) return;
    const seconds = referenceSeconds;
    if (seconds === null) return;
    const bitmap = document.createElement('canvas');
    const position = () => {
      if (Math.abs(media.currentTime - seconds) > 1 / 90_000) media.currentTime = seconds;
    };
    const capture = (_seconds: number, picture: HTMLCanvasElement) => {
      if (Math.abs(media.currentTime - seconds) > 1 / 90_000) return;
      const target = referenceTarget.current;
      if (target.frame === undefined || target.seconds !== seconds) return;
      const currentPlan = latest.current.plan;
      const segment = segmentAt(currentPlan, target.frame);
      if (!segment || proxySecondsAt(currentPlan, target.frame) !== seconds) return;
      if (bitmap.width !== currentPlan.width) bitmap.width = currentPlan.width;
      if (bitmap.height !== currentPlan.height) bitmap.height = currentPlan.height;
      const context = bitmap.getContext('2d');
      if (!context) return;
      try {
        if (
          !drawComposition(context, picture, {
            crop: cropAt(currentPlan, target.frame),
            secondary: cropAt(currentPlan, target.frame, true),
            source: sourceOf(currentPlan, segment),
            width: currentPlan.width,
            height: currentPlan.height,
          })
        )
          return;
        reference.current = { plan: currentPlan, frame: target.frame, canvas: bitmap };
        observer.current?.redraw();
      } catch (error) {
        if (!(error instanceof DOMException && error.name === 'InvalidStateError')) throw error;
      }
    };
    media.addEventListener('loadedmetadata', position);
    position();
    const prepared = observeVideoFrames(media, capture);
    referenceObserver.current = prepared;
    return () => {
      prepared.dispose();
      referenceObserver.current = null;
      media.removeEventListener('loadedmetadata', position);
      if (!media.paused) media.pause();
    };
  }, [referenceUrl, referenceSeconds]);
  useEffect(() => {
    // A crop edit can keep the paused reference at the exact same source
    // position. Recompose its cached decoded picture; no new decode is due.
    reference.current = null;
    referenceObserver.current?.redraw();
  }, [plan, referenceFrame]);
  useEffect(() => {
    const media = video.current;
    const output = canvas.current;
    if (!media || !output) return;
    // Compose offscreen and publish only a complete frame. A decoder becoming
    // unavailable midway through a seek must not clear the visible picture.
    const buffer = document.createElement('canvas');
    const draw = (seconds: number, picture: HTMLCanvasElement) => {
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
        const drawn = drawComposition(context, picture, {
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
