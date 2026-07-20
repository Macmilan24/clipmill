#!/usr/bin/env bash
# W7 measured, signed, cached device-profile gate.
set -euo pipefail
cd "$(dirname "$0")/../.."

iterations="${1:-1}"
if ! [[ "$iterations" =~ ^[1-9][0-9]*$ ]]; then
  echo "device-drill: iterations must be a positive integer" >&2
  exit 2
fi
for tool in .cache/bin/ffmpeg .cache/bin/ffprobe; do
  if [ ! -x "$tool" ]; then
    echo "device-drill: $tool is missing; run ./tools/fetch-ffmpeg.sh" >&2
    exit 2
  fi
done

echo "==> measured and signed device profile ($iterations iterations)"
for iteration in $(seq 1 "$iterations"); do
  echo "device-drill: iteration $iteration/$iterations"
  cargo test --quiet -p clipmilld --test device_profile \
    pinned_ffmpeg_profile_executes_bounded_measurements \
    -- --ignored --nocapture --test-threads=1
done
echo "device-drill: OK ($iterations iterations; runtime, CPU round trip, shared memory, signature)"
