"""Which pinned file is the one whisper.cpp loads.

The registry pins two whisper models with different weight file names, so a
stage that hardcoded one could never be given the other — and a stage that
guessed would load the wrong file on a manifest nobody meant it to see.
"""

from __future__ import annotations

from pathlib import Path

import pytest
from clipmill_worker_asr_whispercpp import weights_file
from clipmill_worker_sdk import DeterministicTaskError
from clipmill_worker_sdk.weights import VerifiedModel


def model(*files: str) -> VerifiedModel:
    return VerifiedModel(
        name="whisper-under-test",
        capability="asr",
        digest="sha256:" + "0" * 64,
        root=Path("/nowhere"),
        files=files,
    )


def test_the_single_ggml_file_is_the_one_loaded():
    assert weights_file(model("ggml-base.bin", "README.md")) == "ggml-base.bin"
    assert weights_file(model("ggml-large-v3-turbo.bin")) == "ggml-large-v3-turbo.bin", (
        "a differently named weight file is found rather than assumed"
    )


def test_a_manifest_pinning_no_weights_is_a_refusal_not_a_guess():
    with pytest.raises(DeterministicTaskError, match="0 GGML weight files"):
        weights_file(model("README.md"))


def test_a_manifest_pinning_two_weights_names_no_choice():
    """Ambiguity belongs to whoever wrote the manifest, not to this worker."""

    with pytest.raises(DeterministicTaskError, match="2 GGML weight files"):
        weights_file(model("ggml-base.bin", "ggml-small.bin"))
