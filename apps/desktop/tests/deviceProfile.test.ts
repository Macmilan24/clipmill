import type { DeviceProfile } from '@clipmill/contracts';
import { describe, expect, it } from 'vitest';

import {
  EM_DASH,
  acceleratorMemory,
  capabilityRows,
  describeAccelerator,
  describeCores,
  describePlatform,
  formatBytes,
  formatFps,
  formatRate,
  isAttested,
  primaryAccelerator,
  shortDigest,
} from '../src/deviceProfile.js';
import { NAV_SECTIONS, findSection } from '../src/shell/navigation.js';

const profile: DeviceProfile = {
  schema_version: 'clipmill.device_profile.v1',
  platform: { os: 'macos', arch: 'arm64', os_version: '15.5' },
  cpu: {
    model: 'Apple M2 Pro',
    logical_cores: 12,
    physical_cores: 12,
    performance_cores: 8,
    efficiency_cores: 4,
  },
  memory: { total_bytes: 34_359_738_368, unified: true },
  accelerators: [
    { kind: 'cpu', name: 'Apple M2 Pro' },
    { kind: 'metal', name: 'Apple M2 Pro GPU' },
  ],
  measured: {
    ffmpeg_build: 'ffmpeg-8.1.2-btb-n8.1.2',
    decode: [{ codec: 'h264', height: 1080, fps_measured: 412.5, hardware: true }],
  },
};

describe('formatBytes', () => {
  it('renders whole gigabytes without a trailing zero', () => {
    expect(formatBytes(34_359_738_368)).toBe('32 GB');
  });

  it('keeps one decimal for fractional gigabytes', () => {
    expect(formatBytes(13_314_398_617)).toBe('12.4 GB');
  });

  it('uses integers below the gigabyte', () => {
    expect(formatBytes(574_619_648)).toBe('548 MB');
  });

  it('refuses to invent a number it does not have', () => {
    expect(formatBytes(undefined)).toBe(EM_DASH);
    expect(formatBytes(Number.NaN)).toBe(EM_DASH);
    expect(formatBytes(-1)).toBe(EM_DASH);
  });

  it('formats rates and frame rates', () => {
    expect(formatRate(1_073_741_824)).toBe('1 GB/s');
    expect(formatFps(412.5)).toBe('412.5 fps');
    expect(formatFps(undefined)).toBe(EM_DASH);
  });
});

describe('device description', () => {
  it('includes the OS version when the daemon reported one', () => {
    expect(describePlatform(profile)).toBe('macOS 15.5 · arm64');
  });

  it('omits the version rather than printing undefined', () => {
    const bare: DeviceProfile = {
      ...profile,
      platform: { os: 'linux', arch: 'x86_64' },
    };
    expect(describePlatform(bare)).toBe('Linux · x86_64');
  });

  it('describes hybrid core layouts', () => {
    expect(describeCores(profile.cpu)).toBe('12 logical · 12 physical · 8P + 4E');
  });

  it('prefers a real accelerator over the CPU fallback', () => {
    expect(primaryAccelerator(profile)?.kind).toBe('metal');
    expect(describeAccelerator(primaryAccelerator(profile))).toBe('Apple M2 Pro GPU · Metal');
  });

  it('reports unified memory when the GPU has no VRAM of its own', () => {
    expect(acceleratorMemory(profile)).toBe('Unified');
  });

  it('reports discrete VRAM when present', () => {
    const discrete: DeviceProfile = {
      ...profile,
      memory: { total_bytes: 34_359_738_368 },
      accelerators: [{ kind: 'cuda', name: 'RTX 4070', vram_bytes: 12_884_901_888 }],
    };
    expect(acceleratorMemory(discrete)).toBe('12 GB');
  });
});

describe('phase0 extension', () => {
  it('treats a profile without the extension as unattested', () => {
    expect(isAttested(profile)).toBe(false);
    expect(capabilityRows(profile)).toEqual([]);
  });

  it('shortens digests without losing the ends', () => {
    const digest = `sha256:${'a1b2c3d4'.repeat(8)}`;
    expect(shortDigest(digest)).toBe('a1b2c3d4a1b2…c3d4');
    expect(shortDigest(undefined)).toBe(EM_DASH);
  });
});

describe('navigation', () => {
  it('matches the design order exactly', () => {
    expect(NAV_SECTIONS.map((section) => section.label)).toEqual([
      'Library',
      'New Project',
      'Results',
      'Editor',
      'Discovery',
      'Brand',
      'Models',
      'Export',
      'Settings',
    ]);
  });

  // Which sections are live is the one thing on this list that changes as the
  // shell is built, so the test names them rather than counting them: a section
  // going live should be a deliberate edit here, not a number going up.
  it('marks a section live only when a screen answers for it', () => {
    const live = NAV_SECTIONS.filter((section) => section.availability.kind === 'live');
    expect(live.map((section) => section.id)).toEqual([
      'library',
      'new-project',
      'results',
      'editor',
      'models',
      'export',
      'settings',
    ]);
  });

  it('gives every section that is not live the phase that will build it', () => {
    for (const section of NAV_SECTIONS) {
      if (section.availability.kind === 'planned') {
        expect(section.availability.phase).toBeGreaterThan(0);
        expect(section.availability.summary.length).toBeGreaterThan(0);
      }
    }
  });

  it('falls back to a real section for an unknown id', () => {
    expect(findSection('nope').id).toBe('models');
  });
});
