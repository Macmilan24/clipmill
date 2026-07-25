# ClipMill

**Local-first AI video clipping studio.** ClipMill turns long-form video —
podcasts, streams, lectures, interviews — into publish-ready short clips, with
every stage of the intelligence running on your own machine. No upload, no
cloud dependency, no per-minute pricing: your footage never leaves your disk
unless you explicitly send it somewhere.

> **Status: pre-alpha, Phase 0 ("Harness") complete.** Contracts, the durable daemon,
> filesystem artifact CAS, reusable durable DAG scheduler, local-source
> evidence pipeline, and authenticated external-worker runtime are implemented:
> private Unix-socket IPC, SQLite/WAL roots and leases, deterministic cache keys,
> atomic publication, cursor-replayed task events, cancellation, recovery/GC,
> hard-kill drills, pinned FFprobe supervision, immutable source fingerprints,
> rational source maps, signed worker registration, daemon-owned output staging,
> one-use read-only shared memory, measured and signed device profiles, and the
> signed offline evaluation harness on macOS and Linux. The integrated security
> workflow and signed, rights-cleared Seed-40 cold/warm proof close the Phase 0
> exit. Nothing here makes clips yet.

## Why

Cloud clipping tools are genuinely good — and structurally unable to offer
what a local tool can: unlimited iteration at zero marginal cost, privacy by
architecture, and full ownership of the editorial pipeline. ClipMill is built
from a complete system-design monograph that treats the local machine as the
studio, with orchestration width and depth standing in for frontier-model
scale.

## Architecture (target)

Polyglot by explicit boundary — each language's territory ends at a process
boundary with a versioned contract:

```
┌─────────────────────────────────────────────────────────┐
│  clipmilld  (Rust) — single writer, owns all truth       │
│  scheduler · project state (SQLite/WAL) · artifact CAS  │
│  IPC gateway · resource manager · render supervision    │
└───────┬───────────────┬───────────────┬─────────────────┘
        │ protobuf/UDS  │               │
┌───────┴───────┐ ┌─────┴─────────┐ ┌───┴──────────────────┐
│ media workers │ │ model workers │ │ desktop shell        │
│ FFmpeg,pinned │ │ Python, per-  │ │ Tauri 2 + React,     │
│ sandboxed     │ │ family venvs  │ │ types generated from │
│               │ │ stateless     │ │ contracts            │
└───────────────┘ └───────────────┘ └──────────────────────┘
```

- **Contracts are the source of truth** (`contracts/`): JSON Schema for
  artifacts, Protobuf for IPC; Rust/Python/TypeScript types are generated,
  never hand-mirrored.
- **All time is rational** — integer ticks at 1/90000, never float seconds.
- **Derived data is content-addressed**; anything that can't be regenerated
  lives in SQLite under a single writer. There is no third place.
- **Local Lock** — a zero-egress mode enforced by CI (network-denial tests),
  not by promise.

## Prerequisites (development)

- Rust (stable, see `rust-toolchain.toml`)
- Node 22+ and `pnpm`
- Python 3.12+ and `uv`
- `just`, `buf`
- macOS (primary) or Linux; Windows support arrives later

```sh
just setup   # fetch pinned FFmpeg and sync all workspaces
```

## License

[AGPL-3.0-only](LICENSE). You can use, study, modify, and share ClipMill
freely; if you offer a modified version as a network service, you must publish
your source.

## Credits

ClipMill is designed and directed by **Sami (Samuel Dagne)**. The system
design and implementation are built in close collaboration with **Claude**
(Anthropic), which serves as the project's pair programmer.
