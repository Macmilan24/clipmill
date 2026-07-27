#!/usr/bin/env bash
# gate-asr-mlx: the accelerated speech path, proved on hardware that has one.
#
#   CI cannot run this. No hosted runner has an Apple GPU, and a gate that
#   silently skipped itself would be a gate that always passes. So this follows
#   the Seed-40 pattern (R18): the measurement happens on real hardware, and
#   what reaches the repository is a small signed document CI verifies without
#   pretending to have measured anything.
#
#   What it proves, in order:
#     1. the pinned Qwen3 weights are installed and still hash to their pins;
#     2. every implementation the daemon can plan is measured over a minute of
#        speech, in a process of its own, in the environment that loads it;
#     3. the accelerated implementations ran here, the accelerator they ran on
#        became admissible, and the daemon bound every contested capability by
#        measurement rather than by falling back;
#     4. the accelerated aligner places words within the same 120 ms bar CI
#        holds the portable one to;
#     5. the result is signed and written where CI can check it.
#
#   What it does not prove is that MLX wins. It sometimes does not — on the
#   machine this was written on, whisper.cpp-base recognizes faster than a 1.7B
#   Qwen3 while the Qwen3 aligner beats the CTC one twice over. Requiring a
#   particular winner would reinstate the static per-platform default D19
#   exists to remove.
#
#   ./tools/drills/asr-mlx-drill.sh --signing-key <path> [--output-dir <dir>]
set -euo pipefail
cd "$(dirname "$0")/../.."

SIGNING_KEY=""
OUTPUT_DIR="models/attestations/mlx-selection"
while [ $# -gt 0 ]; do
  case "$1" in
    --signing-key) SIGNING_KEY="${2:-}"; shift 2 ;;
    --output-dir) OUTPUT_DIR="${2:-}"; shift 2 ;;
    *) echo "asr-mlx-drill: unknown argument $1" >&2; exit 2 ;;
  esac
done
if [ -z "$SIGNING_KEY" ]; then
  echo "asr-mlx-drill: --signing-key is required (a 0600 file of 32 hex-encoded bytes)" >&2
  echo "  the private key never enters Git; only the public half is committed" >&2
  exit 2
fi

if [ "$(uname -s)" != "Darwin" ] || [ "$(uname -m)" != "arm64" ]; then
  echo "asr-mlx-drill: this gate needs Apple silicon; there is nothing to measure here" >&2
  exit 2
fi

FFMPEG="${CLIPMILL_FFMPEG:-.cache/bin/ffmpeg}"
MODELS="${CLIPMILL_WEIGHTS_DIR:-.cache/models}"
for model in silero-vad whisper-base wav2vec2-ctc-en qwen3-asr-mlx qwen3-aligner-mlx; do
  if [ ! -d "$MODELS/$model" ]; then
    echo "asr-mlx-drill: $model is not installed; run ./tools/fetch-models.sh $model" >&2
    exit 1
  fi
done
./tools/fetch-models.sh --verify-only silero-vad whisper-base wav2vec2-ctc-en \
  qwen3-asr-mlx qwen3-aligner-mlx

root="$PWD"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
export CLIPMILL_MLX_DRILL_DIR="$work/daemon"
mkdir -p "$CLIPMILL_MLX_DRILL_DIR"

echo "==> the candidate table the daemon plans from matches the one measured here"
cargo test --quiet -p clipmilld --lib implementations:: -- --nocapture
python3 tools/bench/candidates-agree.py

echo "==> this machine's fingerprint"
cargo test --quiet -p clipmilld --test device_profile \
  mlx_drill_reports_the_hardware_fingerprint -- --ignored --test-threads=1
fingerprint="$(tr -d '[:space:]' < "$CLIPMILL_MLX_DRILL_DIR/fingerprint.txt")"
echo "asr-mlx-drill: $fingerprint"

echo "==> a minute of speech whose word timing is known by construction"
python3 tools/fixtures/make-speech-fixture.py --ffmpeg "$FFMPEG" --repeat 5 "$work/fixture"

echo "==> every implementation, measured in the environment that loads it"
(cd tools/drills/speech-conformance && uv run python3 "$root/tools/bench/speech-benchmark.py" \
  --fixture "$work/fixture" \
  --models "$root/$MODELS" \
  --registry "$root/models/registry" \
  --fingerprint "$fingerprint" \
  --output "$CLIPMILL_MLX_DRILL_DIR/state/speech-benchmark.json")

echo "==> the daemon's binding, re-measured against that benchmark"
cargo test --quiet -p clipmilld --test device_profile \
  mlx_drill_asserts_the_measured_binding -- --ignored --test-threads=1

echo "==> the accelerated aligner against the same 120 ms bar CI uses"
(cd tools/drills/speech-conformance && uv run python3 "$root/tools/drills/speech_conformance.py" \
  "$work/fixture" --models "$root/$MODELS" --registry "$root/models/registry" \
  --implementation mlx --timing-out "$CLIPMILL_MLX_DRILL_DIR/timing.json")

echo "==> signed, path-free evidence for protected main"
uv run --project eval/harness clipmill-eval attest-mlx \
  --profile "$CLIPMILL_MLX_DRILL_DIR/profile.json" \
  --timing "$CLIPMILL_MLX_DRILL_DIR/timing.json" \
  --signing-key "$SIGNING_KEY" \
  --output-dir "$OUTPUT_DIR"

uv run --project eval/harness clipmill-eval verify-mlx-attestation --attestation-dir "$OUTPUT_DIR"
echo "asr-mlx-drill: OK (attestation written to $OUTPUT_DIR)"
