//! The fan-in that closes an analysis, Rust leg.
//!
//! One document naming every observation derived from a source. Two things it
//! has to keep straight, and both are about absence: a stage that produced
//! nothing because the source had nothing for it to read must stay
//! distinguishable from a stage nobody ran, and the span the analysis speaks for
//! must be the span its stages actually examined rather than the source's
//! duration.
//!
//! Tests may panic; the workspace deny targets production code.
#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use clipmill_contracts::proto::ipc::v1::{
    AnalysisStagePayloadV1, AnalyzeSourcePayloadV1, SkippedStageV1,
};
use clipmill_contracts::schemas::analysis_manifest::{AnalysisManifest, StageKind};
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

fn roundtrip(name: &str) -> AnalysisManifest {
    let rel = format!("contracts/fixtures/analysis.manifest/valid/{name}.json");
    let raw = read(&rel);
    let parsed: AnalysisManifest = match serde_json::from_str(&raw) {
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
fn every_valid_analysis_fixture_roundtrips_canonically() {
    for name in [
        "interview",
        "audio_only",
        "silent_footage",
        "partial_coverage",
    ] {
        roundtrip(name);
    }
}

/// A complete analysis names every stage exactly once, in the order they ran.
/// One read instead of nine is the whole reason a shell asks for this document.
#[test]
fn a_complete_analysis_names_every_stage_in_order() {
    let manifest = roundtrip("interview");
    let kinds = manifest
        .stages
        .iter()
        .map(|stage| stage.kind)
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        vec![
            StageKind::EvidenceSourceMapV1,
            StageKind::MediaIngestManifestV1,
            StageKind::SpeechVadV1,
            StageKind::SpeechAsrV1,
            StageKind::SpeechAlignmentV1,
            StageKind::SpeechTranscriptV1,
            StageKind::EvidenceShotsV1,
            StageKind::IndexTranscriptV1,
            StageKind::DiscoveryCandidatesV1,
            StageKind::RankingSetV1,
        ]
    );
    assert!(
        manifest.skipped.is_empty(),
        "nothing was skipped, so nothing is listed"
    );
}

/// The distinction the skip list exists for. A recording with no video is absent
/// from the stage list *and* present in the skip list with the reason — so a
/// consumer can tell it apart from an analysis whose shot detection never ran,
/// without opening a single artifact.
#[test]
fn a_skipped_stage_is_absent_from_the_stages_and_named_with_a_reason() {
    let manifest = roundtrip("audio_only");
    assert!(
        !manifest
            .stages
            .iter()
            .any(|stage| stage.kind == StageKind::EvidenceShotsV1),
        "a skipped stage must not appear as one that produced something"
    );
    assert_eq!(manifest.skipped.len(), 1);
    assert_eq!(manifest.skipped[0].kind.as_str(), "evidence.shots.v1");
    // The reason is an enumerated property of the source, not free text: a
    // consumer has to be able to act on it.
    assert_eq!(
        serde_json::to_value(manifest.skipped[0].reason).unwrap(),
        serde_json::json!("no_video")
    );
}

/// A source with no audio loses the four speech stages and the three that read a
/// transcript. Seven absences, each with the same reason, and shot detection
/// still ran — which is what makes this a partial analysis rather than a failure.
#[test]
fn a_silent_source_skips_everything_downstream_of_speech() {
    let manifest = roundtrip("silent_footage");
    assert_eq!(manifest.skipped.len(), 7);
    assert!(
        manifest
            .stages
            .iter()
            .any(|stage| stage.kind == StageKind::EvidenceShotsV1)
    );
    for skipped in &manifest.skipped {
        assert_eq!(
            serde_json::to_value(skipped.reason).unwrap(),
            serde_json::json!("no_audio")
        );
    }
}

/// Coverage is what the stages examined, and `analyzed` is separate from the
/// span. A range that starts after zero and a false flag are both real states: a
/// consumer reading a candidate outside this range is reading a claim nobody made.
#[test]
fn coverage_states_the_examined_span_and_whether_it_was_analyzed() {
    let whole = roundtrip("interview");
    assert_eq!(whole.coverage.start_ticks, 0);
    assert!(whole.coverage.analyzed);

    let partial = roundtrip("partial_coverage");
    assert_eq!(partial.coverage.start_ticks, 90_000);
    assert!(
        !partial.coverage.analyzed,
        "a stage that read part of the recording cannot claim the whole of it"
    );
    assert!(partial.coverage.end_ticks > partial.coverage.start_ticks);
}

/// The refusals typify generates. `minItems` is not among them — it is carried as
/// documentation — so `no_stages` is asserted in the Python leg instead of being
/// listed here as a refusal this type does not make.
#[test]
fn every_invalid_analysis_fixture_is_refused() {
    for (fixture, why) in [
        ("unknown_stage", "a stage nobody registered"),
        (
            "path_where_an_address_belongs",
            "a path is machine-specific",
        ),
        (
            "coverage_without_analyzed",
            "an empty result and a pass that never ran would read the same",
        ),
        ("skipped_without_reason", "absent without why"),
        ("unknown_skip_reason", "a reason a consumer cannot act on"),
        ("float_ticks", "ticks are integers at 1/90000"),
    ] {
        let rejected = serde_json::from_str::<AnalysisManifest>(&read(&format!(
            "contracts/fixtures/analysis.manifest/invalid/{fixture}.json"
        )));
        assert!(rejected.is_err(), "{why}");
    }
}

/// The fan-in's payload carries the skip list and nothing else that varies. The
/// artifacts it names arrive on the lease, so two analyses that ran the same
/// stages over the same source encode the same bytes here.
#[test]
fn the_analysis_stage_payload_carries_the_skips_and_no_inputs() {
    let message = AnalysisStagePayloadV1 {
        key_version: "clipmill.analysis-stage.v1".to_owned(),
        stage: "analysis-manifest".to_owned(),
        source_fingerprint: "sha256:".to_owned() + &"1".repeat(64),
        skipped: vec![SkippedStageV1 {
            kind: "evidence.shots.v1".to_owned(),
            reason: "no_video".to_owned(),
        }],
    };
    assert_eq!(
        AnalysisStagePayloadV1::decode(message.encode_to_vec().as_slice()).expect("round-trip"),
        message
    );
    let encoded = String::from_utf8_lossy(&message.encode_to_vec()).into_owned();
    assert!(!encoded.contains('/'), "the keyed payload carries a path");

    // An analysis that skipped a stage is a different analysis from one that ran
    // it, even when every artifact it does name is identical — so the skip list
    // has to move the bytes the key is computed from.
    let complete = AnalysisStagePayloadV1 {
        skipped: Vec::new(),
        ..message.clone()
    };
    assert_ne!(message.encode_to_vec(), complete.encode_to_vec());
}

/// The job request names a source and what to ask of it, and no local path.
#[test]
fn the_analyze_job_payload_names_a_source_and_a_request() {
    let message = AnalyzeSourcePayloadV1 {
        key_version: "clipmill.analyze-source.v1".to_owned(),
        source_id: "src_0123456789abcdef".to_owned(),
        language: "en".to_owned(),
        duration: None,
        count: 0,
        diversity_milli: 0,
    };
    assert_eq!(
        AnalyzeSourcePayloadV1::decode(message.encode_to_vec().as_slice()).expect("round-trip"),
        message
    );
    let encoded = String::from_utf8_lossy(&message.encode_to_vec()).into_owned();
    assert!(!encoded.contains('/'), "a path reached the job payload");
}
