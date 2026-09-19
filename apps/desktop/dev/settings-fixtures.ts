/** Synthetic device data for browser styling checks. Never imported by the app entrypoint. */
import type { DeviceProfile } from '@clipmill/contracts';
import type { LocalLock, Readiness, StorageStats } from '../src/daemon/client.js';
export const device: DeviceProfile = {
  schema_version: 'clipmill.device_profile.v1',
  platform: { os: 'macos', arch: 'arm64', os_version: '15.5' },
  cpu: { model: 'Apple M2 Pro', logical_cores: 12, physical_cores: 12 },
  memory: { total_bytes: 34_359_738_368, unified: true },
  accelerators: [{ kind: 'metal', name: 'Apple M2 Pro GPU' }],
  measured: {
    ffmpeg_build: 'ffmpeg-8.1.2',
    decode: [
      { codec: 'h264', height: 1080, fps_measured: 412.5, hardware: true },
      { codec: 'hevc', height: 1080, fps_measured: 368.2, hardware: true },
      { codec: 'hevc', height: 2160, fps_measured: 136.4, hardware: true },
    ],
  },
  phase0: {
    hardware_fingerprint: `sha256:${'a'.repeat(64)}`,
    measurement_generation: 2,
    available_memory_bytes: 8_138_072_064,
    runtime_identities: [
      { kind: 'ffprobe', identity: 'ffprobe-8.1.2', available: true },
      { kind: 'mlx', identity: 'mlx-0.29.3', available: true },
    ],
    capability_results: [
      { capability: 'decode', backend: 'videotoolbox', available: true, detail: 'h264, hevc' },
      { capability: 'encode', backend: 'videotoolbox', available: true, detail: 'h264' },
    ],
    shared_memory: { sample_bytes: 1024, bytes_per_second: 1_073_741_824 },
    hardware_roundtrip: { backend: 'videotoolbox', available: true, milliseconds: 4.25 },
    attestation: { algorithm: 'ed25519', public_key: 'a'.repeat(64), signature: 'b'.repeat(128) },
  },
};
export const readiness: Readiness = {
  ready: true,
  decoderPresent: true,
  decoderPath: '/preview/bin/ffmpeg',
  workers: [
    {
      workerId: 'editorial',
      family: 'editorial',
      capabilities: ['propose', 'review'],
      backend: 'mlx',
      sinceUnixMillis: 1,
    },
  ],
  stages: [
    ['editorial-propose', 'Qwen3.5-9B · 4-bit', 'mlx'],
    ['editorial-review', 'Qwen3.5-9B · 4-bit', 'mlx'],
    ['editorial-look', 'Qwen3.5-9B · 4-bit', 'mlx'],
    ['speech-asr', 'Whisper large-v3', 'mlx'],
    ['speech-align', 'Wav2Vec2 aligner', 'pytorch'],
    ['speech-vad', 'Silero VAD', 'onnx'],
  ].map(([stage, model, backend]) => ({
    stage: stage!,
    model: model!,
    backend: backend!,
    modelPresent: true,
    workerPresent: true,
    ready: true,
    capability: stage!,
    implementation: `${stage}@1.0.0`,
    missingFiles: [],
    remedy: '',
  })),
};
export const storage: StorageStats = {
  categories: [
    {
      key: 'artifacts',
      bytes: 20_078_895_104,
      items: 412,
      path: '/Users/demo/Library/Application Support/dev.clipmill.ClipMill/artifacts',
    },
    {
      key: 'models',
      bytes: 24_588_615_680,
      items: 6,
      path: '/Users/demo/Library/Application Support/dev.clipmill.ClipMill/models',
    },
    {
      key: 'state',
      bytes: 43_008_000,
      items: 4,
      path: '/Users/demo/Library/Application Support/dev.clipmill.ClipMill/state',
    },
  ],
  availableBytes: 335_007_449_088,
  retentionGraceSeconds: 604_800,
};
export const lock: LocalLock = {
  engaged: true,
  stages: 28,
  networkAllowedStages: 2,
  egressAttempts: 0,
};
