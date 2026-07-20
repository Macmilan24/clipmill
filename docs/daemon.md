# Durable daemon, artifacts, orchestration, sources, and workers (W2-W6)

`clipmilld` is a foreground, single-writer process. On macOS and Linux it
serves the existing `clipmill.ipc.v1` protobuf contract over a private Unix
domain socket using protobuf varint length-delimited frames.

## Paths and configuration

The default application data directory is platform-specific. Override it with
`--data-dir` or `CLIPMILL_DATA_DIR`; override the socket with `--socket` or
`CLIPMILL_SOCKET`; override the pinned probe sidecar with `--ffprobe` or
`CLIPMILL_FFPROBE`. The authenticated worker socket uses `--worker-socket` or
`CLIPMILL_WORKER_SOCKET`. Command-line values win over environment values.

```text
<data-dir>/
  state/
    clipmill.db
    backups/
    probe-scratch/
    worker-identities/
    worker-trust/
  artifacts/
    objects/sha256/
    staging/
    quarantine/
  run/
    clipmilld.sock
    clipmill-workers.sock
    clipmill-shm.sock
    daemon.lock
```

Directories are mode `0700`; the database, sockets, lock, private worker
identities, and trust entries are `0600`.
`clipmilld` refuses a second writer, removes a stale socket only after acquiring
the daemon lock, and runs SQLite `quick_check` before serving requests.

Schema v2 adds strict project-to-artifact roots. Schema v3 adds jobs, task DAGs,
resource declarations, leases, task events, circuit breakers, and active-task
artifact roots. Schema v4 adds immutable sources, file observations,
source-artifact roots, and optional source links on jobs. Existing databases
are checked and backed up with SQLite's
backup API before each transactional upgrade; fresh databases receive the
complete v4 schema without a redundant backup. Backups are atomically
published under `state/backups/` with mode `0600`.

## Control API surface

W2 implements ping, health, and create/get/list/delete project operations.
Mutating requests are transactionally deduplicated by `request_id`, so retrying
after a lost response returns the original protobuf response without repeating
the mutation. W4 adds submit/get/list/cancel job operations plus cursor-based
task-event replay and live delivery. The versioned `demo-dag` fixture exercises
a reusable persisted DAG scheduler. W5 adds register/get/list source operations
and the versioned `probe-source` job. W6 moves `demo-dag` execution to a
standalone authenticated pull worker without changing scheduler semantics;
device profiling remains `UNAVAILABLE`. W3-W6 intentionally expose no public
artifact command: only the daemon
publishes and roots CAS objects.

Every task records its artifact kinds and CPU, RAM, accelerator/VRAM, disk,
network, thermal, determinism, checkpoint, preemption, implementation, and
attempt declarations. The W4 built-in executor reserves its declared CPU/RAM/
disk budget, renews a five-second heartbeat against a fifteen-second lease, and
publishes completion only after CAS and SQLite state are durable. Transient
failure backoff is capped at 1/2/4 seconds; deterministic repetitions open a
durable stage/implementation/input circuit breaker.

W5 source registration accepts only absolute UTF-8 regular local files. URLs,
symlinks, directories, devices, and missing paths are rejected. Files up to
16 MiB are fully hashed; larger files use a versioned edge plus sixteen-window
sampling plan. The source fingerprint combines those ordered bytes with
canonical normalized FFprobe metadata, excluding paths and filesystem identity.
The pinned sidecar has a 15-second deadline, bounded output, a private working
directory, a cleared environment, and a file/pipe-only protocol allowlist.
Source maps use rational stream timebases and 90 kHz edit ticks, preserve gaps,
split backward timestamp resets into explicit segments, and are published and
rooted through the existing CAS/job durability transaction.

W6 workers self-connect to the private worker socket. Registration signs a
fresh daemon challenge plus the complete capability descriptor with Ed25519;
the daemon accepts the current and previous protocol minor versions and trusts
only public keys provisioned under `state/worker-trust/`. Workers pull matching
leases, accept them explicitly, heartbeat, observe cancellation, and retry the
same durable completion after response loss. Lease-scoped output directories
are prepared by the daemon. Workers may write only declared regular files; the
daemon alone hashes, publishes, assigns artifact IDs, roots outputs, advances
task/job state, and acknowledges completion.

The separate shared-memory socket transfers one-use, lease-bound descriptors.
Linux publishes sealed `memfd` handles through `SCM_RIGHTS`; macOS publishes a
read-only POSIX shared-memory object and unlinks it after mapping acknowledgement.
The Python SDK validates dimensions, byte length, timebase, lease identity, and
SHA-256 before exposing a zero-copy PyArrow buffer. Handles are revoked when
mapped, disconnected, cancelled, expired, or completed.

## Current recovery claim

The kill drill now submits real four-node jobs while injecting `SIGKILL`. Every
acknowledged job remains queryable; prior-epoch running work becomes retryable
without spending a worker attempt; stale lease completions are rejected; and
all jobs recover to a verified terminal state within 30 seconds. The cache
drill separately covers every CAS publication boundary. Active task roots now
join project roots, transitive inputs, and reader pins in GC reachability.
Both drills run for 25 iterations per operating system in CI and 50 through the
local gates, with five-iteration smoke runs inside network denial.

W6 additionally kills real worker and daemon processes, expires and reissues
leases, loses completion responses, cancels active work, verifies CAS outputs,
and checks shared-memory/staging cleanup. This proves task/job recovery across
the authenticated external-worker boundary. Device-profile and evaluation
recovery remain W7; the private Seed-40 exit remains W8. Phase 0 is therefore
not yet complete.
