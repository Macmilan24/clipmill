"""Shot detection: the boundaries an editor never has to justify.

A cut is the one place a clip can start or end without anybody defending the
choice, which is why the boundary lattice is later allowed to snap to these
positions and why being wrong here is not merely a cosmetic problem. The stage
runs no model — it is arithmetic over decoded pixels — so nothing here loads
weights, and the only versioned input besides the proxy is the decoder that
produced the frames.

That decoder is the reason this worker is unlike the speech family. It does not
resolve FFmpeg; the daemon names one on the lease, having fetched it against
the bill of materials, and the build identity travels separately into the
payload so it reaches the artifact key. A stage that found its own decoder
would publish observations that two machines could disagree about while sharing
one content address.
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
from clipmill_worker_sdk.documents import canonical_bytes
from clipmill_worker_sdk.gen.schemas.evidence_shots import (
    Confidence,
    Coverage,
    Cut,
    Detection,
    EvidenceShots,
    Producer,
    Shot,
    Timebase,
)
from clipmill_worker_sdk.inputs import MissingInputError, require_input
from clipmill_worker_sdk.ticks import frames_to_ticks
from clipmill_worker_sdk.tools import ToolUnavailableError, require_tool

from . import content
from .content import CALIBRATION, IMPLEMENTATION, Parameters
from .decode import AnalysisSize, DecodeFailed, analysis_size, decode_frames
from .proxy import PROXY_DESCRIPTOR, Proxy, read_proxy

__version__ = "0.1.0"
# The registered task kind, which is what the daemon routes by. A capability
# named after the work rather than the stage reads better and matches nothing:
# this worker would connect, be trusted, and never be offered a task.
CAPABILITIES = ("detect-shots",)
OUTPUT_FILE = "shots.json"
KEY_VERSION = "clipmill.shots-stage.v1"
# The rendition this stage decodes, named so the lease can be searched by what an
# artifact is rather than by where it happened to land in a list.
PROXY_KIND = "media.proxy.v1"
STAGE = "detect-shots"
DECODER = "ffmpeg"

LOGGER = logging.getLogger(__name__)


def execute_shots(context: TaskContext) -> tuple[str, ...]:
    context.cancellation.raise_if_cancelled()
    payload = _payload(context)
    try:
        decoder = require_tool(context.lease, DECODER)
    except ToolUnavailableError as error:
        # A decoder that is missing or is not a program will not become one on
        # a retry, and falling back to whatever is on the PATH would publish
        # under an address claiming the pinned build produced these frames.
        raise DeterministicTaskError(str(error)) from error
    if decoder.bom != payload.decoder_bom:
        raise DeterministicTaskError(
            f"this task was keyed for {payload.decoder_bom} and the lease provides "
            f"{decoder.bom}; the artifact address would name a decoder that did not run"
        )

    # The proxy arrives on the lease rather than in the payload: a worker may
    # open exactly what its lease named, and the payload is hashed into the
    # artifact key — an address there would be present when this stage runs alone
    # and absent when it runs inside an analysis, which is one detection with two
    # addresses.
    try:
        proxy_input = require_input(context, PROXY_KIND)
    except MissingInputError as error:
        raise DeterministicTaskError(str(error)) from error
    try:
        artifact = proxy_input.artifact
        proxy = read_proxy(artifact, context.artifact_file(artifact, PROXY_DESCRIPTOR))
        media = context.artifact_file(artifact, proxy.file)
    except ArtifactVerificationError as error:
        raise DeterministicTaskError(str(error)) from error

    height = payload.detection.analysis_height or content.DEFAULT_ANALYSIS_HEIGHT
    threshold = payload.detection.threshold or content.DEFAULT_THRESHOLD
    min_shot_ticks = payload.detection.min_shot_ticks or content.DEFAULT_MIN_SHOT_TICKS
    try:
        size = analysis_size(proxy.width, proxy.height, height)
    except DecodeFailed as error:
        raise DeterministicTaskError(str(error)) from error
    parameters = Parameters(
        threshold=threshold,
        min_shot_frames=_min_shot_frames(min_shot_ticks, proxy),
    )

    # A frame count is the honest total here. The descriptor's duration is what
    # the container claims, and the difference between claim and decode is
    # exactly the sort of thing coverage exists to record.
    def progress(done: int) -> None:
        context.cancellation.raise_if_cancelled()
        context.report_progress("frames_decoded", done)

    try:
        frames = decode_frames(decoder.path, media, size, on_frame=progress)
        cuts, seen = content.detect(
            frames,
            parameters,
            frame_rate=proxy.rate_num / proxy.rate_den,
        )
    except DecodeFailed as error:
        # A proxy that will not decode is a proxy that will not decode next
        # time either. Publishing a document that says "no cuts" would be a lie
        # with a content address attached.
        raise DeterministicTaskError(str(error)) from error
    except content.DetectionRefused as error:
        raise DeterministicTaskError(str(error)) from error

    document = _document(payload, proxy_input.artifact_id, proxy, size, parameters, cuts, seen)
    context.staging.write_bytes(OUTPUT_FILE, canonical_bytes(document))
    return (OUTPUT_FILE,)


def _payload(context: TaskContext) -> daemon_pb2.ShotsStagePayloadV1:
    payload = daemon_pb2.ShotsStagePayloadV1()
    try:
        payload.ParseFromString(context.lease.payload)
    except Exception as error:
        # Every way this can fail is the same fact: the daemon and this worker
        # disagree about what a shots task looks like, and a retry will not
        # change that.
        raise DeterministicTaskError("task payload is not a shots stage payload") from error
    if payload.key_version != KEY_VERSION or payload.stage != STAGE:
        raise DeterministicTaskError("task payload does not describe shot detection")
    if not payload.decoder_bom:
        raise DeterministicTaskError("task payload names no decoder build")
    return payload


def _min_shot_frames(min_shot_ticks: int, proxy: Proxy) -> int:
    """Ceiling, and never below one.

    Rounding down would suppress less than the caller asked for, which is the
    direction that produces two cuts where a viewer sees one.
    """

    per_frame = frames_to_ticks(1, proxy.rate_num, proxy.rate_den)
    if per_frame <= 0:
        raise DeterministicTaskError("the proxy declares a frame rate with no duration")
    return max(1, -(-min_shot_ticks // per_frame))


def _document(
    payload: daemon_pb2.ShotsStagePayloadV1,
    proxy_artifact_id: str,
    proxy: Proxy,
    size: AnalysisSize,
    parameters: Parameters,
    cuts: list[content.Cut],
    frame_count: int,
) -> EvidenceShots:
    def ticks(frame: int) -> int:
        return frames_to_ticks(frame, proxy.rate_num, proxy.rate_den)

    def distribution(value: content.Confidence) -> Confidence:
        return Confidence(p50=value.p50, p10=value.p10)

    return EvidenceShots(
        schema_version="clipmill.evidence.shots.v1",
        source_fingerprint=payload.source_fingerprint or proxy.source_fingerprint,
        proxy_artifact_id=proxy_artifact_id,
        producer=Producer(
            stage=STAGE,
            implementation=f"clipmill-worker-shots@{__version__}+{IMPLEMENTATION}",
            calibration=CALIBRATION,
        ),
        detection=Detection(
            threshold=parameters.threshold,
            min_shot_ticks=ticks(parameters.min_shot_frames),
            analysis_height=size.height,
            frame_rate=Timebase(num=proxy.rate_num, den=proxy.rate_den),
            decoder=payload.decoder_bom,
        ),
        # Frames decoded, not duration promised. A container that claims more
        # than it holds is a source map problem, and this document should say
        # what was examined either way.
        coverage=Coverage(
            start_ticks=0,
            end_ticks=ticks(frame_count),
            analyzed=True,
            sampling_plan=(
                f"{IMPLEMENTATION}-at-{proxy.rate_num}-{proxy.rate_den}-{size.width}x{size.height}"
            ),
        ),
        cuts=[
            Cut(
                t_ticks=ticks(cut.frame),
                score=round(cut.score, 4),
                confidence=distribution(cut.confidence),
            )
            for cut in cuts
        ],
        shots=[
            Shot(
                start_ticks=ticks(span.start_frame),
                end_ticks=ticks(span.end_frame),
                confidence=distribution(span.confidence),
            )
            for span in content.spans(cuts, frame_count)
        ],
        invalid_regions=[],
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="ClipMill shot detection worker")
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
            family="detect-shots",
            capabilities=CAPABILITIES,
            backend="cpu",
            # Declared honestly: one decoder process and one frame at a time.
            # A 320x180 BGR frame is under 200 kB, and OpenCV's scratch buffers
            # are the same order; the allowance is dominated by the decoder.
            cpu_threads=2,
            max_memory_bytes=512 * 1024 * 1024,
        )
    )
    if arguments.once:
        client.run_one(execute_shots)
    else:
        client.run(execute_shots, stop)
    return 0


__all__ = ["CAPABILITIES", "__version__", "execute_shots", "main"]
