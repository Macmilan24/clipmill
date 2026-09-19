"""Offline MLX inference with a JSON grammar and per-token cancellation."""

from __future__ import annotations

import gc
import os
import time
import traceback
from contextlib import suppress


class LocalModel:
    def __init__(self, root, cancellation):
        os.environ["HF_HUB_OFFLINE"] = "1"
        os.environ["TRANSFORMERS_OFFLINE"] = "1"
        import mlx.core as mx
        from mlx_vlm import load

        cancellation.raise_if_cancelled()
        mx.random.seed(0)
        self.model = None
        self.processor = None
        try:
            self.model, self.processor = load(str(root), trust_remote_code=False)
        except BaseException as error:
            # A failed constructor never reaches the stage's runtime.close().
            # Unwound loader frames can still own partial tensors. Clear their
            # locals before collecting, retaining the original error and stack.
            with suppress(Exception):
                traceback.clear_frames(error.__traceback__)
            with suppress(Exception):
                self.close()
            raise
        self.cancellation = cancellation

    def generate(self, prompt, schema, max_tokens, images=None):
        from mlx_vlm import stream_generate
        from mlx_vlm.prompt_utils import apply_chat_template
        from mlx_vlm.structured import build_json_schema_logits_processor

        tokenizer = getattr(self.processor, "tokenizer", self.processor)
        # Match the chat EOS rather than the checkpoint's pretraining EOS.
        tokenizer.stopping_criteria.reset([tokenizer.eos_token_id])
        grammar = build_json_schema_logits_processor(tokenizer, schema)
        deadline = time.monotonic() + 600

        def check_cancel(_tokens, logits):
            self.cancellation.raise_if_cancelled()
            if time.monotonic() > deadline:
                raise TimeoutError("editorial call exceeded the 10-minute limit")
            return logits

        formatted = apply_chat_template(
            self.processor,
            self.model.config,
            prompt,
            num_images=len(images or []),
            enable_thinking=False,
        )
        if len(tokenizer.encode(formatted)) > 12_000:
            raise ValueError("editorial context exceeds the 12,000-token limit")
        chunks = []
        last = None
        for last in stream_generate(
            self.model,
            self.processor,
            formatted,
            image=images,
            max_tokens=max_tokens,
            temperature=0.0,
            logits_processors=[check_cancel, grammar],
            enable_thinking=False,
            prefill_step_size=512,
            resize_shape=(448, 448),
        ):
            self.cancellation.raise_if_cancelled()
            chunks.append(last.text)
        return "".join(chunks), {
            "input": last.prompt_tokens if last else 0,
            "output": last.generation_tokens if last else 0,
        }

    def close(self):
        import mlx.core as mx

        self.model = None
        self.processor = None
        try:
            gc.collect()
        finally:
            mx.clear_cache()
