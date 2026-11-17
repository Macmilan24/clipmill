//! The mind's contracts, Rust leg.
//!
//! A clip plan is parts with roles, the ranges cut out of them and the joins
//! between them. What these assert is the shape a consumer codes against:
//! times are integer ticks, roles and forms are closed sets, and a plan always
//! has at least one part.
//!
//! Tests may panic; the workspace deny targets production code.
#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use clipmill_contracts::schemas::mind_clip_plan::{
    JoinKind, MindClipPlan, MindClipPlanForm, PartRole,
};
use serde::{Serialize, de::DeserializeOwned};

fn read(rel: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()))
}

fn roundtrip<T: DeserializeOwned + Serialize>(rel: &str) -> T {
    let raw = read(rel);
    let parsed: T = serde_json::from_str(&raw)
        .unwrap_or_else(|err| panic!("valid fixture {rel} rejected: {err}"));
    let mut text = serde_json::to_string_pretty(&serde_json::to_value(&parsed).unwrap()).unwrap();
    text.push('\n');
    assert_eq!(
        text, raw,
        "canonical round-trip must be byte-identical for {rel}"
    );
    parsed
}

fn refused<T: DeserializeOwned>(rel: &str) {
    assert!(
        serde_json::from_str::<T>(&read(rel)).is_err(),
        "invalid fixture {rel} was accepted"
    );
}

#[test]
fn a_stitch_from_distant_parts_round_trips() {
    let plan: MindClipPlan = roundtrip("contracts/fixtures/mind.clip_plan/valid/stitch.json");
    assert_eq!(plan.form, MindClipPlanForm::Stitch);
    let roles: Vec<_> = plan.parts.iter().map(|part| part.role).collect();
    assert_eq!(
        roles,
        [PartRole::Question, PartRole::Answer, PartRole::Payoff]
    );
    assert_eq!(plan.parts[1].removed.len(), 1);
    assert_eq!(plan.joins[1].kind, JoinKind::Distant);
    assert!(plan.joins[1].link.is_some());
}

#[test]
fn a_straight_cut_is_a_plan_of_one_part() {
    let plan: MindClipPlan = roundtrip("contracts/fixtures/mind.clip_plan/valid/straight.json");
    assert_eq!(plan.parts.len(), 1);
    assert!(plan.joins.is_empty());
}

#[test]
fn malformed_plans_are_refused() {
    for name in ["unknown_role", "fractional_ticks"] {
        refused::<MindClipPlan>(&format!(
            "contracts/fixtures/mind.clip_plan/invalid/{name}.json"
        ));
    }
}

#[test]
fn a_recording_memory_round_trips_with_its_links() {
    use clipmill_contracts::schemas::mind_memory::{LinkKind, MindMemory};

    let memory: MindMemory = roundtrip("contracts/fixtures/mind.memory/valid/podcast.json");
    assert_eq!(memory.chapters.len(), 2);
    let kinds: Vec<_> = memory.links.iter().map(|link| link.kind).collect();
    assert_eq!(kinds, [LinkKind::QuestionAnswer, LinkKind::PromiseDelivery]);
    assert_eq!(memory.dropped.len(), 1);
}

#[test]
fn malformed_memories_are_refused() {
    use clipmill_contracts::schemas::mind_memory::MindMemory;

    for rel in [
        "contracts/fixtures/mind.memory/invalid/unknown_link_kind.json",
        "contracts/fixtures/mind.memory/invalid/unit_not_cited.json",
        "contracts/fixtures/mind.memory/invalid/float_ticks.json",
    ] {
        refused::<MindMemory>(rel);
    }
}

#[test]
fn unit_scores_mark_every_sentence_and_refuse_unknown_marks() {
    use clipmill_contracts::schemas::mind_unit_scores::{MindUnitScores, RowTrim};

    let scores: MindUnitScores =
        roundtrip("contracts/fixtures/mind.unit_scores/valid/tightened_answer.json");
    assert_eq!(scores.units.len(), 6);
    assert_eq!(scores.units[2].trim, RowTrim::FalseStart);
    assert!(scores.units[3].needs.is_some());
    for rel in [
        "contracts/fixtures/mind.unit_scores/invalid/unknown_hook_level.json",
        "contracts/fixtures/mind.unit_scores/invalid/unknown_trim.json",
    ] {
        refused::<MindUnitScores>(rel);
    }
}
