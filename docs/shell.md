# Desktop shell (W9/W10)

The shell is the first thing a person sees, and Phase 0's rule for it is the
same as everywhere else: show what is true, or say that it is not known. It
renders one working screen and eight honest placeholders.

## Process boundary

`apps/desktop/src-tauri` (crate `clipmill-shell`) is a Tauri 2 host. The React
renderer in `apps/desktop/src` is a pure view layer.

The renderer has no filesystem, shell, or HTTP capability. This is not enforced
by an allowlist that a future edit could widen — those plugin crates are simply
absent from `Cargo.toml`, so the code to touch a disk or open a socket is not in
the binary. The renderer's entire reach is three commands:

| Command            | Returns                                          |
| ------------------ | ------------------------------------------------ |
| `daemon_state`     | current connection state                         |
| `reconnect_daemon` | forces a supervision pass, returns the new state |
| `device_profile`   | `{ artifactId, profileJson }` from the daemon    |

`device_profile` passes the profile through as canonical JSON rather than
reshaping it in Rust. The `clipmill.device_profile.v1` schema stays the only
contract between daemon and renderer, and the renderer parses it with the
generated type.

## Connection supervision

`DaemonSupervisor` probes health on a two-second tick. Each call opens its own
short-lived Unix socket connection: the control plane is low-frequency, and a
fresh socket means a half-dead connection can never wedge the UI.

When the socket stops answering, the supervisor publishes `Disconnected` and
tries to start `clipmilld` — but only if one is not already answering. A daemon
someone else started keeps ownership of its own lifetime; the shell never kills
it.

Transitions are published as edges, not levels, so a steady connection is
silent rather than emitting twice a second.

The socket path is resolved through `clipmilld::Config`, the daemon's own
configuration type, so the two processes cannot disagree about where to meet.

### Why the renderer re-fetches on daemon identity, not connectivity

`connectionKey` combines the daemon version with its process start time. A
daemon that was killed and respawned reports a new start time, so the key
changes and the device profile is fetched again. Keying on "connected" alone
would miss a restart that happened between two ticks.

## Component system

Components are [shadcn/ui](https://ui.shadcn.com) (MIT), vendored into
`src/components/ui`. Copy-paste rather than a runtime dependency is the point:
the source is ours, so it can be restyled to this design instead of fighting a
library's own visual language, and it can be redistributed under AGPL.

Two libraries were considered and not adopted. **HeroUI** (Apache-2.0) is
excellent but ships an opinionated look and a second accessibility stack
(React Aria alongside Radix); one primitive stack is worth more here than any
individual component. **Aceternity UI** was rejected on licensing: its stated
terms forbid redistributing the source files, which cannot be reconciled with
AGPL's requirement to distribute source. Note that a vendored component is not
an npm dependency, so `tools/security/check-node-licenses.py` would not have
caught it — vendored source has to be licence-checked by hand.

### One palette, not two

shadcn components are written against fixed semantic variable names
(`--background`, `--primary`, `--border`, …). Left alone, those become a second
palette beside the design tokens, and the two drift.

Instead every semantic name is generated as an _alias_ for a `--cm-*`
primitive. Aliases are emitted once rather than per theme: `var()` resolves at
use time, so when the attribute swaps the primitive underneath, the semantic
name follows. Surfaces map to the glass values, not to opaque fills, or every
shadcn component would punch a hole through the shell.

### cn() knows the type scale

`tailwind-merge` only recognises Tailwind's stock class groups, so `text-body`
was invisible to it: passing it to a component that already carries `text-sm`
left both classes, and stylesheet order silently picked the winner — the
navigation rendered at 14px instead of the design's 13px. `cn()` extends the
merger with the custom font-size group so the override is deterministic.

## Theme

The design ships a light and a dark artboard for every screen, with identical
geometry, and the Stitch export hardcodes per-theme colours on each element.
Re-expressing that literally would fork every component in two.

Instead `packages/tokens/src/tokens.json` holds each themed value once per
theme. `scripts/build-css.mjs` generates `tokens.css`, where dark lives on
`:root` (so first paint is correct before any script runs) and light is a
wholesale override on `:root[data-theme='light']`. `ThemeController` flips that
one attribute; the attribute always wins over the OS preference, which is only
consulted to choose the initial value. CI checks the generated CSS for drift
against the token document (decision R2).

A test asserts both themes declare the same variable names — a themed value
added to one side only would silently keep the other theme's stale colour, which
is invisible until someone toggles.

## What Models & Device shows, and what it refuses to show

Real, from the daemon: CPU model and core layout, platform, total and available
memory, accelerator and its memory, probed capability results, runtime
identities, decode throughput, shared-memory transfer rate, hardware round-trip,
the profile's content address, and its attestation state.

Deliberately absent: the design's live GPU-load and temperature meters. Phase 0
measures memory but samples nothing continuously, so the top bar shows the
memory it genuinely knows and omits the rest rather than animating a fiction.
Installed models read `0 installed`, because there are none.

The **Local Lock badge is bound to `HealthResponse.local_lock`**, and reads
`unknown` whenever the daemon is unreachable. A badge hardcoded to "ON" would be
worse than no badge: it would assert a guarantee nobody checked. Note that the
daemon currently answers `true` unconditionally — correct for Phase 0, which has
no egress path at all, but it should become a real policy read when one exists.

## Gates

- `just gate-tokens` — token CSS is reproducible; renderer typechecks, tests, builds.
- `just gate-shell` — starts a real daemon, reads real measured hardware over
  the real socket, confirms the second read is a cache hit on the identical
  artifact, then SIGKILLs the daemon and confirms the shell reports the loss
  instead of serving a stale answer.

`gate-shell` needs the pinned `ffprobe` and a built `clipmilld`, so the test is
`#[ignore]`d out of `cargo test --workspace` and runs in the `shell-link` CI job
on macOS and Linux.
