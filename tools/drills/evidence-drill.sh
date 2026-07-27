#!/usr/bin/env bash
# W17 evidence gate: the structure read out of a transcript.
#
#   The levels, against transcripts written by hand. Utterance splitting,
#   sentence boundaries, cut edges, and lexical cohesion are pure functions, so
#   the cases no real recording contains get written out instead of waited for:
#   a speaker who never pauses, a recognizer that punctuates nothing, a segment
#   whose text and word count disagree, four sentences too few to segment.
#
#   The index, against the published contracts. Every committed
#   speech.transcript.v1 fixture indexed end to end and compared against a
#   golden — because a golden that changes is a change in what the system says
#   about a recording, and the diff is the review.
#
#   The invariants a consumer never rechecks. Units tile the word list, topics
#   tile the sentences, every unit lies inside coverage and resolves to words
#   somebody measured, and a second pass produces the same bytes.
#
#   Registration and keying, in the daemon. The stage exists only if it is
#   registered, its inputs are checked against the kind each artifact declares,
#   and its parameters reach the artifact key by name so a re-tune invalidates
#   what depended on it and nothing else.
set -euo pipefail
cd "$(dirname "$0")/../.."

ITERATIONS="${1:-1}"
case "$ITERATIONS" in
  ''|*[!0-9]*)
    echo "evidence-drill: iterations must be a positive integer" >&2
    exit 2
    ;;
esac
if [ "$ITERATIONS" -lt 1 ]; then
  echo "evidence-drill: iterations must be at least 1" >&2
  exit 2
fi

echo "==> the levels, without a recording"
cargo test -p clipmill-evidence --lib -- --nocapture

# A filter that matches nothing still exits zero, so each of these is checked
# for having actually selected tests rather than for merely not failing.
ran() {
  if ! grep -qE "test result: ok\. [1-9]" "$1"; then
    echo "evidence-drill: a test filter selected nothing" >&2
    cat "$1" >&2
    exit 1
  fi
}

echo "==> registration and keying"
log="$(mktemp)"
trap 'rm -f "$log"' EXIT
for filter in recipes:: evidence:: index_tests::; do
  cargo test -p clipmilld --lib "$filter" -- --nocapture | tee "$log"
  ran "$log"
done

echo "==> the published contracts, in three languages"
cargo test -p clipmill-contracts --test index_contracts
(cd workers/sdk && uv run pytest -q tests/test_index_contracts.py)
pnpm --filter @clipmill/contracts test

echo "==> the index over every published transcript ($ITERATIONS iterations)"
for iteration in $(seq 1 "$ITERATIONS"); do
  echo "evidence-drill: iteration $iteration/$ITERATIONS"
  # Deliberately not blessed here. A gate that regenerated its own goldens
  # would pass for any behaviour at all.
  cargo test -p clipmill-evidence --test golden -- --nocapture
done

# The one thing the tests above cannot assert about themselves: that the golden
# is a committed file somebody reviewed, rather than one this run produced.
if ! git diff --quiet -- contracts/fixtures/index.transcript; then
  echo "evidence-drill: the committed index goldens changed during this run" >&2
  git --no-pager diff --stat -- contracts/fixtures/index.transcript >&2
  exit 1
fi

echo "evidence-drill: OK ($ITERATIONS iterations; levels, contracts, goldens, invariants)"
