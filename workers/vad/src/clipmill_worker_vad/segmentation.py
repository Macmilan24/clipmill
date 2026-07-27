"""Turning a stream of per-window speech probabilities into speech segments.

Kept separate from the model on purpose. The neural network answers one narrow
question — "does this 32 ms window sound like speech?" — and every decision
that matters downstream is made here: where a segment starts, how long a pause
has to be before it ends one, how much room to leave so a decoder is not handed
a word already in progress. Those are the parameters an operator tunes and the
gate asserts against, and they are worth being able to test without loading
400 megabytes of weights.
"""

from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass

from clipmill_worker_sdk.confidence import distribution

# Silero's own asymmetry, kept: a window has to be clearly speech to open a
# segment and clearly not speech to close one. A single threshold makes the
# boundary chatter across a breath, producing dozens of one-window segments
# where a listener hears one sentence.
NEGATIVE_THRESHOLD_MARGIN = 0.15
# Below this the negative threshold stops meaning anything.
MINIMUM_NEGATIVE_THRESHOLD = 0.01


@dataclass(frozen=True, slots=True)
class SegmentationParameters:
    """The decision parameters, in samples at the audio's own rate."""

    threshold: float
    window_samples: int
    min_speech_samples: int
    min_silence_samples: int
    speech_pad_samples: int

    @property
    def negative_threshold(self) -> float:
        return max(self.threshold - NEGATIVE_THRESHOLD_MARGIN, MINIMUM_NEGATIVE_THRESHOLD)


@dataclass(frozen=True, slots=True)
class SpeechSpan:
    """A speech region in sample indices, with the confidence behind it."""

    start_sample: int
    end_sample: int
    p50: float
    p10: float


def segment(
    probabilities: Sequence[float],
    parameters: SegmentationParameters,
    total_samples: int,
) -> list[SpeechSpan]:
    """Speech spans, ordered, non-overlapping, and clamped to the recording.

    Two properties this must hold, because later stages assume them without
    checking: the spans are in order and they do not touch. Padding is what
    threatens both — two segments separated by exactly the minimum silence,
    each padded outward, would otherwise overlap and a decoder would transcribe
    the shared audio twice.
    """

    if parameters.window_samples <= 0:
        raise ValueError("window size must be positive")
    negative = parameters.negative_threshold

    raw: list[tuple[int, int, list[float]]] = []
    start: int | None = None
    scores: list[float] = []
    silence_started: int | None = None

    for index, probability in enumerate(probabilities):
        sample = index * parameters.window_samples
        if probability >= parameters.threshold:
            silence_started = None
            if start is None:
                start = sample
                scores = []
            scores.append(probability)
            continue
        if start is None:
            continue
        # In a segment, and this window is not speech.
        if probability >= negative:
            # Ambiguous: neither opens nor closes. Keep going, and keep the
            # score, so a segment full of hesitant windows reports as one.
            scores.append(probability)
            continue
        if silence_started is None:
            silence_started = sample
        if sample + parameters.window_samples - silence_started >= parameters.min_silence_samples:
            raw.append((start, silence_started, scores))
            start = None
            scores = []
            silence_started = None

    if start is not None:
        scored_samples = len(probabilities) * parameters.window_samples
        end = silence_started if silence_started is not None else scored_samples
        raw.append((start, min(end, total_samples), scores))

    spans: list[SpeechSpan] = []
    for start_sample, end_sample, window_scores in raw:
        if end_sample - start_sample < parameters.min_speech_samples:
            continue
        p50, p10 = distribution(window_scores)
        spans.append(
            SpeechSpan(
                start_sample=start_sample,
                end_sample=min(end_sample, total_samples),
                p50=p50,
                p10=p10,
            )
        )

    return _pad(spans, parameters.speech_pad_samples, total_samples)


def silences(spans: Sequence[SpeechSpan], total_samples: int) -> list[tuple[int, int]]:
    """The gaps, including the leading and trailing ones.

    Derived here rather than left to the consumer because these are the edges
    the boundary lattice is allowed to cut on, and a consumer that forgot the
    trailing gap would refuse to end a clip at the end of the recording.
    """

    gaps: list[tuple[int, int]] = []
    cursor = 0
    for span in spans:
        if span.start_sample > cursor:
            gaps.append((cursor, span.start_sample))
        cursor = max(cursor, span.end_sample)
    if cursor < total_samples:
        gaps.append((cursor, total_samples))
    return gaps


def _pad(spans: list[SpeechSpan], padding: int, total_samples: int) -> list[SpeechSpan]:
    if padding <= 0 or not spans:
        return spans
    padded: list[SpeechSpan] = []
    for index, span in enumerate(spans):
        start = max(span.start_sample - padding, 0)
        end = min(span.end_sample + padding, total_samples)
        if padded and start < padded[-1].end_sample:
            # Split the gap between them rather than letting either win: both
            # sides asked for the same audio and neither has a better claim.
            previous = padded[-1]
            boundary = (spans[index - 1].end_sample + span.start_sample) // 2
            boundary = max(boundary, previous.start_sample + 1)
            padded[-1] = SpeechSpan(previous.start_sample, boundary, previous.p50, previous.p10)
            start = max(start, boundary)
        padded.append(SpeechSpan(start, end, span.p50, span.p10))
    return padded


__all__ = [
    "NEGATIVE_THRESHOLD_MARGIN",
    "SegmentationParameters",
    "SpeechSpan",
    "segment",
    "silences",
]
