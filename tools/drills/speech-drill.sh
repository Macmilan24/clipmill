#!/usr/bin/env bash
# W15 speech gate: the chain that turns audio into an observation.
#
#   Stage algorithms, without the models. Segmentation, decode batching, CTC
#   alignment, and the vocabulary are pure functions with hand-written inputs,
#   which is the only way to test the cases no real recording contains: text
#   longer than the audio, a repeated letter that must not collapse, a speech
#   run longer than the decoder's context.
#
#   The chain, with the models. Real weights over a fixture whose word timing
#   is known by construction rather than by annotation: voice activity finds
#   the utterances the fixture was built from, recognition returns its words,
#   alignment places them within 120 ms, and a second run produces byte-
#   identical output.
#
#   Assembly, against the published contracts. The four fixtures are one
#   recording seen by four stages, so assembling the first three reproduces
#   the fourth exactly.
set -euo pipefail
cd "$(dirname "$0")/../.."

ITERATIONS="${1:-1}"
case "$ITERATIONS" in
  ''|*[!0-9]*)
    echo "speech-drill: iterations must be a positive integer" >&2
    exit 2
    ;;
esac
if [ "$ITERATIONS" -lt 1 ]; then
  echo "speech-drill: iterations must be at least 1" >&2
  exit 2
fi

FFMPEG="${CLIPMILL_FFMPEG:-.cache/bin/ffmpeg}"
MODELS="${CLIPMILL_WEIGHTS_DIR:-.cache/models}"

echo "==> stage algorithms and assembly (no models)"
cargo test -p clipmilld --lib speech:: -- --nocapture
cargo test -p clipmilld --lib recipes:: -- --nocapture
cargo test -p clipmill-contracts --test speech_contracts
(cd workers/vad && uv run pytest -q)
(cd workers/asr-whispercpp && uv run pytest -q)
(cd workers/align && uv run pytest -q)
(cd workers/sdk && uv run pytest -q tests/test_ticks.py tests/test_confidence.py tests/test_weights.py)

# Acquisition happens outside the Local Lock, so the gate verifies rather than
# fetches. A machine without the weights is one this drill cannot speak for.
for model in silero-vad whisper-base wav2vec2-ctc-en; do
  if [ ! -d "$MODELS/$model" ]; then
    echo "speech-drill: $model is not installed; run ./tools/fetch-models.sh $model" >&2
    exit 1
  fi
done
./tools/fetch-models.sh --verify-only silero-vad whisper-base wav2vec2-ctc-en

root="$PWD"
fixture="$(mktemp -d)"
trap 'rm -rf "$fixture"' EXIT

echo "==> the chain, over real audio ($ITERATIONS iterations)"
for iteration in $(seq 1 "$ITERATIONS"); do
  echo "speech-drill: iteration $iteration/$ITERATIONS"
  # Regenerated per iteration: the synthesizer is the platform's, so a fixture
  # that only works once would be a fixture nobody could reproduce.
  python3 tools/fixtures/make-speech-fixture.py --ffmpeg "$FFMPEG" "$fixture"
  (cd tools/drills/speech-conformance && uv run python3 ../speech_conformance.py \
    "$fixture" --models "$root/$MODELS" --registry "$root/models/registry")
done

echo "speech-drill: OK ($ITERATIONS iterations; algorithms, chain, assembly)"
