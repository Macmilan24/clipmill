#!/usr/bin/env bash
# A recording long enough to have a middle, whose every frame says when it is.
#
# The Milestone 1 exit scenario opens a clip several minutes into a source,
# corrects a name, trims both ends, and exports — and then has to prove that
# the file it got holds the footage it asked for. A checked-in sample cannot
# be used (media never enters Git) and a synthetic pattern cannot be told
# apart from itself a minute later. So this builds one where it can: the
# audio is a spoken talk from the platform synthesizer, long enough to carry
# a clip at minute seven, and the picture is a flat colour that encodes the
# source time. Luma steps once a minute, Cb ramps once around per minute, Cr
# once per ten seconds; `signalstats` over any patch of a decoded frame gives
# the three back, and the three give the second the frame was cut from —
# through a proxy, a crop, and an encoder, with no OCR and no guessing.
#
#     tools/fixtures/make-long-recording.sh <output.mp4> [--ffmpeg PATH] [--minutes N] [--voice NAME]
set -euo pipefail
cd "$(dirname "$0")/../.."

OUTPUT="${1:?usage: make-long-recording.sh <output.mp4> [--ffmpeg PATH] [--minutes N] [--voice NAME]}"
shift
FFMPEG="$PWD/.cache/bin/ffmpeg"
MINUTES=11
VOICE="Samantha"
while [ "$#" -gt 0 ]; do
  case "$1" in
    --ffmpeg) FFMPEG="${2:?--ffmpeg needs a path}"; shift ;;
    --minutes) MINUTES="${2:?--minutes needs a count}"; shift ;;
    --voice) VOICE="${2:?--voice needs a name}"; shift ;;
    *) echo "make-long-recording: unknown argument $1" >&2; exit 2 ;;
  esac
  shift
done
[ -x "$FFMPEG" ] || { echo "make-long-recording: $FFMPEG is not executable" >&2; exit 2; }
FFPROBE="$(dirname "$FFMPEG")/ffprobe"
[ -x "$FFPROBE" ] || { echo "make-long-recording: $FFPROBE is not executable" >&2; exit 2; }
command -v say >/dev/null 2>&1 || {
  echo "make-long-recording: needs the macOS synthesizer (say); nothing else here speaks" >&2
  exit 2
}

scratch="$(mktemp -d /tmp/clipmill-long.XXXXXX)"
cleanup() {
  case "$scratch" in
    /tmp/clipmill-long.*) rm -rf -- "$scratch" ;;
  esac
}
trap cleanup EXIT INT TERM

# The talk. A guest with a name worth correcting, figures worth quoting, and
# a question or two — the shapes the discovery engine looks for. Sections are
# spoken in turn until the recording is long enough; the guest's name changes
# with each pass so no two passes are the same speech.
python3 - "$scratch" "$MINUTES" "$VOICE" <<'PY'
import subprocess
import sys
import wave
from pathlib import Path

scratch, minutes, voice = Path(sys.argv[1]), int(sys.argv[2]), sys.argv[3]

SECTIONS = [
    "Welcome back to the workshop. Today I am talking with {guest}, who spent "
    "eleven years running audio for a national radio network before starting "
    "a studio of her own. We are going to talk about the one mistake almost "
    "every new creator makes with sound, and why it costs them viewers before "
    "the first sentence is over.",
    "So {first}, let me ask you the question everybody asks me. Why does my "
    "recording sound thin even though I bought the expensive microphone? "
    "And the answer she gave me was not about the microphone at all. It was "
    "about the room. A microphone hears the room before it hears you.",
    "Here is a number that surprised me. In a survey of two thousand podcast "
    "listeners, sixty three percent said they stopped an episode in the first "
    "ninety seconds because of the sound, not the content. Sixty three percent. "
    "Nobody quits a good conversation because the video is a little soft, but "
    "they will quit it because the audio hurts.",
    "{guest} told me about a client who spent four thousand dollars on gear "
    "and then recorded in a kitchen with a tiled floor. Every word came back "
    "twice. The fix cost forty dollars: a blanket on a stand behind the "
    "speaker, and the microphone moved six inches closer. Six inches.",
    "Let me say the rule plainly, because it is the whole episode in one "
    "sentence. Get the microphone close, get the walls far, and turn the "
    "gain down until the loudest thing you say never touches the top. "
    "Everything after that is taste. Everything before it is physics.",
    "What about noise removal, I asked her. Should I run everything through "
    "the cleanup tool? And {first} said something I keep repeating. Noise "
    "removal is a bandage. It hides the wound, and it leaves a mark. If you "
    "can fix the source, fix the source. Use the tool for the day you cannot.",
    "There is a second number worth writing down. A voice recorded at a "
    "conversational level peaks around minus twelve decibels on most "
    "interfaces. If your meter is sitting at minus three, you are not louder, "
    "you are closer to the ceiling, and the first laugh will clip. Leave room.",
    "I want to close on the thing {guest} said at the very end, after we had "
    "stopped recording, which is why I am saying it now. She said the best "
    "sound she ever recorded was in a closet full of coats, with a phone. "
    "The gear did not matter. The coats did. That is the episode.",
]
GUESTS = [
    ("Priya Natarajan", "Priya"),
    ("Marisol Okonkwo", "Marisol"),
    ("Ingrid Halvorsen", "Ingrid"),
    ("Teodora Vasquez", "Teodora"),
    ("Yuki Amaral", "Yuki"),
    ("Beatrix Oyelaran", "Beatrix"),
]

# At the rate below one pass of the sections is about a hundred and fifty
# seconds of speech, so the passes are counted from the target rather than
# guessed; the voice decides the exact length, which is why the callers
# measure the file rather than assume it.
SECONDS_PER_PASS = 150
passes = max(1, -(-minutes * 60 // SECONDS_PER_PASS))
paragraphs = []
for index in range(passes):
    guest, first = GUESTS[index % len(GUESTS)]
    for section in SECTIONS:
        paragraphs.append(section.format(guest=guest, first=first))
script = scratch / "talk.txt"
script.write_text("\n\n".join(paragraphs) + "\n", encoding="utf-8")

# AIFF from the synthesizer; ffmpeg resamples it below.
subprocess.run(
    ["say", "-v", voice, "-r", "150", "-f", str(script), "-o", str(scratch / "talk.aiff")],
    check=True,
)
print(f"make-long-recording: spoke {passes} pass(es), {len(paragraphs)} paragraphs")
PY

speech="$scratch/talk.aiff"
duration="$("$FFPROBE" -v error -show_entries format=duration -of csv=p=0 "$speech")"
[ -n "$duration" ] || { echo "make-long-recording: could not measure the speech" >&2; exit 1; }
seconds="$(python3 -c "import math,sys; print(math.ceil(float(sys.argv[1])))" "$duration")"

# The picture: one colour per frame, chosen by the frame's own time.
#   Y  = 16 + 219 * (minute / 15)         one step a minute, fifteen minutes' room
#   Cb = 16 + 224 * (second-in-minute / 60)  once around per minute
#   Cr = 16 + 224 * (second-in-ten / 10)     once around per ten seconds
# Together they name the second to a tenth, and each survives an encoder's
# rounding because a step is far wider than the noise. The expression is
# evaluated per pixel, so it is evaluated on a tiny frame and the frame is
# then enlarged without interpolation — the same colour either way.
"$FFMPEG" -hide_banner -loglevel error -y \
  -f lavfi -i "color=c=black:size=32x18:rate=30000/1001:duration=$seconds" \
  -i "$speech" \
  -filter_complex "[0:v]format=yuv420p,geq=lum='16+219*min(floor(T/60)/15,1)':cb='16+224*mod(T,60)/60':cr='16+224*mod(T,10)/10',scale=1280:720:flags=neighbor[v]" \
  -map "[v]" -map 1:a:0 \
  -c:v libx264 -preset ultrafast -g 30 -pix_fmt yuv420p \
  -c:a aac -b:a 128k -ar 48000 -ac 1 -shortest \
  "$OUTPUT"

echo "make-long-recording: OK ($OUTPUT, ${duration}s of speech, time-coded picture)"
