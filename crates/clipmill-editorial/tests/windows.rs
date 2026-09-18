//! The windows cut from the indexes the contracts already publish.
//!
//! The unit tests in `windows.rs` cut word counts written for one property
//! each. This is the other half: the committed `index.transcript.v1` fixtures
//! cut end to end under the default budget, compared against a golden that is
//! itself the published `editorial.windows.v1` fixture, and checked against
//! the guarantees a proposer is allowed to assume without looking.
//!
//! Regenerate with `CLIPMILL_BLESS=1 cargo test -p clipmill-editorial`. Do it
//! deliberately: a golden that changes is a change in what a model is shown,
//! and the diff is the review.
//!
//! Tests may panic; the workspace deny targets production code.
#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::too_many_lines
)]

use std::path::{Path, PathBuf};

use clipmill_contracts::schemas::editorial_windows::{EditorialWindows, InvalidRegionReason};
use clipmill_contracts::schemas::index_transcript::IndexTranscript;
use clipmill_editorial::{Budget, Inputs, STAGE, WindowsError, windows};

const INDEX_ID: &str = "sha256:1de0000000000000000000000000000000000000000000000000000000000011";
const TRANSCRIPT_ID: &str =
    "sha256:7a11000000000000000000000000000000000000000000000000000000000042";
const IMPLEMENTATION: &str = "clipmill-editorial-windows@1.0.0";

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn canonical(value: &serde_json::Value) -> String {
    let mut text = serde_json::to_string_pretty(value).unwrap_or_else(|error| panic!("{error}"));
    text.push('\n');
    text
}

fn read<T: serde::de::DeserializeOwned>(path: &Path) -> T {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    serde_json::from_str(&raw)
        .unwrap_or_else(|error| panic!("{} is not what it claims: {error}", path.display()))
}

/// Every committed index, by fixture name.
fn indexes() -> Vec<(String, IndexTranscript)> {
    let directory = repo().join("contracts/fixtures/index.transcript/valid");
    let mut names = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("cannot list {}: {error}", directory.display()))
        .filter_map(|entry| {
            let path = entry.expect("a readable entry").path();
            (path.extension()? == "json")
                .then(|| path.file_name()?.to_str().map(ToOwned::to_owned))?
        })
        .collect::<Vec<_>>();
    names.sort();
    assert!(!names.is_empty(), "there are no index fixtures");
    names
        .into_iter()
        .map(|name| {
            let index = read(&directory.join(&name));
            (name, index)
        })
        .collect()
}

fn interview() -> IndexTranscript {
    read(&repo().join("contracts/fixtures/index.transcript/valid/interview.json"))
}

fn cut(index: &IndexTranscript, budget: Budget) -> EditorialWindows {
    windows(
        index,
        Inputs {
            index: INDEX_ID,
            transcript: TRANSCRIPT_ID,
        },
        budget,
        IMPLEMENTATION,
    )
    .expect("the cut runs")
}

/// A window as the tests describe one: core position, words, topics, context.
#[derive(Debug, PartialEq, Eq)]
struct Shape {
    first: u64,
    count: u64,
    words: u64,
    topics: Vec<u64>,
    before: u64,
    after: u64,
}

fn shapes(document: &EditorialWindows) -> Vec<Shape> {
    document
        .windows
        .iter()
        .map(|window| Shape {
            first: window.first_sentence_index,
            count: window.sentence_count.get(),
            words: window.word_count.get(),
            topics: window.topic_indexes.clone(),
            before: window.context_before_sentences,
            after: window.context_after_sentences,
        })
        .collect()
}

#[test]
fn every_published_index_cuts_to_its_golden() {
    let bless = std::env::var_os("CLIPMILL_BLESS").is_some();
    let directory = repo().join("contracts/fixtures/editorial.windows/valid");
    for (name, index) in indexes() {
        let produced =
            canonical(&serde_json::to_value(cut(&index, Budget::DEFAULT)).expect("serializes"));
        let golden = directory.join(&name);
        if bless {
            std::fs::create_dir_all(&directory).expect("the fixture directory");
            std::fs::write(&golden, &produced).expect("the golden is writable");
            continue;
        }
        let expected = std::fs::read_to_string(&golden).unwrap_or_else(|error| {
            panic!(
                "cannot read {}: {error}\nrun CLIPMILL_BLESS=1 cargo test -p clipmill-editorial",
                golden.display()
            )
        });
        assert_eq!(produced, expected, "the windows over {name} changed");
    }
}

/// What a proposer may assume of any windows document without reading the
/// index it came from. None of it is enforced by the schema, which can express
/// shapes but not arithmetic; all of it is what makes a sentence index the
/// model answers with resolvable.
#[test]
fn every_cut_holds_the_guarantees_a_proposer_relies_on() {
    // The default budget, and a small one that forces several windows over
    // recordings a few sentences long.
    let small = Budget {
        target_words: 40,
        overlap_words: 10,
        context_sentences: 2,
    };
    for (name, index) in indexes() {
        for budget in [Budget::DEFAULT, small] {
            let document = cut(&index, budget);
            let sentences = &document.sentences;

            // The sentences are the index's, in the index's order, and the
            // document repeats the index's own account of itself.
            assert_eq!(sentences.len(), index.sentences.len(), "{name}");
            for (mine, theirs) in sentences.iter().zip(&index.sentences) {
                assert_eq!(mine.index, theirs.index, "{name}");
                assert_eq!(mine.start_ticks, theirs.start_ticks, "{name}");
                assert_eq!(mine.end_ticks, theirs.end_ticks, "{name}");
                assert_eq!(mine.first_word_index, theirs.first_word_index, "{name}");
                assert_eq!(mine.word_count, theirs.word_count, "{name}");
                assert_eq!(mine.text.as_str(), theirs.text.as_str(), "{name}");
            }
            assert_eq!(
                document.language.as_str(),
                index.language.as_str(),
                "{name}"
            );
            assert_eq!(
                document.coverage.start_ticks, index.coverage.start_ticks,
                "{name}"
            );
            assert_eq!(
                document.coverage.end_ticks, index.coverage.end_ticks,
                "{name}"
            );
            assert_eq!(document.producer.stage.as_str(), STAGE, "{name}");
            assert_eq!(
                document.producer.implementation.as_str(),
                IMPLEMENTATION,
                "{name}"
            );
            assert_eq!(
                document.budget.target_words.get(),
                budget.target_words,
                "{name}"
            );
            assert_eq!(
                document.budget.overlap_words, budget.overlap_words,
                "{name}"
            );
            assert_eq!(
                document.budget.context_sentences, budget.context_sentences,
                "{name}"
            );

            // The outline is the index's topics, one line each, in order.
            assert_eq!(document.outline.len(), index.topics.len(), "{name}");
            for (line, topic) in document.outline.iter().zip(&index.topics) {
                assert_eq!(line.topic_index, topic.index, "{name}");
                assert_eq!(
                    line.first_sentence_index, topic.first_sentence_index,
                    "{name}"
                );
                assert_eq!(line.sentence_count, topic.sentence_count, "{name}");
                assert_eq!(line.keywords.len(), topic.keywords.len(), "{name}");
            }

            // Every sentence is in some window's core, the windows are in
            // order, the first starts at the beginning and the last ends at
            // the end.
            let total = u64::try_from(sentences.len()).unwrap();
            let mut covered = vec![false; sentences.len()];
            for (position, window) in document.windows.iter().enumerate() {
                assert_eq!(window.index, u64::try_from(position).unwrap(), "{name}");
                let first = window.first_sentence_index;
                let last = first + window.sentence_count.get();
                assert!(last <= total, "{name}: window past the end");
                for sentence in first..last {
                    covered[usize::try_from(sentence).unwrap()] = true;
                }
                let core =
                    &sentences[usize::try_from(first).unwrap()..usize::try_from(last).unwrap()];
                // The window's ticks and words are its core's, exactly.
                assert_eq!(window.start_ticks, core[0].start_ticks, "{name}");
                assert_eq!(window.end_ticks, core[core.len() - 1].end_ticks, "{name}");
                assert_eq!(
                    window.word_count.get(),
                    core.iter()
                        .map(|sentence| sentence.word_count.get())
                        .sum::<u64>(),
                    "{name}"
                );
                // Context never exceeds the budget or what is there to read.
                assert!(
                    window.context_before_sentences <= budget.context_sentences,
                    "{name}"
                );
                assert!(window.context_before_sentences <= first, "{name}");
                assert!(
                    window.context_after_sentences <= budget.context_sentences,
                    "{name}"
                );
                assert!(window.context_after_sentences <= total - last, "{name}");
                assert_eq!(
                    window.context_before_sentences,
                    first.min(budget.context_sentences),
                    "{name}: context before is what is there, up to the budget"
                );
                assert_eq!(
                    window.context_after_sentences,
                    (total - last).min(budget.context_sentences),
                    "{name}: context after is what is there, up to the budget"
                );
                // A window names every topic its core touches and no other.
                let touching: Vec<u64> = index
                    .topics
                    .iter()
                    .filter(|topic| {
                        let start = topic.first_sentence_index;
                        let end = start + topic.sentence_count.get();
                        start < last && end > first
                    })
                    .map(|topic| topic.index)
                    .collect();
                assert_eq!(window.topic_indexes, touching, "{name}");
            }
            if sentences.is_empty() {
                assert!(document.windows.is_empty(), "{name}: windows over nothing");
                continue;
            }
            assert!(
                covered.iter().all(|seen| *seen),
                "{name}: a sentence no window holds"
            );
            assert_eq!(document.windows[0].first_sentence_index, 0, "{name}");
            let final_window = document.windows.last().unwrap();
            assert_eq!(
                final_window.first_sentence_index + final_window.sentence_count.get(),
                total,
                "{name}: the last window ends before the end"
            );

            // Consecutive windows overlap by whole sentences: by at least the
            // budget's words, unless the earlier window is too short to give
            // that many — then by everything past its first sentence.
            for pair in document.windows.windows(2) {
                let (earlier, later) = (&pair[0], &pair[1]);
                let earlier_first = earlier.first_sentence_index;
                let earlier_last = earlier_first + earlier.sentence_count.get();
                let later_first = later.first_sentence_index;
                assert!(later_first > earlier_first, "{name}: no progress");
                assert!(
                    later_first < earlier_last,
                    "{name}: a seam nobody reads whole"
                );
                let shared: u64 = sentences
                    [usize::try_from(later_first).unwrap()..usize::try_from(earlier_last).unwrap()]
                    .iter()
                    .map(|sentence| sentence.word_count.get())
                    .sum();
                assert!(
                    shared >= budget.overlap_words || later_first == earlier_first + 1,
                    "{name}: {shared} shared words under the budget of {}",
                    budget.overlap_words
                );
            }
        }
    }
}

#[test]
fn a_small_budget_cuts_the_interview_where_the_arithmetic_says() {
    // Fourteen sentences, 116 words, a topic opening at sentence four. Under a
    // forty-word target the first window would take five sentences (39 words)
    // but the topic opens after thirty of them — past two thirds — so it ends
    // there; the rest fill to forty, and each next window backs up two
    // sentences to cover ten words of overlap.
    let document = cut(
        &interview(),
        Budget {
            target_words: 40,
            overlap_words: 10,
            context_sentences: 2,
        },
    );
    let shape = |first, count, words, topics: &[u64], before, after| Shape {
        first,
        count,
        words,
        topics: topics.to_vec(),
        before,
        after,
    };
    assert_eq!(
        shapes(&document),
        [
            shape(0, 4, 30, &[0], 0, 2),
            shape(2, 4, 32, &[0, 1], 2, 2),
            shape(4, 5, 40, &[1], 2, 2),
            shape(7, 4, 34, &[1], 2, 2),
            shape(9, 4, 37, &[1], 2, 1),
            shape(11, 3, 28, &[1], 2, 0),
        ]
    );
    // The first window's ticks are the first sentence's start and the fourth
    // sentence's end, read straight from the index.
    let first = &document.windows[0];
    assert_eq!(first.start_ticks, document.sentences[0].start_ticks);
    assert_eq!(first.end_ticks, document.sentences[3].end_ticks);
}

#[test]
fn the_default_budget_reads_a_short_recording_as_one_window() {
    let document = cut(&interview(), Budget::DEFAULT);
    assert_eq!(
        shapes(&document),
        [Shape {
            first: 0,
            count: 14,
            words: 116,
            topics: vec![0, 1],
            before: 0,
            after: 0,
        }]
    );
}

#[test]
fn invalid_regions_are_carried_through_for_the_validator() {
    let index: IndexTranscript =
        read(&repo().join("contracts/fixtures/index.transcript/valid/interpolated_timing.json"));
    let document = cut(&index, Budget::DEFAULT);
    assert_eq!(document.invalid_regions.len(), 1);
    let region = &document.invalid_regions[0];
    assert_eq!(region.reason, InvalidRegionReason::TimingInterpolated);
    assert_eq!(region.start_ticks, 396_000);
    assert_eq!(region.end_ticks, 828_000);
    assert!(
        region
            .detail
            .as_ref()
            .is_some_and(|detail| detail.as_str().contains("aligner"))
    );
}

#[test]
fn an_index_over_nothing_analyzed_is_refused() {
    let mut index = interview();
    index.coverage.analyzed = false;
    let refused = windows(
        &index,
        Inputs {
            index: INDEX_ID,
            transcript: TRANSCRIPT_ID,
        },
        Budget::DEFAULT,
        IMPLEMENTATION,
    );
    assert!(
        matches!(refused, Err(WindowsError::NotAnalyzed)),
        "{refused:?}"
    );
}

#[test]
fn a_malformed_artifact_address_is_refused_by_name() {
    let refused = windows(
        &interview(),
        Inputs {
            index: "not-an-address",
            transcript: TRANSCRIPT_ID,
        },
        Budget::DEFAULT,
        IMPLEMENTATION,
    );
    assert!(
        matches!(
            refused,
            Err(WindowsError::MalformedAddress {
                field: "index_artifact_id"
            })
        ),
        "{refused:?}"
    );
}

#[test]
fn what_it_writes_is_the_contract_byte_for_byte() {
    let document = cut(&interview(), Budget::DEFAULT);
    let written = canonical(&serde_json::to_value(&document).expect("serializes"));
    let parsed: EditorialWindows = serde_json::from_str(&written).expect("the contract reads it");
    assert_eq!(
        canonical(&serde_json::to_value(&parsed).expect("serializes")),
        written
    );
    assert_eq!(
        document.schema_version,
        serde_json::json!("clipmill.editorial.windows.v1")
    );
}
