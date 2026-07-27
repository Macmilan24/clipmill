"""Forced alignment: when each word was said.

Its own stage, and its own artifact, because word timing must not come from a
decoder's token positions (book ch. 13). Everything word-snapped downstream —
trims, caption cues, the boundary optimizer's refusal to cut inside a word —
resolves to the frames chosen here, so this is the stage whose failures have
to be loud.

An aligner always produces a path; that is what "forced" means. So the failure
mode is not an exception but a bad answer, and the honest signal is the score.
A span the model scores below the payload's threshold is reported unaligned,
with its text, rather than published as a measurement nobody should trust.
"""

from __future__ import annotations

import argparse
import logging
import os
import signal
import threading
from pathlib import Path

import numpy as np
from clipmill.ipc.v1 import daemon_pb2
from clipmill_worker_sdk import (
    DeterministicTaskError,
    TaskContext,
    WorkerClient,
    WorkerConfiguration,
    WorkerIdentity,
)
from clipmill_worker_sdk.artifacts import ArtifactVerificationError
from clipmill_worker_sdk.audio import AUDIO_DESCRIPTOR, AUDIO_PAYLOAD, read_pcm_audio
from clipmill_worker_sdk.confidence import distribution
from clipmill_worker_sdk.documents import canonical_bytes
from clipmill_worker_sdk.gen.schemas.speech_alignment import (
    Confidence,
    Coverage,
    InvalidRegion,
    Producer,
    SpeechAlignment,
    UnalignedSpan,
    Word,
)
from clipmill_worker_sdk.gen.schemas.speech_asr import AsrSegment, SpeechAsr
from clipmill_worker_sdk.ticks import samples_to_ticks, ticks_to_samples
from clipmill_worker_sdk.weights import ModelVerificationError, require_model

from .ctc import AlignmentImpossible, TokenSpan, forced_align
from .vocabulary import Vocabulary
from .wav2vec2 import (
    FRAME_STRIDE_SAMPLES,
    IMPLEMENTATION,
    RECEPTIVE_FIELD_SAMPLES,
    Wav2Vec2Ctc,
    decode_pcm16,
)

__version__ = "0.1.0"
CAPABILITIES = ("speech-align",)
OUTPUT_FILE = "alignment.json"
ASR_FILE = "asr.json"
KEY_VERSION = "clipmill.speech-stage.v1"
SAMPLE_RATE = 16_000
DEFAULT_MIN_SCORE = 0.05

LOGGER = logging.getLogger(__name__)


def execute_align(context: TaskContext) -> tuple[str, ...]:
    context.cancellation.raise_if_cancelled()
    payload = _payload(context)
    try:
        model = require_model(context.lease, "forced-align")
    except ModelVerificationError as error:
        raise DeterministicTaskError(str(error)) from error

    audio_artifact = context.open_artifact(payload.audio_artifact_id)
    audio = read_pcm_audio(
        audio_artifact,
        context.artifact_file(audio_artifact, AUDIO_PAYLOAD),
        context.artifact_file(audio_artifact, AUDIO_DESCRIPTOR),
        expect_sample_rate=SAMPLE_RATE,
        expect_channels=1,
    )
    asr_id, recognized = _recognition(context)

    acoustic = Wav2Vec2Ctc(model)
    if acoustic.sample_rate != audio.sample_rate:
        raise DeterministicTaskError(
            f"{model.name} scores {acoustic.sample_rate} Hz audio, not {audio.sample_rate} Hz"
        )
    vocabulary = Vocabulary.load(acoustic.vocabulary_path)
    minimum_score = payload.alignment.min_score or DEFAULT_MIN_SCORE
    samples = decode_pcm16(audio.frames)

    words: list[Word] = []
    unaligned: list[UnalignedSpan] = []
    aligned_ticks = 0
    for position, segment in enumerate(recognized.segments):
        context.cancellation.raise_if_cancelled()
        context.report_progress("utterances", position, len(recognized.segments))
        placed, missed = _align_segment(
            segment=segment,
            samples=samples,
            sample_rate=audio.sample_rate,
            acoustic=acoustic,
            vocabulary=vocabulary,
            minimum_score=minimum_score,
            first_index=len(words),
        )
        words.extend(placed)
        unaligned.extend(missed)
        if placed:
            aligned_ticks += placed[-1].end_ticks - placed[0].start_ticks
    context.report_progress("utterances", len(recognized.segments), len(recognized.segments))

    document = SpeechAlignment(
        schema_version="clipmill.speech.alignment.v1",
        source_fingerprint=payload.source_fingerprint or audio.source_fingerprint,
        audio_artifact_id=payload.audio_artifact_id,
        asr_artifact_id=asr_id,
        producer=Producer(
            stage="speech-align",
            implementation=f"clipmill-worker-align@{__version__}+{IMPLEMENTATION}",
            model_digest=model.digest,
        ),
        # Stated, so a consumer can say how precise a word edge is instead of
        # implying more precision than the model measured.
        frame_ticks=samples_to_ticks(FRAME_STRIDE_SAMPLES, audio.sample_rate),
        coverage=Coverage(
            start_ticks=0,
            end_ticks=audio.duration_ticks,
            analyzed=True,
            aligned_ticks=aligned_ticks,
            sampling_plan="align-per-utterance",
        ),
        words=words,
        unaligned=unaligned,
        invalid_regions=_invalid_regions(recognized, words, unaligned),
    )
    context.staging.write_bytes(OUTPUT_FILE, canonical_bytes(document))
    return (OUTPUT_FILE,)


def _align_segment(
    *,
    segment: AsrSegment,
    samples: np.ndarray,
    sample_rate: int,
    acoustic: Wav2Vec2Ctc,
    vocabulary: Vocabulary,
    minimum_score: float,
    first_index: int,
) -> tuple[list[Word], list[UnalignedSpan]]:
    """Place one utterance's words, or say why they could not be placed."""

    scoreable, unscoreable = vocabulary.encode(segment.text.split())
    missed = [
        UnalignedSpan(
            segment_index=segment.index,
            # Where the word sits in the utterance, so assembly can put it back
            # between its neighbours rather than guessing.
            word_index=word.index,
            text=word.text,
            reason=word.reason,
        )
        for word in unscoreable
    ]
    if not scoreable:
        missed.append(
            UnalignedSpan(
                segment_index=segment.index,
                text=segment.text,
                reason="no_scoreable_text",
                detail="no character in this utterance is in the model's alphabet",
            )
        )
        return ([], missed)

    # The recognizer's hint is not word timing, but it is exactly the right
    # thing for choosing which audio to score: it is the window the decoder was
    # handed, and these words are somewhere inside it.
    start = ticks_to_samples(segment.hint_start_ticks, sample_rate)
    end = min(ticks_to_samples(segment.hint_end_ticks, sample_rate), int(samples.size))
    if end - start < RECEPTIVE_FIELD_SAMPLES:
        missed.append(
            UnalignedSpan(
                segment_index=segment.index,
                text=segment.text,
                reason="audio_unavailable",
                detail="the utterance is shorter than the model's receptive field",
            )
        )
        return ([], missed)

    emissions = acoustic.emissions(samples[start:end])
    labels, spans = vocabulary.label_sequence(scoreable)
    try:
        placement = forced_align(emissions, labels, blank_id=vocabulary.blank_id)
    except AlignmentImpossible as error:
        # More characters than frames. The words are real; their timing is not
        # available, and inventing it is the one thing this stage must not do.
        missed.append(
            UnalignedSpan(
                segment_index=segment.index,
                text=segment.text,
                reason="audio_unavailable",
                detail=str(error),
            )
        )
        return ([], missed)

    words: list[Word] = []
    for word, (label_start, label_end) in zip(scoreable, spans, strict=True):
        characters: list[TokenSpan] = placement[label_start:label_end]
        scores = [score for character in characters for score in character.scores]
        p50, p10 = distribution(scores)
        if p50 < minimum_score:
            # The path exists because a forced alignment always has one. The
            # score is what says whether to believe it.
            missed.append(
                UnalignedSpan(
                    segment_index=segment.index,
                    word_index=word.index,
                    text=word.text,
                    reason="score_below_threshold",
                    detail=f"median frame score {p50:.4f} is below {minimum_score:.4f}",
                )
            )
            continue
        first_sample = start + characters[0].start_frame * FRAME_STRIDE_SAMPLES
        last_sample = start + characters[-1].end_frame * FRAME_STRIDE_SAMPLES
        words.append(
            Word(
                index=first_index + len(words),
                segment_index=segment.index,
                text=word.text,
                start_ticks=samples_to_ticks(first_sample, sample_rate),
                end_ticks=samples_to_ticks(last_sample, sample_rate),
                confidence=Confidence(p50=round(p50, 4), p10=round(p10, 4)),
            )
        )
    return (words, missed)


def _invalid_regions(
    recognized: SpeechAsr,
    words: list[Word],
    unaligned: list[UnalignedSpan],
) -> list[InvalidRegion]:
    """Every utterance this pass placed nothing within.

    Per utterance rather than per word, because a word the aligner refused has
    no interval of its own to report and the decode window is the smallest
    span certainly known to contain it.

    An utterance where some words landed is not invalid: its timing is
    measured, and the words that were refused become interpolated spans in
    assembly, which marks them there with their own boundaries.
    """

    placed = {word.segment_index for word in words}
    refused = {span.segment_index for span in unaligned}
    return [
        InvalidRegion(
            start_ticks=segment.hint_start_ticks,
            end_ticks=segment.hint_end_ticks,
            reason="alignment_unavailable",
            detail="no word in this utterance could be placed in the audio",
        )
        for segment in recognized.segments
        if segment.index in refused and segment.index not in placed
    ]


def _payload(context: TaskContext) -> daemon_pb2.SpeechStagePayloadV1:
    payload = daemon_pb2.SpeechStagePayloadV1()
    try:
        payload.ParseFromString(context.lease.payload)
    except Exception as error:
        raise DeterministicTaskError("task payload is not a speech stage payload") from error
    if payload.key_version != KEY_VERSION or payload.stage != "speech-align":
        raise DeterministicTaskError("task payload does not describe forced alignment")
    if not payload.audio_artifact_id:
        raise DeterministicTaskError("task payload names no audio rendition")
    return payload


def _recognition(context: TaskContext) -> tuple[str, SpeechAsr]:
    inputs = list(context.lease.input_artifact_ids)
    if len(inputs) != 1:
        raise DeterministicTaskError(f"alignment takes one recognition input, not {len(inputs)}")
    artifact = context.open_artifact(inputs[0])
    try:
        raw = context.artifact_file(artifact, ASR_FILE).read_text(encoding="utf-8")
    except ArtifactVerificationError as error:
        raise DeterministicTaskError(str(error)) from error
    recognized = SpeechAsr.model_validate_json(raw)
    if recognized.timing_authority != "forced_alignment":
        # The recognizer would have to have declared its own token positions
        # authoritative for this to fail, which is the arrangement this whole
        # stage exists to prevent.
        raise DeterministicTaskError("the recognition artifact does not defer timing to alignment")
    return (inputs[0], recognized)


def main() -> int:
    parser = argparse.ArgumentParser(description="ClipMill forced alignment worker")
    parser.add_argument(
        "--identity",
        type=Path,
        default=os.environ.get("CLIPMILL_WORKER_IDENTITY"),
        required=os.environ.get("CLIPMILL_WORKER_IDENTITY") is None,
    )
    parser.add_argument("--data-dir", type=Path, default=os.environ.get("CLIPMILL_DATA_DIR"))
    parser.add_argument(
        "--worker-socket", type=Path, default=os.environ.get("CLIPMILL_WORKER_SOCKET")
    )
    parser.add_argument("--shm-socket", type=Path)
    parser.add_argument("--once", action="store_true", help="run at most one lease, then exit")
    arguments = parser.parse_args()

    worker_socket = arguments.worker_socket
    if worker_socket is None:
        if arguments.data_dir is None:
            parser.error("--worker-socket or --data-dir is required")
        worker_socket = arguments.data_dir / "run" / "clipmill-workers.sock"
    shm_socket = arguments.shm_socket or worker_socket.parent / "clipmill-shm.sock"

    logging.basicConfig(level=os.environ.get("CLIPMILL_WORKER_LOG", "INFO"))
    stop = threading.Event()
    signal.signal(signal.SIGINT, lambda *_: stop.set())
    signal.signal(signal.SIGTERM, lambda *_: stop.set())

    client = WorkerClient(
        WorkerConfiguration(
            socket_path=worker_socket,
            shm_socket_path=shm_socket,
            identity=WorkerIdentity.load(arguments.identity),
            family="speech-align",
            capabilities=CAPABILITIES,
            backend="onnx-cpu",
            cpu_threads=1,
            # The weights, plus the trellis for the longest utterance.
            max_memory_bytes=1152 * 1024 * 1024,
        )
    )
    if arguments.once:
        client.run_one(execute_align)
    else:
        client.run(execute_align, stop)
    return 0


__all__ = ["CAPABILITIES", "__version__", "execute_align", "main"]
