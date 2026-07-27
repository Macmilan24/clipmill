#!/usr/bin/env bash
# W19 ranking gate: what a clip is worth, where it is cut, and which to show.
#
#   The score card, against cards written by hand. An axis nobody measured must
#   be distinguishable from one measured at zero — the whole reason the score is
#   decomposed — so the unmeasured cases are written out rather than waited for.
#
#   J against brute force. The boundary optimizer is exhaustive by design, so
#   the property is checkable directly: the chosen pair is the argmax over every
#   legal pair, and the runner-up is the second, with nothing scoring between.
#
#   Selection, against the temptation to pad. Asked for more clips than a
#   recording holds, the set comes back short with the difference accounted for.
#
#   The published contract, over the fixtures every language reads, and the
#   goldens that say which clips this system would actually show.
set -euo pipefail
cd "$(dirname "$0")/../.."

ITERATIONS="${1:-1}"
case "$ITERATIONS" in
  ''|*[!0-9]*)
    echo "ranking-drill: iterations must be a positive integer" >&2
    exit 2
    ;;
esac
if [ "$ITERATIONS" -lt 1 ]; then
  echo "ranking-drill: iterations must be at least 1" >&2
  exit 2
fi

# What the goldens looked like before this run. Compared afterwards rather than
# diffed against git, because the question is whether *this run* rewrote them —
# not whether the working tree happens to be dirty, which it legitimately is
# while somebody is changing what the goldens should say.
fingerprint() {
  find "$1" -type f -name '*.json' -print0 | sort -z | xargs -0 shasum -a 256 2>/dev/null | shasum -a 256
}
goldens_before="$(fingerprint contracts/fixtures/ranking.set)"

log="$(mktemp)"
trap 'rm -f "$log"' EXIT

# A filter that matches nothing still exits zero.
ran() {
  if ! grep -qE "test result: ok\. [1-9]" "$log"; then
    echo "ranking-drill: a test filter selected nothing" >&2
    cat "$log" >&2
    exit 1
  fi
}

echo "==> the card, the optimizer, and the selector"
for filter in scorecard:: boundary:: ranking::; do
  cargo test -p clipmill-discovery --lib "$filter" -- --nocapture | tee "$log"
  ran
done

echo "==> registration and keying"
for filter in recipes:: ranking::; do
  cargo test -p clipmilld --lib "$filter" -- --nocapture | tee "$log"
  ran
done

echo "==> the published contract, in three languages"
cargo test -p clipmill-contracts --test ranking_contracts | tee "$log"
ran
(cd workers/sdk && uv run pytest -q tests/test_ranking_contracts.py)
pnpm --filter @clipmill/contracts test

echo "==> ranking over every published cohort ($ITERATIONS iterations)"
for iteration in $(seq 1 "$ITERATIONS"); do
  echo "ranking-drill: iteration $iteration/$ITERATIONS"
  # Deliberately not blessed here. A gate that regenerated its own goldens
  # would pass for any behaviour at all.
  cargo test -p clipmill-discovery --test ranking_golden -- --nocapture | tee "$log"
  ran
done

# A gate that regenerated its own goldens would pass for any behaviour at all,
# so the one thing the tests above cannot assert about themselves is asserted
# here: nothing this run did changed the files it was checking against.
if [ "$(fingerprint contracts/fixtures/ranking.set)" != "$goldens_before" ]; then
  echo "ranking-drill: this run rewrote the committed ranking goldens" >&2
  git --no-pager diff --stat -- contracts/fixtures/ranking.set >&2
  exit 1
fi

echo "ranking-drill: OK ($ITERATIONS iterations; card, boundaries, selection, goldens)"
