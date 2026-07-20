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

Future families (`asr/`, `vision/`, `judge/`, …) follow the same shape: own
`pyproject.toml`, own venv, `clipmill-worker-sdk` as a path dependency.

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
