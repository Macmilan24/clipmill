# Daemon foundation (W2)

`clipmilld` is a foreground, single-writer process. On macOS and Linux it
serves the existing `clipmill.ipc.v1` protobuf contract over a private Unix
domain socket using protobuf varint length-delimited frames.

## Paths and configuration

The default application data directory is platform-specific. Override it with
`--data-dir` or `CLIPMILL_DATA_DIR`; override the socket with `--socket` or
`CLIPMILL_SOCKET`. Command-line values win over environment values.

```text
<data-dir>/
  state/clipmill.db
  run/clipmilld.sock
  run/daemon.lock
```

Directories are mode `0700`; the database, socket, and lock are `0600`.
`clipmilld` refuses a second writer, removes a stale socket only after acquiring
the daemon lock, and runs SQLite `quick_check` before serving requests.

Schema v1 is created transactionally. Any future migration from an existing
schema must create a pre-migration backup before beginning its transaction;
the only current exception is initial creation of a new, empty v1 database.

## W2 API surface

W2 implements ping, health, and create/get/list/delete project operations.
Mutating requests are transactionally deduplicated by `request_id`, so retrying
after a lost response returns the original protobuf response without repeating
the mutation. Job submission, task events, and device profiling return
`UNAVAILABLE` until their owning workstreams land.

## Current recovery claim

The kill drill verifies that every acknowledged project mutation survives
`SIGKILL`, WAL recovery, and stale-socket cleanup. It does not yet prove task
lease recovery, content-addressed artifact commits, or partial staging
quarantine; those assertions arrive with W3/W4 before the Phase 0 gate can pass.
