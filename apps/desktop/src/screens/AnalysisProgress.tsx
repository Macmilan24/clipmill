import {
  ArrowLeft,
  Check,
  ChevronDown,
  CircleAlert,
  CircleDashed,
  CircleSlash,
  Cpu,
  Folder,
  Minus,
  ShieldCheck,
} from 'lucide-react';
import { type JSX, useEffect, useMemo, useState } from 'react';

import type { DeviceProfile } from '@clipmill/contracts';

import { StatusBadge } from '@/components/StatusBadge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Progress as ProgressBar } from '@/components/ui/progress';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Separator } from '@/components/ui/separator';
import { Skeleton } from '@/components/ui/skeleton';
import { Spinner } from '@/components/ui/spinner';
import { cn } from '@/lib/utils';

import {
  type StageRow,
  type StageState,
  currentStage,
  describeStageRow,
  formatElapsed,
  shortenPath,
  stageCounts,
  stageRows,
} from '../analysis/model.js';
import { useReadiness, waitingReasons } from '../analysis/readiness.js';
import { AnalysisLoader, useAnalysis } from '../analysis/useAnalysis.js';
import type { Job, TaskEvent } from '../daemon/client.js';
import {
  acceleratorMemory,
  describeAccelerator,
  formatBytes,
  primaryAccelerator,
} from '../deviceProfile.js';
import {
  EM_DASH,
  describeStatus,
  formatDuration,
  formatVideoSpec,
  readStatus,
} from '../library/model.js';
import './import-progress.css';

import { stageFor } from '../pipeline/stages.js';

export interface AnalysisProgressProps {
  readonly projectId: string;
  readonly jobId: string;
  readonly profile: DeviceProfile | null;
  readonly onBack: () => void;
  readonly onRestarted?: (projectId: string, jobId: string) => void;
  readonly onNavigate: (sectionId: string) => void;
  /** Injected by tests, which drive the screen through a fake daemon. */
  readonly loader?: AnalysisLoader;
}

const MUTED = 'text-[var(--cm-text-muted)]';
const SECONDARY = 'text-[var(--cm-text-secondary)]';

const STATE_STYLE: Readonly<
  Record<StageState, { readonly icon: JSX.Element; readonly text: string }>
> = {
  done: {
    icon: <Check className="size-4 text-[var(--color-success)]" />,
    text: 'text-[var(--cm-text-primary)]',
  },
  running: { icon: <Spinner className="size-4 text-[var(--color-primary)]" />, text: '' },
  waiting: { icon: <CircleDashed className={cn('size-4', MUTED)} />, text: MUTED },
  failed: {
    icon: <CircleAlert className="size-4 text-[var(--color-destructive)]" />,
    text: 'text-[var(--cm-text-primary)]',
  },
  cancelled: { icon: <CircleSlash className={cn('size-4', MUTED)} />, text: MUTED },
  skipped: { icon: <Minus className={cn('size-4', MUTED)} />, text: MUTED },
};

function StageLine({
  row,
  waitingFor,
}: {
  readonly row: StageRow;
  /** Why the stage is waiting, when the daemon's readiness report knows. */
  readonly waitingFor: string | null;
}): JSX.Element {
  const style = STATE_STYLE[row.state];
  const active = row.state === 'running';

  return (
    <div
      role="listitem"
      className={cn(
        'analysis-stage flex items-center gap-3 border-b border-[var(--cm-glass-border)] px-4 py-2.5 last:border-b-0',
        active && 'border-l-2 border-l-[var(--color-primary)] bg-[var(--cm-accent-selected)]',
      )}
    >
      <span className="flex size-6 shrink-0 items-center justify-center">{style.icon}</span>
      <div className="min-w-0 flex-1">
        <div
          className={cn('text-body font-(--cm-weight-label)', style.text)}
          title={row.stage.detail}
        >
          {row.stage.label}
        </div>
        {(active || row.state === 'failed' || row.state === 'skipped') && (
          <div className={cn('mt-0.5 text-meta', SECONDARY)}>
            {row.state === 'skipped' ? 'Not needed for this recording' : row.stage.detail}
          </div>
        )}
        {waitingFor !== null && (
          <div
            className="mt-0.5 text-meta text-[var(--cm-warning-ink)]"
            data-testid={`waiting-${row.stage.kind}`}
          >
            {waitingFor}
          </div>
        )}
      </div>
      <span
        className={cn(
          'mono shrink-0 text-technical',
          active ? 'text-[var(--color-primary)]' : MUTED,
        )}
      >
        {describeStageRow(row)}
      </span>
    </div>
  );
}

function SourceCard({
  name,
  path,
  bytes,
  duration,
  spec,
  thumbnail,
}: {
  readonly name: string;
  readonly path: string | null;
  readonly bytes: number | undefined;
  readonly duration: string;
  readonly spec: string;
  readonly thumbnail: string | null;
}): JSX.Element {
  return (
    <Card className="analysis-source">
      <CardHeader>
        <CardTitle className="text-section-title">Source</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="grid gap-3">
          <div className="analysis-source-preview">
            {thumbnail === null ? null : (
              <img src={thumbnail} alt="" className="size-full object-cover" />
            )}
          </div>
          <div className="min-w-0">
            <div className="truncate text-body font-(--cm-weight-label)">{name}</div>
            <div className={cn('mono mt-0.5 text-technical', SECONDARY)}>
              {duration} · {spec}
            </div>
          </div>
        </div>

        <Separator className="my-3 bg-[var(--cm-glass-border)]" />

        <div className="flex items-center justify-between">
          <span className={cn('text-meta', SECONDARY)}>File size</span>
          <span className="mono text-technical">{formatBytes(bytes)}</span>
        </div>
        <div className="mt-2 flex items-center gap-1.5">
          <Folder className={cn('size-3.5 shrink-0', MUTED)} />
          <span className={cn('mono truncate text-technical', MUTED)} title={path ?? undefined}>
            {path === null ? EM_DASH : shortenPath(path)}
          </span>
        </div>
      </CardContent>
    </Card>
  );
}

/** Last measured device profile, not a live resource monitor. */
function DeviceCard({
  profile,
  cloud,
}: {
  readonly profile: DeviceProfile | null;
  readonly cloud: boolean;
}): JSX.Element {
  const accelerator = profile === null ? undefined : primaryAccelerator(profile);
  const rows: readonly (readonly [string, string])[] =
    profile === null
      ? [['Device', 'not measured']]
      : [
          ['Accelerator', describeAccelerator(accelerator)],
          ['Accelerator memory', acceleratorMemory(profile)],
          ['Memory', formatBytes(profile.memory.total_bytes)],
        ];

  return (
    <Card className="glass rounded-xl">
      <CardHeader>
        <CardTitle className="flex items-center gap-1.5 text-section-title">
          <Cpu className="size-4" /> Device
        </CardTitle>
        <StatusBadge tone="neutral">LOCAL</StatusBadge>
      </CardHeader>
      <CardContent>
        <dl className="grid gap-2">
          {rows.map(([label, value]) => (
            <div key={label} className="flex items-center justify-between gap-2">
              <dt className={cn('text-meta', SECONDARY)}>{label}</dt>
              <dd className="mono truncate text-technical">{value}</dd>
            </div>
          ))}
        </dl>
        <Separator className="my-3 bg-[var(--cm-glass-border)]" />
        <div className="flex items-center gap-1.5">
          <ShieldCheck className="size-3.5 text-[var(--color-success)]" />
          <span className={cn('text-meta', SECONDARY)}>
            {cloud
              ? 'Cloud-assisted · transcript sharing enabled'
              : 'Local processing · no cloud stages in this run'}
          </span>
        </div>
        <p className={cn('mt-2 text-technical', MUTED)}>
          Measured at the last device profile. Nothing here is sampled live.
        </p>
      </CardContent>
    </Card>
  );
}

function clock(atUnixMillis: number): string {
  return new Date(atUnixMillis).toLocaleTimeString(undefined, { hour12: false });
}

/** Task transitions received since this screen opened. */
function LiveLog({
  events,
  job,
}: {
  readonly events: readonly TaskEvent[];
  readonly job: Job | null;
}): JSX.Element {
  const stateWord = (event: TaskEvent): string => {
    const row = stageRows(job).find((candidate) =>
      job?.tasks.some(
        (task) => task.taskId === event.taskId && candidate.stage === stageFor(task.outputKind),
      ),
    );
    const label = row?.stage.label ?? 'stage';
    return `${label}  ${event.waitReason === '' ? `state ${event.state}` : event.waitReason}`;
  };

  return (
    <Card className="glass rounded-xl">
      <CardHeader>
        <CardTitle className="text-section-title">Live log</CardTitle>
        <span className={cn('mono text-technical', MUTED)}>{events.length}</span>
      </CardHeader>
      <CardContent>
        <ScrollArea className="h-[184px] rounded-[8px] bg-[var(--cm-recessed)] p-2">
          {events.length === 0 ? (
            <p className={cn('mono text-technical', MUTED)}>
              Waiting for the next transition. This log starts when the screen opens.
            </p>
          ) : (
            <ul className="grid gap-1">
              {events.map((event) => (
                <li key={event.eventId} className={cn('mono flex gap-2 text-technical', SECONDARY)}>
                  <span className={MUTED}>{clock(event.atUnixMillis)}</span>
                  <span className="min-w-0 break-words">{stateWord(event)}</span>
                </li>
              ))}
            </ul>
          )}
        </ScrollArea>
      </CardContent>
    </Card>
  );
}

export function AnalysisProgress({
  projectId,
  jobId,
  profile,
  onBack,
  onRestarted,
  onNavigate,
  loader,
}: AnalysisProgressProps): JSX.Element {
  const { job, projectName, source, sourceMap, thumbnail, events, error, loading } = useAnalysis(
    projectId,
    jobId,
    loader,
  );

  const [restarting, setRestarting] = useState(false);
  const [restartError, setRestartError] = useState<string | null>(null);
  const cloud = job?.tasks.some((task) => task.kind.endsWith('-cloud')) ?? false;
  const status = readStatus(job);
  const running = status.kind === 'analyzing' || status.kind === 'queued';

  // While the run is live, the readiness report is re-read so a stage that
  // is waiting on a worker or a model says so — and stops saying so the
  // moment the worker connects. A finished run has nothing to wait for.
  const api = useMemo(() => (loader ?? new AnalysisLoader()).api, [loader]);
  const { readiness } = useReadiness(running, api);
  const waiting = useMemo(
    () => (running ? waitingReasons(job, readiness) : new Map<string, string>()),
    [job, readiness, running],
  );

  // Elapsed counts up while the run is live, and stops when it stops.
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (!running) {
      return undefined;
    }
    const timer = setInterval(() => {
      setNow(Date.now());
    }, 1000);
    return () => {
      clearInterval(timer);
    };
  }, [running]);

  if (loading && job === null) {
    return <Skeleton className="h-[420px] rounded-2xl" />;
  }

  if (job === null) {
    return (
      <Alert className="glass rounded-xl">
        <CircleAlert className="text-[var(--color-destructive)]" />
        <AlertDescription className="flex items-center justify-between gap-4">
          {error ?? 'That run is not in the daemon&apos;s store.'}
          <Button variant="outline" size="sm" onClick={onBack}>
            <ArrowLeft />
            Back
          </Button>
        </AlertDescription>
      </Alert>
    );
  }

  const rows = stageRows(job);
  const counts = stageCounts(rows);
  const active = currentStage(rows);
  const badge = describeStatus(status);
  const elapsed = formatElapsed((running ? now : job.updatedUnixMillis) - job.createdUnixMillis);

  return (
    <div className="analysis-page">
      <header className="flex items-start justify-between gap-4">
        <div className="min-w-0">
          <div className="flex items-center gap-3">
            <h1 className="workspace-title min-w-0 break-words">
              {running ? `Analyzing ${projectName}` : projectName}
            </h1>
            <StatusBadge tone={badge.tone}>{badge.label}</StatusBadge>
          </div>
          <p className={cn('mt-1 text-meta', SECONDARY)}>
            {cloud ? 'Cloud-assisted analysis · transcript sharing enabled' : 'Local analysis'}
          </p>
        </div>
        <Button variant="outline" size="sm" onClick={onBack}>
          <ArrowLeft />
          Back
        </Button>
      </header>

      <section className="analysis-status" aria-label="Analysis progress">
        <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
          <span className="text-sm font-medium">{active?.stage.label ?? badge.label}</span>
          <span className={cn('mono text-technical', SECONDARY)}>
            {counts.done} of {counts.planned} stages · {elapsed} elapsed
          </span>
        </div>
        <ProgressBar
          aria-label="Analysis stages complete"
          aria-valuetext={`${counts.done} of ${counts.planned} stages complete`}
          value={counts.planned === 0 ? 0 : (counts.done / counts.planned) * 100}
          className="h-1.5"
        />
      </section>

      {status.kind === 'failed' ? (
        <Alert variant="destructive">
          <CircleAlert className="text-[var(--color-destructive)]" />
          <AlertDescription>
            {job.failureDetail === '' ? 'The run failed.' : job.failureDetail}
          </AlertDescription>
        </Alert>
      ) : null}

      <div className="analysis-layout">
        <Card className="analysis-pipeline">
          <CardHeader>
            <CardTitle className="text-section-title">Analysis steps</CardTitle>
          </CardHeader>
          <CardContent className="px-0" role="list" aria-label="Pipeline stages">
            {rows.map((row) => (
              <StageLine
                key={row.stage.kind}
                row={row}
                waitingFor={
                  [row.stage.kind, ...(row.stage.covers ?? [])]
                    .map((kind) => waiting.get(kind))
                    .find((reason) => reason !== undefined) ?? null
                }
              />
            ))}
          </CardContent>
        </Card>

        <div className="flex flex-col gap-4">
          <SourceCard
            name={projectName}
            path={source?.absolutePath ?? null}
            bytes={source?.byteSize}
            duration={formatDuration(sourceMap)}
            spec={formatVideoSpec(sourceMap)}
            thumbnail={thumbnail}
          />
          <Button
            className="w-full"
            disabled={status.kind !== 'analyzed'}
            onClick={() => {
              onNavigate('results');
            }}
          >
            {status.kind === 'analyzed' ? 'View results' : 'View results when ready'}
          </Button>
          {status.kind === 'failed' && source && onRestarted && (
            <>
              <Button
                variant="outline"
                disabled={restarting}
                onClick={() => {
                  setRestarting(true);
                  setRestartError(null);
                  void api
                    .submitAnalyze(projectId, {
                      sourceId: source.sourceId,
                      language: 'en',
                      minTicks: 20 * 90_000,
                      maxTicks: 90 * 90_000,
                      count: 5,
                      localEditorial: true,
                      contentProfile: job?.contentProfile === 'scripted' ? 'scripted' : 'interview',
                    })
                    .then((next) => onRestarted(projectId, next.jobId))
                    .catch((cause: unknown) =>
                      setRestartError(cause instanceof Error ? cause.message : String(cause)),
                    )
                    .finally(() => setRestarting(false));
                }}
              >
                {restarting ? 'Starting…' : 'Analyze again locally'}
              </Button>
              <p className="text-xs">
                Uses this recording with Qwen 3.5: up to five clips, 20–90 seconds. Existing edits
                stay saved.
              </p>
              {restartError && <p role="alert">{restartError}</p>}
            </>
          )}
          <p className={cn('-mt-2 text-center text-technical', MUTED)}>
            Closing ClipMill pauses the run; it resumes when you reopen.
          </p>
          <details className="import-disclosure analysis-diagnostics">
            <summary>
              <ChevronDown className="size-4" /> Run details
            </summary>
            <div className="mt-4 grid gap-4">
              <DeviceCard profile={profile} cloud={cloud} />
              <LiveLog events={events} job={job} />
            </div>
          </details>
        </div>
      </div>
    </div>
  );
}
