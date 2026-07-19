# Durable daemon and artifact foundation (W2-W3)

`clipmilld` is a foreground, single-writer process. On macOS and Linux it
serves the existing `clipmill.ipc.v1` protobuf contract over a private Unix
domain socket using protobuf varint length-delimited frames.

## Paths and configuration

The default application data directory is platform-specific. Override it with
`--data-dir` or `CLIPMILL_DATA_DIR`; override the socket with `--socket` or
`CLIPMILL_SOCKET`. Command-line values win over environment values.

```text
<data-dir>/
  state/
    clipmill.db
    backups/
  artifacts/
    objects/sha256/
    staging/
    quarantine/
  run/
    clipmilld.sock
    daemon.lock
```

Directories are mode `0700`; the database, socket, and lock are `0600`.
`clipmilld` refuses a second writer, removes a stale socket only after acquiring
the daemon lock, and runs SQLite `quick_check` before serving requests.

Schema v2 adds strict project-to-artifact roots. A v1 database is checked and
backed up with SQLite's backup API before the transactional migration. A fresh
database receives the complete v2 schema without a redundant backup. Backups
are atomically published under `state/backups/` with mode `0600`.

## Control API surface

W2 implements ping, health, and create/get/list/delete project operations.
Mutating requests are transactionally deduplicated by `request_id`, so retrying
after a lost response returns the original protobuf response without repeating
the mutation. Job submission, task events, and device profiling return
`UNAVAILABLE` until their owning workstreams land. W3 intentionally adds no
public artifact RPC or command: the daemon exposes an in-process coordinator
for the future scheduler and lifecycle tests.

## Current recovery claim

The W2 kill drill verifies that every acknowledged project mutation survives
`SIGKILL`, WAL recovery, and stale-socket cleanup. The W3 cache drill extends
that claim to acknowledged filesystem-object plus SQLite-root publications,
payload verification, committed-object visibility, and quarantine of partial
staging directories. Both drills run for 25 iterations per operating system in
CI and 50 through the local gates.

Task leases, active-task GC roots, and interrupted-job recovery remain unproven
until W4, so Phase 0 is not yet complete.
