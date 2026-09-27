"""The pinned voice-print model: CAM++ trained on VoxCeleb, from WeSpeaker.

It reads a window's filterbank frames, mean taken out, and answers a
512-number print of the voice in them. The session's threads are fixed, so a
recording printed twice on one machine prints the same.
"""

from __future__ import annotations

import numpy as np
import onnxruntime as ort
from clipmill_worker_sdk.weights import VerifiedModel

MODEL_FILE = "voxceleb_CAM++.onnx"
IMPLEMENTATION = "campplus-voxceleb"
INPUT = "feats"
THREADS = 2


class VoicePrints:
    def __init__(self, model: VerifiedModel) -> None:
        options = ort.SessionOptions()
        options.intra_op_num_threads = THREADS
        options.inter_op_num_threads = 1
        options.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL
        self.session = ort.InferenceSession(
            str(model.path(MODEL_FILE)), options, providers=["CPUExecutionProvider"]
        )
        inputs = self.session.get_inputs()
        if [item.name for item in inputs] != [INPUT] or inputs[0].shape[-1] != 80:
            raise ValueError("the pinned graph does not take 80-bin filterbank frames")

    def __call__(self, frames: np.ndarray) -> np.ndarray:
        """Windows by frames by 80 in; a print per window out."""

        (out,) = self.session.run(None, {INPUT: np.ascontiguousarray(frames, np.float32)})
        return np.asarray(out, np.float32)


__all__ = ["IMPLEMENTATION", "MODEL_FILE", "VoicePrints"]
