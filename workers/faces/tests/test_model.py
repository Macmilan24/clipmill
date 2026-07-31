"""The real weights, loaded and run.

The decoding is pinned by arithmetic elsewhere; what these check is the part
arithmetic cannot: that the graph this package expects is the graph the pinned
file contains, that a frame goes in and boxes come out inside the picture, and
that running it twice gives the same answer.

Skipped when the weights are absent, because that means a machine that has not
run `./tools/fetch-models.sh` — not a regression.
"""

from __future__ import annotations

from pathlib import Path

import numpy as np
import pytest
from clipmill_worker_faces.yunet import INPUT_SIZE, MODEL_FILE, STRIDES, YuNet

REPOSITORY = Path(__file__).resolve().parents[3]
WEIGHTS = REPOSITORY / ".cache" / "models" / "yunet-face" / MODEL_FILE


class _Pinned:
    """The shape `require_model` returns, without a lease to get it from."""

    def __init__(self, root: Path) -> None:
        self._root = root

    def path(self, relative: str) -> Path:
        assert relative == MODEL_FILE
        return self._root


@pytest.fixture(scope="module")
def detector() -> YuNet:
    if not WEIGHTS.is_file():
        pytest.skip(f"pinned weights absent at {WEIGHTS}; run ./tools/fetch-models.sh yunet-face")
    return YuNet(_Pinned(WEIGHTS))  # type: ignore[arg-type]


def test_the_pinned_graph_is_the_one_this_package_decodes(detector: YuNet) -> None:
    names = {output.name for output in detector.session.get_outputs()}
    for stride in STRIDES:
        assert f"cls_{stride}" in names
        assert f"obj_{stride}" in names
        assert f"bbox_{stride}" in names
    shape = detector.session.get_inputs()[0].shape
    assert shape[1:] == [3, INPUT_SIZE, INPUT_SIZE], f"the graph takes {shape}"


def test_a_frame_of_noise_produces_boxes_inside_the_picture(detector: YuNet) -> None:
    """Noise is not a face, but whatever the model claims about it must still be
    a box the rest of the stage can normalize."""

    rng = np.random.default_rng(20260731)
    frame = rng.integers(0, 256, (INPUT_SIZE, INPUT_SIZE, 3), dtype=np.uint8)

    found = detector.detect(frame, score_threshold=0.3, nms_iou=0.3)

    for box in found:
        assert box.w > 0 and box.h > 0
        assert -INPUT_SIZE < box.x < 2 * INPUT_SIZE
        assert 0.0 <= box.score <= 1.0


def test_the_same_frame_twice_gives_the_same_boxes(detector: YuNet) -> None:
    """These boxes reach a content address. A detector whose output moved with
    how busy the machine was would make two runs of one recording disagree while
    sharing one address."""

    rng = np.random.default_rng(7)
    frame = rng.integers(0, 256, (INPUT_SIZE, INPUT_SIZE, 3), dtype=np.uint8)

    assert detector.detect(frame, 0.3, 0.3) == detector.detect(frame, 0.3, 0.3)


def test_a_frame_of_the_wrong_size_is_refused(detector: YuNet) -> None:
    with pytest.raises(ValueError):
        detector.detect(np.zeros((320, 320, 3), dtype=np.uint8), 0.5, 0.3)
