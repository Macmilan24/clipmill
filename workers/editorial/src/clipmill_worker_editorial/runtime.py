"""Offline MLX inference with a JSON grammar and per-token cancellation."""

from __future__ import annotations

import gc
import os
import threading
import time
import traceback
from contextlib import suppress

MLX_RUNTIME = "mlx-vlm@0.7.1/clipmill-json-v2"

#: How long a model stays loaded after its last task, so the next editorial
#: stage of the same recording (propose, then review, then look) does not load
#: it again. Zero closes a model as soon as its task ends.
KEEP_WARM = "CLIPMILL_EDITORIAL_KEEP_WARM_SECONDS"
DEFAULT_KEEP_WARM_SECONDS = 90.0


def open_runtime(model, cancellation):
    """The runtime a verified model runs on: llama.cpp for GGUF files, which is
    how Windows and Linux run editorial models, and MLX for the rest. The
    caller owns it and closes it; stages borrow one with `acquire_runtime`."""
    from .server_runtime import ServerModel, is_gguf

    if is_gguf(model):
        return ServerModel(model, cancellation)
    return LocalModel(model.root, cancellation)


def acquire_runtime(model, cancellation):
    """A runtime for one task, borrowed from the worker's resident model.

    Closing what this returns hands the model back rather than unloading it.
    """
    return RESIDENT.acquire(model, cancellation)


def keep_warm_seconds() -> float:
    value = os.environ.get(KEEP_WARM, "").strip()
    if not value:
        return DEFAULT_KEEP_WARM_SECONDS
    try:
        return max(0.0, float(value))
    except ValueError:
        return DEFAULT_KEEP_WARM_SECONDS


class ResidentModel:
    """At most one editorial model loaded in this worker, kept between tasks.

    Every task declares its own model's memory and is admitted only within
    the worker's ceiling, so holding that one model between tasks stays inside
    what the worker already promised the daemon. A lease for another model
    unloads the resident one before loading its own, so two are never held.
    A model whose call failed is unloaded rather than trusted again, and an
    idle one is unloaded after `keep_warm` seconds.
    """

    def __init__(self, opener=None, *, keep_warm=None, clock=time.monotonic):
        self._opener = opener or open_runtime
        self._keep_warm = keep_warm
        self._clock = clock
        self._lock = threading.Lock()
        self._key = None
        self._runtime = None
        self._borrowed = False
        self._idle_since = None
        self._timer = None

    def acquire(self, model, cancellation):
        key = (model.name, model.digest)
        with self._lock:
            if self._borrowed:
                raise RuntimeError("the resident editorial model is already in use")
            self._cancel_timer()
            if self._runtime is not None and (self._key != key or not _alive(self._runtime)):
                self._unload()
            if self._runtime is None:
                cancellation.raise_if_cancelled()
                self._runtime = self._opener(model, cancellation)
                self._key = key
            self._runtime.cancellation = cancellation
            self._borrowed = True
            self._idle_since = None
            return _Borrowed(self, self._runtime)

    def release(self, runtime, *, healthy):
        with self._lock:
            if runtime is not self._runtime:
                return
            self._borrowed = False
            seconds = self._seconds()
            if not healthy or seconds <= 0:
                self._unload()
                return
            _trim(runtime)
            self._idle_since = self._clock()
            self._timer = threading.Timer(seconds, self.expire)
            self._timer.daemon = True
            self._timer.start()

    def expire(self, now=None):
        """Unload the model if it has been idle for the keep-warm period."""
        with self._lock:
            if self._runtime is None or self._borrowed or self._idle_since is None:
                return
            now = self._clock() if now is None else now
            if now - self._idle_since >= self._seconds():
                self._unload()

    def close(self):
        """Unload whatever is resident: the worker is stopping."""
        with self._lock:
            self._cancel_timer()
            self._borrowed = False
            self._unload()

    @property
    def resident(self):
        return self._key if self._runtime is not None else None

    def _seconds(self):
        return keep_warm_seconds() if self._keep_warm is None else self._keep_warm

    def _cancel_timer(self):
        if self._timer is not None:
            self._timer.cancel()
            self._timer = None

    def _unload(self):
        self._cancel_timer()
        runtime, self._runtime, self._key, self._idle_since = self._runtime, None, None, None
        if runtime is not None:
            with suppress(Exception):
                runtime.close()


class _Borrowed:
    """One task's hold on the resident model. A failed call marks it unhealthy,
    so closing it unloads the model instead of returning it."""

    def __init__(self, owner, runtime):
        self._owner = owner
        self._runtime = runtime
        self._healthy = True
        self._closed = False

    def generate(self, *args, **kwargs):
        try:
            return self._runtime.generate(*args, **kwargs)
        except BaseException:
            self._healthy = False
            raise

    def __getattr__(self, name):
        return getattr(self._runtime, name)

    def close(self):
        if not self._closed:
            self._closed = True
            self._owner.release(self._runtime, healthy=self._healthy)


def _alive(runtime) -> bool:
    process = getattr(runtime, "process", None)
    return process is None or process.poll() is None


def _trim(runtime) -> None:
    """Free what the last task left behind while keeping the weights."""
    if isinstance(runtime, LocalModel):
        with suppress(Exception):
            import mlx.core as mx

            gc.collect()
            mx.clear_cache()


RESIDENT = ResidentModel()


def runtime_name(model) -> str:
    """The runtime as receipts name it."""
    from .server_runtime import is_gguf, server_build

    if is_gguf(model):
        return f"llama.cpp@{server_build()}/clipmill-json-v1"
    return MLX_RUNTIME


def implementation_suffix(model) -> str:
    """What a producer's implementation adds for the llama.cpp runtime, as the
    daemon's implementation table names it. MLX keeps the original names."""
    from .server_runtime import is_gguf

    return "-llama.cpp" if model is not None and is_gguf(model) else ""


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
