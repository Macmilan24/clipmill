"""Offline MLX inference with a JSON grammar and per-token cancellation."""

from __future__ import annotations

import gc
import os
import tempfile
import threading
import time
import traceback
from contextlib import contextmanager, suppress

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


@contextmanager
def text_session(model, cancellation, source, *, needed):
    """Borrow the resident model and scope any prefix checkpoints to this recording."""
    if not needed:
        yield None
        return
    runtime = acquire_runtime(model, cancellation)
    try:
        if begin := getattr(runtime, "begin_text_session", None):
            begin(source)
        yield runtime
    finally:
        try:
            if end := getattr(runtime, "end_text_session", None):
                end()
        finally:
            runtime.close()


#: Where a reasoning model closes its thinking.
THINK_END = "</think>"
#: What follows the reasoning, so the next token is the answer itself.
ANSWER_LEAD = "Answer (one word, yes or no):"
#: The spellings of each answer a tokenizer may start a reply with.
YES_WORDS = ("yes", "Yes", " yes", " Yes")
NO_WORDS = ("no", "No", " no", " No")


def answer_prompt(prompt, thinking, thought):
    """The templated prompt, the reasoning cut where it ended or ran out, and
    the line that makes the next token the answer."""
    if thought:
        return prompt + thinking.split(THINK_END)[0] + "\n" + THINK_END + "\n\n" + ANSWER_LEAD
    return prompt + ANSWER_LEAD


def answer_token_ids(tokenizer):
    """The first token of each spelling of "yes" and of "no"."""

    def first(words):
        return sorted({tokenizer.encode(word, add_special_tokens=False)[0] for word in words})

    return first(YES_WORDS), first(NO_WORDS)


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

    def yes_probability(self, *args, **kwargs):
        try:
            return self._runtime.yes_probability(*args, **kwargs)
        except BaseException:
            self._healthy = False
            raise

    def continuation_logprobs(self, *args, **kwargs):
        try:
            return self._runtime.continuation_logprobs(*args, **kwargs)
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
    #: It can score given text, so it can serve as the likelihood meter.
    has_meter = True

    def __init__(self, root, cancellation):
        os.environ["HF_HUB_OFFLINE"] = "1"
        os.environ["TRANSFORMERS_OFFLINE"] = "1"
        import mlx.core as mx
        from mlx_vlm import load

        cancellation.raise_if_cancelled()
        mx.random.seed(0)
        self.model = None
        self.processor = None
        self.prefix_cache = None
        self.cache_directory = None
        self.cached_source = None
        self.cache_active = False
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

    def begin_text_session(self, source):
        """Keep checkpoints local to one recording and to this resident model."""
        from mlx_vlm import apc

        if self.cached_source != source:
            self._close_prefix_cache()
            self.cache_directory = tempfile.TemporaryDirectory(prefix="clipmill-editorial-")
            self.prefix_cache = apc.from_env(
                model_namespace="editorial",
                overrides={
                    "enabled": True,
                    "memory_max_gb": 0.25,
                    "num_blocks": 64,
                    "disk_enabled": True,
                    "disk_path": self.cache_directory.name,
                    "disk_max_gb": 0.375,
                    "disk_queue_max_gb": 0.0625,
                    "checkpoint_entries": 2,
                    "checkpoint_interval_tokens": 0,
                },
            )
            self.cached_source = source
        self.cache_active = True

    def end_text_session(self):
        self.cache_active = False

    def _close_prefix_cache(self):
        cache, self.prefix_cache = self.prefix_cache, None
        directory, self.cache_directory = self.cache_directory, None
        self.cached_source = None
        self.cache_active = False
        try:
            if cache is not None:
                cache.close()
        finally:
            if directory is not None:
                directory.cleanup()

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
        token_count = len(tokenizer.encode(formatted))
        if token_count > 12_000:
            raise ValueError("editorial context exceeds the 12,000-token limit")
        if self.cache_active and self.prefix_cache is not None and not images:
            prefix, marker, _ = formatted.partition("\nInput data:\n")
            if marker:
                # Stateful attention cannot rewind an arbitrary final snapshot.
                # Ask the runtime to checkpoint the shared instructions and
                # recording context while it prefills the first real request.
                # Leave two tokens for tokenization at the text boundary.
                shared_tokens = max(0, len(tokenizer.encode(prefix)) - 2)
                self.prefix_cache.exact_cache_guard_tokens = max(1, token_count - shared_tokens)
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
            apc_manager=self.prefix_cache if self.cache_active and not images else None,
            resize_shape=(448, 448),
        ):
            self.cancellation.raise_if_cancelled()
            chunks.append(last.text)
        tokens = {
            "input": last.prompt_tokens if last else 0,
            "output": last.generation_tokens if last else 0,
        }
        if self.cache_active and last is not None:
            tokens["cached_input"] = int(getattr(last, "cached_tokens", 0))
        return "".join(chunks), tokens

    def yes_probability(self, question, think_tokens):
        """The model's probability of "yes" against "no", after reasoning for
        at most `think_tokens`: the answer is read from the next token's
        distribution rather than sampled, so it is a number, not a coin flip."""
        import mlx.core as mx
        from mlx_vlm import stream_generate
        from mlx_vlm.prompt_utils import apply_chat_template

        tokenizer = getattr(self.processor, "tokenizer", self.processor)
        tokenizer.stopping_criteria.reset([tokenizer.eos_token_id])
        deadline = time.monotonic() + 600

        def check_cancel(_tokens, logits):
            self.cancellation.raise_if_cancelled()
            if time.monotonic() > deadline:
                raise TimeoutError("a judgement exceeded the 10-minute limit")
            return logits

        prompt = apply_chat_template(
            self.processor,
            self.model.config,
            question,
            num_images=0,
            enable_thinking=think_tokens > 0,
        )
        thinking = ""
        if think_tokens > 0:
            for chunk in stream_generate(
                self.model,
                self.processor,
                prompt,
                max_tokens=think_tokens,
                temperature=0.0,
                logits_processors=[check_cancel],
                enable_thinking=True,
            ):
                self.cancellation.raise_if_cancelled()
                thinking += chunk.text
                if THINK_END in thinking:
                    break
        ids = tokenizer.encode(
            answer_prompt(prompt, thinking, think_tokens > 0), add_special_tokens=False
        )
        logits = self.model.language_model(mx.array([ids])).logits[0, -1].astype(mx.float32)
        yes, no = answer_token_ids(tokenizer)
        p_yes = mx.logsumexp(logits[mx.array(yes)])
        p_no = mx.logsumexp(logits[mx.array(no)])
        probability = mx.exp(p_yes - mx.logaddexp(p_yes, p_no))
        mx.eval(probability)
        return float(probability.item()), thinking.split(THINK_END)[0]

    def continuation_logprobs(self, context, target):
        """The log probability of each token of `target` when it follows
        `context` as plain text. The model is a meter here, not a judge: the
        same line scored after two contexts says how much it relies on what
        one of them adds."""
        import mlx.core as mx

        self.cancellation.raise_if_cancelled()
        tokenizer = getattr(self.processor, "tokenizer", self.processor)
        before = tokenizer.encode(context, add_special_tokens=False)
        after = tokenizer.encode("\n" + target, add_special_tokens=False)
        if not before or not after:
            raise ValueError("the meter needs a context and a line to score")
        logits = self.model.language_model(mx.array([before + after])).logits[0].astype(mx.float32)
        logp = logits - mx.logsumexp(logits, axis=-1, keepdims=True)
        rows = logp[len(before) - 1 : len(before) - 1 + len(after)]
        picked = mx.take_along_axis(rows, mx.array(after)[:, None], axis=1)[:, 0]
        mx.eval(picked)
        return picked.tolist()

    def close(self):
        import mlx.core as mx

        self.model = None
        self.processor = None
        try:
            self._close_prefix_cache()
            gc.collect()
        finally:
            mx.clear_cache()
