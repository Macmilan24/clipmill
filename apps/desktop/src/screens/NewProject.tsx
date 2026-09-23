import {
  ArrowRight,
  Check,
  ChevronDown,
  FileVideo,
  Folder,
  Minus,
  Plus,
  ShieldCheck,
  TriangleAlert,
  Video,
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

import './import-progress.css';

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
import { YouTubeImport } from '../import/YouTubeImport.js';
import { recallYoutube, rememberYoutube } from '../import/youtube.js';
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
import { formatDuration, formatVideoSpec } from '../library/model.js';

export interface NewProjectProps {
  readonly state: ConnectionState;
  readonly onStarted: (projectId: string, jobId: string) => void;
  /** Open the model library, where a missing model is downloaded. */
  readonly onOpenModels?: () => void;
  /** Injected by tests, which drive the screen through a fake daemon. */
  readonly loader?: ImportLoader;
}

const MUTED = 'text-[var(--cm-text-muted)]';
const SECONDARY = 'text-[var(--cm-text-secondary)]';

/** Model availability blocks submission; absent workers leave a recoverable wait. */
function ReadinessCard({
  readiness,
  problem,
  onRefresh,
  onOpenModels,
}: {
  readonly readiness: Readiness | null;
  readonly problem: string | null;
  readonly onRefresh: () => void;
  readonly onOpenModels?: (() => void) | undefined;
}): JSX.Element {
  const models = missingModels(readiness);
  const workers = missingWorkers(readiness);
  const decoder = readiness !== null && !readiness.decoderPresent;
  const ready = readiness?.ready === true;
  return (
    <Card
      className={cn('import-readiness', ready && !problem && 'import-readiness-ready')}
      data-testid="readiness"
    >
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle className="text-body">Engine status</CardTitle>
        <StatusBadge tone={ready ? 'success' : problem ? 'neutral' : 'warning'}>
          {ready ? 'Ready' : problem ? 'Unknown' : 'Not ready'}
        </StatusBadge>
      </CardHeader>
      <CardContent className="flex flex-col gap-2">
        {problem !== null && <p className={cn('text-meta', SECONDARY)}>{problem}</p>}
        {readiness !== null && ready && (
          <p className={cn('text-meta', SECONDARY)}>
            {readiness.workers.length}{' '}
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
        <div className="flex flex-wrap gap-2">
          {models.length > 0 && onOpenModels !== undefined && (
            <Button size="sm" onClick={onOpenModels}>
              Open Models
            </Button>
          )}
          <Button variant="outline" size="sm" onClick={onRefresh}>
            Check again
          </Button>
        </div>
      </CardContent>
    </Card>
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

export function NewProject({
  state,
  onStarted,
  onOpenModels,
  loader,
}: NewProjectProps): JSX.Element {
  const [importer] = useState(() => loader ?? new ImportLoader());
  const [settings, setSettings] = useState(DEFAULT_SETTINGS);
  const [chosen, setChosen] = useState<ChosenSource | null>(null);
  const [sourceKind, setSourceKind] = useState<'local' | 'youtube'>(() =>
    recallYoutube() ? 'youtube' : 'local',
  );
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
    <div className="import-page">
      <header className="flex items-start justify-between gap-4">
        <div>
          <h1 className="workspace-title">New Project</h1>
          <p className={cn('mt-1 text-meta', SECONDARY)}>
            Choose a recording. Find your next clip.
          </p>
        </div>
      </header>

      {error === null ? null : (
        <Alert variant="destructive">
          <TriangleAlert className="text-[var(--color-destructive)]" />
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      <div className="import-layout">
        <Card className="import-source">
          <CardHeader>
            <CardTitle className="flex items-center gap-1.5 text-section-title">
              <FileVideo className="size-4" /> Source footage
            </CardTitle>
            <StatusBadge
              tone={sourceKind === 'youtube' || route === 'cloud' ? 'outbound' : 'success'}
            >
              <ShieldCheck className="size-3.5" />
              {sourceKind === 'youtube'
                ? 'Downloads from YouTube'
                : route === 'cloud'
                  ? 'Transcript sharing enabled for this run'
                  : 'Stays on this device'}
            </StatusBadge>
          </CardHeader>
          <CardContent>
            <div
              role="group"
              aria-label="Source location"
              className="import-source-tabs mb-4 flex w-fit gap-1 rounded-lg border border-[var(--cm-glass-border)] bg-[var(--cm-recessed)] p-1"
            >
              {(['local', 'youtube'] as const).map((kind) => (
                <Button
                  key={kind}
                  type="button"
                  size="sm"
                  variant={sourceKind === kind ? 'secondary' : 'ghost'}
                  aria-pressed={sourceKind === kind}
                  disabled={busy}
                  onClick={() => {
                    if (sourceKind === kind) return;
                    setSourceKind(kind);
                    setChosen(null);
                    setError(null);
                    if (kind === 'local') rememberYoutube(null);
                  }}
                >
                  {kind === 'local' ? (
                    <FileVideo className="size-3.5" />
                  ) : (
                    <Video className="size-3.5" />
                  )}
                  {kind === 'local' ? 'Local file' : 'YouTube'}
                </Button>
              ))}
            </div>
            {sourceKind === 'youtube' ? (
              <YouTubeImport
                importer={importer}
                connected={connected}
                disabled={busy}
                onChosen={setChosen}
              />
            ) : (
              <div className={chosen === null ? 'import-dropzone' : 'import-change-file'}>
                {chosen === null && (
                  <>
                    <FileVideo className={cn('mx-auto size-8', MUTED)} />
                    <p className="mt-3 text-sm font-medium">Choose a local file</p>
                    <p className={cn('mt-1 text-xs', SECONDARY)}>
                      Your original recording stays untouched.
                    </p>
                  </>
                )}
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
                {chosen === null && (
                  <p className={cn('mt-4 text-[11px]', MUTED)}>
                    MP4 · MOV · MKV · WEBM · M4V · AVI
                  </p>
                )}
              </div>
            )}

            {chosen === null ? null : (
              <>
                <div className="mt-3 flex items-center gap-3 rounded-[var(--cm-radius-panel)] border border-[var(--cm-glass-border)] px-3 py-2.5">
                  <Check className="size-4 shrink-0 text-[var(--color-success)]" />
                  <div className="min-w-0 flex-1">
                    <div className="truncate text-body font-(--cm-weight-label)">
                      {chosen.title || chosen.source.absolutePath.split(/[/\\]/).pop()}
                    </div>
                    <div className={cn('mono truncate text-technical', SECONDARY)}>
                      {formatDuration(chosen.sourceMap)} · {formatVideoSpec(chosen.sourceMap)} ·{' '}
                      {formatBytes(chosen.source.byteSize)}
                    </div>
                  </div>
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

        <Card className="import-options">
          <CardHeader>
            <CardTitle className="text-section-title">Clip preferences</CardTitle>
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
                  <SelectItem value="heuristic">Heuristic baseline · no editorial model</SelectItem>
                </SelectContent>
              </Select>
              {route === 'cloud' && (
                <div className="space-y-3 rounded-lg border border-[color-mix(in_srgb,var(--color-outbound)_30%,transparent)] bg-[var(--cm-recessed)] p-3 text-xs leading-relaxed text-[var(--cm-text-secondary)]">
                  <p>
                    Only transcript text and clip references go to Anthropic. Video, audio, and
                    sampled frames stay local.
                  </p>
                  <p>The run stops before a call would exceed your budget.</p>
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
                  <details className="import-disclosure">
                    <summary>
                      <ChevronDown className="size-3.5" /> API key setup
                    </summary>
                    <p className="mt-2 text-xs">
                      Store your API key in macOS Keychain Access: service dev.clipmill.anthropic,
                      account clipmill. No key is stored in this screen.
                    </p>
                  </details>
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
                  ? 'Find a complete dramatic beat. The wider plot can stay unresolved.'
                  : 'Find complete answers, stories and useful insights.'}
              </p>
              {route === 'heuristic' && (
                <p className="text-[11px] leading-relaxed text-[var(--cm-text-muted)]">
                  The baseline does not use an editorial model. This profile is recorded for the
                  run; the genre rubric applies to model analysis.
                </p>
              )}
            </div>
            <div className="mb-2 text-xs font-medium">Clip length</div>
            <RadioGroup
              className="import-presets"
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
                    'import-preset cursor-pointer border border-[var(--cm-glass-border)]',
                    settings.presetId === preset.id &&
                      'border-[color-mix(in_srgb,var(--color-primary)_45%,transparent)] bg-[var(--cm-accent-selected)]',
                  )}
                >
                  <RadioGroupItem value={preset.id} id={`preset-${preset.id}`} />
                  <span className="min-w-0 flex-1">
                    <span className="block text-body font-(--cm-weight-label)">{preset.label}</span>
                    {preset.id === CUSTOM_PRESET_ID ? null : (
                      <span className={cn('import-preset-duration mono text-technical', MUTED)}>
                        {preset.minSeconds}–{preset.maxSeconds}s
                      </span>
                    )}
                  </span>
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
          </CardContent>
        </Card>
        <Card className="import-start">
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

            <p className="mt-4 text-xs text-[var(--cm-text-secondary)]">
              Up to {settings.count} clips · {describeRange(settings)} ·{' '}
              {route === 'cloud' ? 'Transcript sharing' : 'On this device'}
            </p>

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
            onOpenModels={onOpenModels}
          />
        )}
      </div>
    </div>
  );
}
