"""Select a compiled kernel before importing any whisper.cpp native code."""

from __future__ import annotations


def selected_variant(features: dict[str, bool] | None = None) -> str:
    if features is None:
        # NumPy checks both CPUID and OS support for saving vector registers.
        from numpy._core._multiarray_umath import __cpu_features__ as features

    # MSVC's /arch:AVX2 permits the complete x86-64-v3 instruction set.
    if features.get("X86_V3"):
        return "avx2"
    if features.get("AVX") and features.get("SSE42"):
        return "avx"
    return "x64"
