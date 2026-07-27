"""The silero-VAD ONNX session: audio in, one speech probability per window.

This module knows the model's shape and nothing about what the probabilities
mean. It is deliberately thin, because everything it produces is a number that
`segmentation` then has to make a decision with, and mixing the two would make
the decisions untestable without the weights.
"""

from __future__ import annotations

from collections.abc import Callable

import numpy as np
import onnxruntime as ort
from clipmill_worker_sdk.weights import VerifiedModel

# The v5 model is trained on these window sizes and no others; feeding it a
# different length produces numbers, but not calibrated ones.
WINDOW_SAMPLES = {16_000: 512, 8_000: 256}
STATE_SHAPE = (2, 1, 128)
MODEL_FILE = "onnx/model.onnx"
IMPLEMENTATION = "silero-vad-v5"


class SileroVoiceActivity:
    """One loaded session, reused across the whole recording."""

    def __init__(self, model: VerifiedModel, sample_rate: int) -> None:
        if sample_rate not in WINDOW_SAMPLES:
            raise ValueError(f"silero-vad does not support {sample_rate} Hz audio")
        options = ort.SessionOptions()
        # Single-threaded on purpose. Reduction order in a parallel kernel is
        # not fixed, so a multi-threaded run can put a probability on the far
        # side of the threshold and move a segment boundary. A transcript that
        # depends on how busy the machine was is not one a cache can serve.
        options.intra_op_num_threads = 1
        options.inter_op_num_threads = 1
        options.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL
        options.graph_optimization_level = ort.GraphOptimizationLevel.ORT_ENABLE_ALL
        self.session = ort.InferenceSession(
            str(model.path(MODEL_FILE)),
            sess_options=options,
            providers=["CPUExecutionProvider"],
        )
        self.sample_rate = sample_rate
        self.window_samples = WINDOW_SAMPLES[sample_rate]
        self.model = model

    def probabilities(
        self,
        samples: np.ndarray,
        *,
        on_window: Callable[[int, int], None] | None = None,
    ) -> list[float]:
        """One probability per window, over the whole recording.

        The final partial window is zero-padded rather than dropped: speech
        that ends in the last twenty milliseconds of a recording is still
        speech, and dropping it would make the last word unalignable.
        """

        state = np.zeros(STATE_SHAPE, dtype=np.float32)
        rate = np.array(self.sample_rate, dtype=np.int64)
        window = self.window_samples
        total = (len(samples) + window - 1) // window
        scores: list[float] = []
        for index in range(total):
            chunk = samples[index * window : (index + 1) * window]
            if len(chunk) < window:
                chunk = np.pad(chunk, (0, window - len(chunk)))
            output, state = self.session.run(
                None,
                {
                    "input": chunk.reshape(1, window),
                    "state": state,
                    "sr": rate,
                },
            )
            scores.append(float(output[0][0]))
            if on_window is not None:
                on_window(index + 1, total)
        return scores


def decode_pcm16(frames: bytes) -> np.ndarray:
    """Interleaved 16-bit PCM to the float range the model was trained on."""

    return np.frombuffer(frames, dtype="<i2").astype(np.float32) / 32768.0


__all__ = [
    "IMPLEMENTATION",
    "MODEL_FILE",
    "WINDOW_SAMPLES",
    "SileroVoiceActivity",
    "decode_pcm16",
]
