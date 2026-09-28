"""Mouth motion: speech changes the mouth, a moving head changes everything."""

from __future__ import annotations

import numpy as np
from clipmill_worker_faces import activity
from clipmill_worker_faces.yunet import Detection

FACE = Detection(x=100.0, y=100.0, w=160.0, h=200.0, score=0.9)


def _frame(mouth_open: bool, shift: int = 0) -> np.ndarray:
    """A grey frame with a face-shaped block: eyes, and a mouth open or shut."""

    image = np.full((480, 480, 3), 120, dtype=np.uint8)
    image[100:300, 100 + shift : 260 + shift] = 180
    # Eyes, which do not move when someone speaks.
    image[150:165, 140 + shift : 165 + shift] = 30
    image[150:165, 195 + shift : 220 + shift] = 30
    # The mouth: a thin line shut, a tall dark gap open.
    if mouth_open:
        image[240:285, 150 + shift : 210 + shift] = 20
    else:
        image[258:264, 150 + shift : 210 + shift] = 40
    return image


def _patches(image: np.ndarray, box: Detection = FACE) -> activity.Patches:
    found = activity.patches_for(image, [box])[0]
    assert found is not None
    return found


def test_a_still_face_has_no_motion() -> None:
    shut = _patches(_frame(False))
    assert activity.motion(shut, _patches(_frame(False))) == 0.0


def test_a_mouth_opening_is_motion() -> None:
    shut = _patches(_frame(False))
    opened = _patches(_frame(True))
    assert activity.motion(shut, opened) > 0.2


def test_a_whole_head_moving_is_not_speech() -> None:
    # The box stays where the detector left it, so the face slides under it:
    # the upper face changes as much as the mouth, and that cancels.
    still = _patches(_frame(False))
    moved = _patches(_frame(False, shift=18))
    spoke = _patches(_frame(True))
    assert activity.motion(still, moved) < activity.motion(still, spoke)


def test_a_face_too_small_to_read_has_no_patches() -> None:
    tiny = Detection(x=10.0, y=10.0, w=12.0, h=14.0, score=0.9)
    assert activity.patches_for(_frame(False), [tiny]) == [None]
    assert activity.patches_for(_frame(False), []) == []


def test_motion_follows_a_track_and_not_across_gaps_or_guesses() -> None:
    frames = [_frame(False), _frame(True), _frame(False), _frame(True)]
    boxes = [Detection(FACE.x, FACE.y, FACE.w, FACE.h, 0.9) for _ in frames]
    measured = {
        id(box): patches
        for box, image in zip(boxes, frames, strict=True)
        for patches in activity.patches_for(image, [box])
    }
    step = 15_000
    observations = [
        (0, boxes[0], False),
        (step, boxes[1], False),
        # A bridged box is a guess, and nothing is measured from it.
        (2 * step, boxes[2], True),
        # Three steps after the last measured box: too far to compare.
        (4 * step, boxes[3], False),
    ]
    found = activity.track_motion(observations, measured, 2 * step)
    assert set(found) == {step}
    assert found[step] > 0.2
