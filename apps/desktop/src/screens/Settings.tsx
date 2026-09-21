import {
  ChevronDown,
  Database,
  HardDrive,
  Lock,
  Palette,
  Plug,
  RefreshCw,
  ShieldCheck,
  ShieldOff,
  TriangleAlert,
} from 'lucide-react';
import type { JSX, ReactNode } from 'react';
import type { Theme, WorkspaceTheme } from '@clipmill/tokens';

import { StatusBadge } from '@/components/StatusBadge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Spinner } from '@/components/ui/spinner';
import { AppearancePreferences } from './AppearancePreferences.js';

import './preferences.css';

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
  readonly workspaceTheme?: WorkspaceTheme;
  readonly onWorkspaceThemeChange?: (theme: WorkspaceTheme) => void;
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
  workspaceTheme,
  onWorkspaceThemeChange,
  integrations,
}: SettingsProps): JSX.Element {
  const total =
    storage?.categories.reduce((sum, category) => sum + Math.max(0, category.bytes), 0) ?? 0;
  const appearance = theme !== undefined && onThemeChange !== undefined;
  return (
    <div className="preferences-page settings-page">
      <header className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <h1 className="workspace-title">Settings &amp; privacy</h1>
          <p className="workspace-subtitle mt-1">
            Appearance, connected accounts and local storage.
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
        </nav>
        <div className="settings-sections min-w-0">
          {appearance && (
            <section id="settings-appearance" className="scroll-mt-6" aria-label="Appearance">
              <Card className="preference-section gap-0 overflow-hidden py-0">
                <SectionHeading
                  icon={<Palette />}
                  title="Appearance"
                  detail="Make the workspace feel like yours."
                />
                <CardContent className="px-5 py-5">
                  <AppearancePreferences
                    theme={theme}
                    onThemeChange={onThemeChange}
                    {...(workspaceTheme === undefined ? {} : { workspaceTheme })}
                    {...(onWorkspaceThemeChange === undefined ? {} : { onWorkspaceThemeChange })}
                  />
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
            <Card className="preference-section gap-0 overflow-hidden py-0">
              <SectionHeading
                icon={<ShieldCheck />}
                title="Privacy & cloud"
                detail="Local processing is the default. Cloud AI requires your consent."
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
                    <details className="preference-disclosure mt-4">
                      <summary>
                        <ChevronDown className="size-3.5" /> Network activity details
                      </summary>
                      <dl className="mt-3 grid grid-cols-3 gap-3 rounded-lg bg-[var(--cm-recessed)] p-3.5">
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
                        These counts describe task execution, not measured network traffic.
                        Installed cloud stages are only used when enabled.
                      </p>
                    </details>
                  </>
                )}
              </CardContent>
            </Card>
          </section>
          <section id="settings-storage" className="scroll-mt-6" aria-label="Storage">
            <Card className="preference-section gap-0 overflow-hidden py-0">
              <SectionHeading
                icon={<HardDrive />}
                title="Storage"
                detail="Files managed on this device."
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
                            </div>
                            <div className="shrink-0 text-right">
                              <p className="font-mono text-xs">{formatBytes(category.bytes)}</p>
                              <p className="mt-1 text-[11px] text-[var(--cm-text-muted)]">
                                {category.items} {category.items === 1 ? 'item' : 'items'}
                              </p>
                            </div>
                          </div>
                          <details className="preference-disclosure mt-2">
                            <summary>
                              <ChevronDown className="size-3" />
                              Location and details
                              <span className="sr-only">
                                {' '}
                                for {CATEGORY_LABELS[category.key] ?? category.key}
                              </span>
                            </summary>
                            <p className="mt-3 max-w-[530px] text-[11px] leading-relaxed text-[var(--cm-text-secondary)]">
                              {CATEGORY_NOTES[category.key] ?? 'Files managed by the local engine.'}
                            </p>
                            <p className="mt-2 select-all break-all rounded-md bg-[var(--cm-recessed)] px-2.5 py-2 font-mono text-[11px] leading-relaxed text-[var(--cm-text-muted)]">
                              {category.path}
                            </p>
                          </details>
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
                          Unused generated files become eligible for cleanup after this period.
                          Source recordings and saved edits are kept.
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
      <dt className="mt-1 max-w-[155px] text-[11px] leading-relaxed text-[var(--cm-text-secondary)]">
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
