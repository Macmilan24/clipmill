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
    });
    document.video.segments = vec![question, answer, payoff];
    document.plan = Some(Plan {
        plan_id: "plan_7f3a9c21d04e8b65".to_owned(),
        form: PlanForm::Stitch,
        removed: vec![RemovedRange {
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
        assert!(!text.contains(key), "{key} written for a clip that has none");
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
    assert!(matches!(document.validate(), Err(DocumentError::InvalidPlan)));
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
