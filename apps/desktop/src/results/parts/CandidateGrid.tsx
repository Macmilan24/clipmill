/**
 * The same candidates as cards, for comparing by picture rather than by row.
 *
 * Each card's still is the filmstrip tile nearest the clip's first frame — a
 * frame the ingest actually cut from this recording. A card with no still says
 * the run published no filmstrip, because a stock image would be a picture of
 * something that is not this clip.
 *
 * The same state, signal and selection rules as the table, from the same
 * module, so switching layouts changes how the board looks and never what it
 * says.
 */
import { MediaStill } from '../../components/MediaStill.js';
import { Checkbox } from '../../components/ui/checkbox.js';
import { type ClipRow, clock, duration } from '../model.js';
import { ScoreRing } from './ScoreRing.js';
import { TONE_INK, signalsFor, stateOf } from './state.js';

export interface CandidateGridProps {
  readonly rows: readonly ClipRow[];
  readonly focusedId: string | null;
  readonly checked: ReadonlySet<string>;
  readonly tileUrl: (atTicks: number) => string | null;
  readonly onFocus: (candidateId: string) => void;
  readonly onToggle: (candidateId: string) => void;
  readonly onOpen: (candidateId: string) => void;
}

export function CandidateGrid({
  rows,
  focusedId,
  checked,
  tileUrl,
  onFocus,
  onToggle,
  onOpen,
}: CandidateGridProps) {
  return (
    <div
      role="listbox"
      aria-label="Clip candidates"
      className="grid min-h-0 flex-1 auto-rows-max grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-3 overflow-y-auto pr-1"
    >
      {rows.map((row) => {
        const focused = row.candidateId === focusedId;
        const ticked = checked.has(row.candidateId);
        const state = stateOf(row);
        const still = tileUrl(row.startTicks);
        return (
          <div
            key={row.candidateId}
            role="option"
            aria-selected={focused}
            aria-checked={ticked}
            tabIndex={0}
            onKeyDown={(event) => {
              if (event.target !== event.currentTarget) return;
              if (event.key === 'Enter') {
                event.preventDefault();
                onOpen(row.candidateId);
              }
              if (event.key === ' ') {
                event.preventDefault();
                onToggle(row.candidateId);
              }
            }}
            onFocus={(event) => {
              if (event.target === event.currentTarget) onFocus(row.candidateId);
            }}
            onClick={() => onFocus(row.candidateId)}
            onDoubleClick={() => onOpen(row.candidateId)}
            className={`glass group flex cursor-pointer flex-col overflow-hidden rounded-[var(--cm-radius-card)] duration-150 transition-shadow ${focused ? 'ring-2 ring-[var(--cm-accent)]' : ''}`}
          >
            <div className="relative aspect-video w-full bg-[var(--cm-recessed)]">
              <MediaStill src={still} />
              <div
                aria-hidden
                className="pointer-events-none absolute inset-0"
                style={{
                  background: 'linear-gradient(to top, rgba(0,0,0,0.7), transparent 55%)',
                }}
              />
              <span className="absolute top-2 left-2" onClick={(event) => event.stopPropagation()}>
                <Checkbox
                  checked={ticked}
                  onCheckedChange={() => onToggle(row.candidateId)}
                  aria-label={`Select ${row.headline || 'this clip'} for a batch action`}
                  className="bg-black/40"
                />
              </span>
              <span
                className="absolute top-2 right-2 rounded px-1.5 py-0.5 text-[9px] font-bold tracking-wide uppercase"
                style={{
                  color: 'white',
                  background: state.tone === 'accent' ? 'var(--cm-accent)' : TONE_INK[state.tone],
                }}
              >
                {state.label}
              </span>
              <span className="mono absolute right-2 bottom-2 rounded bg-black/60 px-1.5 py-0.5 text-[10px] text-white">
                {duration(row.durationSeconds)}
              </span>
              <span className="absolute bottom-2 left-2">
                {row.review ? (
                  <span
                    title={row.review.reasons.join('; ')}
                    className="rounded bg-black/70 px-2 py-1 text-[10px] text-white"
                  >
                    {row.bandLabel}
                  </span>
                ) : (
                  <ScoreRing score={row.displayScore} band={row.band} size="sm" />
                )}
              </span>
            </div>
            <div className="flex flex-col gap-1.5 p-3">
              <p className="line-clamp-2 text-[12px] leading-snug font-medium text-[var(--cm-text-primary)]">
                {row.headline || <em className="text-[var(--cm-text-muted)]">No opening line</em>}
              </p>
              <div className="flex items-center justify-between">
                <span className="mono text-[10px] text-[var(--cm-text-muted)]">
                  #{row.rank} · {clock(row.startTicks)}
                </span>
                <span className="flex gap-1">
                  {(row.review ? [] : signalsFor(row)).slice(0, 4).map((signal) => (
                    <span
                      key={signal.key}
                      title={signal.label}
                      className="size-1.5 rounded-full"
                      style={{ background: TONE_INK[signal.tone] }}
                    />
                  ))}
                </span>
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}
