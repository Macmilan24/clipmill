/**
 * The clip as a viewer would see it, with transport an editor can drive.
 *
 * The stage is what the render will produce, not a thumbnail of it: the video is
 * the project's own proxy over the media protocol, the crop is the path the
 * reframe solver proposed applied as a transform on the real frame, and the
 * captions are the burned-in grouping from the directed document. Nothing here
 * is drawn beside the picture to describe it — a guide rectangle would show
 * where the camera is pointing, and what an editor is asking is what the camera
 * sees.
 *
 * Playback is confined to the clip's window. Running past the out point would be
 * showing footage that is not in the clip, so the transport treats the window as
 * the whole timeline and loops inside it.
 *
 * The native controls are gone because they describe the proxy — its full
 * duration, its own scrub bar — and the proxy is an hour long while the clip is
 * forty seconds of it. The transport here is about the clip.
 */
import { ChevronFirst, ChevronLast, Pause, Play, SkipBack, SkipForward } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';

import { Button } from '../../components/ui/button.js';
import { Tooltip, TooltipContent, TooltipTrigger } from '../../components/ui/tooltip.js';
import type { CropPath } from '../../daemon/client.js';
import { TICKS_PER_SECOND } from '../../results/model.js';
import type { OverlayCue } from '../Preview.js';

/** One frame at the render rate, 30000/1001. */
export const FRAME_TICKS = 3_003;

export interface PlayerProps {
  readonly src: string | null;
  readonly startTicks: number;
  readonly endTicks: number;
  readonly crop: CropPath | null;
  readonly cues: readonly OverlayCue[];
  /** Where the playhead is, in absolute source ticks. */
  readonly positionTicks: number;
  readonly onPosition: (ticks: number) => void;
  /** Bumped by the timeline when a scrub should move the picture. */
  readonly seekNonce: number;
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

/** `hh:mm:ss;ff` at the render rate, which is how an editor reads a position. */
const pad = (value: number) => String(value).padStart(2, '0');

export function timecode(ticks: number): string {
  const safe = Math.max(0, ticks);
  const totalSeconds = Math.floor(safe / TICKS_PER_SECOND);
  const frames = Math.floor((safe % TICKS_PER_SECOND) / FRAME_TICKS);
  return `${pad(Math.floor(totalSeconds / 3600))}:${pad(Math.floor(totalSeconds / 60) % 60)}:${pad(
    totalSeconds % 60,
  )};${pad(frames)}`;
}

export function Player({
  src,
  startTicks,
  endTicks,
  crop,
  cues,
  positionTicks,
  onPosition,
  seekNonce,
}: PlayerProps) {
  const video = useRef<HTMLVideoElement>(null);
  const [playing, setPlaying] = useState(false);

  // An external seek — a scrub, a handle drag, a different clip — moves the
  // picture. Keyed on the nonce rather than on the position so the player's own
  // time updates do not fight the element that produced them.
  //
  // A seek before the element has metadata is silently dropped: `currentTime`
  // is only writable once the browser knows the duration, and until then the
  // assignment is a no-op. That is why the clip opened at the top of the whole
  // recording rather than at its own first frame — the seek always ran before
  // the proxy had loaded. `readyState` decides whether to seek now or to leave
  // it to `loadedmetadata`, which is the one event that guarantees it will take.
  useEffect(() => {
    const element = video.current;
    if (element && Number.isFinite(positionTicks) && element.readyState >= 1) {
      element.currentTime = positionTicks / TICKS_PER_SECOND;
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps -- the nonce is the signal
  }, [seekNonce]);

  if (!src) {
    return (
      <div className="grid aspect-[9/16] w-full max-w-[300px] place-items-center rounded-[var(--cm-radius-panel)] border border-dashed border-[var(--cm-recessed-border)] p-6 text-center">
        <p className="text-[12px] text-[var(--cm-text-muted)]">
          This project published no proxy, so there is nothing to preview.
        </p>
      </div>
    );
  }

  const at = cropAt(crop, positionTicks);
  const fitted = crop?.fit ?? true;
  const zoom = fitted ? 1 : 1 / Math.max(at.scale, 0.01);
  const intoClip = (positionTicks - startTicks) / TICKS_PER_SECOND;
  const caption = cues.find((cue) => intoClip >= cue.from && intoClip < cue.to);

  const seek = (ticks: number) => {
    const clamped = Math.min(endTicks, Math.max(startTicks, ticks));
    const element = video.current;
    if (element) {
      element.currentTime = clamped / TICKS_PER_SECOND;
    }
    onPosition(clamped);
  };

  const toggle = () => {
    const element = video.current;
    if (!element) {
      return;
    }
    if (element.paused) {
      if (positionTicks >= endTicks - FRAME_TICKS) {
        element.currentTime = startTicks / TICKS_PER_SECOND;
      }
      void element.play();
    } else {
      element.pause();
    }
  };

  const step = (label: string, hint: string, icon: React.ReactNode, to: number) => (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button variant="ghost" size="icon" aria-label={label} onClick={() => seek(to)}>
          {icon}
        </Button>
      </TooltipTrigger>
      <TooltipContent>{hint}</TooltipContent>
    </Tooltip>
  );

  return (
    <div className="flex min-h-0 flex-1 flex-col items-center gap-3">
      <div className="relative flex min-h-0 flex-1 items-center justify-center">
        <div className="relative aspect-[9/16] h-full max-h-full overflow-hidden rounded-[var(--cm-radius-panel)] bg-black ring-1 ring-white/10">
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
              onLoadedMetadata={(event) => {
                // The first seek that can actually land. Without it the element
                // opens at zero, which is a different clip.
                event.currentTarget.currentTime = positionTicks / TICKS_PER_SECOND;
              }}
              onPlay={() => setPlaying(true)}
              onPause={() => setPlaying(false)}
              onTimeUpdate={(event) => {
                const element = event.currentTarget;
                const ticks = element.currentTime * TICKS_PER_SECOND;
                if (ticks >= endTicks) {
                  element.pause();
                  element.currentTime = startTicks / TICKS_PER_SECOND;
                  onPosition(startTicks);
                  return;
                }
                onPosition(ticks);
              }}
            />
          </div>

          <div className="pointer-events-none absolute inset-x-3 top-3 flex justify-between">
            <span className="mono rounded bg-black/55 px-2 py-1 text-[10px] text-white backdrop-blur-sm">
              {timecode(positionTicks)}
            </span>
            <span className="rounded bg-black/55 px-2 py-1 text-[10px] tracking-wide text-white uppercase backdrop-blur-sm">
              9:16
            </span>
          </div>

          {caption && (
            <p className="pointer-events-none absolute inset-x-4 bottom-16 text-center text-[17px] leading-tight font-bold text-white uppercase drop-shadow-[0_2px_6px_rgba(0,0,0,0.95)]">
              {caption.text}
            </p>
          )}
        </div>
      </div>

      <div className="flex w-full shrink-0 items-center justify-center gap-1">
        {step('Jump to the in point', 'In point', <ChevronFirst className="size-4" />, startTicks)}
        {step(
          'Back one second',
          '−1 second',
          <SkipBack className="size-4" />,
          positionTicks - TICKS_PER_SECOND,
        )}
        {step(
          'Back one frame',
          '−1 frame at 30000/1001',
          <span className="mono text-[10px]">−1f</span>,
          positionTicks - FRAME_TICKS,
        )}
        <Button
          variant="outline"
          size="icon"
          aria-label={playing ? 'Pause' : 'Play'}
          onClick={toggle}
          className="mx-1 size-10 rounded-full"
        >
          {playing ? <Pause className="size-4" /> : <Play className="size-4" />}
        </Button>
        {step(
          'Forward one frame',
          '+1 frame at 30000/1001',
          <span className="mono text-[10px]">+1f</span>,
          positionTicks + FRAME_TICKS,
        )}
        {step(
          'Forward one second',
          '+1 second',
          <SkipForward className="size-4" />,
          positionTicks + TICKS_PER_SECOND,
        )}
        {step('Jump to the out point', 'Out point', <ChevronLast className="size-4" />, endTicks)}
      </div>

      <p className="shrink-0 text-[11px] text-[var(--cm-text-muted)]">
        {fitted
          ? `Fitted${crop?.fitReason ? ` — ${crop.fitReason}` : ''}`
          : `Following one face · ${Math.round((crop?.containment ?? 0) * 100)}% contained`}
      </p>
    </div>
  );
}
