"""Qwen3 forced alignment with timestamp-token confidence.

Each word's start and end are classifications at reserved token positions. The
forward pass retains their scores, which the library's ``generate()`` discards,
so incorrect alignments can be identified downstream. Tokenization, language
splitting, and monotonicity repair remain the library's implementations."""

from __future__ import annotations

from dataclasses import dataclass

import numpy as np

from .languages import to_model_name
from .runtime import ALIGNER_MODEL_TYPE, load, supported_languages


@dataclass(frozen=True, slots=True)
class AlignedWord:
    text: str
    start_ms: int
    end_ms: int
    #: The model's probability for each of the two boundary predictions. Two
    #: values rather than one, so a word whose start is certain and whose end
    #: is a guess does not average into "moderately sure".
    scores: tuple[float, ...]


class AlignmentImpossible(RuntimeError):
    """The text and the audio cannot be aligned against each other."""


class Qwen3Aligner:
    """One loaded Qwen3-ForcedAligner, placing whichever text it is handed."""

    def __init__(self, weights_dir, sample_rate: int = 16_000) -> None:
        self._model = load(weights_dir, ALIGNER_MODEL_TYPE)
        self._supported = supported_languages(self._model)
        self._sample_rate = sample_rate

    @property
    def sample_rate(self) -> int:
        return self._sample_rate

    @property
    def resolution_ms(self) -> int:
        """How coarse a boundary this model can express.

        Published on the artifact as `frame_ticks`, so a consumer can state how
        precise a word edge is instead of implying more precision than was
        measured.
        """

        return int(self._model.config.timestamp_segment_time)

    def align(self, samples: np.ndarray, text: str, language: str) -> list[AlignedWord]:
        import mlx.core as mx

        name = to_model_name(language, self._supported)
        audio = np.ascontiguousarray(samples, dtype=np.float32)
        model = self._model

        features, attention_mask, audio_tokens = model._preprocess_audio(audio)
        words, prompt = model.aligner_processor.encode_timestamp(text, name)
        if not words:
            raise AlignmentImpossible("the text contains nothing this aligner can place")
        prompt = prompt.replace("<|audio_pad|>", "<|audio_pad|>" * audio_tokens)
        input_ids = mx.array(
            model._tokenizer.encode(prompt, return_tensors="np", add_special_tokens=False)
        )

        logits = model(
            input_ids,
            input_features=features,
            feature_attention_mask=attention_mask,
        )
        mx.eval(logits)
        # One softmax over the timestamp head, read twice: once for the
        # boundary the model chose, once for how sure it was of it. Widened to
        # float32 first — the weights are quantized and the logits come back as
        # bfloat16, which is enough to pick an argmax and not enough to report
        # a probability with.
        probabilities = mx.softmax(logits.astype(mx.float32), axis=-1)
        predicted = mx.argmax(logits, axis=-1)

        flat_input = np.array(input_ids[0] if input_ids.ndim > 1 else input_ids)
        flat_predicted = np.array(predicted[0] if predicted.ndim > 1 else predicted)
        flat_probabilities = np.array(probabilities[0] if probabilities.ndim > 1 else probabilities)

        positions = np.flatnonzero(flat_input == model.config.timestamp_token_id)
        if positions.size != 2 * len(words):
            # Two reserved positions per word is the shape this model was
            # trained on. Anything else means the prompt and the word list
            # disagree, and placing words against it would be invention.
            raise AlignmentImpossible(
                f"{positions.size} timestamp positions for {len(words)} words"
            )
        chosen = flat_predicted[positions]
        confidence = flat_probabilities[positions, chosen]
        milliseconds = model.aligner_processor.fix_timestamp(
            chosen * model.config.timestamp_segment_time
        )

        placed: list[AlignedWord] = []
        for index, word in enumerate(words):
            placed.append(
                AlignedWord(
                    text=str(word),
                    start_ms=int(milliseconds[index * 2]),
                    end_ms=int(milliseconds[index * 2 + 1]),
                    scores=(
                        float(confidence[index * 2]),
                        float(confidence[index * 2 + 1]),
                    ),
                )
            )
        return placed


__all__ = ["AlignedWord", "AlignmentImpossible", "Qwen3Aligner"]
