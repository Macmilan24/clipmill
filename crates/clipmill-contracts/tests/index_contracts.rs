//! The evidence index's contract, Rust leg.
//!
//! The document is derived, not authored, so what these assert is the shape a
//! consumer codes against and the two things the derivation must never be
//! allowed to drop: the provenance link back to the words, and the difference
//! between a boundary the recognizer punctuated and one that is merely where
//! the speaker stopped.
//!
//! Tests may panic; the workspace deny targets production code.
#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use clipmill_contracts::proto::ipc::v1::{IndexStagePayloadV1, IndexTranscriptPayloadV1};
use clipmill_contracts::schemas::index_transcript::{IndexTranscript, SentenceTerminator};
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

fn roundtrip(rel: &str) -> IndexTranscript {
    let raw = read(rel);
    let parsed: IndexTranscript = match serde_json::from_str(&raw) {
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
fn every_valid_index_fixture_roundtrips_canonically() {
    roundtrip("contracts/fixtures/index.transcript/valid/ten_words.json");
    roundtrip("contracts/fixtures/index.transcript/valid/interpolated_timing.json");
}

/// Rule 14.1: a claim nobody can walk back to an observation is a claim the
/// system should not be making. Every unit here names a word range, and the
/// ranges tile the transcript rather than sampling it.
#[test]
fn every_unit_names_the_words_behind_it() {
    let index = roundtrip("contracts/fixtures/index.transcript/valid/ten_words.json");
    let mut next = 0;
    for utterance in &index.utterances {
        assert_eq!(utterance.first_word_index, next);
        next += utterance.word_count.get();
    }
    let mut next = 0;
    for sentence in &index.sentences {
        assert_eq!(sentence.first_word_index, next);
        next += sentence.word_count.get();
        assert!(sentence.utterance_index < u64::try_from(index.utterances.len()).unwrap());
    }
    let mut next = 0;
    for topic in &index.topics {
        assert_eq!(topic.first_sentence_index, next);
        next += topic.sentence_count.get();
    }
    assert_eq!(next, u64::try_from(index.sentences.len()).unwrap());
}

/// The boundary optimizer weighs these differently, so the document has to
/// keep them apart. A type that collapsed them would make that impossible
/// downstream and undetectable here.
#[test]
fn a_punctuated_boundary_is_distinguishable_from_a_speaker_who_stopped() {
    let index = roundtrip("contracts/fixtures/index.transcript/valid/ten_words.json");
    assert!(index.sentences.iter().all(|sentence| matches!(
        sentence.terminator,
        SentenceTerminator::Punctuation
            | SentenceTerminator::UtteranceEnd
            | SentenceTerminator::CoverageEnd
    )));
    assert!(
        index
            .sentences
            .iter()
            .any(|sentence| matches!(sentence.terminator, SentenceTerminator::Punctuation))
    );
}

/// The segmentation parameters travel with the result. Without them a topic
/// list is a number nobody can reproduce or re-tune from.
#[test]
fn the_index_states_what_it_was_segmented_with() {
    let index = roundtrip("contracts/fixtures/index.transcript/valid/ten_words.json");
    assert_eq!(index.segmentation.stopwords.as_str(), "english-minimal.v1");
    assert!(index.segmentation.block_sentences.get() >= 1);
    assert!(index.segmentation.utterance_gap_ticks > 0);
}

/// An index over interpolated word timing inherits that uncertainty, and must
/// carry it rather than presenting derived structure as though it were solid.
#[test]
fn interpolated_timing_survives_into_the_index() {
    let index = roundtrip("contracts/fixtures/index.transcript/valid/interpolated_timing.json");
    assert!(
        !index.invalid_regions.is_empty(),
        "the transcript disowned a span and the index did not"
    );
}

#[test]
fn invalid_index_fixtures_are_rejected() {
    for (fixture, why) in [
        ("float_ticks", "float ticks must not parse (D06)"),
        (
            "sentence_with_no_words",
            "a sentence with no words is not a sentence",
        ),
        (
            "unknown_terminator",
            "an unlisted terminator must not parse",
        ),
        (
            "sentence_without_provenance",
            "a unit that names no words must not parse",
        ),
    ] {
        let rejected = serde_json::from_str::<IndexTranscript>(&read(&format!(
            "contracts/fixtures/index.transcript/invalid/{fixture}.json"
        )));
        assert!(rejected.is_err(), "{why}");
    }
}

/// The stage payload names two content addresses and nothing else. A path in
/// here would give the same transcript two indexes on two machines; a second
/// copy of the source fingerprint would be a fact that could disagree with the
/// document it came from.
#[test]
fn the_index_stage_payload_carries_addresses_and_nothing_machine_specific() {
    let message = IndexStagePayloadV1 {
        key_version: "clipmill.index-stage.v1".to_owned(),
        stage: "index-transcript".to_owned(),
        transcript_artifact_id: "sha256:".to_owned() + &"a".repeat(64),
        shots_artifact_id: String::new(),
    };
    let decoded =
        IndexStagePayloadV1::decode(message.encode_to_vec().as_slice()).expect("round-trip");
    assert_eq!(decoded, message);
    let encoded = String::from_utf8_lossy(&message.encode_to_vec()).into_owned();
    assert!(!encoded.contains('/'), "the keyed payload carries a path");

    // A source with no video keys differently from one whose cuts were found.
    let with_shots = IndexStagePayloadV1 {
        shots_artifact_id: "sha256:".to_owned() + &"b".repeat(64),
        ..message.clone()
    };
    assert_ne!(message.encode_to_vec(), with_shots.encode_to_vec());
}

#[test]
fn the_index_job_payload_names_only_a_source() {
    let message = IndexTranscriptPayloadV1 {
        key_version: "clipmill.index-transcript.v1".to_owned(),
        source_id: "src_0123456789abcdef".to_owned(),
    };
    assert_eq!(
        IndexTranscriptPayloadV1::decode(message.encode_to_vec().as_slice()).expect("round-trip"),
        message
    );
}
