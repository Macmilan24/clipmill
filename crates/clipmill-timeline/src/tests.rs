#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;

const SECOND: u64 = 90_000;

fn recording() -> Recording {
    let texts = [
        "So what happened when the deal fell through?",
        "We lost everything.",
        "Honestly, the pricing was wrong from day one.",
    ];
    let mut words = Vec::new();
    let mut sentences = Vec::new();
    let mut at = 0;
    for (index, text) in texts.iter().enumerate() {
        let first = words.len() as u64;
        let start = at;
        for word in text.split(' ') {
            words.push(Word {
                index: words.len() as u64,
                text: word.to_owned(),
                start_ticks: at,
                end_ticks: at + SECOND / 3,
            });
            at += SECOND / 2;
        }
        sentences.push(Sentence {
            index: index as u64,
            text: (*text).to_owned(),
            start_ticks: start,
            end_ticks: at - SECOND / 6,
            first_word_index: first,
            word_count: words.len() as u64 - first,
            utterance_index: index as u64,
        });
        at += SECOND;
    }
    Recording {
        source_fingerprint: format!("sha256:{}", "a".repeat(64)),
        language: "en".to_owned(),
        utterances: sentences
            .iter()
            .map(|sentence| Utterance {
                index: sentence.index,
                start_ticks: sentence.start_ticks,
                end_ticks: sentence.end_ticks,
                pause_before_ticks: 0,
                pause_after_ticks: if sentence.index == 1 {
                    SECOND * 2
                } else {
                    SECOND / 4
                },
            })
            .collect(),
        topics: vec![Topic {
            index: 0,
            start_ticks: 0,
            end_ticks: at,
            first_sentence_index: 0,
            sentence_count: 3,
            keywords: vec!["deal".to_owned(), "pricing".to_owned()],
        }],
        turns: vec![
            Turn {
                speaker: "S1".to_owned(),
                start_ticks: 0,
                end_ticks: sentences[0].end_ticks,
            },
            Turn {
                speaker: "S2".to_owned(),
                start_ticks: sentences[1].start_ticks,
                end_ticks: at,
            },
        ],
        shots: vec![
            Shot {
                start_ticks: 0,
                end_ticks: 4 * SECOND,
            },
            Shot {
                start_ticks: 4 * SECOND,
                end_ticks: at,
            },
        ],
        events: vec![Event {
            kind: "laughter".to_owned(),
            start_ticks: sentences[1].end_ticks,
            end_ticks: sentences[1].end_ticks + SECOND,
            value: Some(0.9),
            label: Some("S1".to_owned()),
            producer: "test".to_owned(),
        }],
        words,
        sentences,
    }
}

fn built(recording: &Recording) -> (tempfile::TempDir, std::path::PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("timeline.sqlite");
    build(&path, recording).unwrap();
    (directory, path)
}

#[test]
fn a_build_writes_every_row_and_nothing_beside_the_file() {
    let recording = recording();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("timeline.sqlite");
    let summary = build(&path, &recording).unwrap();
    assert_eq!(summary.words, recording.words.len() as u64);
    assert_eq!(summary.sentences, 3);
    assert_eq!(summary.shots, 2);
    // One measured pause (two seconds after the second sentence) and the
    // recording's own laughter.
    assert_eq!(summary.events, 2);
    let entries: Vec<_> = std::fs::read_dir(directory.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(entries, vec![std::ffi::OsString::from("timeline.sqlite")]);
}

#[test]
fn the_same_recording_builds_the_same_bytes() {
    let recording = recording();
    let (_first_dir, first) = built(&recording);
    let (_second_dir, second) = built(&recording);
    assert_eq!(
        std::fs::read(first).unwrap(),
        std::fs::read(second).unwrap()
    );
}

#[test]
fn sentences_carry_the_voice_that_speaks_most_of_them() {
    let recording = recording();
    let (_dir, path) = built(&recording);
    let index = TimelineIndex::open(&path).unwrap();
    let sentences = index.sentences_between(0, u64::MAX).unwrap();
    let speakers: Vec<_> = sentences.iter().map(|row| row.speaker.as_deref()).collect();
    assert_eq!(speakers, vec![Some("S1"), Some("S2"), Some("S2")]);
    assert_eq!(
        index
            .speaker_at(recording.sentences[2].start_ticks)
            .unwrap()
            .as_deref(),
        Some("S2")
    );
    assert_eq!(
        index.meta("source_fingerprint").unwrap(),
        Some(recording.source_fingerprint)
    );
}

#[test]
fn a_range_returns_only_what_overlaps_it() {
    let recording = recording();
    let (_dir, path) = built(&recording);
    let index = TimelineIndex::open(&path).unwrap();
    let second = &recording.sentences[1];
    let rows = index
        .sentences_between(second.start_ticks, second.start_ticks + 1)
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].text, "We lost everything.");
    assert_eq!(index.shots_between(0, SECOND).unwrap().len(), 1);
}

#[test]
fn pauses_and_reactions_are_events_in_time_order() {
    let recording = recording();
    let (_dir, path) = built(&recording);
    let index = TimelineIndex::open(&path).unwrap();
    let events = index.events_between(0, u64::MAX, &[]).unwrap();
    let kinds: Vec<_> = events.iter().map(|event| event.kind.as_str()).collect();
    assert_eq!(kinds, vec!["laughter", "pause"]);
    let pauses = index.events_between(0, u64::MAX, &["pause"]).unwrap();
    assert_eq!(pauses.len(), 1);
    assert!((pauses[0].value.unwrap() - 2.0).abs() < 1e-9);
}

#[test]
fn search_finds_words_and_phrases_ignoring_case() {
    let (_dir, path) = built(&recording());
    let index = TimelineIndex::open(&path).unwrap();
    let hits = index.search("PRICING", 5).unwrap();
    assert_eq!(hits.len(), 1);
    assert!(hits[0].text.contains("pricing"));
    assert_eq!(index.search("\"lost everything\"", 5).unwrap().len(), 1);
    assert!(index.search("\"everything lost\"", 5).unwrap().is_empty());
}

#[test]
fn search_treats_operators_and_symbols_as_text() {
    let (_dir, path) = built(&recording());
    let index = TimelineIndex::open(&path).unwrap();
    for hostile in [
        "deal OR",
        "NEAR(deal pricing)",
        "deal*",
        "^deal",
        "text:deal",
        "\"",
        "(",
        "-",
    ] {
        // Never a syntax error, whatever was typed.
        index.search(hostile, 5).unwrap();
    }
    assert!(index.search("  ", 5).unwrap().is_empty());
}

#[test]
fn an_index_of_another_version_is_refused() {
    let (_dir, path) = built(&recording());
    {
        let connection = Connection::open(&path).unwrap();
        connection
            .execute(
                "UPDATE meta SET value = 'clipmill.timeline.index.v0' WHERE key = 'schema_version'",
                [],
            )
            .unwrap();
    }
    assert!(matches!(
        TimelineIndex::open(&path),
        Err(TimelineError::Version { .. })
    ));
}

#[test]
fn evidence_out_of_time_order_is_refused() {
    let mut recording = recording();
    recording.shots.reverse();
    let directory = tempfile::tempdir().unwrap();
    assert!(matches!(
        build(&directory.path().join("timeline.sqlite"), &recording),
        Err(TimelineError::Invalid(_))
    ));
}

#[test]
fn the_index_is_read_only() {
    let (_dir, path) = built(&recording());
    let index = TimelineIndex::open(&path).unwrap();
    assert!(index.connection.execute("DELETE FROM words", []).is_err());
}
