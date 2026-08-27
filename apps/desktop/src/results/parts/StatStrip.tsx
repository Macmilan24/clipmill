/**
 * The run, in the four numbers that describe it.
 *
 * Every figure here is one the ranking document states. The design's strip shows
 * "recommended" and "flagged" beside them; neither is a thing this system
 * publishes, so the two that exist take their place — the cohort that was scored
 * and the candidates filtered out before scoring — and the shortfall is spelled
 * out rather than shown as a deficit against a target.
 *
 * The shortfall is the reason this strip is not decoration. "Three of four
 * asked for" is a claim a person can act on, and the sentence that follows says
 * why the fourth is missing instead of leaving them to assume a bug.
 */
import { AlertTriangle } from 'lucide-react';

import type { Summary } from '../model.js';
import type { Tallies } from '../model.js';

export interface StatStripProps {
  readonly summary: Summary;
  readonly tallies: Tallies;
  readonly bestScore: number | null;
}

interface Stat {
  readonly label: string;
  readonly value: string;
  readonly ink?: string | undefined;
  readonly title: string;
}

export function StatStrip({ summary, tallies, bestScore }: StatStripProps) {
  const stats: readonly Stat[] = [
    {
      label: 'Selected',
      value: `${summary.selected}`,
      title: 'Clips the ranker chose to show, after diversity selection.',
    },
    {
      label: 'Requested',
      value: `${summary.requested}`,
      title: 'How many the analysis was asked for.',
    },
    {
      label: 'Cohort',
      value: `${summary.cohort}`,
      title: 'Candidates that were scored.',
    },
    {
      label: 'Approved',
      value: `${tallies.approved}`,
      ink: tallies.approved > 0 ? 'var(--cm-success-ink)' : undefined,
      title: 'Clips sent to the editor.',
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
            <span
              className="mono text-lg leading-none"
              style={{ color: stat.ink ?? 'var(--cm-text-primary)' }}
            >
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
            Fewer than requested: {summary.shortfall.join('; ')}.{' '}
            <span className="text-[var(--cm-text-secondary)]">
              A recording holding three good moments returns three.
            </span>
          </span>
        </p>
      )}
    </section>
  );
}
