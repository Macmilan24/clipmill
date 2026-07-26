"""Segmentation, tested without the weights.

The model answers "does this window sound like speech?" and every decision
that reaches an artifact is made after that answer. So these use hand-written
probability sequences: what they assert is the behaviour an operator tunes
and a later stage depends on, not whether silero is any good at its job.
"""

from __future__ import annotations

import pytest
from clipmill_worker_vad.segmentation import (
    NEGATIVE_THRESHOLD_MARGIN,
    SegmentationParameters,
    SpeechSpan,
    segment,
    silences,
)

WINDOW = 512


def parameters(
    *,
    threshold: float = 0.5,
    min_speech: int = 0,
    min_silence: int = 3 * WINDOW,
    padding: int = 0,
) -> SegmentationParameters:
    return SegmentationParameters(
        threshold=threshold,
        window_samples=WINDOW,
        min_speech_samples=min_speech,
        min_silence_samples=min_silence,
        speech_pad_samples=padding,
    )


def scores(pattern: str) -> list[float]:
    """'..###..' as probabilities: '#' speech, '.' silence, '~' ambiguous."""

    return [{"#": 0.9, ".": 0.05, "~": 0.42}[character] for character in pattern]


def test_a_run_of_speech_windows_becomes_one_segment() -> None:
    probabilities = scores("..####....")
    spans = segment(probabilities, parameters(), len(probabilities) * WINDOW)
    assert [(span.start_sample, span.end_sample) for span in spans] == [(2 * WINDOW, 6 * WINDOW)]


def test_a_short_gap_does_not_split_an_utterance() -> None:
    """A breath is not the end of a sentence.

    Two segments where a listener hears one would put a cut inside a phrase,
    which is the failure the minimum-silence parameter exists to prevent.
    """

    probabilities = scores("###..###")
    spans = segment(probabilities, parameters(min_silence=3 * WINDOW), len(probabilities) * WINDOW)
    assert len(spans) == 1
    assert (spans[0].start_sample, spans[0].end_sample) == (0, 8 * WINDOW)


def test_a_long_gap_does_split_an_utterance() -> None:
    probabilities = scores("###....###")
    spans = segment(probabilities, parameters(min_silence=3 * WINDOW), len(probabilities) * WINDOW)
    assert [(span.start_sample, span.end_sample) for span in spans] == [
        (0, 3 * WINDOW),
        (7 * WINDOW, 10 * WINDOW),
    ]


def test_an_ambiguous_window_neither_opens_nor_closes_a_segment() -> None:
    """The asymmetric threshold, which is the whole reason there are two.

    A window at 0.42 is below the 0.5 that would start a segment but above the
    0.35 that would end one. Inside an utterance it is speech continuing; on
    its own it is not speech starting.
    """

    assert 0.5 - NEGATIVE_THRESHOLD_MARGIN < 0.42 < 0.5
    inside = segment(scores("##~##"), parameters(min_silence=WINDOW), 5 * WINDOW)
    assert len(inside) == 1, "an ambiguous window inside speech does not end it"

    outside = segment(scores("..~.."), parameters(min_silence=WINDOW), 5 * WINDOW)
    assert outside == [], "an ambiguous window on its own does not start speech"


def test_segments_shorter_than_the_minimum_are_dropped() -> None:
    probabilities = scores("#....######")
    spans = segment(
        probabilities,
        parameters(min_speech=2 * WINDOW, min_silence=3 * WINDOW),
        len(probabilities) * WINDOW,
    )
    assert [(span.start_sample, span.end_sample) for span in spans] == [(5 * WINDOW, 11 * WINDOW)]


def test_speech_running_to_the_end_of_the_recording_is_closed_at_the_end() -> None:
    """Nothing tells the loop the recording ended except the recording ending.

    A segment left open would either be dropped or run past the last sample,
    and either way the final word becomes unalignable.
    """

    total = 5 * WINDOW - 100  # a partial final window, as real audio has
    spans = segment(scores("..###"), parameters(), total)
    assert len(spans) == 1
    assert spans[0].end_sample == total


def test_padding_never_makes_two_segments_overlap() -> None:
    """Both sides ask for the same audio; neither may have it twice.

    Overlapping segments would be decoded twice, so the same sentence would
    appear twice in the transcript with two different sets of word timings.
    """

    probabilities = scores("###....###")
    spans = segment(
        probabilities,
        parameters(min_silence=3 * WINDOW, padding=3 * WINDOW),
        len(probabilities) * WINDOW,
    )
    assert len(spans) == 2
    assert spans[0].end_sample <= spans[1].start_sample
    for span in spans:
        assert span.start_sample >= 0
        assert span.end_sample <= len(probabilities) * WINDOW
        assert span.start_sample < span.end_sample


def test_padding_is_clamped_to_the_recording() -> None:
    probabilities = scores("#####")
    spans = segment(probabilities, parameters(padding=10 * WINDOW), 5 * WINDOW)
    assert (spans[0].start_sample, spans[0].end_sample) == (0, 5 * WINDOW)


def test_confidence_quantiles_come_from_scores_the_model_actually_produced() -> None:
    """p10 is a reading, not an average of two readings.

    Downstream stages widen uncertainty using p10; interpolating would let it
    report a confidence no window ever had.
    """

    probabilities = [0.95, 0.9, 0.6, 0.88, 0.92]
    spans = segment(probabilities, parameters(), len(probabilities) * WINDOW)
    assert len(spans) == 1
    assert spans[0].p10 in probabilities
    assert spans[0].p50 in probabilities
    assert spans[0].p10 <= spans[0].p50


def test_silences_cover_the_recording_including_both_ends() -> None:
    """The lattice cuts on these edges; a missing one is a cut nobody can make."""

    spans = [SpeechSpan(1000, 2000, 0.9, 0.8), SpeechSpan(3000, 4000, 0.9, 0.8)]
    assert silences(spans, 5000) == [(0, 1000), (2000, 3000), (4000, 5000)]


def test_a_recording_with_no_speech_is_one_long_silence() -> None:
    assert silences([], 5000) == [(0, 5000)]


def test_speech_filling_the_recording_leaves_no_silence() -> None:
    assert silences([SpeechSpan(0, 5000, 0.9, 0.8)], 5000) == []


def test_a_zero_window_is_refused_rather_than_dividing_by_it() -> None:
    with pytest.raises(ValueError, match="window"):
        segment([0.9], SegmentationParameters(0.5, 0, 0, 0, 0), 512)
