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

## What a renderer may read (W22)

Two doors, two lists, and neither is "everything in the store".

`ReadArtifact` serves **documents** over the control socket: eleven kinds, each
carrying exactly one file that the kind — not the request — names, so a caller
cannot point somewhere else inside the object. Anything larger than a chunk
arrives in several replies and the response states the total, so a truncated
document is an error rather than one with the end missing.

`clipmill-media://` serves **media**: four kinds, and only the files each
artifact's own descriptor named. It answers HTTP byte ranges, because a player
seeks. The daemon authorizes and inventories; the shell process opens the file
and serves the span. **No path crosses between them** — the object directory is
derived from the content address exactly as the store derives it.

Both doors run the same ladder: the address parses, this project produced the
artifact, the kind is on the list, and the store re-verifies the object against
its manifest. A project that does not own an artifact is told **not found**
rather than denied, so it learns nothing about another project's store.

Absent from both by decision: model weights, and the three speech intermediates
(voice activity, recognition, alignment) — the assembly that fuses them is what a
reader wants, and an allowlist that names everything is not one.

A task reports the artifact kind it publishes, which is how a screen finds the
filmstrip it wants without knowing that the daemon calls that work
`ingest-filmstrip`. See R24.

## What the screens show, and what they leave out

Library, New Project and Analysis Progress are built to the design except where
the design specifies a figure nothing produces. Those were left out rather than
filled in — see R25 for the full list and the reasoning. In brief: no project
score, no speaker counts, no category chips, no completion percentage, no
per-stage durations, no sampled GPU load, and a search field that says it covers
titles because that is what it covers.

Two sentences in the design are false here and are not repeated. Closing the
application does not leave the run going — the daemon is the shell's child and
dies with it — but jobs are durable and artifacts are addressed by content, so
reopening resumes rather than restarts, and the copy says that. The live log
begins when the screen opens, because the host holds one subscription for the
whole application and replays from its own cursor.

## The file dialog

Opening a file picker is the host's, not the page's. The dialog plugin is
registered so this crate's own `choose_source_file` command can open one, and the
capability grants no `dialog:` or `fs:` permission — so the WebView cannot invoke
either, and the only route to a path is a command that returns exactly one the
user chose in a native window. `tauri-plugin-fs` appears in the dependency tree
solely as the dialog plugin's own dependency; the shell and HTTP plugins remain
absent from `Cargo.toml`.

## Linux dependency posture

Tauri's Linux WebView is WebKitGTK, which still targets GTK3 while gtk-rs has
moved to GTK4. cargo-deny therefore reports 16 advisories against the shell's
graph. All 16 are `unmaintained`; none are vulnerabilities and none are
unsound. They are listed by exact id in `deny.toml` rather than by relaxing the
advisory class, so a genuine vulnerability in any of those same crates still
fails the gate.

macOS uses WKWebView and links none of it. Revisit when webkit2gtk-rs targets
GTK4.

## Gates

- `just gate-tokens` — token CSS is reproducible; renderer typechecks, tests, builds.
- `just gate-shell` — starts a real daemon, reads real measured hardware over
  the real socket, confirms the second read is a cache hit on the identical
  artifact, then SIGKILLs the daemon and confirms the shell reports the loss
  instead of serving a stale answer.
- `just gate-shell-pipeline` — the whole data plane the screens sit on, out of
  process: the pinned encoder writes a real file, the daemon registers and probes
  it, an analysis is submitted and watched as it moves, a published document is
  read back through the document door, and a filmstrip tile is streamed through
  the same protocol handler the WebView addresses — including a byte range, which
  is what a seeking player sends. Then the refusals: an unlisted kind over both
  doors, another project's id, and a file the descriptor never named. It does not
  wait the analysis out, because the stages after ingest need worker processes
  this drill does not start.

Both need the pinned FFmpeg sidecars and a built `clipmilld`, so their tests are
`#[ignore]`d out of `cargo test --workspace` and run in the `shell-link` CI job on
macOS and Linux.
