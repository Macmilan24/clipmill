#!/usr/bin/env bash
# Regenerate ALL contract code from the sources of truth in contracts/.
# Generated code is committed; CI reruns this script and fails on drift.
#
# Toolchain: buf (remote plugins: prost / python+pyi / protobuf-es),
# cargo-typify, datamodel-code-generator (via uvx), json-schema-to-typescript
# (via pnpm). Prettier normalizes generated TS so drift checks are stable.
set -euo pipefail
cd "$(dirname "$0")/../.."

echo "==> protobuf (buf generate: Rust prost / Python / TypeScript)"
buf lint
buf generate

echo "==> JSON Schema -> Rust (typify)"
cargo typify contracts/schemas/clipmill.artifact.manifest.v1.json \
  --output crates/clipmill-contracts/src/gen/schemas/artifact_manifest.rs

echo "==> JSON Schema -> Python (datamodel-code-generator)"
mkdir -p workers/sdk/src/clipmill_worker_sdk/gen/schemas
uvx --from datamodel-code-generator datamodel-codegen \
  --input contracts/schemas/clipmill.artifact.manifest.v1.json \
  --input-file-type jsonschema \
  --output workers/sdk/src/clipmill_worker_sdk/gen/schemas/artifact_manifest.py \
  --output-model-type pydantic_v2.BaseModel \
  --target-python-version 3.12 \
  --disable-timestamp

echo "==> JSON Schema -> TypeScript (json-schema-to-typescript)"
pnpm --filter @clipmill/contracts exec json2ts \
  -i ../../contracts/schemas/clipmill.artifact.manifest.v1.json \
  -o src/gen/schemas/artifact-manifest.ts \
  --additionalProperties false

echo "==> normalize"
cargo fmt -p clipmill-contracts
pnpm exec prettier --log-level warn --write "packages/contracts/src/gen/**/*.ts"

echo "codegen: done"
