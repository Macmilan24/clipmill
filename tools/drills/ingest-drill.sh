#!/usr/bin/env bash
# W11 ingest fan-out gate.
#
# Uses the pinned FFmpeg/FFprobe sidecars to run the full ingest DAG against
# generated media: every derivative publishes and re-verifies from its
# manifest, a warm re-submit resolves to identical artifact identities,
# mutated sources fail deterministically with a structured diagnostic, and a
# SIGKILLed daemon finishes the fan-out within the 30-second recovery SLO.
set -euo pipefail
cd "$(dirname "$0")/../.."

ITERATIONS="${1:-1}"
case "$ITERATIONS" in
  ''|*[!0-9]*)
    echo "ingest-drill: iterations must be a positive integer" >&2
    exit 2
    ;;
esac
if [ "$ITERATIONS" -lt 1 ]; then
  echo "ingest-drill: iterations must be at least 1" >&2
  exit 2
fi
for tool in .cache/bin/ffmpeg .cache/bin/ffprobe; do
  if [ ! -x "$tool" ]; then
    echo "ingest-drill: $tool is missing; run ./tools/fetch-ffmpeg.sh" >&2
    exit 2
  fi
done

echo "==> ingest fan-out conformance ($ITERATIONS iterations)"
for iteration in $(seq 1 "$ITERATIONS"); do
  echo "ingest-drill: iteration $iteration/$ITERATIONS"
  cargo test -p clipmilld --test ingest_fanout -- --ignored --nocapture
  # An hour-long recording, which the probe refused before it read packets as a
  # stream. Ignored by default because it builds media; this is where it runs.
  cargo test -p clipmilld --lib sources::tests::an_hour_long -- --ignored --nocapture
done
echo "ingest-drill: OK ($ITERATIONS iterations; fan-out, verification, warm identity, mutation refusal, kill recovery)"
