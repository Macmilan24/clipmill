#!/usr/bin/env python3
"""Run the whole speech chain over a fixture whose word timing is known.

The three stages are separately unit-tested against hand-written inputs, which
is where the algorithms are checked. This is the other half: the real models,
the real audio, and the four properties the phase's gate is written in terms
of.

    voice activity  finds the utterances the fixture was built from
    recognition     returns the words the fixture was built from
    alignment       places them within 120 ms of where they were placed
    all three       produce byte-identical documents on a second run

The last one is what makes a cached transcript worth caching. It is also the
one most likely to break quietly, because a run that is merely *usually*
deterministic passes every other check here.

    tools/drills/speech_conformance.py <fixture-dir> --models <dir>
"""

from __future__ import annotations

import argparse
import json
import statistics
import sys
import tomllib
import wave
from dataclasses import dataclass
from pathlib import Path

import numpy as np
from clipmill.worker.v1 import worker_pb2
from clipmill_worker_align.ctc import AlignmentImpossible, forced_align
from clipmill_worker_align.vocabulary import Vocabulary
from clipmill_worker_align.wav2vec2 import FRAME_STRIDE_SAMPLES, Wav2Vec2Ctc
from clipmill_worker_asr_whispercpp.engine import WhisperCppRecognizer
from clipmill_worker_sdk.batching import decode_windows
from clipmill_worker_sdk.confidence import distribution
from clipmill_worker_sdk.ticks import samples_to_ticks, seconds_to_ticks, ticks_to_samples
from clipmill_worker_sdk.weights import VerifiedModel, verify_model
from clipmill_worker_vad.segmentation import SegmentationParameters, segment
from clipmill_worker_vad.silero import SileroVoiceActivity

TICKS_PER_SECOND = 90_000
# The plan's bar: half a frame of video at 30 fps is 16 ms, and a caption cue
# that is a tenth of a second early reads as early. 120 ms is the point past
# which a word-snapped trim starts cutting into speech.
TIMING_BAR_MS = 120
# How much of the fixture the recognizer must return before a timing number
# means anything. Not 100%: the fixture is spoken by whatever voice the
# platform ships, and they do not speak equally clearly — macOS `say` gives
# whisper-base a clean 14 of 14, while espeak-ng's "from the" comes back as
# "flum" and costs three. A gate that demanded a perfect transcript would be
# measuring the synthesizer, on a stage whose accuracy is deliberately
# independent of the recognizer's.
RECOGNITION_BAR = 0.7
# Voice activity's published defaults, so the drill exercises what ships.
THRESHOLD = 0.5
MIN_SPEECH_TICKS = 9_000
MIN_SILENCE_TICKS = 27_000
SPEECH_PAD_TICKS = 2_700


@dataclass
class Failure:
    check: str
    detail: str


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("fixture", type=Path)
    parser.add_argument("--models", type=Path, default=Path(".cache/models"))
    parser.add_argument(
        "--registry",
        type=Path,
        default=Path("models/registry"),
        help="pinned manifests, so the drill verifies the same digests the daemon would",
    )
    parser.add_argument(
        "--implementation",
        choices=("portable", "mlx"),
        default="portable",
        help=(
            "which recognizer and aligner to run. The portable pair is what CI "
            "holds both platforms to; the accelerated pair is held to the same "
            "bar by the local attested drill, over the same fixture."
        ),
    )
    parser.add_argument(
        "--timing-out",
        type=Path,
        help="where to write what the timing check measured, for the attestation",
    )
    options = parser.parse_args()

    truth = json.loads((options.fixture / "truth.json").read_text(encoding="utf-8"))
    with wave.open(str(options.fixture / "speech.wav"), "rb") as handle:
        rate = handle.getframerate()
        total = handle.getnframes()
        frames = handle.readframes(total)
    samples = np.frombuffer(frames, dtype="<i2").astype(np.float32) / 32768.0

    failures: list[Failure] = []
    first = run_chain(samples, rate, total, options)
    # A second pass through the same weights and the same audio. Anything that
    # differs is a stage whose output a cache cannot serve.
    second = run_chain(samples, rate, total, options)
    if first != second:
        failures.append(
            Failure("determinism", "a second run over the same audio produced different output")
        )

    failures.extend(check_activity(first, truth))
    failures.extend(check_recognition(first, truth))
    failures.extend(check_timing(first, truth))

    for failure in failures:
        print(f"speech-conformance: {failure.check}: {failure.detail}", file=sys.stderr)
    if failures:
        return 1
    if options.timing_out is not None:
        # Written for the attestation, which has to carry the number rather
        # than a claim that somebody once saw it pass.
        options.timing_out.parent.mkdir(parents=True, exist_ok=True)
        options.timing_out.write_text(
            json.dumps(
                {
                    "bar_ms": TIMING_BAR_MS,
                    "implementation": options.implementation,
                    "median_error_ms": round(first["median_error_ms"]),
                    # The words the error was actually measured over, not the
                    # words the aligner emitted. An attestation that counted
                    # the latter would overstate what it checked.
                    "words": first["timed_words"],
                },
                indent=2,
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )
    print(
        f"speech-conformance: OK ({first['timed_words']} of {len(truth['words'])} fixture "
        f"words timed in {len(first['utterances'])} utterances via {options.implementation}, "
        f"median timing error {first['median_error_ms']:.0f} ms against a "
        f"{TIMING_BAR_MS} ms bar, identical on a second run)"
    )
    return 0


def run_chain(samples: np.ndarray, rate: int, total: int, options) -> dict:
    """Voice activity, then recognition, then alignment — as the DAG runs them."""

    vad_model = pinned(options, "silero-vad")

    detector = SileroVoiceActivity(vad_model, rate)
    parameters = SegmentationParameters(
        threshold=THRESHOLD,
        window_samples=detector.window_samples,
        min_speech_samples=ticks_to_samples(MIN_SPEECH_TICKS, rate),
        min_silence_samples=ticks_to_samples(MIN_SILENCE_TICKS, rate),
        speech_pad_samples=ticks_to_samples(SPEECH_PAD_TICKS, rate),
    )
    spans = segment(detector.probabilities(samples), parameters, total)
    windows = decode_windows([(span.start_sample, span.end_sample) for span in spans], rate)

    if options.implementation == "mlx":
        return run_accelerated(samples, rate, windows, options)

    asr_model = pinned(options, "whisper-base")
    align_model = pinned(options, "wav2vec2-ctc-en")
    recognizer = WhisperCppRecognizer(asr_model, weights="ggml-base.bin", language="en")
    utterances = []
    for window in windows:
        decoded = recognizer.decode(samples[window.start_sample : window.end_sample])
        p50, p10 = distribution([token.probability for token in decoded.tokens])
        utterances.append(
            {
                "start_ticks": samples_to_ticks(window.start_sample, rate),
                "end_ticks": samples_to_ticks(window.end_sample, rate),
                "text": decoded.text,
                "p50": round(p50, 4),
                "p10": round(p10, 4),
            }
        )

    acoustic = Wav2Vec2Ctc(align_model)
    vocabulary = Vocabulary.load(acoustic.vocabulary_path)
    words = []
    for index, (window, utterance) in enumerate(zip(windows, utterances, strict=True)):
        scoreable, _ = vocabulary.encode(utterance["text"].split())
        if not scoreable:
            continue
        emissions = acoustic.emissions(samples[window.start_sample : window.end_sample])
        labels, spans_in_labels = vocabulary.label_sequence(scoreable)
        try:
            placement = forced_align(emissions, labels, blank_id=vocabulary.blank_id)
        except AlignmentImpossible:
            continue
        for word, (label_start, label_end) in zip(scoreable, spans_in_labels, strict=True):
            characters = placement[label_start:label_end]
            words.append(
                {
                    "utterance": index,
                    "text": word.text,
                    "start_ticks": samples_to_ticks(
                        window.start_sample + characters[0].start_frame * FRAME_STRIDE_SAMPLES,
                        rate,
                    ),
                    "end_ticks": samples_to_ticks(
                        window.start_sample + characters[-1].end_frame * FRAME_STRIDE_SAMPLES,
                        rate,
                    ),
                }
            )
    return {"utterances": utterances, "words": words}


def run_accelerated(samples: np.ndarray, rate: int, windows, options) -> dict:
    """The same three properties, over the accelerated pair.

    Imported here rather than at the top so this file still runs on a machine
    with no MLX — which is every CI runner, and the only reason the portable
    path is the one CI measures.
    """

    from clipmill_worker_speech_mlx.aligner import AlignmentImpossible as MlxAlignmentImpossible
    from clipmill_worker_speech_mlx.aligner import Qwen3Aligner
    from clipmill_worker_speech_mlx.recognizer import Qwen3Recognizer

    asr_model = pinned(options, "qwen3-asr-mlx")
    align_model = pinned(options, "qwen3-aligner-mlx")

    recognizer = Qwen3Recognizer(asr_model.root, rate)
    recognizer.use_language("en")
    utterances = []
    for window in windows:
        decoded = recognizer.decode(samples[window.start_sample : window.end_sample])
        p50, p10 = distribution([token.probability for token in decoded.tokens])
        utterances.append(
            {
                "start_ticks": samples_to_ticks(window.start_sample, rate),
                "end_ticks": samples_to_ticks(window.end_sample, rate),
                "text": decoded.text,
                "p50": round(p50, 4),
                "p10": round(p10, 4),
            }
        )

    aligner = Qwen3Aligner(align_model.root, rate)
    words = []
    for index, (window, utterance) in enumerate(zip(windows, utterances, strict=True)):
        if not utterance["text"]:
            continue
        try:
            placed = aligner.align(
                samples[window.start_sample : window.end_sample],
                utterance["text"],
                "en",
            )
        except MlxAlignmentImpossible:
            continue
        for word in placed:
            words.append(
                {
                    "utterance": index,
                    "text": word.text,
                    "start_ticks": utterance["start_ticks"]
                    + seconds_to_ticks(word.start_ms / 1000.0),
                    "end_ticks": utterance["start_ticks"] + seconds_to_ticks(word.end_ms / 1000.0),
                }
            )
    return {"utterances": utterances, "words": words}


def pinned(options, name: str) -> VerifiedModel:
    """Verify the weights exactly as a worker would before loading them."""

    manifest = tomllib.loads((options.registry / f"{name}.toml").read_text(encoding="utf-8"))
    binding = worker_pb2.ModelBinding(
        name=name,
        root=str((options.models / name).resolve()),
        digest="sha256:" + "0" * 64,
        capability=manifest["capability"],
        files=[
            worker_pb2.ModelFile(path=f["path"], sha256=f["sha256"], bytes=f["bytes"])
            for f in manifest["files"]
        ],
    )
    return verify_model(binding)


def check_activity(result: dict, truth: dict) -> list[Failure]:
    """The fixture was built with gaps chosen to straddle the split threshold:
    within an utterance the gap is under the minimum silence, between them it
    is over. Finding a different number of utterances means the parameters and
    the audio disagree about what a pause is."""

    expected = len(truth["utterances"])
    found = len(result["utterances"])
    if found != expected:
        return [
            Failure(
                "voice activity",
                f"found {found} utterances where the fixture was built from {expected}",
            )
        ]
    return []


def matched_words(placed: list[dict], truth: list[dict]) -> list[tuple[dict, dict]]:
    """Pair placed words with the fixture words they are, in order.

    A longest common subsequence rather than a positional zip. The recognizer
    is allowed to be wrong — on Linux the platform synthesizer says "from the"
    and whisper-base hears "flum" — and pairing by position would blame the
    aligner for the recognizer's error, or worse, silently measure word 5
    against word 7's truth and call the result a timing number.

    What survives the match is exactly the set of words we know the identity
    of, which is the only set a timing measurement can honestly use.
    """

    left = [_normalize(word["text"]) for word in placed]
    right = [_normalize(word["text"]) for word in truth]
    lengths = [[0] * (len(right) + 1) for _ in range(len(left) + 1)]
    for i in range(len(left) - 1, -1, -1):
        for j in range(len(right) - 1, -1, -1):
            lengths[i][j] = (
                lengths[i + 1][j + 1] + 1
                if left[i] == right[j]
                else max(lengths[i + 1][j], lengths[i][j + 1])
            )
    pairs: list[tuple[dict, dict]] = []
    i = j = 0
    while i < len(left) and j < len(right):
        if left[i] == right[j]:
            pairs.append((placed[i], truth[j]))
            i += 1
            j += 1
        elif lengths[i + 1][j] >= lengths[i][j + 1]:
            i += 1
        else:
            j += 1
    return pairs


def _normalize(text: str) -> str:
    """The recognizer capitalizes and punctuates; holding it to the fixture's
    lowercase would be testing the fixture."""

    return text.strip(".,!?'\"").casefold()


def check_recognition(result: dict, truth: dict) -> list[Failure]:
    """How much of the fixture the recognizer actually returned.

    A fraction rather than an equality, because the platform's own voice is
    what speaks the fixture and the two platforms do not speak equally
    clearly. Holding a CPU recognizer to a perfect transcript of espeak-ng
    would be a gate that measures the synthesizer.

    It is still a real floor. A recognizer that returned nothing, or decoded
    the wrong language, or lost a whole utterance, fails here — and this is
    the check that would catch it, since timing is measured only on the words
    that survive.
    """

    spoken = [
        {"text": word}
        for word in " ".join(utterance["text"] for utterance in result["utterances"]).split()
    ]
    recognized = len(matched_words(spoken, truth["words"]))
    fraction = recognized / max(len(truth["words"]), 1)
    result["recognized_words"] = recognized
    if fraction < RECOGNITION_BAR:
        return [
            Failure(
                "recognition",
                f"recognized {recognized} of {len(truth['words'])} fixture words in order "
                f"({fraction:.0%}), below the {RECOGNITION_BAR:.0%} floor",
            )
        ]
    return []


def check_timing(result: dict, truth: dict) -> list[Failure]:
    """Word boundaries against where the fixture generator put the samples.

    Ground truth here is arithmetic, not annotation: each word was synthesized
    alone, trimmed, and placed at an offset this fixture chose. So the error
    below is a real measurement of the aligner rather than agreement with
    somebody's reading of a waveform.

    Measured only on words whose identity is known. Alignment takes text as
    given, so the aligner's accuracy and the recognizer's are separate
    questions — and this is the check that keeps them separate.
    """

    pairs = matched_words(result["words"], truth["words"])
    result["timed_words"] = len(pairs)
    fraction = len(pairs) / max(len(truth["words"]), 1)
    if fraction < RECOGNITION_BAR:
        return [
            Failure(
                "alignment",
                f"placed only {len(pairs)} of {len(truth['words'])} fixture words "
                f"({fraction:.0%}), below the {RECOGNITION_BAR:.0%} floor",
            )
        ]
    errors = []
    for placed, expected in pairs:
        errors.append(
            abs(placed["start_ticks"] - expected["start_ticks"]) / (TICKS_PER_SECOND / 1000)
        )
        errors.append(abs(placed["end_ticks"] - expected["end_ticks"]) / (TICKS_PER_SECOND / 1000))
    median = statistics.median(errors)
    result["median_error_ms"] = median
    if median > TIMING_BAR_MS:
        return [
            Failure(
                "alignment",
                f"median word-boundary error {median:.0f} ms exceeds the {TIMING_BAR_MS} ms bar",
            )
        ]
    return []


if __name__ == "__main__":
    raise SystemExit(main())
