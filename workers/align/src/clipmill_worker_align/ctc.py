"""Forced alignment: fitting known text to the audio that produced it.

The recognizer already decided *what* was said. This decides *when*, and it is
a separate stage for a reason the book is explicit about: word timing taken
from a decoder's token positions is timing nobody measured. Every word-snapped
trim, every caption cue, and every boundary the optimizer refuses to cut
inside ultimately rests on the frames chosen here.

The algorithm is CTC Viterbi over the standard blank-extended label sequence.
Given per-frame log-probabilities over the model's alphabet and the characters
that were said, it finds the single most likely assignment of frames to
characters — not the most likely transcription, which is a different and
easier question the recognizer already answered.

Pure NumPy and no model, so the parts that go wrong quietly — a path that
cannot exist, a token squeezed to zero frames, text longer than the audio —
are testable against emissions written by hand.
"""

from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass

import numpy as np

NEGATIVE_INFINITY = -1e30


class AlignmentImpossible(RuntimeError):
    """No path through the audio can spell this text."""


@dataclass(frozen=True, slots=True)
class TokenSpan:
    """Where one character landed, and how confident the path was there."""

    token_index: int
    label: int
    start_frame: int
    end_frame: int
    scores: tuple[float, ...]
    """Per-frame probabilities along the chosen path, kept so a word's
    confidence can be a distribution rather than one averaged number."""


def log_softmax(logits: np.ndarray) -> np.ndarray:
    """Row-wise, in a form that does not overflow on confident frames."""

    shifted = logits - logits.max(axis=-1, keepdims=True)
    return shifted - np.log(np.exp(shifted).sum(axis=-1, keepdims=True))


def forced_align(
    emissions: np.ndarray,
    labels: Sequence[int],
    blank_id: int = 0,
) -> list[TokenSpan]:
    """Assign every frame to a character, and report each character's span.

    `emissions` is `[frames, alphabet]` log-probabilities; `labels` is the text
    as label ids, already normalized into the model's alphabet.
    """

    if emissions.ndim != 2:
        raise ValueError("emissions must be a frames-by-alphabet matrix")
    if not labels:
        raise AlignmentImpossible("there is no text to align")
    frames = emissions.shape[0]

    # The blank-extended sequence: a blank before, after, and between every
    # character. The blanks between are what let a repeated character ("LL")
    # be two characters rather than one held for longer.
    extended = np.zeros(2 * len(labels) + 1, dtype=np.int64)
    extended[1::2] = labels
    extended[0::2] = blank_id
    states = extended.shape[0]

    # A path must visit every character, and may skip a blank only where the
    # characters on either side differ. The shortest legal path is therefore
    # the character count plus one blank for every repeated pair.
    repeats = (
        int(np.sum(np.asarray(labels[1:]) == np.asarray(labels[:-1]))) if len(labels) > 1 else 0
    )
    if frames < len(labels) + repeats:
        raise AlignmentImpossible(f"{frames} frames cannot spell {len(labels)} characters")

    # A skip is legal into state s only when s is a character and the previous
    # character differs; otherwise the two would collapse into one.
    skippable = np.zeros(states, dtype=bool)
    skippable[2:] = (extended[2:] != blank_id) & (extended[2:] != extended[:-2])

    alpha = np.full((frames, states), NEGATIVE_INFINITY, dtype=np.float64)
    backpointer = np.zeros((frames, states), dtype=np.int8)
    # A path may open on the leading blank or on the first character.
    alpha[0, 0] = emissions[0, extended[0]]
    alpha[0, 1] = emissions[0, extended[1]]

    columns = np.arange(states)
    for frame in range(1, frames):
        stay = alpha[frame - 1]
        advance = np.full(states, NEGATIVE_INFINITY)
        advance[1:] = alpha[frame - 1, :-1]
        skip = np.full(states, NEGATIVE_INFINITY)
        skip[2:] = np.where(skippable[2:], alpha[frame - 1, :-2], NEGATIVE_INFINITY)
        choices = np.stack((stay, advance, skip))
        taken = choices.argmax(axis=0)
        alpha[frame] = choices[taken, columns] + emissions[frame, extended]
        backpointer[frame] = taken

    # A complete path ends on the final character or on the blank after it.
    tail = int(np.argmax(alpha[frames - 1, states - 2 :])) + states - 2
    if alpha[frames - 1, tail] <= NEGATIVE_INFINITY:
        raise AlignmentImpossible("no path through the audio spells this text")

    path = np.zeros(frames, dtype=np.int64)
    state = tail
    for frame in range(frames - 1, -1, -1):
        path[frame] = state
        if frame > 0:
            state -= int(backpointer[frame, state])

    spans: list[TokenSpan] = []
    for index in range(len(labels)):
        occupied = np.flatnonzero(path == 2 * index + 1)
        if occupied.size == 0:
            # Only reachable if the path is inconsistent with the extension,
            # which would be a bug here rather than a property of the audio.
            raise AlignmentImpossible(f"character {index} was assigned no frame")
        start, end = int(occupied[0]), int(occupied[-1]) + 1
        scores = np.exp(emissions[occupied, labels[index]])
        spans.append(
            TokenSpan(
                token_index=index,
                label=int(labels[index]),
                start_frame=start,
                end_frame=end,
                scores=tuple(float(score) for score in scores),
            )
        )
    return spans


__all__ = ["AlignmentImpossible", "TokenSpan", "forced_align", "log_softmax"]
