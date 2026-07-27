//! The index over the transcripts the contracts already publish.
//!
//! The unit tests use transcripts written for one property each. This is the
//! other half: the committed `speech.transcript.v1` fixtures — the documents
//! every other language's contract tests read — indexed end to end, checked
//! against a golden result, and checked against the invariants a consumer is
//! allowed to assume without looking.
//!
//! Regenerate the goldens with `CLIPMILL_BLESS=1 cargo test -p
//! clipmill-evidence`. Do it deliberately: a golden that changes is a change
//! in what the system says about a recording, and the diff is the review.
//!
//! Tests may panic; the workspace deny targets production code.
#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path, PathBuf};

use clipmill_contracts::schemas::index_transcript::IndexTranscript;
use clipmill_contracts::schemas::speech_transcript::SpeechTranscript;
use clipmill_evidence::{Inputs, Parameters, index};

const TRANSCRIPT_ID: &str =
    "sha256:7a11000000000000000000000000000000000000000000000000000000000042";
const IMPLEMENTATION: &str = "clipmill-evidence-index@1.0.0";

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn canonical(value: &serde_json::Value) -> String {
    let mut text = serde_json::to_string_pretty(value).unwrap_or_else(|error| panic!("{error}"));
    text.push('\n');
    text
}

fn transcripts() -> Vec<(String, SpeechTranscript)> {
    let directory = repo().join("contracts/fixtures/speech.transcript/valid");
    let mut entries = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("cannot list {}: {error}", directory.display()))
        .map(|entry| entry.expect("a readable directory entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect::<Vec<_>>();
    entries.sort();
    assert!(!entries.is_empty(), "there are no transcript fixtures");
    entries
        .into_iter()
        .map(|path| {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("a fixture name")
                .to_owned();
            let raw = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
            let parsed = serde_json::from_str(&raw)
                .unwrap_or_else(|error| panic!("{name} is not a transcript: {error}"));
            (name, parsed)
        })
        .collect()
}

fn indexed(transcript: &SpeechTranscript) -> IndexTranscript {
    index(
        transcript,
        None,
        Inputs {
            transcript: TRANSCRIPT_ID,
            shots: None,
        },
        Parameters::DEFAULT,
        IMPLEMENTATION,
    )
    .expect("the index builds")
}

#[test]
fn every_published_transcript_indexes_to_its_golden() {
    let bless = std::env::var_os("CLIPMILL_BLESS").is_some();
    let directory = repo().join("contracts/fixtures/index.transcript/valid");
    for (name, transcript) in transcripts() {
        let produced = canonical(&serde_json::to_value(indexed(&transcript)).expect("serializes"));
        let golden = directory.join(&name);
        if bless {
            std::fs::create_dir_all(&directory).expect("the fixture directory");
            std::fs::write(&golden, &produced).expect("the golden is writable");
            continue;
        }
        let expected = std::fs::read_to_string(&golden).unwrap_or_else(|error| {
            panic!(
                "cannot read {}: {error}\nrun CLIPMILL_BLESS=1 cargo test -p clipmill-evidence",
                golden.display()
            )
        });
        assert_eq!(produced, expected, "the index over {name} changed");
    }
}

/// What a consumer is allowed to assume without opening the transcript. Every
/// one of these is load-bearing for discovery, and none of them is enforced by
/// the schema, which can express shapes but not arithmetic.
#[test]
fn every_index_holds_the_invariants_discovery_relies_on() {
    for (name, transcript) in transcripts() {
        let result = indexed(&transcript);
        let words = u64::try_from(transcript.words.len()).unwrap();
        let sentences = u64::try_from(result.sentences.len()).unwrap();

        // Utterances tile the word list: in order, no gaps, no overlaps.
        let mut next = 0u64;
        for utterance in &result.utterances {
            assert_eq!(utterance.first_word_index, next, "{name}");
            next += utterance.word_count.get();
            assert!(
                utterance.start_ticks >= result.coverage.start_ticks,
                "{name}"
            );
            assert!(utterance.end_ticks <= result.coverage.end_ticks, "{name}");
            assert!(utterance.start_ticks <= utterance.end_ticks, "{name}");
        }
        assert_eq!(next, words, "{name}: the utterances lost a word");

        // Sentences tile them too, and each names the utterance it sits in.
        let mut next = 0u64;
        for sentence in &result.sentences {
            assert_eq!(sentence.first_word_index, next, "{name}");
            next += sentence.word_count.get();
            assert!(
                sentence.utterance_index < u64::try_from(result.utterances.len()).unwrap(),
                "{name}"
            );
            assert!(
                sentence.start_ticks >= result.coverage.start_ticks,
                "{name}"
            );
            assert!(sentence.end_ticks <= result.coverage.end_ticks, "{name}");
        }
        assert_eq!(next, words, "{name}: the sentences lost a word");

        // Topics tile the sentences.
        let mut next = 0u64;
        for topic in &result.topics {
            assert_eq!(topic.first_sentence_index, next, "{name}");
            next += topic.sentence_count.get();
        }
        assert_eq!(next, sentences, "{name}: the topics lost a sentence");

        // Every edge lies inside coverage, and the list is ordered.
        let mut previous = (0u64, 0u64);
        for edge in &result.edges {
            assert!(edge.start_ticks >= result.coverage.start_ticks, "{name}");
            assert!(edge.end_ticks <= result.coverage.end_ticks, "{name}");
            assert!(edge.start_ticks <= edge.end_ticks, "{name}");
            assert!((edge.start_ticks, edge.end_ticks) >= previous, "{name}");
            previous = (edge.start_ticks, edge.end_ticks);
        }

        // The index adds no uncertainty of its own, and hides none either.
        assert_eq!(
            result.invalid_regions.len(),
            transcript.invalid_regions.len(),
            "{name}: the index changed what the transcript disowned"
        );
    }
}

#[test]
fn indexing_the_same_transcript_twice_produces_the_same_bytes() {
    for (name, transcript) in transcripts() {
        let first = canonical(&serde_json::to_value(indexed(&transcript)).expect("serializes"));
        let second = canonical(&serde_json::to_value(indexed(&transcript)).expect("serializes"));
        assert_eq!(first, second, "{name} did not index deterministically");
    }
}

/// The provenance rule: a claim nobody can walk back to an observation is a
/// claim the system should not be making.
#[test]
fn every_unit_resolves_to_the_words_it_came_from() {
    for (name, transcript) in transcripts() {
        let result = indexed(&transcript);
        for sentence in &result.sentences {
            let first = usize::try_from(sentence.first_word_index).unwrap();
            let count = usize::try_from(sentence.word_count.get()).unwrap();
            let members = &transcript.words[first..first + count];
            assert_eq!(
                members.first().unwrap().start_ticks,
                sentence.start_ticks,
                "{name}"
            );
            assert_eq!(
                members.last().unwrap().end_ticks,
                sentence.end_ticks,
                "{name}"
            );
            let rebuilt = members
                .iter()
                .map(|word| word.text.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            assert_eq!(rebuilt, sentence.text.as_str(), "{name}");
        }
    }
}
