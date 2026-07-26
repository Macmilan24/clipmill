#!/usr/bin/env bash
# W13 render gate — the first-slice milestone.
#
# Renders the published first-slice Edit IR through the real pipeline and
# checks what came out: a 1080x1920 H.264 clip at 30000/1001 with the frame
# count the plan pinned, burned karaoke captions from the pinned font, SRT and
# WebVTT sidecars carrying the same words, loudness within 0.5 LU of -14 LUFS,
# and a manifest whose digests match the files it names. Then the properties
# that make it trustworthy: the same document renders to the same bytes in a
# store that never saw it, a repeat is a cache identity rather than a
# re-encode, re-explaining an edit changes nothing, a render with no rights
# attestation is refused, and a killed daemon finishes inside the recovery SLO.
#
# Leaves a watchable clip in target/render-demo/ so the milestone is a file
# rather than a passing test.
set -euo pipefail
cd "$(dirname "$0")/../.."

ITERATIONS="${1:-1}"
case "$ITERATIONS" in
  ''|*[!0-9]*)
    echo "render-drill: iterations must be a positive integer" >&2
    exit 2
    ;;
esac
if [ "$ITERATIONS" -lt 1 ]; then
  echo "render-drill: iterations must be at least 1" >&2
  exit 2
fi
for tool in .cache/bin/ffmpeg .cache/bin/ffprobe; do
  if [ ! -x "$tool" ]; then
    echo "render-drill: $tool is missing; run ./tools/fetch-ffmpeg.sh" >&2
    exit 2
  fi
done
if [ ! -f .cache/fonts/Inter-Bold.ttf ]; then
  echo "render-drill: the pinned caption font is missing; run ./tools/fetch-ffmpeg.sh" >&2
  exit 2
fi

export CLIPMILL_RENDER_DEMO_DIR="${CLIPMILL_RENDER_DEMO_DIR:-$PWD/target/render-demo}"

echo "==> render conformance ($ITERATIONS iterations)"
for iteration in $(seq 1 "$ITERATIONS"); do
  echo "render-drill: iteration $iteration/$ITERATIONS"
  # Serial: each render pins one encoder thread and the byte-stability case
  # runs two daemons of its own.
  cargo test -p clipmilld --test render_clip -- --ignored --nocapture --test-threads=1
done

echo "render-drill: OK ($ITERATIONS iterations; profile, captions, sidecars, loudness, manifest, byte stability, warm identity, refusals, kill recovery)"
echo "render-drill: the first slice is at $CLIPMILL_RENDER_DEMO_DIR/clip.mp4"
