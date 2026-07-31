"""Association, against detections placed by hand.

Each test is one thing a track has to survive, because the failures are not
symmetric: a track that fragments becomes a clip nobody follows, and a track
welded out of two people becomes a camera that swings between them.
"""

from __future__ import annotations

from clipmill_worker_faces.tracking import Parameters, associate, bridge
from clipmill_worker_faces.yunet import Detection

FRAME = 22_500  # a quarter second at 1/90000

SETTINGS = Parameters(
    start_score=0.6,
    match_iou=0.5,
    recover_iou=0.3,
    max_gap_frames=6,
    min_track_frames=4,
)


def at(index: int) -> int:
    return index * FRAME


def face(x: float, score: float = 0.9) -> Detection:
    return Detection(x=x, y=100.0, w=60.0, h=60.0, score=score)


def test_a_steady_face_is_one_track() -> None:
    frames = [(at(index), [face(200.0 + index)]) for index in range(12)]

    tracks = associate(frames, SETTINGS)

    assert len(tracks) == 1
    assert tracks[0].seen == 12


def test_two_people_stay_two_tracks() -> None:
    frames = [(at(index), [face(100.0), face(400.0)]) for index in range(10)]

    tracks = associate(frames, SETTINGS)

    assert len(tracks) == 2
    assert all(track.seen == 10 for track in tracks)


def test_a_face_that_dips_below_the_threshold_keeps_its_identity() -> None:
    """The reason the second pass exists. A turned head loses score before it
    disappears, and ending the track there would restart it under a new id."""

    frames = []
    for index in range(12):
        score = 0.35 if index in (5, 6) else 0.9
        frames.append((at(index), [face(200.0, score)]))

    tracks = associate(frames, SETTINGS)

    assert len(tracks) == 1, "the dip started a second track"
    # The weak frames continued the track, so they are observations too.
    assert len(tracks[0].observations) == 12


def test_a_face_that_leaves_and_a_different_one_that_arrives_are_two_tracks() -> None:
    frames = [(at(index), [face(100.0)]) for index in range(8)]
    frames += [(at(index), []) for index in range(8, 20)]
    frames += [(at(index), [face(500.0)]) for index in range(20, 30)]

    tracks = associate(frames, SETTINGS)

    assert len(tracks) == 2
    assert tracks[0].observations[-1].t_ticks < tracks[1].observations[0].t_ticks


def test_a_flicker_shorter_than_a_track_is_discarded() -> None:
    frames = [(at(index), [face(200.0)]) for index in range(10)]
    # Two frames of something else, which is a detector artefact as often as a
    # person, and a camera that followed it would be chasing.
    frames[4] = (at(4), [face(200.0), face(600.0)])
    frames[5] = (at(5), [face(200.0), face(601.0)])

    tracks = associate(frames, SETTINGS)

    assert len(tracks) == 1


def test_frames_with_nothing_in_them_still_age_a_track() -> None:
    """A missing frame and an empty one are different facts, and only the second
    one means the face was gone."""

    frames = [(at(index), [face(200.0)]) for index in range(6)]
    frames += [(at(index), []) for index in range(6, 20)]

    tracks = associate(frames, SETTINGS)

    assert len(tracks) == 1
    assert tracks[0].observations[-1].t_ticks == at(5)


def test_association_is_deterministic() -> None:
    frames = [(at(index), [face(100.0 + index), face(400.0 - index)]) for index in range(10)]

    first = associate(frames, SETTINGS)
    second = associate(frames, SETTINGS)

    assert [(track.track_id, track.seen) for track in first] == [
        (track.track_id, track.seen) for track in second
    ]


def test_bridging_fills_a_gap_and_marks_what_it_filled() -> None:
    frames = [(at(index), [face(200.0)]) for index in range(4)]
    frames += [(at(4), []), (at(5), [])]
    # Back 15 pixels along: a 60-wide box overlapping its own last position by
    # 0.6, which is a face that moved rather than a different face.
    frames += [(at(index), [face(215.0)]) for index in range(6, 10)]

    tracks = associate(frames, SETTINGS)
    assert len(tracks) == 1

    filled = bridge(tracks[0], [at(index) for index in range(10)])
    times = [item.t_ticks for item in filled.observations]

    assert times == [at(index) for index in range(10)]
    bridged = [item for item in filled.observations if item.interpolated]
    assert [item.t_ticks for item in bridged] == [at(4), at(5)]
    # The bridged boxes sit between the two they were interpolated from.
    assert 200.0 < bridged[0].detection.x < 215.0
    # And they contribute to neither the count nor the mean, because nobody saw
    # them: a gap-filled track must not read as continuous evidence.
    assert filled.seen == 8
