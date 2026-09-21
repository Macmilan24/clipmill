# Desktop shell

The desktop application consists of a React renderer and a Tauri 2 host. The
host owns the daemon connection, native dialogs, and media serving. The daemon
owns projects, jobs, edit documents, and exports. See [Desktop workspace](frontend-workspace.md)
for screen layout and browser development.

## Host boundary

The renderer uses purpose-specific commands registered in
[`src-tauri/src/lib.rs`](../apps/desktop/src-tauri/src/lib.rs). They cover project
and job queries, source import, analysis, edit commands, preview and export plans,
readiness, native file selection, and delivery actions. The renderer does not
receive a general filesystem, shell, or HTTP API.

The native dialog plugin is registered for host-owned file pickers; WebView
capabilities do not grant the plugin's general dialog or filesystem commands.
Authorization and validation remain necessary for every exposed host operation.
See [the threat model](threat-model.md) for the security boundary and limitations.

## Daemon connection and recovery

`DaemonSupervisor` probes the daemon every two seconds. Queries use short-lived
Unix socket connections. State transitions are published to the renderer; an
unchanged connection state does not generate repeated events. Socket location
comes from the daemon's configuration type.

The host starts a daemon when none is available. A daemon started separately
retains ownership of its lifetime. Jobs and edit documents are durable, so a
restarted application can retrieve their current state instead of assuming a
new run. This does not imply that processing continues after a shell-owned
daemon exits.

The renderer's connection key includes the daemon version and process start
time. A changed daemon identity invalidates cached reads even when a restart
happens between health checks. Task events replay from the host's cursor across
reconnections; a newly mounted screen's live log begins when it subscribes.

## Documents and media

`ReadArtifact` serves allowlisted document kinds over the control socket. The
artifact kind determines the permitted file; callers cannot request arbitrary
paths within an object. Chunked reads include the total size so incomplete
responses are detectable.

`clipmill-media://` serves allowlisted media files named by artifact descriptors.
It supports byte ranges for playback and seeking. The daemon authorizes and
inventories media; the host derives the object location from its content address
and serves the requested bytes.

Both paths validate the address, project ownership, allowed kind, and stored
manifest. An artifact outside a project's ownership is reported as not found.
Neither interface exposes model weights or arbitrary files from the store.

## Selection and saved work

Routes carry the identities needed by each screen. Analysis identifies a run;
Inspector identifies a project, source, candidate, and run; Editor and Export
add the selected edit document. Navigation preserves that selection rather
than opening another project's newest edit.

The shell remembers route identities and labels across launches. Restored values
are validated, and documents and preview plans are fetched from the daemon.
Unresolved selections remain unavailable instead of silently opening a different
project. Editor and Export offer a document picker when no clip is selected.

An export job records its document, revision, immutable snapshot, and destination.
The Export screen retrieves that job after navigation or relaunch and follows
its actual running, delivered, or failed state. Failed reads are retryable and
are distinct from a failed export.

## Readiness

`GetReadiness` reports model files, decoder availability, connected workers, and
recovery commands for each stage. Missing models or required tools block
submission. Disconnected workers are reported separately: a job may be submitted
and wait for a compatible worker to connect. Analysis Progress refreshes this
report while a run is active. An unreachable daemon is shown as unknown.

## Preview timing

A preview plan maps program frames to source ticks and proxy time. It includes
source dimensions, segment intervals, and proxy coverage. Playback, scrubbing,
framing, and trim commands use this mapping; a clip beginning several minutes
into a recording must not play the proxy from second zero.

Plans are bound to edit revisions. Responses for an older revision are discarded.
See [preview and render parity](preview-parity.md) for the presentation contract.

## Components and themes

Shared UI primitives live in `src/components/ui`; design tokens live in
`packages/tokens/src/tokens.json`. Generated CSS defines semantic colors for
light and dark themes. Components use those tokens rather than maintaining
independent palettes. `cn()` includes the custom type scale in its Tailwind
class-merging rules.

The theme controller changes one root attribute. Token drift and matching
variable sets across themes are checked automatically. Vendored components must
retain applicable license notices; check their licenses when adding or replacing
them.

## Verification

- `just gate-tokens` checks token generation, renderer types, tests, and build.
- `just gate-shell` checks daemon connection, measured hardware reads, and
  disconnect handling after forced termination.
- `just gate-shell-pipeline` checks source registration, job transitions,
  document reads, media byte ranges, and access refusals through the real host.
- `just gate-milestone-1` checks selected-clip identity, edits, restart recovery,
  and decoded exports through the real host and workers. Its historical name
  remains stable for scripts; it requires macOS speech synthesis and installed
  models.

The native integration checks require pinned FFmpeg sidecars and a built daemon.
They complement renderer tests over the host API; a browser preview alone does
not verify the native workflow.
