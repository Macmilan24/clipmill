#!/usr/bin/env python3
"""Build a speech fixture whose word timing is known exactly.

A timing gate needs ground truth, and ground truth normally means somebody
annotated a recording by hand — slow, and only as accurate as the person with
the waveform. This sidesteps that: each word is synthesized on its own, the
silence around it is trimmed, and the words are laid end to end at offsets
this script chooses. The truth is therefore not measured at all. It is the
arithmetic that placed the samples.

What that buys is a bar the aligner can actually be held to. The audio differs
between macOS and Linux because the voices do, exactly as rendered output
differs between platforms (W13), so the fixture is generated per machine and
the assertions are semantic: how far, in ticks, each word boundary landed from
where this script put it.

    tools/fixtures/make-speech-fixture.py <output-dir>

Writes speech.wav (16 kHz mono PCM) and truth.json.
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
import wave
from dataclasses import dataclass
from pathlib import Path

TICKS_PER_SECOND = 90_000
SAMPLE_RATE = 16_000

# Two utterances, spoken word by word. Deliberately plain vocabulary: the
# aligner is given this text, so recognition accuracy is a separate question,
# but a fixture the recognizer mangles makes failures harder to read.
UTTERANCES = [
    ["the", "first", "slice", "renders", "from", "the", "edit", "document"],
    ["every", "timestamp", "is", "an", "integer", "tick"],
]
# Within an utterance the gap must stay under the detector's minimum silence,
# and between utterances it must exceed it — so a fixture run also proves the
# segmenter splits where a listener would and nowhere else.
GAP_WITHIN_MS = 120
GAP_BETWEEN_MS = 700
LEAD_IN_MS = 400
LEAD_OUT_MS = 500
# Silence below this is trimmed from each synthesized word.
SILENCE_FLOOR_DB = -45


@dataclass(frozen=True, slots=True)
class Placed:
    text: str
    utterance: int
    start_sample: int
    end_sample: int


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument(
        "--ffmpeg",
        default="ffmpeg",
        help="path to the pinned ffmpeg (defaults to whatever is on PATH)",
    )
    options = parser.parse_args()

    synthesizer = _synthesizer()
    if synthesizer is None:
        print(
            "make-speech-fixture: no speech synthesizer found "
            "(expected 'say' on macOS or 'espeak-ng' on Linux)",
            file=sys.stderr,
        )
        return 2

    options.output.mkdir(parents=True, exist_ok=True)
    scratch = options.output / ".words"
    if scratch.exists():
        shutil.rmtree(scratch)
    scratch.mkdir()

    silence = b"\x00\x00"
    audio = bytearray(silence * _ms(LEAD_IN_MS))
    placed: list[Placed] = []

    for index, utterance in enumerate(UTTERANCES):
        if index > 0:
            audio.extend(silence * _ms(GAP_BETWEEN_MS))
        for position, word in enumerate(utterance):
            if position > 0:
                audio.extend(silence * _ms(GAP_WITHIN_MS))
            frames = _render_word(synthesizer, word, scratch, options.ffmpeg, len(placed))
            start = len(audio) // 2
            audio.extend(frames)
            placed.append(Placed(word, index, start, len(audio) // 2))
    audio.extend(silence * _ms(LEAD_OUT_MS))

    wav_path = options.output / "speech.wav"
    with wave.open(str(wav_path), "wb") as output:
        output.setnchannels(1)
        output.setsampwidth(2)
        output.setframerate(SAMPLE_RATE)
        output.writeframes(bytes(audio))

    truth = {
        "sample_rate": SAMPLE_RATE,
        "sample_count": len(audio) // 2,
        "duration_ticks": _ticks(len(audio) // 2),
        "synthesizer": synthesizer[0],
        "gap_within_ticks": _ticks(_ms(GAP_WITHIN_MS)),
        "gap_between_ticks": _ticks(_ms(GAP_BETWEEN_MS)),
        "utterances": [
            {
                "index": index,
                "text": " ".join(words),
                "start_ticks": _ticks(
                    min(word.start_sample for word in placed if word.utterance == index)
                ),
                "end_ticks": _ticks(
                    max(word.end_sample for word in placed if word.utterance == index)
                ),
            }
            for index, words in enumerate(UTTERANCES)
        ],
        "words": [
            {
                "index": index,
                "utterance": word.utterance,
                "text": word.text,
                "start_ticks": _ticks(word.start_sample),
                "end_ticks": _ticks(word.end_sample),
            }
            for index, word in enumerate(placed)
        ],
    }
    (options.output / "truth.json").write_text(
        json.dumps(truth, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    shutil.rmtree(scratch)
    print(
        f"make-speech-fixture: {wav_path} "
        f"({truth['duration_ticks'] / TICKS_PER_SECOND:.2f}s, "
        f"{len(placed)} words in {len(UTTERANCES)} utterances, via {synthesizer[0]})"
    )
    return 0


def _synthesizer() -> tuple[str, list[str]] | None:
    """The platform's own voice. Neither is pinned, and neither needs to be:
    the truth this fixture publishes is where the words were placed, not what
    they sound like."""

    if shutil.which("say"):
        return ("say", ["say", "-v", "Samantha", "-r", "170"])
    if shutil.which("espeak-ng"):
        return ("espeak-ng", ["espeak-ng", "-v", "en-us", "-s", "150"])
    return None


def _render_word(
    synthesizer: tuple[str, list[str]],
    word: str,
    scratch: Path,
    ffmpeg: str,
    ordinal: int,
) -> bytes:
    name, command = synthesizer
    # `say` infers the container from the extension and refuses one it does
    # not know, so both branches write a WAV.
    spoken = scratch / f"{ordinal:03d}-{word}-raw.wav"
    if name == "say":
        subprocess.run(
            [*command, "-o", str(spoken), "--file-format=WAVE", "--data-format=LEI16@16000", word],
            check=True,
        )
    else:
        subprocess.run([*command, "-w", str(spoken), word], check=True)

    trimmed = scratch / f"{ordinal:03d}-{word}.wav"
    subprocess.run(
        [
            ffmpeg,
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-i",
            str(spoken),
            "-af",
            # Trim the synthesizer's own lead-in and tail so the placement
            # arithmetic below is about the word and nothing else.
            f"silenceremove=start_periods=1:start_threshold={SILENCE_FLOOR_DB}dB:"
            f"start_silence=0:detection=rms,"
            f"areverse,"
            f"silenceremove=start_periods=1:start_threshold={SILENCE_FLOOR_DB}dB:"
            f"start_silence=0:detection=rms,"
            f"areverse",
            "-ar",
            str(SAMPLE_RATE),
            "-ac",
            "1",
            "-c:a",
            "pcm_s16le",
            str(trimmed),
        ],
        check=True,
    )
    with wave.open(str(trimmed), "rb") as handle:
        if handle.getframerate() != SAMPLE_RATE or handle.getnchannels() != 1:
            raise RuntimeError(f"{word} did not resample to 16 kHz mono")
        frames = handle.readframes(handle.getnframes())
    if not frames:
        raise RuntimeError(f"the synthesizer produced no audio for {word!r}")
    return frames


def _ms(milliseconds: int) -> int:
    return milliseconds * SAMPLE_RATE // 1000


def _ticks(samples: int) -> int:
    return samples * TICKS_PER_SECOND // SAMPLE_RATE


if __name__ == "__main__":
    raise SystemExit(main())
