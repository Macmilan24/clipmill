//! A clip planned as parts becomes one edit document: each kept range in
//! programme order, its captions moved to where it plays, its joins recorded,
//! and a plan that cannot be made refused with every reason.
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::collections::BTreeSet;

use clipmill_contracts::schemas::{
    mind_clip_plan::MindClipPlan, speech_transcript::SpeechTranscript,
};
use clipmill_director::{
    DirectError, Frame, FrameShape, JOIN_PUNCH_ZOOM, PlanEvidence, PlanRequest, direct_plan,
};
use clipmill_edit_ir::{EditDocument, JoinKind, PartRole, PlanForm};
use serde_json::json;

fn transcript() -> SpeechTranscript {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts/fixtures/speech.transcript/valid/interview.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn start(transcript: &SpeechTranscript, word: usize) -> u64 {
    transcript.words[word].start_ticks
}

fn end(transcript: &SpeechTranscript, word: usize) -> u64 {
    transcript.words[word].end_ticks
}

/// A plan from JSON, with its duration set to what its parts play.
fn plan(value: serde_json::Value) -> MindClipPlan {
    let mut plan: MindClipPlan = serde_json::from_value(value).unwrap();
    plan.duration_ticks = std::num::NonZeroU64::new(clipmill_mind::programme_ticks(&plan)).unwrap();
    plan
}

/// The question that opens the interview, its answer with one passage cut
/// out of the middle, and a line from fifty seconds later that pays it off.
fn stitch(t: &SpeechTranscript) -> MindClipPlan {
    plan(json!({
        "schema_version": "clipmill.mind.clip_plan.v1",
        "source_fingerprint": t.source_fingerprint.as_str(),
        "plan_id": "plan_00000000000000a1",
        "form": "stitch",
        "title": "What makes a renderer deterministic",
        "parts": [
            {"part_id": "p1", "role": "question", "start_ticks": start(t, 0), "end_ticks": end(t, 4)},
            {"part_id": "p2", "role": "answer", "start_ticks": start(t, 5), "end_ticks": end(t, 30),
             "removed": [{"start_ticks": start(t, 12), "end_ticks": end(t, 15), "reason": "tangent"}]},
            {"part_id": "p3", "role": "payoff", "start_ticks": start(t, 90), "end_ticks": end(t, 99)}
        ],
        "joins": [
            {"after": "p1", "kind": "adjacent"},
            {"after": "p2", "kind": "distant", "link": "callback:s9>s1",
             "crossfade_ms": 60, "framing": "punch_in", "room_tone": true}
        ],
        "duration_ticks": 1,
        "provenance": {"by": "mind"}
    }))
}

fn request(plan: &MindClipPlan) -> PlanRequest<'_> {
    PlanRequest {
        plan,
        style_ref: clipmill_captions::DEFAULT_STYLE_REF.to_owned(),
        frame: Frame {
            width: 1920,
            height: 1080,
        },
        shape: FrameShape::Vertical,
    }
}

fn direct(t: &SpeechTranscript, plan: &MindClipPlan) -> Result<EditDocument, DirectError> {
    direct_plan(
        PlanEvidence {
            transcript: t,
            index: None,
            shots: None,
            faces: None,
            lattice: None,
        },
        &request(plan),
    )
}

fn caption_words(document: &EditDocument) -> Vec<(String, i64, i64)> {
    document
        .captions
        .cues
        .iter()
        .flat_map(|cue| cue.lines.iter())
        .flat_map(|line| line.words.iter())
        .map(|word| (word.text.clone(), word.start_ticks, word.end_ticks))
        .collect()
}

#[test]
fn a_stitch_plays_its_parts_in_order_with_what_was_removed_left_out() {
    let t = transcript();
    let plan = stitch(&t);
    let document = direct(&t, &plan).expect("directed");
    document.validate().expect("valid");

    let ranges: Vec<(i64, i64, &str)> = document
        .video
        .segments
        .iter()
        .map(|s| {
            (
                s.in_ticks,
                s.out_ticks,
                s.part.as_ref().unwrap().part_id.as_str(),
            )
        })
        .collect();
    let as_i = |v: u64| i64::try_from(v).unwrap();
    assert_eq!(
        ranges,
        [
            (as_i(start(&t, 0)), as_i(end(&t, 4)), "p1"),
            (as_i(start(&t, 5)), as_i(start(&t, 12)), "p2"),
            (as_i(end(&t, 15)), as_i(end(&t, 30)), "p2"),
            (as_i(start(&t, 90)), as_i(end(&t, 99)), "p3"),
        ]
    );
    let joins: Vec<Option<JoinKind>> = document
        .video
        .segments
        .iter()
        .map(|s| s.join.map(|join| join.kind))
        .collect();
    assert_eq!(
        joins,
        [
            None,
            Some(JoinKind::Adjacent),
            Some(JoinKind::Removed),
            Some(JoinKind::Distant)
        ]
    );
    let distant = document.video.segments[3].join.unwrap();
    assert_eq!(distant.audio_crossfade_ticks, 60 * 90);
    assert!(distant.room_tone);
    let [start, end] = distant.room_tone_range.expect("a VAD silence for the room");
    assert_eq!(end - start, 5_400);
    let (start, end) = (start.cast_unsigned(), end.cast_unsigned());
    assert!(
        t.silences
            .iter()
            .any(|silence| silence.start_ticks <= start && silence.end_ticks >= end)
    );
    assert!(
        t.words
            .iter()
            .all(|word| word.end_ticks <= start || word.start_ticks >= end)
    );
    assert_eq!(
        document.video.segments[3].part.as_ref().unwrap().role,
        PartRole::Payoff
    );

    // The removed passage is gone from the captions, and the payoff's words
    // play straight after the answer.
    let words = caption_words(&document);
    let spoken: Vec<&str> = words.iter().map(|(text, ..)| text.as_str()).collect();
    for removed in 12..=15 {
        let text = t.words[removed].text.as_str();
        let elsewhere = (0..=30)
            .chain(90..=99)
            .filter(|i| !(12..=15).contains(i))
            .any(|i| t.words[i].text.as_str() == text);
        assert!(elsewhere || !spoken.contains(&text), "{text} was removed");
    }
    let programme = clipmill_mind::programme_ticks(&plan);
    assert_eq!(
        document.program_duration_ticks(),
        i64::try_from(programme).unwrap()
    );
    let (_, last_start, last_end) = words.last().unwrap();
    assert!(*last_end <= document.program_duration_ticks());
    assert!(*last_start > document.program_duration_ticks() - 4 * 90_000);

    let recorded = document.plan.as_ref().unwrap();
    assert_eq!(recorded.form, PlanForm::Stitch);
    assert_eq!(recorded.removed.len(), 1);
    assert_eq!(recorded.removed[0].reason, "tangent");
    assert_eq!(
        document.title.as_deref(),
        Some("What makes a renderer deterministic")
    );
}

#[test]
fn cuts_on_one_camera_change_the_framing() {
    let t = transcript();
    let document = direct(&t, &stitch(&t)).expect("directed");
    let zooms: Vec<u16> = document
        .video
        .segments
        .iter()
        .map(|segment| segment.layout.zoom_percent())
        .collect();
    // No face evidence, so every section is fitted: the cut inside the answer
    // and the distant join both move in closer; the rest stay as they were.
    assert_eq!(zooms, [100, 100, JOIN_PUNCH_ZOOM, JOIN_PUNCH_ZOOM]);
}

#[test]
fn a_cold_open_plays_its_hook_once_and_leaves_it_out_of_the_body() {
    let t = transcript();
    let plan = plan(json!({
        "schema_version": "clipmill.mind.clip_plan.v1",
        "source_fingerprint": t.source_fingerprint.as_str(),
        "plan_id": "plan_00000000000000b2",
        "form": "cold_open",
        "parts": [
            {"part_id": "p0", "role": "hook", "start_ticks": start(&t, 20), "end_ticks": end(&t, 26)},
            {"part_id": "p1", "role": "body", "start_ticks": start(&t, 5), "end_ticks": end(&t, 30),
             "removed": [{"start_ticks": start(&t, 20), "end_ticks": end(&t, 26),
                          "reason": "repetition", "note": "Heard first, as the hook"}]}
        ],
        "joins": [{"after": "p0", "kind": "reorder", "framing": "punch_out"}],
        "duration_ticks": 1,
        "provenance": {"by": "mind"}
    }));
    let document = direct(&t, &plan).expect("directed");
    document.validate().expect("a valid document");
    let ids: Vec<String> = document
        .captions
        .cues
        .iter()
        .flat_map(|cue| cue.lines.iter())
        .flat_map(|line| line.words.iter())
        .filter_map(|word| word.word_id.clone())
        .collect();
    let unique: BTreeSet<&String> = ids.iter().collect();
    assert_eq!(unique.len(), ids.len(), "every word is shown once");
    assert!(
        !ids.iter().any(|id| id.ends_with("~p1")),
        "no word plays twice, so none is renamed"
    );
    assert_eq!(
        document.video.segments[1].join.map(|join| join.kind),
        Some(JoinKind::Reorder)
    );
}

#[test]
fn a_cold_open_that_plays_its_hook_again_is_refused() {
    let t = transcript();
    let plan = plan(json!({
        "schema_version": "clipmill.mind.clip_plan.v1",
        "source_fingerprint": t.source_fingerprint.as_str(),
        "plan_id": "plan_00000000000000b3",
        "form": "cold_open",
        "parts": [
            {"part_id": "p0", "role": "hook", "start_ticks": start(&t, 20), "end_ticks": end(&t, 26)},
            {"part_id": "p1", "role": "body", "start_ticks": start(&t, 5), "end_ticks": end(&t, 30)}
        ],
        "joins": [{"after": "p0", "kind": "reorder", "framing": "punch_out"}],
        "duration_ticks": 1,
        "provenance": {"by": "mind"}
    }));
    match direct(&t, &plan) {
        Err(DirectError::InvalidPlan(reasons)) => {
            assert!(reasons.contains("heard twice"), "{reasons}");
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
}

#[test]
fn a_plan_that_cannot_be_made_is_refused_with_its_reasons() {
    let t = transcript();
    let mut plan = stitch(&t);
    plan.joins[1].link = None;
    match direct(&t, &plan) {
        Err(DirectError::InvalidPlan(reasons)) => {
            assert!(reasons.contains("without the evidence"), "{reasons}");
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
}

#[test]
fn a_plan_for_another_recording_is_refused() {
    let t = transcript();
    let mut plan = stitch(&t);
    plan.source_fingerprint = format!("sha256:{}", "f".repeat(64)).parse().unwrap();
    assert!(matches!(direct(&t, &plan), Err(DirectError::Mismatched)));
}

#[test]
fn the_same_plan_directs_the_same_document() {
    let t = transcript();
    let plan = stitch(&t);
    let first = direct(&t, &plan).unwrap().to_canonical_json().unwrap();
    let second = direct(&t, &plan).unwrap().to_canonical_json().unwrap();
    assert_eq!(first, second);
}

#[test]
fn a_plan_with_no_verified_silence_keeps_the_fades_without_sampling_speech() {
    let mut transcript = transcript();
    transcript.silences.clear();
    let document = direct(&transcript, &stitch(&transcript)).unwrap();
    let join = document
        .video
        .segments
        .iter()
        .filter_map(|segment| segment.join)
        .find(|join| join.room_tone)
        .unwrap();
    assert_eq!(join.room_tone_range, None);
    assert_eq!(join.blend_ticks(), 5_400);
}
