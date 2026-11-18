"""Runs of speech long enough to mean something, from the index's sentences.

A recogniser splits speech wherever it hears a break, and a one-word
"sentence" embeds as noise. A passage joins consecutive sentences until it
has enough words to stand on its own, stops before it grows past what a
search hit should cover, and never bridges a long silence.
"""

from __future__ import annotations

from dataclasses import dataclass

MIN_WORDS = 20
MAX_WORDS = 80
# A silence this long ends a passage whatever its length.
BREAK_TICKS = 2 * 90_000


@dataclass(frozen=True)
class Passage:
    first: int
    last: int
    text: str


def passages(sentences: list[dict]) -> list[Passage]:
    """Passages in recording order, every sentence in exactly one."""

    found: list[Passage] = []
    run: list[dict] = []
    words = 0

    def close() -> None:
        nonlocal run, words
        if run:
            text = " ".join(s["text"].strip() for s in run if s["text"].strip())
            found.append(Passage(run[0]["index"], run[-1]["index"], text))
        run, words = [], 0

    for sentence in sentences:
        count = len(sentence["text"].split())
        if run and (
            words >= MIN_WORDS
            or words + count > MAX_WORDS
            or sentence["start_ticks"] - run[-1]["end_ticks"] >= BREAK_TICKS
        ):
            close()
        run.append(sentence)
        words += count
    close()
    return [passage for passage in found if passage.text]
