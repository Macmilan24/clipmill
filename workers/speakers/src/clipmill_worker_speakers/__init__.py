"""Who speaks when: the recording's speech told apart by voice.

Reads the 16 kHz rendition and the voice activity found in it, prints each
stretch of speech with the pinned voice-print model, groups the prints into
voices and lays them back onto the speech as turns (`voices.py`). Silence has
no speaker: only what voice activity called speech is printed.
"""

from __future__ import annotations

import argparse
import json
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
from clipmill_worker_sdk.audio import AUDIO_DESCRIPTOR, AUDIO_PAYLOAD, read_pcm_audio
from clipmill_worker_sdk.documents import canonical_bytes
from clipmill_worker_sdk.gen.schemas.speech_speakers import (
    Clustering as ClusteringDocument,
)
from clipmill_worker_sdk.gen.schemas.speech_speakers import (
    Coverage,
    Producer,
    Speaker,
    SpeechSpeakers,
    Turn,
)
from clipmill_worker_sdk.inputs import MissingInputError, require_input
from clipmill_worker_sdk.ticks import samples_to_ticks, ticks_to_samples
from clipmill_worker_sdk.weights import ModelVerificationError, require_model

from .model import IMPLEMENTATION, VoicePrints
from .voices import Clustering, prints, turns_of, voices_of, windows_of

__version__ = "0.1.0"
CAPABILITIES = ("speech-speakers",)
OUTPUT_FILE = "speakers.json"
KEY_VERSION = "clipmill.speech-stage.v1"
SAMPLE_RATE = 16_000
AUDIO_KIND = "media.audio_16k.v1"
VAD_KIND = "speech.vad.v1"
VAD_FILE = "vad.json"

LOGGER = logging.getLogger(__name__)


def execute_speakers(context: TaskContext) -> tuple[str, ...]:
    context.cancellation.raise_if_cancelled()
    payload = _payload(context)
    try:
        model = require_model(context.lease, "speaker-embed")
    except ModelVerificationError as error:
        raise DeterministicTaskError(str(error)) from error

    audio_input = _input(context, AUDIO_KIND)
    audio = read_pcm_audio(
        audio_input.artifact,
        context.artifact_file(audio_input.artifact, AUDIO_PAYLOAD),
        context.artifact_file(audio_input.artifact, AUDIO_DESCRIPTOR),
        expect_sample_rate=SAMPLE_RATE,
        expect_channels=1,
    )
    vad_input = _input(context, VAD_KIND)
    vad = json.loads(
        context.artifact_file(vad_input.artifact, VAD_FILE).read_text(encoding="utf-8")
    )
    # The voice activity must be of this audio, or every turn is somewhere else.
    if vad.get("audio_artifact_id") != audio_input.artifact_id:
        raise DeterministicTaskError("the voice activity was found in other audio")

    samples = np.frombuffer(audio.frames, dtype="<i2")
    stretches = [
        (
            min(audio.sample_count, ticks_to_samples(int(segment["start_ticks"]), SAMPLE_RATE)),
            min(audio.sample_count, ticks_to_samples(int(segment["end_ticks"]), SAMPLE_RATE)),
        )
        for segment in vad.get("segments", [])
    ]
    clustering = Clustering()
    windows = windows_of(stretches, clustering)

    def progress(done: int, total: int) -> None:
        context.cancellation.raise_if_cancelled()
        context.report_progress("speech_stretches", done, total)

    try:
        embed = VoicePrints(model)
    except ValueError as error:
        raise DeterministicTaskError(str(error)) from error
    points = prints(embed, samples, stretches, windows, on_stretch=progress)
    voices = voices_of(points, clustering)
    turns = turns_of(stretches, windows, [int(voice) for voice in voices], clustering)

    document = _document(
        payload,
        audio_input.artifact_id,
        vad_input.artifact_id,
        audio.source_fingerprint,
        audio.duration_ticks,
        model.digest,
        clustering,
        turns,
    )
    context.staging.write_bytes(OUTPUT_FILE, canonical_bytes(document))
    return (OUTPUT_FILE,)


def _payload(context: TaskContext) -> daemon_pb2.SpeechStagePayloadV1:
    payload = daemon_pb2.SpeechStagePayloadV1()
    try:
        payload.ParseFromString(context.lease.payload)
    except Exception as error:
        raise DeterministicTaskError("task payload is not a speech stage payload") from error
    if payload.key_version != KEY_VERSION or payload.stage != "speech-speakers":
        raise DeterministicTaskError("task payload does not describe telling voices apart")
    return payload


def _input(context: TaskContext, kind: str):
    try:
        return require_input(context, kind)
    except MissingInputError as error:
        raise DeterministicTaskError(str(error)) from error


def _document(
    payload: daemon_pb2.SpeechStagePayloadV1,
    audio_artifact_id: str,
    vad_artifact_id: str,
    source_fingerprint: str,
    duration_ticks: int,
    model_digest: str,
    clustering: Clustering,
    turns: list,
) -> SpeechSpeakers:
    # Numbered by the turns as published, in the order they are first heard:
    # smoothing may have taken the one window a voice was first heard in.
    names: dict[int, str] = {}
    published: list[Turn] = []
    heard: dict[str, int] = {}
    first: dict[str, int] = {}
    for turn in turns:
        start = samples_to_ticks(turn.start, SAMPLE_RATE)
        end = samples_to_ticks(turn.end, SAMPLE_RATE)
        if end <= start:
            continue
        name = names.setdefault(turn.voice, f"spk_{len(names) + 1}")
        published.append(Turn(start_ticks=start, end_ticks=end, speaker_id=name))
        heard[name] = heard.get(name, 0) + end - start
        first.setdefault(name, start)
    return SpeechSpeakers(
        schema_version="clipmill.speech.speakers.v1",
        source_fingerprint=payload.source_fingerprint or source_fingerprint,
        audio_artifact_id=audio_artifact_id,
        vad_artifact_id=vad_artifact_id,
        producer=Producer(
            stage="speech-speakers",
            implementation=f"clipmill-worker-speakers@{__version__}+{IMPLEMENTATION}",
            model_digest=model_digest,
        ),
        clustering=ClusteringDocument(
            window_ticks=samples_to_ticks(clustering.window_samples, SAMPLE_RATE),
            hop_ticks=samples_to_ticks(clustering.hop_samples, SAMPLE_RATE),
            link=clustering.link,
            merge=clustering.merge,
            smallest=clustering.smallest,
        ),
        coverage=Coverage(start_ticks=0, end_ticks=duration_ticks, analyzed=True),
        speakers=[
            Speaker(speaker_id=name, speech_ticks=heard[name], first_ticks=first[name])
            for name in names.values()
            if name in heard
        ],
        turns=published,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="ClipMill speakers worker")
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
            family="speech-speakers",
            capabilities=CAPABILITIES,
            backend="onnx-cpu",
            # Two session threads, fixed so a recording prints the same twice;
            # the whole rendition is held, as voice activity holds it.
            cpu_threads=2,
            max_memory_bytes=1536 * 1024 * 1024,
        )
    )
    if arguments.once:
        client.run_one(execute_speakers)
    else:
        client.run(execute_speakers, stop)
    return 0


__all__ = ["CAPABILITIES", "__version__", "execute_speakers", "main"]
