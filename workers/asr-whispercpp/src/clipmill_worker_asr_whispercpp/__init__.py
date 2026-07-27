"""Recognition, on the universal fallback path.

This worker lands before the accelerated primary and outlives it. Everything
downstream — alignment, the transcript, discovery, captions — speaks to the
`asr` contract and not to whisper.cpp, so the daemon can put a faster
implementation behind the same contract on machines that have one, choose
between them by measured benchmark rather than by brand, and still run the
whole pipeline on a laptop with no accelerator at all.

What this stage does not produce is word timing. The intervals it publishes
are the decoder's own bookkeeping, named as hints, and the document states
once at the top that forced alignment owns timing.
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
from clipmill_worker_sdk.artifacts import ArtifactVerificationError
from clipmill_worker_sdk.audio import AUDIO_DESCRIPTOR, AUDIO_PAYLOAD, PcmAudio, read_pcm_audio
from clipmill_worker_sdk.batching import DecodeWindow, decode_windows
from clipmill_worker_sdk.confidence import distribution
from clipmill_worker_sdk.documents import canonical_bytes
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
from clipmill_worker_sdk.ticks import samples_to_ticks, samples_to_ticks_ceil, ticks_to_samples
from clipmill_worker_sdk.weights import ModelVerificationError, VerifiedModel, require_model

from .engine import IMPLEMENTATION, WhisperCppRecognizer, decode_pcm16

__version__ = "0.1.0"
CAPABILITIES = ("speech-asr",)
OUTPUT_FILE = "asr.json"
VAD_FILE = "vad.json"
KEY_VERSION = "clipmill.speech-stage.v1"
SAMPLE_RATE = 16_000

LOGGER = logging.getLogger(__name__)


def execute_asr(context: TaskContext) -> tuple[str, ...]:
    context.cancellation.raise_if_cancelled()
    payload = _payload(context)
    try:
        model = require_model(context.lease, "asr")
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
    vad_id, activity = _voice_activity(context)

    windows = decode_windows(
        [
            (
                ticks_to_samples(entry.start_ticks, audio.sample_rate),
                min(ticks_to_samples(entry.end_ticks, audio.sample_rate), audio.sample_count),
            )
            for entry in activity.segments
        ],
        audio.sample_rate,
    )
    samples = decode_pcm16(audio.frames)

    recognizer = WhisperCppRecognizer(
        model,
        language=payload.recognition.language,
        weights=weights_file(model),
    )
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

    document = _document(
        payload=payload,
        audio=audio,
        vad_artifact_id=vad_id,
        model=model,
        language=language,
        language_confidence=language_confidence,
        segments=segments,
        undecodable=undecodable,
    )
    context.staging.write_bytes(OUTPUT_FILE, canonical_bytes(document))
    return (OUTPUT_FILE,)


def weights_file(model: VerifiedModel) -> str:
    """The GGML weights among the model's pinned files.

    Found rather than hardcoded, because the registry pins two whisper models
    with different file names and a stage that named one could never be given
    the other. Ambiguity is refused: a model pinning two weight files is a
    manifest error, not a choice for this worker to make.
    """

    candidates = [name for name in model.files if name.endswith(".bin")]
    if len(candidates) != 1:
        raise DeterministicTaskError(
            f"{model.name} pins {len(candidates)} GGML weight files; expected exactly one"
        )
    return candidates[0]


def _payload(context: TaskContext) -> daemon_pb2.SpeechStagePayloadV1:
    payload = daemon_pb2.SpeechStagePayloadV1()
    try:
        payload.ParseFromString(context.lease.payload)
    except Exception as error:
        # Every way this can fail is one fact: the daemon and this worker
        # disagree about what a speech task looks like, and a retry will not
        # change that.
        raise DeterministicTaskError("task payload is not a speech stage payload") from error
    if payload.key_version != KEY_VERSION or payload.stage != "speech-asr":
        raise DeterministicTaskError("task payload does not describe recognition")
    if not payload.audio_artifact_id:
        raise DeterministicTaskError("task payload names no audio rendition")
    return payload


def _voice_activity(context: TaskContext) -> tuple[str, SpeechVad]:
    """The speech boundaries this decode is confined to, verified.

    Exactly one input, and it must be voice activity. Guessing which of
    several inputs was meant is how a stage ends up reading last week's
    artifact and publishing it under this week's key.
    """

    inputs = list(context.lease.input_artifact_ids)
    if len(inputs) != 1:
        raise DeterministicTaskError(
            f"recognition takes one voice-activity input, not {len(inputs)}"
        )
    artifact = context.open_artifact(inputs[0])
    try:
        raw = context.artifact_file(artifact, VAD_FILE).read_text(encoding="utf-8")
    except ArtifactVerificationError as error:
        raise DeterministicTaskError(str(error)) from error
    activity = SpeechVad.model_validate_json(raw)
    if not activity.coverage.analyzed:
        # A pass that never ran is not a recording with no speech in it.
        raise DeterministicTaskError("voice activity was never analyzed for this audio")
    return (inputs[0], activity)


def _language(
    recognizer: WhisperCppRecognizer,
    payload: daemon_pb2.SpeechStagePayloadV1,
    samples,
    windows: list[DecodeWindow],
) -> tuple[str, float | None]:
    """The language, decided once for the whole recording.

    Detecting per window would let one noisy utterance be decoded as a
    different language than the rest, which produces text nobody can align and
    a transcript that claims two things at once.
    """

    if payload.recognition.language:
        return (payload.recognition.language, None)
    if not windows:
        return ("und", None)
    first = windows[0]
    window = samples[first.start_sample : first.end_sample]
    language, probability = recognizer.detect_language(window)
    recognizer.use_language(language)
    return (language, round(probability, 4))


def _document(
    *,
    payload: daemon_pb2.SpeechStagePayloadV1,
    audio: PcmAudio,
    vad_artifact_id: str,
    model: VerifiedModel,
    language: str,
    language_confidence: float | None,
    segments: list[AsrSegment],
    undecodable: list[DecodeWindow],
) -> SpeechAsr:
    rate = audio.sample_rate
    return SpeechAsr(
        schema_version="clipmill.speech.asr.v1",
        source_fingerprint=payload.source_fingerprint or audio.source_fingerprint,
        audio_artifact_id=payload.audio_artifact_id,
        vad_artifact_id=vad_artifact_id,
        producer=Producer(
            stage="speech-asr",
            implementation=f"clipmill-worker-asr@{__version__}+{IMPLEMENTATION}/{model.name}",
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
        timing_authority="forced_alignment",
        coverage=Coverage(
            start_ticks=0,
            end_ticks=audio.duration_ticks,
            analyzed=True,
            # Speech actually handed to the recognizer and turned into
            # text. Less than the speech total whenever a window failed, which
            # is the difference a consumer needs in order to distinguish a
            # quiet recording from a partly failed pass.
            decoded_ticks=sum(
                segment.hint_end_ticks - segment.hint_start_ticks for segment in segments
            ),
            sampling_plan="asr-vad-batched",
        ),
        segments=segments,
        invalid_regions=[
            InvalidRegion(
                start_ticks=samples_to_ticks(window.start_sample, rate),
                end_ticks=samples_to_ticks_ceil(window.end_sample, rate),
                reason="decode_failed",
                detail="the recognizer returned no text for this speech segment",
            )
            for window in undecodable
        ],
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="ClipMill whisper.cpp recognition worker")
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
            family="speech-asr",
            capabilities=CAPABILITIES,
            backend="cpu",
            cpu_threads=1,
            # The base model's weights plus whisper.cpp's compute buffers.
            max_memory_bytes=768 * 1024 * 1024,
        )
    )
    if arguments.once:
        client.run_one(execute_asr)
    else:
        client.run(execute_asr, stop)
    return 0


__all__ = ["CAPABILITIES", "__version__", "execute_asr", "main", "weights_file"]
