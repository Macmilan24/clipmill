/**
 * The Results board: every clip the ranking believes in, and what it believed.
 *
 * The board shows counts rather than adjectives. "Three of eight, four asked
 * for" is a sentence a person can act on; "great results!" is not, and the
 * shortfall reasons are shown rather than padded away — a recording that holds
 * three good moments should return three and say so, because the fourth would
 * be a clip the system does not believe in.
 *
 * Filtering is client-side because the answer is already here. Every row was
 * fetched to draw the summary, so asking the daemon again to hide some of them
 * would be a round trip that can only produce what is already on screen.
 */
import { AlertCircle, ArrowRight, Filter } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';

import { Badge } from '../components/ui/badge.js';
import { Button } from '../components/ui/button.js';
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from '../components/ui/empty.js';
import { Skeleton } from '../components/ui/skeleton.js';
import type { ClipDecision } from '../daemon/client.js';
import {
  type ClipRow,
  type Filters,
  NO_FILTERS,
  type Summary,
  applyFilters,
  clock,
} from '../results/model.js';

export interface ResultsProps {
  readonly loading: boolean;
  readonly rows: readonly ClipRow[];
  readonly summary: Summary | null;
  /** Why there is nothing, when there is nothing. */
  readonly problem: { readonly kind: string; readonly detail?: string } | null;
  readonly onInspect: (candidateId: string) => void;
  readonly onReload: () => void;
}

/** The score, drawn as a ring rather than a number in a box. */
function ScoreRing({ score, band }: { readonly score: number; readonly band: string }) {
  const filled = Math.max(0, Math.min(100, score));
  const tone =
    band === 'strong'
      ? 'var(--cm-success-ink)'
      : band === 'needs_review'
        ? 'var(--cm-warning-ink)'
        : 'var(--cm-accent)';
  return (
    <div
      className="relative grid size-14 shrink-0 place-items-center rounded-full"
      style={{
        background: `conic-gradient(${tone} ${filled * 3.6}deg, var(--cm-surface-2) 0deg)`,
      }}
      role="img"
      aria-label={`Score ${score} of 99`}
    >
      <span className="grid size-11 place-items-center rounded-full bg-[var(--cm-surface-1)] text-sm font-semibold text-[var(--cm-ink-1)]">
        {score}
      </span>
    </div>
  );
}

const DECISION_LABELS: Readonly<Record<ClipDecision, string>> = {
  approved: 'Approved',
  kept: 'Kept',
  rejected: 'Rejected',
};

export function Results({ loading, rows, summary, problem, onInspect, onReload }: ResultsProps) {
  const [filters, setFilters] = useState<Filters>(NO_FILTERS);
  const shown = useMemo(() => applyFilters(rows, filters), [rows, filters]);

  useEffect(() => {
    onReload();
  }, [onReload]);

  if (loading) {
    return (
      <div className="flex flex-col gap-3 p-8">
        <Skeleton className="h-20 w-full" />
        <Skeleton className="h-24 w-full" />
        <Skeleton className="h-24 w-full" />
      </div>
    );
  }

  if (problem) {
    return (
      <div className="p-8">
        <Empty>
          <EmptyHeader>
            <AlertCircle className="size-6 text-[var(--cm-ink-3)]" />
            <EmptyTitle>
              {problem.kind === 'no-source'
                ? 'No recording in this project yet'
                : problem.kind === 'not-analyzed'
                  ? 'This recording has not been analyzed'
                  : 'The published ranking could not be read'}
            </EmptyTitle>
            <EmptyDescription>
              {problem.kind === 'unreadable'
                ? problem.detail
                : 'Results appear once an analysis finishes and publishes a ranked set.'}
            </EmptyDescription>
          </EmptyHeader>
        </Empty>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-6 p-8">
      {summary && (
        <section
          className="flex flex-wrap items-center gap-x-8 gap-y-3 rounded-xl border border-[var(--cm-line-1)] bg-[var(--cm-surface-1)] px-5 py-4"
          aria-label="Summary"
        >
          <div>
            <p className="text-2xl font-semibold text-[var(--cm-ink-1)]">
              {summary.selected}
              <span className="text-[var(--cm-ink-3)]"> / {summary.requested}</span>
            </p>
            <p className="text-xs text-[var(--cm-ink-2)]">selected, of the number asked for</p>
          </div>
          <div>
            <p className="text-2xl font-semibold text-[var(--cm-ink-1)]">{summary.cohort}</p>
            <p className="text-xs text-[var(--cm-ink-2)]">scored in the cohort</p>
          </div>
          {summary.filtered > 0 && (
            <div>
              <p className="text-2xl font-semibold text-[var(--cm-ink-1)]">{summary.filtered}</p>
              <p className="text-xs text-[var(--cm-ink-2)]">removed before scoring</p>
            </div>
          )}
          {summary.shortfall.length > 0 && (
            <p className="max-w-md text-xs text-[var(--cm-warning-ink)]">
              Fewer than requested: {summary.shortfall.join('; ')}.
            </p>
          )}
        </section>
      )}

      <section className="flex flex-wrap items-center gap-2" aria-label="Filters">
        <Filter className="size-4 text-[var(--cm-ink-3)]" />
        {(['any', 'strong', 'promising', 'needs_review'] as const).map((band) => (
          <Button
            key={band}
            size="sm"
            variant={filters.band === band ? 'default' : 'outline'}
            onClick={() => setFilters((current) => ({ ...current, band }))}
          >
            {band === 'any' ? 'Any confidence' : band.replaceAll('_', ' ')}
          </Button>
        ))}
        <span className="mx-2 h-5 w-px bg-[var(--cm-line-1)]" />
        {(['any', 'undecided', 'approved', 'kept', 'rejected'] as const).map((decision) => (
          <Button
            key={decision}
            size="sm"
            variant={filters.decision === decision ? 'default' : 'outline'}
            onClick={() => setFilters((current) => ({ ...current, decision }))}
          >
            {decision === 'any' ? 'Any decision' : decision}
          </Button>
        ))}
        <span className="ml-auto text-xs text-[var(--cm-ink-2)]">
          {shown.length} of {rows.length} shown
        </span>
      </section>

      <ul className="flex flex-col gap-3">
        {shown.map((row) => (
          <li key={row.candidateId}>
            <button
              type="button"
              onClick={() => onInspect(row.candidateId)}
              className="flex w-full items-center gap-5 rounded-xl border border-[var(--cm-line-1)] bg-[var(--cm-surface-1)] px-5 py-4 text-left transition-colors hover:border-[var(--cm-accent)]"
            >
              <span className="w-6 shrink-0 text-sm text-[var(--cm-ink-3)]">{row.rank}</span>
              <ScoreRing score={row.displayScore} band={row.band} />
              <span className="min-w-0 flex-1">
                <span className="flex items-center gap-2">
                  <Badge variant="outline">{row.bandLabel}</Badge>
                  {row.decision && (
                    <Badge variant="secondary">{DECISION_LABELS[row.decision]}</Badge>
                  )}
                  <span className="text-xs text-[var(--cm-ink-2)]">
                    {clock(row.startTicks)}–{clock(row.endTicks)} · {row.durationSeconds.toFixed(1)}
                    s
                  </span>
                </span>
                <span className="mt-1 block truncate text-[15px] text-[var(--cm-ink-1)]">
                  {row.headline || (
                    <em className="text-[var(--cm-ink-3)]">No opening line indexed</em>
                  )}
                </span>
                {row.warnings.length > 0 && (
                  <span className="mt-1 block truncate text-xs text-[var(--cm-warning-ink)]">
                    {row.warnings.join(' · ')}
                  </span>
                )}
              </span>
              <ArrowRight className="size-4 shrink-0 text-[var(--cm-ink-3)]" />
            </button>
          </li>
        ))}
      </ul>

      {shown.length === 0 && rows.length > 0 && (
        <p className="text-sm text-[var(--cm-ink-2)]">No clip matches those filters.</p>
      )}
    </div>
  );
}
