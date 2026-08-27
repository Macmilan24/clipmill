/**
 * Every candidate, dense enough to compare and quiet enough to scan.
 *
 * A row is a button in a grid rather than a table cell, because what it is is a
 * control: pressing it opens that clip. The columns are laid out with one grid
 * template shared by the header and every row, so the header cannot drift out of
 * alignment with what it labels.
 *
 * The signal dots are the row's own warnings and penalties, each with the text
 * behind it on hover. They are not a rating: a dot means the ranker recorded
 * something about this clip, and the tooltip says what. Inventing a green
 * "strong hook" dot would be inventing a measurement, so a clip the ranker had
 * nothing to say about shows no dots at all.
 *
 * Arrow keys move the selection and Enter opens it, because a board whose rows
 * can only be reached with a pointer is a board an editor cannot work quickly.
 */
import { ChevronRight } from 'lucide-react';
import { useCallback, useEffect, useRef } from 'react';

import type { ClipDecision } from '../../daemon/client.js';
import { type ClipRow, clock, duration } from '../model.js';
import { bandInk } from './ScoreRing.js';

// The trailing column is the open affordance. Selecting and opening are
// different acts, and below the rail's breakpoint selecting has nothing visible
// to show for itself — so the way into the inspector has to be on the row
// rather than in a panel that is not on screen.
const COLUMNS = 'grid-cols-[2.25rem_minmax(0,1fr)_4rem_4.5rem_3.5rem_5.5rem_2rem]';

const DECISION_STYLE: Readonly<
  Record<ClipDecision, { readonly label: string; readonly ink: string; readonly wash: string }>
> = {
  approved: {
    label: 'Approved',
    ink: 'var(--cm-success-ink)',
    wash: 'color-mix(in srgb, var(--cm-success-ink) 12%, transparent)',
  },
  kept: {
    label: 'Kept',
    ink: 'var(--cm-text-secondary)',
    wash: 'var(--cm-recessed)',
  },
  rejected: {
    label: 'Rejected',
    ink: 'var(--cm-danger-ink)',
    wash: 'color-mix(in srgb, var(--cm-danger-ink) 12%, transparent)',
  },
};

export interface CandidateTableProps {
  readonly rows: readonly ClipRow[];
  readonly selectedId: string | null;
  readonly onSelect: (candidateId: string) => void;
  readonly onOpen: (candidateId: string) => void;
}

export function CandidateTable({ rows, selectedId, onSelect, onOpen }: CandidateTableProps) {
  const listRef = useRef<HTMLDivElement>(null);

  // Keeps the keyboard selection in view without stealing focus from the row,
  // which would break the arrow-key loop it exists to serve.
  useEffect(() => {
    if (!selectedId) {
      return;
    }
    // Asked for by its selected state rather than by its id: an id is caller
    // data and would need escaping before it could go in a selector, and the
    // row already announces which one it is.
    const active = listRef.current?.querySelector('[aria-selected="true"]');
    // Guarded because scrolling is a convenience, not a behaviour: an
    // environment without it (jsdom, and any headless renderer) should show the
    // right row selected rather than fail to render the board at all.
    if (active instanceof HTMLElement && typeof active.scrollIntoView === 'function') {
      active.scrollIntoView({ block: 'nearest' });
    }
  }, [selectedId]);

  const onKeyDown = useCallback(
    (event: React.KeyboardEvent) => {
      if (rows.length === 0) {
        return;
      }
      const at = rows.findIndex((row) => row.candidateId === selectedId);
      if (event.key === 'ArrowDown' || event.key === 'j') {
        event.preventDefault();
        onSelect(rows[Math.min(rows.length - 1, at + 1)]!.candidateId);
      } else if (event.key === 'ArrowUp' || event.key === 'k') {
        event.preventDefault();
        onSelect(rows[Math.max(0, at <= 0 ? 0 : at - 1)]!.candidateId);
      } else if (event.key === 'Enter' && selectedId) {
        event.preventDefault();
        onOpen(selectedId);
      }
    },
    [onOpen, onSelect, rows, selectedId],
  );

  return (
    <div className="glass flex min-h-0 flex-col overflow-hidden rounded-[var(--cm-radius-card)]">
      <div
        className={`grid ${COLUMNS} shrink-0 items-center gap-4 border-b border-[var(--cm-glass-border)] bg-[var(--cm-recessed)] px-4 py-2.5 text-[10px] font-medium tracking-[0.09em] text-[var(--cm-text-muted)] uppercase`}
        role="row"
      >
        <span className="text-center">#</span>
        <span>Candidate</span>
        <span className="text-right">Length</span>
        <span className="pl-3">Signals</span>
        <span className="text-right">Score</span>
        <span className="text-right">State</span>
        <span className="sr-only">Open</span>
      </div>

      <div
        ref={listRef}
        role="listbox"
        aria-label="Clip candidates"
        tabIndex={0}
        onKeyDown={onKeyDown}
        className="min-h-0 flex-1 overflow-y-auto focus:outline-none"
      >
        {rows.map((row, index) => {
          const selected = row.candidateId === selectedId;
          const decision = row.decision ? DECISION_STYLE[row.decision] : null;
          return (
            <div
              key={row.candidateId}
              data-candidate={row.candidateId}
              role="option"
              aria-selected={selected}
              tabIndex={-1}
              onClick={() => onSelect(row.candidateId)}
              onDoubleClick={() => onOpen(row.candidateId)}
              style={{
                animationDelay: `${Math.min(index, 12) * 24}ms`,
                borderLeftColor: selected ? 'var(--cm-accent)' : 'transparent',
                background: selected ? 'var(--cm-accent-selected)' : undefined,
              }}
              className={`grid ${COLUMNS} animate-in fade-in slide-in-from-bottom-1 cursor-pointer items-center gap-4 border-b border-l-2 border-b-[var(--cm-glass-border)] px-4 py-3 duration-300 [animation-fill-mode:backwards] hover:bg-[var(--cm-glass-elevated)]`}
            >
              <span className="mono text-center text-[11px] text-[var(--cm-text-muted)]">
                {String(row.rank).padStart(2, '0')}
              </span>

              <span className="flex min-w-0 flex-col gap-0.5">
                <span className="truncate text-[13px] font-medium text-[var(--cm-text-primary)]">
                  {row.headline || (
                    <em className="text-[var(--cm-text-muted)]">No opening line indexed</em>
                  )}
                </span>
                <span className="mono truncate text-[10px] text-[var(--cm-text-muted)]">
                  {clock(row.startTicks)} – {clock(row.endTicks)}
                </span>
              </span>

              <span className="mono text-right text-[11px] text-[var(--cm-text-secondary)]">
                {duration(row.durationSeconds)}
              </span>

              <span
                className="flex items-center gap-1 pl-3"
                role="group"
                aria-label={
                  row.warnings.length + row.penalties.length === 0
                    ? 'No signals recorded'
                    : `${row.warnings.length + row.penalties.length} signals`
                }
              >
                {row.warnings.map((warning) => (
                  <span
                    key={warning}
                    title={warning}
                    className="size-2 rounded-full"
                    style={{ background: 'var(--cm-warning-ink)' }}
                  />
                ))}
                {row.penalties.map((penalty) => (
                  <span
                    key={penalty.reason}
                    title={`${penalty.reason.replaceAll('_', ' ')} −${penalty.value}`}
                    className="size-2 rounded-full"
                    style={{ background: 'var(--cm-danger-ink)' }}
                  />
                ))}
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
                  className="rounded px-2 py-0.5 text-[10px] font-semibold tracking-wide uppercase"
                  style={{
                    color: decision?.ink ?? 'var(--cm-text-muted)',
                    background: decision?.wash ?? 'var(--cm-recessed)',
                  }}
                >
                  {decision?.label ?? 'New'}
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
