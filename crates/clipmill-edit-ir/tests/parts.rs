//! A clip planned as parts (decision R76): segments remember the part they
//! play and how they join the one before, the document remembers its plan,
//! and a document that never used either is byte-for-byte what it was.
#![allow(clippy::expect_used)]

use clipmill_edit_ir::{
    CropEasing, CropKeyframe, CropRect, DocumentError, EditCommand, EditDocument, Join, JoinKind,
    Layout, LayoutState, PartRef, PartRole, Plan, PlanForm, RemovedRange, VideoSegment,
};

const SOURCE: &str = "sha256:1111111111111111111111111111111111111111111111111111111111111111";
const SECOND: i64 = 90_000;

fn segment(id: &str, in_s: i64, out_s: i64) -> VideoSegment {
    VideoSegment {
        segment_id: id.to_owned(),
        source_fingerprint: SOURCE.to_owned(),
        in_ticks: in_s * SECOND,
        out_ticks: out_s * SECOND,
        layout: Layout {
            state: LayoutState::SpeakerFill,
            crop_path: vec![CropKeyframe {
                t_ticks: 0,
                rect: CropRect {
                    x: 100,
                    y: 0,
                    width: 608,
                    height: 1080,
                },
                easing: CropEasing::Linear,
            }],
            ..Layout::default()
        },
        part: None,
        join: None,
    }
}

/// A question at 12:41, its answer straight after, and a payoff from 29:18.
fn stitched() -> EditDocument {
    let mut document = EditDocument::default();
    let mut question = segment("question", 761, 764);
    question.part = Some(PartRef {
        part_id: "p1".to_owned(),
        role: PartRole::Question,
    });
    let mut answer = segment("answer", 765, 781);
    answer.part = Some(PartRef {
        part_id: "p2".to_owned(),
        role: PartRole::Answer,
    });
    answer.join = Some(Join {
        kind: JoinKind::Adjacent,
        audio_crossfade_ticks: 0,
        room_tone: false,
        room_tone_range: None,
    });
    let mut payoff = segment("payoff", 1758, 1767);
    payoff.part = Some(PartRef {
        part_id: "p3".to_owned(),
        role: PartRole::Payoff,
    });
    payoff.join = Some(Join {
        kind: JoinKind::Distant,
        audio_crossfade_ticks: 5_400,
        room_tone: true,
        room_tone_range: None,
    });
    document.video.segments = vec![question, answer, payoff];
    document.plan = Some(Plan {
        plan_id: "plan_7f3a9c21d04e8b65".to_owned(),
        form: PlanForm::Stitch,
        removed: vec![RemovedRange {
            part_id: None,
            source_fingerprint: SOURCE.to_owned(),
            in_ticks: 781 * SECOND,
            out_ticks: 786 * SECOND,
            reason: "tangent".to_owned(),
        }],
    });
    document.validate().expect("a valid stitched clip");
    document
}

#[test]
fn a_stitched_clip_round_trips_through_its_canonical_form() {
    let document = stitched();
    let bytes = document.to_canonical_json().expect("serializes");
    let read = EditDocument::from_canonical_json(&bytes).expect("reads back");
    assert_eq!(read, document);
    let text = String::from_utf8(bytes).expect("utf-8");
    assert!(text.contains("\"role\":\"payoff\"") && text.contains("\"kind\":\"distant\""));
}

#[test]
fn a_document_without_parts_is_written_exactly_as_before() {
    let mut document = EditDocument::default();
    document.video.segments = vec![segment("section", 1, 2)];
    let text = String::from_utf8(document.to_canonical_json().expect("serializes")).expect("utf-8");
    for key in ["\"part\"", "\"join\"", "\"plan\""] {
        assert!(
            !text.contains(key),
            "{key} written for a clip that has none"
        );
    }
}

#[test]
fn set_join_inverts_exactly() {
    let original = stitched();
    let mut edited = original.clone();
    let inverse = EditCommand::SetJoin {
        segment_id: "payoff".to_owned(),
        join: None,
    }
    .apply(&mut edited)
    .expect("applies");
    assert_eq!(edited.video.segments[2].join, None);
    inverse.apply(&mut edited).expect("inverse applies");
    assert_eq!(
        edited.to_canonical_json().expect("serializes"),
        original.to_canonical_json().expect("serializes")
    );
}

#[test]
fn a_crossfade_longer_than_half_a_second_is_refused() {
    let mut edited = stitched();
    let error = EditCommand::SetJoin {
        segment_id: "payoff".to_owned(),
        join: Some(Join {
            kind: JoinKind::Distant,
            audio_crossfade_ticks: Join::MAX_CROSSFADE_TICKS + 1,
            room_tone: false,
            room_tone_range: None,
        }),
    }
    .apply(&mut edited)
    .expect_err("refused");
    assert!(error.to_string().contains("payoff"), "{error}");
    assert_eq!(edited, stitched(), "a refused command changes nothing");
}

#[test]
fn a_plan_with_an_empty_removed_range_is_refused() {
    let mut document = stitched();
    if let Some(plan) = document.plan.as_mut() {
        plan.removed[0].out_ticks = plan.removed[0].in_ticks;
    }
    assert!(matches!(
        document.validate(),
        Err(DocumentError::InvalidPlan)
    ));
}

#[test]
fn splitting_a_joined_section_leaves_the_join_on_its_head() {
    let original = stitched();
    let mut edited = original.clone();
    let inverse = EditCommand::SplitSegment {
        segment_id: "payoff".to_owned(),
        at_ticks: 1762 * SECOND,
        new_segment_id: "payoff_b".to_owned(),
    }
    .apply(&mut edited)
    .expect("splits");
    let (head, tail) = (&edited.video.segments[2], &edited.video.segments[3]);
    assert_eq!(head.join.map(|join| join.kind), Some(JoinKind::Distant));
    assert_eq!(tail.join, None);
    assert_eq!(tail.part, head.part, "both halves play the same part");
    inverse.apply(&mut edited).expect("inverse applies");
    assert_eq!(edited, original);
}

fn cue(id: &str, text: &str, first: i64, last: i64) -> clipmill_edit_ir::CaptionCue {
    serde_json::from_value(serde_json::json!({
        "cue_id": id, "start_ticks": first * SECOND, "end_ticks": last * SECOND,
        "region": "lower_safe", "anim": "none", "lines": [{"words":[{
            "text": text, "word_id": id, "start_ticks": first * SECOND, "end_ticks": last * SECOND
        }]}]
    }))
    .expect("cue")
}

#[test]
fn moving_and_removing_parts_carries_edits_and_restores_the_same_bytes() {
    let mut original = stitched();
    original.captions.cues = vec![
        cue("q", "Who?", 0, 2),
        cue("a", "Alice.", 3, 6),
        cue("p", "Now.", 20, 22),
    ];
    original
        .captions
        .burn_in
        .clone_from(&original.captions.cues);
    original.audio.gain_curve = vec![
        clipmill_edit_ir::GainPoint {
            t_ticks: 0,
            gain_db: -2.0,
        },
        clipmill_edit_ir::GainPoint {
            t_ticks: 20 * SECOND,
            gain_db: 3.0,
        },
    ];
    original.validate().expect("fixture");
    let mut edited = original.clone();
    let command = EditCommand::MovePart {
        part_id: "p3".to_owned(),
        to_index: 0,
    };
    let undo = command.apply(&mut edited).expect("move payoff to opening");
    assert_eq!(edited.video.segments[0].segment_id, "payoff");
    assert_eq!(edited.video.segments[0].join, None);
    assert_eq!(
        edited.video.segments[1].join.expect("incoming cut").kind,
        JoinKind::Reorder
    );
    assert_eq!(edited.captions.burn_in[0].start_ticks, SECOND);
    assert_eq!(
        edited.captions.cues[0]
            .words()
            .next()
            .expect("spoken word")
            .text,
        "Now."
    );
    let bytes = edited.to_canonical_json().expect("serializes");
    undo.apply(&mut edited).expect("undo move");
    assert_eq!(edited, original);
    EditCommand::from_canonical_json(&command.to_canonical_json().expect("serializes"))
        .expect("replays")
        .apply(&mut edited)
        .expect("reapplies");
    assert_eq!(edited.to_canonical_json().expect("serializes"), bytes);
    let before = edited.clone();
    let undo = EditCommand::RemovePart {
        part_id: "p2".to_owned(),
    }
    .apply(&mut edited)
    .expect("remove answer");
    assert!(
        edited
            .captions
            .cues
            .iter()
            .flat_map(clipmill_edit_ir::CaptionCue::words)
            .all(|w| w.text != "Alice.")
    );
    assert_eq!(edited.video.segments.len(), 2);
    assert!(
        edited
            .plan
            .as_ref()
            .expect("source plan")
            .removed
            .is_empty(),
        "removed part owns the old omission"
    );
    undo.apply(&mut edited).expect("undo");
    assert_eq!(edited, before);
}

#[test]
fn restoring_a_removed_range_reinstates_picture_words_and_undo() {
    let mut original = stitched();
    let answer = original.video.segments[1].clone();
    let mut tail = segment("tail", 786, 789);
    tail.part.clone_from(&answer.part);
    tail.join = Some(Join {
        kind: JoinKind::Removed,
        audio_crossfade_ticks: 3600,
        room_tone: false,
        room_tone_range: None,
    });
    original.video.segments.insert(2, tail);
    original.captions.cues = vec![cue("again", "Elsewhere", 0, 1), cue("tail", "Back", 19, 20)];
    original
        .captions
        .burn_in
        .clone_from(&original.captions.cues);
    let restored = cue("again", "Restored", 1, 2);
    let cmd = EditCommand::RestoreRemovedRange {
        index: 0,
        segments: vec![segment("one", 781, 783), segment("two", 783, 786)],
        reading_cues: vec![restored.clone()],
        burn_in_cues: vec![restored],
    };
    let mut edited = original.clone();
    let undo = cmd
        .apply(&mut edited)
        .expect("restores old documents without part IDs too");
    assert_eq!(edited.video.segments[2].part, answer.part);
    assert_eq!(edited.video.segments[4].join, None);
    assert!(
        edited
            .plan
            .as_ref()
            .expect("source plan")
            .removed
            .is_empty()
    );
    assert_eq!(edited.captions.cues[1].start_ticks, 20 * SECOND);
    assert_eq!(edited.captions.cues[2].start_ticks, 24 * SECOND);
    assert_eq!(
        edited.captions.burn_in[1]
            .words()
            .next()
            .expect("spoken word")
            .word_id
            .as_deref(),
        Some("again~restored")
    );
    undo.apply(&mut edited).expect("undo");
    assert_eq!(edited, original);
    let mut bad = original.clone();
    bad.video.segments[1].out_ticks += SECOND;
    let unchanged = bad.clone();
    assert!(cmd.apply(&mut bad).is_err());
    assert_eq!(bad, unchanged);
}

#[test]
fn cannot_move_outside_the_plan_or_remove_its_last_part() {
    let mut edit = stitched();
    assert!(
        EditCommand::MovePart {
            part_id: "p3".into(),
            to_index: 3
        }
        .apply(&mut edit)
        .is_err()
    );
    assert_eq!(edit, stitched());
    edit.video.segments.truncate(1);
    let original = edit.clone();
    assert!(
        EditCommand::RemovePart {
            part_id: "p1".into()
        }
        .apply(&mut edit)
        .is_err()
    );
    assert_eq!(edit, original);
}
