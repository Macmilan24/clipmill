"""Asking Whisper for what was said, fillers included.

Whisper writes clean text unless it is shown text that is not clean. The
prompt is context for every window, never output, and only languages with a
prompt someone has listened to get one.
"""

from __future__ import annotations

from pathlib import Path
from typing import ClassVar

from clipmill_worker_asr_whispercpp import engine
from clipmill_worker_sdk.weights import VerifiedModel


def model() -> VerifiedModel:
    return VerifiedModel(
        name="whisper-under-test",
        capability="asr",
        digest="sha256:" + "0" * 64,
        root=Path("/nowhere"),
        files=("ggml-base.bin",),
    )


class FakeWhisper:
    """Records the parameters whisper.cpp would be given."""

    params: ClassVar[dict[str, object]] = {}

    def __init__(self, path: str, **params: object) -> None:
        FakeWhisper.params = dict(params)

    def _set_params(self, params: dict[str, object]) -> None:
        FakeWhisper.params.update(params)


def test_a_window_is_primed_with_fillers_only_when_asked(monkeypatch):
    monkeypatch.setattr(engine, "Model", FakeWhisper)
    recognizer = engine.WhisperCppRecognizer(model(), weights="ggml-base.bin")
    assert "initial_prompt" not in FakeWhisper.params, "unprimed unless asked"

    recognizer.use_prompt(engine.VERBATIM_PROMPTS["en"])
    prompt = str(FakeWhisper.params["initial_prompt"]).lower()
    assert "umm" in prompt and "like" in prompt
    # Each window decodes on its own, primed the same way.
    assert FakeWhisper.params["no_context"] is True


def test_only_a_listened_to_prompt_is_used():
    assert set(engine.VERBATIM_PROMPTS) == {"en"}
    assert engine.VERBATIM == "verbatim-1"
