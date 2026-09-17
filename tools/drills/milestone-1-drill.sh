#!/usr/bin/env bash
# Milestone 1 gate: one selected clip survives the complete workflow.
#
# The plan's exit for the milestone is a scenario, not a property, so this is
# the scenario run for real. From an older project while a newer one exists,
# a clip several minutes into the recording is chosen, played and scrubbed,
# corrected, trimmed at both ends, undone and redone, carried through a
# daemon restart, and exported — and the delivered file is decoded to check
# it holds the footage that was asked for, at the timing that was asked for,
# with the correction in both caption outputs and the revision that was
# reviewed. It runs through the shell's own daemon link and media door, the
# bridge the renderer stands on, against a spawned daemon and the real
# worker fleet.
#
# The recording is synthesized here: a talk from the platform voice over a
# picture whose colour names its own source second, which is what lets the
# delivered frames be read back without OCR. That needs macOS for the voice,
# the pinned FFmpeg, the worker environments (`just setup`) and the fetched
# weights (`tools/fetch-models.sh`). None of that exists in CI, so this is a
# local gate, run before a milestone is called done.
set -euo pipefail
cd "$(dirname "$0")/../.."

for tool in .cache/bin/ffmpeg .cache/bin/ffprobe; do
  if [ ! -x "$tool" ]; then
    echo "milestone-1-drill: $tool is missing; run ./tools/fetch-ffmpeg.sh" >&2
    exit 2
  fi
done
if [ ! -f .cache/fonts/Inter-Bold.ttf ]; then
  echo "milestone-1-drill: the pinned caption font is missing; run ./tools/fetch-ffmpeg.sh" >&2
  exit 2
fi
if ! command -v say >/dev/null 2>&1; then
  echo "milestone-1-drill: the recording needs the macOS synthesizer (say)" >&2
  exit 2
fi
for family in vad asr-whispercpp align shots faces; do
  if [ ! -d "workers/$family/.venv" ]; then
    echo "milestone-1-drill: workers/$family has no environment; run just setup" >&2
    exit 2
  fi
done
for model in silero-vad whisper-base wav2vec2-ctc-en yunet-face; do
  if [ ! -d ".cache/models/$model" ]; then
    echo "milestone-1-drill: the $model weights are missing; run tools/fetch-models.sh" >&2
    exit 2
  fi
done

echo "==> building the daemon the shell spawns"
cargo build -p clipmilld --bin clipmilld

WORK="${CLIPMILL_M1_WORK:-$PWD/target/milestone-1}"
mkdir -p "$WORK"
RECORDING="$WORK/talk.mp4"
if [ ! -f "$RECORDING" ]; then
  echo "==> synthesizing the recording"
  tools/fixtures/make-long-recording.sh "$RECORDING"
else
  echo "==> reusing the recording at $RECORDING"
fi

echo "==> the scenario, through the shell's bridge against a real daemon and workers"
CLIPMILL_M1_RECORDING="$RECORDING" \
CLIPMILL_M1_DELIVERED_DIR="$WORK/delivered" \
  cargo test -p clipmill-shell --test milestone_1 -- --ignored --nocapture

echo "milestone-1-drill: OK (older project beside a newer one, clip minutes in, play and scrub, name corrected in both presentations, both ends trimmed, undo and redo, restart, export of the reviewed revision, delivered frames decoded to their source seconds, both caption outputs, saved revision)"
echo "milestone-1-drill: the delivered clip is in $WORK/delivered"
