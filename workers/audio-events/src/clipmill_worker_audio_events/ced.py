"""The CED-mini ONNX session: waveform windows in, 527 class scores out.

Thin for the same reason the VAD session is: everything it returns is a number
that `events` has to make a decision with, and keeping the two apart is what
lets the decisions be tested without the weights.
"""

from __future__ import annotations

import json
from collections.abc import Callable

import numpy as np
import onnxruntime as ort
from clipmill_worker_sdk.weights import VerifiedModel

MODEL_FILE = "model.onnx"
CONFIG_FILE = "config.json"
IMPLEMENTATION = "ced-mini"
SAMPLE_RATE = 16_000
# Two-second windows a second apart: long enough for the tagger to recognise a
# laugh, short enough that an event's edges land within a second of the sound.
WINDOW_SAMPLES = 2 * SAMPLE_RATE
HOP_SAMPLES = SAMPLE_RATE
# A fixed batch, padded with silence at the end. The scores for a window must
# not depend on how many windows the recording happened to have, and a GEMM
# blocked differently for a different batch size can move the last digit.
BATCH = 16


def window_starts(total_samples: int) -> list[int]:
    """The first sample of every window; at least one, however short the audio."""

    if total_samples <= WINDOW_SAMPLES:
        return [0]
    count = -(-(total_samples - WINDOW_SAMPLES) // HOP_SAMPLES) + 1
    return [index * HOP_SAMPLES for index in range(count)]


def labels(model: VerifiedModel) -> list[str]:
    """The class name of every output column, from the pinned config."""

    config = json.loads(model.path(CONFIG_FILE).read_text(encoding="utf-8"))
    names = config["id2label"]
    return [names[str(index)] for index in range(len(names))]


class AudioTagger:
    """One loaded session, reused across the whole recording."""

    def __init__(self, model: VerifiedModel) -> None:
        options = ort.SessionOptions()
        # Single-threaded for the reason the VAD session is: reduction order in
        # a parallel kernel is not fixed, and an event whose edge depends on how
        # busy the machine was is not one a cache can serve.
        options.intra_op_num_threads = 1
        options.inter_op_num_threads = 1
        options.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL
        options.graph_optimization_level = ort.GraphOptimizationLevel.ORT_ENABLE_ALL
        self.session = ort.InferenceSession(
            str(model.path(MODEL_FILE)),
            sess_options=options,
            providers=["CPUExecutionProvider"],
        )
        self.input = self.session.get_inputs()[0].name

    def scores(
        self,
        samples: np.ndarray,
        *,
        on_batch: Callable[[int, int], None] | None = None,
    ) -> list[list[float]]:
        """One row of class scores per window, in window order.

        The last window is zero-padded rather than dropped, so applause in the
        final second of a recording is still heard.
        """

        starts = window_starts(len(samples))
        rows: list[list[float]] = []
        for first in range(0, len(starts), BATCH):
            batch = np.zeros((BATCH, WINDOW_SAMPLES), dtype=np.float32)
            chosen = starts[first : first + BATCH]
            for row, start in enumerate(chosen):
                chunk = samples[start : start + WINDOW_SAMPLES]
                batch[row, : len(chunk)] = chunk
            (output,) = self.session.run(None, {self.input: batch})
            rows.extend(output[: len(chosen)].astype(float).tolist())
            if on_batch is not None:
                on_batch(min(first + BATCH, len(starts)), len(starts))
        return rows


def decode_pcm16(frames: bytes) -> np.ndarray:
    """16-bit PCM to the float range the model was trained on."""

    return np.frombuffer(frames, dtype="<i2").astype(np.float32) / 32768.0


__all__ = [
    "BATCH",
    "HOP_SAMPLES",
    "IMPLEMENTATION",
    "SAMPLE_RATE",
    "WINDOW_SAMPLES",
    "AudioTagger",
    "decode_pcm16",
    "labels",
    "window_starts",
]
