//! The score card and the ranked set, Rust leg.
//!
//! What these assert is the shape a results board and an inspector code
//! against, and the two promises they rest on: a factor nobody measured is
//! distinguishable from one measured as zero, and a set smaller than the one
//! requested accounts for the difference.
//!
//! Tests may panic; the workspace deny targets production code.
#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use clipmill_contracts::proto::ipc::v1::{
    AnalyzeSourcePayloadV1, RankCandidatesPayloadV1, RankStagePayloadV1,
};
use clipmill_contracts::schemas::ranking_set::{FactorName, RankingSet};
use prost::Message;

fn read(rel: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) => panic!("cannot read {}: {err}", path.display()),
    }
}

fn canonical(value: &serde_json::Value) -> String {
    let mut text = serde_json::to_string_pretty(value).unwrap_or_else(|err| panic!("{err}"));
    text.push('\n');
    text
}

fn roundtrip(rel: &str) -> RankingSet {
    let raw = read(rel);
    let parsed: RankingSet = match serde_json::from_str(&raw) {
        Ok(parsed) => parsed,
        Err(err) => panic!("valid fixture {rel} rejected: {err}"),
    };
    let reserialized =
        serde_json::to_value(&parsed).unwrap_or_else(|err| panic!("reserialize {rel}: {err}"));
    assert_eq!(
        canonical(&reserialized),
        raw,
        "canonical round-trip must be byte-identical for {rel}"
    );
    parsed
}

#[test]
fn every_valid_ranking_fixture_roundtrips_canonically() {
    roundtrip("contracts/fixtures/ranking.set/valid/interview.json");
    roundtrip("contracts/fixtures/ranking.set/valid/ten_words.json");
}

/// The distinction the whole card is built around, at the contract level: an
/// axis nobody measured carries no number and states a reason, and one measured
/// at zero carries the zero.
#[test]
fn an_unmeasured_axis_is_not_a_zero() {
    let set = roundtrip("contracts/fixtures/ranking.set/valid/interview.json");
    let card = &set.cohort.first().expect("a ranked clip").factors;
    assert_eq!(card.len(), 8, "all eight axes are always named");
    let prompt = card
        .iter()
        .find(|factor| factor.name == FactorName::PromptRelevance)
        .expect("the prompt axis is named");
    assert!(!prompt.available);
    assert!(prompt.value.is_none());
    assert!(prompt.weight.is_none());
    assert!(prompt.unavailable_reason.is_some());
    // And a measured one does carry its number and its weight.
    let hook = card
        .iter()
        .find(|factor| factor.name == FactorName::Hook)
        .expect("the hook axis is named");
    assert!(hook.available && hook.value.is_some() && hook.weight.is_some());
}

/// Asked for more clips than a recording holds, the honest answer is fewer plus
/// a reason. The arithmetic has to add up or the reason is decoration.
#[test]
fn a_short_set_accounts_for_every_clip_it_did_not_return() {
    let set = roundtrip("contracts/fixtures/ranking.set/valid/interview.json");
    let accounted: u64 = set.shortfall.iter().map(|reason| reason.count.get()).sum();
    assert_eq!(
        u64::try_from(set.selected.len()).unwrap() + accounted,
        set.requested.count.get()
    );
}

/// The runner-up ships beside the winner because the optimizer's second choice
/// is frequently the editor's first.
#[test]
fn a_ranked_clip_carries_the_boundary_it_beat() {
    let set = roundtrip("contracts/fixtures/ranking.set/valid/interview.json");
    let entry = set.cohort.first().expect("a ranked clip");
    assert_eq!(entry.boundary.terms.len(), 7);
    let alternative = entry
        .boundary
        .alternative
        .as_ref()
        .expect("this lattice offered more than one legal pair");
    assert!(alternative.score <= entry.boundary.score);
}

/// The displayed number is a percentile within one recording's cohort. Naming
/// it `display_score` beside the raw `score` is what stops it being stored as a
/// probability.
#[test]
fn the_displayed_number_is_bounded_and_the_raw_one_is_not() {
    let set = roundtrip("contracts/fixtures/ranking.set/valid/interview.json");
    for entry in &set.cohort {
        assert!((0..=99).contains(&entry.display_score));
    }
    assert!(set.cohort.iter().any(|entry| entry.score > 1.0));
}

/// A recording with nothing long enough to clip still publishes its ranking,
/// and it is a different document from one nobody ranked.
#[test]
fn an_empty_cohort_is_still_a_ranking() {
    let set = roundtrip("contracts/fixtures/ranking.set/valid/ten_words.json");
    assert!(set.cohort.is_empty());
    assert!(set.selected.is_empty());
    assert!(!set.shortfall.is_empty(), "an empty set still says why");
    assert!(!set.rubric.scorer.as_str().is_empty());
}

#[test]
fn invalid_ranking_fixtures_are_rejected() {
    // The empty-array and numeric-bound cases belong to the schema and the
    // Python leg: typify enforces string patterns, enums, required fields, and
    // non-zero integers through newtypes, but carries `minItems` and `maximum`
    // as documentation.
    for (fixture, why) in [
        ("float_ticks", "float ticks must not parse (D06)"),
        ("unknown_factor", "an unlisted axis must not parse"),
        ("rank_from_zero", "ranks count from one"),
        (
            "shortfall_counted_zero",
            "a reason accounting for no clips explains nothing",
        ),
        ("unnamed_rubric", "the arithmetic must name itself"),
    ] {
        let rejected = serde_json::from_str::<RankingSet>(&read(&format!(
            "contracts/fixtures/ranking.set/invalid/{fixture}.json"
        )));
        assert!(rejected.is_err(), "{why}");
    }
}

/// The stage payload carries what ranking was *asked for* and nothing about what
/// it reads. That is the property that makes the two routes one: a ranked set
/// produced inside an analysis and the same one produced by a standalone job
/// encode identical bytes here, so they key to one address instead of two.
#[test]
fn the_ranking_stage_payload_asks_without_naming_inputs() {
    let standalone = RankStagePayloadV1 {
        key_version: "clipmill.rank-stage.v1".to_owned(),
        stage: "rank-candidates".to_owned(),
        count: 0,
        diversity_milli: 0,
    };
    assert_eq!(
        RankStagePayloadV1::decode(standalone.encode_to_vec().as_slice()).expect("round-trip"),
        standalone
    );
    let encoded = String::from_utf8_lossy(&standalone.encode_to_vec()).into_owned();
    assert!(
        !encoded.contains("sha256:") && !encoded.contains('/'),
        "an address or a path in the payload would key one observation twice"
    );
    // Asking for a different set is a different answer, so both knobs move the
    // bytes the artifact key is computed from.
    for changed in [
        RankStagePayloadV1 {
            count: 3,
            ..standalone.clone()
        },
        RankStagePayloadV1 {
            diversity_milli: 500,
            ..standalone.clone()
        },
    ] {
        assert_ne!(standalone.encode_to_vec(), changed.encode_to_vec());
    }
}

#[test]
fn the_job_payloads_name_only_a_source_and_what_was_asked_for() {
    let ranking = RankCandidatesPayloadV1 {
        key_version: "clipmill.rank-candidates.v1".to_owned(),
        source_id: "src_0123456789abcdef".to_owned(),
        count: 0,
        diversity_milli: 0,
    };
    assert_eq!(
        RankCandidatesPayloadV1::decode(ranking.encode_to_vec().as_slice()).expect("round-trip"),
        ranking
    );
    let analyze = AnalyzeSourcePayloadV1 {
        key_version: "clipmill.analyze-source.v1".to_owned(),
        source_id: "src_0123456789abcdef".to_owned(),
        language: String::new(),
        duration: None,
        count: 0,
        diversity_milli: 0,
    };
    assert_eq!(
        AnalyzeSourcePayloadV1::decode(analyze.encode_to_vec().as_slice()).expect("round-trip"),
        analyze
    );
}
