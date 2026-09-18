#!/usr/bin/env bash
# Milestone 2 editorial gate, step 1: the windows a model reads, and the
# contracts everything it will say is held to.
#
#   The cut, without a recording. Filling to the target, ending at a topic
#   boundary, overlapping by whole sentences, and always making progress are
#   properties of arithmetic over word counts, so the cases no recording
#   contains get written out instead of waited for: a sentence longer than
#   the target, an overlap wider than a window, a topic too early to close on.
#
#   The windows, against the published indexes. Every committed
#   index.transcript.v1 fixture cut end to end and compared against a golden
#   that is itself the published editorial.windows.v1 fixture — because a
#   golden that changes is a change in what a model is shown, and the diff is
#   the review — then the guarantees a proposer relies on: every sentence in
#   some window, seams read whole, ticks and words the core's own, topics named
#   exactly, context never past the budget or the recording.
#
#   The contracts, in three languages. Windows, proposals, judgments, looks,
#   and the trace: the valid fixtures round-trip byte for byte, the invalid
#   ones are refused for the reason they were written, and the shell can read
#   what it is meant to read and nothing else.
#
#   Registration and keying, in the daemon. The stage exists only if it is
#   registered, its inputs are checked against the kind each artifact declares,
#   an index and a transcript that do not belong together are refused, the
#   budget reaches the artifact key by name, and the analyze DAG runs the stage
#   after the index and roots it through the manifest.
set -euo pipefail
cd "$(dirname "$0")/../.."

ITERATIONS="${1:-1}"
case "$ITERATIONS" in
  ''|*[!0-9]*)
    echo "editorial-drill: iterations must be a positive integer" >&2
    exit 2
    ;;
esac
if [ "$ITERATIONS" -lt 1 ]; then
  echo "editorial-drill: iterations must be at least 1" >&2
  exit 2
fi

# What the goldens looked like before this run. Compared afterwards rather than
# diffed against git, because the question is whether *this run* rewrote them —
# not whether the working tree happens to be dirty, which it legitimately is
# while somebody is changing what the goldens should say.
fingerprint() {
  find "$1" -type f -name '*.json' -print0 | sort -z | xargs -0 shasum -a 256 2>/dev/null | shasum -a 256
}
goldens_before="$(fingerprint contracts/fixtures/editorial.windows)"

echo "==> the cut, without a recording"
cargo test -p clipmill-editorial --lib -- --nocapture

# A filter that matches nothing still exits zero, so each of these is checked
# for having actually selected tests rather than for merely not failing.
ran() {
  if ! grep -qE "test result: ok\. [1-9]" "$1"; then
    echo "editorial-drill: a test filter selected nothing" >&2
    cat "$1" >&2
    exit 1
  fi
}

echo "==> registration and keying"
log="$(mktemp)"
trap 'rm -f "$log"' EXIT
for filter in recipes:: editorial:: analysis:: analyze_tests::; do
  cargo test -p clipmilld --lib "$filter" -- --nocapture | tee "$log"
  ran "$log"
done

echo "==> the published contracts, in three languages"
cargo test -p clipmill-contracts --test editorial_contracts --test analysis_contracts
(cd workers/sdk && uv run pytest -q tests/test_editorial_contracts.py tests/test_analysis_contracts.py)
pnpm --filter @clipmill/contracts test

echo "==> the windows over every published index ($ITERATIONS iterations)"
for iteration in $(seq 1 "$ITERATIONS"); do
  echo "editorial-drill: iteration $iteration/$ITERATIONS"
  # Deliberately not blessed here. A gate that regenerated its own goldens
  # would pass for any behaviour at all.
  cargo test -p clipmill-editorial --test windows -- --nocapture
done

# A gate that regenerated its own goldens would pass for any behaviour at all,
# so the one thing the tests above cannot assert about themselves is asserted
# here: nothing this run did changed the files it was checking against.
if [ "$(fingerprint contracts/fixtures/editorial.windows)" != "$goldens_before" ]; then
  echo "editorial-drill: this run rewrote the committed windows goldens" >&2
  git --no-pager diff --stat -- contracts/fixtures/editorial.windows >&2
  exit 1
fi

echo "editorial-drill: OK ($ITERATIONS iterations; cut, daemon, contracts, goldens, guarantees)"
