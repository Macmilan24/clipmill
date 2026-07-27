#!/usr/bin/env bash
# W14 worker-plane gate: the three Phase 0 blockers, checked.
#
#   A  A stage exists only if it is registered. An unregistered kind — or a
#      registered one claiming an artifact kind that belongs to something
#      else — is refused at submit, and a stage that names a model cannot be
#      keyed without it.
#   B  A worker's declaration is a request, not a fact: it is admitted against
#      what the verified device profile measured, over-declaring is refused,
#      unified memory is budgeted once, and declared resources are covered by
#      the registration signature.
#   C  A worker reads the daemon's store without trusting it: corrupt payloads,
#      smuggled files, undeclared paths, and artifacts this lease never named
#      are all refused rather than read.
#
# Plus the model plane: the licence policy holds, and the pinned digests are
# verified rather than assumed.
set -euo pipefail
cd "$(dirname "$0")/../.."

ITERATIONS="${1:-1}"
case "$ITERATIONS" in
  ''|*[!0-9]*)
    echo "worker2-drill: iterations must be a positive integer" >&2
    exit 2
    ;;
esac
if [ "$ITERATIONS" -lt 1 ]; then
  echo "worker2-drill: iterations must be at least 1" >&2
  exit 2
fi

echo "==> stage registry, admission, and verified artifact reads ($ITERATIONS iterations)"
for iteration in $(seq 1 "$ITERATIONS"); do
  echo "worker2-drill: iteration $iteration/$ITERATIONS"
  cargo test -p clipmilld --lib recipes:: -- --nocapture
  cargo test -p clipmilld --lib worker:: -- --nocapture
  cargo test -p clipmilld --lib models:: -- --nocapture
  (cd workers/sdk && uv run pytest tests/test_artifacts.py -q)
  # A real Python worker registering over the current protocol, leasing work,
  # and surviving hard kills is what proves the preimage agrees across
  # languages: a descriptor that differs by one byte never registers at all.
  ./tools/drills/worker-drill.sh 1
done

echo "==> model plane"
python3 tools/security/check-models.py

# The policy must refuse, not merely permit. Weights end up inside what a
# creator sells, so a non-commercial licence has to fail closed.
refusal_registry="$(mktemp -d)"
trap 'rm -rf "$refusal_registry"' EXIT
sed 's/^class = "permissive"/class = "noncommercial"/; s/^spdx = "MIT"/spdx = "CC-BY-NC-4.0"/' \
  models/registry/silero-vad.toml > "$refusal_registry/silero-vad.toml"
if python3 tools/security/check-models.py --registry "$refusal_registry" >/dev/null 2>&1; then
  echo "worker2-drill: a non-commercial model was accepted by the licence policy" >&2
  exit 1
fi
echo "worker2-drill: non-commercial weights refused by policy"

# A re-export carries no licence of its own, so it declares the upstream whose
# terms it inherits. Claiming terms the upstream never granted is the way that
# declaration would rot into a rubber stamp, so the policy compares them.
awk '/^\[license.inherited_from\]/{seen=1} {if(!seen) sub(/^spdx = "Apache-2.0"/,"spdx = \"MIT\""); print}' \
  models/registry/wav2vec2-ctc-en.toml > "$refusal_registry/wav2vec2-ctc-en.toml"
rm -f "$refusal_registry/silero-vad.toml"
if python3 tools/security/check-models.py --registry "$refusal_registry" >/dev/null 2>&1; then
  echo "worker2-drill: a model claimed terms its upstream never granted" >&2
  exit 1
fi
echo "worker2-drill: a widened licence claim refused by policy"

# Verify whatever is installed. Nothing is fetched here: acquisition happens
# outside the Lock, and CI must not depend on a multi-gigabyte download.
if [ -d .cache/models ]; then
  installed=()
  for directory in .cache/models/*/; do
    [ -d "$directory" ] || continue
    installed+=("$(basename "$directory")")
  done
  if [ ${#installed[@]} -gt 0 ]; then
    ./tools/fetch-models.sh --verify-only "${installed[@]}"
  else
    echo "worker2-drill: no models installed; skipping digest verification"
  fi
else
  echo "worker2-drill: no models installed; skipping digest verification"
fi

echo "worker2-drill: OK ($ITERATIONS iterations; registry, admission, verified reads, licence policy)"
