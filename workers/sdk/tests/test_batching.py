"""What the recognizer gets handed, tested without handing it anything.

The cases worth checking are the ones no fixture recording contains: a speech
run longer than the decoder's context, an empty segment list, a segment that
ends exactly on the limit. Each is a decision about what a decoder is asked,
and getting one wrong shows up as silently truncated text.
"""

from __future__ import annotations

from itertools import pairwise

import pytest
from clipmill_worker_sdk.batching import (
    MAX_WINDOW_SECONDS,
    decode_windows,
)

RATE = 16_000


def test_each_speech_segment_becomes_one_decode_window() -> None:
    windows = decode_windows([(0, RATE), (3 * RATE, 5 * RATE)], RATE)
    assert [(w.vad_segment_index, w.start_sample, w.end_sample) for w in windows] == [
        (0, 0, RATE),
        (1, 3 * RATE, 5 * RATE),
    ]
    assert not any(window.split for window in windows)


def test_silence_between_segments_is_never_decoded() -> None:
    """Not only a throughput argument.

    A recognizer asked what was said during four seconds of room tone will
    frequently answer, and the answer will read like speech.
    """

    windows = decode_windows([(0, RATE), (10 * RATE, 11 * RATE)], RATE)
    decoded = sum(window.sample_count for window in windows)
    assert decoded == 2 * RATE, "the nine-second gap is not handed to anyone"


def test_a_segment_longer_than_the_decoder_context_is_split() -> None:
    """Handing whisper more than thirty seconds silently discards the tail."""

    total = (MAX_WINDOW_SECONDS * 3 + 5) * RATE
    windows = decode_windows([(0, total)], RATE)
    assert len(windows) == 4
    assert all(window.split for window in windows)
    assert all(window.sample_count <= MAX_WINDOW_SECONDS * RATE for window in windows)
    # Split, not dropped: every sample of speech still reaches a decoder.
    assert windows[0].start_sample == 0
    assert windows[-1].end_sample == total
    for earlier, later in pairwise(windows):
        assert earlier.end_sample == later.start_sample


def test_a_split_window_says_it_was_split() -> None:
    """The boundary may land inside a word, and a consumer deserves to know."""

    exact = decode_windows([(0, MAX_WINDOW_SECONDS * RATE)], RATE)
    assert len(exact) == 1
    assert not exact[0].split

    over = decode_windows([(0, MAX_WINDOW_SECONDS * RATE + 1)], RATE)
    assert len(over) == 2
    assert all(window.split for window in over)


def test_every_split_window_keeps_its_speech_segment_index() -> None:
    """Text from a split segment must still be attributable to the utterance
    it came from, or alignment cannot put it back together."""

    # The second segment is exactly two windows long, so it splits into two —
    # both still belonging to the utterance they came from.
    windows = decode_windows([(0, RATE), (RATE, (2 * MAX_WINDOW_SECONDS + 1) * RATE)], RATE)
    assert {window.vad_segment_index for window in windows} == {0, 1}
    assert [w.vad_segment_index for w in windows if w.split] == [1, 1]


def test_an_empty_or_inverted_segment_produces_no_window() -> None:
    assert decode_windows([], RATE) == []
    assert decode_windows([(RATE, RATE), (5 * RATE, 4 * RATE)], RATE) == []


def test_a_recording_with_no_speech_asks_the_recognizer_nothing() -> None:
    assert decode_windows([], RATE) == []


@pytest.mark.parametrize(("rate", "limit"), [(0, MAX_WINDOW_SECONDS), (RATE, 0)])
def test_degenerate_limits_are_refused_rather_than_looping_forever(rate: int, limit: int) -> None:
    with pytest.raises(ValueError, match="positive"):
        decode_windows([(0, RATE)], rate, limit)
