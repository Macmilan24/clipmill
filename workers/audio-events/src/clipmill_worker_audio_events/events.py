"""Window scores into timed events: the deterministic half of the worker.

The tagger scores each window for 527 AudioSet classes. Each event kind takes
the highest score among its classes; a run of windows that reaches the kind's
`on` threshold and stays above `off` is one event, from the start of its first
window to the end of its last. Hysteresis keeps a laugh that dips for a moment
one laugh, and a gap of one window inside a run is bridged for the same reason.
"""

from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass

#: The AudioSet classes each kind is heard as, by their published names.
KIND_LABELS: dict[str, tuple[str, ...]] = {
    "laughter": (
        "Laughter",
        "Baby laughter",
        "Giggle",
        "Snicker",
        "Belly laugh",
        "Chuckle",
    ),
    "applause": ("Applause", "Clapping"),
    "cheering": ("Cheering", "Crowd"),
    "music": ("Music",),
    "shout": ("Shout", "Yell", "Screaming"),
    "gasp": ("Gasp",),
}

#: Start and continue thresholds per kind. Music is everywhere at a low level
#: in produced content, so it needs more to count; a gasp is brief and quiet.
THRESHOLDS: dict[str, tuple[float, float]] = {
    "laughter": (0.35, 0.2),
    "applause": (0.4, 0.25),
    "cheering": (0.4, 0.25),
    "music": (0.5, 0.35),
    "shout": (0.45, 0.3),
    "gasp": (0.3, 0.2),
}

KINDS: tuple[str, ...] = tuple(KIND_LABELS)


@dataclass(frozen=True)
class Event:
    kind: str
    start_sample: int
    end_sample: int
    peak: float
    mean: float


def kind_columns(labels: Sequence[str]) -> dict[str, tuple[int, ...]]:
    """For each kind, the indexes of its classes in the tagger's output.

    A kind none of whose classes the model has is an error: a silently empty
    kind would publish "no laughter" for a model that cannot hear it.
    """
    index = {label: position for position, label in enumerate(labels)}
    columns: dict[str, tuple[int, ...]] = {}
    for kind, names in KIND_LABELS.items():
        found = tuple(index[name] for name in names if name in index)
        if not found:
            raise ValueError(f"the tagger names none of the classes for {kind}")
        columns[kind] = found
    return columns


def detect(
    scores: Sequence[Sequence[float]],
    columns: dict[str, tuple[int, ...]],
    *,
    window_samples: int,
    hop_samples: int,
    total_samples: int,
) -> list[Event]:
    """Events from per-window class scores, in time order, then by kind."""
    events: list[Event] = []
    for kind in KINDS:
        on, off = THRESHOLDS[kind]
        series = [max(row[column] for column in columns[kind]) for row in scores]
        events.extend(_runs(kind, series, on, off, window_samples, hop_samples, total_samples))
    events.sort(key=lambda event: (event.start_sample, event.end_sample, event.kind))
    return events


def _runs(
    kind: str,
    series: list[float],
    on: float,
    off: float,
    window: int,
    hop: int,
    total: int,
) -> list[Event]:
    events: list[Event] = []
    position = 0
    while position < len(series):
        if series[position] < on:
            position += 1
            continue
        start = position
        # Walk back over the windows above `off` that led up to the peak.
        while start > 0 and series[start - 1] >= off:
            start -= 1
        end = position
        while True:
            if end + 1 < len(series) and series[end + 1] >= off:
                end += 1
            elif end + 2 < len(series) and series[end + 2] >= on:
                # One quiet window inside a run does not end it.
                end += 2
            else:
                break
        values = series[start : end + 1]
        events.append(
            Event(
                kind=kind,
                start_sample=start * hop,
                end_sample=min(end * hop + window, total),
                peak=round(max(values), 4),
                mean=round(sum(values) / len(values), 4),
            )
        )
        position = end + 1
    return events
