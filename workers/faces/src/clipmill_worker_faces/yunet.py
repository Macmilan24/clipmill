"""The YuNet session, and the arithmetic that turns its tensors into boxes.

The model emits twelve tensors — classification, objectness, box regression and
keypoints at three strides — and none of them is a face until this module
decodes them against the anchor grid each stride implies. That decoding is the
part a wrapper would normally hide, and writing it out is what lets it be tested
against tensors built by hand rather than only against whatever the model
happens to say about a photograph.

Determinism is not incidental here. The boxes reach an artifact addressed by
content, and the crop path solved from them is compared against a golden, so a
detector whose output shifted with how busy the machine was would make two runs
of the same recording disagree while sharing one address. The session is
single-threaded and sequential for exactly the reason the VAD worker's is.
"""

from __future__ import annotations

from dataclasses import dataclass

import numpy as np
import onnxruntime as ort
from clipmill_worker_sdk.weights import VerifiedModel

MODEL_FILE = "models/face_detection_yunet/face_detection_yunet_2023mar.onnx"
IMPLEMENTATION = "yunet-2023mar"
#: The graph's input is a fixed 640x640 NCHW tensor; the anchor grids below are
#: derived from it and the model cannot be fed anything else.
INPUT_SIZE = 640
#: One anchor grid per stride, 80x80, 40x40 and 20x20 respectively.
STRIDES = (8, 16, 32)


@dataclass(frozen=True, slots=True)
class Detection:
    """One face in one frame, in pixels of the letterboxed input."""

    x: float
    y: float
    w: float
    h: float
    score: float


def _priors(stride: int) -> tuple[np.ndarray, np.ndarray]:
    """Column and row index of every anchor at this stride, in grid order.

    Row-major, because that is the order the model flattens its own grid in.
    Reading it the other way produces boxes that are plausible, mirrored about
    the diagonal, and wrong in a way nothing downstream would catch.
    """

    side = INPUT_SIZE // stride
    rows, columns = np.meshgrid(np.arange(side), np.arange(side), indexing="ij")
    return columns.reshape(-1).astype(np.float32), rows.reshape(-1).astype(np.float32)


def decode(outputs: dict[str, np.ndarray], score_threshold: float) -> list[Detection]:
    """Turn the model's twelve tensors into boxes above a score.

    The score is the geometric mean of the classification and objectness heads,
    which is YuNet's own combination: either head alone is confident about
    things the other rejects.
    """

    found: list[Detection] = []
    for stride in STRIDES:
        classification = outputs[f"cls_{stride}"].reshape(-1)
        objectness = outputs[f"obj_{stride}"].reshape(-1)
        boxes = outputs[f"bbox_{stride}"].reshape(-1, 4)
        columns, rows = _priors(stride)

        scores = np.sqrt(np.clip(classification, 0.0, 1.0) * np.clip(objectness, 0.0, 1.0))
        keep = np.nonzero(scores >= score_threshold)[0]
        if keep.size == 0:
            continue

        # The regression head predicts an offset from the anchor's own cell and
        # a size in log space, both in units of the stride.
        centre_x = (columns[keep] + boxes[keep, 0]) * stride
        centre_y = (rows[keep] + boxes[keep, 1]) * stride
        width = np.exp(boxes[keep, 2]) * stride
        height = np.exp(boxes[keep, 3]) * stride

        for index in range(keep.size):
            found.append(
                Detection(
                    x=float(centre_x[index] - width[index] / 2.0),
                    y=float(centre_y[index] - height[index] / 2.0),
                    w=float(width[index]),
                    h=float(height[index]),
                    score=float(scores[keep[index]]),
                )
            )
    return found


def intersection_over_union(left: Detection, right: Detection) -> float:
    """Overlap of two boxes, zero when they do not touch."""

    x0 = max(left.x, right.x)
    y0 = max(left.y, right.y)
    x1 = min(left.x + left.w, right.x + right.w)
    y1 = min(left.y + left.h, right.y + right.h)
    if x1 <= x0 or y1 <= y0:
        return 0.0
    overlap = (x1 - x0) * (y1 - y0)
    union = left.w * left.h + right.w * right.h - overlap
    return overlap / union if union > 0.0 else 0.0


def suppress(detections: list[Detection], iou: float) -> list[Detection]:
    """Non-maximum suppression, strongest first.

    Ordered by score and then by position, so two detections of identical
    strength resolve the same way on every machine. A stable order is not
    fastidiousness — it is what makes the surviving box the same box twice.
    """

    ordered = sorted(detections, key=lambda item: (-item.score, item.x, item.y))
    kept: list[Detection] = []
    for candidate in ordered:
        if all(intersection_over_union(candidate, existing) <= iou for existing in kept):
            kept.append(candidate)
    return kept


class YuNet:
    """One loaded session, reused across every frame of a recording."""

    def __init__(self, model: VerifiedModel) -> None:
        options = ort.SessionOptions()
        # Single-threaded on purpose, and for the reason the VAD worker states:
        # reduction order in a parallel kernel is not fixed, so a busy machine
        # can put a score on the far side of the threshold and change which
        # faces exist. These boxes reach a content address.
        options.intra_op_num_threads = 1
        options.inter_op_num_threads = 1
        options.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL
        options.graph_optimization_level = ort.GraphOptimizationLevel.ORT_ENABLE_ALL
        self.session = ort.InferenceSession(
            str(model.path(MODEL_FILE)),
            sess_options=options,
            providers=["CPUExecutionProvider"],
        )
        self.model = model
        self._input = self.session.get_inputs()[0].name
        self._outputs = [output.name for output in self.session.get_outputs()]

    def detect(self, frame: np.ndarray, score_threshold: float, nms_iou: float) -> list[Detection]:
        """Faces in one letterboxed BGR frame, suppressed and above a score.

        Blue first, because these weights were trained through OpenCV and that
        is the order they learned faces in. `frames.decode_frames` asks the
        decoder for exactly this, and the reason is written down there.
        """

        if frame.shape != (INPUT_SIZE, INPUT_SIZE, 3):
            raise ValueError(f"expected a {INPUT_SIZE}x{INPUT_SIZE} BGR frame, got {frame.shape}")
        # NCHW float, which is what the graph declares. No mean subtraction and
        # no scaling: YuNet is trained on raw 0-255 values.
        tensor = np.transpose(frame, (2, 0, 1))[np.newaxis, ...].astype(np.float32)
        produced = self.session.run(self._outputs, {self._input: tensor})
        outputs = dict(zip(self._outputs, produced, strict=True))
        return suppress(decode(outputs, score_threshold), nms_iou)
