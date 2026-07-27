#!/usr/bin/env bash
# W16 shots gate: where the camera changed.
#
#   Stage algorithms, without a video. The confidence mapping, the span
#   tiling, and the frame-size arithmetic are pure functions over arrays
#   written by hand, which is the only way to test the cases no real footage
#   contains: a cut on the first frame, a flash exactly at the minimum shot, a
#   recording that never changes by a single pixel.
#
#   The stage, over a real encode. A fixture whose cuts are known by
#   construction rather than by annotation, encoded exactly as the ingest proxy
#   is: the detector finds every cut the fixture made and none it did not, a
#   pan faster than a screen width per second is not one of them, a second pass
#   produces the same document, and a proxy that is not video is refused with a
#   reason rather than published as a recording with no cuts in it.
#
#   The plan, against the registry. A stage exists only if it is registered,
#   and this one is registered as leased-but-modelless with the right to be
#   handed one pinned binary.
set -euo pipefail
cd "$(dirname "$0")/../.."

ITERATIONS="${1:-1}"
case "$ITERATIONS" in
  ''|*[!0-9]*)
    echo "shots-drill: iterations must be a positive integer" >&2
    exit 2
    ;;
esac
if [ "$ITERATIONS" -lt 1 ]; then
  echo "shots-drill: iterations must be at least 1" >&2
  exit 2
fi

FFMPEG="${CLIPMILL_FFMPEG:-.cache/bin/ffmpeg}"
if [ ! -x "$FFMPEG" ]; then
  echo "shots-drill: no pinned decoder at $FFMPEG; run ./tools/fetch-ffmpeg.sh" >&2
  exit 1
fi

echo "==> stage algorithms and registration (no video)"
cargo test -p clipmilld --lib shots -- --nocapture
cargo test -p clipmilld --lib recipes:: -- --nocapture
cargo test -p clipmill-contracts --test shots_contracts
(cd workers/shots && uv run pytest -q)
(cd workers/sdk && uv run pytest -q tests/test_tools.py tests/test_ticks.py)

root="$PWD"
fixture="$(mktemp -d)"
trap 'rm -rf "$fixture"' EXIT

echo "==> the stage, over a real encode ($ITERATIONS iterations)"
for iteration in $(seq 1 "$ITERATIONS"); do
  echo "shots-drill: iteration $iteration/$ITERATIONS"
  # Regenerated per iteration: the encoder is the pinned one, so a fixture that
  # only worked once would be a fixture nobody could reproduce.
  python3 tools/fixtures/make-shots-fixture.py --ffmpeg "$FFMPEG" "$fixture"
  (cd tools/drills/shots-conformance && uv run python3 ../shots_conformance.py \
    "$fixture" --ffmpeg "$root/$FFMPEG")
done

echo "shots-drill: OK ($ITERATIONS iterations; algorithms, registration, detection)"
