//! The audio events document, Rust leg: the reactions a transcript cannot show
//! parse into the generated type, round-trip canonically, and a document that
//! times an event in fractions or names a kind nobody listens for is refused.
//!
//! Tests may panic; the workspace deny targets production code.
#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use clipmill_contracts::schemas::audio_events::AudioEvents;

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

#[test]
fn audio_events_parse_and_roundtrip_canonically() {
    for rel in [
        "contracts/fixtures/audio.events/valid/delivery_accent.json",
        "contracts/fixtures/audio.events/valid/live_talk.json",
        "contracts/fixtures/audio.events/valid/nothing_heard.json",
    ] {
        let raw = read(rel);
        let parsed: AudioEvents =
            serde_json::from_str(&raw).unwrap_or_else(|err| panic!("{rel} rejected: {err}"));
        let value = serde_json::to_value(&parsed).unwrap();
        assert_eq!(
            canonical(&value),
            raw,
            "{rel} must round-trip byte-for-byte"
        );
    }
}

#[test]
fn a_covered_recording_with_no_events_says_nothing_was_heard() {
    let raw = read("contracts/fixtures/audio.events/valid/nothing_heard.json");
    let parsed: AudioEvents = serde_json::from_str(&raw).unwrap();
    assert!(parsed.coverage.analyzed);
    assert!(parsed.events.is_empty());
}

#[test]
fn malformed_audio_events_are_refused() {
    for rel in [
        "contracts/fixtures/audio.events/invalid/float_ticks.json",
        "contracts/fixtures/audio.events/invalid/unknown_kind.json",
    ] {
        assert!(
            serde_json::from_str::<AudioEvents>(&read(rel)).is_err(),
            "{rel} must fail to parse into the typed contract"
        );
    }
}
