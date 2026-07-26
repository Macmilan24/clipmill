"""Exact conversion between audio samples and ClipMill's timebase.

Every interval in the system is an integer count of 1/90000-second ticks
(decision D06). Float seconds are not an intermediate representation here:
they are how two stages come to disagree about where a word ended, which is
the one disagreement a word-snapped editor cannot survive.

The conversions below are integer-only and state their rounding, so a caller
choosing a segment start and a caller choosing the previous segment's end
land on the same tick rather than a tick apart.
"""

from __future__ import annotations

TICKS_PER_SECOND = 90_000


def samples_to_ticks(samples: int, sample_rate: int) -> int:
    """Floor: the tick the sample falls within.

    Exact at 16 kHz, where the ratio is 45/8, whenever the sample index is a
    multiple of eight — which is every window boundary the speech chain uses.
    """

    if sample_rate <= 0:
        raise ValueError("sample rate must be positive")
    if samples < 0:
        raise ValueError("sample index must not be negative")
    return samples * TICKS_PER_SECOND // sample_rate


def samples_to_ticks_ceil(samples: int, sample_rate: int) -> int:
    """Ceiling, for the exclusive end of a span.

    A segment that ends mid-tick must be reported as covering that tick, or a
    consumer computing durations from the boundaries loses the tail sample.
    """

    if sample_rate <= 0:
        raise ValueError("sample rate must be positive")
    if samples < 0:
        raise ValueError("sample index must not be negative")
    return -(-samples * TICKS_PER_SECOND // sample_rate)


def ticks_to_samples(ticks: int, sample_rate: int) -> int:
    """Floor: the first sample at or after this tick's start."""

    if sample_rate <= 0:
        raise ValueError("sample rate must be positive")
    if ticks < 0:
        raise ValueError("tick must not be negative")
    return ticks * sample_rate // TICKS_PER_SECOND


def seconds_to_ticks(seconds: float) -> int:
    """For model outputs that are only available as float seconds.

    Deliberately the only door float seconds may come through, and it closes
    behind them: the result is an integer tick, and nothing downstream ever
    sees the float again.
    """

    return round(seconds * TICKS_PER_SECOND)


__all__ = [
    "TICKS_PER_SECOND",
    "samples_to_ticks",
    "samples_to_ticks_ceil",
    "seconds_to_ticks",
    "ticks_to_samples",
]
