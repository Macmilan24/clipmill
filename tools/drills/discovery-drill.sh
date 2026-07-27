#!/usr/bin/env bash
# W18 discovery gate: the spans worth considering, and the lattice under them.
#
#   The mesh, against recordings written by hand. A topic of one sentence, a
#   question nobody answered, a shot cut landing inside a word, two seeds that
#   expand to the same interval — the cases that decide whether a nomination is
#   honest, and the ones no real recording reliably contains.
#
#   The lattice, against Phi. Every point discovery publishes must pair with
#   something legal, or ranking would search it and find nothing; every
#   rejection must name a reason that fired, because a term recorded as never
#   firing reads like a term that was checked.
#
#   The guarantees, over the published contracts. Every committed index
#   searched end to end and compared against a reviewed golden, then the three
#   promises ranking relies on without rechecking: candidates are explicable,
#   their boundaries are legal, and every one of them is grouped.
#
#   Registration and keying, in the daemon. The stage exists only if it is
#   registered, its inputs are checked against the kind each artifact declares,
#   and the clip length it was asked for reaches the artifact key — a different
#   length is a different search, not a filter over this one.
set -euo pipefail
cd "$(dirname "$0")/../.."

ITERATIONS="${1:-1}"
case "$ITERATIONS" in
  ''|*[!0-9]*)
    echo "discovery-drill: iterations must be a positive integer" >&2
    exit 2
    ;;
esac
if [ "$ITERATIONS" -lt 1 ]; then
  echo "discovery-drill: iterations must be at least 1" >&2
  exit 2
fi

log="$(mktemp)"
trap 'rm -f "$log"' EXIT

# A filter that matches nothing still exits zero, so each run below is checked
# for having selected tests rather than merely for not failing.
ran() {
  if ! grep -qE "test result: ok\. [1-9]" "$log"; then
    echo "discovery-drill: a test filter selected nothing" >&2
    cat "$log" >&2
    exit 1
  fi
}

echo "==> the mesh, the lattice, and the clustering"
cargo test -p clipmill-discovery --lib -- --nocapture | tee "$log"
ran

echo "==> registration and keying"
for filter in recipes:: discovery:: discovery_tests::; do
  cargo test -p clipmilld --lib "$filter" -- --nocapture | tee "$log"
  ran
done

echo "==> the published contract, in three languages"
cargo test -p clipmill-contracts --test discovery_contracts | tee "$log"
ran
(cd workers/sdk && uv run pytest -q tests/test_discovery_contracts.py)
pnpm --filter @clipmill/contracts test

echo "==> discovery over every published index ($ITERATIONS iterations)"
for iteration in $(seq 1 "$ITERATIONS"); do
  echo "discovery-drill: iteration $iteration/$ITERATIONS"
  # Deliberately not blessed here. A gate that regenerated its own goldens
  # would pass for any behaviour at all.
  cargo test -p clipmill-discovery --test golden -- --nocapture | tee "$log"
  ran
done

# The one thing the tests cannot assert about themselves: that the golden is a
# committed file somebody reviewed rather than one this run produced.
if ! git diff --quiet -- contracts/fixtures/discovery.candidates; then
  echo "discovery-drill: the committed candidate goldens changed during this run" >&2
  git --no-pager diff --stat -- contracts/fixtures/discovery.candidates >&2
  exit 1
fi

echo "discovery-drill: OK ($ITERATIONS iterations; mesh, lattice, contract, goldens)"
