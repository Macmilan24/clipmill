//! Shot detection's contract, Rust leg.
//!
//! The observation contract again, on a stage that produces boundaries rather
//! than words: a recording nobody decoded must not read as a recording with no
//! cuts in it, a shot must be no more certain than the cut that starts it, and
//! what the detector was configured with must survive into the document so a
//! re-tune is a new observation rather than a correction of this one.
//!
//! Tests may panic; the workspace deny targets production code.
#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use clipmill_contracts::proto::ipc::v1::{DetectShotsPayloadV1, ShotDetectionV1};
use clipmill_contracts::schemas::evidence_shots::EvidenceShots;
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

/// Two readings of the same number. The confidences compared below are parsed
/// from one document and copied, never recomputed, so this is an identity check
/// written in a form that does not compare floats with `==`.
fn same(left: f64, right: f64) -> bool {
    (left - right).abs() <= f64::EPSILON
}

fn roundtrip(rel: &str) -> EvidenceShots {
    let raw = read(rel);
    let parsed: EvidenceShots = match serde_json::from_str(&raw) {
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
fn every_valid_shots_fixture_roundtrips_canonically() {
    roundtrip("contracts/fixtures/evidence.shots/valid/three_shots.json");
    roundtrip("contracts/fixtures/evidence.shots/valid/one_unbroken_shot.json");
    roundtrip("contracts/fixtures/evidence.shots/valid/never_examined.json");
}

/// An empty cut list means two opposite things depending on coverage, and only
/// one of them is a statement about the footage.
#[test]
fn an_unbroken_shot_and_an_undecoded_recording_are_different_documents() {
    let unbroken = roundtrip("contracts/fixtures/evidence.shots/valid/one_unbroken_shot.json");
    let unexamined = roundtrip("contracts/fixtures/evidence.shots/valid/never_examined.json");

    assert!(unbroken.cuts.is_empty());
    assert!(unexamined.cuts.is_empty());
    assert!(unbroken.coverage.analyzed);
    assert!(!unexamined.coverage.analyzed);

    // The examined recording still says what it found: one shot, the whole
    // thing. The unexamined one publishes no shots at all and says why.
    assert_eq!(unbroken.shots.len(), 1);
    assert!(unexamined.shots.is_empty());
    assert!(unbroken.invalid_regions.is_empty());
    assert_eq!(unexamined.invalid_regions.len(), 1);
}

/// A shot is bounded by cuts, and it cannot be worth more than the weaker of
/// them. The middle shot here is bounded by both cuts and must carry the
/// lower pair; the opening shot is bounded by the recording's start, which is
/// a fact rather than a detection and claims nothing on its own.
#[test]
fn a_shot_is_no_more_certain_than_the_cut_that_starts_it() {
    let shots = roundtrip("contracts/fixtures/evidence.shots/valid/three_shots.json");
    assert_eq!(shots.cuts.len(), 2);
    assert_eq!(shots.shots.len(), shots.cuts.len() + 1);

    let weaker = &shots.cuts[1].confidence;
    let middle = &shots.shots[1].confidence;
    assert!(middle.p50 <= shots.cuts[0].confidence.p50);
    assert!(same(middle.p50, weaker.p50));
    assert!(same(middle.p10, weaker.p10));

    // Spans tile coverage exactly: no gaps, no overlaps, first and last
    // touching the coverage bounds.
    assert_eq!(shots.shots[0].start_ticks, shots.coverage.start_ticks);
    assert_eq!(
        shots.shots[shots.shots.len() - 1].end_ticks,
        shots.coverage.end_ticks
    );
    for pair in shots.shots.windows(2) {
        assert_eq!(pair[0].end_ticks, pair[1].start_ticks);
    }
    // And every cut is where a span begins.
    for (cut, shot) in shots.cuts.iter().zip(shots.shots.iter().skip(1)) {
        assert_eq!(cut.t_ticks, shot.start_ticks);
    }
}

/// The raw score is kept alongside the confidence because it is the only
/// number a re-tune can be reasoned about from without decoding again.
#[test]
fn a_cut_carries_the_distance_that_produced_it() {
    let shots = roundtrip("contracts/fixtures/evidence.shots/valid/three_shots.json");
    let threshold = shots.detection.threshold;
    for cut in &shots.cuts {
        assert!(
            cut.score >= threshold,
            "a reported cut scored below the threshold that reported it"
        );
    }
    assert!(shots.producer.calibration.is_some());
}

#[test]
fn invalid_shots_fixtures_are_rejected() {
    for (fixture, why) in [
        ("float_seconds", "float seconds must not parse (D06)"),
        (
            "cut_without_a_score",
            "a cut with no distance behind it must not parse",
        ),
        (
            "scalar_confidence",
            "a scalar confidence must not parse: p10 is not optional",
        ),
    ] {
        let rejected = serde_json::from_str::<EvidenceShots>(&read(&format!(
            "contracts/fixtures/evidence.shots/invalid/{fixture}.json"
        )));
        assert!(rejected.is_err(), "{why}");
    }
}

/// Zero means "no opinion" on the wire, which is how a caller with nothing to
/// say avoids having to know the defaults. The stage payload is where those
/// defaults get resolved, so the job payload must survive the round trip
/// unset rather than acquiring numbers on the way.
#[test]
fn a_detect_shots_payload_with_no_opinion_round_trips_empty() {
    let message = DetectShotsPayloadV1 {
        key_version: "clipmill.detect-shots.v1".to_owned(),
        source_id: "src_0123456789abcdef".to_owned(),
        detection: None,
    };
    let decoded = DetectShotsPayloadV1::decode(message.encode_to_vec().as_slice())
        .expect("round-trip the job payload");
    assert_eq!(decoded, message);
    assert!(decoded.detection.is_none());

    let opinionated = DetectShotsPayloadV1 {
        detection: Some(ShotDetectionV1 {
            threshold: 31.5,
            min_shot_ticks: 45_045,
            analysis_height: 180,
        }),
        ..message
    };
    assert_eq!(
        DetectShotsPayloadV1::decode(opinionated.encode_to_vec().as_slice()).expect("round-trip"),
        opinionated
    );
}
