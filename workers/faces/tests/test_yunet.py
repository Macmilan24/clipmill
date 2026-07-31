"""The decoding, against tensors built by hand.

Written this way because the alternative — testing only against whatever the
model says about a photograph — cannot tell a correct decode from a plausible
one. Here the anchor, the offset and the log-space size are chosen so the
expected box is arithmetic anybody can check by reading the test.
"""

from __future__ import annotations

import math

import numpy as np
import pytest
from clipmill_worker_faces.yunet import (
    INPUT_SIZE,
    STRIDES,
    Detection,
    decode,
    intersection_over_union,
    suppress,
)


def _empty_outputs() -> dict[str, np.ndarray]:
    outputs: dict[str, np.ndarray] = {}
    for stride in STRIDES:
        cells = (INPUT_SIZE // stride) ** 2
        outputs[f"cls_{stride}"] = np.zeros((1, cells, 1), dtype=np.float32)
        outputs[f"obj_{stride}"] = np.zeros((1, cells, 1), dtype=np.float32)
        outputs[f"bbox_{stride}"] = np.zeros((1, cells, 4), dtype=np.float32)
    return outputs


def test_an_anchor_decodes_to_the_box_its_offsets_describe() -> None:
    outputs = _empty_outputs()
    stride = 8
    side = INPUT_SIZE // stride
    # Row 3, column 5, which is index 3*80 + 5 in the model's row-major grid.
    row, column = 3, 5
    index = row * side + column
    outputs[f"cls_{stride}"][0, index, 0] = 0.81
    outputs[f"obj_{stride}"][0, index, 0] = 1.0
    # Centre half a cell past the anchor, and a box four strides on a side.
    outputs[f"bbox_{stride}"][0, index] = [0.5, 0.5, math.log(4.0), math.log(4.0)]

    found = decode(outputs, score_threshold=0.5)

    assert len(found) == 1
    box = found[0]
    assert box.w == pytest.approx(32.0)
    assert box.h == pytest.approx(32.0)
    # Centre = (column + 0.5) * 8 = 44, so the left edge is 44 - 16.
    assert box.x == pytest.approx(44.0 - 16.0)
    assert box.y == pytest.approx((row + 0.5) * stride - 16.0)


def test_the_score_is_the_geometric_mean_of_both_heads() -> None:
    outputs = _empty_outputs()
    outputs["cls_8"][0, 0, 0] = 0.64
    outputs["obj_8"][0, 0, 0] = 0.49
    outputs["bbox_8"][0, 0] = [0.0, 0.0, math.log(2.0), math.log(2.0)]

    found = decode(outputs, score_threshold=0.1)

    assert len(found) == 1
    assert found[0].score == pytest.approx(math.sqrt(0.64 * 0.49))


def test_the_grid_is_read_row_major() -> None:
    """Reading it the other way produces boxes mirrored about the diagonal —
    plausible, wrong, and invisible to anything downstream."""

    outputs = _empty_outputs()
    stride = 32
    side = INPUT_SIZE // stride
    row, column = 1, 4
    index = row * side + column
    outputs[f"cls_{stride}"][0, index, 0] = 1.0
    outputs[f"obj_{stride}"][0, index, 0] = 1.0
    outputs[f"bbox_{stride}"][0, index] = [0.0, 0.0, math.log(1.0), math.log(1.0)]

    box = decode(outputs, score_threshold=0.5)[0]

    # x follows the column and y follows the row, not the other way round.
    assert box.x + box.w / 2 == pytest.approx(column * stride)
    assert box.y + box.h / 2 == pytest.approx(row * stride)


def test_nothing_below_the_threshold_survives() -> None:
    outputs = _empty_outputs()
    outputs["cls_8"][0, 0, 0] = 0.3
    outputs["obj_8"][0, 0, 0] = 0.3
    assert decode(outputs, score_threshold=0.5) == []


def test_overlap_is_zero_for_boxes_that_do_not_touch() -> None:
    left = Detection(x=0, y=0, w=10, h=10, score=1.0)
    right = Detection(x=20, y=20, w=10, h=10, score=1.0)
    assert intersection_over_union(left, right) == 0.0
    assert intersection_over_union(left, left) == pytest.approx(1.0)


def test_suppression_keeps_the_strongest_of_an_overlapping_pair() -> None:
    strong = Detection(x=0, y=0, w=10, h=10, score=0.9)
    weak = Detection(x=1, y=1, w=10, h=10, score=0.6)
    apart = Detection(x=50, y=50, w=10, h=10, score=0.7)

    kept = suppress([weak, apart, strong], iou=0.3)

    assert [item.score for item in kept] == [0.9, 0.7]


def test_suppression_is_stable_for_equal_scores() -> None:
    """Two detections of identical strength must resolve the same way twice, or
    the surviving box depends on the order they arrived in."""

    first = Detection(x=0, y=0, w=10, h=10, score=0.8)
    second = Detection(x=2, y=0, w=10, h=10, score=0.8)
    assert suppress([first, second], iou=0.3) == suppress([second, first], iou=0.3)
