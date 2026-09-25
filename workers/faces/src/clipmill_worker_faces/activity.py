"""How much each tracked face's mouth moves, from the frames the detector saw.

Framing that follows whoever is talking needs to know who is talking. A face
that is speaking changes around the mouth from one sampled frame to the next;
one that is listening mostly does not. So every detected face leaves two tiny
grey patches behind — the mouth region and the upper face — and along a track
the mouth's change is measured against the previous measured box, less what
the upper face changed with it: a head that turns or nods moves both, and only
the difference is speech.

No landmarks and no second model: a patch is sampled from the box YuNet
already found, so the measurement costs a few hundred numpy operations per
face. It is a cue, not an identification, and the director treats it as one.
"""

from __future__ import annotations

import numpy as np

from .yunet import Detection

#: Carried in the producer's implementation, beside the detector's: a change to
#: how mouths are read is a different face track, and must be keyed as one.
VERSION = "mouth-1"
#: The patch grid. Coarse on purpose: lips opening and closing survive it,
#: skin texture and compression noise do not.
ROWS = 8
COLUMNS = 16
#: Where the mouth and the upper face sit in a YuNet box, as shares of it.
MOUTH = (0.22, 0.62, 0.78, 0.96)
UPPER = (0.15, 0.16, 0.85, 0.48)
#: How much of the upper face's change is taken to be the head moving.
HEAD_SHARE = 0.8
#: A box smaller than this, in pixels of the letterboxed frame, has no mouth
#: worth reading.
SMALLEST = 18.0
#: Change between standardized patches runs 0..~2; this maps it onto 0..1.
SCALE = 1.2

Patches = tuple[np.ndarray, np.ndarray]


def _gray(image: np.ndarray) -> np.ndarray:
    """Luma of a blue-first frame, as the Rec. 601 weights give it."""

    pixels = image.astype(np.float32)
    return pixels[..., 0] * 0.114 + pixels[..., 1] * 0.587 + pixels[..., 2] * 0.299


def _sample(
    gray: np.ndarray, box: Detection, region: tuple[float, float, float, float]
) -> np.ndarray:
    """A ROWS x COLUMNS grid of the region, standardized so light does not count."""

    height, width = gray.shape
    left = box.x + region[0] * box.w
    top = box.y + region[1] * box.h
    right = box.x + region[2] * box.w
    bottom = box.y + region[3] * box.h
    rows = np.clip(np.linspace(top, bottom, ROWS), 0, height - 1).astype(np.int32)
    columns = np.clip(np.linspace(left, right, COLUMNS), 0, width - 1).astype(np.int32)
    patch = gray[rows[:, None], columns[None, :]]
    spread = float(patch.std())
    return (patch - float(patch.mean())) / (spread + 4.0)


def patches_for(image: np.ndarray, boxes: list[Detection]) -> list[Patches | None]:
    """The mouth and upper-face patches of every face found in one frame, in
    the order they were found; None for a face too small to read."""

    if not boxes:
        return []
    gray = _gray(image)
    return [
        None
        if box.w < SMALLEST or box.h < SMALLEST
        else (_sample(gray, box, MOUTH), _sample(gray, box, UPPER))
        for box in boxes
    ]


def track_motion(
    observations: list[tuple[int, Detection, bool]],
    measured: dict[int, Patches | None],
    longest_gap: int,
) -> dict[int, float]:
    """Mouth motion along one track, by the tick of each measured box.

    `observations` are the track's (tick, detection, interpolated) in time
    order and `measured` maps a detection's identity to its patches. A box is
    compared with the track's previous measured box only when the two are at
    most `longest_gap` ticks apart: across a longer gap the face may have
    turned or been cut away from, and the change would not be speech.
    """

    found: dict[int, float] = {}
    previous: tuple[int, Patches] | None = None
    for t_ticks, detection, interpolated in observations:
        if interpolated:
            continue
        current = measured.get(id(detection))
        if current is None:
            previous = None
            continue
        if previous is not None and t_ticks - previous[0] <= longest_gap:
            found[t_ticks] = motion(previous[1], current)
        previous = (t_ticks, current)
    return found


def motion(previous: Patches, current: Patches) -> float:
    """How much the mouth changed beyond the head, from 0 to 1."""

    mouth = float(np.abs(current[0] - previous[0]).mean())
    upper = float(np.abs(current[1] - previous[1]).mean())
    return round(min(max((mouth - HEAD_SHARE * upper) / SCALE, 0.0), 1.0), 4)
