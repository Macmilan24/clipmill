import {
  ArrowRight,
  Check,
  FileVideo,
  Folder,
  Minus,
  Plus,
  ShieldCheck,
  TriangleAlert,
} from 'lucide-react';
import { type JSX, useState } from 'react';

import { StatusBadge } from '@/components/StatusBadge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Separator } from '@/components/ui/separator';
import { Spinner } from '@/components/ui/spinner';
import { cn } from '@/lib/utils';

import { shortenPath } from '../analysis/model.js';
import {
  missingModels,
  missingWorkers,
  submissionBlocker,
  useReadiness,
} from '../analysis/readiness.js';
import type { ConnectionState, Readiness } from '../daemon/client.js';
import { formatBytes } from '../deviceProfile.js';
import { type ChosenSource, ImportLoader } from '../import/loader.js';
import {
  COUNT_BOUNDS,
  CUSTOM_PRESET_ID,
  DEFAULT_SETTINGS,
  DURATION_BOUNDS,
  LANGUAGES,
  PRESETS,
  applyPreset,
  blockingReason,
  clamp,
  describeRange,
} from '../import/model.js';
import { EM_DASH, formatDuration, formatVideoSpec } from '../library/model.js';

export interface NewProjectProps {
  readonly state: ConnectionState;
  readonly onStarted: (projectId: string, jobId: string) => void;
  /** Injected by tests, which drive the screen through a fake daemon. */
  readonly loader?: ImportLoader;
}

const MUTED = 'text-[var(--cm-text-muted)]';
const SECONDARY = 'text-[var(--cm-text-secondary)]';

/**
 * What the run would need, stage by stage, and what to run about it.
 *
 * Shown before the wait rather than during it. A stage whose model is not
 * installed is the reason the button above is shut; a stage no worker serves
 * is a wait the daemon will sit in until one connects, said here so nobody
 * discovers it as a spinner.
 */
function ReadinessCard({
  readiness,
  problem,
  onRefresh,
}: {
  readonly readiness: Readiness | null;
  readonly problem: string | null;
  readonly onRefresh: () => void;
}): JSX.Element {
  const models = missingModels(readiness);
  const workers = missingWorkers(readiness);
  const decoder = readiness !== null && !readiness.decoderPresent;
  const ready = readiness?.ready === true;
  return (
    <Card data-testid="readiness">
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle className="text-body">Before it runs</CardTitle>
        <StatusBadge tone={ready ? 'success' : problem ? 'neutral' : 'warning'}>
          {ready ? 'Ready' : problem ? 'Unknown' : 'Not ready'}
        </StatusBadge>
      </CardHeader>
      <CardContent className="flex flex-col gap-2">
        {problem !== null && <p className={cn('text-meta', SECONDARY)}>{problem}</p>}
        {readiness !== null && ready && (
          <p className={cn('text-meta', SECONDARY)}>
            Every stage has its model and a worker to run it. {readiness.workers.length}{' '}
            {readiness.workers.length === 1 ? 'worker is' : 'workers are'} connected.
          </p>
        )}
        {decoder && (
          <p className="text-meta text-[var(--cm-danger-ink)]">
            The pinned decoder is missing at <span className="mono">{readiness?.decoderPath}</span>;
            run <span className="mono">just setup</span>.
          </p>
        )}
        {models.length > 0 && (
          <ul className="flex flex-col gap-1" aria-label="Models not installed">
            {models.map((stage) => (
              <li key={stage.stage} className="text-meta">
                <span className="mono text-[var(--cm-danger-ink)]">{stage.stage}</span>{' '}
                <span className={SECONDARY}>{stage.remedy}</span>
              </li>
            ))}
          </ul>
        )}
        {workers.length > 0 && (
          <ul className="flex flex-col gap-1" aria-label="Stages with no worker">
            {workers.map((stage) => (
              <li key={stage.stage} className="text-meta">
                <span className="mono text-[var(--color-warning)]">{stage.stage}</span>{' '}
                <span className={SECONDARY}>{stage.remedy}</span>
              </li>
            ))}
          </ul>
        )}
        {workers.length > 0 && models.length === 0 && !decoder && (
          <p className={cn('text-meta', MUTED)}>
            The run can be started; those stages wait until a worker connects.
          </p>
        )}
        <Button variant="outline" size="sm" className="self-start" onClick={onRefresh}>
          Check again
        </Button>
      </CardContent>
    </Card>
  );
}

function SummaryRow({
  label,
  value,
}: {
  readonly label: string;
  readonly value: string;
}): JSX.Element {
  return (
    <div className="flex items-center justify-between gap-2">
      <span className={cn('text-meta', SECONDARY)}>{label}</span>
      <span className="mono truncate text-technical">{value}</span>
    </div>
  );
}

/** A number with two buttons, bounded, so the field can never be asked for 0. */
function Stepper({
  value,
  onChange,
  low,
  high,
  label,
}: {
  readonly value: number;
  readonly onChange: (value: number) => void;
  readonly low: number;
  readonly high: number;
  readonly label: string;
}): JSX.Element {
  return (
    <div className="flex items-center gap-1">
      <Button
        variant="outline"
        size="icon-sm"
        aria-label={`Fewer ${label}`}
        disabled={value <= low}
        onClick={() => {
          onChange(clamp(value - 1, low, high));
        }}
      >
        <Minus />
      </Button>
      <span className="mono w-8 text-center text-body">{value}</span>
      <Button
        variant="outline"
        size="icon-sm"
        aria-label={`More ${label}`}
        disabled={value >= high}
        onClick={() => {
          onChange(clamp(value + 1, low, high));
        }}
      >
        <Plus />
      </Button>
    </div>
  );
}

export function NewProject({ state, onStarted, loader }: NewProjectProps): JSX.Element {
  const [importer] = useState(() => loader ?? new ImportLoader());
  const [settings, setSettings] = useState(DEFAULT_SETTINGS);
  const [chosen, setChosen] = useState<ChosenSource | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const connected = state.status === 'connected';
  // What the run would need, asked before it is submitted: a model that is
  // not installed blocks the button, and a worker that is not connected is
  // said out loud, since the run would sit on that stage until one is.
  const { readiness, problem: readinessProblem, refresh } = useReadiness(connected, importer.api);
  const route = settings.editorialRoute ?? 'local';
  const routeReadiness = readiness
    ? {
        ...readiness,
        stages: readiness.stages.filter(
          (s) =>
            !s.stage.startsWith('editorial-') ||
            (route === 'local'
              ? !s.stage.endsWith('-cloud')
              : route === 'cloud'
                ? s.stage.endsWith('-cloud') || s.stage === 'editorial-look'
                : false),
        ),
      }
    : null;
  const cloudBlocker =
    route === 'cloud' &&
    (!settings.cloudConsent ||
      !Number.isFinite(settings.cloudBudgetUsd ?? 2) ||
      (settings.cloudBudgetUsd ?? 2) < 0.01 ||
      (settings.cloudBudgetUsd ?? 2) > 100)
      ? 'Confirm transcript sharing and set a run budget between $0.01 and $100.'
      : null;
  const blocked =
    cloudBlocker ??
    blockingReason(settings, chosen !== null, busy) ??
    (connected ? submissionBlocker(routeReadiness) : null);

  const choose = async (): Promise<void> => {
    setBusy(true);
    setError(null);
    try {
      const path = await importer.choose();
      if (path !== null) {
        // The same project is reused when the choice changes, so looking at
        // three files does not leave three empty projects behind.
        setChosen(await importer.register(path, chosen?.projectId ?? null));
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  };

  const start = async (): Promise<void> => {
    if (chosen === null) {
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const job = await importer.start(chosen, settings);
      onStarted(chosen.projectId, job.jobId);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
      setBusy(false);
    }
  };

  return (
    <>
      <div className="mb-4 flex h-12 items-start justify-between gap-4">
        <div>
          <h1 className="text-page-title font-(--cm-weight-heading) tracking-[-0.01em]">
            New Project
          </h1>
          <p className={cn('mt-1 text-meta', SECONDARY)}>
            Find complete moments in English podcasts, interviews and scripted scenes.
          </p>
        </div>
      </div>

      {error === null ? null : (
        <Alert className="glass mb-4 rounded-xl">
          <TriangleAlert className="text-[var(--color-destructive)]" />
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      <div className="grid grid-cols-[minmax(0,1fr)_320px] items-start gap-5">
        <Card className="glass rounded-xl">
          <CardHeader>
            <CardTitle className="flex items-center gap-1.5 text-section-title">
              <FileVideo className="size-4" /> Source footage
            </CardTitle>
            <StatusBadge tone="success">
              <ShieldCheck className="size-3.5" />
              {route === 'cloud'
                ? 'Transcript sharing enabled for this run'
                : 'Stays on this device'}
            </StatusBadge>
          </CardHeader>
          <CardContent>
            <div className="rounded-[var(--cm-radius-panel)] border border-dashed border-[var(--cm-glass-border)] bg-[var(--cm-recessed)] px-5 py-10 text-center">
              <FileVideo className={cn('mx-auto size-7', MUTED)} />
              <p className="mt-2 text-body font-(--cm-weight-label)">Choose a local file</p>
              <p className={cn('mt-0.5 text-meta', SECONDARY)}>
                Video and audio are processed on this device. Cloud-assisted mode sends transcript
                text only.
              </p>
              <Button
                variant="outline"
                size="sm"
                className="mt-3"
                disabled={busy || !connected}
                onClick={() => {
                  void choose();
                }}
              >
                {busy ? <Spinner /> : null}
                {chosen === null ? 'Browse files' : 'Choose a different file'}
              </Button>
              <p className={cn('mono mt-3 text-technical', MUTED)}>
                MP4 · MOV · MKV · WEBM · M4V · AVI
              </p>
            </div>

            {chosen === null ? null : (
              <>
                <div className="mt-3 flex items-center gap-3 rounded-[var(--cm-radius-panel)] border border-[var(--cm-glass-border)] px-3 py-2.5">
                  <Check className="size-4 shrink-0 text-[var(--color-success)]" />
                  <div className="min-w-0 flex-1">
                    <div className="truncate text-body font-(--cm-weight-label)">
                      {chosen.source.absolutePath.split(/[/\\]/).pop()}
                    </div>
                    <div className={cn('mono truncate text-technical', SECONDARY)}>
                      {formatDuration(chosen.sourceMap)} · {formatVideoSpec(chosen.sourceMap)} ·{' '}
                      {formatBytes(chosen.source.byteSize)}
                    </div>
                  </div>
                  {chosen.cached ? (
                    <span className={cn('mono shrink-0 text-technical', MUTED)}>probe cached</span>
                  ) : null}
                </div>
                <div className="mt-2 flex items-center gap-1.5">
                  <Folder className={cn('size-3.5 shrink-0', MUTED)} />
                  <span
                    className={cn('mono truncate text-technical', MUTED)}
                    title={chosen.source.absolutePath}
                  >
                    {shortenPath(chosen.source.absolutePath, 76)}
                  </span>
                </div>
              </>
            )}
          </CardContent>
        </Card>

        <div className="flex flex-col gap-4">
          <Card className="glass rounded-xl">
            <CardHeader>
              <CardTitle className="text-section-title">Analysis setup</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="mb-4 space-y-3">
                <Label htmlFor="editorial-route">Editorial analysis</Label>
                <Select
                  value={route}
                  disabled={busy}
                  onValueChange={(value) =>
                    setSettings((current) => ({
                      ...current,
                      editorialRoute: value as 'local' | 'cloud' | 'heuristic',
                      cloudConsent: false,
                    }))
                  }
                >
                  <SelectTrigger id="editorial-route" className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="local">Local · Qwen 3.5</SelectItem>
                    <SelectItem value="cloud">
                      Cloud-assisted · Anthropic Claude Sonnet 4.6
                    </SelectItem>
                    <SelectItem value="heuristic">
                      Heuristic baseline · no editorial model
                    </SelectItem>
                  </SelectContent>
                </Select>
                {route === 'cloud' && (
                  <div className="space-y-3 rounded-lg border border-[color-mix(in_srgb,var(--color-outbound)_30%,transparent)] bg-[var(--cm-recessed)] p-3 text-xs leading-relaxed text-[var(--cm-text-secondary)]">
                    <p>
                      Only transcript text and clip references go to Anthropic. Video, audio, and
                      sampled frames stay local.
                    </p>
                    <p>
                      Cost example: a call with 10,000 input tokens and 2,000 output tokens costs
                      about $0.06. Each transcript window and candidate needs a call, so the total
                      depends on the recording. The run stops before a call would exceed your
                      budget.
                    </p>
                    <Label htmlFor="cloud-budget">Maximum spend for this run (USD)</Label>
                    <Input
                      id="cloud-budget"
                      type="number"
                      min="0.01"
                      max="100"
                      step="0.25"
                      value={settings.cloudBudgetUsd ?? 2}
                      onChange={(event) =>
                        setSettings((current) => ({
                          ...current,
                          cloudBudgetUsd: Number(event.target.value),
                        }))
                      }
                    />
                    <Label>
                      <Checkbox
                        checked={settings.cloudConsent ?? false}
                        onCheckedChange={(checked) =>
                          setSettings((current) => ({ ...current, cloudConsent: checked === true }))
                        }
                      />
                      Allow transcript sharing with Anthropic for this run
                    </Label>
                    <p className="text-xs">
                      Store your API key in macOS Keychain Access: service dev.clipmill.anthropic,
                      account clipmill. No key is stored in this screen.
                    </p>
                  </div>
                )}
              </div>
              <div className="mb-5 space-y-2.5">
                <Label htmlFor="content-profile">Footage type</Label>
                <Select
                  value={settings.contentProfile ?? 'interview'}
                  disabled={busy}
                  onValueChange={(value) =>
                    setSettings((current) => ({
                      ...current,
                      contentProfile: value as 'interview' | 'scripted',
                    }))
                  }
                >
                  <SelectTrigger id="content-profile" className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="interview">Podcast / interview</SelectItem>
                    <SelectItem value="scripted">TV / movie scene</SelectItem>
                  </SelectContent>
                </Select>
                <p className="text-[11px] leading-relaxed text-[var(--cm-text-secondary)]">
                  {settings.contentProfile === 'scripted'
                    ? 'Looks for a complete dramatic beat: a reveal and reaction, confrontation, joke or emotional turn. The wider plot can stay unresolved.'
                    : 'Looks for a complete answer, story, useful insight or takeaway, with the context a new viewer needs.'}
                </p>
                {route === 'heuristic' && (
                  <p className="text-[11px] leading-relaxed text-[var(--cm-text-muted)]">
                    The baseline does not use an editorial model. This profile is recorded for the
                    run; the genre rubric applies to model analysis.
                  </p>
                )}
              </div>
              <RadioGroup
                value={settings.presetId}
                onValueChange={(presetId) => {
                  setSettings((current) => applyPreset(current, presetId));
                }}
                aria-label="Clip length"
              >
                {PRESETS.map((preset) => (
                  <Label
                    key={preset.id}
                    htmlFor={`preset-${preset.id}`}
                    className={cn(
                      'flex cursor-pointer items-center gap-3 rounded-[var(--cm-radius-panel)] border border-[var(--cm-glass-border)] px-3 py-2.5',
                      settings.presetId === preset.id &&
                        'border-[color-mix(in_srgb,var(--color-primary)_45%,transparent)] bg-[var(--cm-accent-selected)]',
                    )}
                  >
                    <RadioGroupItem value={preset.id} id={`preset-${preset.id}`} />
                    <span className="min-w-0 flex-1">
                      <span className="block text-body font-(--cm-weight-label)">
                        {preset.label}
                      </span>
                      <span className={cn('block text-meta', SECONDARY)}>{preset.detail}</span>
                    </span>
                    {preset.id === CUSTOM_PRESET_ID ? null : (
                      <span className={cn('mono shrink-0 text-technical', MUTED)}>
                        {preset.minSeconds}–{preset.maxSeconds}s
                      </span>
                    )}
                  </Label>
                ))}
              </RadioGroup>

              {settings.presetId === CUSTOM_PRESET_ID ? (
                <div className="mt-2 flex items-center gap-2">
                  {(['minSeconds', 'maxSeconds'] as const).map((field) => (
                    <span key={field} className="flex-1">
                      <Label htmlFor={field} className={cn('mb-1 block text-meta', SECONDARY)}>
                        {field === 'minSeconds' ? 'Shortest' : 'Longest'}
                      </Label>
                      <Input
                        id={field}
                        type="number"
                        className="mono h-8"
                        min={DURATION_BOUNDS.min}
                        max={DURATION_BOUNDS.max}
                        value={settings[field]}
                        onChange={(event) => {
                          const seconds = Number.parseInt(event.target.value, 10);
                          setSettings((current) => ({
                            ...current,
                            [field]: Number.isNaN(seconds) ? current[field] : seconds,
                          }));
                        }}
                      />
                    </span>
                  ))}
                </div>
              ) : null}

              <Separator className="my-3 bg-[var(--cm-glass-border)]" />

              <div className="flex items-center justify-between gap-2">
                <span className="text-body">Clips to find</span>
                <Stepper
                  value={settings.count}
                  low={COUNT_BOUNDS.min}
                  high={COUNT_BOUNDS.max}
                  label="clips"
                  onChange={(count) => {
                    setSettings((current) => ({ ...current, count }));
                  }}
                />
              </div>

              <div className="mt-3 flex items-center justify-between gap-2">
                <Label htmlFor="language" className="text-body">
                  Language
                </Label>
                <Select
                  value={settings.language}
                  onValueChange={(language) => {
                    setSettings((current) => ({ ...current, language }));
                  }}
                >
                  <SelectTrigger id="language" className="w-[184px]" aria-label="Language">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {LANGUAGES.map((option) => (
                      <SelectItem key={option.value} value={option.value}>
                        {option.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <Separator className="my-3 bg-[var(--cm-glass-border)]" />
            </CardContent>
          </Card>

          <Card className="glass rounded-xl">
            <CardHeader>
              <CardTitle className="text-section-title">Rights &amp; run</CardTitle>
            </CardHeader>
            <CardContent>
              <Label
                htmlFor="rights"
                className="flex cursor-pointer items-start gap-2.5 text-meta leading-snug"
              >
                <Checkbox
                  id="rights"
                  checked={settings.rightsAttested}
                  onCheckedChange={(checked) => {
                    setSettings((current) => ({
                      ...current,
                      rightsAttested: checked === true,
                    }));
                  }}
                  className="mt-0.5"
                />
                I own this footage or hold the rights to clip and publish it.
              </Label>

              <Separator className="my-3 bg-[var(--cm-glass-border)]" />

              <div className="grid gap-2">
                <SummaryRow
                  label="Source"
                  value={chosen === null ? EM_DASH : `${formatBytes(chosen.source.byteSize)} local`}
                />
                <SummaryRow label="Clip length" value={describeRange(settings)} />
                <SummaryRow label="Network" value="0 bytes" />
              </div>

              {/* Disabled reads as disabled, not as a dimmed primary: indigo is
                  reserved for an action that can actually be taken. */}
              <Button
                variant={blocked === null && connected ? 'default' : 'outline'}
                className="mt-4 w-full"
                disabled={blocked !== null || !connected}
                onClick={() => {
                  void start();
                }}
              >
                {busy ? <Spinner /> : null}
                Analyze video
                <ArrowRight />
              </Button>
              {/* A disabled button that says nothing teaches nobody what to do. */}
              <p className={cn('mt-2 text-center text-technical', MUTED)}>
                {blocked ?? 'Closing ClipMill pauses the run; it resumes when you reopen.'}
              </p>
            </CardContent>
          </Card>

          {connected && (
            <ReadinessCard
              readiness={routeReadiness}
              problem={readinessProblem}
              onRefresh={refresh}
            />
          )}
        </div>
      </div>
    </>
  );
}
