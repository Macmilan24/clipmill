"""The detection arithmetic, against frames written by hand.

Real footage does not contain a cut on frame zero, a flash exactly at the
minimum-shot boundary, or a recording that never changes by a single pixel.
Those are the cases that break the wiring around the detector — the score
lookup, the flush, the span tiling — so they are the cases tested here, where
the frames can be constructed rather than found.
"""

from __future__ import annotations

import numpy as np
import pytest
from clipmill_worker_shots.content import (
    DetectionRefused,
    Parameters,
    confidence,
    detect,
    spans,
)

FPS = 30000 / 1001
HEIGHT = 32
WIDTH = 48


def solid(blue: int, green: int, red: int) -> np.ndarray:
    frame = np.zeros((HEIGHT, WIDTH, 3), dtype=np.uint8)
    frame[:, :, 0] = blue
    frame[:, :, 1] = green
    frame[:, :, 2] = red
    return frame


def sequence(*runs: tuple[np.ndarray, int]) -> list[np.ndarray]:
    return [frame for frame, count in runs for _ in range(count)]


def test_a_confidence_reads_the_threshold_as_the_halfway_point() -> None:
    at_the_bar = confidence(27.0, 27.0)
    assert at_the_bar.p50 == 0.5
    # Nothing above the bar, so the pessimistic reading credits nothing.
    assert at_the_bar.p10 == 0.0

    doubled = confidence(54.0, 27.0)
    assert doubled.p50 == 1.0
    assert doubled.p10 == 0.5

    # Neither number ever leaves the unit interval, however far over the bar a
    # score lands. A raw content distance can exceed 100.
    extreme = confidence(400.0, 27.0)
    assert extreme.p50 == 1.0
    assert extreme.p10 == 1.0


def test_a_threshold_of_zero_is_refused() -> None:
    with pytest.raises(DetectionRefused):
        confidence(10.0, 0.0)


def test_a_change_of_scene_is_reported_once_at_the_incoming_frame() -> None:
    frames = sequence((solid(30, 30, 30), 20), (solid(220, 40, 200), 20))
    cuts, seen = detect(frames, Parameters(threshold=27.0, min_shot_frames=5), FPS)
    assert seen == 40
    assert [cut.frame for cut in cuts] == [20]
    assert cuts[0].score >= 27.0
    assert 0.0 < cuts[0].confidence.p10 <= cuts[0].confidence.p50 <= 1.0


def test_an_unchanging_recording_reports_no_cuts_at_all() -> None:
    """Not "one cut at frame zero". The first frame has no predecessor to
    differ from, and a recording starting is not the camera changing."""

    frames = sequence((solid(90, 90, 90), 30))
    cuts, seen = detect(frames, Parameters(threshold=27.0, min_shot_frames=5), FPS)
    assert cuts == []
    assert seen == 30


def test_the_minimum_shot_collapses_a_flash_into_one_boundary() -> None:
    """A single bright frame is two content changes a frame apart.

    The minimum shot cannot make a flash not a change — it genuinely is one,
    and this detector has no notion of rejecting it. What it does is stop the
    recording being reported as having gained a shot one frame long, which is
    the reading that would put a clip boundary inside a camera flash.
    """

    dark = solid(20, 20, 20)
    flash = solid(250, 250, 250)
    frames = sequence((dark, 20), (flash, 1), (dark, 20))
    lenient = detect(frames, Parameters(threshold=27.0, min_shot_frames=1), FPS)[0]
    strict = detect(frames, Parameters(threshold=27.0, min_shot_frames=15), FPS)[0]
    assert [cut.frame for cut in lenient] == [20, 21]
    assert [cut.frame for cut in strict] == [20]


def test_a_minimum_shot_below_one_frame_is_refused() -> None:
    with pytest.raises(DetectionRefused):
        detect([solid(0, 0, 0)], Parameters(threshold=27.0, min_shot_frames=0), FPS)


def test_no_frames_at_all_is_an_empty_result_rather_than_a_crash() -> None:
    """The worker turns this into a refusal, because a proxy that decodes to
    nothing is a broken proxy. The arithmetic still has to survive it."""

    cuts, seen = detect([], Parameters(threshold=27.0, min_shot_frames=5), FPS)
    assert (cuts, seen) == ([], 0)


def test_the_same_frames_produce_the_same_cuts_twice() -> None:
    frames = sequence((solid(30, 30, 30), 18), (solid(200, 60, 20), 18), (solid(10, 180, 90), 18))
    parameters = Parameters(threshold=27.0, min_shot_frames=5)
    first = detect(frames, parameters, FPS)
    second = detect(frames, parameters, FPS)
    assert first == second


class _Cut:
    """A stand-in, so span arithmetic is tested without running a detector."""

    def __init__(self, frame: int, p50: float) -> None:
        from clipmill_worker_shots.content import Confidence, Cut

        self.value = Cut(frame=frame, score=p50 * 54.0, confidence=Confidence(p50=p50, p10=0.1))


def test_spans_tile_the_recording_and_take_their_weakest_edge() -> None:
    cuts = [_Cut(100, 0.9).value, _Cut(250, 0.6).value]
    result = spans(cuts, 400)
    assert [(span.start_frame, span.end_frame) for span in result] == [
        (0, 100),
        (100, 250),
        (250, 400),
    ]
    # The opening span is bounded by the recording's start, which claims
    # nothing, and by the first cut — so it is worth exactly that cut.
    assert result[0].confidence.p50 == 0.9
    # The middle span is bounded by both, and takes the weaker.
    assert result[1].confidence.p50 == 0.6
    assert result[2].confidence.p50 == 0.6


def test_a_recording_with_no_cuts_is_one_certain_shot() -> None:
    """Certain because coverage already says the whole thing was examined and
    nobody proposed a boundary — not because the detector was confident."""

    result = spans([], 400)
    assert len(result) == 1
    assert (result[0].start_frame, result[0].end_frame) == (0, 400)
    assert result[0].confidence.p50 == 1.0
    assert result[0].confidence.p10 == 1.0


def test_a_recording_with_no_frames_has_no_shots() -> None:
    assert spans([], 0) == []
