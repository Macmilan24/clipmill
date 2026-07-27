"""Recognition with Qwen3-ASR, decoded one token at a time.

`mlx-audio` offers a one-call `generate()` that returns text and nothing else.
This module does not use it, and the reason is the observation contract: every
perception output carries a confidence *distribution*, and a recognizer that
returns only a string leaves nobody able to say which words it was unsure of.

Decoding through the streaming step instead yields the model's log-probability
vector at each position, so the emitted token's own probability is a
measurement rather than a placeholder. It is the same quantity whisper.cpp
reports as `whisper_full_get_token_p`, which is what makes the two
implementations' documents comparable.
"""

from __future__ import annotations

import math
from dataclasses import dataclass
from typing import Any

import numpy as np

from .languages import UNDETERMINED, to_language_code, to_model_name
from .runtime import ASR_MODEL_TYPE, load, supported_languages

#: A runaway generation is a real failure mode on a window of near-silence, so
#: the budget is bounded by how much speech the window could contain rather
#: than left open.
TOKENS_PER_SECOND = 12
MINIMUM_TOKEN_BUDGET = 128


@dataclass(frozen=True, slots=True)
class DecodedToken:
    text: str
    probability: float


@dataclass(frozen=True, slots=True)
class Decoded:
    text: str
    tokens: tuple[DecodedToken, ...]
    language: str


class Qwen3Recognizer:
    """One loaded Qwen3-ASR, decoding whichever windows it is handed."""

    def __init__(self, weights_dir, sample_rate: int = 16_000) -> None:
        self._model = load(weights_dir, ASR_MODEL_TYPE)
        self._supported = supported_languages(self._model)
        self._sample_rate = sample_rate
        self._language_name: str | None = None

    @property
    def sample_rate(self) -> int:
        return self._sample_rate

    def use_language(self, code: str) -> None:
        """Fix the language for every subsequent window.

        Deciding per window would let one noisy utterance be decoded as a
        different language than the rest, which produces text nobody can align
        and a transcript that claims two things at once.
        """

        self._language_name = to_model_name(code, self._supported)

    def decode(self, samples: np.ndarray) -> Decoded:
        import mlx.core as mx

        audio = np.ascontiguousarray(samples, dtype=np.float32)
        seconds = audio.size / max(self._sample_rate, 1)
        budget = max(MINIMUM_TOKEN_BUDGET, int(seconds * TOKENS_PER_SECOND) + 64)

        identifiers: list[int] = []
        probabilities: list[float] = []
        for token, logprobs in self._model.stream_generate(
            audio,
            max_tokens=budget,
            # Argmax, stated rather than inherited. A cached transcript is only
            # worth caching if the same audio produces the same bytes.
            sampler=lambda values: mx.argmax(values, axis=-1),
            language=self._language_name,
        ):
            identifier = int(token)
            identifiers.append(identifier)
            # The step's log-probability for the token it actually emitted.
            probabilities.append(math.exp(float(logprobs[identifier])))

        tokenizer = self._model._tokenizer
        # Decoded whole, because byte-level BPE splits characters across
        # tokens and joining per-token strings would corrupt anything
        # non-ASCII.
        text = tokenizer.decode(identifiers, skip_special_tokens=True).strip()
        language = self._language_name
        if language is None:
            language, text = self._model.extract_language(text)
            text = text.strip()
        return Decoded(
            text=text,
            tokens=tuple(
                DecodedToken(
                    text=tokenizer.decode([identifier], skip_special_tokens=True),
                    probability=probability,
                )
                for identifier, probability in zip(identifiers, probabilities, strict=True)
            ),
            language=to_language_code(language) if language else UNDETERMINED,
        )

    def detect_language(self, samples: np.ndarray) -> tuple[str, float]:
        """The language of one window, with how sure the model was.

        Qwen3 detects by generating `language X<asr_text>…` when no language is
        fixed, so the detection is the model's own first few tokens and its
        confidence is theirs. Reported as a rounded median rather than a
        product, so a long window is not automatically less certain than a
        short one.
        """

        decoded = self.decode(samples)
        head = decoded.tokens[:4]
        if not head:
            return (UNDETERMINED, 0.0)
        ranked = sorted(token.probability for token in head)
        return (decoded.language, ranked[len(ranked) // 2])


def peak_resident_bytes(model: Any) -> int:
    """What this model actually cost, as MLX measured it."""

    import mlx.core as mx

    return int(mx.get_peak_memory())


__all__ = [
    "Decoded",
    "DecodedToken",
    "Qwen3Recognizer",
    "peak_resident_bytes",
]
