"""Exercise Windows kernel selection on every release test host."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path

PROJECT = Path(__file__).resolve().parents[3] / "integrations/whisper-windows"
SPEC = importlib.util.spec_from_file_location("_clipmill_cpu", PROJECT / "_clipmill_cpu.py")
assert SPEC is not None and SPEC.loader is not None
cpu = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(cpu)


class KernelSelection(unittest.TestCase):
    def test_ivy_bridge_uses_avx(self):
        self.assertEqual(cpu.selected_variant({"AVX": True, "SSE42": True, "F16C": True}), "avx")

    def test_unavailable_avx_state_falls_back_to_x64(self):
        self.assertEqual(cpu.selected_variant({"SSE42": True}), "x64")

    def test_v3_cpu_keeps_vectorized_backend(self):
        self.assertEqual(cpu.selected_variant({"X86_V3": True, "AVX": True}), "avx2")

    def test_partial_v3_features_do_not_load_avx2_extension(self):
        self.assertEqual(cpu.selected_variant({"AVX2": True, "AVX": True, "SSE42": True}), "avx")
