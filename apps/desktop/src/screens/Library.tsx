import { LayoutGrid, List, Plug, Plus, Search, TriangleAlert } from 'lucide-react';
import { type JSX, useMemo, useState } from 'react';

import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from '@/components/ui/empty';
import { Input } from '@/components/ui/input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Skeleton } from '@/components/ui/skeleton';
import {
  Table,
  TableBody,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { cn } from '@/lib/utils';

import type { ConnectionState } from '../daemon/client.js';
import { ProjectCard } from '../library/ProjectCard.js';
import { ProjectRow } from '../library/ProjectRow.js';
import { StorageStrip } from '../library/StorageStrip.js';
import type { LibraryLoader } from '../library/loader.js';
import {
  type LibraryProject,
  SORTS,
  type SortKey,
  type StatusFilter,
  applyFilter,
  availableFilters,
  matchesQuery,
  sortProjects,
} from '../library/model.js';
import { useLibrary } from '../library/useLibrary.js';

export interface LibraryProps {
  readonly state: ConnectionState;
  readonly onNavigate: (sectionId: string) => void;
  /** Opens one run. Only ever called for a project that has one. */
  readonly onOpenAnalysis: (projectId: string, jobId: string) => void;
  readonly onReconnect: () => void;
  /** Injected by tests, which drive the whole screen through a fake daemon. */
  readonly loader?: LibraryLoader;
}

type View = 'grid' | 'list';

function ViewToggle({
  view,
  onChange,
}: {
  readonly view: View;
  readonly onChange: (view: View) => void;
}): JSX.Element {
  const options: readonly (readonly [View, typeof LayoutGrid, string])[] = [
    ['grid', LayoutGrid, 'Grid view'],
    ['list', List, 'List view'],
  ];

  return (
    <div
      role="group"
      aria-label="View"
      className="flex h-8 items-center gap-0.5 rounded-[var(--cm-radius-control)] border border-[var(--cm-glass-border)] bg-[var(--cm-recessed)] p-0.5"
    >
      {options.map(([value, Icon, label]) => (
        <Button
          key={value}
          variant="ghost"
          size="icon-sm"
          aria-label={label}
          aria-pressed={view === value}
          onClick={() => {
            onChange(value);
          }}
          className={cn(
            'size-7',
            view === value && 'bg-[var(--cm-accent-selected)] text-[var(--cm-text-primary)]',
          )}
        >
          <Icon />
        </Button>
      ))}
    </div>
  );
}

/** The grid the design specifies, and a cheap stand-in while it is loading. */
function LoadingGrid(): JSX.Element {
  return (
    <div className="grid grid-cols-3 gap-4" aria-busy="true" aria-label="Loading projects">
      {[0, 1, 2].map((index) => (
        <Skeleton key={index} className="h-[276px] rounded-2xl" />
      ))}
    </div>
  );
}

export function Library({
  state,
  onNavigate,
  onOpenAnalysis,
  onReconnect,
  loader,
}: LibraryProps): JSX.Element {
  const { loading, projects, storage, error, reload } = useLibrary(state, loader);
  const [query, setQuery] = useState('');
  const [filter, setFilter] = useState<StatusFilter>('all');
  const [sort, setSort] = useState<SortKey>('created');
  const [view, setView] = useState<View>('grid');

  const filters = useMemo(() => availableFilters(projects), [projects]);
  const shown = useMemo(
    () =>
      sortProjects(
        applyFilter(projects, filter).filter((entry) => matchesQuery(entry, query)),
        sort,
      ),
    [projects, filter, query, sort],
  );

  /**
   * Where a card goes.
   *
   * A run that is still working, or one that failed, opens at its own progress
   * screen — that is where the answer to "what is it doing" and "why did it
   * stop" lives. A finished one opens at its results, which is the screen Phase
   * 1 builds; until then that lands on the placeholder naming the phase, which
   * is the truth rather than a card that swallows the click.
   */
  const open = (entry: LibraryProject): void => {
    const watchable =
      entry.job !== null &&
      (entry.status.kind === 'analyzing' ||
        entry.status.kind === 'queued' ||
        entry.status.kind === 'failed');
    if (watchable && entry.job !== null) {
      onOpenAnalysis(entry.project.projectId, entry.job.jobId);
      return;
    }
    onNavigate('results');
  };

  if (state.status !== 'connected') {
    return (
      <Empty className="glass rounded-xl" aria-label="Daemon not connected">
        <EmptyHeader>
          <EmptyMedia variant="icon" className="glass-elevated size-14 rounded-full">
            <Plug className="size-5" />
          </EmptyMedia>
          <EmptyTitle className="text-card-title">Daemon not connected</EmptyTitle>
          <EmptyDescription>
            Projects live in the daemon&apos;s store. Nothing can be listed until it answers.
          </EmptyDescription>
        </EmptyHeader>
        <EmptyContent>
          <Button onClick={onReconnect}>Retry now</Button>
        </EmptyContent>
      </Empty>
    );
  }

  return (
    <>
      <div className="mb-4 flex h-12 items-center justify-between gap-4">
        <div className="flex items-baseline gap-3">
          <h1 className="text-page-title font-(--cm-weight-heading) tracking-[-0.01em]">Library</h1>
          <span className="text-meta text-[var(--cm-text-secondary)]">
            {projects.length === 1 ? '1 project' : `${projects.length} projects`}
          </span>
        </div>

        <div className="flex items-center gap-3">
          <div className="relative w-[280px]">
            <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-[var(--cm-text-muted)]" />
            <Input
              value={query}
              onChange={(event) => {
                setQuery(event.target.value);
              }}
              // The design offers transcripts and speakers here. Neither is
              // searchable from this screen, so the field says what it does.
              placeholder="Search project titles"
              aria-label="Search project titles"
              className="pl-9"
            />
          </div>
          <ViewToggle view={view} onChange={setView} />
          {/* The one primary action on this screen, at the design's 36px. */}
          <Button
            onClick={() => {
              onNavigate('new-project');
            }}
          >
            <Plus />
            Import video
          </Button>
        </div>
      </div>

      {projects.length === 0 ? null : (
        <div className="mb-4 flex items-center justify-between gap-4">
          <div role="group" aria-label="Filter by status" className="flex flex-wrap items-center gap-2">
            {filters.map((entry) => (
              <Button
                key={entry.filter}
                variant="ghost"
                size="sm"
                aria-pressed={filter === entry.filter}
                onClick={() => {
                  setFilter(entry.filter);
                }}
                className={cn(
                  'h-7 rounded-[var(--cm-radius-control)] border border-[var(--cm-glass-border)] text-meta',
                  filter === entry.filter &&
                    'border-[color-mix(in_srgb,var(--color-primary)_45%,transparent)] bg-[var(--cm-accent-selected)] text-[var(--color-primary)]',
                )}
              >
                {entry.label}
                <Badge variant="outline" className="mono border-0 px-0 text-technical opacity-70">
                  {entry.count}
                </Badge>
              </Button>
            ))}
          </div>

          <div className="flex items-center gap-2">
            <span className="text-meta text-[var(--cm-text-muted)]">Sort</span>
            <Select
              value={sort}
              onValueChange={(value) => {
                setSort(value as SortKey);
              }}
            >
              <SelectTrigger className="h-7 w-[168px]" aria-label="Sort projects">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {SORTS.map((option) => (
                  <SelectItem key={option.key} value={option.key}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>
      )}

      {error === null ? null : (
        <Alert className="glass mb-4 rounded-xl">
          <TriangleAlert className="text-[var(--color-warning)]" />
          <AlertDescription className="flex items-center justify-between gap-4">
            {error}
            <Button variant="outline" size="sm" onClick={reload}>
              Try again
            </Button>
          </AlertDescription>
        </Alert>
      )}

      {loading && projects.length === 0 ? (
        <LoadingGrid />
      ) : projects.length === 0 ? (
        <Empty className="glass rounded-xl" aria-label="No projects yet">
          <EmptyHeader>
            <EmptyMedia variant="icon" className="glass-elevated size-14 rounded-full">
              <Plus className="size-5" />
            </EmptyMedia>
            <EmptyTitle className="text-card-title">No projects yet</EmptyTitle>
            <EmptyDescription>
              Import a long-form recording and ClipMill will analyse it on this machine.
            </EmptyDescription>
          </EmptyHeader>
          <EmptyContent>
            <Button
              onClick={() => {
                onNavigate('new-project');
              }}
            >
              <Plus />
              Import video
            </Button>
          </EmptyContent>
        </Empty>
      ) : shown.length === 0 ? (
        <Empty className="glass rounded-xl" aria-label="Nothing matches">
          <EmptyHeader>
            <EmptyTitle className="text-card-title">Nothing matches</EmptyTitle>
            <EmptyDescription>
              {projects.length === 1 ? '1 project' : `${projects.length} projects`} are here, but
              none match this search and filter.
            </EmptyDescription>
          </EmptyHeader>
        </Empty>
      ) : view === 'grid' ? (
        <div className="grid grid-cols-3 gap-4">
          {shown.map((entry) => (
            <ProjectCard key={entry.project.projectId} entry={entry} onOpen={open} />
          ))}
        </div>
      ) : (
        <div className="glass rounded-xl px-3 py-1">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Project</TableHead>
                <TableHead>Duration</TableHead>
                <TableHead>Video</TableHead>
                <TableHead>Size</TableHead>
                <TableHead>Created</TableHead>
                <TableHead className="text-right">Status</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {shown.map((entry) => (
                <ProjectRow key={entry.project.projectId} entry={entry} onOpen={open} />
              ))}
            </TableBody>
          </Table>
        </div>
      )}

      <StorageStrip
        stats={storage}
        onManage={() => {
          onNavigate('settings');
        }}
      />
    </>
  );
}
