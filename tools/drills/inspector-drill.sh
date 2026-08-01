#!/usr/bin/env bash
# W23 inspector gate.
#
# An editor approves a clip and gets an edit document. What has to be true of
# that step is that it is not a judgement — every decision it looks like the
# director is making was made upstream and is being read — and the way to hold
# a claim like that is a golden: the same candidate, the same boundary and the
# same evidence must produce the same document byte for byte. An editor who
# approves the same clip twice and gets two different edits has been told the
# tool is guessing.
#
# The boundary swap is the second half of the same claim. Taking the ranking's
# runner-up must produce the runner-up's document and not a re-derivation of it,
# because the whole reason the alternative ships beside the choice is that
# re-running the search to find it again is work the ranking already did.
#
# The lattice arithmetic is checked on its own because it is where a person's
# hand meets the search's rules: no amount of care with a mouse gets somebody
# within a frame of a sentence edge, so a drag resolves to an edge or it is
# refused.
#
# Then the durable half. Rejecting a clip is small work done a dozen times a
# session, and losing it is worse than losing something large because nobody
# remembers what they rejected. The decision has to survive the daemon dying
# between the write and the read.
set -euo pipefail
cd "$(dirname "$0")/../.."

ITERATIONS="${1:-1}"
case "$ITERATIONS" in
  ''|*[!0-9]*)
    echo "inspector-drill: iterations must be a positive integer" >&2
    exit 2
    ;;
esac
if [ "$ITERATIONS" -lt 1 ]; then
  echo "inspector-drill: iterations must be at least 1" >&2
  exit 2
fi

echo "==> the director: goldens, the boundary swap, and the lattice arithmetic"
for iteration in $(seq 1 "$ITERATIONS"); do
  echo "inspector-drill: iteration $iteration/$ITERATIONS"
  cargo test -p clipmill-director
done

echo "==> the stage the Inspector reads, and the frame it crops inside"
cargo test -p clipmilld --lib inspector::

echo "==> decisions survive the daemon being killed between the write and the read"
cargo test -p clipmilld --lib db::tests::

echo "==> the board's joins: an unmeasured axis, a shortfall, and the cohort's order"
pnpm --filter @clipmill/desktop test

echo "inspector-drill: OK ($ITERATIONS iterations; director goldens, boundary swap, lattice snapping, durable decisions, board joins)"
