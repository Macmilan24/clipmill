#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use clipmill_contracts::schemas::mind_clip_plan::{JoinKind, MindClipPlanForm, PartRole};
use serde_json::json;

use super::*;
use crate::script::Sentence;

const S: u64 = SECOND;

/// Sixty sentences six seconds apart, five one-second words each.
fn recording() -> (Source, Vec<(u64, u64)>) {
    let sentences: Vec<Sentence> = (0..60)
        .map(|i| Sentence {
            index: i,
            start_ticks: i * 6 * S,
            end_ticks: i * 6 * S + 5 * S,
            speaker: Some(if i < 20 { "host" } else { "guest" }.to_owned()),
            text: if matches!(i, 12 | 25 | 49) {
                format!("Line {i}: back when we were at Northwind.")
            } else {
                format!("Line {i} of the conversation.")
            },
        })
        .collect();
    let words = sentences
        .iter()
        .flat_map(|s| (0..5).map(move |k| (s.start_ticks + k * S, s.start_ticks + (k + 1) * S)))
        .collect();
    (
        Source {
            duration_ticks: 360 * S,
            sentences,
            ..Source::default()
        },
        words,
    )
}

/// The markup: hooks, landings, a tangent, a dependency.
fn scores() -> MindUnitScores {
    let mut units: Vec<serde_json::Value> = (0..60)
        .map(|i| json!({ "unit": format!("s{i}"), "hook": "none", "landing": "none", "trim": "keep" }))
        .collect();
    for (unit, field, value) in [
        (3, "hook", "strong"),
        (25, "hook", "strong"),
        (45, "hook", "good"),
        (18, "landing", "strong"),
        (30, "landing", "good"),
        (42, "landing", "strong"),
        (50, "landing", "good"),
        (13, "trim", "tangent"),
        (14, "trim", "tangent"),
    ] {
        units[unit][field] = json!(value);
    }
    units[11]["needs"] = json!("s10");
    serde_json::from_value(json!({
        "schema_version": "clipmill.mind.unit_scores.v1",
        "source_fingerprint": format!("sha256:{}", "a".repeat(64)),
        "script": { "level": "compact", "digest": format!("sha256:{}", "c".repeat(64)), "windows": 1 },
        "producer": { "stage": "mind-score", "implementation": "test", "model": "test", "prompt_version": "score.v1" },
        "units": units,
        "unscored": [],
    }))
    .unwrap()
}

fn event(id: &str, kind: &str, first: u64, last: u64, entities: &[&str]) -> serde_json::Value {
    let chapter = if first < 30 { "c1" } else { "c2" };
    json!({
        "chapter_id": chapter,
        "event_id": id, "kind": kind, "title": format!("Event {id}"), "mood": "neutral",
        "first_unit": format!("s{first}"), "last_unit": format!("s{last}"),
        "start_ticks": first * 6 * S, "end_ticks": last * 6 * S + 5 * S,
        "speakers": ["A"], "entities": entities,
    })
}

/// A question at s3 answered at s40, a story with a tangent, a story with a
/// strong line in its middle, and one company named in three moments.
fn memory() -> MindMemory {
    serde_json::from_value(json!({
        "schema_version": "clipmill.mind.memory.v1",
        "source_fingerprint": format!("sha256:{}", "a".repeat(64)),
        "script": { "level": "compact", "digest": format!("sha256:{}", "c".repeat(64)), "windows": 1 },
        "producer": { "stage": "mind-memory", "implementation": "test", "model": "test", "prompt_versions": ["memory.read.v1"] },
        "chapters": [],
        "events": [
            event("m1", "question", 3, 3, &[]),
            event("m2", "story", 10, 18, &["e1"]),
            event("m3", "story", 22, 30, &["e1"]),
            event("m4", "answer", 40, 42, &[]),
            event("m5", "claim", 48, 50, &["e1"]),
        ],
        "entities": [{ "entity_id": "e1", "name": "Northwind", "kind": "organization", "aliases": [], "events": ["m2", "m3", "m5"] }],
        "links": [{ "link_id": "l1", "kind": "question_answer", "from_event": "m1", "to_event": "m4", "note": "The question is answered six minutes later." }],
        "dropped": [],
    }))
    .unwrap()
}

fn cut(with_memory: bool) -> Vec<Candidate> {
    let (source, words) = recording();
    let scores = scores();
    let memory = memory();
    candidates(&Material {
        source_fingerprint: &format!("sha256:{}", "a".repeat(64)),
        source: &source,
        words: &words,
        scores: Some(&scores),
        memory: with_memory.then_some(&memory),
        length: (15 * S, 60 * S),
    })
}

fn find(found: &[Candidate], form: MindClipPlanForm) -> &Candidate {
    found
        .iter()
        .find(|c| c.plan.form == form)
        .unwrap_or_else(|| panic!("no {form:?} among {:?}", forms(found)))
}

fn forms(found: &[Candidate]) -> Vec<MindClipPlanForm> {
    found.iter().map(|c| c.plan.form).collect()
}

#[test]
fn a_distant_answer_is_stitched_to_its_question() {
    let found = cut(true);
    let stitch = find(&found, MindClipPlanForm::Stitch);
    let plan = &stitch.plan;
    assert_eq!(
        plan.parts.iter().map(|p| p.role).collect::<Vec<_>>(),
        [PartRole::Question, PartRole::Answer]
    );
    assert_eq!(plan.parts[0].start_ticks, 18 * S);
    assert_eq!(plan.parts[1].start_ticks, 240 * S);
    assert_eq!(plan.joins[0].kind, JoinKind::Distant);
    assert!(
        plan.joins[0]
            .link
            .as_deref()
            .unwrap()
            .starts_with("l1 question_answer")
    );
    assert_eq!(stitch.scout, Scout::Link);
}

#[test]
fn a_tangent_inside_a_story_is_cut_out() {
    let found = cut(true);
    let tight = find(&found, MindClipPlanForm::Tightened);
    let [part] = tight.plan.parts.as_slice() else {
        panic!("one part");
    };
    let [removed] = part.removed.as_slice() else {
        panic!("one removal");
    };
    // From the end of s12 to the start of s15.
    assert_eq!(
        (removed.start_ticks, removed.end_ticks.get()),
        (77 * S, 90 * S)
    );
    assert!(!part.units.iter().any(|u| u.as_str() == "s13"));
}

#[test]
fn a_strong_line_inside_a_story_opens_it_cold() {
    let found = cut(true);
    let cold = find(&found, MindClipPlanForm::ColdOpen);
    let plan = &cold.plan;
    assert_eq!(plan.parts[0].role, PartRole::Hook);
    assert_eq!(plan.parts[0].start_ticks, 150 * S);
    assert_eq!(plan.parts[1].start_ticks, 132 * S);
    assert_eq!(plan.joins[0].kind, JoinKind::Reorder);
}

#[test]
fn three_moments_about_one_company_make_a_montage() {
    let found = cut(true);
    let montage = find(&found, MindClipPlanForm::ThemeMontage);
    assert_eq!(montage.plan.parts.len(), 3);
    assert!(montage.plan.parts.iter().all(|p| p.role == PartRole::Item));
    assert!(montage.plan.programme_ticks_fit((15 * S, 60 * S)));
}

#[test]
fn most_candidates_are_more_than_one_straight_span() {
    let found = cut(true);
    let shaped = found
        .iter()
        .filter(|c| c.plan.form != MindClipPlanForm::Straight)
        .count();
    assert!(shaped * 2 > found.len(), "{:?}", forms(&found));
}

#[test]
fn no_candidate_opens_on_a_line_that_needs_an_earlier_one() {
    for candidate in cut(true).iter().chain(cut(false).iter()) {
        for part in &candidate.plan.parts {
            assert_ne!(
                part.units[0].as_str(),
                "s11",
                "{:?}",
                candidate.plan.plan_id
            );
        }
    }
}

#[test]
fn without_a_memory_moments_open_on_hooks() {
    let found = cut(false);
    assert!(!found.is_empty());
    assert!(
        found
            .iter()
            .all(|c| matches!(c.scout, Scout::Span | Scout::ColdOpen))
    );
    let starts: Vec<&str> = found
        .iter()
        .filter(|c| c.scout == Scout::Span)
        .map(|c| c.plan.parts[0].units[0].as_str())
        .collect();
    assert!(starts.contains(&"s3"), "{starts:?}");
}

#[test]
fn the_same_material_gives_the_same_plans() {
    let ids = |found: Vec<Candidate>| -> Vec<String> {
        found
            .into_iter()
            .map(|c| c.plan.plan_id.to_string())
            .collect()
    };
    assert_eq!(ids(cut(true)), ids(cut(true)));
}

trait Fits {
    fn programme_ticks_fit(&self, range: (u64, u64)) -> bool;
}

impl Fits for MindClipPlan {
    fn programme_ticks_fit(&self, (min, max): (u64, u64)) -> bool {
        (min..=max).contains(&crate::programme_ticks(self))
    }
}

/// A short talk once yielded no clip at all: the markup called nearly every
/// line a retake or an aside, and no moment had enough left. Markup is advice;
/// each event still yields its plain moment.
#[test]
fn over_marked_markup_still_leaves_every_moment() {
    let (source, words) = recording();
    let mut scores = serde_json::to_value(scores()).unwrap();
    for (index, row) in scores["units"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .enumerate()
    {
        if index % 7 != 0 {
            row["trim"] = json!(if index % 2 == 0 { "retake" } else { "aside" });
        }
    }
    let scores: MindUnitScores = serde_json::from_value(scores).unwrap();
    let memory = memory();
    let found = candidates(&Material {
        source_fingerprint: &format!("sha256:{}", "a".repeat(64)),
        source: &source,
        words: &words,
        scores: Some(&scores),
        memory: Some(&memory),
        length: (15 * S, 60 * S),
    });
    assert!(found.len() >= 3, "{:?}", forms(&found));
    for candidate in &found {
        let span: u64 = candidate
            .plan
            .parts
            .iter()
            .map(|p| p.end_ticks.get() - p.start_ticks)
            .sum();
        let kept = crate::programme_ticks(&candidate.plan);
        assert!(
            kept * 100 >= span * (100 - MAX_CUT_PERCENT),
            "{} hollowed out",
            candidate.plan.plan_id.as_str()
        );
    }
}
