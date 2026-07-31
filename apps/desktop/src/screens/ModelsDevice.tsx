import {
  Cpu,
  Gauge,
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

import type { ConnectionState } from '../daemon/client.js';
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

/**
 * One headline number, with the one line of context that makes it mean
 * something.
 *
 * A tile rather than a chart: a single current value has no shape to plot, and a
 * one-bar bar chart is the classic way of spending a whole panel on a number.
 */
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
    <Card className="glass gap-0 rounded-2xl py-4">
      <CardContent className="px-4">
        <div className={cn('flex items-center gap-1.5 text-meta', SECONDARY)}>
          <span className="[&_svg]:size-3.5">{icon}</span>
          {label}
        </div>
        {/* Device names are as long as their vendors made them. Truncated with
            the whole string on hover beats a tile that reflows to three lines. */}
        <div
          title={value}
          className="mono mt-2 truncate text-page-title font-(--cm-weight-heading) text-[var(--cm-text-primary)]"
        >
          {value}
        </div>
        {meter === undefined ? null : (
          // A ratio against a limit is a meter, not a two-slice pie. The track is
          // the same recessed surface every other field uses.
          <div className="mt-2 h-1 overflow-hidden rounded-full bg-[var(--cm-recessed)]">
            <div
              className="h-full rounded-full bg-[var(--color-primary)]"
              style={{ width: `${Math.round(Math.min(1, Math.max(0, meter)) * 100)}%` }}
            />
          </div>
        )}
        <div
          title={typeof detail === 'string' ? detail : undefined}
          className={cn('mono mt-2 truncate text-technical', MUTED)}
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

/**
 * Measured decode throughput, one bar per codec and height.
 *
 * A bar chart because the job is comparing magnitude, and one hue because the
 * bars are the same measurement of different inputs — colouring them by value
 * would spend the identity channel re-encoding what bar length already shows.
 * Every bar carries its own number, so nothing here depends on reading a colour.
 */
const DECODE_CONFIG: ChartConfig = {
  fps: { label: 'Throughput', color: 'var(--color-primary)' },
};

function DecodeCard({ profile }: { readonly profile: DeviceProfile }): JSX.Element {
  const bars = decodeBars(profile);

  return (
    <Card className="glass rounded-2xl">
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
                Throughput is measured against the pinned FFmpeg build. This profile was taken
                without it, so there is nothing to compare — rescanning on a machine that has it
                fills this in.
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

/**
 * One capability, one tile.
 *
 * The table this replaces made a reader parse four columns to learn one thing:
 * whether the machine can do it. State is a dot and a word — never colour alone —
 * and the reason a backend was refused sits where it belongs, under the backend
 * that was refused.
 */
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
    <Card className="glass rounded-2xl">
      <CardHeader>
        <CardTitle className="flex items-center gap-1.5 text-section-title">
          <Gauge className="size-4" /> Measured capabilities
        </CardTitle>
        <span className={cn('mono text-technical', SECONDARY)}>
          {rows.length === 0 ? EM_DASH : `${ready}/${rows.length} ready`}
        </span>
      </CardHeader>
      <CardContent>
        <p className={cn('mb-3 text-meta', SECONDARY)}>
          Backends are admitted by measurement on this machine, never by a per-platform assumption.
        </p>
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
    <Card className="glass rounded-2xl">
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
          When on, ClipMill blocks model requests that would send source media, frames, transcript,
          or embeddings off-device.
        </p>
        <Separator className="my-3 bg-[var(--cm-glass-border)]" />
        <dl className="grid gap-2">
          {(
            [
              ['Profile attestation', isAttested(profile) ? 'ed25519' : EM_DASH],
              ['Hardware fingerprint', shortDigest(profile.phase0?.hardware_fingerprint)],
              ['Measurement generation', String(profile.phase0?.measurement_generation ?? EM_DASH)],
            ] as const
          ).map(([label, value]) => (
            <div key={label} className="flex items-center justify-between gap-2">
              <dt className={cn('text-meta', SECONDARY)}>{label}</dt>
              <dd className="mono truncate text-technical">{value}</dd>
            </div>
          ))}
        </dl>
      </CardContent>
    </Card>
  );
}

function RuntimesCard({ profile }: { readonly profile: DeviceProfile }): JSX.Element {
  const runtimes = profile.phase0?.runtime_identities ?? [];
  const roundtrip = profile.phase0?.hardware_roundtrip;

  return (
    <Card className="glass rounded-2xl">
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
    <>
      <div className="mb-4 flex items-start justify-between gap-4">
        <div>
          <h1 className="text-page-title font-(--cm-weight-heading) tracking-[-0.01em]">
            Models &amp; Device
          </h1>
          <p className={cn('mt-1 text-meta', SECONDARY)}>
            What this machine measured, what may use a connection, and how resources are shared.
          </p>
        </div>
        <Button variant="outline" size="sm" onClick={onRescan} disabled={busy || !connected}>
          {busy ? <Spinner /> : <RefreshCw />}
          {busy ? 'Measuring…' : 'Rescan hardware'}
        </Button>
      </div>

      {profile === null ? (
        <Empty className="glass rounded-xl" aria-label="Device profile unavailable">
          <EmptyHeader>
            <EmptyMedia variant="icon" className="glass-elevated size-14 rounded-full">
              <TriangleAlert className="size-5" />
            </EmptyMedia>
            <EmptyTitle className="text-card-title">
              {connected ? 'No device profile yet' : 'Daemon not connected'}
            </EmptyTitle>
            <EmptyDescription>
              {error ?? 'The shell will show measured hardware as soon as the daemon answers.'}
            </EmptyDescription>
          </EmptyHeader>
          <div>
            <Button onClick={onReconnect}>
              <RefreshCw />
              Retry now
            </Button>
          </div>
        </Empty>
      ) : (
        <>
          <div className="mb-4 grid grid-cols-[repeat(auto-fit,minmax(210px,1fr))] gap-4">
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
              label="Session egress"
              value={connected && state.localLock ? '0 B' : EM_DASH}
              detail={
                connected
                  ? state.localLock
                    ? 'Local Lock enforced'
                    : 'egress is permitted'
                  : 'daemon not connected'
              }
            />
          </div>

          {error === null ? null : (
            <Alert className="glass mb-4 rounded-xl">
              <TriangleAlert className="text-[var(--color-warning)]" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          <div className="grid grid-cols-[minmax(0,744fr)_minmax(0,400fr)] items-start gap-4">
            <div className="flex flex-col gap-4">
              <DecodeCard profile={profile} />
              <CapabilitiesCard profile={profile} />
            </div>
            <div className="flex flex-col gap-4">
              <LocalLockCard state={state} profile={profile} />
              <Card className="glass rounded-2xl">
                <CardHeader>
                  <CardTitle className="text-section-title">Local models</CardTitle>
                  <StatusBadge tone="neutral">0 installed</StatusBadge>
                </CardHeader>
                <CardContent>
                  <p className={cn('text-meta', SECONDARY)}>
                    Weights are pinned by the bill of materials and fetched on demand. Nothing is
                    installed on this machine yet; Phase 1 is what puts them here.
                  </p>
                </CardContent>
              </Card>
              <Card className="glass rounded-2xl">
                <CardHeader>
                  <CardTitle className="text-section-title">Shared memory</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="mono text-page-title font-(--cm-weight-heading)">
                    {formatRate(profile.phase0?.shared_memory?.bytes_per_second)}
                  </div>
                  <p className={cn('mt-1 text-meta', SECONDARY)}>
                    Measured transfer between the daemon and a worker, which is how frames move
                    without a copy through the socket.
                  </p>
                </CardContent>
              </Card>
              <RuntimesCard profile={profile} />
            </div>
          </div>

          <p className={cn('mono mt-4 text-technical', MUTED)}>
            device_profile · {shortDigest(artifactId ?? undefined)}
          </p>
        </>
      )}
    </>
  );
}
