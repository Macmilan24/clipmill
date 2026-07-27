"""The confidence distribution every perception stage publishes.

The observation contract asks for a distribution rather than a scalar because
downstream stages read different parts of it: ranking orders on the median and
decides whether a quote is safe to show on the low quantile. These pin the two
properties that make that possible — the numbers are real readings, and p10 is
the pessimistic one.
"""

from __future__ import annotations

from clipmill_worker_sdk.confidence import distribution, quantile


def test_the_reported_numbers_are_scores_the_model_actually_produced() -> None:
    """Nearest-rank, not interpolated.

    An interpolated median can report a confidence no window ever had, which
    is a small lie in a document whose whole job is to be trustworthy about
    how sure it is.
    """

    scores = [0.2, 0.9, 0.95, 0.99]
    p50, p10 = distribution(scores)
    assert p50 in scores
    assert p10 in scores


def test_the_low_quantile_is_the_pessimistic_one() -> None:
    p50, p10 = distribution([0.1, 0.5, 0.9, 0.95, 0.99])
    assert p10 <= p50


def test_a_bad_tenth_of_a_sentence_moves_p10_and_not_the_median() -> None:
    """The reason a segment average is not enough.

    A confident sentence whose worst tenth is guesswork looks untouched on the
    median and visibly worse on p10 — which is the signal ranking needs before
    it puts that sentence on screen as a quote.

    The threshold is a real property of a nearest-rank quantile rather than a
    weakness: p10 tracks the bottom decile, so a single bad token in twenty
    genuinely should not move it. Anything more sensitive would fire on the
    ordinary hesitation in every recording.
    """

    clean = [0.95] * 20
    ragged = [*[0.95] * 17, 0.05, 0.05, 0.05]
    assert distribution(ragged)[0] == distribution(clean)[0], "the median is unmoved"
    assert distribution(ragged)[1] < distribution(clean)[1], "p10 registers it"

    barely = [*[0.95] * 19, 0.05]
    assert distribution(barely)[1] == distribution(clean)[1], "one token in twenty does not"


def test_order_does_not_matter() -> None:
    assert distribution([0.9, 0.1, 0.5]) == distribution([0.5, 0.9, 0.1])


def test_a_single_score_is_both_quantiles() -> None:
    assert distribution([0.77]) == (0.77, 0.77)


def test_nothing_to_be_confident_about_reports_zero() -> None:
    """The caller is expected to be recording an explicit absence instead."""

    assert distribution([]) == (0.0, 0.0)
    assert quantile([], 0.5) == 0.0
