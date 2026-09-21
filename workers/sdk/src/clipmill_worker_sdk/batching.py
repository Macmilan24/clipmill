"""Convert speech segments into bounded decode windows.

All recognizers share this pure batching logic so implementation benchmarks use
comparable windows. The context limit remains a parameter because it varies by
model."""

from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass

# Whisper's encoder sees a fixed 30-second window. Handing it more means the
# tail is silently discarded, so a longer speech segment has to be split.
# Staying under the limit rather than at it leaves room for the model's own
# padding without truncating a word at the boundary. A recognizer with a wider
# context passes its own.
MAX_WINDOW_SECONDS = 28


@dataclass(frozen=True, slots=True)
class DecodeWindow:
    """One call to the recognizer, and which speech segment it came from."""

    vad_segment_index: int
    start_sample: int
    end_sample: int
    # True when this window is a slice of a longer segment rather than the
    # whole of one. The split lands wherever the arithmetic put it, which may
    # be inside a word, and a consumer deserves to know that happened.
    split: bool = False

    @property
    def sample_count(self) -> int:
        return self.end_sample - self.start_sample


def decode_windows(
    segments: Sequence[tuple[int, int]],
    sample_rate: int,
    max_window_seconds: int = MAX_WINDOW_SECONDS,
) -> list[DecodeWindow]:
    """One window per speech segment, split only where the decoder demands it.

    Silence between segments is never decoded. That is the throughput argument,
    but it is also the accuracy one: a recognizer asked what was said during
    four seconds of room tone will frequently answer, and the answer will be
    plausible.
    """

    if sample_rate <= 0:
        raise ValueError("sample rate must be positive")
    limit = max_window_seconds * sample_rate
    if limit <= 0:
        raise ValueError("window limit must be positive")

    windows: list[DecodeWindow] = []
    for index, (start, end) in enumerate(segments):
        if end <= start:
            continue
        if end - start <= limit:
            windows.append(DecodeWindow(index, start, end))
            continue
        # A speech run longer than the decoder's context. Splitting at the
        # quietest interior point would be kinder to whichever word straddles
        # the boundary; doing that needs the per-window probabilities, which
        # live in the detector rather than in its published segments.
        cursor = start
        while cursor < end:
            stop = min(cursor + limit, end)
            windows.append(DecodeWindow(index, cursor, stop, split=True))
            cursor = stop
    return windows


__all__ = ["MAX_WINDOW_SECONDS", "DecodeWindow", "decode_windows"]
