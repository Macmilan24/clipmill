# Contributing to ClipMill

Thanks for your interest! ClipMill is in **Phase 0 (pre-alpha)** — the
project is young, the architecture is deliberate, and contributions are
welcome within that frame.

## Ground rules

- **The design book governs.** ClipMill implements a written system design.
  Architectural changes (new processes, new durable state, new IPC surfaces,
  contract changes) need a discussion issue first, referencing
  [`docs/decisions.md`](docs/decisions.md) — decisions there are revisited by
  falsification, not by preference.
- **Contracts are the source of truth.** Never hand-edit generated code under
  `crates/clipmill-contracts`, `packages/contracts`, or `workers/sdk/**/gen`.
  Change the schema or proto in `contracts/`, run `just codegen`, and commit
  both.
- **No float seconds.** All times in contracts and durable records are
  rational (integer ticks at 1/90000). CI rejects violations.
- **Derived data is content-addressed; user state lives in SQLite.** There is
  no third place.

## Development setup

```sh
just setup    # fetches the pinned FFmpeg, syncs Rust/TS/Python workspaces
just test     # everything
just app      # run the desktop shell
```

Prerequisites: Rust stable, Node 22+ with pnpm, Python 3.12+ with uv,
`just`, `buf`.

## Pull requests

- Keep PRs workstream-sized: one concern, tests included, CI green.
- Every behavioral change carries a test that fails without it. Recovery and
  Local Lock guarantees are only real when exercised — that is why the
  kill-drill and network-denial jobs run on every push.
- Commit messages: short imperative subject, optionally
  `area: what changed` style. No `Co-Authored-By` trailers.

## Developer Certificate of Origin

External contributions must be signed off (`git commit -s`), certifying the
[DCO](https://developercertificate.org/): you wrote the change or have the
right to submit it under AGPL-3.0.

## Reporting bugs

Open a GitHub issue with reproduction steps. For anything
security-sensitive, see [SECURITY.md](SECURITY.md) — do **not** open a public
issue.
