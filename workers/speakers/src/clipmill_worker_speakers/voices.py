"""Telling voices apart, and laying them back onto the speech as turns.

Each stretch of speech voice activity found is cut into windows a second and
a half long, overlapping by half, and each window gets a print of how it
sounds. Prints of one voice point the same way; of two voices, apart.

They are grouped in two steps. First the most alike are joined while their
average likeness stays high — on an even sample of at most two thousand
windows, so an hour-long recording costs what a short one does. That leaves
a few large groups and many stragglers: laughter, two people at once, a
cough. Then groups whose centres are alike are joined, because one person
heard through a different microphone, or leaning away from it, makes groups
that average likeness keeps apart but whose centres point the same way.
Every window then goes to the voice whose centre it is nearest, twice over
so the centres settle, and a voice holding too little of the recording is
folded into its nearest: a voice heard for a minute of an hour is usually a
clip played on the show, and a turn nobody can name helps no one.

Voices are numbered in the order they are first heard. The same recording
read twice names them the same.
"""

from __future__ import annotations

from collections.abc import Callable, Sequence
from dataclasses import dataclass

import numpy as np

from .fbank import SHIFT, fbank, frame_count


@dataclass(frozen=True)
class Clustering:
    """How voices are told apart. Recorded in what is published."""

    window_samples: int = 24_000
    hop_samples: int = 12_000
    # A stretch shorter than this gets no print: too little to say who it is.
    shortest_samples: int = 8_000
    link: float = 0.5
    merge: float = 0.65
    smallest: float = 0.02
    sample_limit: int = 2_000
    # A pause the same voice resumes after stays one turn.
    turn_gap_samples: int = 16_000


@dataclass(frozen=True)
class Window:
    """A span of one speech stretch, in samples."""

    stretch: int
    start: int
    end: int


@dataclass(frozen=True)
class Turn:
    """Speech by one voice, in samples, the voice numbered from zero."""

    start: int
    end: int
    voice: int


# Frames of filterbank in, one print per row out.
Embedder = Callable[[np.ndarray], np.ndarray]


def windows_of(stretches: Sequence[tuple[int, int]], clustering: Clustering) -> list[Window]:
    """Windows over each stretch: one for a short stretch, overlapping ones
    over a long one, the last ending where the stretch ends."""

    windows: list[Window] = []
    span, hop = clustering.window_samples, clustering.hop_samples
    for index, (start, end) in enumerate(stretches):
        length = end - start
        if length < clustering.shortest_samples:
            continue
        if length <= span:
            windows.append(Window(index, start, end))
            continue
        at = start
        while at + span < end:
            windows.append(Window(index, at, at + span))
            at += hop
        windows.append(Window(index, end - span, end))
    return windows


def prints(
    embed: Embedder,
    samples: np.ndarray,
    stretches: Sequence[tuple[int, int]],
    windows: Sequence[Window],
    on_stretch: Callable[[int, int], None] | None = None,
) -> np.ndarray:
    """A unit-length print per window.

    The filterbank is computed once per stretch and each window takes its
    frames from it, with its own mean taken out — the normalisation the model
    was trained with, per window.
    """

    result = np.zeros((len(windows), 0), np.float32)
    by_stretch: dict[int, list[int]] = {}
    for position, window in enumerate(windows):
        by_stretch.setdefault(window.stretch, []).append(position)
    rows: dict[int, np.ndarray] = {}
    for done, (index, positions) in enumerate(sorted(by_stretch.items())):
        start, end = stretches[index]
        feats = fbank(samples[start:end])
        batch: dict[int, list[tuple[int, np.ndarray]]] = {}
        for position in positions:
            window = windows[position]
            first = (window.start - start) // SHIFT
            count = frame_count(window.end - window.start)
            frames = feats[first : first + count]
            frames = frames - frames.mean(axis=0)
            batch.setdefault(len(frames), []).append((position, frames))
        for length in sorted(batch):
            members = batch[length]
            out = embed(np.stack([frames for _, frames in members]))
            for (position, _), row in zip(members, out, strict=True):
                rows[position] = row
        if on_stretch is not None:
            on_stretch(done + 1, len(by_stretch))
    if rows:
        result = np.stack([rows[position] for position in range(len(windows))]).astype(np.float32)
        norms = np.linalg.norm(result, axis=1, keepdims=True)
        result = result / np.maximum(norms, 1e-9)
    return result


def _unit(vector: np.ndarray) -> np.ndarray:
    return vector / max(float(np.linalg.norm(vector)), 1e-9)


def average_linkage(points: np.ndarray, threshold: float) -> np.ndarray:
    """Join the two most alike groups while their average likeness is at or
    above the threshold. Returns a group label per point."""

    count = len(points)
    labels = np.arange(count)
    if count < 2:
        return labels
    likeness = (points @ points.T).astype(np.float64)
    np.fill_diagonal(likeness, -np.inf)
    sizes = np.ones(count)
    while True:
        a, b = divmod(int(np.argmax(likeness)), count)
        if likeness[a, b] < threshold:
            break
        joined = (sizes[a] * likeness[a] + sizes[b] * likeness[b]) / (sizes[a] + sizes[b])
        likeness[a] = joined
        likeness[:, a] = joined
        likeness[a, a] = -np.inf
        likeness[b] = -np.inf
        likeness[:, b] = -np.inf
        sizes[a] += sizes[b]
        labels[labels == b] = a
    return labels


def _join_alike(centres: list[np.ndarray], sizes: list[int], merge: float) -> list[np.ndarray]:
    """Join voices whose centres are alike, most alike first."""

    centres, sizes = list(centres), list(sizes)
    while len(centres) > 1:
        stacked = np.array(centres)
        likeness = stacked @ stacked.T
        np.fill_diagonal(likeness, -np.inf)
        a, b = divmod(int(np.argmax(likeness)), len(centres))
        if likeness[a, b] < merge:
            break
        centres[a] = _unit(sizes[a] * centres[a] + sizes[b] * centres[b])
        sizes[a] += sizes[b]
        del centres[b], sizes[b]
    return centres


def voices_of(points: np.ndarray, clustering: Clustering) -> np.ndarray:
    """A voice per print, numbered in the order the voices are first heard."""

    count = len(points)
    if count == 0:
        return np.zeros(0, np.int64)
    sample = np.arange(0, count, max(1, -(-count // clustering.sample_limit)))
    labels = average_linkage(points[sample], clustering.link)
    values, sizes = np.unique(labels, return_counts=True)
    floor = max(3, int(0.005 * len(sample)))
    groups = [(value, size) for value, size in zip(values, sizes, strict=True) if size >= floor]
    if not groups:
        groups = [(values[int(np.argmax(sizes))], int(sizes.max()))]
    centres = [_unit(points[sample[labels == value]].mean(axis=0)) for value, _ in groups]
    centres = _join_alike(centres, [int(size) for _, size in groups], clustering.merge)
    keep_at = max(8, clustering.smallest * count)
    for _ in range(3):
        assigned = np.argmax(points @ np.array(centres).T, axis=1)
        held = np.bincount(assigned, minlength=len(centres))
        kept = [index for index in range(len(centres)) if held[index] >= keep_at]
        if not kept:
            kept = [int(np.argmax(held))]
        centres = [_unit(points[assigned == index].mean(axis=0)) for index in kept]
        centres = _join_alike(centres, [int(held[index]) for index in kept], clustering.merge)
    assigned = np.argmax(points @ np.array(centres).T, axis=1)
    order: dict[int, int] = {}
    for value in assigned:
        order.setdefault(int(value), len(order))
    return np.array([order[int(value)] for value in assigned], np.int64)


def turns_of(
    stretches: Sequence[tuple[int, int]],
    windows: Sequence[Window],
    voices: Sequence[int],
    clustering: Clustering,
) -> list[Turn]:
    """Speech by voice.

    A window between two of one other voice in the same stretch is taken to be
    that voice: a print over a cough or a word said over someone is less sure
    than the two either side of it. Overlapping windows split their overlap
    down the middle; a stretch's first and last window reach its edges.
    """

    smoothed = list(voices)
    for index in range(1, len(windows) - 1):
        before, here, after = windows[index - 1], windows[index], windows[index + 1]
        if (
            before.stretch == here.stretch == after.stretch
            and voices[index - 1] == voices[index + 1] != voices[index]
        ):
            smoothed[index] = voices[index - 1]
    turns: list[Turn] = []
    for index, (window, voice) in enumerate(zip(windows, smoothed, strict=True)):
        start, end = stretches[window.stretch]
        previous = windows[index - 1] if index > 0 else None
        following = windows[index + 1] if index + 1 < len(windows) else None
        begin = (
            (previous.end + window.start) // 2
            if previous is not None and previous.stretch == window.stretch
            else start
        )
        finish = (
            (window.end + following.start) // 2
            if following is not None and following.stretch == window.stretch
            else end
        )
        if (
            turns
            and turns[-1].voice == voice
            and begin - turns[-1].end <= clustering.turn_gap_samples
        ):
            turns[-1] = Turn(turns[-1].start, finish, voice)
        else:
            turns.append(Turn(begin, finish, int(voice)))
    return turns


__all__ = [
    "Clustering",
    "Embedder",
    "Turn",
    "Window",
    "average_linkage",
    "prints",
    "turns_of",
    "voices_of",
    "windows_of",
]
