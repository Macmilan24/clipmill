/**
 * The Results board: every clip the ranking believes in, and what it believed.
 *
 * The board shows counts rather than adjectives. "Four asked for, one
 * recommended" is a sentence a person can act on; "great results!" is not, and
 * the shortfall reasons are shown rather than padded away — a recording that
 * holds three good moments should return three and say so, because the fourth
 * would be a clip the system does not believe in.
 *
 * Three acts on a row, kept apart on purpose. Focusing a row moves the detail
 * rail so an editor can compare without losing their place. Ticking it adds it
 * to the set the footer and the header act on. Opening it is a third, deliberate
 * step into the inspector. A click that did all three would make looking at a
 * clip the same gesture as committing to it.
 *
 * Filtering, search and ordering are client-side because the answer is already
 * here. Every row was fetched to draw the summary, so asking the daemon again to
 * hide some of them would be a round trip that can only produce what is already
 * on screen.
 */
import { JobState } from '@clipmill/contracts';
import { AlertCircle, ArrowRight } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';

import { Button } from '../components/ui/button.js';
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from '../components/ui/empty.js';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../components/ui/select.js';
import { Skeleton } from '../components/ui/skeleton.js';
import type { Project } from '../daemon/client.js';
import type { RunInfo } from '../results/loader.js';
import { CandidateGrid } from '../results/parts/CandidateGrid.js';
import { CandidateTable } from '../results/parts/CandidateTable.js';
import { DetailRail } from '../results/parts/DetailRail.js';
import { StatStrip } from '../results/parts/StatStrip.js';
import { type BoardView, Toolbar } from '../results/parts/Toolbar.js';
import { TONE_INK, type Tone, wash } from '../results/parts/state.js';
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
  /** The analysis that produced them, by the facts the job records. */
  readonly run: RunInfo | null;
  readonly tileUrl: (atTicks: number) => string | null;
  /** Every project, so this screen can reach a recording it was not routed to. */
  readonly projects: readonly Project[];
  readonly activeProjectId: string | null;
  readonly busy: boolean;
  readonly onChooseProject: (projectId: string) => void;
  readonly onInspect: (candidateId: string) => void;
  /** Open a clip that already has an edit document in the editor. */
  readonly onEdit: (candidateId: string) => void;
  readonly onApproveMany: (candidateIds: readonly string[]) => void;
  readonly onReload: () => void;
  readonly notice?: string | null;
}

/** The job's state as the design's badge, from the job rather than assumed. */
function runBadge(run: RunInfo | null): { label: string; tone: Tone } | null {
  if (!run) {
    return null;
  }
  switch (run.state) {
    case JobState.SUCCEEDED:
      return { label: 'Analyzed', tone: 'success' };
    case JobState.FAILED:
      return { label: 'Failed', tone: 'danger' };
    case JobState.CANCELLED:
      return { label: 'Cancelled', tone: 'muted' };
    default:
      return { label: 'Analyzing', tone: 'accent' };
  }
}

function completedAt(millis: number): string {
  return new Date(millis).toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' });
}

export function Results({
  loading,
  rows,
  summary,
  problem,
  sourceName,
  run,
  tileUrl,
  projects,
  activeProjectId,
  busy,
  onChooseProject,
  onInspect,
  onEdit,
  onApproveMany,
  onReload,
  notice,
}: ResultsProps) {
  const [filters, setFilters] = useState<Filters>({ ...NO_FILTERS, query: '' });
  const [sort, setSort] = useState<SortKey>('rank');
  const [view, setView] = useState<BoardView>('list');
  const [focusedId, setFocusedId] = useState<string | null>(null);
  const [checked, setChecked] = useState<ReadonlySet<string>>(new Set());

  // A new set of rows is a new board; a tick made against the old one would be
  // a tick against a clip that may no longer be on screen.
  useEffect(() => {
    setChecked(new Set());
  }, [rows]);

  const tallies = useMemo(() => tally(rows), [rows]);
  const shown = useMemo(() => sortRows(applyFilters(rows, filters), sort), [rows, filters, sort]);

  const focused = useMemo(
    () => shown.find((row) => row.candidateId === focusedId) ?? shown[0] ?? null,
    [shown, focusedId],
  );

  const toggle = (candidateId: string) =>
    setChecked((current) => {
      const next = new Set(current);
      if (next.has(candidateId)) {
        next.delete(candidateId);
      } else {
        next.add(candidateId);
      }
      return next;
    });

  const checkedRows = useMemo(
    () => rows.filter((row) => checked.has(row.candidateId)),
    [rows, checked],
  );
  const incomplete = (summary?.warnings?.length ?? 0) > 0;
  const badge =
    incomplete && run?.state === JobState.SUCCEEDED
      ? { label: 'Partial results', tone: 'warning' as const }
      : runBadge(run);

  if (loading) {
    return (
      <div className="workspace-page" role="status" aria-busy="true">
        <p className="text-xs text-[var(--cm-text-secondary)]">Loading results…</p>
        <Skeleton className="h-[92px] w-full rounded-[var(--cm-radius-card)]" />
        <Skeleton className="h-[var(--cm-control-standard)] w-full rounded-[var(--cm-radius-control)]" />
        <Skeleton className="h-[420px] w-full rounded-[var(--cm-radius-card)]" />
      </div>
    );
  }

  return (
    <div className="workspace-page">
      <header className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex min-w-0 flex-col gap-1">
          <div className="flex items-center gap-3">
            <h1 className="text-[length:var(--cm-type-page-title)] font-semibold tracking-tight text-[var(--cm-text-primary)]">
              {problem
                ? 'Results'
                : `${rows.length} clip ${rows.length === 1 ? 'candidate' : 'candidates'}`}
            </h1>
            {badge && !problem && (
              <span
                className="rounded px-2 py-0.5 text-[10px] font-bold tracking-wider uppercase"
                style={{ color: TONE_INK[badge.tone], background: wash(badge.tone) }}
              >
                {badge.label}
              </span>
            )}
          </div>
          <p className="flex flex-wrap items-center gap-x-2 text-[13px] text-[var(--cm-text-secondary)]">
            {sourceName && <span className="truncate">{sourceName}</span>}
            {run && (
              <>
                <span aria-hidden>·</span>
                <span className="mono text-[11px] text-[var(--cm-text-muted)]">
                  {run.state === JobState.SUCCEEDED
                    ? `Completed ${completedAt(run.completedUnixMillis)}`
                    : badge?.label}
                  <span className="sr-only">{run.jobId.replace(/^job_/, 'run_').slice(0, 12)}</span>
                </span>
              </>
            )}
          </p>
        </div>

        <div className="flex items-center gap-2">
          {(projects.length > 1 || (projects.length > 0 && !activeProjectId)) && (
            <Select value={activeProjectId ?? ''} onValueChange={onChooseProject} disabled={busy}>
              <SelectTrigger
                aria-label="Which project's results to show"
                className="glass h-[var(--cm-control-primary)] w-[220px] text-[12px]"
              >
                <SelectValue placeholder="Choose a project" />
              </SelectTrigger>
              <SelectContent>
                {projects.map((project) => (
                  <SelectItem key={project.projectId} value={project.projectId}>
                    {project.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          )}
          {!problem && (
            <Button
              className="h-[var(--cm-control-primary)] gap-2"
              disabled={busy || focused === null}
              onClick={() => {
                const first = checkedRows[0] ?? focused;
                if (first) {
                  onInspect(first.candidateId);
                }
              }}
              title={
                checkedRows.length === 0
                  ? 'Open the selected clip for review'
                  : `Open the first of ${checkedRows.length} selected in the inspector`
              }
            >
              {checkedRows.length ? 'Review selected' : 'Review clips'}
              <ArrowRight className="size-4" aria-hidden />
            </Button>
          )}
        </div>
      </header>

      {notice && (
        <p
          role="status"
          className="workspace-panel px-4 py-3 text-xs text-[var(--cm-text-secondary)]"
        >
          {notice}
        </p>
      )}
      {problem && (
        <Empty className="glass rounded-[var(--cm-radius-card)]">
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
                : 'Results appear once an analysis finishes and publishes a ranked set. Another recording can be chosen above.'}
            </EmptyDescription>
          </EmptyHeader>
          <Button variant="outline" size="sm" onClick={onReload}>
            Try again
          </Button>
        </Empty>
      )}

      {!problem && summary && (
        <StatStrip
          summary={summary}
          tallies={tallies}
          bestScore={
            rows.length > 0 && !rows.some((row) => row.review)
              ? Math.max(...rows.map((row) => row.displayScore))
              : null
          }
        />
      )}

      {!problem && (
        <>
          <Toolbar
            editorial={rows.some((row) => row.review !== undefined)}
            filters={filters}
            sort={sort}
            view={view}
            tallies={tallies}
            shown={shown.length}
            onFilters={setFilters}
            onSort={setSort}
            onView={setView}
          />

          <div className="results-layout">
            {shown.length === 0 ? (
              <div className="glass grid place-items-center rounded-[var(--cm-radius-card)] p-10">
                <p className="text-[13px] text-[var(--cm-text-secondary)]">
                  {rows.length === 0
                    ? incomplete
                      ? 'No clips are ready from this partial analysis. Unassessed sections may still contain worthwhile moments.'
                      : 'No suitable complete moments were found in this recording.'
                    : 'No clip matches those filters.'}
                </p>
              </div>
            ) : view === 'list' ? (
              <CandidateTable
                rows={shown}
                focusedId={focused?.candidateId ?? null}
                checked={checked}
                onFocus={setFocusedId}
                onToggle={toggle}
                onOpen={onInspect}
              />
            ) : (
              <CandidateGrid
                rows={shown}
                focusedId={focused?.candidateId ?? null}
                checked={checked}
                tileUrl={tileUrl}
                onFocus={setFocusedId}
                onToggle={toggle}
                onOpen={onInspect}
              />
            )}
            <DetailRail
              row={focused}
              tileUrl={tileUrl}
              approvedCount={tallies.approved}
              checkedCount={checkedRows.length}
              busy={busy}
              onApproveChecked={() => onApproveMany(checkedRows.map((row) => row.candidateId))}
              onOpen={onInspect}
              onEdit={onEdit}
            />
          </div>
        </>
      )}
    </div>
  );
}
