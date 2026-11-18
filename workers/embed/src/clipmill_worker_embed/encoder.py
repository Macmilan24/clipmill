"""The E5 encoder on onnxruntime: texts in, unit vectors out.

E5 is trained with a "passage: " prefix on what is searched and "query: " on
what is searched for; the vectors are the attention-masked mean of the last
hidden state, normalized to length one, so a dot product is the cosine.
"""

from __future__ import annotations

from collections.abc import Callable

import numpy as np
import onnxruntime as ort
from clipmill_worker_sdk.weights import VerifiedModel
from tokenizers import Tokenizer

MODEL_FILE = "onnx/model_quantized.onnx"
TOKENIZER_FILE = "tokenizer.json"
IMPLEMENTATION = "multilingual-e5-small-q8"
MAX_TOKENS = 256
BATCH = 16


class Encoder:
    def __init__(self, model: VerifiedModel) -> None:
        options = ort.SessionOptions()
        # One thread, as every ONNX worker here: a vector that depends on how
        # busy the machine was is not one a cache can serve.
        options.intra_op_num_threads = 1
        options.inter_op_num_threads = 1
        options.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL
        self.session = ort.InferenceSession(
            str(model.path(MODEL_FILE)), sess_options=options, providers=["CPUExecutionProvider"]
        )
        self.tokenizer = Tokenizer.from_file(str(model.path(TOKENIZER_FILE)))
        self.tokenizer.enable_truncation(MAX_TOKENS)
        self.tokenizer.enable_padding()
        self.dims = int(self.session.get_outputs()[0].shape[-1])

    def encode(
        self,
        texts: list[str],
        *,
        on_batch: Callable[[int, int], None] | None = None,
    ) -> np.ndarray:
        rows: list[np.ndarray] = []
        for first in range(0, len(texts), BATCH):
            batch = self.tokenizer.encode_batch(texts[first : first + BATCH])
            ids = np.array([item.ids for item in batch], dtype=np.int64)
            mask = np.array([item.attention_mask for item in batch], dtype=np.int64)
            (hidden,) = self.session.run(
                None,
                {"input_ids": ids, "attention_mask": mask, "token_type_ids": np.zeros_like(ids)},
            )
            weights = mask[..., None].astype(np.float32)
            pooled = (hidden * weights).sum(axis=1) / np.maximum(weights.sum(axis=1), 1.0)
            norms = np.linalg.norm(pooled, axis=1, keepdims=True)
            rows.append(pooled / np.maximum(norms, 1e-12))
            if on_batch is not None:
                on_batch(min(first + BATCH, len(texts)), len(texts))
        if not rows:
            return np.zeros((0, self.dims), dtype=np.float32)
        return np.concatenate(rows).astype(np.float32)
