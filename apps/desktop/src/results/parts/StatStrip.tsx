/**
 * The run, in the five numbers the design puts above the board.
 *
 * Every figure is one the documents state. "Candidates" is the cohort the
 * ranker scored; "Recommended" is its selected set — the one opinion the ranker
 * holds, and the figure the design leads with; "Approved" and "Flagged" are the
 * rows a person decided on and the rows the ranker warned about. The design's
 * strip carries exactly these labels, and every one of them has a real number
 * behind it here.
 *
 * The shortfall is why this strip is not decoration. "Four asked for, one
 * recommended" is a claim a person can act on, and the sentence under it says
 * why the other three are missing instead of leaving them to assume a bug.
 */
import { AlertTriangle } from 'lucide-react';

import type { Summary, Tallies } from '../model.js';

export interface StatStripProps {
  readonly summary: Summary;
  readonly tallies: Tallies;
  readonly bestScore: number | null;
}

interface Stat {
  readonly label: string;
  readonly value: number;
  readonly ink: string;
  readonly title: string;
}

export function StatStrip({ summary, tallies, bestScore }: StatStripProps) {
  const stats: readonly Stat[] = [
    {
      label: 'Candidates',
      value: summary.cohort,
      ink: 'var(--cm-text-primary)',
      title: 'Every candidate the ranker scored.',
    },
    {
      label: 'Recommended',
      value: tallies.recommended,
      ink: tallies.recommended > 0 ? 'var(--cm-success-ink)' : 'var(--cm-text-muted)',
      title: `The ranker's selected set — ${summary.requested} were asked for.`,
    },
    {
      label: 'Approved',
      value: tallies.approved,
      ink: tallies.approved > 0 ? 'var(--cm-accent)' : 'var(--cm-text-muted)',
      title: 'Clips a person approved and sent to the editor.',
    },
    {
      label: 'Flagged',
      value: tallies.flagged,
      ink: tallies.flagged > 0 ? 'var(--cm-danger-ink)' : 'var(--cm-text-muted)',
      title: 'Clips the ranker recorded a warning or penalty against.',
    },
  ];

  return (
    <section
      className="glass flex flex-wrap items-center gap-x-8 gap-y-4 rounded-[var(--cm-radius-card)] px-6 py-4"
      aria-label="Run summary"
    >
      {stats.map((stat, index) => (
        <div key={stat.label} className="flex items-center gap-8">
          {index > 0 && <span aria-hidden className="h-8 w-px bg-[var(--cm-glass-border)]" />}
          <div className="flex flex-col gap-1" title={stat.title}>
            <span className="text-[10px] font-medium tracking-[0.09em] text-[var(--cm-text-muted)] uppercase">
              {stat.label}
            </span>
            <span className="mono text-lg leading-none" style={{ color: stat.ink }}>
              {stat.value}
            </span>
          </div>
        </div>
      ))}

      {bestScore !== null && (
        <div className="ml-auto flex flex-col items-end gap-1 border-l border-[var(--cm-glass-border)] pl-8">
          <span className="text-[10px] font-medium tracking-[0.09em] text-[var(--cm-text-muted)] uppercase">
            Best score
          </span>
          <span className="mono text-xl leading-none text-[var(--cm-text-primary)]">
            {bestScore}
          </span>
        </div>
      )}

      {summary.shortfall.length > 0 && (
        <p className="flex w-full items-start gap-2 border-t border-[var(--cm-glass-border)] pt-3 text-[12px] text-[var(--cm-warning-ink)]">
          <AlertTriangle className="mt-px size-3.5 shrink-0" aria-hidden />
          <span>
            {summary.requested} asked for, {summary.selected} recommended:{' '}
            {summary.shortfall.join('; ')}.
          </span>
        </p>
      )}
    </section>
  );
}
