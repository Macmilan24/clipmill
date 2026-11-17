#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::{Value, json};

use super::*;
use crate::SECOND;
use crate::script::Sentence;

/// A hundred sentences, six seconds apart.
fn recording() -> Source {
    Source {
        duration_ticks: 600 * SECOND,
        sentences: (0..100)
            .map(|i| Sentence {
                index: i,
                start_ticks: i * 6 * SECOND,
                end_ticks: i * 6 * SECOND + 5 * SECOND,
                speaker: Some(if i % 2 == 0 { "host" } else { "guest" }.to_owned()),
                text: format!("Sentence {i}."),
            })
            .collect(),
        ..Source::default()
    }
}

/// The podcast fixture, with every time set from the units it cites.
fn fixture() -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts/fixtures/mind.memory/valid/podcast.json");
    let mut value: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    for list in ["chapters", "events"] {
        for item in value[list].as_array_mut().unwrap() {
            retime(item);
        }
    }
    value
}

fn unit(value: &Value) -> u64 {
    value.as_str().unwrap()[1..].parse().unwrap()
}

fn retime(item: &mut Value) {
    let (first, last) = (unit(&item["first_unit"]), unit(&item["last_unit"]));
    item["start_ticks"] = json!(first * 6 * SECOND);
    item["end_ticks"] = json!(last * 6 * SECOND + 5 * SECOND);
}

fn codes(value: &Value) -> Vec<&'static str> {
    let memory: MindMemory = serde_json::from_value(value.clone()).unwrap();
    check(&memory, &recording())
        .into_iter()
        .map(|problem| problem.code)
        .collect()
}

#[test]
fn a_memory_that_cites_the_recording_correctly_passes() {
    assert_eq!(codes(&fixture()), Vec::<&str>::new());
}

#[test]
fn a_payoff_before_its_setup_is_refused() {
    let mut memory = fixture();
    memory["links"][1]["from_event"] = json!("m4");
    memory["links"][1]["to_event"] = json!("m3");
    assert_eq!(codes(&memory), ["backwards_link"]);

    memory["links"][1]["kind"] = json!("contradiction");
    assert_eq!(codes(&memory), Vec::<&str>::new());
}

#[test]
fn an_event_outside_its_chapter_is_refused() {
    let mut memory = fixture();
    memory["events"][3]["chapter_id"] = json!("c1");
    assert_eq!(codes(&memory), ["outside_chapter"]);
    memory["events"][3]["chapter_id"] = json!("c9");
    assert_eq!(codes(&memory), ["unknown_chapter"]);
}

#[test]
fn units_must_exist_and_carry_their_times() {
    let mut memory = fixture();
    memory["events"][2]["last_unit"] = json!("s400");
    memory["events"][1]["end_ticks"] = json!(1);
    let found = codes(&memory);
    assert!(found.contains(&"unknown_unit"), "{found:?}");
    assert!(found.contains(&"wrong_times"), "{found:?}");
    // The link to the event that failed now has nothing to stand on.
    assert!(found.contains(&"unknown_event"), "{found:?}");
}

#[test]
fn chapters_may_not_overlap() {
    let mut memory = fixture();
    memory["chapters"][1]["first_unit"] = json!("s40");
    retime(&mut memory["chapters"][1]);
    assert!(codes(&memory).contains(&"chapters_overlap"));
}

#[test]
fn references_resolve_both_ways() {
    let mut memory = fixture();
    memory["events"][0]["entities"] = json!(["e7"]);
    memory["entities"][1]["events"] = json!(["m9"]);
    memory["events"][1]["event_id"] = json!("m1");
    let found = codes(&memory);
    for code in ["unknown_entity", "unknown_event", "duplicate_id"] {
        assert!(found.contains(&code), "{code} missing from {found:?}");
    }
}

#[test]
fn a_link_between_overlapping_events_joins_nothing() {
    let mut memory = fixture();
    memory["links"][0]["to_event"] = json!("m1");
    assert_eq!(codes(&memory), ["linked_to_itself"]);
}
