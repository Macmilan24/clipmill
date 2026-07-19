#!/usr/bin/env bash
# W5 source-evidence and media-conformance gate.
#
# Uses the pinned FFmpeg/FFprobe sidecars, registers real local media through
# Protobuf IPC, proves warm observation and artifact-cache hits, verifies the
# published source map, detects post-registration mutation, and rejects hostile
# paths and malformed media. Repetition is used by the no-network smoke job.
set -euo pipefail
cd "$(dirname "$0")/../.."

ITERATIONS="${1:-1}"
case "$ITERATIONS" in
  ''|*[!0-9]*)
    echo "media-drill: iterations must be a positive integer" >&2
    exit 2
    ;;
esac
if [ "$ITERATIONS" -lt 1 ]; then
  echo "media-drill: iterations must be at least 1" >&2
  exit 2
fi
for tool in .cache/bin/ffmpeg .cache/bin/ffprobe; do
  if [ ! -x "$tool" ]; then
    echo "media-drill: $tool is missing; run ./tools/fetch-ffmpeg.sh" >&2
    exit 2
  fi
done

echo "==> source evidence and media conformance ($ITERATIONS iterations)"
for iteration in $(seq 1 "$ITERATIONS"); do
  echo "media-drill: iteration $iteration/$ITERATIONS"
  cargo test -p clipmilld --test source_evidence -- --ignored --nocapture
done
echo "media-drill: OK ($ITERATIONS iterations; source cache, mapping, CAS, mutation, hostile input)"
