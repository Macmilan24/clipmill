"""Where the camera changed, and how much the detector believes it.

The detection itself is PySceneDetect's content detector: consecutive frames
are compared in HSV, and a boundary is called where the mean per-pixel distance
crosses a threshold. That algorithm is not reimplemented here — what is here is
everything around it that the observation contract needs and the library does
not provide.

    the score       kept per cut, because it is the only number a re-tune can
                    be reasoned about from without decoding the video again
    the confidence  a distribution rather than a flag; a cut two points over
                    the bar and a cut three times over it are both "a cut" to
                    the library and must not be to anything downstream
    the spans       the shots between cuts, tiling coverage exactly, because
                    that is what the boundary lattice consumes

Everything in this module is a pure function of frames and parameters, so the
cases real footage does not contain — a cut on the first frame, a flash, a
recording that never changes — are tested against arrays written by hand.

One limit is worth stating rather than discovering: the minimum shot enforces a
minimum *length*, not a rejection. A camera flash is two large content changes
a frame apart, and the minimum shot collapses them into one boundary rather
than none, because a flash really is a change and this detector has no way to
know it was not a cut. Rejecting them belongs to a detector that models motion,
which is a Phase 2 question.
"""

from __future__ import annotations

from collections.abc import Callable, Iterable
from dataclasses import dataclass

import numpy as np
from scenedetect.common import FrameTimecode
from scenedetect.detectors import ContentDetector
from scenedetect.stats_manager import StatsManager

IMPLEMENTATION = "pyscenedetect-content"
# What maps a raw content distance onto the confidence pair below. Named in the
# document, so a later calibration is a distinguishable claim rather than a
# quiet change to what the same numbers mean.
CALIBRATION = "content-score-over-threshold.v1"

# Defaults for a payload that leaves a parameter at zero. PySceneDetect's own
# threshold, half a second of minimum shot at broadcast rates, and a height
# that keeps the score about the shot rather than about sensor noise.
DEFAULT_THRESHOLD = 27.0
DEFAULT_MIN_SHOT_TICKS = 45_000
DEFAULT_ANALYSIS_HEIGHT = 180


class DetectionRefused(ValueError):
    """A parameter that would make the result meaningless."""


@dataclass(frozen=True, slots=True)
class Parameters:
    threshold: float
    min_shot_frames: int


@dataclass(frozen=True, slots=True)
class Confidence:
    p50: float
    p10: float


@dataclass(frozen=True, slots=True)
class Cut:
    """A detected boundary, at the first frame of the incoming shot."""

    frame: int
    score: float
    confidence: Confidence


@dataclass(frozen=True, slots=True)
class Span:
    """A shot, as a half-open range of frame indices."""

    start_frame: int
    end_frame: int
    confidence: Confidence


def confidence(score: float, threshold: float) -> Confidence:
    """Map a content distance onto what the cut is worth.

    The mapping is deliberately plain and deliberately named. `p50` reads the
    threshold as the halfway point: a boundary that only just cleared the bar
    is an even bet, and one at twice the bar is as certain as this detector
    gets. `p10` credits only the margin above the bar, which is the pessimistic
    reading a stage widening its uncertainty needs — a cut one point over the
    threshold is worth almost nothing under it, and should be.

    Neither number is a probability of anything. It is a stated, reproducible
    reading of a distance, which is what `calibration` in the document says.
    """

    if threshold <= 0:
        raise DetectionRefused("a threshold of zero or less calls every frame a cut")
    span = 2.0 * threshold
    return Confidence(
        p50=round(min(1.0, max(0.0, score / span)), 4),
        p10=round(min(1.0, max(0.0, (score - threshold) / span)), 4),
    )


def detect(
    frames: Iterable[np.ndarray],
    parameters: Parameters,
    frame_rate: float,
    on_frame: Callable[[int], None] | None = None,
) -> tuple[list[Cut], int]:
    """Run the detector over a stream of BGR frames.

    Returns the cuts and how many frames were actually seen. The count is not
    derivable from the proxy's declared duration — a container can round, and
    coverage must describe what was examined rather than what was promised.
    """

    if parameters.min_shot_frames < 1:
        raise DetectionRefused("a minimum shot of less than one frame suppresses nothing")
    detector = ContentDetector(
        threshold=parameters.threshold,
        min_scene_len=parameters.min_shot_frames,
    )
    # The score is not returned by `process_frame`, and the supported way to
    # read it is to give the detector somewhere to record its metrics. The cost
    # is that this also switches on edge detection, whose weight is zero here —
    # work that does not change the score, paid to avoid reaching into a
    # private attribute for a number the document is required to carry.
    metrics = StatsManager()
    detector.stats_manager = metrics

    cuts: list[Cut] = []
    scores: dict[int, float] = {}
    seen = 0
    for index, frame in enumerate(frames):
        timecode = FrameTimecode(index, frame_rate)
        boundaries = detector.process_frame(timecode, frame)
        # The first frame records no score, because there is nothing behind it
        # to differ from. Zero is the honest reading rather than a missing one.
        recorded = metrics.get_metrics(timecode, [ContentDetector.FRAME_SCORE_KEY])[0]
        scores[index] = 0.0 if recorded is None else float(recorded)
        for boundary in boundaries:
            cuts.append(_cut(boundary.frame_num, scores, parameters.threshold))
        seen = index + 1
        if on_frame is not None:
            on_frame(seen)

    if seen:
        # The flash filter can be holding a boundary back when the frames run
        # out. Asking for it is what stops a cut in the last half-second from
        # being lost to an implementation detail.
        for boundary in detector.post_process(FrameTimecode(seen, frame_rate)):
            cuts.append(_cut(boundary.frame_num, scores, parameters.threshold))

    # A boundary the filter delayed can be emitted after a later one, and the
    # same frame can be reported twice across the loop and the flush.
    unique = {cut.frame: cut for cut in cuts}
    # Frame zero has no predecessor to differ from, so a "cut" there is the
    # recording starting rather than the camera changing.
    return [unique[frame] for frame in sorted(unique) if frame > 0], seen


def _cut(frame: int, scores: dict[int, float], threshold: float) -> Cut:
    score = scores.get(frame, 0.0)
    return Cut(frame=frame, score=score, confidence=confidence(score, threshold))


def spans(cuts: list[Cut], frame_count: int) -> list[Span]:
    """The shots between the cuts, tiling the analyzed range exactly.

    A shot is worth no more than the weaker of the two cuts that bound it, and
    an edge that is the start or end of the recording is a fact rather than a
    detection and claims nothing on its own. A recording with no cuts is
    therefore one certain shot: nobody proposed a boundary, and coverage
    already says the whole thing was examined.
    """

    if frame_count <= 0:
        return []
    certain = Confidence(p50=1.0, p10=1.0)
    boundaries = [0, *[cut.frame for cut in cuts], frame_count]
    result: list[Span] = []
    for position in range(len(boundaries) - 1):
        # The cut that starts this span, and the one that ends it. Either may
        # be absent at the ends of the recording.
        opening = cuts[position - 1] if position > 0 else None
        closing = cuts[position] if position < len(cuts) else None
        edges = [cut for cut in (opening, closing) if cut is not None]
        weakest = min(edges, key=lambda cut: cut.confidence.p50).confidence if edges else certain
        result.append(
            Span(
                start_frame=boundaries[position],
                end_frame=boundaries[position + 1],
                confidence=weakest,
            )
        )
    return result


__all__ = [
    "CALIBRATION",
    "DEFAULT_ANALYSIS_HEIGHT",
    "DEFAULT_MIN_SHOT_TICKS",
    "DEFAULT_THRESHOLD",
    "IMPLEMENTATION",
    "Confidence",
    "Cut",
    "DetectionRefused",
    "Parameters",
    "Span",
    "confidence",
    "detect",
    "spans",
]
