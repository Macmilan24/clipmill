//! Framing that follows whoever is talking.
//!
//! Inside one camera shot with two people in it, the director used to show
//! both in equal portraits or fit the frame, because nothing said which of
//! them was speaking. The face tracks now carry each face's mouth motion, and
//! a shot is read sentence by sentence: where one face is clearly the one
//! talking, the camera follows that face, and it changes between sentences —
//! never inside one — with the clip's soft cut. Where nobody is clearly
//! talking the shot keeps the framing it would have had.
//!
//! Conservative by construction. A turn shorter than [`MIN_TURN_TICKS`] is
//! folded into the one before it, so the camera never flickers between people
//! on a one-word reply, and a sentence nobody clearly spoke keeps the previous
//! speaker when it is short: a listener's "yeah" is not worth a cut.

use clipmill_contracts::schemas::{
    index_transcript::IndexTranscript, vision_face_track::VisionFaceTrack,
};
use clipmill_reframe::{SpeakerGate, speaker};

use crate::lattice::Boundary;

/// The shortest stretch the camera stays on one person.
const MIN_TURN_TICKS: i64 = 2 * 90_000;
/// An unclear sentence up to this long keeps the previous speaker.
const HOLD_TICKS: i64 = 4 * 90_000;
/// The shortest shot worth reading turn by turn.
const MIN_SHOT_TICKS: i64 = 4 * 90_000;

/// One stretch of a shot, and the face talking through it when one clearly is.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Turn {
    pub span: Boundary,
    pub track: Option<u64>,
}

/// The turns of one shot, or `None` when the shot is better framed whole: one
/// person or none in it, too short, no sentences to cut between, or nobody
/// clearly talking anywhere in it.
pub(crate) fn turns(
    faces: &VisionFaceTrack,
    index: Option<&IndexTranscript>,
    shot: Boundary,
) -> Option<Vec<Turn>> {
    if shot.duration_ticks() < MIN_SHOT_TICKS || people_in(faces, shot) < 2 {
        return None;
    }
    let mut points = vec![shot.start_ticks];
    points.extend(sentence_breaks(index?, shot));
    points.push(shot.end_ticks);
    points.dedup();

    let mut read: Vec<Turn> = points
        .windows(2)
        .map(|pair| {
            let span = Boundary {
                start_ticks: pair[0],
                end_ticks: pair[1],
            };
            Turn {
                span,
                track: speaker(
                    faces,
                    as_u64(span.start_ticks),
                    as_u64(span.end_ticks),
                    SpeakerGate::default(),
                ),
            }
        })
        .collect();
    if read.iter().all(|turn| turn.track.is_none()) {
        return None;
    }
    // A short sentence nobody clearly spoke keeps whoever spoke before it.
    for at in 1..read.len() {
        if read[at].track.is_none() && read[at].span.duration_ticks() <= HOLD_TICKS {
            read[at].track = read[at - 1].track;
        }
    }
    // Folding a short turn into a long unclear one can leave nobody followed.
    let merged = merge(read);
    merged
        .iter()
        .any(|turn| turn.track.is_some())
        .then_some(merged)
}

/// Consecutive turns by the same face become one, and a turn too short to
/// hold the camera joins the one before it (or, first of all, the one after).
fn merge(read: Vec<Turn>) -> Vec<Turn> {
    let mut merged: Vec<Turn> = Vec::with_capacity(read.len());
    for turn in read {
        match merged.last_mut() {
            Some(last) if last.track == turn.track => last.span.end_ticks = turn.span.end_ticks,
            Some(last) if turn.span.duration_ticks() < MIN_TURN_TICKS => {
                last.span.end_ticks = turn.span.end_ticks;
            }
            _ => merged.push(turn),
        }
    }
    if merged.len() > 1 && merged[0].span.duration_ticks() < MIN_TURN_TICKS {
        let first = merged.remove(0);
        merged[0].span.start_ticks = first.span.start_ticks;
    }
    // Folding can leave two neighbours on the same face; join them too.
    let mut joined: Vec<Turn> = Vec::with_capacity(merged.len());
    for turn in merged {
        match joined.last_mut() {
            Some(last) if last.track == turn.track => last.span.end_ticks = turn.span.end_ticks,
            _ => joined.push(turn),
        }
    }
    joined
}

/// Where a sentence ends and the next begins, inside the shot: the middle of
/// the gap between them, so neither sentence is cut.
fn sentence_breaks(index: &IndexTranscript, shot: Boundary) -> Vec<i64> {
    index
        .sentences
        .windows(2)
        .filter_map(|pair| {
            let end = as_i64(pair[0].end_ticks);
            let start = as_i64(pair[1].start_ticks);
            let at = end + (start - end).max(0) / 2;
            (at > shot.start_ticks && at < shot.end_ticks).then_some(at)
        })
        .collect()
}

/// How many faces are present for at least half the shot.
fn people_in(faces: &VisionFaceTrack, shot: Boundary) -> usize {
    let (start, end) = (as_u64(shot.start_ticks), as_u64(shot.end_ticks));
    let rate = &faces.detection.frame_rate;
    #[allow(
        clippy::cast_precision_loss,
        reason = "ticks and frame rates stay far inside a double's exact integer range"
    )]
    let frames = (end.saturating_sub(start)) as f64 / 90_000.0 * rate.num.get() as f64
        / rate.den.get() as f64;
    faces
        .tracks
        .iter()
        .filter(|track| {
            let seen = track
                .boxes
                .iter()
                .filter(|b| b.t_ticks >= start && b.t_ticks < end && b.interpolated != Some(true))
                .count();
            #[allow(
                clippy::cast_precision_loss,
                reason = "sample count is bounded by recording length"
            )]
            let share = seen as f64 / frames.max(1.0);
            share >= 0.5
        })
        .count()
}

fn as_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn as_u64(value: i64) -> u64 {
    u64::try_from(value.max(0)).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{MIN_TURN_TICKS, Turn, merge};
    use crate::lattice::Boundary;

    fn turn(start: i64, end: i64, track: Option<u64>) -> Turn {
        Turn {
            span: Boundary {
                start_ticks: start,
                end_ticks: end,
            },
            track,
        }
    }

    const S: i64 = 90_000;

    #[test]
    fn a_one_word_reply_does_not_move_the_camera() {
        let merged = merge(vec![
            turn(0, 6 * S, Some(0)),
            turn(6 * S, 7 * S, Some(1)),
            turn(7 * S, 12 * S, Some(0)),
        ]);
        assert_eq!(merged, vec![turn(0, 12 * S, Some(0))]);
    }

    #[test]
    fn real_turns_stay_turns() {
        let merged = merge(vec![
            turn(0, 5 * S, Some(0)),
            turn(5 * S, 9 * S, Some(1)),
            turn(9 * S, 14 * S, Some(0)),
        ]);
        assert_eq!(merged.len(), 3);
        assert!(
            merged
                .iter()
                .all(|turn| turn.span.duration_ticks() >= MIN_TURN_TICKS)
        );
    }

    #[test]
    fn a_short_opening_joins_what_follows() {
        let merged = merge(vec![turn(0, S, Some(1)), turn(S, 8 * S, Some(0))]);
        assert_eq!(merged, vec![turn(0, 8 * S, Some(0))]);
    }
}
