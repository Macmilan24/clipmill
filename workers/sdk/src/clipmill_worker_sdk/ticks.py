"""Integer conversions between audio samples and 1/90000-second ticks.

Each conversion states its rounding so adjacent boundaries agree without an
intermediate floating-point representation."""

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


def frames_to_ticks(frame_index: int, rate_num: int, rate_den: int) -> int:
    """Floor: the tick a frame is presented at, under a rational frame rate.

    Integer-only for the same reason as everything else here. At 30000/1001 the
    per-frame interval is 3003 ticks exactly, so a video stage and an audio
    stage describing the same instant land on the same number — which they do
    not if either of them goes through 0.0333... seconds first.
    """

    if rate_num <= 0 or rate_den <= 0:
        raise ValueError("frame rate must be positive")
    if frame_index < 0:
        raise ValueError("frame index must not be negative")
    return frame_index * rate_den * TICKS_PER_SECOND // rate_num


def seconds_to_ticks(seconds: float) -> int:
    """For model outputs that are only available as float seconds.

    Deliberately the only door float seconds may come through, and it closes
    behind them: the result is an integer tick, and nothing downstream ever
    sees the float again.
    """

    return round(seconds * TICKS_PER_SECOND)


__all__ = [
    "TICKS_PER_SECOND",
    "frames_to_ticks",
    "samples_to_ticks",
    "samples_to_ticks_ceil",
    "seconds_to_ticks",
    "ticks_to_samples",
]
