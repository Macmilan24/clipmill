import {
  Database,
  HardDrive,
  Lock,
  Moon,
  Palette,
  Plug,
  RefreshCw,
  ShieldCheck,
  ShieldOff,
  Sun,
  TriangleAlert,
} from 'lucide-react';
import type { JSX, ReactNode } from 'react';
import type { Theme } from '@clipmill/tokens';

import { StatusBadge } from '@/components/StatusBadge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { Spinner } from '@/components/ui/spinner';
import { cn } from '@/lib/utils';

import type { LocalLock, StorageStats } from '../daemon/client.js';
import { formatBytes } from '../deviceProfile.js';

const CATEGORY_LABELS: Readonly<Record<string, string>> = {
  artifacts: 'Generated media',
  models: 'Model weights',
  state: 'Project state',
  imports: 'Imported originals',
};
const CATEGORY_NOTES: Readonly<Record<string, string>> = {
  artifacts:
    'Proxies, transcripts, analysis and rendered media. Unreferenced files follow the retention policy.',
  models: 'Pinned model files, downloaded once and reused across projects.',
  state: 'Projects, saved edits and decisions. Keep these files to preserve your work.',
  imports:
    'Downloaded originals are kept with their project. Deleting the project removes its managed copies.',
};
const CATEGORY_COLORS: Readonly<Record<string, string>> = {
  artifacts: 'var(--cm-text-secondary)',
  models: 'var(--cm-text-muted)',
  state: 'var(--cm-text-primary)',
  imports: 'var(--color-primary)',
};

export interface SettingsProps {
  readonly storage: StorageStats | null;
  readonly lock: LocalLock | null;
  readonly loading: boolean;
  readonly error: string | null;
  readonly storageError?: string | null;
  readonly lockError?: string | null;
  readonly onRefresh?: () => void;
  readonly theme?: Theme;
  readonly onThemeChange?: (theme: Theme) => void;
  readonly integrations?: ReactNode;
}

function SectionHeading({
  icon,
  title,
  detail,
}: {
  readonly icon: ReactNode;
  readonly title: string;
  readonly detail: string;
}) {
  return (
    <CardHeader className="border-b border-[var(--cm-glass-border)] px-5 py-4">
      <div className="flex items-center gap-2">
        <span className="text-[var(--cm-text-secondary)] [&_svg]:size-4">{icon}</span>
        <CardTitle className="text-sm">{title}</CardTitle>
      </div>
      <p className="text-xs leading-relaxed text-[var(--cm-text-secondary)]">{detail}</p>
    </CardHeader>
  );
}

export function Settings({
  storage,
  lock,
  loading,
  error,
  storageError,
  lockError,
  onRefresh,
  theme,
  onThemeChange,
  integrations,
}: SettingsProps): JSX.Element {
  const total =
    storage?.categories.reduce((sum, category) => sum + Math.max(0, category.bytes), 0) ?? 0;
  const appearance = theme !== undefined && onThemeChange !== undefined;
  return (
    <div className="mx-auto flex w-full max-w-[1160px] flex-col gap-6">
      <header className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <p className="mb-2 text-[10px] font-semibold uppercase tracking-[0.16em] text-[var(--cm-text-muted)]">
            Your workspace
          </p>
          <h1 className="workspace-title">Settings &amp; privacy</h1>
          <p className="workspace-subtitle mt-1">
            Make the studio yours. Know where your work lives.
          </p>
        </div>
        {onRefresh && (
          <Button variant="outline" size="sm" disabled={loading} onClick={onRefresh}>
            {loading ? <Spinner /> : <RefreshCw />}
            {loading ? 'Refreshing…' : 'Refresh status'}
          </Button>
        )}
      </header>
      {error !== null && (
        <Alert variant="destructive">
          <TriangleAlert />
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}
      <div className="grid items-start gap-5 xl:grid-cols-[168px_minmax(0,1fr)] xl:gap-8">
        <nav
          aria-label="Settings sections"
          className="flex flex-wrap gap-1 xl:sticky xl:top-0 xl:flex-col"
        >
          {appearance && (
            <SectionLink href="#settings-appearance" icon={<Palette />} label="Appearance" />
          )}
          {integrations && (
            <SectionLink href="#settings-integrations" icon={<Plug />} label="Connections" />
          )}
          <SectionLink href="#settings-privacy" icon={<ShieldCheck />} label="Privacy & cloud" />
          <SectionLink href="#settings-storage" icon={<HardDrive />} label="Storage" />
          <p className="mt-5 hidden px-2 text-[11px] leading-relaxed text-[var(--cm-text-muted)] xl:block">
            Preferences apply to this installation. Local files stay in their original location;
            imported YouTube copies are managed below.
          </p>
        </nav>
        <div className="min-w-0 space-y-5">
          {appearance && (
            <section id="settings-appearance" className="scroll-mt-6" aria-label="Appearance">
              <Card className="gap-0 overflow-hidden py-0">
                <SectionHeading
                  icon={<Palette />}
                  title="Appearance"
                  detail="Choose the workspace that feels right for your editing session."
                />
                <CardContent className="px-5 py-5">
                  <RadioGroup
                    value={theme}
                    onValueChange={(value) => {
                      if (value === 'dark' || value === 'light') onThemeChange(value);
                    }}
                    aria-label="Workspace theme"
                    className="grid grid-cols-2 gap-3"
                  >
                    {(['dark', 'light'] as const).map((option) => (
                      <label
                        key={option}
                        htmlFor={`theme-${option}`}
                        className={cn(
                          'cursor-pointer rounded-xl border p-3 transition-colors',
                          theme === option
                            ? 'border-[var(--color-primary)] bg-[var(--cm-accent-selected)]'
                            : 'border-[var(--cm-glass-border)] hover:bg-[var(--cm-recessed)]',
                        )}
                      >
                        <div
                          aria-hidden="true"
                          className={cn(
                            'flex h-[82px] overflow-hidden rounded-md border',
                            option === 'dark'
                              ? 'border-slate-700 bg-slate-900'
                              : 'border-slate-200 bg-slate-50',
                          )}
                        >
                          <div
                            className={cn(
                              'w-1/4 space-y-2 border-r p-2.5',
                              option === 'dark'
                                ? 'border-slate-700 bg-slate-950'
                                : 'border-slate-200 bg-white',
                            )}
                          >
                            <div
                              className={cn(
                                'h-1.5 w-5 rounded-sm',
                                option === 'dark' ? 'bg-slate-400' : 'bg-slate-400',
                              )}
                            />
                            <div
                              className={cn(
                                'h-1 w-full rounded-sm',
                                option === 'dark' ? 'bg-slate-700' : 'bg-slate-200',
                              )}
                            />
                            <div
                              className={cn(
                                'h-1 w-3/4 rounded-sm',
                                option === 'dark' ? 'bg-slate-700' : 'bg-slate-200',
                              )}
                            />
                          </div>
                          <div className="flex-1 p-3">
                            <div
                              className={cn(
                                'h-1.5 w-2/3 rounded-sm',
                                option === 'dark' ? 'bg-slate-400' : 'bg-slate-500',
                              )}
                            />
                            <div className="mt-3 flex gap-1.5">
                              {[0, 1, 2].map((key) => (
                                <div
                                  key={key}
                                  className={cn(
                                    'h-8 flex-1 rounded-sm border',
                                    option === 'dark'
                                      ? 'border-slate-700 bg-slate-800'
                                      : 'border-slate-200 bg-white',
                                  )}
                                />
                              ))}
                            </div>
                          </div>
                        </div>
                        <div className="mt-3 flex items-center justify-between gap-2">
                          <span className="flex items-center gap-2 text-xs font-medium">
                            {option === 'dark' ? (
                              <Moon className="size-3.5" />
                            ) : (
                              <Sun className="size-3.5" />
                            )}
                            {option === 'dark' ? 'Dark studio' : 'Light studio'}
                          </span>
                          <RadioGroupItem
                            id={`theme-${option}`}
                            value={option}
                            aria-label={option === 'dark' ? 'Dark studio' : 'Light studio'}
                          />
                        </div>
                      </label>
                    ))}
                  </RadioGroup>
                  <p className="mt-3 text-[11px] text-[var(--cm-text-muted)]">
                    Saved automatically on this device. Change it any time from the toolbar.
                  </p>
                </CardContent>
              </Card>
            </section>
          )}
          {integrations && (
            <section
              id="settings-integrations"
              className="scroll-mt-6"
              aria-label="Connected accounts"
            >
              {integrations}
            </section>
          )}
          <section
            id="settings-privacy"
            className="scroll-mt-6"
            aria-label="Privacy and cloud processing"
          >
            <Card className="gap-0 overflow-hidden py-0">
              <SectionHeading
                icon={<ShieldCheck />}
                title="Privacy & cloud"
                detail="Local processing is the default. Cloud analysis requires your explicit consent."
              />
              <CardContent className="px-5 py-5">
                {lockError && (
                  <p
                    role="status"
                    className="mb-4 text-xs leading-relaxed text-[var(--cm-warning-ink)]"
                  >
                    Privacy status could not be refreshed. {lockError}
                    {lock && ' Showing the last successful check.'}
                  </p>
                )}
                {lock === null ? (
                  <p className="text-xs text-[var(--cm-text-secondary)]">
                    {loading ? 'Asking…' : 'This daemon reports no policy.'}
                  </p>
                ) : (
                  <>
                    <div className="flex flex-wrap items-start gap-3">
                      <div className="flex size-10 shrink-0 items-center justify-center rounded-lg border border-[var(--cm-glass-border)] bg-[var(--cm-recessed)]">
                        {lock.engaged ? (
                          <Lock className="size-5 text-[var(--cm-text-secondary)]" />
                        ) : (
                          <ShieldOff className="size-5 text-[var(--cm-warning-ink)]" />
                        )}
                      </div>
                      <div className="min-w-0 flex-1">
                        <div className="flex flex-wrap items-center gap-2">
                          <h3 className="text-sm font-semibold">Local Lock</h3>
                          <StatusBadge
                            tone={lockError ? 'neutral' : lock.engaged ? 'success' : 'outbound'}
                          >
                            {lockError
                              ? 'Last known status'
                              : lock.engaged
                                ? 'Engaged'
                                : 'Not engaged'}
                          </StatusBadge>
                        </div>
                        <p className="mt-1.5 text-xs leading-relaxed text-[var(--cm-text-secondary)]">
                          {lock.engaged
                            ? 'No network operations have started in this daemon session.'
                            : 'Network operations have started in this daemon session.'}
                        </p>
                      </div>
                    </div>
                    <dl className="mt-5 grid grid-cols-3 gap-3 rounded-lg border border-[var(--cm-glass-border)] bg-[var(--cm-recessed)] p-3.5">
                      <Count label="Stages registered" value={lock.stages} />
                      <Count
                        label="Stages allowed to use the network"
                        value={lock.networkAllowedStages}
                      />
                      <Count
                        label="Network operations started this session"
                        value={lock.egressAttempts}
                      />
                    </dl>
                    <p className="mt-2 text-xs text-[var(--cm-text-muted)]">
                      Includes enabled cloud analysis, YouTube imports, channel sign-in and
                      publishing. Importing a video does not enable cloud AI. Model downloads are
                      managed separately.
                    </p>
                    <p className="mt-3 text-[11px] leading-relaxed text-[var(--cm-text-muted)]">
                      Cloud-capable stages may be installed without being used. These counts
                      describe task policy and execution, not measured network traffic. Publishing
                      to a connected channel is a separate action from AI processing.
                    </p>
                  </>
                )}
              </CardContent>
            </Card>
          </section>
          <section id="settings-storage" className="scroll-mt-6" aria-label="Storage">
            <Card className="gap-0 overflow-hidden py-0">
              <SectionHeading
                icon={<HardDrive />}
                title="Storage"
                detail="A clear account of the files ClipMill manages on this device."
              />
              <CardContent className="px-5 py-5">
                {storageError && (
                  <p
                    role="status"
                    className="mb-4 text-xs leading-relaxed text-[var(--cm-warning-ink)]"
                  >
                    Storage could not be refreshed. {storageError}
                    {storage && ' Showing the last successful measurement.'}
                  </p>
                )}
                {loading && storage === null ? (
                  <p className="flex items-center gap-2 text-xs text-[var(--cm-text-secondary)]">
                    <Spinner className="size-3" />
                    Measuring…
                  </p>
                ) : storage === null ? (
                  <p className="text-xs text-[var(--cm-text-secondary)]">
                    This daemon measures no storage.
                  </p>
                ) : (
                  <>
                    <div className="flex flex-wrap items-end justify-between gap-3">
                      <div>
                        <p className="text-xs text-[var(--cm-text-secondary)]">Managed files</p>
                        <p className="mt-1 font-mono text-2xl font-medium tracking-tight">
                          {formatBytes(total)}
                        </p>
                      </div>
                      <div className="text-right">
                        <p className="text-[11px] text-[var(--cm-text-muted)]">
                          Free on this volume
                        </p>
                        <p className="mt-1 font-mono text-sm">
                          {storage.availableBytes === undefined
                            ? 'not readable'
                            : formatBytes(storage.availableBytes)}
                        </p>
                      </div>
                    </div>
                    <div
                      className="mt-4 flex h-2 gap-0.5 overflow-hidden rounded-full bg-[var(--cm-recessed)]"
                      aria-hidden="true"
                    >
                      {storage.categories.map((category) => (
                        <div
                          key={category.key}
                          style={{
                            width: `${total > 0 ? (Math.max(0, category.bytes) / total) * 100 : 0}%`,
                            backgroundColor:
                              CATEGORY_COLORS[category.key] ?? 'var(--cm-text-muted)',
                          }}
                        />
                      ))}
                    </div>
                    <ul className="mt-4 divide-y divide-[var(--cm-glass-border)]">
                      {storage.categories.map((category) => (
                        <li key={category.key} className="py-4 first:pt-0">
                          <div className="flex items-start justify-between gap-4">
                            <div className="min-w-0">
                              <h3 className="flex items-center gap-2 text-xs font-semibold">
                                <span
                                  aria-hidden="true"
                                  className="size-1.5 rounded-full"
                                  style={{
                                    backgroundColor:
                                      CATEGORY_COLORS[category.key] ?? 'var(--cm-text-muted)',
                                  }}
                                />
                                {CATEGORY_LABELS[category.key] ?? category.key}
                              </h3>
                              <p className="mt-1.5 max-w-[530px] text-[11px] leading-relaxed text-[var(--cm-text-secondary)]">
                                {CATEGORY_NOTES[category.key] ??
                                  'Files managed by the local engine.'}
                              </p>
                            </div>
                            <div className="shrink-0 text-right">
                              <p className="font-mono text-xs">{formatBytes(category.bytes)}</p>
                              <p className="mt-1 text-[10px] text-[var(--cm-text-muted)]">
                                {category.items} {category.items === 1 ? 'item' : 'items'}
                              </p>
                            </div>
                          </div>
                          <p className="mt-2 select-all break-all rounded-md bg-[var(--cm-recessed)] px-2.5 py-2 font-mono text-[10px] leading-relaxed text-[var(--cm-text-muted)]">
                            {category.path}
                          </p>
                        </li>
                      ))}
                    </ul>
                    <div className="flex items-start gap-3 rounded-lg border border-[var(--cm-glass-border)] bg-[var(--cm-recessed)] px-3.5 py-3.5">
                      <Database className="mt-0.5 size-4 shrink-0 text-[var(--cm-text-secondary)]" />
                      <div>
                        <div className="flex flex-wrap items-center gap-2">
                          <h3 className="text-xs font-semibold">Unused generated files</h3>
                          <StatusBadge tone="neutral">
                            {describeGrace(storage.retentionGraceSeconds)}
                          </StatusBadge>
                        </div>
                        <p className="mt-2 text-[11px] leading-relaxed text-[var(--cm-text-secondary)]">
                          Unreferenced generated files become eligible for cleanup after this
                          retention period. Your source recordings and saved edits are kept.
                        </p>
                      </div>
                    </div>
                  </>
                )}
              </CardContent>
            </Card>
            <p className="mt-3 text-[11px] leading-relaxed text-[var(--cm-text-muted)]">
              Storage locations and retention are managed by the local engine.
            </p>
          </section>
        </div>
      </div>
    </div>
  );
}

function SectionLink({
  href,
  icon,
  label,
}: {
  readonly href: string;
  readonly icon: ReactNode;
  readonly label: string;
}) {
  return (
    <a
      href={href}
      className="flex items-center gap-2 rounded-md px-2.5 py-2 text-xs font-medium text-[var(--cm-text-secondary)] transition-colors hover:bg-[var(--cm-recessed)] hover:text-[var(--cm-text-primary)] [&_svg]:size-3.5"
    >
      {icon}
      {label}
    </a>
  );
}
function Count({ label, value }: { readonly label: string; readonly value: number }) {
  return (
    <div>
      <dd className="font-mono text-lg">{value}</dd>
      <dt className="mt-1 max-w-[155px] text-[10px] leading-relaxed text-[var(--cm-text-secondary)]">
        {label}
      </dt>
    </div>
  );
}
function describeGrace(seconds: number): string {
  if (seconds <= 0) return 'collected immediately';
  const days = Math.floor(seconds / 86_400);
  if (days >= 1) return `${days} day${days === 1 ? '' : 's'}`;
  const hours = Math.floor(seconds / 3_600);
  if (hours >= 1) return `${hours} hour${hours === 1 ? '' : 's'}`;
  const minutes = Math.max(1, Math.floor(seconds / 60));
  return `${minutes} minute${minutes === 1 ? '' : 's'}`;
}
