"""The whisper.cpp session: audio in, text and per-token confidence out.

This is the universal fallback (book ch. 13). It lands before the accelerated
primary and stays after it, because "runs on any machine" is the property the
phase's offline exit gate actually rests on — an accelerated recognizer is a
speed decision, never an availability one.

Two settings here are not tuning:

Greedy at temperature zero, with the temperature fallback disabled. Whisper's
default is to retry a low-confidence window at rising temperatures, which is
sampling, which means the same audio can decode to different text on two runs.
A cached transcript is only worth caching if that cannot happen.

The CPU backend, explicitly. whisper.cpp will reach for Metal or CUDA if it
finds one, and the pinned manifest declares this model runs on the CPU —
admission budgets against that declaration, and a runtime that quietly used an
accelerator would make the declaration false in both directions.
"""

from __future__ import annotations

from dataclasses import dataclass

import numpy as np
from clipmill_worker_sdk.weights import VerifiedModel
from pywhispercpp.model import ContextParams, Model, pw

IMPLEMENTATION = "whisper.cpp"


@dataclass(frozen=True, slots=True)
class Token:
    text: str
    probability: float


@dataclass(frozen=True, slots=True)
class Decoded:
    text: str
    tokens: tuple[Token, ...]


class WhisperCppRecognizer:
    """One loaded model, reused across every decode window."""

    def __init__(self, model: VerifiedModel, *, weights: str, language: str = ""):
        self.model = model
        self.language = language
        self._whisper = Model(
            str(model.path(weights)),
            redirect_whispercpp_logs_to=False,
            context_params=ContextParams(use_gpu=False),
            n_threads=1,
            temperature=0.0,
            # Zero disables the fallback ladder entirely. Without this, the
            # temperature above is only the first thing tried.
            temperature_inc=0.0,
            # No window may see the previous window's text: one bad decode
            # should degrade one utterance, not everything after it.
            no_context=True,
            single_segment=False,
            print_progress=False,
            print_realtime=False,
            print_timestamps=False,
            suppress_blank=True,
            language=language or "en",
        )

    def detect_language(self, samples: np.ndarray) -> tuple[str, float]:
        (language, probability), _ = self._whisper.auto_detect_language(samples, n_threads=1)
        return (str(language), float(probability))

    def use_language(self, language: str) -> None:
        self.language = language
        # Reaching past the wrapper. It exposes no way to change the language
        # after construction, and re-loading the model to change one field
        # would cost a second read of the weights on every recording.
        self._whisper._set_params({"language": language})

    def decode(self, samples: np.ndarray) -> Decoded:
        """Decode one window, keeping the text and every token's probability.

        Per-token probabilities come from the C API rather than the Python
        wrapper's per-segment average. That average is exactly what hides the
        problem worth catching: one hallucinated proper noun inside an
        otherwise confident sentence, which ranking would happily quote.
        """

        self._whisper.transcribe(samples, n_processors=None)
        # Also past the wrapper: it surfaces a per-segment average, and the
        # per-token probabilities below are the whole reason to look.
        context = self._whisper._ctx
        # Everything at or above the end-of-transcript id is a control token —
        # timestamps, language markers, the segment beginning. They are not
        # things anyone said.
        special_from = pw.whisper_token_eot(context)

        pieces: list[str] = []
        tokens: list[Token] = []
        for segment in range(pw.whisper_full_n_segments(context)):
            for index in range(pw.whisper_full_n_tokens(context, segment)):
                if pw.whisper_full_get_token_id(context, segment, index) >= special_from:
                    continue
                text = pw.whisper_full_get_token_text(context, segment, index)
                probability = float(pw.whisper_full_get_token_p(context, segment, index))
                pieces.append(text)
                tokens.append(Token(text=text, probability=probability))
        return Decoded(text="".join(pieces).strip(), tokens=tuple(tokens))


def decode_pcm16(frames: bytes) -> np.ndarray:
    return np.frombuffer(frames, dtype="<i2").astype(np.float32) / 32768.0


__all__ = [
    "IMPLEMENTATION",
    "Decoded",
    "Token",
    "WhisperCppRecognizer",
    "decode_pcm16",
]
