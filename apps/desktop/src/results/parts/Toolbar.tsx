/**
 * Search, the filter chips, the order, and how the rows are laid out.
 *
 * The chips are the design's four — All, Recommended, Approved, Flagged — plus
 * Needs review, because that band is a real state the ranker assigns and an
 * editor triaging a board wants it in reach. Every chip carries the count it
 * would leave behind, from rows already loaded, and disables itself at zero so
 * a filter cannot advertise a result nobody gets.
 *
 * The list/grid switch is real: the grid draws a card per clip with its still,
 * which is a different way of comparing than a table row and is what the design
 * offers the toggle for.
 */
import { LayoutGrid, List, Search, X } from 'lucide-react';

import { Button } from '../../components/ui/button.js';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../../components/ui/select.js';
import { type Filters, SORT_LABELS, type SortKey, type Tallies } from '../model.js';

export type BoardView = 'list' | 'grid';

export interface ToolbarProps {
  readonly filters: Filters;
  readonly sort: SortKey;
  readonly view: BoardView;
  readonly tallies: Tallies;
  readonly shown: number;
  readonly onFilters: (next: Filters) => void;
  readonly onSort: (next: SortKey) => void;
  readonly onView: (next: BoardView) => void;
}

interface Chip {
  readonly id: string;
  readonly label: string;
  readonly count: number;
  readonly active: boolean;
  readonly apply: () => void;
}

export function Toolbar({
  filters,
  sort,
  view,
  tallies,
  shown,
  onFilters,
  onSort,
  onView,
}: ToolbarProps) {
  const state = (decision: Filters['decision']) => () =>
    onFilters({ ...filters, decision, band: 'any' });

  const chips: readonly Chip[] = [
    {
      id: 'all',
      label: 'All',
      count: tallies.all,
      active: filters.band === 'any' && filters.decision === 'any',
      apply: () => onFilters({ ...filters, band: 'any', decision: 'any' }),
    },
    {
      id: 'recommended',
      label: 'Recommended',
      count: tallies.recommended,
      active: filters.decision === 'recommended',
      apply: state('recommended'),
    },
    {
      id: 'approved',
      label: 'Approved',
      count: tallies.approved,
      active: filters.decision === 'approved',
      apply: state('approved'),
    },
    {
      id: 'flagged',
      label: 'Flagged',
      count: tallies.flagged,
      active: filters.decision === 'flagged',
      apply: state('flagged'),
    },
    {
      id: 'needs_review',
      label: 'Needs review',
      count: tallies.needsReview,
      active: filters.band === 'needs_review',
      apply: () => onFilters({ ...filters, band: 'needs_review', decision: 'any' }),
    },
  ];

  const filtered =
    filters.band !== 'any' || filters.decision !== 'any' || (filters.query ?? '') !== '';

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
          className="glass h-[var(--cm-control-standard)] w-52 rounded-[var(--cm-radius-control)] pr-8 pl-9 text-[13px] text-[var(--cm-text-primary)] transition-colors placeholder:text-[var(--cm-text-muted)] focus:border-[var(--cm-accent)]"
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

      <div className="ml-auto flex items-center gap-2">
        <span className="mono mr-1 text-[11px] text-[var(--cm-text-muted)]" aria-live="polite">
          {shown} of {tallies.all}
        </span>

        <Select value={sort} onValueChange={(next) => onSort(next as SortKey)}>
          <SelectTrigger
            aria-label="Order the board"
            className="glass h-[var(--cm-control-standard)] w-[190px] text-[12px]"
          >
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {Object.entries(SORT_LABELS).map(([key, label]) => (
              <SelectItem key={key} value={key}>
                {label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>

        <div
          className="glass flex h-[var(--cm-control-standard)] items-center gap-0.5 rounded-[var(--cm-radius-control)] p-1"
          role="group"
          aria-label="Layout"
        >
          {(
            [
              ['list', List, 'List'],
              ['grid', LayoutGrid, 'Grid'],
            ] as const
          ).map(([id, Icon, label]) => (
            <button
              key={id}
              type="button"
              aria-label={`${label} view`}
              aria-pressed={view === id}
              onClick={() => onView(id)}
              className="grid size-6 place-items-center rounded-[6px] transition-colors"
              style={
                view === id
                  ? { background: 'var(--cm-accent-selected)', color: 'var(--cm-text-primary)' }
                  : { color: 'var(--cm-text-muted)' }
              }
            >
              <Icon className="size-3.5" aria-hidden />
            </button>
          ))}
        </div>

        {filtered && (
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
