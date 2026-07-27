#!/usr/bin/env python3
"""Measure what each speech implementation costs on this machine (D19).

Backend selection comes from measurement, not from a per-platform default
somebody wrote down. The measurement has to happen somewhere, and the only
place a model's real cost can be observed is inside the environment that loads
it — so it happens here, in a process that imports the same worker code the
daemon leases, and the result is written where the device profiler will read
it.

What is written is deliberately small and path-free: an implementation name, a
model digest, a real-time factor, and a peak resident size. No directories, no
machine names, nothing that would make one machine's measurement look valid on
another. What binds it to this machine is the hardware fingerprint, which the
daemon computes and this tool copies; what binds it to these weights is the
model digest, which is why re-pinning a model retires its own measurement
rather than quietly outliving it.

The daemon believes this file exactly as much as it believes its own
attestation key, and for the same reason: both live in a private state
directory that only the user the daemon runs as can write.

    tools/bench/speech-benchmark.py --fixture <dir> --models <dir> \\
        --registry models/registry --fingerprint sha256:... --output <file>
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import resource
import subprocess
import sys
import time
import tomllib
import wave
from dataclasses import asdict, dataclass
from pathlib import Path

import numpy as np
from clipmill.worker.v1 import worker_pb2
from clipmill_worker_sdk.batching import decode_windows
from clipmill_worker_sdk.weights import ModelVerificationError, VerifiedModel, verify_model

SCHEMA = "clipmill.speech_benchmark.v1"
SAMPLE_RATE = 16_000
FINGERPRINT_PATTERN = 64


@dataclass(frozen=True, slots=True)
class Candidate:
    """One implementation, named exactly as the daemon's registry names it."""

    implementation: str
    capability: str
    model: str


#: Kept in step with `crates/clipmilld/src/implementations.rs` by the drill,
#: which fails when the two lists disagree — a benchmark measuring something
#: the daemon cannot plan is a measurement nobody will ever read.
CANDIDATES = (
    Candidate("clipmill-worker-vad@0.1.0", "vad", "silero-vad"),
    Candidate("clipmill-worker-asr@0.1.0", "asr", "whisper-base"),
    Candidate("clipmill-worker-speech-mlx@0.1.0/asr", "asr", "qwen3-asr-mlx"),
    Candidate("clipmill-worker-align@0.1.0", "forced-align", "wav2vec2-ctc-en"),
    Candidate("clipmill-worker-speech-mlx@0.1.0/align", "forced-align", "qwen3-aligner-mlx"),
)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture", type=Path, required=True)
    parser.add_argument("--models", type=Path, default=Path(".cache/models"))
    parser.add_argument("--registry", type=Path, default=Path("models/registry"))
    parser.add_argument(
        "--fingerprint",
        required=True,
        help="the daemon's hardware fingerprint; the measurement is worthless without it",
    )
    parser.add_argument("--output", type=Path)
    parser.add_argument(
        "--only",
        help=(
            "measure one implementation and print it as JSON. This is how the "
            "full run works internally: peak resident size is a process "
            "high-water mark that never falls, so measuring several models in "
            "one process would report the largest one's footprint for all of "
            "them. A fresh process per candidate is the only way the number "
            "means what it says."
        ),
    )
    options = parser.parse_args()

    if not _is_fingerprint(options.fingerprint):
        print("speech-benchmark: --fingerprint must be sha256:<64 hex>", file=sys.stderr)
        return 2

    audio, seconds = _load_fixture(options.fixture)
    if options.only is not None:
        chosen = next((c for c in CANDIDATES if c.implementation == options.only), None)
        if chosen is None:
            print(f"speech-benchmark: no candidate named {options.only}", file=sys.stderr)
            return 2
        print(json.dumps(asdict(_measure(chosen, audio, seconds, options))))
        return 0
    if options.output is None:
        parser.error("--output is required unless --only is given")

    measurements = [_measure_in_child(candidate, options) for candidate in CANDIDATES]
    document = {
        "schema_version": SCHEMA,
        "hardware_fingerprint": options.fingerprint,
        "measurements": [
            {key: value for key, value in asdict(measurement).items() if value is not None}
            for measurement in measurements
        ],
    }
    _write(options.output, document)

    for measurement in measurements:
        if measurement.runnable:
            print(
                f"speech-benchmark: {measurement.implementation} "
                f"{measurement.real_time_factor:.2f}x real time, "
                f"{measurement.peak_resident_bytes / (1024 * 1024):.0f} MiB peak"
            )
        else:
            print(
                f"speech-benchmark: {measurement.implementation} unavailable "
                f"({measurement.unavailable_reason})"
            )
    runnable = sum(1 for measurement in measurements if measurement.runnable)
    print(
        f"speech-benchmark: OK ({runnable} of {len(measurements)} implementations ran "
        f"over {seconds:.1f}s of speech; wrote {options.output.name})"
    )
    return 0


@dataclass(frozen=True, slots=True)
class Measurement:
    implementation: str
    capability: str
    model: str
    model_digest: str
    runnable: bool
    real_time_factor: float | None = None
    peak_resident_bytes: int | None = None
    unavailable_reason: str | None = None


def _measure_in_child(candidate: Candidate, options) -> Measurement:
    """One candidate, in a process of its own.

    A crash is an answer here rather than an interruption: an implementation
    that takes the interpreter down with it is one this device cannot run, and
    saying so is more useful than losing the other four measurements.
    """

    result = subprocess.run(
        [
            sys.executable,
            str(Path(__file__).resolve()),
            "--fixture",
            str(options.fixture),
            "--models",
            str(options.models),
            "--registry",
            str(options.registry),
            "--fingerprint",
            options.fingerprint,
            "--only",
            candidate.implementation,
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode == 0:
        try:
            return Measurement(**json.loads(result.stdout.strip().splitlines()[-1]))
        except (IndexError, TypeError, ValueError):
            pass
    detail = (result.stderr.strip().splitlines() or ["no output"])[-1]
    return Measurement(
        implementation=candidate.implementation,
        capability=candidate.capability,
        model=candidate.model,
        model_digest=_UNPINNED,
        runnable=False,
        unavailable_reason=f"the measurement process failed: {detail}"[:512],
    )


def _measure(candidate: Candidate, audio: np.ndarray, seconds: float, options) -> Measurement:
    try:
        model = _verified(candidate, options)
    except (ModelVerificationError, FileNotFoundError, KeyError) as error:
        return Measurement(
            implementation=candidate.implementation,
            capability=candidate.capability,
            model=candidate.model,
            model_digest=_UNPINNED,
            runnable=False,
            unavailable_reason=f"weights are unavailable: {error}",
        )
    started = time.perf_counter()
    try:
        _run(candidate, model, audio)
    except Exception as error:
        return Measurement(
            implementation=candidate.implementation,
            capability=candidate.capability,
            model=candidate.model,
            model_digest=model.digest,
            runnable=False,
            unavailable_reason=f"{type(error).__name__}: {error}"[:512],
        )
    elapsed = max(time.perf_counter() - started, 1e-6)
    return Measurement(
        implementation=candidate.implementation,
        capability=candidate.capability,
        model=candidate.model,
        model_digest=model.digest,
        runnable=True,
        real_time_factor=round(seconds / elapsed, 4),
        # This process's high-water mark, and this process measured exactly
        # one implementation. What decides whether a machine can run two
        # stages at once is what the operating system had to find room for,
        # not what a runtime says it allocated.
        peak_resident_bytes=_peak_resident_bytes(),
    )


def _run(candidate: Candidate, model: VerifiedModel, audio: np.ndarray) -> None:
    """One full pass of this implementation over the fixture."""

    if candidate.capability == "vad":
        from clipmill_worker_vad.silero import SileroVoiceActivity

        SileroVoiceActivity(model, SAMPLE_RATE).probabilities(audio)
        return

    windows = _speech_windows(audio)
    if candidate.implementation.startswith("clipmill-worker-speech-mlx"):
        _run_mlx(candidate, model, audio, windows)
        return
    if candidate.capability == "asr":
        from clipmill_worker_asr_whispercpp import weights_file
        from clipmill_worker_asr_whispercpp.engine import WhisperCppRecognizer

        recognizer = WhisperCppRecognizer(model, weights=weights_file(model), language="en")
        for start, end in windows:
            recognizer.decode(audio[start:end])
        return

    from clipmill_worker_align.ctc import forced_align
    from clipmill_worker_align.vocabulary import Vocabulary
    from clipmill_worker_align.wav2vec2 import Wav2Vec2Ctc

    acoustic = Wav2Vec2Ctc(model)
    vocabulary = Vocabulary.load(acoustic.vocabulary_path)
    scoreable, _ = vocabulary.encode(_BENCHMARK_TEXT.split())
    labels, _ = vocabulary.label_sequence(scoreable)
    for start, end in windows:
        emissions = acoustic.emissions(audio[start:end])
        forced_align(emissions, labels, blank_id=vocabulary.blank_id)


def _run_mlx(candidate: Candidate, model: VerifiedModel, audio: np.ndarray, windows) -> None:
    from clipmill_worker_speech_mlx.aligner import Qwen3Aligner
    from clipmill_worker_speech_mlx.recognizer import Qwen3Recognizer

    if candidate.capability == "asr":
        recognizer = Qwen3Recognizer(model.root, SAMPLE_RATE)
        recognizer.use_language("en")
        for start, end in windows:
            recognizer.decode(audio[start:end])
        return
    aligner = Qwen3Aligner(model.root, SAMPLE_RATE)
    for start, end in windows:
        aligner.align(audio[start:end], _BENCHMARK_TEXT, "en")


#: What the aligners are asked to place. The fixture speaks it, so the work is
#: the same shape the real stage does — the point is the cost, and an aligner
#: handed text unrelated to the audio would still do that work, just less
#: honestly.
_BENCHMARK_TEXT = "the first slice renders from the edit document"
_UNPINNED = "sha256:" + "0" * 64


def _speech_windows(audio: np.ndarray) -> list[tuple[int, int]]:
    """Fixed windows over the fixture, so every implementation decodes the same
    audio in the same pieces. Running voice activity first would make the
    recognizers' measurements depend on a third model's output."""

    windows = decode_windows([(0, int(audio.size))], SAMPLE_RATE, 28)
    return [(window.start_sample, window.end_sample) for window in windows]


def _verified(candidate: Candidate, options) -> VerifiedModel:
    manifest = tomllib.loads(
        (options.registry / f"{candidate.model}.toml").read_text(encoding="utf-8")
    )
    binding = worker_pb2.ModelBinding(
        name=candidate.model,
        root=str((options.models / candidate.model).resolve()),
        # The daemon's identity for these weights, recomputed exactly as
        # `ModelManifest::digest` does it. This is the field that makes a
        # measurement stale when a model is re-pinned.
        digest=_model_digest(candidate.model, manifest),
        capability=manifest["capability"],
        files=[
            worker_pb2.ModelFile(path=f["path"], sha256=f["sha256"], bytes=f["bytes"])
            for f in manifest["files"]
        ],
    )
    return verify_model(binding)


def _model_digest(name: str, manifest: dict) -> str:
    import hashlib

    digest = hashlib.sha256()
    digest.update(b"clipmill.model.identity.v1\0")
    for field in (
        name,
        manifest["source"]["repo"],
        manifest["source"]["revision"],
        manifest["quantization"],
    ):
        digest.update(field.encode("utf-8"))
        digest.update(b"\0")
    for entry in sorted(manifest["files"], key=lambda file: (file["path"], file["sha256"])):
        digest.update(entry["path"].encode("utf-8"))
        digest.update(b"\0")
        digest.update(entry["sha256"].encode("utf-8"))
        digest.update(b"\0")
    return f"sha256:{digest.hexdigest()}"


def _load_fixture(fixture: Path) -> tuple[np.ndarray, float]:
    with wave.open(str(fixture / "speech.wav"), "rb") as handle:
        if handle.getframerate() != SAMPLE_RATE or handle.getnchannels() != 1:
            raise SystemExit("speech-benchmark: the fixture must be 16 kHz mono")
        total = handle.getnframes()
        frames = handle.readframes(total)
    audio = np.frombuffer(frames, dtype="<i2").astype(np.float32) / 32768.0
    return (audio, total / SAMPLE_RATE)


def _peak_resident_bytes() -> int:
    peak = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
    # Linux reports kibibytes; macOS reports bytes. Getting this backwards
    # would make one platform's models look a thousand times cheaper.
    return int(peak if platform.system() == "Darwin" else peak * 1024)


def _is_fingerprint(value: str) -> bool:
    prefix, _, hexadecimal = value.partition(":")
    return (
        prefix == "sha256"
        and len(hexadecimal) == FINGERPRINT_PATTERN
        and all(character in "0123456789abcdef" for character in hexadecimal)
    )


def _write(path: Path, document: dict) -> None:
    """Written atomically into a private directory, like everything else the
    daemon reads out of its own state."""

    path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    temporary = path.with_name(path.name + ".partial")
    payload = json.dumps(document, indent=2, sort_keys=True).encode("utf-8") + b"\n"
    descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    try:
        os.write(descriptor, payload)
        os.fsync(descriptor)
    finally:
        os.close(descriptor)
    os.replace(temporary, path)
    directory = os.open(path.parent, os.O_RDONLY)
    try:
        os.fsync(directory)
    finally:
        os.close(directory)


if __name__ == "__main__":
    raise SystemExit(main())
