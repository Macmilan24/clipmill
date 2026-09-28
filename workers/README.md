# Authenticated external workers

Python model and media workers use separate uv projects and locked environments
for each family. Dependencies stay isolated from other worker families and the
daemon. Workers accept leased tasks, send heartbeats, and return artifacts;
`clipmilld` owns durable job and artifact state. The optional cloud adapter also
keeps a local, run-scoped budget ledger to account for external requests.

| Directory         | Purpose                                                                                                                        |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `sdk/`            | Shared protocol client, generated contract types, and lease-scoped input and output helpers.                                   |
| `echo/`           | Reference worker for protocol integration tests.                                                                               |
| `vad/`            | Silero VAD on ONNX Runtime for speech and silence intervals.                                                                   |
| `asr-whispercpp/` | Portable CPU speech recognition with whisper.cpp.                                                                              |
| `align/`          | Portable wav2vec2 CTC forced alignment.                                                                                        |
| `speakers/`       | CAM++ voice prints on ONNX Runtime, grouped into voices: who speaks when.                                                      |
| `speech-mlx/`     | Qwen3-ASR and Qwen3-ForcedAligner on Apple silicon through the shared speech contracts.                                        |
| `shots/`          | Model-free PySceneDetect content detection over the ingest proxy.                                                              |
| `faces/`          | YuNet face detection on sampled ingest frames and deterministic tracking for reframing.                                        |
| `editorial/`      | Local Qwen 3.5 proposal, review, and targeted visual checks on Apple silicon; a separate, opt-in transcript-only cloud worker. |

Each family has its own `pyproject.toml`, lockfile, and environment, with
`clipmill-worker-sdk` as a path dependency. Apple-only dependencies use platform
markers so the projects can also be resolved on other platforms.

## Pinned models and binaries

The daemon names allowed model weights and executables on each lease. Workers
verify model files against their manifests immediately before loading them.
Shot and face detection also use the leased FFmpeg executable: decoder changes
can change pixels and therefore detection results.

The stage registry declares which binaries a stage may receive. The daemon
supplies an absolute path and a bill-of-materials build identity. The SDK's
`require_tool` rejects missing or duplicate entries, relative paths, symlinks,
and non-executable files; it never falls back to `PATH`. These checks do not
re-verify the extracted executable's digest: the BOM pins the downloaded archive.

Build identities enter stage payloads and artifact keys. Machine-specific paths
do not, so identical inputs and tool builds can share an address across machines.
Re-pinning a decoder invalidates the stages that use it.

## Speech implementation selection

`asr` and `forced-align` each have portable and MLX implementations.
`tools/bench/speech-benchmark.py` measures installed implementations over a
fixture. The daemon records the results in its signed device profile and fixes
the selected implementation on each task when planning a job.

Implementation identity enters the artifact key because different implementations
can produce different observations from the same audio. Re-measuring a device
changes future plans without changing published artifacts. An unmeasured device
uses the portable implementation, explicitly marked `unmeasured_fallback`.

## Editorial workers

`clipmill-worker-editorial` serves local proposal, review, and visual-check tasks
using pinned MLX weights. The separately launched
`clipmill-worker-editorial-cloud` serves only cloud proposal and review tasks.
Cloud use requires transcript consent and a run budget; media is not sent. The
adapter retrieves its credential from the OS credential store and reserves budget
before requests, retaining the charge for uncertain or interrupted calls.

## Provision and run the reference worker

The worker trust store is local to one daemon data directory. Provisioning writes
a mode-`0600` private identity and a separate trusted public-key entry; it never
prints the private key. Do not commit generated identities.

```sh
cargo run -p clipmilld --bin clipmill-worker-keygen -- \
  --data-dir /path/to/clipmill-data \
  --identity /private/path/echo-worker.json

uv sync --frozen --project workers/echo
uv run --project workers/echo clipmill-worker-echo -- \
  --data-dir /path/to/clipmill-data \
  --identity /private/path/echo-worker.json
```

`clipmilld` listens on `<data-dir>/run/clipmill-workers.sock` by default. The daemon
accepts `--worker-socket`; both daemon and worker understand
`CLIPMILL_WORKER_SOCKET`. The shared-memory broker uses the private
`<data-dir>/run/clipmill-shm.sock` path. Only explicitly provisioned local public
keys are trusted.

## Protocol and durability boundary

The daemon sends a fresh random challenge. A worker signs the challenge and its
complete descriptor (worker ID, family, protocol, capabilities, backend, and
memory limit) with Ed25519. The daemon accepts only the current and previous
minor protocol versions, rejects replay and duplicate active worker IDs, and
leases only tasks covered by the authenticated descriptor.

Workers pull work and explicitly accept a lease before heartbeating. Each lease
provides a staging directory, and completion declares relative output paths.
Workers never assign artifact IDs or write SQLite. The daemon validates the exact
file set, hashes and atomically publishes it through CAS, roots it, advances the
task/job transaction, and only then acknowledges completion. Retrying identical
success or failure completion bytes returns the original durable acknowledgement;
conflicting reuse is rejected.

The SDK maps shared data read-only and exposes a zero-copy `pyarrow.Buffer`.
Linux uses a sealed `memfd` transferred with `SCM_RIGHTS`; macOS uses a read-only
POSIX shared-memory object unlinked after acknowledgement. The SDK validates the
one-use token, lease, data type, dimensions, overflow-safe byte length, timebase,
and SHA-256. The daemon revokes mappings on acknowledgement, lease end,
cancellation, disconnect, or process death.

Run `just gate-workers` for the authenticated response-loss and hard-kill drill.
It covers worker death, lease expiry/reissue, daemon death/reconnect,
cancellation, staging cleanup, shared-memory cleanup, and verified CAS output.
