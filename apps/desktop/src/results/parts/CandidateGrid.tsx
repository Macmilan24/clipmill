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
      aria-multiselectable
      className="grid min-h-0 flex-1 auto-rows-max grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-3 overflow-y-auto pr-1"
    >
      {rows.map((row, index) => {
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
            onClick={() => onFocus(row.candidateId)}
            onDoubleClick={() => onOpen(row.candidateId)}
            style={{ animationDelay: `${Math.min(index, 12) * 30}ms` }}
            className={`glass animate-in fade-in zoom-in-95 group flex cursor-pointer flex-col overflow-hidden rounded-[var(--cm-radius-card)] duration-300 [animation-fill-mode:backwards] transition-shadow ${focused ? 'ring-2 ring-[var(--cm-accent)]' : ''}`}
          >
            <div className="relative aspect-video w-full bg-[var(--cm-recessed)]">
              {still ? (
                <img src={still} alt="" className="size-full object-cover" />
              ) : (
                <p className="grid size-full place-items-center px-4 text-center text-[10px] text-[var(--cm-text-muted)]">
                  No filmstrip published
                </p>
              )}
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
                style={{ color: 'white', background: TONE_INK[state.tone] }}
              >
                {state.label}
              </span>
              <span className="mono absolute right-2 bottom-2 rounded bg-black/60 px-1.5 py-0.5 text-[10px] text-white">
                {duration(row.durationSeconds)}
              </span>
              <span className="absolute bottom-2 left-2">
                <ScoreRing score={row.displayScore} band={row.band} size="sm" />
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
                  {signalsFor(row)
                    .slice(0, 4)
                    .map((signal) => (
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
