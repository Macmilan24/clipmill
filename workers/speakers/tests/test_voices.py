"""Grouping prints into voices and voices into turns, on prints made by hand.

The model is not needed to check the arithmetic: a voice is a direction, and
prints of it are that direction plus a little noise.
"""

from __future__ import annotations

import numpy as np
from clipmill_worker_speakers.voices import (
    Clustering,
    Turn,
    Window,
    average_linkage,
    prints,
    turns_of,
    voices_of,
    windows_of,
)

CLUSTERING = Clustering()
SECOND = 16_000


def direction(seed: int, size: int = 64) -> np.ndarray:
    vector = np.random.default_rng(seed).normal(size=size)
    return vector / np.linalg.norm(vector)


def spoken(voice: np.ndarray, count: int, rng: np.random.Generator, noise: float = 0.35):
    points = voice[None, :] + noise * rng.normal(size=(count, len(voice))) / np.sqrt(len(voice))
    return points / np.linalg.norm(points, axis=1, keepdims=True)


def test_windows_cover_each_stretch_and_the_last_ends_with_it() -> None:
    stretches = [(0, 4_000), (10_000, 30_000), (40_000, 40_000 + 3 * SECOND)]
    windows = windows_of(stretches, CLUSTERING)
    # Too short for a print; one print over a short stretch; overlapping
    # prints over a long one, the last ending where the stretch ends.
    assert windows[0] == Window(1, 10_000, 30_000)
    long = [window for window in windows if window.stretch == 2]
    assert [(window.start - 40_000) for window in long] == [0, 12_000, 24_000]
    assert long[-1].end == 40_000 + 3 * SECOND
    assert all(window.end - window.start == CLUSTERING.window_samples for window in long)


def test_the_most_alike_are_joined_while_they_stay_alike() -> None:
    rng = np.random.default_rng(1)
    points = np.vstack([spoken(direction(1), 5, rng), spoken(direction(2), 5, rng)])
    labels = average_linkage(points, 0.5)
    assert len(set(labels[:5])) == 1
    assert len(set(labels[5:])) == 1
    assert labels[0] != labels[5]


def test_voices_are_numbered_in_the_order_they_are_first_heard() -> None:
    rng = np.random.default_rng(2)
    host, guest = direction(3), direction(4)
    points = np.vstack(
        [
            spoken(guest, 30, rng),
            spoken(host, 20, rng),
            spoken(guest, 25, rng),
            spoken(host, 15, rng),
        ]
    )
    voices = voices_of(points, CLUSTERING)
    assert voices.tolist() == [0] * 30 + [1] * 20 + [0] * 25 + [1] * 15


def test_one_voice_split_by_the_room_is_one_voice() -> None:
    """The same person twice over: two groups a little apart, whose centres
    still point the same way."""

    rng = np.random.default_rng(5)
    voice = direction(6)
    near = voice + 0.35 * direction(7)
    near /= np.linalg.norm(near)
    points = np.vstack([spoken(voice, 40, rng, 0.9), spoken(near, 40, rng, 0.9)])
    assert set(voices_of(points, CLUSTERING).tolist()) == {0}


def test_a_voice_heard_for_a_moment_is_folded_into_the_nearest() -> None:
    rng = np.random.default_rng(8)
    points = np.vstack(
        [
            spoken(direction(9), 300, rng),
            spoken(direction(10), 4, rng),
            spoken(direction(11), 300, rng),
        ]
    )
    voices = voices_of(points, CLUSTERING)
    assert set(voices.tolist()) == {0, 1}


def test_nobody_speaking_is_nobody() -> None:
    assert voices_of(np.zeros((0, 64), np.float32), CLUSTERING).tolist() == []
    assert turns_of([], [], [], CLUSTERING) == []


def test_turns_split_overlaps_down_the_middle_and_smooth_a_stray_print() -> None:
    stretch = [(0, 6 * SECOND)]
    windows = windows_of(stretch, CLUSTERING)
    # Seven windows; the third alone says someone else.
    assert len(windows) == 7
    voices = [0, 0, 1, 0, 0, 1, 1]
    turns = turns_of(stretch, windows, voices, CLUSTERING)
    # The stray is taken to be the voice either side of it, and the change
    # falls halfway through the overlap between the last print of one and the
    # first of the other.
    assert turns == [
        Turn(0, (windows[4].end + windows[5].start) // 2, 0),
        Turn((windows[4].end + windows[5].start) // 2, 6 * SECOND, 1),
    ]


def test_a_pause_the_same_voice_resumes_after_is_one_turn() -> None:
    stretches = [(0, SECOND), (SECOND + SECOND // 2, 3 * SECOND), (6 * SECOND, 7 * SECOND)]
    windows = windows_of(stretches, CLUSTERING)
    turns = turns_of(stretches, windows, [0] * len(windows), CLUSTERING)
    # Half a second apart is one turn; three seconds apart is two.
    assert turns == [Turn(0, 3 * SECOND, 0), Turn(6 * SECOND, 7 * SECOND, 0)]


def test_each_print_is_of_its_own_window_with_its_mean_taken_out() -> None:
    seen: list[np.ndarray] = []

    def embed(batch: np.ndarray) -> np.ndarray:
        seen.append(batch)
        # A print that is the window's first frame, so windows differ.
        return batch[:, 0, :].copy() + 1.0

    rng = np.random.default_rng(12)
    samples = (rng.normal(0, 3000, 4 * SECOND)).astype(np.int16)
    stretches = [(0, 4 * SECOND)]
    windows = windows_of(stretches, CLUSTERING)
    done: list[tuple[int, int]] = []
    points = prints(embed, samples, stretches, windows, on_stretch=lambda a, b: done.append((a, b)))
    assert points.shape == (len(windows), 80)
    assert np.allclose(np.linalg.norm(points, axis=1), 1.0, atol=1e-5)
    for batch in seen:
        assert batch.shape[1:] == (148, 80)
        assert np.abs(batch.mean(axis=1)).max() < 1e-3
    assert done == [(1, 1)]
