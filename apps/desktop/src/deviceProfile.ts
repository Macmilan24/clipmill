/**
 * Presentation helpers for the measured device profile.
 *
 * Pure and separately tested: these turn daemon measurements into the strings
 * the Models screen shows, and getting a unit wrong here would quietly
 * misreport someone's hardware.
 */
import type { DeviceProfile } from '@clipmill/contracts';

const BYTE_UNITS = ['B', 'KB', 'MB', 'GB', 'TB'] as const;
const OS_LABELS: Record<DeviceProfile['platform']['os'], string> = {
  macos: 'macOS',
  linux: 'Linux',
  windows: 'Windows',
};
const ACCELERATOR_LABELS: Record<DeviceProfile['accelerators'][number]['kind'], string> = {
  metal: 'Metal',
  cuda: 'CUDA',
  vulkan: 'Vulkan',
  videotoolbox: 'VideoToolbox',
  vaapi: 'VA-API',
  cpu: 'CPU',
};

export const EM_DASH = '—';

/** Drops a trailing `.0` so 32 GB reads as "32 GB" and 12.4 GB keeps its digit. */
function trimZero(value: string): string {
  return value.endsWith('.0') ? value.slice(0, -2) : value;
}

export function formatBytes(bytes: number | undefined): string {
  if (bytes === undefined || !Number.isFinite(bytes) || bytes < 0) {
    return EM_DASH;
  }
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < BYTE_UNITS.length - 1) {
    value /= 1024;
    unit += 1;
  }
  // Gigabytes and up carry one decimal; smaller units read better as integers.
  return `${trimZero(value.toFixed(unit >= 3 ? 1 : 0))} ${BYTE_UNITS[unit]}`;
}

export function formatRate(bytesPerSecond: number | undefined): string {
  const formatted = formatBytes(bytesPerSecond);
  return formatted === EM_DASH ? EM_DASH : `${formatted}/s`;
}

export function formatFps(fps: number | undefined): string {
  if (fps === undefined || !Number.isFinite(fps)) {
    return EM_DASH;
  }
  return `${fps.toFixed(1)} fps`;
}

export function formatMilliseconds(value: number | undefined): string {
  if (value === undefined || !Number.isFinite(value)) {
    return EM_DASH;
  }
  return `${value.toFixed(1)} ms`;
}

export function describePlatform(profile: DeviceProfile): string {
  const { os, arch, os_version: version } = profile.platform;
  const name = OS_LABELS[os];
  return version === undefined ? `${name} · ${arch}` : `${name} ${version} · ${arch}`;
}

export function describeCores(cpu: DeviceProfile['cpu']): string {
  const parts = [`${cpu.logical_cores} logical`];
  if (cpu.physical_cores !== undefined) {
    parts.push(`${cpu.physical_cores} physical`);
  }
  const performance = cpu.performance_cores;
  const efficiency = cpu.efficiency_cores;
  if (performance !== undefined && efficiency !== undefined) {
    parts.push(`${performance}P + ${efficiency}E`);
  }
  return parts.join(' · ');
}

export function describeAccelerator(
  accelerator: DeviceProfile['accelerators'][number] | undefined,
): string {
  if (accelerator === undefined) {
    return EM_DASH;
  }
  const label = ACCELERATOR_LABELS[accelerator.kind];
  return `${accelerator.name} · ${label}`;
}

/**
 * The accelerator the pipeline would actually prefer: anything measured beats
 * the CPU fallback, which is only meaningful when it is all that exists.
 */
export function primaryAccelerator(
  profile: DeviceProfile,
): DeviceProfile['accelerators'][number] | undefined {
  return profile.accelerators.find((item) => item.kind !== 'cpu') ?? profile.accelerators[0];
}

/** Total VRAM when the accelerator reports it; unified memory has none of its own. */
export function acceleratorMemory(profile: DeviceProfile): string {
  const accelerator = primaryAccelerator(profile);
  if (accelerator?.vram_bytes !== undefined) {
    return formatBytes(accelerator.vram_bytes);
  }
  return profile.memory.unified === true ? 'Unified' : EM_DASH;
}

export interface CapabilityRow {
  readonly capability: string;
  readonly backend: string;
  readonly available: boolean;
  readonly detail: string | undefined;
}

export function capabilityRows(profile: DeviceProfile): readonly CapabilityRow[] {
  return (profile.phase0?.capability_results ?? []).map((result) => ({
    capability: result.capability,
    backend: result.backend,
    available: result.available,
    detail: result.detail,
  }));
}

export function isAttested(profile: DeviceProfile): boolean {
  return profile.phase0?.attestation.algorithm === 'ed25519';
}

/** Short, copyable form of a `sha256:`-prefixed digest. */
export function shortDigest(digest: string | undefined): string {
  if (digest === undefined || digest.length === 0) {
    return EM_DASH;
  }
  const bare = digest.startsWith('sha256:') ? digest.slice('sha256:'.length) : digest;
  return bare.length <= 16 ? bare : `${bare.slice(0, 12)}…${bare.slice(-4)}`;
}
