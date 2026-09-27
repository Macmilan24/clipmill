"""The real weights, loaded and run.

The arithmetic around them is checked by hand elsewhere; these check what
arithmetic cannot: that the pinned graph takes the frames this package makes
and gives a print per window, the same one twice.

Skipped when the weights are absent, because that means a machine that has
not run `./tools/fetch-models.sh campplus-voxceleb` — not a regression.
"""

from __future__ import annotations

from pathlib import Path

import numpy as np
import pytest
from clipmill_worker_speakers.fbank import fbank
from clipmill_worker_speakers.model import MODEL_FILE, VoicePrints

REPOSITORY = Path(__file__).resolve().parents[3]
WEIGHTS = REPOSITORY / ".cache" / "models" / "campplus-voxceleb" / MODEL_FILE


class _Pinned:
    """The shape `require_model` returns, without a lease to get it from."""

    def __init__(self, root: Path) -> None:
        self._root = root

    def path(self, relative: str) -> Path:
        assert relative == MODEL_FILE
        return self._root


@pytest.fixture(scope="module")
def model() -> VoicePrints:
    if not WEIGHTS.is_file():
        pytest.skip(
            f"pinned weights absent at {WEIGHTS}; run ./tools/fetch-models.sh campplus-voxceleb"
        )
    return VoicePrints(_Pinned(WEIGHTS))  # type: ignore[arg-type]


def windows(seed: int) -> np.ndarray:
    rng = np.random.default_rng(seed)
    samples = rng.normal(0, 2000, 24_000).astype(np.int16)
    frames = fbank(samples)
    return (frames - frames.mean(axis=0))[None]


def test_a_window_of_frames_gives_one_print(model: VoicePrints) -> None:
    print_ = model(windows(1))
    assert print_.shape == (1, 512)
    assert np.isfinite(print_).all()


def test_the_same_window_twice_prints_the_same(model: VoicePrints) -> None:
    """Prints reach a content address. A model whose output moved with how
    busy the machine was would make two readings of one recording disagree."""

    assert np.array_equal(model(windows(2)), model(windows(2)))


def test_windows_are_printed_together_as_they_are_alone(model: VoicePrints) -> None:
    together = model(np.concatenate([windows(3), windows(4)]))
    assert np.allclose(together[0], model(windows(3))[0], atol=1e-4)
    assert np.allclose(together[1], model(windows(4))[0], atol=1e-4)
