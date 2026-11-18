"""Whether this processor can run the bundled whisper.cpp at all.

The pywhispercpp wheel carries ggml built for x86-64 with AVX2, FMA and F16C.
A processor without them (an Ivy Bridge i7, issue #140) does not get an error
from it: the worker dies on an illegal instruction, restarts, and dies again.
Asking the CPU first turns that into one sentence a person can act on.
"""

from __future__ import annotations

import platform

# ggml's names for what its x86-64 build uses, as numpy reports them.
REQUIRED = ("AVX2", "FMA3", "F16C")


def missing_instructions(
    features: dict[str, bool] | None = None, machine: str | None = None
) -> list[str]:
    """What the bundled build needs that this processor lacks; empty when nothing is
    missing or the processor is not x86-64."""
    machine = (machine or platform.machine()).lower()
    if machine not in ("x86_64", "amd64"):
        return []
    if features is None:
        from numpy._core._multiarray_umath import __cpu_features__ as features
    return [name for name in REQUIRED if not features.get(name, False)]
