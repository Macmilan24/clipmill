import type { DeviceProfile } from '@clipmill/contracts';
import { render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import { TooltipProvider } from '@/components/ui/tooltip';

import { ModelsDevice } from '../src/screens/ModelsDevice.js';
import type { ConnectionState } from '../src/daemon/client.js';

const profile: DeviceProfile = {
  schema_version: 'clipmill.device_profile.v1',
  platform: { os: 'macos', arch: 'arm64', os_version: '15.5' },
  cpu: { model: 'Apple M2 Pro', logical_cores: 12, physical_cores: 12 },
  memory: { total_bytes: 34_359_738_368, unified: true },
  accelerators: [{ kind: 'metal', name: 'Apple M2 Pro GPU' }],
  measured: {
    ffmpeg_build: 'ffmpeg-8.1.2-btb-n8.1.2',
    decode: [{ codec: 'h264', height: 1080, fps_measured: 412.5, hardware: true }],
  },
  phase0: {
    hardware_fingerprint: `sha256:${'0'.repeat(64)}`,
    measurement_generation: 1,
    available_memory_bytes: 8_138_072_064,
    runtime_identities: [{ kind: 'ffprobe', identity: 'ffprobe-8.1.2', available: true }],
    capability_results: [
      { capability: 'decode', backend: 'videotoolbox', available: true, detail: 'h264' },
      { capability: 'encode', backend: 'nvenc', available: false },
    ],
    shared_memory: { sample_bytes: 1024, bytes_per_second: 1_073_741_824 },
    hardware_roundtrip: { backend: 'videotoolbox', available: true, milliseconds: 4.25 },
    attestation: {
      algorithm: 'ed25519',
      public_key: 'a'.repeat(64),
      signature: 'b'.repeat(128),
    },
  },
};

const connected: ConnectionState = {
  status: 'connected',
  daemonVersion: '0.0.1',
  localLock: true,
  startedUnixMillis: 1,
};

function renderScreen(state: ConnectionState, value: DeviceProfile | null) {
  return render(
    <TooltipProvider>
      <ModelsDevice
        state={state}
        profile={value}
        artifactId={`sha256:${'c'.repeat(64)}`}
        error={null}
        busy={false}
        onRescan={() => undefined}
        onReconnect={() => undefined}
      />
    </TooltipProvider>,
  );
}

afterEach(() => {
  document.body.innerHTML = '';
});

describe('Models & Device', () => {
  it('reports measured hardware rather than placeholders', () => {
    renderScreen(connected, profile);

    expect(screen.getByText('Apple M2 Pro')).toBeDefined();
    expect(screen.getByText('macOS 15.5 · arm64')).toBeDefined();
    expect(screen.getByText('32 GB')).toBeDefined();
    expect(screen.getByText('7.6 GB available')).toBeDefined();
    expect(screen.getByText('Apple M2 Pro GPU · Metal')).toBeDefined();
    // Unified memory has no VRAM of its own; it must not read as an em dash.
    expect(screen.getByText('Unified')).toBeDefined();
  });

  it('renders probed capabilities with their availability', () => {
    renderScreen(connected, profile);

    expect(screen.getByText('videotoolbox')).toBeDefined();
    expect(screen.getByText('nvenc')).toBeDefined();
    expect(screen.getByText('Available')).toBeDefined();
    expect(screen.getByText('Unavailable')).toBeDefined();
  });

  it('renders measured throughput in the technical face', () => {
    renderScreen(connected, profile);

    expect(screen.getByText('412.5 fps')).toBeDefined();
    expect(screen.getByText('1 GB/s')).toBeDefined();
    expect(screen.getByText('4.3 ms')).toBeDefined();
  });

  it('shows Local Lock as ON only when the daemon says so', () => {
    renderScreen(connected, profile);
    expect(screen.getByText('ON')).toBeDefined();
  });

  it('reports Local Lock as unknown when the daemon is unreachable', () => {
    // The badge must never claim a guarantee that nobody checked.
    renderScreen({ status: 'disconnected', reason: 'gone' }, profile);
    expect(screen.getByText('UNKNOWN')).toBeDefined();
    expect(screen.queryByText('ON')).toBeNull();
  });

  it('never presents installed models it does not have', () => {
    renderScreen(connected, profile);
    expect(screen.getByText('0 installed')).toBeDefined();
  });

  it('falls back to an empty state without a profile', () => {
    renderScreen({ status: 'disconnected', reason: 'gone' }, null);
    expect(screen.getByText('Daemon not connected')).toBeDefined();
    expect(screen.getByText('Retry now')).toBeDefined();
  });
});
