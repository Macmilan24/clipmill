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

# What the goldens looked like before this run. Compared afterwards rather than
# diffed against git, because the question is whether *this run* rewrote them —
# not whether the working tree happens to be dirty, which it legitimately is
# while somebody is changing what the goldens should say.
fingerprint() {
  find "$1" -type f -name '*.json' -print0 | sort -z | xargs -0 shasum -a 256 2>/dev/null | shasum -a 256
}
goldens_before="$(fingerprint contracts/fixtures/index.transcript)"

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

# A gate that regenerated its own goldens would pass for any behaviour at all,
# so the one thing the tests above cannot assert about themselves is asserted
# here: nothing this run did changed the files it was checking against.
if [ "$(fingerprint contracts/fixtures/index.transcript)" != "$goldens_before" ]; then
  echo "evidence-drill: this run rewrote the committed index goldens" >&2
  git --no-pager diff --stat -- contracts/fixtures/index.transcript >&2
  exit 1
fi

echo "evidence-drill: OK ($ITERATIONS iterations; levels, contracts, goldens, invariants)"
