/**
 * Every other candidate, so a judgement is made in comparison.
 *
 * A clip is rarely good or bad on its own — it is better or worse than the one
 * below it — so the rail stays visible while a decision is being made rather
 * than sending an editor back to the board between clips. Each row carries the
 * decision already recorded against it, because the most useful thing to know
 * about the next clip is whether it has already been dealt with.
 */
import type { ClipDecision } from '../../daemon/client.js';
import { type ClipRow, clock } from '../../results/model.js';
import { bandInk } from '../../results/parts/ScoreRing.js';

const DECISION_MARK: Readonly<Record<ClipDecision, { label: string; ink: string }>> = {
  approved: { label: 'Approved', ink: 'var(--cm-success-ink)' },
  kept: { label: 'Kept', ink: 'var(--cm-text-secondary)' },
  rejected: { label: 'Rejected', ink: 'var(--cm-danger-ink)' },
};

export interface CandidateRailProps {
  readonly rows: readonly ClipRow[];
  readonly candidateId: string;
  readonly onSelect: (candidateId: string) => void;
}

export function CandidateRail({ rows, candidateId, onSelect }: CandidateRailProps) {
  return (
    <nav
      className="glass flex w-[212px] shrink-0 flex-col overflow-hidden rounded-[var(--cm-radius-card)]"
      aria-label="Candidates"
    >
      <h2 className="shrink-0 border-b border-[var(--cm-glass-border)] px-3 py-2.5 text-[10px] font-medium tracking-[0.09em] text-[var(--cm-text-muted)] uppercase">
        Candidates ({rows.length})
      </h2>
      <ul className="min-h-0 flex-1 overflow-y-auto p-2">
        {rows.map((row) => {
          const active = row.candidateId === candidateId;
          const mark = row.decision ? DECISION_MARK[row.decision] : null;
          return (
            <li key={row.candidateId}>
              <button
                type="button"
                onClick={() => onSelect(row.candidateId)}
                aria-current={active}
                className="relative flex w-full flex-col gap-1 rounded-[var(--cm-radius-control)] px-3 py-2.5 text-left transition-colors duration-150 hover:bg-[var(--cm-glass-elevated)]"
                style={active ? { background: 'var(--cm-accent-selected)' } : undefined}
              >
                {active && (
                  <span
                    aria-hidden
                    className="absolute inset-y-2 left-0 w-0.5 rounded-full"
                    style={{ background: 'var(--cm-accent)' }}
                  />
                )}
                <span className="flex items-start justify-between gap-2">
                  <span
                    className="truncate text-[12px] font-medium"
                    style={{
                      color: active ? 'var(--cm-text-primary)' : 'var(--cm-text-secondary)',
                    }}
                  >
                    {row.headline || 'Untitled clip'}
                  </span>
                  <span
                    className="mono shrink-0 text-[11px]"
                    style={{ color: active ? bandInk(row.band) : 'var(--cm-text-muted)' }}
                  >
                    {row.displayScore}
                  </span>
                </span>
                <span className="flex items-center gap-1.5 text-[10px] text-[var(--cm-text-muted)]">
                  <span className="mono">{clock(row.startTicks)}</span>
                  <span aria-hidden>·</span>
                  <span>{row.bandLabel}</span>
                  {mark && (
                    <>
                      <span aria-hidden>·</span>
                      <span style={{ color: mark.ink }}>{mark.label}</span>
                    </>
                  )}
                </span>
              </button>
            </li>
          );
        })}
      </ul>
    </nav>
  );
}
