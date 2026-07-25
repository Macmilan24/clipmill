#!/usr/bin/env bash
# Verify the committed public evidence without requiring private Seed-40 media.
set -euo pipefail
cd "$(dirname "$0")/../.."

attestation_dir="${1:-eval/seed40}"
expected=(
  verification-key.hex
  corpus-metadata.json
  license-summary.json
  run-attestation.json
)
if [ ! -d "$attestation_dir" ]; then
  echo "phase0-attestation: $attestation_dir is missing" >&2
  echo "run 'just gate-seed40 ...' with the rights-cleared private corpus first" >&2
  exit 1
fi
for filename in "${expected[@]}"; do
  path="$attestation_dir/$filename"
  if [ ! -f "$path" ] || [ -L "$path" ]; then
    echo "phase0-attestation: required public artifact $path is missing or unsafe" >&2
    exit 1
  fi
  if ! git ls-files --error-unmatch "$path" >/dev/null 2>&1; then
    echo "phase0-attestation: $path is not committed to Git" >&2
    exit 1
  fi
done
actual_count="$(find "$attestation_dir" -mindepth 1 -maxdepth 1 -type f | wc -l | tr -d ' ')"
if [ "$actual_count" != "4" ]; then
  echo "phase0-attestation: public directory must contain exactly four files" >&2
  exit 1
fi
uv run --offline --frozen --project eval/harness clipmill-eval verify-attestation \
  --attestation-dir "$attestation_dir"
echo "phase0-attestation: OK (signed Seed-40 proof is committed and self-verifying)"
