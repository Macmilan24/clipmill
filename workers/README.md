# Authenticated external workers (W6)

Python model/media workers. Each worker **family** gets its own uv project and
its own locked virtual environment, so a dependency conflict (CUDA, PyTorch,
Paddle, …) is contained to one pool and can never take down another family or
the daemon. Workers are strictly stateless: they take leased tasks over the
worker protocol, stream heartbeats, and return artifacts — all durable state
belongs to `clipmilld`.

- `sdk/` — `clipmill_worker_sdk`: the worker protocol client (handshake with
  capability descriptor, lease/heartbeat/complete/decline) and generated
  contract types. Every worker family depends on it.
- `echo/` — the null worker: exercises the full protocol without doing any
  work. It is the protocol's reference implementation and test target.
- `vad/` — silero-VAD on onnxruntime: where speech is, before anything
  transcribes it.
- `asr-whispercpp/` — recognition on the path that runs everywhere.
- `align/` — forced alignment on a wav2vec2 CTC model, the aligner every
  machine has.
- `speech-mlx/` — Qwen3-ASR and Qwen3-ForcedAligner on Apple silicon, behind
  the same two contracts. macOS-only by dependency marker, so its lockfile
  resolves to nothing on Linux rather than to a broken environment.

Future families (`vision/`, `judge/`, …) follow the same shape: own
`pyproject.toml`, own venv, `clipmill-worker-sdk` as a path dependency.

## Two implementations of one capability

`asr` and `forced-align` each have two families behind them, and which one runs
is not a preference written down anywhere — it is measured. `tools/bench/speech-benchmark.py`
runs every installed implementation over a fixture, the daemon folds what it
measured into its signed device profile, and a job records the chosen
implementation on each task when it is planned (D19, R19).

That choice reaches the artifact key. Two implementations produce different
bytes from the same audio, so they are different producers of different
observations and must not share a content address; re-measuring a device
changes what the next job chooses and never re-attributes anything already
published. A machine nobody has benchmarked runs the portable implementation
and its profile says `unmeasured_fallback`, so a fallback never reads as a
choice.

## Provision and run the reference worker

The Phase 0 development trust store is local to one daemon data directory.
Provisioning writes a mode-`0600` private identity and a separate trusted
public-key entry; it never prints or commits the private key.

```sh
cargo run -p clipmilld --bin clipmill-worker-keygen -- \
  --data-dir /path/to/clipmill-data \
  --identity /private/path/echo-worker.json

uv sync --frozen --project workers/echo
uv run --project workers/echo clipmill-worker-echo -- \
  --data-dir /path/to/clipmill-data \
  --identity /private/path/echo-worker.json
```

`clipmilld` listens on `<data-dir>/run/clipmill-workers.sock` by default. The
daemon accepts `--worker-socket`; the daemon and worker both understand
`CLIPMILL_WORKER_SOCKET`. The shared-memory broker remains at the private
`<data-dir>/run/clipmill-shm.sock` path.

Never commit a generated identity. Phase 0 trusts explicitly provisioned local
public keys. The signed worker/model registry planned for Phase 4 replaces this
development trust anchor without changing the challenge/signature fields.

## Protocol and durability boundary

The daemon sends a fresh random challenge. A worker signs the challenge and
its complete descriptor (worker ID, family, protocol, capabilities, backend,
and memory limit) with Ed25519. The daemon accepts only the current and previous
minor protocol versions, rejects replay and duplicate active worker IDs, and
leases only tasks covered by the authenticated descriptor.

Workers pull work and explicitly accept a lease before heartbeating. A worker
receives a lease-scoped staging directory and declares completed relative paths;
it never assigns artifact IDs or writes SQLite. The daemon validates the exact
file set, hashes and atomically publishes it through CAS, roots it, advances the
task/job transaction, and only then acknowledges completion. Retrying identical
success or failure completion bytes returns the original durable acknowledgement;
conflicting reuse is rejected.

The SDK maps shared data read-only and exposes a zero-copy `pyarrow.Buffer`.
Linux uses a sealed `memfd` transferred with `SCM_RIGHTS`; macOS uses a read-only
POSIX shared-memory object that is unlinked after acknowledgement. The SDK
validates the one-use token, lease, data type, dimensions, overflow-safe byte
length, timebase, and SHA-256. The daemon revokes mappings on acknowledgement,
lease end, cancellation, disconnect, or process death.

Run `just gate-workers` for the authenticated response-loss and hard-kill drill.
It covers worker death, lease expiry/reissue, daemon death/reconnect,
cancellation, staging cleanup, shared-memory cleanup, and verified CAS output.
