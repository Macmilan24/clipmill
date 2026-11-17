"""Audio events: what an audience or a room does that a transcript cannot show.

Laughter after a line, applause after a point, music under an intro, a shout or
a gasp: the editorial mind reads these from the Timeline Index to find payoffs
and reactions, and to keep a cut from landing in the middle of one.
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
from clipmill_worker_sdk.audio import AUDIO_DESCRIPTOR, AUDIO_PAYLOAD, PcmAudio, read_pcm_audio
from clipmill_worker_sdk.documents import canonical_bytes
from clipmill_worker_sdk.gen.schemas.audio_events import (
    AudioEvents,
    Coverage,
    Detection,
    Event,
    Producer,
    Threshold,
)
from clipmill_worker_sdk.inputs import MissingInputError, require_input
from clipmill_worker_sdk.ticks import samples_to_ticks, samples_to_ticks_ceil
from clipmill_worker_sdk.weights import ModelVerificationError, VerifiedModel, require_model

from . import events as detection
from .ced import (
    HOP_SAMPLES,
    IMPLEMENTATION,
    SAMPLE_RATE,
    WINDOW_SAMPLES,
    AudioTagger,
    decode_pcm16,
    labels,
)

__version__ = "0.1.0"
CAPABILITIES = ("audio-events",)
STAGE = "audio-events"
OUTPUT_FILE = "audio_events.json"
KEY_VERSION = "clipmill.audio-events-stage.v1"
AUDIO_KIND = "media.audio_16k.v1"

LOGGER = logging.getLogger(__name__)


def execute_audio_events(context: TaskContext) -> tuple[str, ...]:
    context.cancellation.raise_if_cancelled()
    payload = _payload(context)
    try:
        model = require_model(context.lease, "audio-events")
        columns = detection.kind_columns(labels(model))
    except (ModelVerificationError, ValueError, KeyError) as error:
        # Weights that are missing, altered, or name classes this worker does
        # not know will not become right on a retry.
        raise DeterministicTaskError(str(error)) from error

    try:
        audio_input = require_input(context, AUDIO_KIND)
    except MissingInputError as error:
        raise DeterministicTaskError(str(error)) from error
    artifact = audio_input.artifact
    audio = read_pcm_audio(
        artifact,
        context.artifact_file(artifact, AUDIO_PAYLOAD),
        context.artifact_file(artifact, AUDIO_DESCRIPTOR),
        expect_sample_rate=SAMPLE_RATE,
        expect_channels=1,
    )

    def progress(done: int, total: int) -> None:
        context.cancellation.raise_if_cancelled()
        context.report_progress("audio_windows", done, total)

    scores = AudioTagger(model).scores(decode_pcm16(audio.frames), on_batch=progress)
    found = detection.detect(
        scores,
        columns,
        window_samples=WINDOW_SAMPLES,
        hop_samples=HOP_SAMPLES,
        total_samples=audio.sample_count,
    )
    document = build_document(payload, audio_input.artifact_id, audio, model, found)
    context.staging.write_bytes(OUTPUT_FILE, canonical_bytes(document))
    return (OUTPUT_FILE,)


def _payload(context: TaskContext) -> daemon_pb2.AudioEventsStagePayloadV1:
    payload = daemon_pb2.AudioEventsStagePayloadV1()
    try:
        payload.ParseFromString(context.lease.payload)
    except Exception as error:
        raise DeterministicTaskError("task payload is not an audio events payload") from error
    if payload.key_version != KEY_VERSION or payload.stage != STAGE:
        raise DeterministicTaskError("task payload does not describe audio event detection")
    return payload


def build_document(
    payload: daemon_pb2.AudioEventsStagePayloadV1,
    audio_artifact_id: str,
    audio: PcmAudio,
    model: VerifiedModel,
    found: list[detection.Event],
) -> AudioEvents:
    rate = audio.sample_rate
    return AudioEvents(
        schema_version="clipmill.audio.events.v1",
        source_fingerprint=payload.source_fingerprint or audio.source_fingerprint,
        audio_artifact_id=audio_artifact_id,
        producer=Producer(
            stage=STAGE,
            implementation=f"clipmill-worker-audio-events@{__version__}+{IMPLEMENTATION}",
            model_digest=model.digest,
        ),
        detection=Detection(
            window_ticks=samples_to_ticks(WINDOW_SAMPLES, rate),
            hop_ticks=samples_to_ticks(HOP_SAMPLES, rate),
            thresholds=[
                Threshold(kind=kind, on=on, off=off)
                for kind, (on, off) in detection.THRESHOLDS.items()
            ],
        ),
        # The whole rendition was listened to, so a stretch with no event in it
        # is a stretch where nothing was heard.
        coverage=Coverage(start_ticks=0, end_ticks=audio.duration_ticks, analyzed=True),
        events=[
            Event(
                kind=event.kind,
                start_ticks=samples_to_ticks(event.start_sample, rate),
                end_ticks=samples_to_ticks_ceil(event.end_sample, rate),
                peak=event.peak,
                mean=event.mean,
            )
            for event in found
        ],
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="ClipMill audio events worker")
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
            family="audio-events",
            capabilities=CAPABILITIES,
            backend="onnx-cpu",
            # One thread, and a batch of sixteen two-second windows through a
            # 10 MB model fits well under this.
            cpu_threads=1,
            max_memory_bytes=320 * 1024 * 1024,
        )
    )
    if arguments.once:
        client.run_one(execute_audio_events)
    else:
        client.run(execute_audio_events, stop)
    return 0


__all__ = ["CAPABILITIES", "__version__", "build_document", "execute_audio_events", "main"]
