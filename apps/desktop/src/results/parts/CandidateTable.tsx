/**
 * Every candidate, dense enough to compare and quiet enough to scan.
 *
 * A row is a control in a grid rather than a table cell: pressing it selects
 * that clip for the rail, ticking it adds it to the set the footer will act on,
 * and the chevron opens it. Three different acts, three different targets,
 * because a click that did all of them would leave an editor unable to look at
 * a clip without also committing to it.
 *
 * The columns share one grid template with the header, so the header cannot
 * drift out of alignment with what it labels.
 *
 * Arrow keys move the selection, Space ticks it and Enter opens it, because a
 * board whose rows can only be reached with a pointer is a board an editor
 * cannot work quickly.
 */
import { ChevronRight } from 'lucide-react';
import { useCallback, useEffect, useRef } from 'react';

import { Checkbox } from '../../components/ui/checkbox.js';
import { type ClipRow, clock, duration } from '../model.js';
import { bandInk } from './ScoreRing.js';
import { TONE_INK, signalsFor, stateOf, wash } from './state.js';

const COLUMNS = 'grid-cols-[2rem_1.75rem_minmax(0,1fr)_3.5rem_5rem_3.25rem_6.25rem_2rem]';

export interface CandidateTableProps {
  readonly rows: readonly ClipRow[];
  /** The row the rail describes. */
  readonly focusedId: string | null;
  /** The rows ticked for a batch action. */
  readonly checked: ReadonlySet<string>;
  readonly onFocus: (candidateId: string) => void;
  readonly onToggle: (candidateId: string) => void;
  readonly onOpen: (candidateId: string) => void;
}

export function CandidateTable({
  rows,
  focusedId,
  checked,
  onFocus,
  onToggle,
  onOpen,
}: CandidateTableProps) {
  const listRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const active = listRef.current?.querySelector('[aria-selected="true"]');
    if (active instanceof HTMLElement && typeof active.scrollIntoView === 'function') {
      active.scrollIntoView({ block: 'nearest' });
    }
  }, [focusedId]);

  const onKeyDown = useCallback(
    (event: React.KeyboardEvent) => {
      if (rows.length === 0) {
        return;
      }
      const at = rows.findIndex((row) => row.candidateId === focusedId);
      if (event.key === 'ArrowDown' || event.key === 'j') {
        event.preventDefault();
        onFocus(rows[Math.min(rows.length - 1, at + 1)]!.candidateId);
      } else if (event.key === 'ArrowUp' || event.key === 'k') {
        event.preventDefault();
        onFocus(rows[Math.max(0, at <= 0 ? 0 : at - 1)]!.candidateId);
      } else if (event.key === ' ' && focusedId) {
        event.preventDefault();
        onToggle(focusedId);
      } else if (event.key === 'Enter' && focusedId) {
        event.preventDefault();
        onOpen(focusedId);
      }
    },
    [focusedId, onFocus, onOpen, onToggle, rows],
  );

  return (
    <div className="glass flex min-h-0 flex-col overflow-hidden rounded-[var(--cm-radius-card)]">
      <div
        className={`grid ${COLUMNS} shrink-0 items-center gap-3 border-b border-[var(--cm-glass-border)] bg-[var(--cm-recessed)] px-4 py-2.5 text-[10px] font-medium tracking-[0.09em] text-[var(--cm-text-muted)] uppercase`}
        role="row"
      >
        <span className="text-center">#</span>
        <span className="sr-only">Select</span>
        <span>Candidate</span>
        <span className="text-right">Dur</span>
        <span className="pl-2">Signals</span>
        <span className="text-right">Score</span>
        <span className="text-right">State</span>
        <span className="sr-only">Open</span>
      </div>

      <div
        ref={listRef}
        role="listbox"
        aria-label="Clip candidates"
        aria-multiselectable
        tabIndex={0}
        onKeyDown={onKeyDown}
        className="min-h-0 flex-1 overflow-y-auto focus:outline-none"
      >
        {rows.map((row, index) => {
          const focused = row.candidateId === focusedId;
          const ticked = checked.has(row.candidateId);
          const state = stateOf(row);
          const signals = signalsFor(row);
          return (
            <div
              key={row.candidateId}
              role="option"
              aria-selected={focused}
              aria-checked={ticked}
              tabIndex={-1}
              onClick={() => onFocus(row.candidateId)}
              onDoubleClick={() => onOpen(row.candidateId)}
              style={{
                animationDelay: `${Math.min(index, 12) * 24}ms`,
                borderLeftColor: focused ? 'var(--cm-accent)' : 'transparent',
                background: focused
                  ? 'var(--cm-accent-selected)'
                  : ticked
                    ? 'color-mix(in srgb, var(--cm-accent) 6%, transparent)'
                    : undefined,
                opacity: row.decision === 'rejected' ? 0.6 : 1,
              }}
              className={`group grid ${COLUMNS} animate-in fade-in slide-in-from-bottom-1 cursor-pointer items-center gap-3 border-b border-l-2 border-b-[var(--cm-glass-border)] px-4 py-3 duration-300 [animation-fill-mode:backwards] hover:bg-[var(--cm-glass-elevated)]`}
            >
              <span className="mono text-center text-[11px] text-[var(--cm-text-muted)]">
                {String(row.rank).padStart(2, '0')}
              </span>

              <span className="flex justify-center" onClick={(event) => event.stopPropagation()}>
                <Checkbox
                  checked={ticked}
                  onCheckedChange={() => onToggle(row.candidateId)}
                  aria-label={`Select ${row.headline || 'this clip'} for a batch action`}
                />
              </span>

              <span className="flex min-w-0 flex-col gap-0.5">
                <span
                  className={`truncate text-[13px] font-medium text-[var(--cm-text-primary)] ${row.decision === 'rejected' ? 'line-through decoration-[var(--cm-text-muted)]' : ''}`}
                >
                  {row.headline || (
                    <em className="text-[var(--cm-text-muted)]">No opening line indexed</em>
                  )}
                </span>
                <span className="mono flex items-center gap-2 truncate text-[10px] text-[var(--cm-text-muted)]">
                  {clock(row.startTicks)} – {clock(row.endTicks)}
                  {row.proposer && (
                    <>
                      <span aria-hidden>·</span>
                      <span className="font-sans">{row.proposer}</span>
                    </>
                  )}
                </span>
              </span>

              <span className="mono text-right text-[11px] text-[var(--cm-text-secondary)]">
                {duration(row.durationSeconds)}
              </span>

              <span
                className="flex items-center gap-1 pl-2"
                role="group"
                aria-label={
                  signals.length === 0 ? 'No signals recorded' : `${signals.length} signals`
                }
              >
                {signals.slice(0, 4).map((signal) => (
                  <span
                    key={signal.key}
                    title={signal.label}
                    className="size-2 rounded-full"
                    style={{ background: TONE_INK[signal.tone] }}
                  />
                ))}
                {signals.length > 4 && (
                  <span className="mono text-[9px] text-[var(--cm-text-muted)]">
                    +{signals.length - 4}
                  </span>
                )}
              </span>

              <span
                className="mono text-right text-[13px] font-medium"
                style={{ color: bandInk(row.band) }}
                title={row.bandLabel}
              >
                {row.displayScore}
              </span>

              <span className="flex justify-end">
                <span
                  className="rounded px-2 py-0.5 text-[10px] font-semibold tracking-wide whitespace-nowrap uppercase"
                  style={{ color: TONE_INK[state.tone], background: wash(state.tone) }}
                >
                  {state.label}
                </span>
              </span>

              <button
                type="button"
                aria-label={`Open ${row.headline || 'this clip'} in the inspector`}
                onClick={(event) => {
                  event.stopPropagation();
                  onOpen(row.candidateId);
                }}
                className="grid size-7 place-items-center justify-self-end rounded-[6px] text-[var(--cm-text-muted)] transition-colors hover:bg-[var(--cm-glass-elevated)] hover:text-[var(--cm-text-primary)]"
              >
                <ChevronRight className="size-4" aria-hidden />
              </button>
            </div>
          );
        })}
      </div>
    </div>
  );
}
