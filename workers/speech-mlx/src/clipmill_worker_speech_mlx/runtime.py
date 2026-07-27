"""Loading pinned MLX weights, and saying so when the runtime is not here.

MLX is Apple silicon and nothing else. This module is the one place that knows
it: everywhere else imports from here and gets either a loaded model or an
error that names the reason, rather than an `ImportError` traceback the
scheduler would have to interpret.

Nothing is downloaded. `base_load_model` will happily fetch from Hugging Face
when handed a repository id, which is exactly what the Local Lock forbids, so
it is only ever handed a `Path` that `verify_model` has already hashed.
"""

from __future__ import annotations

import os
from pathlib import Path
from typing import Any

#: Deterministic decoding starts before any model is loaded: MLX picks its
#: default device from the environment, and a run that silently fell back to
#: the CPU would be a different measurement under the same name.
os.environ.setdefault("MLX_DISABLE_COMPILE", "0")

#: The model types as `mlx-audio` registers them. Passed explicitly so the
#: architecture never depends on what the weights directory happens to be
#: called.
ASR_MODEL_TYPE = "qwen3_asr"
ALIGNER_MODEL_TYPE = "qwen3_forced_aligner"

SAMPLE_RATE = 16_000


class MlxUnavailable(RuntimeError):
    """MLX is not installed here, or this machine cannot run it."""


def implementation() -> str:
    """What produced a document, precisely enough to reproduce it.

    The runtime version belongs in the producer string because these models are
    generative: `mlx-audio` changing how it builds a prompt changes the text,
    and two transcripts that differ must not share a producer identity.
    """

    from mlx_audio.version import __version__ as mlx_audio_version

    return f"mlx-audio-{mlx_audio_version}"


def require_mlx() -> None:
    """Refuse early, with the reason, on a machine that cannot run this."""

    try:
        import mlx.core as mx
    except ImportError as error:  # pragma: no cover - platform-specific
        raise MlxUnavailable(
            "mlx is not installed; this implementation runs only on Apple silicon"
        ) from error
    if not mx.metal.is_available():  # pragma: no cover - platform-specific
        raise MlxUnavailable("no Metal device is available to this process")


def load(weights_dir: Path, model_type: str) -> Any:
    """Load one pinned model from a directory whose digests were checked."""

    require_mlx()
    from mlx_audio.stt.utils import load_model

    return load_model(
        Path(weights_dir),
        lazy=False,
        # Every pinned file must be accounted for. A quietly partial load is a
        # model whose outputs nobody can attribute to the weights we pinned.
        strict=True,
        model_type=model_type,
        model_name_parts=[model_type],
    )


def supported_languages(model: Any) -> frozenset[str] | None:
    """The languages the weights themselves claim, lower-cased."""

    listed = getattr(model, "get_supported_languages", None)
    if listed is None:
        return None
    names = listed()
    if not names:
        return None
    return frozenset(str(name).strip().lower() for name in names)


__all__ = [
    "ALIGNER_MODEL_TYPE",
    "ASR_MODEL_TYPE",
    "SAMPLE_RATE",
    "MlxUnavailable",
    "implementation",
    "load",
    "require_mlx",
    "supported_languages",
]
