# codegen

`generate.sh` regenerates every generated contract artifact from the sources
of truth in `contracts/`:

| Source                     | Generator                                                        | Output (committed)                                 |
| -------------------------- | ---------------------------------------------------------------- | -------------------------------------------------- |
| `contracts/proto/**`       | `buf generate` → remote plugin `community/neoeinstein-prost`     | `crates/clipmill-contracts/src/gen/proto/`         |
| `contracts/proto/**`       | `buf generate` → remote plugins `protocolbuffers/python` + `pyi` | `workers/sdk/src/clipmill/`                        |
| `contracts/proto/**`       | `buf generate` → remote plugin `bufbuild/es`                     | `packages/contracts/src/gen/proto/`                |
| `contracts/schemas/*.json` | `cargo typify`                                                   | `crates/clipmill-contracts/src/gen/schemas/`       |
| `contracts/schemas/*.json` | `datamodel-code-generator` (pydantic v2, via `uvx`)              | `workers/sdk/src/clipmill_worker_sdk/gen/schemas/` |
| `contracts/schemas/*.json` | `json-schema-to-typescript` (via pnpm)                           | `packages/contracts/src/gen/schemas/`              |

Run it as `just codegen`. Generated code is committed (decision R2): builds
are reproducible without the codegen toolchain, and contract changes show up
as reviewable diffs. CI regenerates and fails on drift, so `contracts/` and
the generated code can never disagree on `main`.

Prerequisites: `buf`, `cargo install cargo-typify`, `uv`, `pnpm install`.
Note `buf generate` uses remote plugins (network required at codegen time
only — never at build or run time).
