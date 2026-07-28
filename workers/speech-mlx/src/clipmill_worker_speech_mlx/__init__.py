"""The accelerated speech path, behind the contracts the portable one publishes.

One family rather than two. The plan sketched `workers/asr-mlx/` and an MLX
aligner inside `workers/align/`, written before it was clear that one library
ships both Qwen3 speech models: splitting them would duplicate a fifteen-package
macOS-only dependency tree across two environments to serve one model family,
which is the opposite of what per-family environments are for (book ch. 9).
So this worker declares two capabilities and the daemon leases it either.

Nothing downstream can tell which implementation ran. `speech.asr.v1` and
`speech.alignment.v1` say the same things here as they do on whisper.cpp and
the CTC aligner — rational ticks, a confidence distribution, an explicit
coverage statement, and timing that belongs to alignment rather than to a
decoder. What differs is the producer identity and the model digest, which is
exactly the difference that must reach the artifact key: a transcript
recognized by Qwen3 is not the same observation as one recognized by whisper,
and the two must never share a content address.
"""

from __future__ import annotations

import argparse
import logging
import os
import signal
import threading
from pathlib import Path

from clipmill.ipc.v1 import daemon_pb2
from clipmill_worker_sdk import (
    DeterministicTaskError,
    TaskContext,
    WorkerClient,
    WorkerConfiguration,
    WorkerIdentity,
)
from clipmill_worker_sdk.artifacts import ArtifactVerificationError, artifact_file
from clipmill_worker_sdk.audio import AUDIO_DESCRIPTOR, AUDIO_PAYLOAD, PcmAudio, read_pcm_audio
from clipmill_worker_sdk.confidence import distribution
from clipmill_worker_sdk.documents import canonical_bytes
from clipmill_worker_sdk.gen.schemas.speech_alignment import (
    Confidence as AlignmentConfidence,
)
from clipmill_worker_sdk.gen.schemas.speech_alignment import (
    Coverage as AlignmentCoverage,
)
from clipmill_worker_sdk.gen.schemas.speech_alignment import (
    InvalidRegion as AlignmentInvalidRegion,
)
from clipmill_worker_sdk.gen.schemas.speech_alignment import (
    Producer as AlignmentProducer,
)
from clipmill_worker_sdk.gen.schemas.speech_alignment import (
    SpeechAlignment,
    UnalignedSpan,
    Word,
)
from clipmill_worker_sdk.gen.schemas.speech_asr import (
    AsrSegment,
    Confidence,
    Coverage,
    Decoding,
    InvalidRegion,
    Producer,
    SpeechAsr,
    Token,
)
from clipmill_worker_sdk.gen.schemas.speech_vad import SpeechVad
from clipmill_worker_sdk.inputs import LeaseInputs, MissingInputError
from clipmill_worker_sdk.ticks import (
    samples_to_ticks,
    samples_to_ticks_ceil,
    seconds_to_ticks,
    ticks_to_samples,
)
from clipmill_worker_sdk.weights import ModelVerificationError, VerifiedModel, require_model

from .languages import UNDETERMINED, UnsupportedLanguage
from .runtime import MlxUnavailable, implementation, require_mlx

__version__ = "0.1.0"
CAPABILITIES = ("speech-asr", "speech-align")
ASR_FILE = "asr.json"
VAD_FILE = "vad.json"
ALIGNMENT_FILE = "alignment.json"
KEY_VERSION = "clipmill.speech-stage.v1"
# The rendition every speech stage reads, named so the lease can be searched by
# what an artifact is rather than by where it happened to land in a list.
AUDIO_KIND = "media.audio_16k.v1"
SAMPLE_RATE = 16_000
DEFAULT_MIN_SCORE = 0.05
#: Both models see whole utterances, so the decode window is the whole speech
#: segment. Qwen3-ASR chunks internally past twenty minutes; nothing here comes
#: close, and imposing a second window on top would cut sentences the model can
#: read in one piece.
MAX_WINDOW_SECONDS = 60

LOGGER = logging.getLogger(__name__)


def execute_asr(context: TaskContext) -> tuple[str, ...]:
    """Recognition, on the accelerated path."""

    from clipmill_worker_sdk.batching import DecodeWindow, decode_windows

    from .recognizer import Qwen3Recognizer

    context.cancellation.raise_if_cancelled()
    payload = _payload(context, "speech-asr")
    model = _model(context, "asr")
    inputs = _inputs(context)
    audio_input = inputs.require(AUDIO_KIND)
    audio = _audio(audio_input)
    vad_id, activity = _voice_activity(inputs)

    windows = decode_windows(
        [
            (
                ticks_to_samples(entry.start_ticks, audio.sample_rate),
                min(ticks_to_samples(entry.end_ticks, audio.sample_rate), audio.sample_count),
            )
            for entry in activity.segments
        ],
        audio.sample_rate,
        MAX_WINDOW_SECONDS,
    )
    samples = _samples(audio)

    recognizer = Qwen3Recognizer(model.root, audio.sample_rate)
    language, language_confidence = _language(recognizer, payload, samples, windows)

    segments: list[AsrSegment] = []
    undecodable: list[DecodeWindow] = []
    for index, window in enumerate(windows):
        context.cancellation.raise_if_cancelled()
        context.report_progress("speech_segments", index, len(windows))
        decoded = recognizer.decode(samples[window.start_sample : window.end_sample])
        if not decoded.text:
            # Speech the recognizer would not turn into text. Reported, not
            # dropped: a consumer that read the gap as silence would place a
            # cut inside a sentence.
            undecodable.append(window)
            continue
        p50, p10 = distribution([token.probability for token in decoded.tokens])
        segments.append(
            AsrSegment(
                index=len(segments),
                vad_segment_index=window.vad_segment_index,
                hint_start_ticks=samples_to_ticks(window.start_sample, audio.sample_rate),
                hint_end_ticks=samples_to_ticks_ceil(window.end_sample, audio.sample_rate),
                text=decoded.text,
                confidence=Confidence(p50=round(p50, 4), p10=round(p10, 4)),
                tokens=[
                    Token(text=token.text, confidence=round(token.probability, 4))
                    for token in decoded.tokens
                ],
            )
        )
    context.report_progress("speech_segments", len(windows), len(windows))

    document = SpeechAsr(
        schema_version="clipmill.speech.asr.v1",
        source_fingerprint=payload.source_fingerprint or audio.source_fingerprint,
        audio_artifact_id=audio_input.artifact_id,
        vad_artifact_id=vad_id,
        producer=Producer(
            stage="speech-asr",
            implementation=f"clipmill-worker-speech-mlx@{__version__}"
            f"+{implementation()}/{model.name}",
            model_digest=model.digest,
        ),
        language=language,
        language_confidence=language_confidence,
        decoding=Decoding(
            strategy="greedy",
            temperature=0.0,
            beam_size=0,
            conditioned_on_previous=payload.recognition.conditioned_on_previous,
        ),
        # Stated once, at the top, by every recognizer: word timing is
        # alignment's, and these intervals are the windows the decoder saw.
        timing_authority="forced_alignment",
        coverage=Coverage(
            start_ticks=0,
            end_ticks=audio.duration_ticks,
            analyzed=True,
            decoded_ticks=sum(
                segment.hint_end_ticks - segment.hint_start_ticks for segment in segments
            ),
            sampling_plan="asr-vad-batched",
        ),
        segments=segments,
        invalid_regions=[
            InvalidRegion(
                start_ticks=samples_to_ticks(window.start_sample, audio.sample_rate),
                end_ticks=samples_to_ticks_ceil(window.end_sample, audio.sample_rate),
                reason="decode_failed",
                detail="the recognizer returned no text for this speech segment",
            )
            for window in undecodable
        ],
    )
    context.staging.write_bytes(ASR_FILE, canonical_bytes(document))
    return (ASR_FILE,)


def execute_align(context: TaskContext) -> tuple[str, ...]:
    """Forced alignment, on the accelerated path."""

    from .aligner import AlignmentImpossible, Qwen3Aligner

    context.cancellation.raise_if_cancelled()
    payload = _payload(context, "speech-align")
    model = _model(context, "forced-align")
    inputs = _inputs(context)
    audio_input = inputs.require(AUDIO_KIND)
    audio = _audio(audio_input)
    asr_id, recognized = _recognition(inputs)

    aligner = Qwen3Aligner(model.root, audio.sample_rate)
    minimum_score = payload.alignment.min_score or DEFAULT_MIN_SCORE
    samples = _samples(audio)

    words: list[Word] = []
    unaligned: list[UnalignedSpan] = []
    aligned_ticks = 0
    for position, segment in enumerate(recognized.segments):
        context.cancellation.raise_if_cancelled()
        context.report_progress("utterances", position, len(recognized.segments))
        start = ticks_to_samples(segment.hint_start_ticks, audio.sample_rate)
        end = min(ticks_to_samples(segment.hint_end_ticks, audio.sample_rate), int(samples.size))
        try:
            placed = aligner.align(
                samples[start:end],
                segment.text,
                recognized.language,
            )
        except (AlignmentImpossible, UnsupportedLanguage) as error:
            unaligned.append(
                UnalignedSpan(
                    segment_index=segment.index,
                    text=segment.text,
                    reason="audio_unavailable"
                    if isinstance(error, AlignmentImpossible)
                    else "no_scoreable_text",
                    detail=str(error),
                )
            )
            continue
        first = len(words)
        for word_index, word in enumerate(placed):
            p50, p10 = distribution(list(word.scores))
            if p50 < minimum_score:
                # The model always answers; the score is what says whether to
                # believe it.
                unaligned.append(
                    UnalignedSpan(
                        segment_index=segment.index,
                        word_index=word_index,
                        text=word.text,
                        reason="score_below_threshold",
                        detail=f"median boundary score {p50:.4f} is below {minimum_score:.4f}",
                    )
                )
                continue
            # The model places words within the window it was given, so the
            # window's own offset is what turns a local time into a recording
            # time. Seconds enter through the one door that converts them.
            words.append(
                Word(
                    index=len(words),
                    segment_index=segment.index,
                    text=word.text,
                    start_ticks=segment.hint_start_ticks + seconds_to_ticks(word.start_ms / 1000.0),
                    end_ticks=segment.hint_start_ticks + seconds_to_ticks(word.end_ms / 1000.0),
                    confidence=AlignmentConfidence(p50=round(p50, 4), p10=round(p10, 4)),
                )
            )
        if len(words) > first:
            aligned_ticks += words[-1].end_ticks - words[first].start_ticks
    context.report_progress("utterances", len(recognized.segments), len(recognized.segments))

    placed_segments = {word.segment_index for word in words}
    refused = {span.segment_index for span in unaligned}
    document = SpeechAlignment(
        schema_version="clipmill.speech.alignment.v1",
        source_fingerprint=payload.source_fingerprint or audio.source_fingerprint,
        audio_artifact_id=audio_input.artifact_id,
        asr_artifact_id=asr_id,
        producer=AlignmentProducer(
            stage="speech-align",
            implementation=f"clipmill-worker-speech-mlx@{__version__}"
            f"+{implementation()}/{model.name}",
            model_digest=model.digest,
        ),
        frame_ticks=seconds_to_ticks(aligner.resolution_ms / 1000.0),
        coverage=AlignmentCoverage(
            start_ticks=0,
            end_ticks=audio.duration_ticks,
            analyzed=True,
            aligned_ticks=aligned_ticks,
            sampling_plan="align-per-utterance",
        ),
        words=words,
        unaligned=unaligned,
        invalid_regions=[
            AlignmentInvalidRegion(
                start_ticks=segment.hint_start_ticks,
                end_ticks=segment.hint_end_ticks,
                reason="alignment_unavailable",
                detail="no word in this utterance could be placed in the audio",
            )
            for segment in recognized.segments
            if segment.index in refused and segment.index not in placed_segments
        ],
    )
    context.staging.write_bytes(ALIGNMENT_FILE, canonical_bytes(document))
    return (ALIGNMENT_FILE,)


def execute(context: TaskContext) -> tuple[str, ...]:
    """Whichever of this family's two stages was leased."""

    if context.lease.kind == "speech-asr":
        return execute_asr(context)
    if context.lease.kind == "speech-align":
        return execute_align(context)
    raise DeterministicTaskError(f"{context.lease.kind} is not a stage this worker serves")


def _model(context: TaskContext, capability: str) -> VerifiedModel:
    try:
        require_mlx()
        return require_model(context.lease, capability)
    except (ModelVerificationError, MlxUnavailable) as error:
        # Both are permanent for this machine: a retry loads the same weights
        # on the same absent accelerator.
        raise DeterministicTaskError(str(error)) from error


def _inputs(context: TaskContext) -> LeaseInputs:
    """What this lease delivered, indexed by kind.

    Everything a stage reads arrives here rather than through its payload: a
    worker may open exactly what its lease named, and the payload is hashed into
    the artifact key — an address there would be present when the stage runs alone
    and absent when it runs inside an analysis, which is one observation with two
    addresses.
    """

    try:
        return LeaseInputs(context)
    except MissingInputError as error:
        raise DeterministicTaskError(str(error)) from error


def _audio(found) -> PcmAudio:
    artifact = found.artifact
    return read_pcm_audio(
        artifact,
        artifact_file(artifact, AUDIO_PAYLOAD),
        artifact_file(artifact, AUDIO_DESCRIPTOR),
        expect_sample_rate=SAMPLE_RATE,
        expect_channels=1,
    )


def _samples(audio: PcmAudio):
    import numpy as np

    return np.frombuffer(audio.frames, dtype="<i2").astype(np.float32) / 32768.0


def _payload(context: TaskContext, stage: str) -> daemon_pb2.SpeechStagePayloadV1:
    payload = daemon_pb2.SpeechStagePayloadV1()
    try:
        payload.ParseFromString(context.lease.payload)
    except Exception as error:
        raise DeterministicTaskError("task payload is not a speech stage payload") from error
    if payload.key_version != KEY_VERSION or payload.stage != stage:
        raise DeterministicTaskError(f"task payload does not describe {stage}")
    return payload


def _voice_activity(inputs: LeaseInputs) -> tuple[str, SpeechVad]:
    """The speech boundaries this decode is confined to, found by kind."""

    raw, artifact_id = _read(inputs, "speech.vad.v1", VAD_FILE)
    activity = SpeechVad.model_validate_json(raw)
    if not activity.coverage.analyzed:
        # A pass that never ran is not a recording with no speech in it.
        raise DeterministicTaskError("voice activity was never analyzed for this audio")
    return (artifact_id, activity)


def _recognition(inputs: LeaseInputs) -> tuple[str, SpeechAsr]:
    """The text this pass places in time, found by kind."""

    raw, artifact_id = _read(inputs, "speech.asr.v1", ASR_FILE)
    recognized = SpeechAsr.model_validate_json(raw)
    if recognized.timing_authority != "forced_alignment":
        raise DeterministicTaskError("the recognition artifact does not defer timing to alignment")
    return (artifact_id, recognized)


def _read(inputs: LeaseInputs, kind: str, file: str) -> tuple[str, str]:
    try:
        found = inputs.require(kind)
    except MissingInputError as error:
        raise DeterministicTaskError(str(error)) from error
    try:
        return (
            artifact_file(found.artifact, file).read_text(encoding="utf-8"),
            found.artifact_id,
        )
    except ArtifactVerificationError as error:
        raise DeterministicTaskError(str(error)) from error


def _language(recognizer, payload, samples, windows) -> tuple[str, float | None]:
    """The language, decided once for the whole recording."""

    requested = payload.recognition.language
    if requested:
        try:
            recognizer.use_language(requested)
        except UnsupportedLanguage as error:
            raise DeterministicTaskError(str(error)) from error
        return (requested, None)
    if not windows:
        return (UNDETERMINED, None)
    first = windows[0]
    language, probability = recognizer.detect_language(
        samples[first.start_sample : first.end_sample]
    )
    if language != UNDETERMINED:
        recognizer.use_language(language)
    return (language, round(probability, 4))


def main() -> int:
    parser = argparse.ArgumentParser(description="ClipMill MLX speech worker")
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

    # Refused here rather than at lease time. A worker that registers on a
    # machine without MLX would be handed accelerated tasks it can only fail.
    require_mlx()

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
            family="speech-mlx",
            capabilities=CAPABILITIES,
            backend="mlx",
            cpu_threads=1,
            # Unified memory: the larger of the two models plus its runtime,
            # since this process may be leased either stage but never both at
            # once.
            max_memory_bytes=3712 * 1024 * 1024,
        )
    )
    if arguments.once:
        client.run_one(execute)
    else:
        client.run(execute, stop)
    return 0


__all__ = [
    "CAPABILITIES",
    "__version__",
    "execute",
    "execute_align",
    "execute_asr",
    "main",
]
