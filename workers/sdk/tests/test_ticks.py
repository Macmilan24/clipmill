"""Sample-to-tick conversion, which every word boundary in the product rests on.

At 16 kHz the ratio to the 1/90000 timebase is 45/8 — not an integer, so the
rounding is a decision rather than an accident. These pin which decision, and
they pin the property that actually matters: a span's end and the next span's
start must not be computed by two rules that disagree by a tick, because a
word-snapped trim would then either overlap or leave a gap.
"""

from __future__ import annotations

import pytest
from clipmill_worker_sdk.ticks import (
    TICKS_PER_SECOND,
    samples_to_ticks,
    samples_to_ticks_ceil,
    seconds_to_ticks,
    ticks_to_samples,
)

RATE = 16_000


def test_a_second_of_audio_is_the_timebase() -> None:
    assert samples_to_ticks(RATE, RATE) == TICKS_PER_SECOND
    assert ticks_to_samples(TICKS_PER_SECOND, RATE) == RATE


def test_conversion_is_exact_on_the_windows_the_speech_chain_uses() -> None:
    """The 45/8 ratio is exact whenever the sample index is a multiple of 8,
    and every silero window boundary is a multiple of 512."""

    for window in range(0, 40):
        samples = window * 512
        assert samples * 45 % 8 == 0
        assert samples_to_ticks(samples, RATE) == samples_to_ticks_ceil(samples, RATE)


def test_floor_and_ceiling_differ_only_off_the_grid() -> None:
    # 16000 * 45 / 8: one sample is 5.625 ticks, so an odd sample index is
    # genuinely between two ticks.
    assert samples_to_ticks(1, RATE) == 5
    assert samples_to_ticks_ceil(1, RATE) == 6


def test_a_span_end_covers_the_tick_its_last_sample_falls_in() -> None:
    """Floor the start, ceil the end.

    The alternative loses the tail sample: a consumer computing a duration
    from the reported boundaries would come up short, and repeated across a
    recording that is how a transcript drifts away from its audio.
    """

    start, end = 1000, 1001
    assert samples_to_ticks_ceil(end, RATE) > samples_to_ticks(start, RATE)


def test_round_tripping_a_tick_never_moves_past_it() -> None:
    for ticks in range(0, 100_000, 337):
        samples = ticks_to_samples(ticks, RATE)
        assert samples_to_ticks(samples, RATE) <= ticks


def test_float_seconds_may_only_enter_through_one_door() -> None:
    """Some model APIs report float seconds and nothing can change that.

    What can be changed is how far the float travels: it is converted once, at
    the boundary, and the integer is what the rest of the system sees.
    """

    assert seconds_to_ticks(1.0) == TICKS_PER_SECOND
    assert seconds_to_ticks(0.5) == TICKS_PER_SECOND // 2
    assert isinstance(seconds_to_ticks(0.123456), int)


@pytest.mark.parametrize(
    ("call", "argument"),
    [
        (samples_to_ticks, -1),
        (samples_to_ticks_ceil, -1),
        (ticks_to_samples, -1),
    ],
)
def test_negative_positions_are_refused(call, argument: int) -> None:
    with pytest.raises(ValueError, match="negative"):
        call(argument, RATE)


def test_a_zero_sample_rate_is_refused_rather_than_dividing_by_it() -> None:
    with pytest.raises(ValueError, match="sample rate"):
        samples_to_ticks(1, 0)
