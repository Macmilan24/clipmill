/**
 * Where the cut is, everywhere it could legally be, and where the playhead is.
 *
 * The ticks are the boundary lattice — every start and end the optimizer was
 * allowed to choose between, at their real positions. Drawing them is what makes
 * the strip an instrument rather than a diagram: an editor who thinks the cut is
 * late can see whether an earlier edge exists, and the handles snap to those
 * edges because a boundary between them is not a cut this system can make. Every
 * boundary from the index is word-aligned by construction, so snapping is not a
 * convenience — it is the constraint.
 *
 * Dragging a handle proposes a cut; it does not perform one. The daemon holds
 * the authority and will move a boundary it disagrees with, which is why taking
 * the cut is a separate, named action and why the response says where it landed.
 */
import { useCallback, useMemo, useRef, useState } from 'react';

import type { Peaks } from '../../results/loader.js';
import { TICKS_PER_SECOND } from '../../results/model.js';
import { timecode } from './Player.js';

export type Handle = 'in' | 'out';

export interface TimelineProps {
  /** The cut as it currently stands, which is the draft while one is dragged. */
  readonly startTicks: number;
  readonly endTicks: number;
  readonly latticeStarts: readonly number[];
  readonly latticeEnds: readonly number[];
  /** The runner-up, drawn where the lattice offered one. */
  readonly alternative: { readonly startTicks: number; readonly endTicks: number } | null;
  readonly positionTicks: number;
  /** The recording's loudness contour, drawn behind the lattice when published. */
  readonly peaks: Peaks | null;
  readonly onScrub: (ticks: number) => void;
  readonly onDraft: (edge: Handle, ticks: number) => void;
}

/**
 * The waveform for a window, as one SVG path.
 *
 * Read from the peaks the ingest measured — one min/max pair per bucket — and
 * never synthesised. Drawn as a mirrored band around the midline, which is how
 * an editor expects a waveform to read, and sampled to at most one bucket per
 * pixel-ish column so a long window does not produce a path with ten thousand
 * points.
 */
function waveformPath(peaks: Peaks, from: number, to: number, columns = 240): string {
  const firstBucket = Math.max(0, Math.floor(from / peaks.bucketTicks));
  const lastBucket = Math.min(peaks.values.length - 1, Math.ceil(to / peaks.bucketTicks));
  if (lastBucket <= firstBucket) {
    return '';
  }
  const step = Math.max(1, Math.floor((lastBucket - firstBucket) / columns));
  const top: string[] = [];
  const bottom: string[] = [];
  for (let bucket = firstBucket; bucket <= lastBucket; bucket += step) {
    let lo = 0;
    let hi = 0;
    for (let inner = bucket; inner < Math.min(bucket + step, lastBucket + 1); inner += 1) {
      const [min, max] = peaks.values[inner]!;
      lo = Math.min(lo, min);
      hi = Math.max(hi, max);
    }
    const x = (((bucket * peaks.bucketTicks - from) / (to - from)) * 100).toFixed(2);
    top.push(`${x},${(50 - (hi / 32_767) * 46).toFixed(1)}`);
    bottom.push(`${x},${(50 - (lo / 32_767) * 46).toFixed(1)}`);
  }
  return `M${top.join(' L')} L${bottom.toReversed().join(' L')} Z`;
}

/** The nearest legal edge, or the raw value when there is no lattice to hold. */
function snap(ticks: number, edges: readonly number[]): number {
  if (edges.length === 0) {
    return Math.round(ticks);
  }
  let best = edges[0]!;
  for (const edge of edges) {
    if (Math.abs(edge - ticks) < Math.abs(best - ticks)) {
      best = edge;
    }
  }
  return best;
}

export function Timeline({
  startTicks,
  endTicks,
  latticeStarts,
  latticeEnds,
  alternative,
  positionTicks,
  peaks,
  onScrub,
  onDraft,
}: TimelineProps) {
  const track = useRef<HTMLDivElement>(null);
  const [dragging, setDragging] = useState<Handle | 'playhead' | null>(null);

  // The window is the lattice's own extent where there is one, so every edge a
  // handle can snap to is reachable without scrolling.
  const edges = [...latticeStarts, ...latticeEnds];
  const pad = Math.max((endTicks - startTicks) * 0.25, TICKS_PER_SECOND);
  const from = Math.max(
    0,
    Math.min(startTicks - pad, ...(edges.length > 0 ? edges : [startTicks])),
  );
  const to = Math.max(endTicks + pad, ...(edges.length > 0 ? edges : [endTicks]));
  const width = Math.max(1, to - from);
  const at = (ticks: number) => `${((ticks - from) / width) * 100}%`;
  const waveform = useMemo(() => (peaks ? waveformPath(peaks, from, to) : ''), [peaks, from, to]);

  const ticksAtClientX = useCallback(
    (clientX: number) => {
      const box = track.current?.getBoundingClientRect();
      if (!box || box.width === 0) {
        return from;
      }
      const ratio = Math.min(1, Math.max(0, (clientX - box.left) / box.width));
      return from + ratio * width;
    },
    [from, width],
  );

  const move = useCallback(
    (what: Handle | 'playhead', clientX: number) => {
      const raw = ticksAtClientX(clientX);
      if (what === 'playhead') {
        onScrub(Math.round(raw));
        return;
      }
      onDraft(what, snap(raw, what === 'in' ? latticeStarts : latticeEnds));
    },
    [latticeEnds, latticeStarts, onDraft, onScrub, ticksAtClientX],
  );

  const grab = (what: Handle | 'playhead') => (event: React.PointerEvent) => {
    event.preventDefault();
    event.stopPropagation();
    event.currentTarget.setPointerCapture?.(event.pointerId);
    setDragging(what);
    move(what, event.clientX);
  };

  const onPointerMove = (event: React.PointerEvent) => {
    if (dragging) {
      move(dragging, event.clientX);
    }
  };

  const release = (event: React.PointerEvent) => {
    if (dragging) {
      event.currentTarget.releasePointerCapture?.(event.pointerId);
      setDragging(null);
    }
  };

  /** Arrow keys nudge a handle to its neighbouring legal edge. */
  const nudge = (edge: Handle) => (event: React.KeyboardEvent) => {
    const legal = (edge === 'in' ? latticeStarts : latticeEnds).toSorted((a, b) => a - b);
    if (legal.length === 0) {
      return;
    }
    const current = edge === 'in' ? startTicks : endTicks;
    if (event.key === 'ArrowLeft') {
      event.preventDefault();
      const before = legal.findLast((tick) => tick < current);
      if (before !== undefined) {
        onDraft(edge, before);
      }
    } else if (event.key === 'ArrowRight') {
      event.preventDefault();
      const after = legal.find((tick) => tick > current);
      if (after !== undefined) {
        onDraft(edge, after);
      }
    }
  };

  return (
    <figure className="flex shrink-0 flex-col gap-2">
      <div
        ref={track}
        onPointerDown={grab('playhead')}
        onPointerMove={onPointerMove}
        onPointerUp={release}
        onPointerCancel={release}
        className="relative h-[72px] cursor-pointer overflow-hidden rounded-[var(--cm-radius-control)] border border-[var(--cm-recessed-border)] bg-[var(--cm-recessed)] select-none"
        role="group"
        aria-label="The cut, against the boundary lattice"
      >
        {waveform && (
          <svg
            aria-hidden
            className="pointer-events-none absolute inset-0 size-full"
            viewBox="0 0 100 100"
            preserveAspectRatio="none"
          >
            <path d={waveform} fill="var(--cm-text-disabled)" fillOpacity="0.45" />
          </svg>
        )}

        {latticeStarts.map((tick) => (
          <span
            key={`s-${tick}`}
            title={`A legal start at ${timecode(tick)}`}
            className="pointer-events-none absolute top-1.5 h-4 w-px"
            style={{ left: at(tick), background: 'var(--cm-text-disabled)' }}
          />
        ))}
        {latticeEnds.map((tick) => (
          <span
            key={`e-${tick}`}
            title={`A legal end at ${timecode(tick)}`}
            className="pointer-events-none absolute bottom-1.5 h-4 w-px"
            style={{ left: at(tick), background: 'var(--cm-text-disabled)' }}
          />
        ))}

        {alternative && (
          <span
            className="pointer-events-none absolute inset-y-6 rounded-sm border border-dashed"
            style={{
              left: at(alternative.startTicks),
              width: `${((alternative.endTicks - alternative.startTicks) / width) * 100}%`,
              borderColor: 'var(--cm-text-muted)',
            }}
            title="The runner-up cut"
          />
        )}

        <span
          className="pointer-events-none absolute inset-y-0"
          style={{
            left: at(startTicks),
            width: `${((endTicks - startTicks) / width) * 100}%`,
            background: 'var(--cm-accent-selected)',
            borderTop: '1px solid var(--cm-accent)',
            borderBottom: '1px solid var(--cm-accent)',
          }}
        />

        {(['in', 'out'] as const).map((edge) => {
          const ticks = edge === 'in' ? startTicks : endTicks;
          return (
            <button
              key={edge}
              type="button"
              onPointerDown={grab(edge)}
              onPointerMove={onPointerMove}
              onPointerUp={release}
              onPointerCancel={release}
              onKeyDown={nudge(edge)}
              aria-label={`${edge === 'in' ? 'In' : 'Out'} point at ${timecode(ticks)}. Arrow keys move it to the next legal edge.`}
              className="absolute inset-y-0 w-2.5 cursor-col-resize touch-none transition-[width,background] hover:w-3.5"
              style={{
                left: at(ticks),
                marginLeft: edge === 'out' ? '-0.625rem' : 0,
                background: 'var(--cm-accent)',
                borderRadius: edge === 'in' ? '3px 0 0 3px' : '0 3px 3px 0',
              }}
            >
              <span aria-hidden className="mx-auto block h-4 w-px bg-white/60" />
            </button>
          );
        })}

        <span
          onPointerDown={grab('playhead')}
          className="absolute inset-y-0 z-10 w-0.5 cursor-col-resize"
          style={{
            left: at(positionTicks),
            background: 'var(--cm-warning-ink)',
            boxShadow: '0 0 8px color-mix(in srgb, var(--cm-warning-ink) 70%, transparent)',
          }}
        />
      </div>

      <figcaption className="mono flex items-center justify-between text-[10px] text-[var(--cm-text-muted)]">
        <span>IN {timecode(startTicks)}</span>
        <span className="text-[var(--cm-warning-ink)]">
          {((endTicks - startTicks) / TICKS_PER_SECOND).toFixed(2)}s
          {latticeStarts.length > 0 && (
            <span className="ml-2 text-[var(--cm-text-muted)]">
              {latticeStarts.length}×{latticeEnds.length} legal pairs
            </span>
          )}
        </span>
        <span>OUT {timecode(endTicks)}</span>
      </figcaption>
    </figure>
  );
}
