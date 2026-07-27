//! Discovery over the indexes the contracts already publish.
//!
//! The unit tests use recordings written for one property each. This is the
//! other half: the committed `index.transcript.v1` fixtures — themselves
//! derived from the committed transcripts — searched end to end, compared
//! against a golden result, and checked against the guarantees a consumer is
//! allowed to assume without looking.
//!
//! Regenerate with `CLIPMILL_BLESS=1 cargo test -p clipmill-discovery`. Do it
//! deliberately: a golden that changes is a change in which moments the system
//! thinks are worth clipping, and the diff is the review.
//!
//! Tests may panic; the workspace deny targets production code.
#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use clipmill_contracts::schemas::discovery_candidates::{
    self as contract, DiscoveryCandidates, EvidenceReferenceKind,
};
use clipmill_contracts::schemas::index_transcript::IndexTranscript;
use clipmill_contracts::schemas::speech_transcript::SpeechTranscript;
use clipmill_discovery::{Inputs, Parameters, discover, is_well_formed};

const INDEX_ID: &str = "sha256:1de0000000000000000000000000000000000000000000000000000000000011";
const TRANSCRIPT_ID: &str =
    "sha256:7a11000000000000000000000000000000000000000000000000000000000042";
const IMPLEMENTATION: &str = "clipmill-discovery-mesh@1.0.0";

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

/// Each committed index, paired with the transcript it was derived from. The
/// two fixture sets share a filename, which is what makes the pairing an
/// assertion rather than a lookup table.
fn recordings() -> Vec<(String, IndexTranscript, SpeechTranscript)> {
    let indexes = repo().join("contracts/fixtures/index.transcript/valid");
    let transcripts = repo().join("contracts/fixtures/speech.transcript/valid");
    let mut names = std::fs::read_dir(&indexes)
        .unwrap_or_else(|error| panic!("cannot list {}: {error}", indexes.display()))
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
            let index = read(&indexes.join(&name));
            let transcript = read(&transcripts.join(&name));
            (name, index, transcript)
        })
        .collect()
}

fn searched(index: &IndexTranscript, transcript: &SpeechTranscript) -> DiscoveryCandidates {
    discover(
        index,
        transcript,
        None,
        Inputs {
            index: INDEX_ID,
            transcript: TRANSCRIPT_ID,
            loudness: None,
        },
        Parameters::DEFAULT,
        IMPLEMENTATION,
    )
    .expect("the search runs")
}

#[test]
fn every_published_index_searches_to_its_golden() {
    let bless = std::env::var_os("CLIPMILL_BLESS").is_some();
    let directory = repo().join("contracts/fixtures/discovery.candidates/valid");
    for (name, index, transcript) in recordings() {
        let produced =
            canonical(&serde_json::to_value(searched(&index, &transcript)).expect("serializes"));
        let golden = directory.join(&name);
        if bless {
            std::fs::create_dir_all(&directory).expect("the fixture directory");
            std::fs::write(&golden, &produced).expect("the golden is writable");
            continue;
        }
        let expected = std::fs::read_to_string(&golden).unwrap_or_else(|error| {
            panic!(
                "cannot read {}: {error}\nrun CLIPMILL_BLESS=1 cargo test -p clipmill-discovery",
                golden.display()
            )
        });
        assert_eq!(produced, expected, "discovery over {name} changed");
    }
}

/// The three guarantees, over real documents. None of them is enforced by the
/// schema, which can express shapes but not arithmetic.
#[test]
fn every_candidate_set_holds_the_guarantees_ranking_relies_on() {
    for (name, index, transcript) in recordings() {
        let found = searched(&index, &transcript);
        assert!(is_well_formed(&found), "{name}: the set is inconsistent");

        let min = found.duration_target.min_ticks.get();
        let max = found.duration_target.max_ticks.get();
        let clusters = found
            .clusters
            .iter()
            .map(|cluster| cluster.id.as_str())
            .collect::<BTreeSet<_>>();

        for candidate in &found.candidates {
            // Legal: inside coverage, inside the requested length, and drawn
            // from its own lattice.
            for interval in &candidate.intervals {
                assert!(interval.start_ticks < interval.end_ticks, "{name}");
                assert!(interval.start_ticks >= found.coverage.start_ticks, "{name}");
                assert!(interval.end_ticks <= found.coverage.end_ticks, "{name}");
                let span = interval.end_ticks - interval.start_ticks;
                assert!(span >= min && span <= max, "{name}: illegal length");
            }
            let lattice = &candidate.boundary_lattice;
            assert!(
                lattice.starts.contains(&candidate.intervals[0].start_ticks),
                "{name}: the interval starts off its own lattice"
            );
            assert!(
                lattice
                    .ends
                    .contains(&candidate.intervals[candidate.intervals.len() - 1].end_ticks),
                "{name}: the interval ends off its own lattice"
            );
            // Ascending and distinct, so a consumer can binary-search them.
            assert!(
                lattice.starts.windows(2).all(|pair| pair[0] < pair[1]),
                "{name}: lattice starts are not ordered"
            );
            assert!(
                lattice.ends.windows(2).all(|pair| pair[0] < pair[1]),
                "{name}: lattice ends are not ordered"
            );

            // Explicable: every reference resolves into the index.
            assert!(!candidate.evidence.is_empty(), "{name}");
            for reference in &candidate.evidence {
                let at = usize::try_from(reference.index).unwrap();
                let exists = match reference.kind {
                    EvidenceReferenceKind::Sentence => at < index.sentences.len(),
                    EvidenceReferenceKind::Topic => at < index.topics.len(),
                    EvidenceReferenceKind::Utterance => at < index.utterances.len(),
                };
                assert!(exists, "{name}: evidence points outside the index");
            }

            // Grouped: the cluster it names is published.
            assert!(
                clusters.contains(candidate.cluster_id.as_str()),
                "{name}: a candidate names an unpublished cluster"
            );
        }
    }
}

/// A lattice point that pairs with nothing legal is a point that should never
/// have been published: ranking would search it and find nothing.
#[test]
fn every_lattice_point_pairs_with_something_legal() {
    for (name, index, transcript) in recordings() {
        let found = searched(&index, &transcript);
        let min = found.duration_target.min_ticks.get();
        let max = found.duration_target.max_ticks.get();
        let legal = |start: u64, end: u64| {
            start < end && {
                let span = end - start;
                span >= min && span <= max
            }
        };
        for candidate in &found.candidates {
            let lattice = &candidate.boundary_lattice;
            for start in &lattice.starts {
                assert!(
                    lattice.ends.iter().any(|end| legal(*start, *end)),
                    "{name}: lattice start {start} pairs with nothing legal"
                );
            }
            for end in &lattice.ends {
                assert!(
                    lattice.starts.iter().any(|start| legal(*start, *end)),
                    "{name}: lattice end {end} pairs with nothing legal"
                );
            }
        }
    }
}

/// A reject counted under a reason that never fired would make the record
/// unreadable — and a reason recorded as zero would read like a term that was
/// checked and passed.
#[test]
fn every_recorded_rejection_names_a_reason_that_fired() {
    for (name, index, transcript) in recordings() {
        let found = searched(&index, &transcript);
        for candidate in &found.candidates {
            let mut seen = BTreeSet::new();
            for reject in &candidate.boundary_lattice.phi_rejects {
                assert!(
                    seen.insert(format!("{:?}", reject.reason)),
                    "{name}: a reason is counted twice"
                );
                assert!(reject.count.get() > 0, "{name}");
                // The terms this phase cannot measure are absent, not zero.
                assert!(
                    matches!(
                        reject.reason,
                        contract::PhiRejectReason::MidWord
                            | contract::PhiRejectReason::TooShort
                            | contract::PhiRejectReason::TooLong
                            | contract::PhiRejectReason::OutsideCoverage
                    ),
                    "{name}"
                );
            }
        }
    }
}

#[test]
fn searching_the_same_recording_twice_produces_the_same_bytes() {
    for (name, index, transcript) in recordings() {
        let first = canonical(&serde_json::to_value(searched(&index, &transcript)).unwrap());
        let second = canonical(&serde_json::to_value(searched(&index, &transcript)).unwrap());
        assert_eq!(first, second, "{name} did not search deterministically");
    }
}
