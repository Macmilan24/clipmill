/**
 * The episode overview and the zoomable boundary strip. Handles land between
 * words (R63); the search's suggested edges are marked but not required.
 * Playheads subscribe to the clock so the strips do not re-render while playing.
 */
import { useCallback, useEffect, useMemo, useRef, useState, useSyncExternalStore } from 'react';

import type { Peaks } from '../results/loader.js';
import { type ClipRow, TICKS_PER_SECOND } from '../results/model.js';
import { type Transcript, snapEnd, snapStart } from '../results/transcript.js';
import { waveformPath } from '../results/waveform.js';
import type { PlaybackController } from './playback.js';
import { type Cut, FRAME_TICKS, clamp, clockTenths, rulerStep, timecode } from './review.js';

/** The shortest cut a handle can make: a second, so a clip is never a flash. */
const MINIMUM_CUT_TICKS = TICKS_PER_SECOND;

/** How much of a cut's own length is shown either side of it at first. */
export function initialView(cut: Cut, durationTicks: number): Cut {
  const length = cut.endTicks - cut.startTicks;
  const margin = Math.max(8 * TICKS_PER_SECOND, length * 0.35);
  return {
    startTicks: Math.max(0, cut.startTicks - margin),
    endTicks: Math.min(durationTicks, cut.endTicks + margin),
  };
}

/** A view zoomed by `factor` about `anchor`, kept inside the recording. */
export function zoomView(view: Cut, factor: number, anchor: number, durationTicks: number): Cut {
  const span = view.endTicks - view.startTicks;
  const next = clamp(span * factor, 2 * TICKS_PER_SECOND, durationTicks);
  const share = span <= 0 ? 0.5 : (anchor - view.startTicks) / span;
  const start = clamp(anchor - share * next, 0, Math.max(0, durationTicks - next));
  return { startTicks: start, endTicks: start + next };
}

/** A view slid by `ticks`, kept inside the recording. */
export function panView(view: Cut, ticks: number, durationTicks: number): Cut {
  const span = view.endTicks - view.startTicks;
  const start = clamp(view.startTicks + ticks, 0, Math.max(0, durationTicks - span));
  return { startTicks: start, endTicks: start + span };
}

function Playhead({
  controller,
  from,
  span,
}: {
  readonly controller: PlaybackController;
  readonly from: number;
  readonly span: number;
}) {
  const ticks = useSyncExternalStore(controller.subscribe, () => controller.getState().ticks);
  const share = (ticks - from) / span;
  if (share < 0 || share > 1) return null;
  return (
    <span
      className="review-playhead"
      style={{ left: `${share * 100}%` }}
      data-testid="playhead"
      aria-hidden="true"
    />
  );
}

export interface OverviewProps {
  readonly rows: readonly ClipRow[];
  readonly candidateId: string;
  readonly durationTicks: number;
  readonly view: Cut;
  readonly controller: PlaybackController;
  readonly onSelect: (candidateId: string) => void;
  readonly onSeek: (ticks: number) => void;
}

/** The whole episode, every candidate on it. */
export function Overview({
  rows,
  candidateId,
  durationTicks,
  view,
  controller,
  onSelect,
  onSeek,
}: OverviewProps) {
  const lane = useRef<HTMLDivElement>(null);
  const span = Math.max(1, durationTicks);
  const at = (ticks: number) => `${(ticks / span) * 100}%`;
  return (
    <div className="review-overview">
      <span className="review-lane-label">Episode</span>
      <div
        ref={lane}
        className="review-overview-lane"
        role="group"
        aria-label="The whole recording, with every clip in this review"
        onPointerDown={(event) => {
          if ((event.target as HTMLElement).closest('button')) return;
          const box = lane.current?.getBoundingClientRect();
          if (!box || box.width === 0) return;
          onSeek(((event.clientX - box.left) / box.width) * span);
        }}
      >
        <span
          className="review-overview-window"
          style={{ left: at(view.startTicks), width: at(view.endTicks - view.startTicks) }}
          aria-hidden="true"
        />
        {rows.map((row) => (
          <button
            key={row.candidateId}
            type="button"
            className="review-overview-mark"
            data-decision={row.decision ?? 'none'}
            aria-current={row.candidateId === candidateId ? 'true' : undefined}
            aria-label={`Clip ${String(row.rank).padStart(2, '0')}: ${row.headline || 'Untitled clip'}`}
            title={`${String(row.rank).padStart(2, '0')} · ${row.headline}`}
            style={{ left: at(row.startTicks), width: at(row.endTicks - row.startTicks) }}
            onClick={() => onSelect(row.candidateId)}
          />
        ))}
        <Playhead controller={controller} from={0} span={span} />
      </div>
      <span className="review-lane-label mono">
        {clockTenths(durationTicks).replace(/\.\d$/, '')}
      </span>
    </div>
  );
}

export interface BoundaryStripProps {
  /** The cut on screen: the reviewer's draft, or the search's own. */
  readonly cut: Cut;
  /** The search's chosen cut, drawn when the reviewer has moved away from it. */
  readonly chosen: Cut;
  readonly alternative: Cut | null;
  /** Where the search suggested a clip could start and end. */
  readonly suggestedStarts: readonly number[];
  readonly suggestedEnds: readonly number[];
  readonly transcript: Transcript | null;
  readonly peaks: Peaks | null;
  readonly tileUrl: (atTicks: number) => string | null;
  readonly durationTicks: number;
  readonly view: Cut;
  readonly onView: (view: Cut) => void;
  readonly controller: PlaybackController;
  /** Null while the handles are not the reviewer's to move (auditioning). */
  readonly onCut: ((cut: Cut) => void) | null;
}

type Grab = 'in' | 'out' | 'playhead';

export function BoundaryStrip({
  cut,
  chosen,
  alternative,
  suggestedStarts,
  suggestedEnds,
  transcript,
  peaks,
  tileUrl,
  durationTicks,
  view,
  onView,
  controller,
  onCut,
}: BoundaryStripProps) {
  const track = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(0);
  const [dragging, setDragging] = useState<Grab | null>(null);
  const [hover, setHover] = useState<number | null>(null);
  const span = Math.max(1, view.endTicks - view.startTicks);
  const at = (ticks: number) => `${((ticks - view.startTicks) / span) * 100}%`;
  const wide = (from: number, to: number) => `${((to - from) / span) * 100}%`;

  useEffect(() => {
    const element = track.current;
    if (!element) return;
    const measure = () => setWidth(element.clientWidth);
    measure();
    if (typeof ResizeObserver === 'undefined') return;
    const watcher = new ResizeObserver(measure);
    watcher.observe(element);
    return () => watcher.disconnect();
  }, []);

  // A playhead sent out of view — by the overview, the transcript, a jump —
  // brings the view with it rather than leaving the strip showing somewhere
  // the reviewer is not.
  const ticks = useSyncExternalStore(controller.subscribe, () => controller.getState().ticks);
  useEffect(() => {
    if (dragging) return;
    if (ticks < view.startTicks || ticks > view.endTicks) {
      onView(panView(view, ticks - (view.startTicks + span / 2), durationTicks));
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps -- follows the playhead only
  }, [ticks]);

  const ticksAt = useCallback(
    (clientX: number) => {
      const box = track.current?.getBoundingClientRect();
      if (!box || box.width === 0) return view.startTicks;
      return view.startTicks + clamp((clientX - box.left) / box.width, 0, 1) * span;
    },
    [view.startTicks, span],
  );

  /** Where a handle lands: between two words, or failing a transcript, a suggestion. */
  const snap = useCallback(
    (edge: 'in' | 'out', raw: number) => {
      if (transcript && transcript.words.length > 0) {
        return edge === 'in' ? snapStart(transcript, raw) : snapEnd(transcript, raw);
      }
      const points = edge === 'in' ? suggestedStarts : suggestedEnds;
      if (points.length === 0) return Math.round(raw / FRAME_TICKS) * FRAME_TICKS;
      return points.reduce((best, point) =>
        Math.abs(point - raw) < Math.abs(best - raw) ? point : best,
      );
    },
    [transcript, suggestedStarts, suggestedEnds],
  );

  const moveEdge = useCallback(
    (edge: 'in' | 'out', raw: number) => {
      if (!onCut) return;
      const landed = snap(edge, raw);
      const next =
        edge === 'in'
          ? {
              startTicks: Math.min(landed, cut.endTicks - MINIMUM_CUT_TICKS),
              endTicks: cut.endTicks,
            }
          : {
              startTicks: cut.startTicks,
              endTicks: Math.max(landed, cut.startTicks + MINIMUM_CUT_TICKS),
            };
      onCut(next);
      controller.pause();
      controller.seek(edge === 'in' ? next.startTicks : next.endTicks);
    },
    [onCut, snap, cut, controller],
  );

  const handlePointer = (what: Grab) => (event: React.PointerEvent) => {
    event.preventDefault();
    event.stopPropagation();
    event.currentTarget.setPointerCapture?.(event.pointerId);
    setDragging(what);
    const raw = ticksAt(event.clientX);
    if (what === 'playhead') controller.seek(raw);
    else moveEdge(what, raw);
  };

  const onPointerMove = (event: React.PointerEvent) => {
    const raw = ticksAt(event.clientX);
    setHover(raw);
    if (!dragging) return;
    if (dragging === 'playhead') controller.seek(raw);
    else moveEdge(dragging, raw);
  };

  const release = (event: React.PointerEvent) => {
    if (!dragging) return;
    event.currentTarget.releasePointerCapture?.(event.pointerId);
    setDragging(null);
  };

  /** Arrow keys walk a handle one word boundary at a time. */
  const nudge = (edge: 'in' | 'out') => (event: React.KeyboardEvent) => {
    if (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight') return;
    event.preventDefault();
    event.stopPropagation();
    const current = edge === 'in' ? cut.startTicks : cut.endTicks;
    const direction = event.key === 'ArrowLeft' ? -1 : 1;
    // Step past the current boundary by a sliver and let the snap find the
    // next one, so a handle always moves one word and never zero.
    let probe = current + direction * FRAME_TICKS;
    for (let attempt = 0; attempt < 200; attempt += 1) {
      const landed = snap(edge, probe);
      if (landed !== current) {
        moveEdge(edge, landed);
        return;
      }
      probe += direction * FRAME_TICKS * 3;
    }
  };

  // Registered by hand, not as `onWheel`: React's wheel listener is passive,
  // and a pinch the strip cannot cancel zooms the whole window instead.
  const wheel = useRef({ view, width, span, durationTicks, onView, ticksAt });
  wheel.current = { view, width, span, durationTicks, onView, ticksAt };
  useEffect(() => {
    const element = track.current;
    if (!element) return;
    const listener = (event: WheelEvent) => {
      const current = wheel.current;
      // A pinch arrives as a wheel with ctrl held; so does a ctrl-scroll.
      if (event.ctrlKey || event.metaKey) {
        event.preventDefault();
        current.onView(
          zoomView(
            current.view,
            Math.exp(event.deltaY * 0.01),
            current.ticksAt(event.clientX),
            current.durationTicks,
          ),
        );
        return;
      }
      const sideways =
        Math.abs(event.deltaX) > Math.abs(event.deltaY)
          ? event.deltaX
          : event.shiftKey
            ? event.deltaY
            : 0;
      if (sideways !== 0) {
        event.preventDefault();
        current.onView(
          panView(
            current.view,
            (sideways / Math.max(1, current.width)) * current.span,
            current.durationTicks,
          ),
        );
      }
    };
    element.addEventListener('wheel', listener, { passive: false });
    return () => element.removeEventListener('wheel', listener);
  }, []);

  const step = rulerStep(span, width);
  const ticksOnRuler: number[] = [];
  for (let tick = Math.ceil(view.startTicks / step) * step; tick <= view.endTicks; tick += step) {
    ticksOnRuler.push(tick);
  }
  const fine = step < TICKS_PER_SECOND;
  const slots = Math.max(1, Math.floor(width / 54));
  const stills = Array.from({ length: slots }, (_, index) =>
    tileUrl(view.startTicks + ((index + 0.5) / slots) * span),
  );
  const waveform = useMemo(
    () => (peaks ? waveformPath(peaks, view.startTicks, view.endTicks) : ''),
    [peaks, view.startTicks, view.endTicks],
  );
  const sentenceStarts = (transcript?.sentences ?? [])
    .map((sentence) => sentence.startTicks)
    .filter((tick) => tick >= view.startTicks && tick <= view.endTicks);
  const moved = cut.startTicks !== chosen.startTicks || cut.endTicks !== chosen.endTicks;
  const inView = (tick: number) => tick >= view.startTicks && tick <= view.endTicks;

  return (
    <div className="review-strip">
      <div
        ref={track}
        className="review-strip-track"
        role="group"
        aria-label="The cut, in its surroundings"
        onPointerDown={handlePointer('playhead')}
        onPointerMove={onPointerMove}
        onPointerUp={release}
        onPointerCancel={release}
        onPointerLeave={() => setHover(null)}
        onDoubleClick={() => onView(initialView(cut, durationTicks))}
        data-dragging={dragging ?? undefined}
      >
        <div className="review-ruler" aria-hidden="true">
          {ticksOnRuler.map((tick) => (
            <span key={tick} style={{ left: at(tick) }}>
              {fine ? clockTenths(tick) : clockTenths(tick).replace(/\.\d$/, '')}
            </span>
          ))}
        </div>
        <div className="review-stills" aria-hidden="true">
          {stills.map((still, index) =>
            still ? (
              // eslint-disable-next-line react/no-array-index-key -- slots are positions
              <img key={index} src={still} alt="" loading="lazy" draggable={false} />
            ) : (
              // eslint-disable-next-line react/no-array-index-key -- slots are positions
              <span key={index} />
            ),
          )}
        </div>
        <div className="review-sound" aria-hidden="true">
          {waveform && (
            <svg viewBox="0 0 100 100" preserveAspectRatio="none">
              <path d={waveform} />
            </svg>
          )}
          {sentenceStarts.map((tick) => (
            <span key={tick} className="review-sentence-tick" style={{ left: at(tick) }} />
          ))}
        </div>

        {suggestedStarts.filter(inView).map((tick) => (
          <span
            key={`s${tick}`}
            className="review-suggestion"
            data-edge="in"
            style={{ left: at(tick) }}
            title={`Suggested start · ${timecode(tick)}`}
          />
        ))}
        {suggestedEnds.filter(inView).map((tick) => (
          <span
            key={`e${tick}`}
            className="review-suggestion"
            data-edge="out"
            style={{ left: at(tick) }}
            title={`Suggested end · ${timecode(tick)}`}
          />
        ))}
        {alternative && (
          <span
            className="review-cut-ghost"
            data-kind="alternative"
            style={{
              left: at(alternative.startTicks),
              width: wide(alternative.startTicks, alternative.endTicks),
            }}
            title="The search's alternative cut"
            aria-hidden="true"
          />
        )}
        {moved && (
          <span
            className="review-cut-ghost"
            data-kind="chosen"
            style={{ left: at(chosen.startTicks), width: wide(chosen.startTicks, chosen.endTicks) }}
            title="The search's cut"
            aria-hidden="true"
          />
        )}
        <span
          className="review-outside"
          style={{ left: 0, width: at(cut.startTicks) }}
          aria-hidden="true"
        />
        <span
          className="review-outside"
          style={{ left: at(cut.endTicks), right: 0 }}
          aria-hidden="true"
        />
        <span
          className="review-cut"
          style={{ left: at(cut.startTicks), width: wide(cut.startTicks, cut.endTicks) }}
          aria-hidden="true"
        />
        {onCut &&
          (['in', 'out'] as const).map((edge) => {
            const tick = edge === 'in' ? cut.startTicks : cut.endTicks;
            return (
              <button
                key={edge}
                type="button"
                role="slider"
                className="review-handle"
                data-edge={edge}
                style={{ left: at(tick) }}
                aria-label={edge === 'in' ? 'Start of the cut' : 'End of the cut'}
                aria-valuemin={0}
                aria-valuemax={durationTicks}
                aria-valuenow={tick}
                aria-valuetext={timecode(tick)}
                onPointerDown={handlePointer(edge)}
                onPointerMove={onPointerMove}
                onPointerUp={release}
                onPointerCancel={release}
                onKeyDown={nudge(edge)}
              />
            );
          })}
        {hover !== null && !dragging && (
          <span className="review-hover" style={{ left: at(hover) }} aria-hidden="true">
            <span className="mono">{clockTenths(hover)}</span>
          </span>
        )}
        <Playhead controller={controller} from={view.startTicks} span={span} />
      </div>
    </div>
  );
}
