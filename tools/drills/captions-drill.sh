#!/usr/bin/env bash
# W21 captions gate.
#
# Captions fail in a way nobody reports. A viewer who cannot keep up with a cue
# does not file a bug; they stop watching. So the things checked here are the
# things that are invisible in review.
#
# The segmentation claims to be optimal, and the property test earns that word
# by comparing the dynamic program against every possible segmentation of runs
# small enough to enumerate. A greedy segmenter agrees with the optimum on most
# inputs — which is exactly why the disagreement has to be measured.
#
# Then the two rules that are absolute rather than weighted: no cue spans a cut
# that falls in a silence, and the accessibility grouping has zero reading-speed
# violations. That grouping is what every sidecar is written from, and a sidecar
# is what a deaf viewer is left with when the burn-in is not enough.
#
# Last, the round trip. The cues published by the engine, projected into the
# Edit IR and written out by the W13 writers, must still be the same words in
# the same order with the same breaks — because the failure this whole design
# exists to prevent is a burn-in and a sidecar describing different recordings.
set -euo pipefail
cd "$(dirname "$0")/../.."

ITERATIONS="${1:-1}"
case "$ITERATIONS" in
  ''|*[!0-9]*)
    echo "captions-drill: iterations must be a positive integer" >&2
    exit 2
    ;;
esac
if [ "$ITERATIONS" -lt 1 ]; then
  echo "captions-drill: iterations must be at least 1" >&2
  exit 2
fi

echo "==> the exact segmentation, the profiles, the presets, and the validator"
for iteration in $(seq 1 "$ITERATIONS"); do
  echo "captions-drill: iteration $iteration/$ITERATIONS"
  cargo test -p clipmill-captions
done

echo "==> the goldens: no cue spans a cut, and the sidecar profile is met exactly"
cargo test -p clipmill-captions --test goldens -- --nocapture

echo "==> the projection into the Edit IR, and the W13 writers' round trip"
cargo test -p clipmill-render captions::
cargo test -p clipmill-render --test subtitles_round_trip -- --nocapture

echo "==> the stage is registered, keyed, and executed by the daemon"
cargo test -p clipmilld --lib recipes::
cargo test -p clipmilld --lib jobs::

echo "captions-drill: OK ($ITERATIONS iterations; DP optimality, goldens, zero reading-speed violations in the sidecar intent, no cue over a cut, lossless round trip)"
