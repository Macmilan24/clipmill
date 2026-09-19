//! The edit director.
//!
//! A candidate somebody approved becomes an edit document, and nothing about
//! that step is a judgement. Every decision it looks like the director is making
//! was made upstream and is being *read*: discovery proposed the span, the
//! boundary optimizer chose where to cut, the caption engine decided where lines
//! break, the reframe solver decided whether a face earned the frame. What
//! happens here is assembly.
//!
//! That is why the gate's headline test is a golden. The same candidate, the
//! same boundary and the same evidence must produce the same document byte for
//! byte — not because bytes are precious, but because an editor who approves the
//! same clip twice and gets two different edits has been told the tool is
//! guessing.
//!
//! ## What the director refuses to invent
//!
//! A boundary that is not on the candidate's lattice is refused rather than
//! rounded, because the lattice is what "legal cut" means and a document built
//! off it is a mid-word cut waiting to be discovered in the render.
//!
//! A camera move is proposed only when the reframe gate says a face earned the
//! frame. Otherwise the layout is `Fit` and the document says why in its
//! rationale — a fitted clip with a stated reason is honest; a confident crop of
//! the wrong person is not.
//!
//! ## Both caption groupings reach the document
//!
//! The caption engine produces two groupings of one token array, and the
//! director writes both: the reading cues that every sidecar is written from,
//! and the kinetic cues that are burned into the picture. Writing only one
//! would have made the other a grouping that is computed, stored, and then
//! discarded at the render boundary — and everything a viewer ever sees comes
//! through this document.
//!
//! The two lists index the same span and carry the same words. That is checked
//! rather than assumed, because it is the one property the caption engine's
//! whole shape exists to guarantee and the one a bug here would quietly undo.

pub mod lattice;
mod placement;

use clipmill_captions::{DeriveRequest, Inputs};
use clipmill_contracts::schemas::{
    discovery_candidates::{Candidate, DiscoveryCandidates},
    evidence_shots::EvidenceShots,
    index_transcript::IndexTranscript,
    ranking_set::{Ranked, RankingSet},
    speech_transcript::SpeechTranscript,
    vision_face_track::VisionFaceTrack,
};
use clipmill_edit_ir::{
    CropKeyframe, CropRect, EditDocument, Layout, LayoutState, Rationale, VideoSegment,
};
use clipmill_reframe::{FocusGate, Weights as CropWeights};
use clipmill_render::captions::{Intent, project};
use thiserror::Error;

pub use lattice::{Boundary, Duration, Edge, Lattice, SnapError, is_legal, nearest, snap};

/// The implementation the produced document was assembled by.
pub const IMPLEMENTATION: &str = "clipmill-director@1.1.0";
/// The one segment a directed clip has. Named rather than generated: an id that
/// changed run to run would make two identical edits different documents.
const SEGMENT_ID: &str = "seg_1";

/// The published documents the director reads.
#[derive(Clone, Copy, Debug)]
pub struct Evidence<'a> {
    pub candidates: &'a DiscoveryCandidates,
    pub ranking: &'a RankingSet,
    pub transcript: &'a SpeechTranscript,
    /// Sentence boundaries and salient terms for the captions. Absent is a
    /// weaker cue set, not a missing one.
    pub index: Option<&'a IndexTranscript>,
    /// Where the picture changes, so no cue spans a cut.
    pub shots: Option<&'a EvidenceShots>,
    /// Absent when nothing looked for faces, which is a fitted frame with a
    /// different reason from "nobody earned it".
    pub faces: Option<&'a VisionFaceTrack>,
}

/// The source frame the crop rectangles are measured in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Frame {
    pub width: i64,
    pub height: i64,
}

/// The vertical output the crop is being fitted to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Aspect {
    pub width: u32,
    pub height: u32,
}

impl Default for Aspect {
    fn default() -> Self {
        Self {
            width: 9,
            height: 16,
        }
    }
}

/// Which cut to build from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cut {
    /// What the boundary optimizer chose.
    Chosen,
    /// Its runner-up, which the Inspector offers one click away because the
    /// optimizer's second choice is frequently the editor's first.
    Alternative,
    /// A pair the user put there, which must still be on the lattice.
    Exact(Boundary),
}

/// What the director was asked for.
#[derive(Clone, Debug)]
pub struct Request {
    pub candidate_id: String,
    pub cut: Cut,
    pub style_ref: String,
    pub frame: Frame,
    pub aspect: Aspect,
}

#[derive(Debug, Error)]
pub enum DirectError {
    #[error("no candidate in this cohort is called {0}")]
    UnknownCandidate(String),
    #[error("candidate {0} was proposed but never ranked, so it has no boundary")]
    Unranked(String),
    #[error("candidate {0} has no boundary alternative — its lattice offered one legal pair")]
    NoAlternative(String),
    #[error("that boundary is not one this candidate's lattice offers")]
    IllegalBoundary,
    #[error("the frame has no extent to crop inside")]
    EmptyFrame,
    #[error("the requested source span must be nonempty and inside the transcript coverage")]
    InvalidSpan,
    #[error(
        "the {edge} falls inside the word '{word}'. Move {edge} to {seconds} seconds to keep the whole word"
    )]
    ClippedManualWord {
        edge: &'static str,
        word: String,
        seconds: String,
    },
    #[error("the documents describe different recordings")]
    Mismatched,
    #[error("captions could not be derived for this span: {0}")]
    Captions(String),
    #[error("the caption style {0} names a preset that does not exist")]
    UnknownStyle(String),
}

/// Assemble the edit document for an approved candidate.
pub fn direct(evidence: Evidence<'_>, request: &Request) -> Result<EditDocument, DirectError> {
    validate_evidence(evidence, request)?;

    let candidate = evidence
        .candidates
        .candidates
        .iter()
        .find(|item| item.id.as_str() == request.candidate_id)
        .ok_or_else(|| DirectError::UnknownCandidate(request.candidate_id.clone()))?;
    // The cohort holds the score cards; `selected` is only the ids chosen to
    // show. A candidate can be worth directing without having made that cut.
    let ranked = evidence
        .ranking
        .cohort
        .iter()
        .chain(&evidence.ranking.declined)
        .find(|item| item.candidate_id.as_str() == request.candidate_id)
        .ok_or_else(|| DirectError::Unranked(request.candidate_id.clone()))?;

    let boundary = resolve(ranked, request)?;
    let duration = Duration {
        min_ticks: as_i64(evidence.candidates.duration_target.min_ticks.get()),
        max_ticks: as_i64(evidence.candidates.duration_target.max_ticks.get()),
    };
    let (starts, ends) = points(candidate);
    if !is_legal(
        Lattice {
            starts: &starts,
            ends: &ends,
        },
        boundary,
        duration,
    ) {
        return Err(DirectError::IllegalBoundary);
    }

    let mut decisions = vec![cut_sentence(request.cut, boundary)];
    if evidence
        .ranking
        .declined
        .iter()
        .any(|row| row.candidate_id == ranked.candidate_id)
    {
        decisions.push(format!(
            "Human override of an editorial decline: {}",
            ranked
                .review
                .as_ref()
                .map_or("review the source context", |review| review
                    .summary
                    .as_deref()
                    .unwrap_or("review the source context"))
        ));
    }
    assemble_span(
        evidence,
        request,
        boundary,
        Some(request.candidate_id.clone()),
        decisions,
    )
}

/// A human-selected source span remains editable even when analysis recommended none.
pub fn direct_span(
    evidence: Evidence<'_>,
    request: &Request,
    boundary: Boundary,
) -> Result<EditDocument, DirectError> {
    validate_evidence(evidence, request)?;
    let coverage = &evidence.transcript.coverage;
    if boundary.start_ticks < as_i64(coverage.start_ticks)
        || boundary.end_ticks <= boundary.start_ticks
        || boundary.end_ticks > as_i64(coverage.end_ticks)
    {
        return Err(DirectError::InvalidSpan);
    }
    for word in &evidence.transcript.words {
        let start = as_i64(word.start_ticks);
        let end = as_i64(word.end_ticks);
        for (edge, tick, suggested) in [
            ("start", boundary.start_ticks, start),
            ("end", boundary.end_ticks, end),
        ] {
            if start < tick && tick < end {
                // The manual form accepts decimal seconds. Round outwards to
                // a millisecond so copying this suggestion keeps the word.
                let millis = if edge == "end" {
                    suggested.saturating_add(89) / 90
                } else {
                    suggested / 90
                };
                return Err(DirectError::ClippedManualWord {
                    edge,
                    word: word.text.to_string(),
                    seconds: format!("{}.{:03}", millis / 1000, millis % 1000),
                });
            }
        }
    }
    assemble_span(
        evidence,
        request,
        boundary,
        None,
        vec!["Manual source span; not an editorial recommendation.".to_owned()],
    )
}

fn validate_evidence(evidence: Evidence<'_>, request: &Request) -> Result<(), DirectError> {
    let fingerprint = evidence.candidates.source_fingerprint.as_str();
    if evidence.ranking.source_fingerprint.as_str() != fingerprint
        || evidence.transcript.source_fingerprint.as_str() != fingerprint
        || evidence
            .faces
            .is_some_and(|faces| faces.source_fingerprint.as_str() != fingerprint)
        || evidence
            .shots
            .is_some_and(|shots| shots.source_fingerprint.as_str() != fingerprint)
    {
        return Err(DirectError::Mismatched);
    }
    if request.frame.width <= 0
        || request.frame.height <= 0
        || request.aspect.width == 0
        || request.aspect.height == 0
    {
        return Err(DirectError::EmptyFrame);
    }

    Ok(())
}

fn assemble_span(
    evidence: Evidence<'_>,
    request: &Request,
    boundary: Boundary,
    candidate_id: Option<String>,
    mut decisions: Vec<String>,
) -> Result<EditDocument, DirectError> {
    let segments = direct_shots(evidence, boundary, request, &mut decisions);
    let mut captions = caption_track(evidence, boundary, request)?;
    let lane = placement::caption_lane(&segments, evidence.faces, request);
    for cue in captions.cues.iter_mut().chain(&mut captions.burn_in) {
        cue.region = lane;
    }
    if lane != clipmill_edit_ir::CaptionRegion::LowerSafe {
        decisions.push(
            "Captions use one stable lane chosen around the composed face positions.".to_owned(),
        );
    }
    decisions.push(format!(
        "{} cues to read and {} to watch, grouped from the same words.",
        captions.cues.len(),
        captions.burned().len(),
    ));
    let mut document = EditDocument {
        video: clipmill_edit_ir::VideoTrack { segments },
        captions,
        ..EditDocument::default()
    };
    // The audio track's defaults are the delivery targets, and the director has
    // nothing to add to them: -14 LUFS is what the platforms normalize to, and
    // a director that restated it would be a second place for it to drift.
    document.rationale = Some(Rationale {
        candidate_id,
        decisions,
    });
    Ok(document)
}

fn direct_shots(
    evidence: Evidence<'_>,
    boundary: Boundary,
    request: &Request,
    decisions: &mut Vec<String>,
) -> Vec<VideoSegment> {
    let fingerprint = evidence.candidates.source_fingerprint.as_str();
    let mut cuts = vec![boundary.start_ticks, boundary.end_ticks];
    if let Some(shots) = evidence.shots {
        cuts.extend(
            shots
                .cuts
                .iter()
                .map(|cut| as_i64(cut.t_ticks))
                .filter(|cut| *cut > boundary.start_ticks && *cut < boundary.end_ticks),
        );
    }
    cuts.sort_unstable();
    cuts.dedup();
    cuts.windows(2)
        .enumerate()
        .map(|(index, span)| {
            let shot = Boundary {
                start_ticks: span[0],
                end_ticks: span[1],
            };
            let (layout, camera) = layout_for(evidence.faces, shot, request);
            decisions.push(camera);
            VideoSegment {
                segment_id: if index == 0 {
                    SEGMENT_ID.to_owned()
                } else {
                    format!("seg_{}", index + 1)
                },
                source_fingerprint: fingerprint.to_owned(),
                in_ticks: shot.start_ticks,
                out_ticks: shot.end_ticks,
                layout,
            }
        })
        .collect()
}

/// The boundary the request names, as ticks.
fn resolve(ranked: &Ranked, request: &Request) -> Result<Boundary, DirectError> {
    match request.cut {
        Cut::Chosen => Ok(Boundary {
            start_ticks: as_i64(ranked.boundary.chosen.start_ticks),
            end_ticks: as_i64(ranked.boundary.chosen.end_ticks),
        }),
        Cut::Alternative => {
            let alternative = ranked
                .boundary
                .alternative
                .as_ref()
                .ok_or_else(|| DirectError::NoAlternative(request.candidate_id.clone()))?;
            Ok(Boundary {
                start_ticks: as_i64(alternative.interval.start_ticks),
                end_ticks: as_i64(alternative.interval.end_ticks),
            })
        }
        Cut::Exact(boundary) => Ok(boundary),
    }
}

/// The candidate's lattice as two sorted lists.
fn points(candidate: &Candidate) -> (Vec<i64>, Vec<i64>) {
    let mut starts: Vec<i64> = candidate
        .boundary_lattice
        .starts
        .iter()
        .map(|at| as_i64(*at))
        .collect();
    let mut ends: Vec<i64> = candidate
        .boundary_lattice
        .ends
        .iter()
        .map(|at| as_i64(*at))
        .collect();
    starts.sort_unstable();
    starts.dedup();
    ends.sort_unstable();
    ends.dedup();
    (starts, ends)
}

/// Whether the camera follows anybody, and the sentence saying so.
///
/// The reframe gate is the authority and this is not a second opinion on it: a
/// refusal comes back with its own reason, and the director's only job is to
/// carry that reason into the document rather than replace it with silence.
fn layout_for(
    faces: Option<&VisionFaceTrack>,
    boundary: Boundary,
    request: &Request,
) -> (Layout, String) {
    let Some(document) = faces else {
        return (
            Layout::default(),
            "Fitted, because nothing looked for faces in this recording.".to_owned(),
        );
    };
    if boundary.duration_ticks() < 90_000 {
        return (
            Layout::default(),
            "Fitted for this rapid camera cut; inspect composition in the rendered preview."
                .to_owned(),
        );
    }
    if let Some(pair) = clipmill_reframe::resolve_pair(
        document,
        as_u64(boundary.start_ticks),
        as_u64(boundary.end_ticks),
    ) {
        let mut half = request.clone();
        half.aspect.width = half.aspect.width.saturating_mul(2);
        let paths: Option<Vec<_>> = pair
            .iter()
            .map(|id| {
                let mut one = document.clone();
                one.tracks.retain(|track| track.track_id == *id);
                solve_layout(&one, boundary, &half)
            })
            .collect();
        if let Some(mut paths) = paths {
            let lower = paths.pop();
            let upper = paths.pop();
            if let (Some(upper), Some(lower)) = (upper, lower) {
                return (Layout {
                    state: LayoutState::TwoUp,
                    crop_path: upper,
                    secondary_crop_path: lower,
                }, "Two people remain visible in equal portraits, ordered left-to-right from the source; no speaking-person guess.".to_owned());
            }
        }
    }
    if let Some(crop_path) = solve_layout(document, boundary, request) {
        let sentence = format!(
            "Following the single clear face with a steady-size crop across {} keyframes.",
            crop_path.len()
        );
        return (
            Layout {
                state: LayoutState::SpeakerFill,
                crop_path,
                secondary_crop_path: Vec::new(),
            },
            sentence,
        );
    }
    (Layout::default(), "Fitted because face evidence does not support a reliable one- or two-person crop; inspect the rendered composition.".to_owned())
}

fn solve_layout(
    document: &VisionFaceTrack,
    boundary: Boundary,
    request: &Request,
) -> Option<Vec<CropKeyframe>> {
    let path = clipmill_reframe::solve_in_frame(
        document,
        as_u64(boundary.start_ticks),
        as_u64(boundary.end_ticks),
        clipmill_reframe::FrameGeometry {
            source_width: u32::try_from(request.frame.width).ok()?,
            source_height: u32::try_from(request.frame.height).ok()?,
            output_width: request.aspect.width,
            output_height: request.aspect.height,
        },
        CropWeights::default(),
        FocusGate::default(),
    )
    .ok()?;
    if path.fit || path.containment < 0.95 {
        return None;
    }
    let frames: Vec<CropKeyframe> = path
        .keyframes
        .iter()
        .map(|keyframe| CropKeyframe {
            t_ticks: as_i64(keyframe.t_ticks) - boundary.start_ticks,
            rect: rect_of(*keyframe, request),
        })
        .collect();
    Some(frames)
}

/// A normalized keyframe as a rectangle of source pixels.
///
/// The height is what the solver decided and the width follows the output
/// aspect, because a crop whose own aspect differed from the output's would be
/// re-fitted by the renderer and the camera move would not be the one solved.
fn rect_of(keyframe: clipmill_reframe::Keyframe, request: &Request) -> CropRect {
    let frame = request.frame;
    let target_height = round(keyframe.scale * as_f64(frame.height)).clamp(2, frame.height);
    // Width is nearest, not always floored: one-source-pixel aspect tolerance.
    let mut height = target_height - target_height % 2;
    let mut width = 2 * round(
        as_f64(height) * f64::from(request.aspect.width) / f64::from(request.aspect.height) / 2.0,
    );
    if width > frame.width {
        width = frame.width - frame.width % 2;
        height = round(
            as_f64(width) * f64::from(request.aspect.height) / f64::from(request.aspect.width),
        );
    }
    let width = width.max(2);
    let height = height.max(2);
    let x = round(keyframe.center_x * as_f64(frame.width)) - width / 2;
    let y = round(keyframe.center_y * as_f64(frame.height)) - height / 2;
    CropRect {
        x: x.clamp(0, (frame.width - width).max(0)),
        y: y.clamp(0, (frame.height - height).max(0)),
        width,
        height,
    }
}

/// The cues for this span, in program time.
///
/// Derived here rather than read from a published caption artifact, and the
/// reason is the span. A cue may not cross the edge of its window, so cues
/// segmented over the whole recording are not the cues this clip should carry —
/// and publishing an artifact per boundary would mint a cache entry every time
/// somebody nudged a handle.
fn caption_track(
    evidence: Evidence<'_>,
    boundary: Boundary,
    request: &Request,
) -> Result<clipmill_edit_ir::CaptionTrack, DirectError> {
    let mut derive = DeriveRequest::new(IMPLEMENTATION);
    derive.span = Some(clipmill_captions::Span {
        start_ticks: boundary.start_ticks,
        end_ticks: boundary.end_ticks,
    });
    let cues = match clipmill_captions::derive(
        evidence.transcript,
        evidence.index,
        evidence.shots,
        Inputs {
            // The director assembles rather than publishes, so the addresses it
            // states are the ones it read.
            transcript_artifact_id: evidence.transcript.source_fingerprint.as_str(),
            index_artifact_id: None,
            shots_artifact_id: None,
        },
        &derive,
    ) {
        Ok(document) => document,
        // A span nobody spoke in is a clip with no captions, not a failure.
        Err(clipmill_captions::DeriveError::NoWords) => {
            return Ok(clipmill_edit_ir::CaptionTrack {
                style_ref: request.style_ref.clone(),
                cues: Vec::new(),
                burn_in: Vec::new(),
            });
        }
        Err(error) => return Err(DirectError::Captions(error.to_string())),
    };
    let reading = project(
        &cues,
        Intent::Accessibility,
        &request.style_ref,
        boundary.start_ticks,
    )
    .map_err(|_| DirectError::UnknownStyle(request.style_ref.clone()))?;
    let kinetic = project(
        &cues,
        Intent::BurnIn,
        &request.style_ref,
        boundary.start_ticks,
    )
    .map_err(|_| DirectError::UnknownStyle(request.style_ref.clone()))?;
    Ok(clipmill_edit_ir::CaptionTrack {
        style_ref: reading.style_ref,
        cues: reading.cues,
        burn_in: kinetic.cues,
    })
}

fn cut_sentence(cut: Cut, boundary: Boundary) -> String {
    let seconds = as_f64(boundary.duration_ticks()) / 90_000.0;
    let source = match cut {
        Cut::Chosen => "the boundary the search chose",
        Cut::Alternative => "the search's runner-up boundary",
        Cut::Exact(_) => "a boundary set by hand, snapped to the lattice",
    };
    format!("Cut at {source}: {seconds:.2}s.")
}

fn as_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn as_u64(value: i64) -> u64 {
    u64::try_from(value).unwrap_or(0)
}

#[allow(
    clippy::cast_precision_loss,
    reason = "a pixel count or a tick span, both far inside exact double range"
)]
fn as_f64(value: i64) -> f64 {
    value as f64
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "a pixel coordinate, clamped into the frame by the caller"
)]
fn round(value: f64) -> i64 {
    value.round() as i64
}
