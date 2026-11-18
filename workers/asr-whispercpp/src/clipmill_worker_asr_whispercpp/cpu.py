"""Check a stock fixed-ISA wheel before importing its native extension."""

from __future__ import annotations

import platform
import sys
from importlib.util import find_spec

# ggml's names for what its x86-64 build uses, as numpy reports them.
REQUIRED = ("AVX2", "FMA3", "F16C", "BMI2")


def missing_instructions(
    features: dict[str, bool] | None = None,
    machine: str | None = None,
    *,
    dispatched: bool | None = None,
) -> list[str]:
    """What the bundled build needs that this processor lacks; empty when nothing is
    missing or the processor is not x86-64."""
    machine = (machine or platform.machine()).lower()
    if machine not in ("x86_64", "amd64"):
        return []
    if dispatched is None:
        dispatched = sys.platform == "win32" and find_spec("pywhispercpp._clipmill_cpu") is not None
    if dispatched:
        return []
    if features is None:
        from numpy._core._multiarray_umath import __cpu_features__ as features
    return [name for name in REQUIRED if not features.get(name, False)]
