"""Face detection and tracking: who was on screen, and for how long.

The reframe solver has one piece of evidence about who deserves the frame, and
this stage produces it. That makes what it publishes more consequential than it
looks: a track that fragments becomes a clip nobody follows, and a track welded
together out of two people becomes a camera that swings between them.

So the document records what was detected rather than who mattered. Which track
the camera follows is decided later, in `clipmill-reframe`, by something that can
be argued with — and every number this stage publishes is kept in the form that
decision needs: presence and score apart rather than fused, bridged boxes marked
as bridged, and the parameters that produced all of it in the artifact key.

Two things are pinned rather than found. The weights arrive on the lease, as
they do for the speech family, because a stage that resolved its own model would
publish under an address claiming the pinned one produced these boxes. The JPEG
decoder arrives the same way, for the same reason the shot detector's does.
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
from clipmill_worker_sdk.gen.schemas.vision_face_track import (
    Box,
    Coverage,
    Producer,
    Timebase,
    VisionFaceTrack,
)
from clipmill_worker_sdk.gen.schemas.vision_face_track import (
    Detection as DetectionParameters,
)
from clipmill_worker_sdk.gen.schemas.vision_face_track import (
    Track as TrackDocument,
)
from clipmill_worker_sdk.inputs import MissingInputError, require_input
from clipmill_worker_sdk.tools import ToolUnavailableError, require_tool
from clipmill_worker_sdk.weights import ModelVerificationError, require_model

from . import frames as frames_module
from . import tracking
from .frames import FRAMES_DESCRIPTOR, DecodeFailed, decode_frames, letterbox_for, read_frames
from .yunet import IMPLEMENTATION, INPUT_SIZE, YuNet

__version__ = "0.1.0"
#: The registered task kind, which is what the daemon routes by.
CAPABILITIES = ("detect-faces",)
OUTPUT_FILE = "faces.json"
KEY_VERSION = "clipmill.faces-stage.v1"
FRAMES_KIND = "media.frames.v1"
STAGE = "detect-faces"
DECODER = "ffmpeg"

#: Defaults for anything the payload leaves at zero. Chosen against the failure
#: each prevents rather than tuned on a benchmark, which is what the reframe
#: corpus in W26 is for.
DEFAULT_SCORE_THRESHOLD = 0.6
DEFAULT_NMS_IOU = 0.3
DEFAULT_MATCH_IOU = 0.5
DEFAULT_RECOVER_IOU = 0.3
DEFAULT_MAX_GAP_FRAMES = 6
DEFAULT_MIN_TRACK_FRAMES = 4
#: How far below the detection threshold the second association pass reaches.
#: A face turning away loses roughly a third of its score before it disappears.
RECOVERY_FLOOR = 0.25

#: How many frames go into the decoder at once. Bounded because the whole batch
#: is held as raw pixels: at 640x640 RGB a frame is 1.2 MB, so this is under
#: 150 MB of buffer regardless of how long the recording is.
BATCH_FRAMES = 120

LOGGER = logging.getLogger(__name__)


def execute_faces(context: TaskContext) -> tuple[str, ...]:
    context.cancellation.raise_if_cancelled()
    payload = _payload(context)

    try:
        decoder = require_tool(context.lease, DECODER)
    except ToolUnavailableError as error:
        # A decoder that is missing will not become present on a retry, and
        # falling back to whatever is on the PATH would publish under an address
        # claiming the pinned build produced these boxes.
        raise DeterministicTaskError(str(error)) from error
    try:
        model = require_model(context.lease, STAGE)
    except ModelVerificationError as error:
        raise DeterministicTaskError(str(error)) from error

    try:
        frames_input = require_input(context, FRAMES_KIND)
    except MissingInputError as error:
        raise DeterministicTaskError(str(error)) from error
    try:
        artifact = frames_input.artifact
        sampled = read_frames(artifact, context.artifact_file(artifact, FRAMES_DESCRIPTOR))
    except ArtifactVerificationError as error:
        raise DeterministicTaskError(str(error)) from error

    settings = _settings(payload)
    if not sampled.frames:
        # A frame set with nothing in it is a recording with no video, which is
        # a fact rather than a failure — and it is a different fact from a pass
        # nobody ran.
        document = _document(payload, sampled, settings, [], 0)
        context.staging.write_bytes(OUTPUT_FILE, canonical_bytes(document))
        return (OUTPUT_FILE,)

    detector = YuNet(model)
    ordered = sorted(sampled.frames, key=lambda frame: frame.t_ticks)
    try:
        first = context.artifact_file(artifact, ordered[0].file)
        width, height = frames_module.jpeg_size(first)
        box = letterbox_for(width, height, INPUT_SIZE)
    except (ArtifactVerificationError, DecodeFailed) as error:
        raise DeterministicTaskError(str(error)) from error

    per_frame: list[tuple[int, list]] = []
    examined = 0
    for start in range(0, len(ordered), BATCH_FRAMES):
        context.cancellation.raise_if_cancelled()
        batch = ordered[start : start + BATCH_FRAMES]
        try:
            paths = [context.artifact_file(artifact, frame.file) for frame in batch]
            pixels = decode_frames(decoder.path, paths, box)
        except (ArtifactVerificationError, DecodeFailed) as error:
            # Frames that will not decode will not decode next time either.
            # Publishing "nobody was on screen" would be a lie with a content
            # address attached.
            raise DeterministicTaskError(str(error)) from error
        for frame, image in zip(batch, pixels, strict=True):
            found = detector.detect(image, settings.score_threshold, settings.nms_iou)
            per_frame.append((frame.t_ticks, found))
            examined += 1
        context.report_progress("frames_examined", examined, len(ordered))

    tracks = tracking.associate(
        per_frame,
        tracking.Parameters(
            start_score=settings.score_threshold,
            match_iou=settings.match_iou,
            recover_iou=settings.recover_iou,
            max_gap_frames=settings.max_gap_frames,
            min_track_frames=settings.min_track_frames,
        ),
    )
    frame_times = [at for at, _ in per_frame]
    bridged = [tracking.bridge(track, frame_times) for track in tracks]

    document = _document(payload, sampled, settings, bridged, examined, box)
    context.staging.write_bytes(OUTPUT_FILE, canonical_bytes(document))
    return (OUTPUT_FILE,)


class _Settings:
    """The parameters this run actually used, after defaults are applied."""

    __slots__ = (
        "match_iou",
        "max_gap_frames",
        "min_track_frames",
        "nms_iou",
        "recover_iou",
        "score_threshold",
    )

    def __init__(self, detection: daemon_pb2.FaceDetectionV1) -> None:
        self.score_threshold = detection.score_threshold or DEFAULT_SCORE_THRESHOLD
        self.nms_iou = detection.nms_iou or DEFAULT_NMS_IOU
        self.match_iou = detection.match_iou or DEFAULT_MATCH_IOU
        self.recover_iou = detection.recover_iou or DEFAULT_RECOVER_IOU
        self.max_gap_frames = detection.max_gap_frames or DEFAULT_MAX_GAP_FRAMES
        self.min_track_frames = detection.min_track_frames or DEFAULT_MIN_TRACK_FRAMES


def _settings(payload: daemon_pb2.FacesStagePayloadV1) -> _Settings:
    return _Settings(payload.detection)


def _payload(context: TaskContext) -> daemon_pb2.FacesStagePayloadV1:
    payload = daemon_pb2.FacesStagePayloadV1()
    try:
        payload.ParseFromString(context.lease.payload)
    except Exception as error:
        # Every way this can fail is the same fact: the daemon and this worker
        # disagree about what a faces task looks like, and a retry will not
        # change that.
        raise DeterministicTaskError("task payload is not a faces stage payload") from error
    if payload.key_version != KEY_VERSION or payload.stage != STAGE:
        raise DeterministicTaskError("task payload does not describe face detection")
    return payload


def _document(
    payload: daemon_pb2.FacesStagePayloadV1,
    sampled: frames_module.Frames,
    settings: _Settings,
    tracks: list[tracking.Track],
    examined: int,
    box: frames_module.Letterbox | None = None,
) -> VisionFaceTrack:
    published: list[TrackDocument] = []
    for index, track in enumerate(tracks):
        boxes: list[Box] = []
        for observation in track.observations:
            if box is None:
                continue
            x, y, w, h = box.to_normalized(
                observation.detection.x,
                observation.detection.y,
                observation.detection.w,
                observation.detection.h,
            )
            boxes.append(
                Box(
                    t_ticks=observation.t_ticks,
                    x=round(x, 6),
                    y=round(y, 6),
                    w=round(w, 6),
                    h=round(h, 6),
                    score=round(min(max(observation.detection.score, 0.0), 1.0), 4),
                    interpolated=True if observation.interpolated else None,
                )
            )
        if not boxes:
            continue
        published.append(
            TrackDocument(
                # Renumbered on publication so ids are dense and start at zero.
                # The association's own ids count every track it ever opened,
                # including the short ones it discarded, and a document whose
                # ids had holes in it would invite somebody to read meaning into
                # them.
                track_id=index,
                first_ticks=boxes[0].t_ticks,
                last_ticks=boxes[-1].t_ticks,
                frames_present=max(track.seen, 1),
                mean_score=round(track.mean_score, 4),
                boxes=boxes,
            )
        )

    return VisionFaceTrack(
        schema_version="clipmill.vision.face_track.v1",
        source_fingerprint=payload.source_fingerprint or sampled.source_fingerprint,
        frames_artifact_id=sampled.artifact_id,
        producer=Producer(
            stage=STAGE,
            implementation=f"clipmill-worker-faces@{__version__}+{IMPLEMENTATION}",
        ),
        detection=DetectionParameters(
            score_threshold=settings.score_threshold,
            nms_iou=settings.nms_iou,
            input_width=INPUT_SIZE,
            input_height=INPUT_SIZE,
            match_iou=settings.match_iou,
            recover_iou=settings.recover_iou,
            max_gap_frames=settings.max_gap_frames,
            min_track_frames=settings.min_track_frames,
            frame_rate=Timebase(num=sampled.rate_num, den=sampled.rate_den),
        ),
        # Frames examined, not duration promised. The two differ when a frame
        # set is short of what its coverage claims, and this document should say
        # what was looked at either way.
        coverage=Coverage(
            start_ticks=sampled.coverage_start_ticks,
            end_ticks=sampled.coverage_end_ticks,
            analyzed=examined > 0,
            frames_examined=examined,
        ),
        tracks=published,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="ClipMill face detection worker")
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
            family="detect-faces",
            capabilities=CAPABILITIES,
            backend="onnx-cpu",
            # Declared honestly: one session, one decoder process, and a bounded
            # batch of raw frames. The weights are 230 kB; the allowance is
            # dominated by onnxruntime's arena and the frame buffer.
            cpu_threads=2,
            max_memory_bytes=768 * 1024 * 1024,
        )
    )
    if arguments.once:
        client.run_one(execute_faces)
    else:
        client.run(execute_faces, stop)
    return 0


__all__ = ["CAPABILITIES", "__version__", "execute_faces", "main"]
