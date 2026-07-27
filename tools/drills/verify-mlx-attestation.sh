#!/usr/bin/env bash
# Verify the committed MLX evidence without needing an accelerator.
#
# The measurement happened on hardware CI does not have. What is checked here
# is that the document is committed, complete, signed, and says what it has to
# say — that this project's accelerated path was bound by measurement on a real
# device, not by a default somebody wrote down.
set -euo pipefail
cd "$(dirname "$0")/../.."

attestation_dir="${1:-models/attestations/mlx-selection}"
expected=(
  verification-key.hex
  mlx-attestation.json
)
if [ ! -d "$attestation_dir" ]; then
  echo "mlx-attestation: $attestation_dir is missing" >&2
  echo "run 'just gate-asr-mlx <signing-key>' on a machine with the accelerator" >&2
  exit 1
fi
for filename in "${expected[@]}"; do
  path="$attestation_dir/$filename"
  if [ ! -f "$path" ] || [ -L "$path" ]; then
    echo "mlx-attestation: required public artifact $path is missing or unsafe" >&2
    exit 1
  fi
  if ! git ls-files --error-unmatch "$path" >/dev/null 2>&1; then
    echo "mlx-attestation: $path is not committed to Git" >&2
    exit 1
  fi
done
actual_count="$(find "$attestation_dir" -mindepth 1 -maxdepth 1 -type f | wc -l | tr -d ' ')"
if [ "$actual_count" != "2" ]; then
  echo "mlx-attestation: public directory must contain exactly two files" >&2
  exit 1
fi
uv run --offline --frozen --project eval/harness clipmill-eval verify-mlx-attestation \
  --attestation-dir "$attestation_dir"
echo "mlx-attestation: OK (the accelerated path's measured binding is committed and self-verifying)"
