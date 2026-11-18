"""Measure sustained changes in voiced pitch and RMS level on the 16 kHz rendition."""

from __future__ import annotations

from collections import deque
from collections.abc import Callable, Iterable, Iterator
from itertools import chain, groupby
from math import log2
from statistics import median

import numpy as np
from clipmill_worker_sdk.gen.schemas.speech_vad import SpeechVad
from clipmill_worker_sdk.ticks import TICKS_PER_SECOND, ticks_to_samples

from .events import Delivery, Event

RATE = 16_000
FRAME = 640  # 40 ms, almost three periods at the bottom of the pitch range
HOP = 320
BATCH = 64
BASELINE = 2 * RATE
MIN_VOICED_FRAMES = 25  # half a second of preceding periodic speech
MIN_ACCENT_FRAMES = 5

Voice = tuple[float, float, float]
Marked = tuple[int, float, float, float, float, float]


def speech_ranges(vad: SpeechVad) -> Iterator[tuple[int, int]]:
    """Return sample spans wholly inside covered speech, excluding any invalid evidence."""
    if not vad.coverage.analyzed:
        return
    invalid = sorted(vad.invalid_regions, key=lambda region: region.start_ticks)
    first_invalid = 0
    for segment in vad.segments:
        start = max(segment.start_ticks, vad.coverage.start_ticks)
        end = min(segment.end_ticks, vad.coverage.end_ticks)
        while first_invalid < len(invalid) and invalid[first_invalid].end_ticks <= start:
            first_invalid += 1
        at = first_invalid
        while at < len(invalid) and invalid[at].start_ticks < end:
            gap = invalid[at]
            if start < gap.start_ticks:
                yield -(-start * RATE // TICKS_PER_SECOND), ticks_to_samples(gap.start_ticks, RATE)
            start = max(start, gap.end_ticks)
            at += 1
        if start < end:
            yield -(-start * RATE // TICKS_PER_SECOND), ticks_to_samples(end, RATE)


def _frames(
    samples: np.ndarray, start: int, end: int, check: Callable[[], None]
) -> Iterator[tuple[int, Voice | None]]:
    """Analyze at most 64 overlapping frames per FFT, independent of recording length."""
    last = end - FRAME + 1
    for base in range(start, last, BATCH * HOP):
        check()
        chunk = samples[base : min(base + (BATCH - 1) * HOP + FRAME, end)]
        frames = np.lib.stride_tricks.sliding_window_view(chunk, FRAME)[::HOP].astype(np.float64)
        frames -= frames.mean(axis=1, keepdims=True)
        levels = 10 * np.log10(np.maximum(np.mean(frames * frames, axis=1), 1e-12))
        # Four-sample averaging leaves the 70 to 400 Hz fundamental below Nyquist.
        low = frames.reshape(-1, FRAME // 4, 4).mean(axis=2)
        spectrum = np.fft.rfft(low, n=512, axis=1)
        autocorr = np.fft.irfft(spectrum.real**2 + spectrum.imag**2, n=512, axis=1)
        lags = np.arange(9, 59)
        power = np.pad(np.cumsum(low * low, axis=1), ((0, 0), (1, 0)))
        overlap_power = power[:, low.shape[1] - lags] * (power[:, -1:] - power[:, lags])
        correlation = autocorr[:, lags] / np.sqrt(np.maximum(overlap_power, 1e-20))
        # Choose the first comparable local peak: a pure tone also correlates
        # at twice its period, which would otherwise halve the reported pitch.
        middle = correlation[:, 1:-1]
        peaks = (middle > correlation[:, :-2]) & (middle >= correlation[:, 2:])
        best = np.max(np.where(peaks, middle, 0), axis=1, keepdims=True)
        strong = peaks & (middle >= np.maximum(0.72, 0.92 * best))
        chosen = np.argmax(strong, axis=1) + 1
        rows = np.arange(len(frames))
        centre = correlation[rows, chosen]
        left = correlation[rows, chosen - 1]
        right = correlation[rows, chosen + 1]
        curvature = left - 2 * centre + right
        adjustment = np.divide(
            0.5 * (left - right), curvature, out=np.zeros_like(centre), where=curvature < -1e-9
        )
        periods = lags[chosen] + np.clip(adjustment, -0.5, 0.5)
        valid = strong.any(axis=1) & (levels >= -52)
        for row, voiced in enumerate(valid):
            yield (
                base + row * HOP,
                (float(RATE / 4 / periods[row]), float(levels[row]), min(1.0, float(centre[row])))
                if voiced
                else None,
            )


def _prominent(
    samples: np.ndarray, regions: Iterable[tuple[int, int]], check: Callable[[], None]
) -> Iterator[Marked | None]:
    history: deque[tuple[int, float, float]] = deque()
    for start, end in regions:
        # A run cannot span a VAD boundary. History expires in source time,
        # so an isolated sound after silence cannot supply its own baseline.
        yield None
        for index, voice in _frames(samples, max(0, start), min(end, len(samples)), check):
            while history and history[0][0] < index - BASELINE:
                history.popleft()
            marked = None
            if voice is not None:
                pitch, level, strength = voice
                if len(history) >= MIN_VOICED_FRAMES:
                    pitch_change = 12 * log2(pitch / median(h[1] for h in history))
                    level_change = level - median(h[2] for h in history)
                    if (
                        (pitch_change >= 4 and level_change >= -2)
                        or (level_change >= 6 and pitch_change >= -1.5)
                        or (pitch_change >= 3 and level_change >= 3)
                    ):
                        marked = (index, pitch, level, pitch_change, level_change, strength)
                history.append((index, pitch, level))
            yield marked


def emphasis(
    samples: np.ndarray,
    regions: Iterable[tuple[int, int]],
    check: Callable[[], None],
) -> list[Event]:
    """Emit measured accents inside verified speech; levels are dB relative to full scale."""
    events = []
    for active, run in groupby(
        _prominent(samples, regions, check), lambda frame: frame is not None
    ):
        if not active:
            continue
        first = next(run)
        count, peak = 0, 0.0
        totals = np.zeros(5)
        for frame in chain((first,), run):
            index, pitch, level, semitones, decibels, strength = frame
            end = index + (FRAME + HOP) // 2
            totals += (pitch, level, semitones, decibels, strength)
            count += 1
            peak = max(peak, strength)
        if count >= MIN_ACCENT_FRAMES:
            pitch, level, semitones, decibels, strength = (float(x) for x in totals / count)
            events.append(
                Event(
                    "emphasis",
                    first[0] + (FRAME - HOP) // 2,
                    end,
                    round(peak, 4),
                    round(strength, 4),
                    Delivery(
                        round(pitch, 1), round(level, 1), round(semitones, 1), round(decibels, 1)
                    ),
                )
            )
    return events
