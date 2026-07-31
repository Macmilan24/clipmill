"""Which detections are the same face, frame after frame.

Association in the ByteTrack manner, written out rather than imported: two
passes over each frame, the confident detections first and the weak ones after.
That ordering is the whole idea. A face that turns away, gets half-lit, or is
briefly occluded drops below the score a detection needs to *start* a track, but
not below what it needs to *continue* one — and letting it continue is the
difference between one person and a stream of one-frame strangers.

It matters more here than in the tracking literature's own setting. Downstream,
a track's length is what decides whether the camera follows it. A face that
fragments into six short tracks is a face nothing will follow, and the clip
comes back centred with a reason that is true but unhelpful: "the clearest face
appears in too little of this clip".

Everything here is arithmetic over boxes and is deterministic by construction:
candidates are considered in a fixed order and ties resolve the same way twice.
"""

from __future__ import annotations

from dataclasses import dataclass, field

from .yunet import Detection, intersection_over_union


@dataclass(slots=True)
class Observation:
    """One box on one track, and whether anybody actually saw it."""

    t_ticks: int
    detection: Detection
    interpolated: bool = False


@dataclass(slots=True)
class Track:
    track_id: int
    observations: list[Observation] = field(default_factory=list)
    #: Frames since the last time a detection matched. Reset on every match.
    missed: int = 0
    #: Where the box was last seen, which is what the next frame matches against.
    last: Detection | None = None

    @property
    def seen(self) -> int:
        return sum(1 for item in self.observations if not item.interpolated)

    @property
    def mean_score(self) -> float:
        measured = [item.detection.score for item in self.observations if not item.interpolated]
        return sum(measured) / len(measured) if measured else 0.0


@dataclass(frozen=True, slots=True)
class Parameters:
    """The association thresholds, all of which reach the artifact key."""

    #: A detection at or above this may start a track of its own.
    start_score: float
    #: Overlap at which a confident detection continues a track.
    match_iou: float
    #: The lower overlap the second pass uses, for detections too weak to start
    #: anything but strong enough to be the same face as something already
    #: being followed.
    recover_iou: float
    #: Frames a track may go unmatched before it closes.
    max_gap_frames: int
    #: Shortest track worth publishing.
    min_track_frames: int


def _assign(
    tracks: list[Track],
    candidates: list[Detection],
    threshold: float,
    t_ticks: int,
) -> list[Detection]:
    """Greedily match candidates to open tracks, returning what was left over.

    Greedy by overlap rather than optimal by assignment: with the handful of
    faces a clip contains, the Hungarian algorithm and a greedy pass agree, and
    the greedy one is easier to be sure is deterministic. Pairs are considered
    strongest-overlap first, and equal overlaps resolve by track id and then by
    position, so the same frame associates the same way on every machine.
    """

    pairs: list[tuple[float, int, int]] = []
    for track_index, track in enumerate(tracks):
        if track.last is None:
            continue
        for candidate_index, candidate in enumerate(candidates):
            overlap = intersection_over_union(track.last, candidate)
            if overlap >= threshold:
                pairs.append((overlap, track_index, candidate_index))
    pairs.sort(key=lambda item: (-item[0], tracks[item[1]].track_id, item[2]))

    used_tracks: set[int] = set()
    used_candidates: set[int] = set()
    for _, track_index, candidate_index in pairs:
        if track_index in used_tracks or candidate_index in used_candidates:
            continue
        used_tracks.add(track_index)
        used_candidates.add(candidate_index)
        track = tracks[track_index]
        detection = candidates[candidate_index]
        track.observations.append(Observation(t_ticks=t_ticks, detection=detection))
        track.last = detection
        track.missed = 0
    return [candidate for index, candidate in enumerate(candidates) if index not in used_candidates]


def associate(
    frames: list[tuple[int, list[Detection]]],
    parameters: Parameters,
) -> list[Track]:
    """Turn per-frame detections into tracks.

    `frames` is every sampled frame in time order, including the ones nothing
    was detected in — a frame missing from the list would be indistinguishable
    from a frame in which everybody was absent, and the gap counter needs to
    know the difference.
    """

    open_tracks: list[Track] = []
    closed: list[Track] = []
    next_id = 0

    for t_ticks, detections in frames:
        strong = [item for item in detections if item.score >= parameters.start_score]
        weak = [item for item in detections if item.score < parameters.start_score]

        # First pass: confident detections claim their track before anything
        # else is allowed to.
        leftover_strong = _assign(open_tracks, strong, parameters.match_iou, t_ticks)
        # Second pass: what is left of the weak ones may only *continue* a
        # track that nothing confident claimed, and only at the lower overlap.
        unclaimed = [
            track
            for track in open_tracks
            if not track.observations or track.observations[-1].t_ticks != t_ticks
        ]
        _assign(unclaimed, weak, parameters.recover_iou, t_ticks)

        # Whatever confident detection matched nothing is a new face.
        for detection in leftover_strong:
            track = Track(track_id=next_id)
            track.observations.append(Observation(t_ticks=t_ticks, detection=detection))
            track.last = detection
            next_id += 1
            open_tracks.append(track)

        # Age every track that went unmatched this frame, and close the ones
        # that have been unmatched too long.
        still_open: list[Track] = []
        for track in open_tracks:
            matched_here = bool(track.observations) and track.observations[-1].t_ticks == t_ticks
            if matched_here:
                still_open.append(track)
                continue
            track.missed += 1
            if track.missed > parameters.max_gap_frames:
                closed.append(track)
            else:
                still_open.append(track)
        open_tracks = still_open

    closed.extend(open_tracks)
    published = [track for track in closed if track.seen >= parameters.min_track_frames]
    published.sort(key=lambda track: (track.observations[0].t_ticks, track.track_id))
    return published


def bridge(track: Track, frame_times: list[int]) -> Track:
    """Fill the frames a track was followed through but not seen in.

    A gap the association bridged is a gap the solver may keep aiming across,
    and giving it a box is what stops the crop path lurching back to centre for
    three frames. The filled boxes are marked interpolated, so nothing reads
    them as evidence the face was seen: presence, the mean score, and the gate
    that decides whether to follow this track at all all count only what was
    measured.
    """

    if len(track.observations) < 2:
        return track
    by_time = {item.t_ticks: item for item in track.observations}
    first = track.observations[0].t_ticks
    last = track.observations[-1].t_ticks
    filled: list[Observation] = []
    previous: Observation | None = None
    for at in frame_times:
        if at < first or at > last:
            continue
        found = by_time.get(at)
        if found is not None:
            filled.append(found)
            previous = found
            continue
        following = next(
            (item for item in track.observations if item.t_ticks > at),
            None,
        )
        if previous is None or following is None:
            continue
        span = following.t_ticks - previous.t_ticks
        ratio = 0.0 if span <= 0 else (at - previous.t_ticks) / span
        start, end = previous.detection, following.detection

        def mix(low: float, high: float, ratio: float = ratio) -> float:
            return low + (high - low) * ratio

        filled.append(
            Observation(
                t_ticks=at,
                detection=Detection(
                    x=mix(start.x, end.x),
                    y=mix(start.y, end.y),
                    w=mix(start.w, end.w),
                    h=mix(start.h, end.h),
                    score=mix(start.score, end.score),
                ),
                interpolated=True,
            )
        )
    track.observations = filled
    return track
