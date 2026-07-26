"""Forced alignment, tested against emissions written by hand.

Every word-snapped edit in the product rests on the frames this picks, and the
ways it can go quietly wrong are not things a fixture recording contains: text
longer than the audio, a repeated letter that must not collapse, a path that
does not exist. Hand-built emissions make each of those a two-line test.
"""

from __future__ import annotations

from itertools import pairwise

import numpy as np
import pytest
from clipmill_worker_align.ctc import (
    AlignmentImpossible,
    forced_align,
    log_softmax,
)

BLANK = 0
A, B, C = 1, 2, 3
ALPHABET = 4


def emissions(pattern: list[int], confidence: float = 0.9) -> np.ndarray:
    """One row per frame, `confidence` on the named label and the rest spread."""

    matrix = np.full((len(pattern), ALPHABET), (1 - confidence) / (ALPHABET - 1))
    for frame, label in enumerate(pattern):
        matrix[frame] = (1 - confidence) / (ALPHABET - 1)
        matrix[frame, label] = confidence
    return np.log(matrix)


def test_each_character_gets_the_frames_it_was_spoken_in() -> None:
    spans = forced_align(emissions([BLANK, A, A, BLANK, B, BLANK]), [A, B])
    assert [(span.start_frame, span.end_frame) for span in spans] == [(1, 3), (4, 5)]


def test_spans_are_ordered_and_never_overlap() -> None:
    """A trim that snaps to a word boundary needs a total order to snap within."""

    spans = forced_align(emissions([A, B, C, A, B, C]), [A, B, C, A, B, C])
    for earlier, later in pairwise(spans):
        assert earlier.end_frame <= later.start_frame
        assert earlier.start_frame < earlier.end_frame


def test_a_repeated_character_is_two_characters_not_one_held_longer() -> None:
    """The reason the label sequence is blank-extended at all.

    Without a blank between them, the two Ls of "HELLO" are indistinguishable
    from one L spoken slowly, and the second would be assigned no frames.
    """

    spans = forced_align(emissions([A, A, BLANK, A, A]), [A, A])
    assert len(spans) == 2
    assert spans[0].end_frame <= spans[1].start_frame
    assert spans[0].start_frame < spans[0].end_frame
    assert spans[1].start_frame < spans[1].end_frame


def test_text_longer_than_the_audio_is_refused_rather_than_squeezed() -> None:
    """Three frames cannot contain four characters.

    The alternative is a span of zero frames, which would reach the transcript
    as a word that starts and ends at the same tick — and would then be
    published as measured timing.
    """

    with pytest.raises(AlignmentImpossible, match="cannot spell"):
        forced_align(emissions([A, B, C]), [A, B, C, A])


def test_repeated_characters_count_against_the_frame_budget() -> None:
    """ "AA" needs three frames, not two: the blank between them is mandatory."""

    with pytest.raises(AlignmentImpossible, match="cannot spell"):
        forced_align(emissions([A, A]), [A, A])
    assert len(forced_align(emissions([A, BLANK, A]), [A, A])) == 2


def test_empty_text_is_refused() -> None:
    with pytest.raises(AlignmentImpossible, match="no text"):
        forced_align(emissions([A, B]), [])


def test_alignment_survives_audio_that_does_not_match_the_text() -> None:
    """The recognizer said one thing; the audio sounds like another.

    Forced alignment always produces a path — that is what "forced" means — so
    the honest signal is not a failure but a low score. Callers decide what to
    do with it, and this asserts the score actually collapses.
    """

    matching = forced_align(emissions([A, BLANK, B]), [A, B])
    mismatched = forced_align(emissions([C, C, C]), [A, B])
    best = max(max(span.scores) for span in mismatched)
    assert best < min(min(span.scores) for span in matching)


def test_every_span_carries_the_scores_along_its_own_frames() -> None:
    """A word's confidence has to be a distribution, so the per-frame scores
    are kept rather than averaged away here."""

    spans = forced_align(emissions([A, A, A, BLANK, B]), [A, B])
    assert len(spans[0].scores) == spans[0].end_frame - spans[0].start_frame
    assert all(0.0 <= score <= 1.0 for span in spans for score in span.scores)


def test_log_softmax_normalizes_each_frame() -> None:
    logits = np.array([[1000.0, 999.0, 0.0, -5.0], [0.1, 0.2, 0.3, 0.4]])
    normalized = log_softmax(logits)
    # No overflow on a confident frame, and every row is a distribution.
    assert np.all(np.isfinite(normalized))
    assert np.allclose(np.exp(normalized).sum(axis=1), 1.0)


def test_emissions_must_be_a_matrix() -> None:
    with pytest.raises(ValueError, match="frames-by-alphabet"):
        forced_align(np.zeros(4), [A])
