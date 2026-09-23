# Contributing to ClipMill

Contributions are welcome. ClipMill is pre-release software; please open an issue
before substantial architecture changes so the scope and compatibility impact
can be discussed.

## Development setup

Follow [Run from source](README.md#run-from-source) for dependencies, model weights,
and the two-terminal app and worker setup. `just --list` lists available commands.

For frontend layout work, `pnpm --filter @clipmill/desktop dev` serves a separate
browser preview at `http://127.0.0.1:5173/preview.html`. It uses synthetic data;
verify persistence, processing, and export through the native application. See
[the desktop workspace guide](docs/frontend-workspace.md).

## Finding your way around

| Directory      | Purpose                                                         |
| -------------- | --------------------------------------------------------------- |
| `apps/desktop` | React interface and Tauri host                                  |
| `crates`       | Rust daemon, media processing, editing, and export              |
| `workers`      | Python model workers and shared SDK                             |
| `contracts`    | JSON schemas, Protobuf definitions, and fixtures                |
| `packages`     | Shared TypeScript contracts and design tokens                   |
| `integrations` | External source integrations                                    |
| `tools`        | Setup, code generation, security checks, and integration drills |
| `docs`         | Feature guides, architecture, and decision records              |

## Implementation conventions

- Keep durable project state in the daemon. Derived artifacts use the
  content-addressed store; the renderer accesses them through the host API.
- Use the shared timebase of integer ticks at 1/90000 for contract and durable
  timeline values. Convert at presentation boundaries.
- Change schemas or Protobuf definitions in `contracts/`, then run `just codegen`
  and commit the generated output. Do not edit generated files directly.
  Code generation additionally needs `buf` and `cargo-typify` 0.7.0; see
  [the generation script](tools/codegen/generate.sh).
- Preserve cancellation, recovery, and explicit network consent when extending
  processing paths. Document changes to these boundaries in the relevant guide
  and [threat model](docs/threat-model.md).
- Write comments that explain constraints or non-obvious decisions. Keep task
  history and progress reports out of source comments.

## Verification

Run the checks relevant to your change, and report what you ran in the PR:

```sh
just lint  # Rust, Python, TypeScript, formatting, and schema checks
just test  # Workspace unit and integration suites
```

Specialized `gate-*` recipes exercise real media, workers, failure recovery, and
network isolation. Some require installed weights, specific hardware, or private
evaluation data; read the recipe before running it. Historical recipe names are
retained for compatibility with CI.

Add regression coverage for behavior changes. Documentation-only changes should
be checked for accuracy, links, and formatting; comment-only edits must preserve
executable code and test coverage.

## Pull requests

Keep each PR focused. Explain the problem, resulting behavior, relevant checks,
and any remaining limitations. Complete the security review categories identified
by CI when a sensitive boundary is involved. Commit generated changes alongside
their source definitions, and keep private recordings, credentials, and build
outputs out of Git.

Use a short, imperative commit subject, optionally prefixed with the area changed.

## Developer Certificate of Origin

External contributions must be signed off (`git commit -s`), certifying the
[DCO](https://developercertificate.org/): you wrote the change or have the
right to submit it under AGPL-3.0.

## Reporting bugs

Open a GitHub issue with reproduction steps, your operating system, and relevant
error messages. Remove private paths, transcript content, and credentials from
logs before sharing them. Report security-sensitive issues privately as described
in [SECURITY.md](SECURITY.md).
