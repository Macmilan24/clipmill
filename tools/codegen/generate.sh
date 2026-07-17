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

# Module name for a schema file: clipmill.artifact.manifest.v1.json -> artifact_manifest
module_name() {
  basename "$1" .json | sed -e 's/^clipmill\.//' -e 's/\.v[0-9]*$//' -e 's/\./_/g'
}

mkdir -p workers/sdk/src/clipmill_worker_sdk/gen/schemas \
  crates/clipmill-contracts/src/gen/schemas \
  packages/contracts/src/gen/schemas

for schema in contracts/schemas/*.json; do
  name="$(module_name "$schema")"

  echo "==> $schema -> Rust (typify)"
  cargo typify "$schema" \
    --output "crates/clipmill-contracts/src/gen/schemas/${name}.rs"

  echo "==> $schema -> Python (datamodel-code-generator)"
  uvx --from datamodel-code-generator datamodel-codegen \
    --input "$schema" \
    --input-file-type jsonschema \
    --output "workers/sdk/src/clipmill_worker_sdk/gen/schemas/${name}.py" \
    --output-model-type pydantic_v2.BaseModel \
    --target-python-version 3.12 \
    --disable-timestamp

  echo "==> $schema -> TypeScript (json-schema-to-typescript)"
  pnpm --filter @clipmill/contracts exec json2ts \
    -i "../../$schema" \
    -o "src/gen/schemas/$(echo "$name" | tr '_' '-').ts" \
    --additionalProperties false
done

echo "==> normalize"
cargo fmt -p clipmill-contracts
pnpm exec prettier --log-level warn --write "packages/contracts/src/gen/**/*.ts"

echo "codegen: done"
