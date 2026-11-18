"""Parakeet speech recognition and acoustic word timing over leased artifacts."""

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
    LeaseInputs,
    TaskContext,
    WorkerClient,
    WorkerConfiguration,
    WorkerIdentity,
)
from clipmill_worker_sdk.audio import AUDIO_DESCRIPTOR, AUDIO_PAYLOAD, read_pcm_audio
from clipmill_worker_sdk.batching import decode_windows
from clipmill_worker_sdk.confidence import distribution
from clipmill_worker_sdk.documents import canonical_bytes
from clipmill_worker_sdk.gen.schemas.speech_alignment import SpeechAlignment
from clipmill_worker_sdk.gen.schemas.speech_asr import AsrSegment, SpeechAsr
from clipmill_worker_sdk.gen.schemas.speech_vad import SpeechVad
from clipmill_worker_sdk.ticks import samples_to_ticks, samples_to_ticks_ceil, ticks_to_samples
from clipmill_worker_sdk.weights import ModelVerificationError, require_model

from .recognizer import FRAME_TICKS, SAMPLE_RATE, ParakeetRecognizer, words_from_emissions

__version__ = "0.1.0"
CAPABILITIES = ("speech-asr", "speech-tdt-timing")
IMPLEMENTATION = "clipmill-worker-parakeet@0.1.0"
TIMING_IMPLEMENTATION = "clipmill-worker-parakeet@0.1.0/timing"
LANGUAGES = frozenset(
    [
        "bg",
        "cs",
        "da",
        "de",
        "el",
        "en",
        "es",
        "et",
        "fi",
        "fr",
        "hr",
        "hu",
        "it",
        "lt",
        "lv",
        "mt",
        "nl",
        "pl",
        "pt",
        "ro",
        "ru",
        "sk",
        "sl",
        "sv",
        "uk",
    ]
)


def recognition(context: TaskContext, payload, inputs: LeaseInputs) -> SpeechAsr:
    language = payload.recognition.language.lower().split("-", 1)[0]
    if language and language != "und" and language not in LANGUAGES:
        raise DeterministicTaskError(
            f"Parakeet does not cover {language}. Select Whisper for this recording."
        )
    source = inputs.require("media.audio_16k.v1")
    audio = read_pcm_audio(
        source.artifact,
        context.artifact_file(source.artifact, AUDIO_PAYLOAD),
        context.artifact_file(source.artifact, AUDIO_DESCRIPTOR),
        expect_sample_rate=SAMPLE_RATE,
        expect_channels=1,
    )
    vad = inputs.require("speech.vad.v1")
    activity = SpeechVad.model_validate_json(
        context.artifact_file(vad.artifact, "vad.json").read_bytes()
    )
    if not activity.coverage.analyzed or activity.audio_artifact_id.root != source.artifact_id:
        raise DeterministicTaskError("voice activity does not describe this audio")
    fingerprint = payload.source_fingerprint or audio.source_fingerprint
    if fingerprint != audio.source_fingerprint or activity.source_fingerprint.root != fingerprint:
        raise DeterministicTaskError("speech inputs describe different recordings")
    try:
        model = require_model(context.lease, "asr")
    except ModelVerificationError as error:
        raise DeterministicTaskError(str(error)) from error
    windows = decode_windows(
        [
            (
                ticks_to_samples(s.start_ticks, SAMPLE_RATE),
                min(ticks_to_samples(s.end_ticks, SAMPLE_RATE), audio.sample_count),
            )
            for s in activity.segments
        ],
        SAMPLE_RATE,
        max_window_seconds=30,
    )
    # Decode only observed speech. Convert one bounded window at a time so float
    # buffers do not grow with the length of the recording.
    samples = np.frombuffer(audio.frames, dtype="<i2")
    engine = ParakeetRecognizer(model) if windows else None
    segments, invalid = [], []
    for i, window in enumerate(windows):
        context.cancellation.raise_if_cancelled()
        context.report_progress("speech_segments", i, len(windows))
        start = samples_to_ticks(window.start_sample, SAMPLE_RATE)
        end = samples_to_ticks_ceil(window.end_sample, SAMPLE_RATE)
        waveform = samples[window.start_sample : window.end_sample].astype(np.float32) / 32768
        emissions = engine.decode(waveform, check_cancelled=context.cancellation.raise_if_cancelled)
        words, tokens = words_from_emissions(emissions, start, end)
        if not words:
            invalid.append({"start_ticks": start, "end_ticks": end, "reason": "decode_failed"})
            continue
        p50, p10 = distribution([t.confidence for t in tokens])
        segments.append(
            AsrSegment(
                index=len(segments),
                vad_segment_index=window.vad_segment_index,
                hint_start_ticks=start,
                hint_end_ticks=end,
                text=" ".join(w.text for w in words),
                confidence={"p50": round(p50, 4), "p10": round(p10, 4)},
                tokens=tokens,
                transducer_words=words,
            )
        )
    context.report_progress("speech_segments", len(windows), len(windows))
    return SpeechAsr(
        schema_version="clipmill.speech.asr.v1",
        source_fingerprint=fingerprint,
        audio_artifact_id=source.artifact_id,
        vad_artifact_id=vad.artifact_id,
        producer={
            "stage": "speech-asr",
            "implementation": IMPLEMENTATION,
            "model_digest": model.digest,
        },
        # TDT performs multilingual recognition without a language-ID posterior.
        language=language or "und",
        decoding={
            "strategy": "greedy",
            "temperature": 0.0,
            "beam_size": 0,
            "conditioned_on_previous": False,
        },
        timing_authority="transducer",
        coverage={
            "start_ticks": 0,
            "end_ticks": audio.duration_ticks,
            "analyzed": True,
            "decoded_ticks": sum(s.hint_end_ticks - s.hint_start_ticks for s in segments),
            "sampling_plan": "asr-vad-batched",
        },
        segments=segments,
        invalid_regions=invalid,
    )


def timing(recognized: SpeechAsr, artifact_id: str) -> SpeechAlignment:
    """Publish the duration head's intervals, preserving their ASR and recording identity."""
    if (
        recognized.timing_authority != "transducer"
        or recognized.producer.implementation != IMPLEMENTATION
    ):
        raise DeterministicTaskError("native word timing requires a Parakeet recognition artifact")
    words = []
    aligned_ticks = 0
    for segment in recognized.segments:
        measured = segment.transducer_words or []
        if not measured or [w.text for w in measured] != segment.text.split():
            raise DeterministicTaskError("native timing does not cover the recognized words")
        previous = segment.hint_start_ticks
        for word in measured:
            if not previous <= word.start_ticks <= word.end_ticks <= segment.hint_end_ticks:
                raise DeterministicTaskError(
                    "native word interval is unordered or outside its utterance"
                )
            previous = word.end_ticks
            words.append({"index": len(words), "segment_index": segment.index, **word.model_dump()})
        aligned_ticks += measured[-1].end_ticks - measured[0].start_ticks
    return SpeechAlignment(
        schema_version="clipmill.speech.alignment.v1",
        source_fingerprint=recognized.source_fingerprint.root,
        audio_artifact_id=recognized.audio_artifact_id.root,
        asr_artifact_id=artifact_id,
        producer={"stage": "speech-tdt-timing", "implementation": TIMING_IMPLEMENTATION},
        frame_ticks=FRAME_TICKS,
        coverage={
            "start_ticks": recognized.coverage.start_ticks,
            "end_ticks": recognized.coverage.end_ticks,
            "analyzed": recognized.coverage.analyzed,
            "aligned_ticks": aligned_ticks,
            "sampling_plan": "tdt-acoustic-durations",
        },
        words=words,
        unaligned=[],
        invalid_regions=[],
    )


def execute(context: TaskContext) -> tuple[str, ...]:
    context.cancellation.raise_if_cancelled()
    payload = daemon_pb2.SpeechStagePayloadV1.FromString(context.lease.payload)
    stage = context.lease.kind
    if payload.key_version != "clipmill.speech-stage.v1" or payload.stage != stage:
        raise DeterministicTaskError("invalid speech stage payload")
    inputs = LeaseInputs(context)
    if stage == "speech-asr":
        document = recognition(context, payload, inputs)
        path = "asr.json"
    elif stage == "speech-tdt-timing":
        source = inputs.require("speech.asr.v1")
        recognized = SpeechAsr.model_validate_json(
            context.artifact_file(source.artifact, "asr.json").read_bytes()
        )
        if recognized.source_fingerprint.root != payload.source_fingerprint:
            raise DeterministicTaskError("native timing was requested for a different recording")
        document = timing(recognized, source.artifact_id)
        path = "alignment.json"
    else:
        raise DeterministicTaskError(f"unknown Parakeet stage {stage}")
    context.cancellation.raise_if_cancelled()
    context.staging.write_bytes(path, canonical_bytes(document))
    return (path,)


def main() -> int:
    parser = argparse.ArgumentParser(description="ClipMill Parakeet speech worker")
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
            family="parakeet",
            capabilities=CAPABILITIES,
            backend="onnx-cpu",
            cpu_threads=1,
            max_memory_bytes=2 * 1024 * 1024 * 1024,
        )
    )
    if arguments.once:
        client.run_one(execute)
    else:
        client.run(execute, stop)
    return 0
