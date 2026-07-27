"""Summarizing a model's scores as the distribution the contract asks for.

Every perception output carries a confidence distribution rather than a bare
scalar (book ch. 13), because the stages downstream do different things with
it: ranking reads the median when it orders candidates and the low quantile
when it decides whether a quote is safe to put on screen. A single averaged
number cannot answer both, and averaging is also what hides the one bad token
inside an otherwise confident sentence.

Nearest-rank rather than interpolated, so every number published is a score
some model actually produced rather than the midpoint of two it did not.
"""

from __future__ import annotations

from collections.abc import Sequence

LOW_QUANTILE = 0.1


def quantile(values: Sequence[float], fraction: float) -> float:
    """Nearest-rank quantile of an unsorted sequence.

    The rank is floored after adding a half rather than handed to `round`.
    Python rounds halves to even and Rust rounds them away from zero, and the
    daemon computes this same quantile over the same numbers when it assembles
    a transcript — so the built-in would make the two languages disagree about
    a published confidence whenever a list has an even length.
    """

    if not values:
        return 0.0
    ordered = sorted(values)
    rank = max(int(fraction * (len(ordered) - 1) + 0.5), 0)
    return float(ordered[min(rank, len(ordered) - 1)])


def distribution(values: Sequence[float]) -> tuple[float, float]:
    """`(p50, p10)` — the median, and what this is worth on a bad day.

    An empty sequence reports `(0.0, 0.0)`: a stage with nothing to be
    confident about should not be publishing confidence, and the caller is
    expected to be recording an explicit absence instead.
    """

    return (quantile(values, 0.5), quantile(values, LOW_QUANTILE))


__all__ = ["LOW_QUANTILE", "distribution", "quantile"]
