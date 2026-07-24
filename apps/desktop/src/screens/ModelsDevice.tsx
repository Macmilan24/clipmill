import type { JSX } from 'react';

import type { DeviceProfile } from '@clipmill/contracts';

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
import { AlertIcon, CpuIcon, RefreshIcon, ShieldCheckIcon } from '../shell/icons.js';

interface ModelsDeviceProps {
  readonly state: ConnectionState;
  readonly profile: DeviceProfile | null;
  readonly artifactId: string | null;
  readonly error: string | null;
  readonly busy: boolean;
  readonly onRescan: () => void;
  readonly onReconnect: () => void;
}

function DeviceStrip({ profile }: { readonly profile: DeviceProfile }): JSX.Element {
  const accelerator = primaryAccelerator(profile);
  const attested = isAttested(profile);

  return (
    <section className="device-strip glass card" aria-label="Device overview">
      <div className="device-identity">
        <CpuIcon size={20} className="secondary" />
        <div>
          <div className="t-meta muted">This device</div>
          <div className="t-section-title">{profile.cpu.model}</div>
          <div className="mono secondary">{describePlatform(profile)}</div>
        </div>
      </div>

      <dl className="device-cell">
        <dt>Accelerator</dt>
        <dd>{describeAccelerator(accelerator)}</dd>
        <dd className="mono secondary">{acceleratorMemory(profile)}</dd>
      </dl>

      <dl className="device-cell">
        <dt>Memory</dt>
        <dd className="mono">{formatBytes(profile.memory.total_bytes)}</dd>
        <dd className="mono secondary">
          {profile.phase0 === undefined
            ? EM_DASH
            : `${formatBytes(profile.phase0.available_memory_bytes)} available`}
        </dd>
      </dl>

      <dl className="device-cell">
        <dt>CPU</dt>
        <dd className="mono">{describeCores(profile.cpu)}</dd>
        <dd className="mono secondary">{profile.measured.ffmpeg_build}</dd>
      </dl>

      <dl className="device-cell">
        <dt>Local models</dt>
        <dd className="mono">0 installed</dd>
        <dd className="mono secondary">arrives in Phase 1</dd>
      </dl>

      <div>
        <span className={`badge ${attested ? 'badge-success' : 'badge-warning'}`}>
          <ShieldCheckIcon />
          {attested ? 'Signed profile' : 'Unattested'}
        </span>
        <div className="mono secondary" style={{ marginTop: 6 }}>
          gen {profile.phase0?.measurement_generation ?? EM_DASH}
        </div>
      </div>
    </section>
  );
}

function CapabilityCard({ profile }: { readonly profile: DeviceProfile }): JSX.Element {
  const rows = capabilityRows(profile);

  return (
    <section className="glass card" aria-label="Measured capabilities">
      <div className="card-header">
        <div>
          <h2 className="t-section-title">Measured capabilities</h2>
          <p className="t-meta" style={{ margin: '4px 0 0' }}>
            Backends are admitted by measurement on this machine, never by a per-platform
            assumption.
          </p>
        </div>
        <span className="mono secondary">{rows.length} probed</span>
      </div>

      {rows.length === 0 ? (
        <p className="t-meta muted">This profile predates capability probing.</p>
      ) : (
        <table className="table">
          <thead>
            <tr>
              <th>Capability</th>
              <th>Backend</th>
              <th>Detail</th>
              <th>State</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((row) => (
              <tr key={`${row.capability}:${row.backend}`}>
                <td>{row.capability}</td>
                <td className="mono secondary">{row.backend}</td>
                <td className="mono muted">{row.detail ?? EM_DASH}</td>
                <td>
                  <span className={`badge ${row.available ? 'badge-success' : 'badge-warning'}`}>
                    {row.available ? 'Available' : 'Unavailable'}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

function ThroughputCard({ profile }: { readonly profile: DeviceProfile }): JSX.Element {
  const decode = profile.measured.decode ?? [];
  const roundtrip = profile.phase0?.hardware_roundtrip;
  const sharedMemory = profile.phase0?.shared_memory;

  return (
    <section className="glass card" aria-label="Measured throughput">
      <div className="card-header">
        <h2 className="t-section-title">Measured throughput</h2>
        <span className="mono secondary">{profile.measured.ffmpeg_build}</span>
      </div>

      {decode.length === 0 ? (
        <p className="t-meta muted">No decode benchmarks recorded in this profile.</p>
      ) : (
        <table className="table">
          <thead>
            <tr>
              <th>Decode</th>
              <th>Height</th>
              <th>Path</th>
              <th>Throughput</th>
            </tr>
          </thead>
          <tbody>
            {decode.map((bench) => (
              <tr key={`${bench.codec}:${bench.height}`}>
                <td className="mono">{bench.codec}</td>
                <td className="mono secondary">{bench.height}p</td>
                <td className="secondary">{bench.hardware === true ? 'Hardware' : 'Software'}</td>
                <td className="mono">{formatFps(bench.fps_measured)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      <hr className="rule" />

      <div className="card-header" style={{ marginBottom: 0 }}>
        <span className="t-meta secondary">Shared-memory transfer</span>
        <span className="mono">{formatRate(sharedMemory?.bytes_per_second)}</span>
      </div>
      <div className="card-header" style={{ marginBottom: 0, marginTop: 8 }}>
        <span className="t-meta secondary">
          Hardware round-trip{roundtrip === undefined ? '' : ` · ${roundtrip.backend}`}
        </span>
        <span className="mono">
          {roundtrip === undefined
            ? EM_DASH
            : roundtrip.available
              ? formatMilliseconds(roundtrip.milliseconds)
              : (roundtrip.unavailable_reason ?? 'unavailable')}
        </span>
      </div>
    </section>
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
    <section className="glass card" aria-label="Local Lock">
      <div className="card-header">
        <h2 className="t-section-title">
          <ShieldCheckIcon /> Local Lock
        </h2>
        <span className={`badge ${locked ? 'badge-success' : 'badge-warning'}`}>
          {connected ? (locked ? 'ON' : 'OFF') : 'UNKNOWN'}
        </span>
      </div>

      <p className="t-meta" style={{ margin: 0 }}>
        When on, ClipMill blocks model requests that would send source media, frames, transcript, or
        embeddings off-device.
      </p>

      <hr className="rule" />

      <dl style={{ margin: 0, display: 'grid', gap: 8 }}>
        <div className="card-header" style={{ margin: 0 }}>
          <dt className="t-meta secondary">Session egress</dt>
          <dd className="mono" style={{ margin: 0 }}>
            {locked ? '0 B' : EM_DASH}
          </dd>
        </div>
        <div className="card-header" style={{ margin: 0 }}>
          <dt className="t-meta secondary">Profile attestation</dt>
          <dd className="mono" style={{ margin: 0 }}>
            {isAttested(profile) ? 'ed25519' : EM_DASH}
          </dd>
        </div>
        <div className="card-header" style={{ margin: 0 }}>
          <dt className="t-meta secondary">Hardware fingerprint</dt>
          <dd className="mono" style={{ margin: 0 }}>
            {shortDigest(profile.phase0?.hardware_fingerprint)}
          </dd>
        </div>
      </dl>
    </section>
  );
}

function RuntimesCard({ profile }: { readonly profile: DeviceProfile }): JSX.Element {
  const runtimes = profile.phase0?.runtime_identities ?? [];

  return (
    <section className="glass card" aria-label="Runtimes">
      <div className="card-header">
        <h2 className="t-section-title">Runtimes</h2>
        <span className="mono secondary">{runtimes.length}</span>
      </div>
      {runtimes.length === 0 ? (
        <p className="t-meta muted">No runtimes recorded.</p>
      ) : (
        <div style={{ display: 'grid', gap: 10 }}>
          {runtimes.map((runtime) => (
            <div
              key={`${runtime.kind}:${runtime.identity}`}
              className="card-header"
              style={{ margin: 0 }}
            >
              <div>
                <div className="t-label">{runtime.kind}</div>
                <div className="mono muted">{runtime.identity}</div>
              </div>
              <span className={runtime.available ? 'success' : 'muted'}>
                {runtime.available ? 'ready' : 'absent'}
              </span>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}

function EndpointsCard(): JSX.Element {
  return (
    <section className="glass card" aria-label="External endpoints">
      <div className="card-header">
        <h2 className="t-section-title">External endpoints</h2>
        <span className="badge badge-outbound">NETWORK</span>
      </div>
      <p className="t-meta" style={{ margin: 0 }}>
        None configured. Phase 0 ships no network broker at all, so there is nothing here to enable
        yet — and enabling one will always require explicit approval.
      </p>
    </section>
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
      <div className="page-header">
        <div>
          <h1 className="t-page-title">Models &amp; Device</h1>
          <p>
            What this machine measured, what may use a connection, and how resources are shared.
          </p>
        </div>
        <div style={{ display: 'flex', gap: 8 }}>
          <button
            type="button"
            className="button"
            onClick={onRescan}
            disabled={busy || state.status !== 'connected'}
          >
            <RefreshIcon />
            {busy ? 'Measuring…' : 'Rescan hardware'}
          </button>
        </div>
      </div>

      {profile === null ? (
        <section className="glass card empty" aria-label="Device profile unavailable">
          <span className="empty-well">
            <AlertIcon size={22} />
          </span>
          <h2 className="t-card-title">
            {state.status === 'connected' ? 'No device profile yet' : 'Daemon not connected'}
          </h2>
          <p>{error ?? 'The shell will show measured hardware as soon as the daemon answers.'}</p>
          <button type="button" className="button button-primary" onClick={onReconnect}>
            <RefreshIcon />
            Retry now
          </button>
        </section>
      ) : (
        <>
          <DeviceStrip profile={profile} />
          {error === null ? null : (
            <p className="t-meta warning" style={{ marginTop: 0 }}>
              {error}
            </p>
          )}
          <div className="columns">
            <div className="stack">
              <CapabilityCard profile={profile} />
              <ThroughputCard profile={profile} />
            </div>
            <div className="stack">
              <LocalLockCard state={state} profile={profile} />
              <RuntimesCard profile={profile} />
              <EndpointsCard />
            </div>
          </div>
          <p className="mono muted" style={{ marginTop: 16 }}>
            device_profile · {shortDigest(artifactId ?? undefined)}
          </p>
        </>
      )}
    </>
  );
}
