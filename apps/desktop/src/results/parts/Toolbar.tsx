/**
 * Search, the filter chips, and the order.
 *
 * Every chip carries the count it would leave behind, taken from the rows that
 * are already loaded. A chip that shows a number cannot lie about how many rows
 * are behind it, and one showing zero tells a person not to bother pressing it —
 * which is the whole reason the count is on the chip rather than discovered
 * after the click.
 *
 * All of it is client-side. The rows were fetched to draw the summary, so asking
 * the daemon to hide some of them would be a round trip that can only return
 * what is already here.
 */
import { Search, X } from 'lucide-react';

import { Button } from '../../components/ui/button.js';
import { type Filters, type SortKey, SORT_LABELS, type Tallies } from '../model.js';

export interface ToolbarProps {
  readonly filters: Filters;
  readonly sort: SortKey;
  readonly tallies: Tallies;
  readonly shown: number;
  readonly onFilters: (next: Filters) => void;
  readonly onSort: (next: SortKey) => void;
}

interface Chip {
  readonly id: string;
  readonly label: string;
  readonly count: number;
  readonly active: boolean;
  readonly apply: () => void;
}

export function Toolbar({ filters, sort, tallies, shown, onFilters, onSort }: ToolbarProps) {
  const band = (value: Filters['band']) => () =>
    onFilters({ ...filters, band: value, decision: 'any' });
  const decision = (value: Filters['decision']) => () =>
    onFilters({ ...filters, decision: value, band: 'any' });

  const chips: readonly Chip[] = [
    {
      id: 'all',
      label: 'All',
      count: tallies.all,
      active: filters.band === 'any' && filters.decision === 'any',
      apply: () => onFilters({ ...filters, band: 'any', decision: 'any' }),
    },
    {
      id: 'strong',
      label: 'Strong',
      count: tallies.strong,
      active: filters.band === 'strong',
      apply: band('strong'),
    },
    {
      id: 'promising',
      label: 'Promising',
      count: tallies.promising,
      active: filters.band === 'promising',
      apply: band('promising'),
    },
    {
      id: 'needs_review',
      label: 'Needs review',
      count: tallies.needsReview,
      active: filters.band === 'needs_review',
      apply: band('needs_review'),
    },
    {
      id: 'undecided',
      label: 'Undecided',
      count: tallies.undecided,
      active: filters.decision === 'undecided',
      apply: decision('undecided'),
    },
    {
      id: 'approved',
      label: 'Approved',
      count: tallies.approved,
      active: filters.decision === 'approved',
      apply: decision('approved'),
    },
  ];

  return (
    <div className="flex flex-wrap items-center gap-3" role="search">
      <label className="relative">
        <span className="sr-only">Search clips by their opening line or timecode</span>
        <Search
          aria-hidden
          className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-[var(--cm-text-muted)]"
        />
        <input
          type="search"
          value={filters.query ?? ''}
          onChange={(event) => onFilters({ ...filters, query: event.target.value })}
          placeholder="Search clips…"
          className="glass h-[var(--cm-control-standard)] w-56 rounded-[var(--cm-radius-control)] pr-8 pl-9 text-[13px] text-[var(--cm-text-primary)] transition-colors placeholder:text-[var(--cm-text-muted)] focus:border-[var(--cm-accent)]"
        />
        {(filters.query ?? '') !== '' && (
          <button
            type="button"
            aria-label="Clear the search"
            onClick={() => onFilters({ ...filters, query: '' })}
            className="absolute top-1/2 right-2 -translate-y-1/2 rounded p-0.5 text-[var(--cm-text-muted)] transition-colors hover:text-[var(--cm-text-primary)]"
          >
            <X className="size-3.5" />
          </button>
        )}
      </label>

      <div
        className="glass flex h-[var(--cm-control-standard)] items-center gap-0.5 rounded-[var(--cm-radius-control)] p-1"
        role="group"
        aria-label="Filter the board"
      >
        {chips.map((chip) => (
          <button
            key={chip.id}
            type="button"
            onClick={chip.apply}
            aria-pressed={chip.active}
            disabled={chip.count === 0 && !chip.active}
            className="flex items-center gap-1.5 rounded-[6px] px-2.5 py-1 text-[12px] whitespace-nowrap transition-all duration-150 disabled:opacity-40"
            style={
              chip.active
                ? {
                    background: 'var(--cm-accent-selected)',
                    color: 'var(--cm-text-primary)',
                    fontWeight: 'var(--cm-weight-label)',
                  }
                : { color: 'var(--cm-text-secondary)' }
            }
          >
            {chip.label}
            <span className="mono text-[10px] opacity-70">{chip.count}</span>
          </button>
        ))}
      </div>

      <div className="ml-auto flex items-center gap-3">
        <span className="mono text-[11px] text-[var(--cm-text-muted)]" aria-live="polite">
          {shown} of {tallies.all}
        </span>
        <label className="flex items-center gap-2">
          <span className="sr-only">Order the board</span>
          <select
            value={sort}
            onChange={(event) => onSort(event.target.value as SortKey)}
            className="glass h-[var(--cm-control-standard)] rounded-[var(--cm-radius-control)] px-3 text-[12px] text-[var(--cm-text-primary)] transition-colors focus:border-[var(--cm-accent)]"
          >
            {Object.entries(SORT_LABELS).map(([key, label]) => (
              <option key={key} value={key}>
                {label}
              </option>
            ))}
          </select>
        </label>
        {(filters.band !== 'any' || filters.decision !== 'any' || (filters.query ?? '') !== '') && (
          <Button
            size="sm"
            variant="ghost"
            onClick={() => onFilters({ ...filters, band: 'any', decision: 'any', query: '' })}
          >
            Reset
          </Button>
        )}
      </div>
    </div>
  );
}
