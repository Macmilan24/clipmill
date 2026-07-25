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
> exit. The desktop shell now boots in both themes and reports this machine's
> real measured hardware over the daemon socket, reconnecting on its own when
> the daemon dies. Nothing here makes clips yet.

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

On Linux the shell additionally needs the WebView toolkit:

```sh
sudo apt-get install libwebkit2gtk-4.1-dev libsoup-3.0-dev \
  libjavascriptcoregtk-4.1-dev libgtk-3-dev librsvg2-dev patchelf
```

```sh
just setup   # fetch pinned FFmpeg and sync all workspaces
just app     # launch the desktop shell against a live daemon
```

## The desktop shell

`apps/desktop` is a Tauri 2 host with a React renderer. The split is deliberate:

- The **renderer has no capabilities** — no filesystem, shell, or HTTP plugin is
  compiled into the binary at all, so there is no ACL to misconfigure and no
  code path to re-enable them. It reaches the daemon through exactly three
  commands (`daemon_state`, `reconnect_daemon`, `device_profile`).
- The **host owns the socket**, starts `clipmilld` when it is not already
  running, and publishes every connection transition as an event.
- **One theme switch.** The design ships a light and a dark artboard per screen;
  `packages/tokens` expresses that as one token document, generated into
  `tokens.css` and checked for drift in CI. Components never branch on theme.
- **Phase 0 ships one real screen.** Models & Device renders measured hardware,
  probed capabilities, and a Local Lock badge bound to the daemon's own answer —
  including saying `unknown` when the daemon is unreachable. The other eight
  sections state which phase builds them instead of showing a mockup.

## License

[AGPL-3.0-only](LICENSE). You can use, study, modify, and share ClipMill
freely; if you offer a modified version as a network service, you must publish
your source.

## Credits

ClipMill is designed and directed by **Sami (Samuel Dagne)**. The system
design and implementation are built in close collaboration with **Claude**
(Anthropic), which serves as the project's pair programmer.
