//! Who deserves the frame, and when the honest answer is "nobody".
//!
//! The book's focus resolver fuses active-speaker probability, diarization
//! agreement, track continuity, reaction salience and scene intent. Three of
//! those five are not measured at this phase — there is no active-speaker
//! detector, no diarization, and no salience model — so what is left is
//! continuity and how much of the frame a face occupies, which is exactly the
//! single-speaker case this workstream is scoped to.
//!
//! That makes the gate the important part. A stage that picked the best of a
//! bad set would put the camera on whoever happened to be detected most, and
//! the failure would be invisible: a confident-looking crop of the wrong
//! person. So dominance has to be earned against a threshold, and falling short
//! produces a fitted frame **and a sentence saying why**, which is the first
//! thing anyone asks when a clip is not tracking.

use clipmill_contracts::schemas::vision_face_track::{Track, VisionFaceTrack};

/// What the camera decided to follow, or why it decided not to.
#[derive(Clone, Debug, PartialEq)]
pub enum Focus {
    /// One track, clearly ahead of the others and present for enough of the
    /// span to be worth following.
    Track {
        track_id: u64,
        /// Share of the span's frames this face was actually detected in.
        presence: f64,
        /// Mean detector score over those frames.
        score: f64,
    },
    /// No track earned the frame. The reason is not decoration — it is what a
    /// user is owed when a clip comes back centred instead of tracked.
    Fit { reason: FitReason },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FitReason {
    /// The pass ran and found no faces at all.
    NoFaces,
    /// Faces exist, but none overlaps the span being solved.
    NoneInSpan,
    /// The best track is present for too little of the span. A camera that
    /// followed it would spend most of the clip pointing at where somebody
    /// used to be.
    TooIntermittent,
    /// The best track is detected weakly enough that following it would be
    /// following the detector's noise.
    TooUncertain,
    /// Two or more tracks are equally worth following. Choosing between them
    /// needs an active-speaker signal this phase does not measure, and picking
    /// one anyway would be a coin toss presented as a decision.
    Ambiguous,
    /// The face-track document says nobody examined these frames.
    NotAnalyzed,
}

impl FitReason {
    /// The sentence shown beside a fitted frame.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoFaces => "no faces were detected in this recording",
            Self::NoneInSpan => "no face was detected inside this clip",
            Self::TooIntermittent => {
                "the clearest face appears in too little of this clip to follow"
            }
            Self::TooUncertain => "the clearest face is detected too weakly to follow",
            Self::Ambiguous => {
                "two faces are equally present, and choosing between them needs a speaker signal this build does not measure"
            }
            Self::NotAnalyzed => "these frames were never examined for faces",
        }
    }
}

/// The bar a track must clear to earn the frame.
#[derive(Clone, Copy, Debug)]
pub struct FocusGate {
    /// Least share of the span's frames the winner must appear in.
    pub min_presence: f64,
    /// Least mean detector score the winner must carry.
    pub min_score: f64,
    /// How far ahead of the runner-up the winner must be, in presence. Below
    /// this the two are called ambiguous rather than ranked.
    pub min_margin: f64,
}

impl Default for FocusGate {
    /// Chosen against the failure they prevent rather than tuned on a
    /// benchmark, which is what the reframe corpus in W26 is for.
    ///
    /// Presence at 0.6: a face on screen for less than two thirds of a clip
    /// leaves the camera pointing at an empty chair for the rest. Score at 0.5:
    /// `YuNet`'s own operating point, below which its boxes are as often
    /// furniture as faces. Margin at 0.15: two people in conversation trade
    /// presence within a few percent, and a fifteen-point gap is the difference
    /// between an interview and a monologue with an occasional listener.
    fn default() -> Self {
        Self {
            min_presence: 0.6,
            min_score: 0.5,
            min_margin: 0.15,
        }
    }
}

/// How much of `[start, end)` a track is present for, and how well.
fn presence_in_span(track: &Track, start: u64, end: u64, frames_in_span: f64) -> (f64, f64) {
    if frames_in_span <= 0.0 {
        return (0.0, 0.0);
    }
    let mut seen = 0_u32;
    let mut total = 0.0_f64;
    for box_ in &track.boxes {
        if box_.t_ticks < start || box_.t_ticks >= end {
            continue;
        }
        // A bridged box is not evidence the face was seen; it is evidence the
        // solver may keep aiming. Presence counts what was measured.
        if box_.interpolated == Some(true) {
            continue;
        }
        seen += 1;
        total += box_.score;
    }
    if seen == 0 {
        return (0.0, 0.0);
    }
    let seen_count = f64::from(seen);
    ((seen_count / frames_in_span).min(1.0), total / seen_count)
}

/// How many sampled frames fall inside the span, counted from the document's
/// own frame rate rather than from what happened to be detected.
///
/// From the rate rather than from the detections, because the denominator has
/// to be the frames that *could* have shown a face. Counting only detected ones
/// would make every track present in 100% of "its" frames, which is the number
/// the gate exists to disbelieve.
#[allow(
    clippy::cast_precision_loss,
    reason = "ticks and frame rates stay far inside a double's exact integer range"
)]
fn frames_in_span(document: &VisionFaceTrack, start: u64, end: u64) -> f64 {
    let rate = &document.detection.frame_rate;
    if end <= start {
        return 0.0;
    }
    // Ticks are 1/90000 of a second; the rate is frames per second.
    let seconds = (end - start) as f64 / TICKS_PER_SECOND;
    seconds * (rate.num.get() as f64 / rate.den.get() as f64)
}

/// The project's fixed timebase denominator.
const TICKS_PER_SECOND: f64 = 90_000.0;

/// Choose the track the camera follows over one span, or refuse with a reason.
pub fn resolve(document: &VisionFaceTrack, start: u64, end: u64, gate: FocusGate) -> Focus {
    if !document.coverage.analyzed {
        return Focus::Fit {
            reason: FitReason::NotAnalyzed,
        };
    }
    if document.tracks.is_empty() {
        return Focus::Fit {
            reason: FitReason::NoFaces,
        };
    }
    let total_frames = frames_in_span(document, start, end);
    let mut ranked: Vec<(u64, f64, f64)> = document
        .tracks
        .iter()
        .map(|track| {
            let (presence, score) = presence_in_span(track, start, end, total_frames);
            (track.track_id, presence, score)
        })
        .filter(|(_, presence, _)| *presence > 0.0)
        .collect();
    if ranked.is_empty() {
        return Focus::Fit {
            reason: FitReason::NoneInSpan,
        };
    }
    // Presence first, then score, then id: the last is not a tie-break anybody
    // should rely on, but a stable order is what makes the same evidence
    // produce the same crop twice.
    ranked.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(
                right
                    .2
                    .partial_cmp(&left.2)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
            .then(left.0.cmp(&right.0))
    });

    let (track_id, presence, score) = ranked[0];
    if presence < gate.min_presence {
        return Focus::Fit {
            reason: FitReason::TooIntermittent,
        };
    }
    if score < gate.min_score {
        return Focus::Fit {
            reason: FitReason::TooUncertain,
        };
    }
    if let Some(runner_up) = ranked.get(1)
        && presence - runner_up.1 < gate.min_margin
    {
        return Focus::Fit {
            reason: FitReason::Ambiguous,
        };
    }
    Focus::Track {
        track_id,
        presence,
        score,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::{FitReason, Focus, FocusGate, resolve};
    use crate::testing::{document, track};

    const SECOND: u64 = 90_000;

    #[test]
    fn a_single_steady_face_earns_the_frame() {
        let doc = document(vec![track(0, 0, 10, 0.9)]);
        let focus = resolve(&doc, 0, 10 * SECOND, FocusGate::default());
        assert!(matches!(focus, Focus::Track { track_id: 0, .. }));
    }

    /// The failure this gate exists for: a confident-looking crop of somebody
    /// who was barely on screen.
    #[test]
    fn a_face_present_for_a_third_of_the_clip_does_not_earn_it() {
        let doc = document(vec![track(0, 0, 3, 0.95)]);
        assert_eq!(
            resolve(&doc, 0, 10 * SECOND, FocusGate::default()),
            Focus::Fit {
                reason: FitReason::TooIntermittent
            }
        );
    }

    #[test]
    fn a_weakly_detected_face_does_not_earn_it() {
        let doc = document(vec![track(0, 0, 10, 0.3)]);
        assert_eq!(
            resolve(&doc, 0, 10 * SECOND, FocusGate::default()),
            Focus::Fit {
                reason: FitReason::TooUncertain
            }
        );
    }

    /// Two people in conversation. Picking one needs a speaker signal nothing
    /// here measures, and a coin toss presented as a decision is worse than a
    /// fitted frame.
    #[test]
    fn two_equally_present_faces_are_ambiguous_rather_than_ranked() {
        let doc = document(vec![track(0, 0, 10, 0.9), track(1, 0, 10, 0.88)]);
        assert_eq!(
            resolve(&doc, 0, 10 * SECOND, FocusGate::default()),
            Focus::Fit {
                reason: FitReason::Ambiguous
            }
        );
    }

    #[test]
    fn a_clear_winner_beside_a_bit_player_still_earns_the_frame() {
        let doc = document(vec![track(0, 0, 10, 0.9), track(1, 0, 2, 0.9)]);
        assert!(matches!(
            resolve(&doc, 0, 10 * SECOND, FocusGate::default()),
            Focus::Track { track_id: 0, .. }
        ));
    }

    #[test]
    fn faces_outside_the_span_are_not_faces_in_it() {
        let doc = document(vec![track(0, 20, 30, 0.95)]);
        assert_eq!(
            resolve(&doc, 0, 10 * SECOND, FocusGate::default()),
            Focus::Fit {
                reason: FitReason::NoneInSpan
            }
        );
    }

    #[test]
    fn an_empty_document_and_an_unexamined_one_are_different_answers() {
        assert_eq!(
            resolve(&document(vec![]), 0, 10 * SECOND, FocusGate::default()),
            Focus::Fit {
                reason: FitReason::NoFaces
            }
        );
        let mut unexamined = document(vec![]);
        unexamined.coverage.analyzed = false;
        assert_eq!(
            resolve(&unexamined, 0, 10 * SECOND, FocusGate::default()),
            Focus::Fit {
                reason: FitReason::NotAnalyzed
            }
        );
    }

    /// Every refusal says something a person can act on.
    #[test]
    fn every_reason_reads_as_a_sentence() {
        for reason in [
            FitReason::NoFaces,
            FitReason::NoneInSpan,
            FitReason::TooIntermittent,
            FitReason::TooUncertain,
            FitReason::Ambiguous,
            FitReason::NotAnalyzed,
        ] {
            assert!(reason.as_str().len() > 20, "{reason:?} says too little");
        }
    }
}
