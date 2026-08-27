/**
 * The Results board: every clip the ranking believes in, and what it believed.
 *
 * The board shows counts rather than adjectives. "Three of eight, four asked
 * for" is a sentence a person can act on; "great results!" is not, and the
 * shortfall reasons are shown rather than padded away — a recording that holds
 * three good moments should return three and say so, because the fourth would
 * be a clip the system does not believe in.
 *
 * Two panes: the candidates, and whichever one is selected. Selecting is not the
 * same act as opening — a row click moves the rail so an editor can compare
 * without losing their place, and opening the inspector is a second, deliberate
 * step. That is why the rail carries its own button rather than the click doing
 * both.
 *
 * Filtering, search and ordering are all client-side because the answer is
 * already here. Every row was fetched to draw the summary, so asking the daemon
 * again to hide some of them would be a round trip that can only produce what is
 * already on screen.
 */
import { AlertCircle } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';

import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from '../components/ui/empty.js';
import { Skeleton } from '../components/ui/skeleton.js';
import { CandidateTable } from '../results/parts/CandidateTable.js';
import { DetailRail } from '../results/parts/DetailRail.js';
import { StatStrip } from '../results/parts/StatStrip.js';
import { Toolbar } from '../results/parts/Toolbar.js';
import {
  type ClipRow,
  type Filters,
  NO_FILTERS,
  type SortKey,
  type Summary,
  applyFilters,
  sortRows,
  tally,
} from '../results/model.js';

export interface ResultsProps {
  readonly loading: boolean;
  readonly rows: readonly ClipRow[];
  readonly summary: Summary | null;
  /** Why there is nothing, when there is nothing. */
  readonly problem: { readonly kind: string; readonly detail?: string } | null;
  /** The recording these clips came out of, named rather than implied. */
  readonly sourceName: string | null;
  readonly proxyUrl: string | null;
  readonly onInspect: (candidateId: string) => void;
  readonly onReload: () => void;
}

export function Results({
  loading,
  rows,
  summary,
  problem,
  sourceName,
  proxyUrl,
  onInspect,
  onReload,
}: ResultsProps) {
  const [filters, setFilters] = useState<Filters>({ ...NO_FILTERS, query: '' });
  const [sort, setSort] = useState<SortKey>('rank');
  const [selectedId, setSelectedId] = useState<string | null>(null);

  useEffect(() => {
    onReload();
  }, [onReload]);

  const tallies = useMemo(() => tally(rows), [rows]);
  const shown = useMemo(() => sortRows(applyFilters(rows, filters), sort), [rows, filters, sort]);

  // The rail follows the list. A selection that has been filtered away is a
  // rail describing a row nobody can see, so it falls back to the first row
  // still standing rather than holding on to a ghost.
  const selected = useMemo(
    () => shown.find((row) => row.candidateId === selectedId) ?? shown[0] ?? null,
    [shown, selectedId],
  );

  if (loading) {
    return (
      <div className="flex flex-col gap-4 p-6">
        <Skeleton className="h-[92px] w-full rounded-[var(--cm-radius-card)]" />
        <Skeleton className="h-[var(--cm-control-standard)] w-full rounded-[var(--cm-radius-control)]" />
        <Skeleton className="h-[420px] w-full rounded-[var(--cm-radius-card)]" />
      </div>
    );
  }

  if (problem) {
    return (
      <div className="p-8">
        <Empty>
          <EmptyHeader>
            <AlertCircle className="size-6 text-[var(--cm-text-muted)]" />
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
    <div className="flex min-h-0 flex-1 flex-col gap-4 p-6">
      <header className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex flex-col gap-1">
          <div className="flex items-center gap-3">
            <h1 className="text-[length:var(--cm-type-page-title)] font-semibold tracking-tight text-[var(--cm-text-primary)]">
              {rows.length} clip {rows.length === 1 ? 'candidate' : 'candidates'}
            </h1>
            <span
              className="rounded px-2 py-0.5 text-[10px] font-bold tracking-wider uppercase"
              style={{
                color: 'var(--cm-success-ink)',
                background: 'color-mix(in srgb, var(--cm-success-ink) 12%, transparent)',
              }}
            >
              Analyzed
            </span>
          </div>
          {sourceName && (
            <p className="text-[13px] text-[var(--cm-text-secondary)]">{sourceName}</p>
          )}
        </div>
      </header>

      {summary && (
        <StatStrip
          summary={summary}
          tallies={tallies}
          bestScore={rows.length > 0 ? Math.max(...rows.map((row) => row.displayScore)) : null}
        />
      )}

      <Toolbar
        filters={filters}
        sort={sort}
        tallies={tallies}
        shown={shown.length}
        onFilters={setFilters}
        onSort={setSort}
      />

      <div className="grid min-h-0 flex-1 grid-cols-1 gap-4 xl:grid-cols-[minmax(0,1fr)_336px]">
        {shown.length > 0 ? (
          <CandidateTable
            rows={shown}
            selectedId={selected?.candidateId ?? null}
            onSelect={setSelectedId}
            onOpen={onInspect}
          />
        ) : (
          <div className="glass grid place-items-center rounded-[var(--cm-radius-card)] p-10">
            <p className="text-[13px] text-[var(--cm-text-secondary)]">
              No clip matches those filters.
            </p>
          </div>
        )}
        <DetailRail
          row={selected}
          proxyUrl={proxyUrl}
          approvedCount={tallies.approved}
          onOpen={onInspect}
        />
      </div>
    </div>
  );
}
