#!/usr/bin/env bash
# A recording with both halves: real synthesized speech, and video to see.
#
# `make-speech-fixture.py` builds the audio whose word timing is known by
# construction, which is what the speech gates need. A full analysis needs more
# than that — it derives a proxy, a filmstrip, frames and shot cuts, and none of
# those exist for a bare WAV. So this muxes that same audio against generated
# video with real cuts in it.
#
# The video is deliberately synthetic rather than a checked-in sample: media
# never enters Git (the corpus policy), and a fixture built from a pinned
# encoder is reproducible on any machine that has the pinned encoder. The cuts
# are hard switches between two very different sources, so a shot detector that
# found none would be wrong rather than unlucky.
#
#     tools/fixtures/make-spoken-video.sh <output.mp4> [--ffmpeg PATH]
set -euo pipefail
cd "$(dirname "$0")/../.."

OUTPUT="${1:?usage: make-spoken-video.sh <output.mp4> [--ffmpeg PATH]}"
shift
FFMPEG="$PWD/.cache/bin/ffmpeg"
FFPROBE=""
# Five passes of the two utterances is a little over forty seconds. The floor
# that matters is discovery's: a candidate is fifteen seconds at minimum
# (`Parameters::DEFAULT`), so a fixture shorter than that yields an empty
# ranking and a gate that fails for a reason having nothing to do with the Lock.
REPEAT=5
while [ "$#" -gt 0 ]; do
  case "$1" in
    --ffmpeg) FFMPEG="${2:?--ffmpeg needs a path}"; shift ;;
    --ffprobe) FFPROBE="${2:?--ffprobe needs a path}"; shift ;;
    --repeat) REPEAT="${2:?--repeat needs a count}"; shift ;;
    *) echo "make-spoken-video: unknown argument $1" >&2; exit 2 ;;
  esac
  shift
done
# The pair is pinned together and installed together, so the prober is derived
# from the encoder rather than asked for twice.
[ -n "$FFPROBE" ] || FFPROBE="$(dirname "$FFMPEG")/ffprobe"
for tool in "$FFMPEG" "$FFPROBE"; do
  [ -x "$tool" ] || { echo "make-spoken-video: $tool is not executable" >&2; exit 2; }
done

scratch="$(mktemp -d /tmp/clipmill-spoken.XXXXXX)"
cleanup() {
  case "$scratch" in
    /tmp/clipmill-spoken.*) rm -rf -- "$scratch" ;;
  esac
}
trap cleanup EXIT INT TERM

# The synthesizer is the platform's, and this generator already refuses with a
# readable sentence when there is none — so it is not checked again here.
python3 tools/fixtures/make-speech-fixture.py --ffmpeg "$FFMPEG" --repeat "$REPEAT" "$scratch"

speech="$scratch/speech.wav"
[ -f "$speech" ] || { echo "make-spoken-video: the generator wrote no speech.wav" >&2; exit 1; }

# However long the speech turned out to be — it varies with the platform voice,
# which is why the speech gates assert semantically rather than by duration.
# Measured with ffprobe rather than parsed out of ffmpeg's banner: ffmpeg with
# no output file exits non-zero by design, which under `pipefail` kills the
# script for a measurement that actually succeeded.
duration="$("$FFPROBE" -v error -show_entries format=duration -of csv=p=0 "$speech")"
[ -n "$duration" ] || { echo "make-spoken-video: could not measure the speech" >&2; exit 1; }

half="$(python3 -c "import sys; print(f'{float(sys.argv[1]) / 2:.3f}')" "$duration")"

# Two visually unrelated halves concatenated: a detector that reports no cut
# here has not looked. `smptebars` and `testsrc2` differ in every block of the
# frame, which is the signal a content-based detector is built to see.
"$FFMPEG" -hide_banner -loglevel error -y \
  -f lavfi -i "smptebars=size=640x360:rate=30:duration=$half" \
  -f lavfi -i "testsrc2=size=640x360:rate=30:duration=$half" \
  -filter_complex "[0:v][1:v]concat=n=2:v=1:a=0[v]" \
  -map "[v]" -c:v libx264 -preset veryfast -pix_fmt yuv420p \
  "$scratch/video.mp4"

"$FFMPEG" -hide_banner -loglevel error -y \
  -i "$scratch/video.mp4" -i "$speech" \
  -map 0:v:0 -map 1:a:0 \
  -c:v copy -c:a aac -b:a 128k -shortest \
  "$OUTPUT"

echo "make-spoken-video: OK ($OUTPUT, ${duration}s, two shots, real speech)"
