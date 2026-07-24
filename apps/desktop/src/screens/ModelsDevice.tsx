import { Cpu, RefreshCw, ShieldCheck, TriangleAlert } from 'lucide-react';
import type { JSX } from 'react';

import type { DeviceProfile } from '@clipmill/contracts';

import { StatusBadge } from '@/components/StatusBadge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from '@/components/ui/empty';
import { Separator } from '@/components/ui/separator';
import { Spinner } from '@/components/ui/spinner';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';

import type { ConnectionState } from '../daemon/client.js';
import {
  EM_DASH,
  acceleratorMemory,
  capabilityRows,
  describeAccelerator,
  describeCores,
  describePlatform,
  formatBytes,
  formatFps,
  formatMilliseconds,
  formatRate,
  isAttested,
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

function Cell({
  label,
  value,
  detail,
}: {
  readonly label: string;
  readonly value: string;
  readonly detail: string;
}): JSX.Element {
  return (
    <div className="border-l border-[var(--cm-glass-border)] pl-4">
      <div className={`mb-1 text-meta ${MUTED}`}>{label}</div>
      <div className="mono text-body">{value}</div>
      <div className={`mono text-technical ${SECONDARY}`}>{detail}</div>
    </div>
  );
}

function DeviceStrip({ profile }: { readonly profile: DeviceProfile }): JSX.Element {
  const accelerator = primaryAccelerator(profile);
  const attested = isAttested(profile);

  return (
    <Card className="glass mb-4 rounded-xl py-4">
      <CardContent className="grid grid-cols-[248px_repeat(4,1fr)_auto] items-center gap-4 px-4">
        <div className="flex items-center gap-2.5">
          <Cpu className={`size-5 ${SECONDARY}`} />
          <div>
            <div className={`text-meta ${MUTED}`}>This device</div>
            <div className="text-section-title font-(--cm-weight-heading)">{profile.cpu.model}</div>
            <div className={`mono text-technical ${SECONDARY}`}>{describePlatform(profile)}</div>
          </div>
        </div>

        <Cell
          label="Accelerator"
          value={describeAccelerator(accelerator)}
          detail={acceleratorMemory(profile)}
        />
        <Cell
          label="Memory"
          value={formatBytes(profile.memory.total_bytes)}
          detail={
            profile.phase0 === undefined
              ? EM_DASH
              : `${formatBytes(profile.phase0.available_memory_bytes)} available`
          }
        />
        <Cell
          label="CPU"
          value={describeCores(profile.cpu)}
          detail={profile.measured.ffmpeg_build}
        />
        <Cell label="Local models" value="0 installed" detail="arrives in Phase 1" />

        <div>
          <StatusBadge tone={attested ? 'success' : 'warning'}>
            <ShieldCheck className="size-3.5" />
            {attested ? 'Signed profile' : 'Unattested'}
          </StatusBadge>
          <div className={`mono mt-1.5 text-technical ${SECONDARY}`}>
            gen {profile.phase0?.measurement_generation ?? EM_DASH}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

function CapabilityCard({ profile }: { readonly profile: DeviceProfile }): JSX.Element {
  const rows = capabilityRows(profile);

  return (
    <Card className="glass rounded-xl">
      <CardHeader>
        <CardTitle className="text-section-title">Measured capabilities</CardTitle>
        <p className={`mt-1 text-meta ${SECONDARY}`}>
          Backends are admitted by measurement on this machine, never by a per-platform assumption.
        </p>
      </CardHeader>
      <CardContent>
        {rows.length === 0 ? (
          <p className={`text-meta ${MUTED}`}>This profile predates capability probing.</p>
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Capability</TableHead>
                <TableHead>Backend</TableHead>
                <TableHead>Detail</TableHead>
                <TableHead className="text-right">State</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {rows.map((row) => (
                <TableRow key={`${row.capability}:${row.backend}`}>
                  <TableCell>{row.capability}</TableCell>
                  <TableCell className={`mono ${SECONDARY}`}>{row.backend}</TableCell>
                  <TableCell className={`mono ${MUTED}`}>{row.detail ?? EM_DASH}</TableCell>
                  <TableCell className="text-right">
                    <StatusBadge tone={row.available ? 'success' : 'warning'}>
                      {row.available ? 'Available' : 'Unavailable'}
                    </StatusBadge>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </CardContent>
    </Card>
  );
}

function ThroughputCard({ profile }: { readonly profile: DeviceProfile }): JSX.Element {
  const decode = profile.measured.decode ?? [];
  const roundtrip = profile.phase0?.hardware_roundtrip;
  const sharedMemory = profile.phase0?.shared_memory;

  return (
    <Card className="glass rounded-xl">
      <CardHeader>
        <CardTitle className="text-section-title">Measured throughput</CardTitle>
        <span className={`mono text-technical ${SECONDARY}`}>{profile.measured.ffmpeg_build}</span>
      </CardHeader>
      <CardContent>
        {decode.length === 0 ? (
          <p className={`text-meta ${MUTED}`}>No decode benchmarks recorded in this profile.</p>
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Decode</TableHead>
                <TableHead>Height</TableHead>
                <TableHead>Path</TableHead>
                <TableHead className="text-right">Throughput</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {decode.map((bench) => (
                <TableRow key={`${bench.codec}:${bench.height}`}>
                  <TableCell className="mono">{bench.codec}</TableCell>
                  <TableCell className={`mono ${SECONDARY}`}>{bench.height}p</TableCell>
                  <TableCell className={SECONDARY}>
                    {bench.hardware === true ? 'Hardware' : 'Software'}
                  </TableCell>
                  <TableCell className="mono text-right">{formatFps(bench.fps_measured)}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}

        <Separator className="my-3 bg-[var(--cm-glass-border)]" />

        <div className="flex items-center justify-between">
          <span className={`text-meta ${SECONDARY}`}>Shared-memory transfer</span>
          <span className="mono text-technical">{formatRate(sharedMemory?.bytes_per_second)}</span>
        </div>
        <div className="mt-2 flex items-center justify-between">
          <span className={`text-meta ${SECONDARY}`}>
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

function LocalLockCard({
  state,
  profile,
}: {
  readonly state: ConnectionState;
  readonly profile: DeviceProfile;
}): JSX.Element {
  const connected = state.status === 'connected';
  const locked = connected && state.localLock;

  const rows: readonly (readonly [string, string])[] = [
    ['Session egress', locked ? '0 B' : EM_DASH],
    ['Profile attestation', isAttested(profile) ? 'ed25519' : EM_DASH],
    ['Hardware fingerprint', shortDigest(profile.phase0?.hardware_fingerprint)],
  ];

  return (
    <Card className="glass rounded-xl">
      <CardHeader>
        <CardTitle className="flex items-center gap-1.5 text-section-title">
          <ShieldCheck className="size-4" /> Local Lock
        </CardTitle>
        <StatusBadge tone={connected ? (locked ? 'success' : 'warning') : 'neutral'}>
          {connected ? (locked ? 'ON' : 'OFF') : 'UNKNOWN'}
        </StatusBadge>
      </CardHeader>
      <CardContent>
        <p className={`text-meta ${SECONDARY}`}>
          When on, ClipMill blocks model requests that would send source media, frames, transcript,
          or embeddings off-device.
        </p>

        <Separator className="my-3 bg-[var(--cm-glass-border)]" />

        <dl className="grid gap-2">
          {rows.map(([label, value]) => (
            <div key={label} className="flex items-center justify-between">
              <dt className={`text-meta ${SECONDARY}`}>{label}</dt>
              <dd className="mono text-technical">{value}</dd>
            </div>
          ))}
        </dl>
      </CardContent>
    </Card>
  );
}

function RuntimesCard({ profile }: { readonly profile: DeviceProfile }): JSX.Element {
  const runtimes = profile.phase0?.runtime_identities ?? [];

  return (
    <Card className="glass rounded-xl">
      <CardHeader>
        <CardTitle className="text-section-title">Runtimes</CardTitle>
        <span className={`mono text-technical ${SECONDARY}`}>{runtimes.length}</span>
      </CardHeader>
      <CardContent>
        {runtimes.length === 0 ? (
          <p className={`text-meta ${MUTED}`}>No runtimes recorded.</p>
        ) : (
          <div className="grid gap-2.5">
            {runtimes.map((runtime) => (
              <div
                key={`${runtime.kind}:${runtime.identity}`}
                className="flex items-center justify-between gap-2"
              >
                <div className="min-w-0">
                  <div className="text-label font-(--cm-weight-label)">{runtime.kind}</div>
                  <div className={`mono truncate text-technical ${MUTED}`}>{runtime.identity}</div>
                </div>
                <span
                  className={`text-meta ${runtime.available ? 'text-[var(--color-success)]' : MUTED}`}
                >
                  {runtime.available ? 'ready' : 'absent'}
                </span>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function EndpointsCard(): JSX.Element {
  return (
    <Card className="glass rounded-xl">
      <CardHeader>
        <CardTitle className="text-section-title">External endpoints</CardTitle>
        <StatusBadge tone="outbound">NETWORK</StatusBadge>
      </CardHeader>
      <CardContent>
        <p className={`text-meta ${SECONDARY}`}>
          None configured. Phase 0 ships no network broker at all, so there is nothing here to
          enable yet — and enabling one will always require explicit approval.
        </p>
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
  return (
    <>
      <div className="mb-4 flex items-start justify-between gap-4">
        <div>
          <h1 className="text-page-title font-(--cm-weight-heading) tracking-[-0.01em]">
            Models &amp; Device
          </h1>
          <p className={`mt-1 text-meta ${SECONDARY}`}>
            What this machine measured, what may use a connection, and how resources are shared.
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={onRescan}
          disabled={busy || state.status !== 'connected'}
        >
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
              {state.status === 'connected' ? 'No device profile yet' : 'Daemon not connected'}
            </EmptyTitle>
            <EmptyDescription>
              {error ?? 'The shell will show measured hardware as soon as the daemon answers.'}
            </EmptyDescription>
          </EmptyHeader>
          <EmptyContent>
            <Button onClick={onReconnect}>
              <RefreshCw />
              Retry now
            </Button>
          </EmptyContent>
        </Empty>
      ) : (
        <>
          <DeviceStrip profile={profile} />
          {error === null ? null : (
            <Alert className="glass mb-4 rounded-xl">
              <TriangleAlert className="text-[var(--color-warning)]" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}
          <div className="grid grid-cols-[minmax(0,744fr)_minmax(0,400fr)] items-start gap-4">
            <div className="flex flex-col gap-4">
              <CapabilityCard profile={profile} />
              <ThroughputCard profile={profile} />
            </div>
            <div className="flex flex-col gap-4">
              <LocalLockCard state={state} profile={profile} />
              <RuntimesCard profile={profile} />
              <EndpointsCard />
            </div>
          </div>
          <p className={`mono mt-4 text-technical ${MUTED}`}>
            device_profile · {shortDigest(artifactId ?? undefined)}
          </p>
        </>
      )}
    </>
  );
}
