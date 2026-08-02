/**
 * The clip, as a viewer would see it.
 *
 * This is a **read-only** preview and it is deliberately not a player: it seeks
 * the proxy to the clip's window, crops it to 9:16 along the path the reframe
 * solver proposed, and draws the burned-in cues over the result. W24 builds the
 * editor's player; what this has to be is honest about what the render will
 * produce, and nothing here is a mock — the video is the project's own proxy
 * served over the media protocol, the crop is a solve nobody stored, and the
 * cues are the kinetic grouping the encoder will burn in.
 *
 * The crop is done with a CSS transform over the real frame rather than by
 * drawing a rectangle beside it. A guide overlay would show where the camera
 * is pointing; this shows what the camera sees, which is the question an editor
 * is actually asking.
 */
import { useEffect, useRef, useState } from 'react';

import type { CropPath } from '../daemon/client.js';
import { TICKS_PER_SECOND } from '../results/model.js';

export interface OverlayCue {
  readonly from: number;
  readonly to: number;
  readonly text: string;
}

export interface PreviewProps {
  /** The proxy's URL, or null when this project published no proxy. */
  readonly src: string | null;
  readonly startTicks: number;
  readonly endTicks: number;
  /** Null while the solve is in flight; a fitted path is not null. */
  readonly crop: CropPath | null;
  readonly cues: readonly OverlayCue[];
}

/** Linear interpolation along the sparse keyframes, the way a player would. */
function cropAt(path: CropPath | null, atTicks: number): { x: number; y: number; scale: number } {
  const frames = path?.keyframes ?? [];
  const first = frames.at(0);
  const last = frames.at(-1);
  if (!first || !last) {
    return { x: 0.5, y: 0.5, scale: 1 };
  }
  if (atTicks <= first.tTicks) {
    return { x: first.centerX, y: first.centerY, scale: first.scale };
  }
  if (atTicks >= last.tTicks) {
    return { x: last.centerX, y: last.centerY, scale: last.scale };
  }
  const index = frames.findIndex((frame) => frame.tTicks > atTicks);
  const before = frames.at(index - 1);
  const after = frames.at(index);
  if (!before || !after) {
    return { x: last.centerX, y: last.centerY, scale: last.scale };
  }
  const span = after.tTicks - before.tTicks;
  const ratio = span <= 0 ? 0 : (atTicks - before.tTicks) / span;
  return {
    x: before.centerX + (after.centerX - before.centerX) * ratio,
    y: before.centerY + (after.centerY - before.centerY) * ratio,
    scale: before.scale + (after.scale - before.scale) * ratio,
  };
}

export function Preview({ src, startTicks, endTicks, crop, cues }: PreviewProps) {
  const video = useRef<HTMLVideoElement>(null);
  const [atSeconds, setAtSeconds] = useState(0);
  const startSeconds = startTicks / TICKS_PER_SECOND;
  const endSeconds = endTicks / TICKS_PER_SECOND;

  // Seek to the clip's start whenever the window moves, so a boundary nudge is
  // visible immediately rather than after somebody scrubs.
  useEffect(() => {
    const element = video.current;
    if (element) {
      element.currentTime = startSeconds;
      setAtSeconds(startSeconds);
    }
  }, [startSeconds]);

  if (!src) {
    return (
      <div className="grid aspect-[9/16] w-full max-w-[320px] place-items-center rounded-xl border border-dashed border-[var(--cm-line-1)] bg-[var(--cm-surface-2)] p-6 text-center text-sm text-[var(--cm-ink-2)]">
        This project published no proxy, so there is nothing to preview.
      </div>
    );
  }

  const at = cropAt(crop, startTicks + (atSeconds - startSeconds) * TICKS_PER_SECOND);
  const fitted = crop?.fit ?? true;
  // A fitted clip shows the whole frame letterboxed; a followed one is scaled so
  // the crop fills the 9:16 box and translated so its centre is in the middle.
  const zoom = fitted ? 1 : 1 / Math.max(at.scale, 0.01);
  const caption = cues.find(
    (cue) => atSeconds - startSeconds >= cue.from && atSeconds - startSeconds < cue.to,
  );

  return (
    <div className="flex flex-col items-center gap-3">
      <div className="relative aspect-[9/16] w-full max-w-[320px] overflow-hidden rounded-xl bg-black">
        <div
          className="absolute inset-0 flex items-center justify-center"
          style={
            fitted
              ? undefined
              : {
                  transform: `scale(${zoom}) translate(${(0.5 - at.x) * 100}%, ${(0.5 - at.y) * 100}%)`,
                  transformOrigin: 'center',
                }
          }
        >
          {/* eslint-disable-next-line jsx-a11y/media-has-caption -- the cues are
              drawn below from the document rather than as a text track. */}
          <video
            ref={video}
            src={src}
            className={fitted ? 'max-h-full max-w-full' : 'h-full w-auto max-w-none'}
            muted
            playsInline
            controls
            onTimeUpdate={(event) => {
              const element = event.currentTarget;
              if (element.currentTime >= endSeconds) {
                element.pause();
                element.currentTime = startSeconds;
              }
              setAtSeconds(element.currentTime);
            }}
          />
        </div>
        {caption && (
          <p className="pointer-events-none absolute inset-x-3 bottom-14 whitespace-pre-line text-center text-[15px] leading-tight font-semibold text-white drop-shadow-[0_2px_4px_rgba(0,0,0,0.9)]">
            {caption.text}
          </p>
        )}
      </div>
      <p className="text-xs text-[var(--cm-ink-2)]">
        {fitted
          ? `Fitted${crop?.fitReason ? ` — ${crop.fitReason}` : ''}`
          : `Following one face · ${Math.round((crop?.containment ?? 0) * 100)}% contained`}
      </p>
    </div>
  );
}
