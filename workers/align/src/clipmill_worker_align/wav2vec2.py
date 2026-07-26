"""The CTC acoustic model: audio in, per-frame character posteriors out.

Deliberately knows nothing about words. It reports how likely each of the
model's thirty-two labels was during each twenty-millisecond frame, and `ctc`
turns that into an assignment of frames to the characters somebody actually
said. Keeping the two apart is what makes the alignment algorithm testable
against emissions written by hand.
"""

from __future__ import annotations

import json
from pathlib import Path

import numpy as np
import onnxruntime as ort
from clipmill_worker_sdk.weights import VerifiedModel

MODEL_FILE = "onnx/model.onnx"
VOCAB_FILE = "vocab.json"
PREPROCESSOR_FILE = "preprocessor_config.json"
IMPLEMENTATION = "wav2vec2-base-960h-ctc"

# The convolutional feature extractor reduces 320 input samples to one frame.
# At 16 kHz that is exactly 20 ms, and exactly 1800 ticks — the resolution of
# every word boundary this stage can honestly report.
FRAME_STRIDE_SAMPLES = 320
# The receptive field of that stack. A window shorter than this produces no
# frames at all rather than a short answer.
RECEPTIVE_FIELD_SAMPLES = 400


class Wav2Vec2Ctc:
    """One loaded session, reused across every utterance."""

    def __init__(self, model: VerifiedModel) -> None:
        self.model = model
        preprocessor = json.loads(model.path(PREPROCESSOR_FILE).read_text(encoding="utf-8"))
        # Read rather than assumed. A model trained on normalized waveforms and
        # fed raw ones still returns posteriors; they are simply wrong, and
        # every word would be mis-timed with no error anywhere.
        self.normalize = bool(preprocessor.get("do_normalize", True))
        self.sample_rate = int(preprocessor.get("sampling_rate", 16_000))

        options = ort.SessionOptions()
        # Single-threaded, for the same reason the render profile pins its
        # thread count: a reduction whose order depends on scheduling can move
        # a posterior across the argmax and shift a word boundary by a frame.
        options.intra_op_num_threads = 1
        options.inter_op_num_threads = 1
        options.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL
        self.session = ort.InferenceSession(
            str(model.path(MODEL_FILE)),
            sess_options=options,
            providers=["CPUExecutionProvider"],
        )
        self.input_name = self.session.get_inputs()[0].name

    @property
    def vocabulary_path(self) -> Path:
        return self.model.path(VOCAB_FILE)

    def emissions(self, samples: np.ndarray) -> np.ndarray:
        """Per-frame log-probabilities over the alphabet, `[frames, labels]`."""

        if samples.size < RECEPTIVE_FIELD_SAMPLES:
            return np.zeros((0, 0), dtype=np.float32)
        window = samples.astype(np.float32)
        if self.normalize:
            # Zero mean, unit variance, per utterance — what the feature
            # extractor config declares and what the model was trained on.
            window = (window - window.mean()) / np.sqrt(window.var() + 1e-7)
        logits = self.session.run(None, {self.input_name: window.reshape(1, -1)})[0][0]
        return _log_softmax(logits.astype(np.float64))


def _log_softmax(logits: np.ndarray) -> np.ndarray:
    shifted = logits - logits.max(axis=-1, keepdims=True)
    return shifted - np.log(np.exp(shifted).sum(axis=-1, keepdims=True))


def decode_pcm16(frames: bytes) -> np.ndarray:
    return np.frombuffer(frames, dtype="<i2").astype(np.float32) / 32768.0


__all__ = [
    "FRAME_STRIDE_SAMPLES",
    "IMPLEMENTATION",
    "MODEL_FILE",
    "PREPROCESSOR_FILE",
    "RECEPTIVE_FIELD_SAMPLES",
    "VOCAB_FILE",
    "Wav2Vec2Ctc",
    "decode_pcm16",
]
