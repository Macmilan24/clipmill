#!/usr/bin/env bash
# W20 reframe gate.
#
# The camera the editor never has to argue with, in the two halves it fails in.
#
# The solver's half is arithmetic and is checked against trajectories built one
# behaviour each: a still subject reduces to two keyframes, a walker is followed
# with bounded jerk and containment at or above the floor, and a detector
# flickering by a fifteenth of the frame moves the camera by almost nothing —
# which is the chasing case ch. 18 says users punish hardest. The projections
# are checked too: the crop never leaves the picture, and the camera never
# exceeds the speed it was given.
#
# The detector's half is a model, and what matters there is that the graph this
# repository decodes is the graph the pinned file contains, that the decoding
# arithmetic is the documented one, and that the same frame twice gives the same
# boxes. A detector whose output moved with how busy the machine was would make
# two runs of one recording disagree while sharing one content address.
#
# The refusal is checked from both ends: the gate that decides nobody earned the
# frame, and the sentence a user is owed when a clip comes back centred.
set -euo pipefail
cd "$(dirname "$0")/../.."

ITERATIONS="${1:-1}"
case "$ITERATIONS" in
  ''|*[!0-9]*)
    echo "reframe-drill: iterations must be a positive integer" >&2
    exit 2
    ;;
esac
if [ "$ITERATIONS" -lt 1 ]; then
  echo "reframe-drill: iterations must be at least 1" >&2
  exit 2
fi

WEIGHTS=".cache/models/yunet-face/models/face_detection_yunet/face_detection_yunet_2023mar.onnx"
if [ ! -f "$WEIGHTS" ]; then
  echo "reframe-drill: pinned weights missing; run ./tools/fetch-models.sh yunet-face" >&2
  exit 2
fi
if [ ! -x .cache/bin/ffmpeg ]; then
  echo "reframe-drill: .cache/bin/ffmpeg is missing; run ./tools/fetch-ffmpeg.sh" >&2
  exit 2
fi

echo "==> the solver, and the gate that refuses to use it"
cargo test -p clipmill-reframe

echo "==> the stage is registered, keyed, and implemented"
cargo test -p clipmilld --lib recipes:: -- --nocapture
cargo test -p clipmilld --lib implementations:: -- --nocapture

echo "==> the detector, its decoding, and its determinism ($ITERATIONS iterations)"
for iteration in $(seq 1 "$ITERATIONS"); do
  echo "reframe-drill: iteration $iteration/$ITERATIONS"
  uv run --project workers/faces pytest workers/faces/tests -q
done

echo "reframe-drill: OK ($ITERATIONS iterations; solver goldens, bounded jerk, containment, fit-with-reason, detector determinism)"
