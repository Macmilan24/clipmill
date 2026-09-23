import {
  ChevronDown,
  Cpu,
  Gauge,
  Laptop,
  MemoryStick,
  RefreshCw,
  ShieldCheck,
  ShieldOff,
  TriangleAlert,
  Zap,
} from 'lucide-react';
import type { JSX, ReactNode } from 'react';
import { Bar, BarChart, Tooltip, XAxis, YAxis } from 'recharts';

import type { DeviceProfile } from '@clipmill/contracts';

import { StatusBadge } from '@/components/StatusBadge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { type ChartConfig, ChartContainer, ChartTooltipContent } from '@/components/ui/chart';
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from '@/components/ui/empty';
import { Separator } from '@/components/ui/separator';
import { Spinner } from '@/components/ui/spinner';
import { cn } from '@/lib/utils';

import './preferences.css';

import { ModelLibraryPanel } from '../models/ModelLibraryPanel.js';
import type { ConnectionState } from '../daemon/client.js';
import { type ShellApi, daemonApi } from '../daemon/api.js';
import {
  EM_DASH,
  acceleratorMemory,
  capabilityRows,
  decodeBars,
  describeAccelerator,
  describeCores,
  describePlatform,
  formatBytes,
  formatFps,
  formatMilliseconds,
  formatRate,
  isAttested,
  memoryUse,
  primaryAccelerator,
  shortDigest,
} from '../deviceProfile.js';

interface ModelsDeviceProps {
  readonly api?: ShellApi;
  readonly state: ConnectionState;
  readonly profile: DeviceProfile | null;
  readonly artifactId: string | null;
  readonly error: string | null;
  readonly busy: boolean;
  readonly onRescan: () => void;
  readonly onReconnect: () => void;
}

const MUTED = 'text-[var(--cm-text-muted)]';
const SECONDARY = 'text-[var(--cm-text-secondary)]';

function Stat({
  icon,
  label,
  value,
  detail,
  meter,
}: {
  readonly icon: ReactNode;
  readonly label: string;
  readonly value: string;
  readonly detail: ReactNode;
  /** 0–1, when the value is a ratio against a limit. */
  readonly meter?: number;
}): JSX.Element {
  return (
    <Card className="device-fact">
      <CardContent className="device-fact-content">
        <div className={cn('flex items-center gap-1.5 text-meta', SECONDARY)}>
          <span className="[&_svg]:size-3.5">{icon}</span>
          {label}
        </div>
        <div title={value} className="device-fact-value">
          {value}
        </div>
        {meter === undefined ? null : (
          <div className="mt-2 h-1 overflow-hidden rounded-full bg-[var(--cm-recessed)]">
            <div
              className="h-full rounded-full bg-[var(--color-primary)]"
              style={{ width: `${Math.round(Math.min(1, Math.max(0, meter)) * 100)}%` }}
            />
          </div>
        )}
        <div
          title={typeof detail === 'string' ? detail : undefined}
          className={cn('mt-1 text-technical', MUTED)}
        >
          {detail}
        </div>
      </CardContent>
    </Card>
  );
}

/** Memory is a ratio when availability was measured, and a total when it was not. */
function MemoryStat({ profile }: { readonly profile: DeviceProfile }): JSX.Element {
  const { label, ratio } = memoryUse(profile);
  const detail = `${formatBytes(profile.memory.total_bytes)} total`;
  return ratio === undefined ? (
    <Stat icon={<MemoryStick />} label="Memory" value={label} detail={detail} />
  ) : (
    <Stat
      icon={<MemoryStick />}
      label="Memory in use"
      value={label}
      detail={detail}
      meter={ratio}
    />
  );
}

const DECODE_CONFIG: ChartConfig = {
  fps: { label: 'Throughput', color: 'var(--color-primary)' },
};

function DecodeCard({ profile }: { readonly profile: DeviceProfile }): JSX.Element {
  const bars = decodeBars(profile);

  return (
    <Card className="glass rounded-xl">
      <CardHeader>
        <CardTitle className="flex items-center gap-1.5 text-section-title">
          <Zap className="size-4" /> Decode throughput
        </CardTitle>
        <span className={cn('mono text-technical', SECONDARY)}>
          fps · {profile.measured.ffmpeg_build}
        </span>
      </CardHeader>
      <CardContent>
        {bars.length === 0 ? (
          <Empty className="py-8" aria-label="No decode benchmarks">
            <EmptyHeader>
              <EmptyTitle className="text-body">No decode benchmarks in this profile</EmptyTitle>
              <EmptyDescription>
                Install the pinned FFmpeg build, then rescan to measure decode speed.
              </EmptyDescription>
            </EmptyHeader>
          </Empty>
        ) : (
          <>
            {/* The bars carry their own numbers on screen. This is the same list
              for a reader who cannot see them — identity and value never depend
              on the mark alone. */}
            <ul className="sr-only">
              {bars.map((bar) => (
                <li key={bar.label}>
                  {bar.label}: {formatFps(bar.fps)}
                </li>
              ))}
            </ul>
            <ChartContainer
              config={DECODE_CONFIG}
              role="img"
              aria-label={`Decode throughput for ${bars.length} measured paths`}
              className="h-[var(--chart-height)]"
              style={{ '--chart-height': `${bars.length * 34 + 24}px` } as React.CSSProperties}
            >
              <BarChart
                data={[...bars]}
                layout="vertical"
                margin={{ left: 0, right: 56, top: 4, bottom: 4 }}
                barCategoryGap={6}
              >
                <XAxis type="number" hide domain={[0, 'dataMax']} />
                <YAxis
                  type="category"
                  dataKey="label"
                  // Wide enough for "hevc 2160p · hw"; a narrower axis clips the
                  // first characters rather than the last, which reads as a bug.
                  width={132}
                  tickLine={false}
                  axisLine={false}
                />
                <Tooltip
                  cursor={{ fill: 'var(--cm-accent-selected)' }}
                  content={({ active, payload, label }) => (
                    <ChartTooltipContent
                      active={active}
                      payload={payload}
                      label={label as ReactNode}
                      config={DECODE_CONFIG}
                      unit="fps"
                    />
                  )}
                />
                {/* Data-ends rounded, baseline square: the bar still starts at zero. */}
                <Bar
                  dataKey="fps"
                  fill="var(--color-fps)"
                  radius={[0, 4, 4, 0]}
                  barSize={14}
                  // No entry animation: this is a measurement readout, and the
                  // labels only appear once the animation settles, so an animated
                  // one is a chart that is briefly wrong.
                  isAnimationActive={false}
                  // Direct-labelled, which is what keeps the value off the colour
                  // channel: the number is readable without reading the bar.
                  //
                  // Position only. The label config is handed to the library's own
                  // Label, which renders nothing at all when given a prop it does
                  // not know — so the type and colour are applied as CSS on the
                  // container, beside the rest of the chart's styling. The unit
                  // sits in the card header, said once rather than on every bar.
                  label={{ position: 'right', offset: 8 }}
                />
              </BarChart>
            </ChartContainer>
          </>
        )}
      </CardContent>
    </Card>
  );
}

function CapabilityTile({
  capability,
  backend,
  detail,
  available,
}: {
  readonly capability: string;
  readonly backend: string;
  readonly detail: string | undefined;
  readonly available: boolean;
}): JSX.Element {
  return (
    <div className="rounded-[var(--cm-radius-panel)] border border-[var(--cm-glass-border)] p-3">
      <div className="flex items-start justify-between gap-2">
        <span className="min-w-0">
          <span className="block truncate text-body font-(--cm-weight-label)">{capability}</span>
          <span className={cn('mono block truncate text-technical', SECONDARY)}>{backend}</span>
        </span>
        <StatusBadge tone={available ? 'success' : 'warning'}>
          {available ? 'Ready' : 'Unavailable'}
        </StatusBadge>
      </div>
      {detail === undefined || detail === '' ? null : (
        <p className={cn('mt-2 line-clamp-2 text-meta', MUTED)}>{detail}</p>
      )}
    </div>
  );
}

function CapabilitiesCard({ profile }: { readonly profile: DeviceProfile }): JSX.Element {
  const rows = capabilityRows(profile);
  const ready = rows.filter((row) => row.available).length;

  return (
    <Card className="glass rounded-xl">
      <CardHeader>
        <CardTitle className="flex items-center gap-1.5 text-section-title">
          <Gauge className="size-4" /> Measured capabilities
        </CardTitle>
        <span className={cn('mono text-technical', SECONDARY)}>
          {rows.length === 0 ? EM_DASH : `${ready}/${rows.length} ready`}
        </span>
      </CardHeader>
      <CardContent>
        {rows.length === 0 ? (
          <p className={cn('text-meta', MUTED)}>This profile predates capability probing.</p>
        ) : (
          <div className="grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-2">
            {rows.map((row) => (
              <CapabilityTile
                key={`${row.capability}:${row.backend}`}
                capability={row.capability}
                backend={row.backend}
                detail={row.detail}
                available={row.available}
              />
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function LocalLockCard({
  state,
  profile,
}: {
  readonly state: ConnectionState;
  readonly profile: DeviceProfile;
}): JSX.Element {
  const connected = state.status === 'connected';
  const locked = connected && state.localLock;

  return (
    <Card className="glass rounded-xl">
      <CardHeader>
        <CardTitle className="flex items-center gap-1.5 text-section-title">
          {locked ? <ShieldCheck className="size-4" /> : <ShieldOff className="size-4" />} Local
          Lock
        </CardTitle>
        <StatusBadge tone={connected ? (locked ? 'success' : 'warning') : 'neutral'}>
          {connected ? (locked ? 'ON' : 'OFF') : 'UNKNOWN'}
        </StatusBadge>
      </CardHeader>
      <CardContent>
        <p className={cn('text-meta', SECONDARY)}>
          {locked
            ? 'No network operations have started in this engine session. Local analysis keeps source media, frames and transcripts on this device.'
            : connected
              ? 'Network operations have started in this engine session. This includes YouTube imports, channel sign-in, publishing and explicitly enabled cloud analysis; importing a video does not enable cloud AI.'
              : 'Reconnect to check whether network operations have started in this engine session.'}
        </p>
        <details className="preference-disclosure mt-3">
          <summary>
            <ChevronDown className="size-3.5" /> Privacy and attestation details
          </summary>
          <p className={cn('mt-3 text-[11px] leading-relaxed', MUTED)}>
            Records task execution, not measured traffic or a network firewall. Model downloads and
            Hugging Face look-ups started above count as network operations.
          </p>
          <dl className="mt-3 grid gap-2">
            {(
              [
                ['Profile attestation', isAttested(profile) ? 'ed25519' : EM_DASH],
                ['Hardware fingerprint', shortDigest(profile.phase0?.hardware_fingerprint)],
                [
                  'Measurement generation',
                  String(profile.phase0?.measurement_generation ?? EM_DASH),
                ],
              ] as const
            ).map(([label, value]) => (
              <div key={label} className="flex items-center justify-between gap-2">
                <dt className={cn('text-meta', SECONDARY)}>{label}</dt>
                <dd className="mono truncate text-technical">{value}</dd>
              </div>
            ))}
          </dl>
        </details>
      </CardContent>
    </Card>
  );
}

function RuntimesCard({ profile }: { readonly profile: DeviceProfile }): JSX.Element {
  const runtimes = profile.phase0?.runtime_identities ?? [];
  const roundtrip = profile.phase0?.hardware_roundtrip;

  return (
    <Card className="glass rounded-xl">
      <CardHeader>
        <CardTitle className="text-section-title">Runtimes</CardTitle>
        <span className={cn('mono text-technical', SECONDARY)}>{runtimes.length}</span>
      </CardHeader>
      <CardContent>
        {runtimes.length === 0 ? (
          <p className={cn('text-meta', MUTED)}>No runtimes recorded.</p>
        ) : (
          <div className="grid gap-2">
            {runtimes.map((runtime) => (
              <div
                key={`${runtime.kind}:${runtime.identity}`}
                className="flex items-center justify-between gap-2"
              >
                <span className="min-w-0">
                  <span className="block text-label font-(--cm-weight-label)">{runtime.kind}</span>
                  <span className={cn('mono block truncate text-technical', MUTED)}>
                    {runtime.identity}
                  </span>
                </span>
                <StatusBadge tone={runtime.available ? 'success' : 'warning'}>
                  {runtime.available ? 'Ready' : 'Absent'}
                </StatusBadge>
              </div>
            ))}
          </div>
        )}

        <Separator className="my-3 bg-[var(--cm-glass-border)]" />

        <div className="flex items-center justify-between gap-2">
          <span className={cn('text-meta', SECONDARY)}>
            Hardware round-trip{roundtrip === undefined ? '' : ` · ${roundtrip.backend}`}
          </span>
          <span className="mono text-technical">
            {roundtrip === undefined
              ? EM_DASH
              : roundtrip.available
                ? formatMilliseconds(roundtrip.milliseconds)
                : (roundtrip.unavailable_reason ?? 'unavailable')}
          </span>
        </div>
      </CardContent>
    </Card>
  );
}

export function ModelsDevice({
  api = daemonApi,
  state,
  profile,
  artifactId,
  error,
  busy,
  onRescan,
  onReconnect,
}: ModelsDeviceProps): JSX.Element {
  const connected = state.status === 'connected';

  return (
    <div className="preferences-page models-page">
      <header className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <h1 className="workspace-title">Models &amp; Device</h1>
          <p className="workspace-subtitle mt-1 max-w-[620px]">
            Download, choose and remove the models ClipMill runs, and check your device.
          </p>
        </div>
        <StatusBadge tone={connected ? 'success' : 'warning'}>
          <span className="size-1.5 rounded-full bg-current" />
          {connected ? 'Engine connected' : 'Engine disconnected'}
        </StatusBadge>
      </header>
      {connected && <ModelLibraryPanel api={api} />}
      <section aria-labelledby="device-heading" className="space-y-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h2 id="device-heading" className="flex items-center gap-2 text-sm font-semibold">
              <Laptop className="size-4 text-[var(--cm-text-secondary)]" />
              Your device
            </h2>
            <p className="mt-1 text-xs text-[var(--cm-text-secondary)]">
              Hardware and available memory at the last scan.
            </p>
          </div>
          <Button variant="outline" size="sm" onClick={onRescan} disabled={busy || !connected}>
            {busy ? <Spinner /> : <RefreshCw />}
            {busy ? 'Measuring…' : 'Rescan hardware'}
          </Button>
        </div>
        {profile === null ? (
          <Empty
            className="rounded-xl border border-[var(--cm-glass-border)] bg-[var(--cm-glass)]"
            aria-label="Device profile unavailable"
          >
            <EmptyHeader>
              <EmptyMedia variant="icon">
                <TriangleAlert className="size-5" />
              </EmptyMedia>
              <EmptyTitle className="text-card-title">
                {connected ? 'No device profile yet' : 'Daemon not connected'}
              </EmptyTitle>
              <EmptyDescription>
                {error ?? 'Hardware measurements appear as soon as the engine answers.'}
              </EmptyDescription>
            </EmptyHeader>
            <Button onClick={onReconnect}>
              <RefreshCw />
              Retry now
            </Button>
          </Empty>
        ) : (
          <>
            {!connected && (
              <Alert>
                <TriangleAlert />
                <AlertDescription>
                  Showing the last device profile. Reconnect to verify current readiness.
                </AlertDescription>
                <Button
                  className="col-start-2 mt-2 w-fit"
                  variant="outline"
                  size="sm"
                  onClick={onReconnect}
                >
                  Reconnect engine
                </Button>
              </Alert>
            )}
            {error !== null && (
              <Alert variant="destructive">
                <TriangleAlert />
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}
            <div className="device-facts">
              <Stat
                icon={<Zap />}
                label="Accelerator"
                value={describeAccelerator(primaryAccelerator(profile))}
                detail={acceleratorMemory(profile)}
              />
              <MemoryStat profile={profile} />
              <Stat
                icon={<Cpu />}
                label="CPU"
                value={profile.cpu.model}
                detail={`${describeCores(profile.cpu)} · ${describePlatform(profile)}`}
              />
              <Stat
                icon={<ShieldCheck />}
                label="Network operations"
                value={connected ? (state.localLock ? 'Unused' : 'Used') : EM_DASH}
                detail={
                  connected
                    ? state.localLock
                      ? 'No network operations this session'
                      : 'Network used this session'
                    : 'daemon not connected'
                }
              />
            </div>
            <LocalLockCard state={state} profile={profile} />
            <details className="preference-disclosure device-measurements">
              <summary>
                <ChevronDown className="size-4" />
                <span>Device measurements</span>
                <span className="preference-disclosure-hint">
                  Performance, capabilities and runtimes
                </span>
              </summary>
              <div className="mt-4 grid items-start gap-4 xl:grid-cols-[minmax(0,1.4fr)_minmax(0,1fr)]">
                <div className="min-w-0 space-y-4">
                  <DecodeCard profile={profile} />
                  <CapabilitiesCard profile={profile} />
                </div>
                <div className="min-w-0 space-y-4">
                  <Card className="rounded-xl">
                    <CardHeader>
                      <CardTitle className="text-section-title">Shared memory</CardTitle>
                    </CardHeader>
                    <CardContent>
                      <div className="mono text-page-title font-(--cm-weight-heading)">
                        {formatRate(profile.phase0?.shared_memory?.bytes_per_second)}
                      </div>
                      <p className={cn('mt-1 text-meta leading-relaxed', SECONDARY)}>
                        Measured transfer speed between the engine and a worker.
                      </p>
                    </CardContent>
                  </Card>
                  <RuntimesCard profile={profile} />
                </div>
              </div>
              <div className="mt-4 flex flex-wrap items-center justify-between gap-2 border-t border-[var(--cm-glass-border)] pt-3 text-[11px] text-[var(--cm-text-muted)]">
                <p>Measurements describe this profile, not live resource use.</p>
                <p className="mono" title={artifactId ?? undefined}>
                  device_profile · {shortDigest(artifactId ?? undefined)}
                </p>
              </div>
            </details>
          </>
        )}
      </section>
    </div>
  );
}
