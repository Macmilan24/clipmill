import {
  ArrowRight,
  Check,
  FileVideo,
  Folder,
  Globe,
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
import { Switch } from '@/components/ui/switch';
import { cn } from '@/lib/utils';

import { shortenPath } from '../analysis/model.js';
import type { ConnectionState } from '../daemon/client.js';
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
  const blocked = blockingReason(settings, chosen !== null, busy);

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
            Import one long-form recording and say what a strong clip means for this run.
          </p>
        </div>
      </div>

      {error === null ? null : (
        <Alert className="glass mb-4 rounded-xl">
          <TriangleAlert className="text-[var(--color-destructive)]" />
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      <div className="grid grid-cols-[minmax(0,744fr)_minmax(0,400fr)] items-start gap-4">
        <Card className="glass rounded-2xl">
          <CardHeader>
            <CardTitle className="flex items-center gap-1.5 text-section-title">
              <FileVideo className="size-4" /> Source footage
            </CardTitle>
            <StatusBadge tone="success">
              <ShieldCheck className="size-3.5" />
              Stays on this device
            </StatusBadge>
          </CardHeader>
          <CardContent>
            <div className="rounded-[var(--cm-radius-panel)] border border-dashed border-[var(--cm-glass-border)] bg-[var(--cm-recessed)] px-4 py-6 text-center">
              <FileVideo className={cn('mx-auto size-7', MUTED)} />
              <p className="mt-2 text-body font-(--cm-weight-label)">Choose a local file</p>
              <p className={cn('mt-0.5 text-meta', SECONDARY)}>
                It is read where it sits and never copied off this machine.
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
          <Card className="glass rounded-2xl">
            <CardHeader>
              <CardTitle className="text-section-title">Analysis setup</CardTitle>
            </CardHeader>
            <CardContent>
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
                      <Label
                        htmlFor={field}
                        className={cn('mb-1 block text-meta', SECONDARY)}
                      >
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

              {/* Off and not switchable: a statement of what this build does,
                  not a preference. There is no network broker to turn on. */}
              <div className="flex items-center justify-between gap-3">
                <span className="flex min-w-0 items-center gap-2">
                  <Globe className="size-4 shrink-0 text-[var(--color-outbound)]" />
                  <span className="min-w-0">
                    <span className="block text-body">Use cloud models</span>
                    <span className={cn('block text-meta', SECONDARY)}>
                      Nothing leaves this device. No cloud path exists to enable.
                    </span>
                  </span>
                </span>
                <Switch checked={false} disabled aria-label="Use cloud models" />
              </div>
            </CardContent>
          </Card>

          <Card className="glass rounded-2xl">
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
                  value={
                    chosen === null ? EM_DASH : `${formatBytes(chosen.source.byteSize)} local`
                  }
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
        </div>
      </div>
    </>
  );
}
