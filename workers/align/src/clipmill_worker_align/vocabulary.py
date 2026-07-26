"""Turning recognized text into the alphabet the acoustic model can score.

The CTC model knows thirty-two labels: uppercase English, an apostrophe, a
word delimiter, and a blank. Recognized text contains rather more than that —
punctuation, digits, casing, the occasional emoji — and every character that
is not in the alphabet has to be dealt with explicitly, because the two silent
options are both wrong. Dropping a word loses what was said; pretending it was
aligned invents a measurement.

So a word that survives normalization is aligned, and a word that does not is
reported unaligned with a reason. Assembly then carries its text with
interpolated timing and marks the span, and the boundary optimizer refuses to
cut there.
"""

from __future__ import annotations

import json
import re
from dataclasses import dataclass
from pathlib import Path

BLANK_TOKEN = "<pad>"
DELIMITER_TOKEN = "|"
# Everything the alphabet cannot represent. Digits are the common case and the
# reason this is a stated limitation rather than an oversight: spelling them
# out is language-specific ("101" is three words in one language and two in
# another), and a wrong expansion would align the wrong sounds to the wrong
# word rather than admitting it could not.
_STRIPPED = re.compile(r"[^A-Z']+")


@dataclass(frozen=True, slots=True)
class ScoreableWord:
    """One word of the reference text, and its label ids."""

    index: int
    text: str
    """As the recognizer wrote it, punctuation and all. This is what a caption
    displays; the normalized form below is only what gets scored."""
    normalized: str
    labels: tuple[int, ...]


@dataclass(frozen=True, slots=True)
class UnscoreableWord:
    index: int
    text: str
    reason: str


class Vocabulary:
    """The model's label set, read rather than assumed."""

    def __init__(self, labels: dict[str, int]) -> None:
        if BLANK_TOKEN not in labels or DELIMITER_TOKEN not in labels:
            raise ValueError("this alphabet has no blank or no word delimiter")
        self.labels = labels
        self.blank_id = labels[BLANK_TOKEN]
        self.delimiter_id = labels[DELIMITER_TOKEN]

    @classmethod
    def load(cls, path: Path) -> Vocabulary:
        return cls(json.loads(path.read_text(encoding="utf-8")))

    def normalize(self, word: str) -> str:
        return _STRIPPED.sub("", word.upper())

    def encode(self, words: list[str]) -> tuple[list[ScoreableWord], list[UnscoreableWord]]:
        """Split the reference text into what can be scored and what cannot."""

        scoreable: list[ScoreableWord] = []
        unscoreable: list[UnscoreableWord] = []
        for index, word in enumerate(words):
            normalized = self.normalize(word)
            ids = [self.labels[character] for character in normalized if character in self.labels]
            if not ids:
                unscoreable.append(
                    UnscoreableWord(
                        index=index,
                        text=word,
                        reason="out_of_vocabulary",
                    )
                )
                continue
            scoreable.append(
                ScoreableWord(
                    index=index,
                    text=word,
                    normalized=normalized,
                    labels=tuple(ids),
                )
            )
        return (scoreable, unscoreable)

    def label_sequence(self, words: list[ScoreableWord]) -> tuple[list[int], list[tuple[int, int]]]:
        """The whole utterance as labels, with each word's slice into it.

        Words are separated by the delimiter the model was trained with, which
        is what stops "a cat" and "acat" from scoring identically — and what
        gives each word an end that is not simply the next word's start.
        """

        labels: list[int] = []
        spans: list[tuple[int, int]] = []
        for position, word in enumerate(words):
            if position > 0:
                labels.append(self.delimiter_id)
            start = len(labels)
            labels.extend(word.labels)
            spans.append((start, len(labels)))
        return (labels, spans)


__all__ = [
    "BLANK_TOKEN",
    "DELIMITER_TOKEN",
    "ScoreableWord",
    "UnscoreableWord",
    "Vocabulary",
]
