"""Voice activity detection: the first link in the speech chain.

Nothing transcribes before this runs. Recognition is the expensive stage, and
handing it the silence between utterances costs real time and invites the
hallucinated text that recognizers produce when asked what was said in a room
where nobody spoke. The silence edges this publishes are also where the
boundary optimizer is later allowed to cut, so being wrong here is not only a
throughput problem.
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
from clipmill_worker_sdk.audio import (
    AUDIO_DESCRIPTOR,
    AUDIO_PAYLOAD,
    PcmAudio,
    read_pcm_audio,
)
from clipmill_worker_sdk.documents import canonical_bytes
from clipmill_worker_sdk.gen.schemas.speech_vad import (
    Confidence,
    Coverage,
    Detection,
    Interval,
    Producer,
    SpeechSegment,
    SpeechVad,
)
from clipmill_worker_sdk.ticks import samples_to_ticks, samples_to_ticks_ceil, ticks_to_samples
from clipmill_worker_sdk.weights import ModelVerificationError, VerifiedModel, require_model

from .segmentation import SegmentationParameters, SpeechSpan, segment, silences
from .silero import IMPLEMENTATION, SileroVoiceActivity, decode_pcm16

__version__ = "0.1.0"
CAPABILITIES = ("speech-vad",)
OUTPUT_FILE = "vad.json"
KEY_VERSION = "clipmill.speech-stage.v1"
SAMPLE_RATE = 16_000

# Defaults for a payload that leaves a parameter at zero. Chosen for
# conversational speech: a third of a second of quiet ends an utterance, a
# tenth of a second of noise does not start one, and a segment is padded by
# 30 ms so a decoder is never handed a word already in progress.
DEFAULT_THRESHOLD = 0.5
DEFAULT_MIN_SPEECH_TICKS = 9_000
DEFAULT_MIN_SILENCE_TICKS = 27_000
DEFAULT_SPEECH_PAD_TICKS = 2_700

LOGGER = logging.getLogger(__name__)


def execute_vad(context: TaskContext) -> tuple[str, ...]:
    context.cancellation.raise_if_cancelled()
    payload = _payload(context)
    try:
        model = require_model(context.lease, "vad")
    except ModelVerificationError as error:
        # Weights that are missing or no longer what was pinned will not become
        # correct on a retry, and running anyway would publish a transcript
        # under an address claiming the pinned model produced it.
        raise DeterministicTaskError(str(error)) from error

    artifact = context.open_artifact(payload.audio_artifact_id)
    audio = read_pcm_audio(
        artifact,
        context.artifact_file(artifact, AUDIO_PAYLOAD),
        context.artifact_file(artifact, AUDIO_DESCRIPTOR),
        expect_sample_rate=SAMPLE_RATE,
        expect_channels=1,
    )

    detector = SileroVoiceActivity(model, audio.sample_rate)
    parameters = _parameters(payload.detection, audio.sample_rate, detector.window_samples)
    samples = decode_pcm16(audio.frames)

    def progress(done: int, total: int) -> None:
        context.cancellation.raise_if_cancelled()
        context.report_progress("audio_windows", done, total)

    scores = detector.probabilities(samples, on_window=progress)
    spans = segment(scores, parameters, audio.sample_count)

    document = _document(payload, audio, model, parameters, spans)
    context.staging.write_bytes(OUTPUT_FILE, canonical_bytes(document))
    return (OUTPUT_FILE,)


def _payload(context: TaskContext) -> daemon_pb2.SpeechStagePayloadV1:
    payload = daemon_pb2.SpeechStagePayloadV1()
    try:
        payload.ParseFromString(context.lease.payload)
    except Exception as error:
        # Every way this can fail is the same fact: the daemon and this worker
        # disagree about what a speech task looks like, and a retry will not
        # change that.
        raise DeterministicTaskError("task payload is not a speech stage payload") from error
    if payload.key_version != KEY_VERSION or payload.stage != "speech-vad":
        raise DeterministicTaskError("task payload does not describe voice activity detection")
    if not payload.audio_artifact_id:
        raise DeterministicTaskError("task payload names no audio rendition")
    return payload


def _parameters(
    detection: daemon_pb2.SpeechDetectionV1,
    sample_rate: int,
    window_samples: int,
) -> SegmentationParameters:
    threshold = detection.threshold or DEFAULT_THRESHOLD
    if not 0 < threshold < 1:
        raise DeterministicTaskError("speech threshold must lie strictly between zero and one")
    return SegmentationParameters(
        threshold=threshold,
        window_samples=window_samples,
        min_speech_samples=ticks_to_samples(
            detection.min_speech_ticks or DEFAULT_MIN_SPEECH_TICKS, sample_rate
        ),
        min_silence_samples=ticks_to_samples(
            detection.min_silence_ticks or DEFAULT_MIN_SILENCE_TICKS, sample_rate
        ),
        speech_pad_samples=ticks_to_samples(
            detection.speech_pad_ticks or DEFAULT_SPEECH_PAD_TICKS, sample_rate
        ),
    )


def _document(
    payload: daemon_pb2.SpeechStagePayloadV1,
    audio: PcmAudio,
    model: VerifiedModel,
    parameters: SegmentationParameters,
    spans: list[SpeechSpan],
) -> SpeechVad:
    rate = audio.sample_rate
    segments = [
        SpeechSegment(
            start_ticks=samples_to_ticks(span.start_sample, rate),
            end_ticks=samples_to_ticks_ceil(span.end_sample, rate),
            confidence=Confidence(p50=round(span.p50, 4), p10=round(span.p10, 4)),
        )
        for span in spans
    ]
    gaps = [
        Interval(
            start_ticks=samples_to_ticks(start, rate),
            end_ticks=samples_to_ticks_ceil(end, rate),
        )
        for start, end in silences(spans, audio.sample_count)
    ]
    return SpeechVad(
        schema_version="clipmill.speech.vad.v1",
        source_fingerprint=payload.source_fingerprint or audio.source_fingerprint,
        audio_artifact_id=payload.audio_artifact_id,
        producer=Producer(
            stage="speech-vad",
            implementation=f"clipmill-worker-vad@{__version__}+{IMPLEMENTATION}",
            model_digest=model.digest,
        ),
        detection=Detection(
            threshold=parameters.threshold,
            min_speech_ticks=samples_to_ticks(parameters.min_speech_samples, rate),
            min_silence_ticks=samples_to_ticks(parameters.min_silence_samples, rate),
            speech_pad_ticks=samples_to_ticks(parameters.speech_pad_samples, rate),
            window_samples=parameters.window_samples,
        ),
        # This pass read the whole rendition. Saying so is what lets a later
        # stage tell "nobody spoke" from "nobody listened".
        coverage=Coverage(
            start_ticks=0,
            end_ticks=audio.duration_ticks,
            analyzed=True,
            sampling_plan=f"silero-{parameters.window_samples}-at-{rate}",
        ),
        speech_ticks=sum(entry.end_ticks - entry.start_ticks for entry in segments),
        segments=segments,
        silences=gaps,
        invalid_regions=[],
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="ClipMill voice activity worker")
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
            family="speech-vad",
            capabilities=CAPABILITIES,
            backend="onnx-cpu",
            # Declared honestly: the session is single-threaded, and the model
            # plus onnxruntime's arena fit in well under this.
            cpu_threads=1,
            max_memory_bytes=192 * 1024 * 1024,
        )
    )
    if arguments.once:
        client.run_one(execute_vad)
    else:
        client.run(execute_vad, stop)
    return 0


__all__ = ["CAPABILITIES", "__version__", "execute_vad", "main"]
