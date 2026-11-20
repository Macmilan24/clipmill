//! Assemble a clip planned as parts (decision R76) into an edit document.
//!
//! A plan names ranges anywhere in one recording, in programme order, with
//! what was removed from each and how each joins the one before. The plan is
//! checked by `clipmill-mind` first; then every kept range is directed the way
//! a single span is (shots, who is speaking, the followed crop), its captions
//! are derived for exactly that range and moved to where it plays, and the
//! joins are recorded on the segments, with a change of framing where the
//! plan asks for one so a cut on one camera reads as intended.

use std::collections::BTreeSet;

use clipmill_contracts::schemas::{
    evidence_shots::EvidenceShots,
    index_transcript::IndexTranscript,
    mind_clip_plan::{
        JoinFraming, JoinKind as PlannedJoin, MindClipPlan, MindClipPlanForm, PartRole as Role,
    },
    speech_transcript::SpeechTranscript,
    vision_face_track::VisionFaceTrack,
};
use clipmill_edit_ir::{
    CaptionCue, CaptionTrack, EditDocument, Join, JoinKind, LayoutState, PartRef, PartRole, Plan,
    PlanForm, Punch, Rationale, RemovedRange, VideoSegment,
};

use crate::{
    Boundary, Cut, DirectError, FrameShape, Request, Sources, captions_for_span, placement,
};

/// How much closer a section moves when a join changes the framing, in
/// percent: enough to read as a new shot, not so much that a face leaves it.
pub const JOIN_PUNCH_ZOOM: u16 = 115;

/// The audio blend over a cut inside one part, where a removed passage would
/// otherwise click or swallow a breath: forty milliseconds.
pub const REMOVED_CROSSFADE_TICKS: i64 = 3_600;

/// What the director reads to assemble a plan.
#[derive(Clone, Copy, Debug)]
pub struct PlanEvidence<'a> {
    pub transcript: &'a SpeechTranscript,
    pub index: Option<&'a IndexTranscript>,
    pub shots: Option<&'a EvidenceShots>,
    pub faces: Option<&'a VisionFaceTrack>,
    /// Where a cut may go, when the recording's pauses were measured: every
    /// edge of the plan must then sit on one of its cut points.
    pub lattice: Option<&'a clipmill_mind::lattice::Lattice>,
}

/// What the director was asked to make of a plan.
#[derive(Clone, Debug)]
pub struct PlanRequest<'a> {
    pub plan: &'a MindClipPlan,
    pub style_ref: String,
    pub frame: crate::Frame,
    pub shape: FrameShape,
}

/// One kept range of the source, in programme order.
#[derive(Clone, Debug)]
struct Range {
    span: Boundary,
    part: PartRef,
    /// How the range joins the one before it; `None` for the first.
    join: Option<Join>,
    /// Move in closer for this range, so the cut before it reads as a new shot.
    punch_in: bool,
}

/// Assemble the edit document for a plan, after checking it.
pub fn direct_plan(
    evidence: PlanEvidence<'_>,
    request: &PlanRequest<'_>,
) -> Result<EditDocument, DirectError> {
    check(evidence, request)?;
    let plan = request.plan;
    let ranges = ranges(plan, evidence.transcript);
    // The single-span director's request shape: frame, shape and style are
    // all the shared helpers read from it.
    let single = Request {
        candidate_id: plan.plan_id.to_string(),
        cut: Cut::Exact(ranges[0].span),
        style_ref: request.style_ref.clone(),
        frame: request.frame,
        shape: request.shape,
    };
    assemble(evidence, request, &ranges, &single)
}

/// The plan is for this recording, the frame has an extent, and every rule in
/// `clipmill-mind` holds; otherwise every reason it cannot be made.
fn check(evidence: PlanEvidence<'_>, request: &PlanRequest<'_>) -> Result<(), DirectError> {
    let plan = request.plan;
    let fingerprint = plan.source_fingerprint.as_str();
    if evidence.transcript.source_fingerprint.as_str() != fingerprint
        || evidence
            .index
            .is_some_and(|index| index.source_fingerprint.as_str() != fingerprint)
        || evidence
            .shots
            .is_some_and(|shots| shots.source_fingerprint.as_str() != fingerprint)
        || evidence
            .faces
            .is_some_and(|faces| faces.source_fingerprint.as_str() != fingerprint)
    {
        return Err(DirectError::Mismatched);
    }
    if request.frame.width <= 0 || request.frame.height <= 0 {
        return Err(DirectError::EmptyFrame);
    }
    let words: Vec<(u64, u64)> = evidence
        .transcript
        .words
        .iter()
        .map(|word| (word.start_ticks, word.end_ticks))
        .collect();
    let violations = clipmill_mind::check(
        plan,
        &clipmill_mind::Bounds {
            words: &words,
            lattice: evidence.lattice,
            duration: None,
            source_ticks: evidence.transcript.coverage.end_ticks,
        },
    );
    if !violations.is_empty() {
        return Err(DirectError::InvalidPlan(
            violations
                .iter()
                .map(|violation| violation.message.as_str())
                .collect::<Vec<_>>()
                .join(" "),
        ));
    }
    Ok(())
}

/// Direct each range, move its captions to where it plays, and record the
/// plan on the document.
fn assemble(
    evidence: PlanEvidence<'_>,
    request: &PlanRequest<'_>,
    ranges: &[Range],
    single: &Request,
) -> Result<EditDocument, DirectError> {
    let plan = request.plan;
    let fingerprint = plan.source_fingerprint.as_str();
    let sources = Sources {
        fingerprint,
        index: evidence.index,
        shots: evidence.shots,
        faces: evidence.faces,
    };
    let mut decisions = vec![summary(plan)];
    let mut segments: Vec<VideoSegment> = Vec::new();
    let mut cues: Vec<CaptionCue> = Vec::new();
    let mut burn_in: Vec<CaptionCue> = Vec::new();
    let mut style: Option<(String, clipmill_edit_ir::CaptionOptions)> = None;
    let mut used_words: BTreeSet<String> = BTreeSet::new();
    let mut offset: i64 = 0;
    for (position, range) in ranges.iter().enumerate() {
        let mut camera = Vec::new();
        let mut directed = crate::direct_shots_in(sources, range.span, single, &mut camera);
        if position == 0 {
            decisions.extend(camera);
        }
        for (index, segment) in directed.iter_mut().enumerate() {
            segment.part = Some(range.part.clone());
            segment.join = if index == 0 { range.join } else { None };
            if range.punch_in {
                closer(segment);
            }
        }
        segments.extend(directed);

        let track = captions_for_span(
            evidence.transcript,
            evidence.index,
            evidence.shots,
            range.span,
            &request.style_ref,
        )?;
        if style.is_none() {
            style = Some((track.style_ref.clone(), track.options.clone()));
        }
        let placed = Placed {
            prefix: format!("r{}", position + 1),
            part_id: range.part.part_id.as_str(),
            offset,
        };
        placed.add(track, &mut used_words, &mut cues, &mut burn_in);
        offset += range.span.duration_ticks();
    }
    for (index, segment) in segments.iter_mut().enumerate() {
        segment.segment_id = format!("seg_{}", index + 1);
    }
    let lane = placement::caption_lane(&segments, evidence.faces, single);
    for cue in cues.iter_mut().chain(&mut burn_in) {
        cue.region = lane;
    }
    let (style_ref, options) = style.unwrap_or_else(|| {
        (
            request.style_ref.clone(),
            clipmill_edit_ir::CaptionOptions::default(),
        )
    });

    let mut document = EditDocument {
        video: clipmill_edit_ir::VideoTrack {
            shape: request.shape,
            segments,
            transition_ticks: 0,
            cutaways: Vec::new(),
        },
        captions: CaptionTrack {
            style_ref,
            options,
            cues,
            burn_in,
        },
        ..EditDocument::default()
    };
    document.plan = Some(plan_record(plan, fingerprint));
    document.title = plan.title.as_ref().map(|title| title.as_str().to_owned());
    document.rationale = Some(Rationale {
        candidate_id: None,
        decisions,
    });
    document
        .validate()
        .map_err(|error| DirectError::InvalidPlan(error.to_string()))?;
    Ok(document)
}

fn plan_record(plan: &MindClipPlan, fingerprint: &str) -> Plan {
    Plan {
        plan_id: plan.plan_id.to_string(),
        form: form(plan.form),
        removed: plan
            .parts
            .iter()
            .flat_map(|part| {
                part.removed.iter().map(move |range| RemovedRange {
                    part_id: Some(part.part_id.to_string()),
                    source_fingerprint: fingerprint.to_owned(),
                    in_ticks: crate::as_i64(range.start_ticks),
                    out_ticks: crate::as_i64(range.end_ticks.get()),
                    reason: range.reason.to_string(),
                })
            })
            .collect(),
    }
}

/// Every kept range of the plan in programme order: each part split around
/// what was removed from it, with the joins between them.
fn ranges(plan: &MindClipPlan, transcript: &SpeechTranscript) -> Vec<Range> {
    let mut ranges = Vec::new();
    for (index, part) in plan.parts.iter().enumerate() {
        let reference = PartRef {
            part_id: part.part_id.to_string(),
            role: role(part.role),
        };
        let planned = index
            .checked_sub(1)
            .and_then(|before| plan.joins.get(before));
        let mut start = crate::as_i64(part.start_ticks);
        let mut pieces = Vec::new();
        for removed in &part.removed {
            pieces.push((start, crate::as_i64(removed.start_ticks)));
            start = crate::as_i64(removed.end_ticks.get());
        }
        pieces.push((start, crate::as_i64(part.end_ticks.get())));
        for (piece, (start_ticks, end_ticks)) in pieces.into_iter().enumerate() {
            let (join, punch_in) = if piece > 0 {
                // A cut inside one moment, on the same camera: alternate the
                // framing so the jump reads as a deliberate cut.
                (
                    Some(Join {
                        kind: JoinKind::Removed,
                        audio_crossfade_ticks: REMOVED_CROSSFADE_TICKS,
                        room_tone: false,
                        room_tone_range: None,
                    }),
                    piece % 2 == 1,
                )
            } else if let Some(planned) = planned {
                (
                    Some(Join {
                        kind: match planned.kind {
                            PlannedJoin::Adjacent => JoinKind::Adjacent,
                            PlannedJoin::Reorder => JoinKind::Reorder,
                            PlannedJoin::Distant => JoinKind::Distant,
                        },
                        audio_crossfade_ticks: planned
                            .crossfade_ms
                            .map_or(0, |ms| ms.clamp(0, 500) * 90),
                        room_tone: planned.room_tone.unwrap_or(false),
                        room_tone_range: planned
                            .room_tone
                            .unwrap_or(false)
                            .then(|| {
                                quiet_range(
                                    transcript,
                                    start_ticks,
                                    planned.crossfade_ms.unwrap_or(0).clamp(0, 500) * 90,
                                )
                            })
                            .flatten(),
                    }),
                    planned.framing == Some(JoinFraming::PunchIn),
                )
            } else {
                (None, false)
            };
            ranges.push(Range {
                span: Boundary {
                    start_ticks,
                    end_ticks,
                },
                part: reference.clone(),
                join,
                punch_in,
            });
        }
    }
    ranges
}

/// Use the closest verified silence, leaving 40 ms clear of both ends.
fn quiet_range(transcript: &SpeechTranscript, near: i64, requested: i64) -> Option<[i64; 2]> {
    const MARGIN: u64 = 3_600;
    let duration = if requested > 0 {
        requested.cast_unsigned()
    } else {
        5_400
    };
    let near = u64::try_from(near).ok()?;
    transcript
        .silences
        .iter()
        .filter_map(|silence| {
            let low = silence
                .start_ticks
                .max(transcript.coverage.start_ticks)
                .checked_add(MARGIN)?;
            let high = silence
                .end_ticks
                .min(transcript.coverage.end_ticks)
                .checked_sub(MARGIN + duration)?;
            if low > high {
                return None;
            }
            let start = near.saturating_sub(duration / 2).clamp(low, high);
            let end = start.checked_add(duration)?;
            let next = transcript
                .words
                .partition_point(|word| word.end_ticks <= start);
            if transcript
                .words
                .get(next)
                .is_some_and(|word| word.start_ticks < end)
                || transcript
                    .invalid_regions
                    .iter()
                    .any(|span| span.start_ticks < end && span.end_ticks > start)
            {
                return None;
            }
            Some((
                start.abs_diff(near),
                [i64::try_from(start).ok()?, i64::try_from(end).ok()?],
            ))
        })
        .min_by_key(|entry| entry.0)
        .map(|entry| entry.1)
}

/// Move a section in closer: a punch across a followed picture, a zoom on a
/// fitted one. Two-person layouts keep both people in frame instead.
fn closer(segment: &mut VideoSegment) {
    match segment.layout.state {
        LayoutState::SpeakerFill if !segment.layout.crop_path.is_empty() => {
            let duration = segment.duration_ticks();
            if duration >= 2 * Punch::MOVE_TICKS {
                segment.layout.punches = vec![Punch {
                    start_ticks: 0,
                    end_ticks: duration,
                    zoom: JOIN_PUNCH_ZOOM,
                }];
            }
        }
        LayoutState::Fit => segment.layout.zoom = Some(JOIN_PUNCH_ZOOM),
        _ => {}
    }
}

/// Where one range's captions go in the programme.
struct Placed<'a> {
    /// Cue ids are unique per range, so a part split by a removed passage
    /// keeps them apart.
    prefix: String,
    part_id: &'a str,
    offset: i64,
}

impl Placed<'_> {
    /// Move a range's cues to where it plays and add them to the clip's. A
    /// word heard twice (a cold open repeats its hook) is two words to the
    /// captions, so the second gets an id of its own.
    fn add(
        &self,
        track: CaptionTrack,
        used_words: &mut BTreeSet<String>,
        cues: &mut Vec<CaptionCue>,
        burn_in: &mut Vec<CaptionCue>,
    ) {
        let renamed = repeated_words(&track, used_words);
        for (list, out) in [(track.cues, &mut *cues), (track.burn_in, &mut *burn_in)] {
            for mut cue in list {
                cue.cue_id = format!("{}_{}", self.prefix, cue.cue_id);
                shift(&mut cue, self.offset);
                for word in cue.lines.iter_mut().flat_map(|line| line.words.iter_mut()) {
                    if let Some(id) = word.word_id.as_mut()
                        && renamed.contains(id.as_str())
                    {
                        *id = format!("{id}~{}", self.part_id);
                    }
                }
                out.push(cue);
            }
        }
        used_words.extend(word_ids(cues));
    }
}

fn shift(cue: &mut CaptionCue, offset: i64) {
    cue.start_ticks += offset;
    cue.end_ticks += offset;
    for word in cue.lines.iter_mut().flat_map(|line| line.words.iter_mut()) {
        word.start_ticks += offset;
        word.end_ticks += offset;
    }
}

fn word_ids(cues: &[CaptionCue]) -> impl Iterator<Item = String> + '_ {
    cues.iter()
        .flat_map(|cue| cue.lines.iter())
        .flat_map(|line| line.words.iter())
        .filter_map(|word| word.word_id.clone())
}

/// The word ids in `track` already used by an earlier range.
fn repeated_words(track: &CaptionTrack, used: &BTreeSet<String>) -> BTreeSet<String> {
    word_ids(&track.cues)
        .chain(word_ids(&track.burn_in))
        .filter(|id| used.contains(id))
        .collect()
}

const fn role(role: Role) -> PartRole {
    match role {
        Role::Hook => PartRole::Hook,
        Role::Setup => PartRole::Setup,
        Role::Question => PartRole::Question,
        Role::Answer => PartRole::Answer,
        Role::Body => PartRole::Body,
        Role::Payoff => PartRole::Payoff,
        Role::Proof => PartRole::Proof,
        Role::Reaction => PartRole::Reaction,
        Role::Item => PartRole::Item,
    }
}

const fn form(form: MindClipPlanForm) -> PlanForm {
    match form {
        MindClipPlanForm::Straight => PlanForm::Straight,
        MindClipPlanForm::Tightened => PlanForm::Tightened,
        MindClipPlanForm::Stitch => PlanForm::Stitch,
        MindClipPlanForm::ColdOpen => PlanForm::ColdOpen,
        MindClipPlanForm::ThemeMontage => PlanForm::ThemeMontage,
    }
}

/// One sentence saying how the clip is built, for the rationale.
fn summary(plan: &MindClipPlan) -> String {
    let removed: usize = plan.parts.iter().map(|part| part.removed.len()).sum();
    let parts = plan.parts.len();
    let kind = match plan.form {
        MindClipPlanForm::Straight => "One continuous moment",
        MindClipPlanForm::Tightened => "One moment, tightened",
        MindClipPlanForm::Stitch => "Stitched from distant moments",
        MindClipPlanForm::ColdOpen => "A cold open: the strongest line first, then the story",
        MindClipPlanForm::ThemeMontage => "A montage of moments on one theme",
    };
    format!(
        "{kind}: {parts} {}, {removed} {} removed.",
        if parts == 1 { "part" } else { "parts" },
        if removed == 1 { "passage" } else { "passages" }
    )
}
