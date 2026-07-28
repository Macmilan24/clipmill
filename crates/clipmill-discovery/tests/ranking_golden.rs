//! Ranking over the candidate sets the contracts already publish.
//!
//! Regenerate with `CLIPMILL_BLESS=1 cargo test -p clipmill-discovery`. A
//! golden that changes is a change in which clips the system would show a user
//! and where it would cut them, and the diff is the review.
//!
//! Tests may panic; the workspace deny targets production code.
#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use clipmill_contracts::schemas::discovery_candidates::DiscoveryCandidates;
use clipmill_contracts::schemas::index_transcript::IndexTranscript;
use clipmill_contracts::schemas::ranking_set::{self as contract, RankingSet};
use clipmill_contracts::schemas::speech_transcript::SpeechTranscript;
use clipmill_discovery::{RankingInputs, Request, rank, ranking::is_well_formed};

const CANDIDATES_ID: &str =
    "sha256:ca4d000000000000000000000000000000000000000000000000000000000077";
const INDEX_ID: &str = "sha256:1de0000000000000000000000000000000000000000000000000000000000011";
const TRANSCRIPT_ID: &str =
    "sha256:7a11000000000000000000000000000000000000000000000000000000000042";
const IMPLEMENTATION: &str = "clipmill-ranking-baseline@1.0.0";

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

/// Each committed candidate set, with the index and transcript behind it. The
/// four fixture directories share a filename, which makes the pairing an
/// assertion rather than a lookup table.
fn recordings() -> Vec<(
    String,
    DiscoveryCandidates,
    IndexTranscript,
    SpeechTranscript,
)> {
    let base = repo().join("contracts/fixtures");
    let mut names = std::fs::read_dir(base.join("discovery.candidates/valid"))
        .expect("the candidate fixtures exist")
        .filter_map(|entry| {
            let path = entry.expect("a readable entry").path();
            (path.extension()? == "json")
                .then(|| path.file_name()?.to_str().map(ToOwned::to_owned))?
        })
        .collect::<Vec<_>>();
    names.sort();
    assert!(!names.is_empty(), "there are no candidate fixtures");
    names
        .into_iter()
        .map(|name| {
            let candidates = read(&base.join("discovery.candidates/valid").join(&name));
            let index = read(&base.join("index.transcript/valid").join(&name));
            let transcript = read(&base.join("speech.transcript/valid").join(&name));
            (name, candidates, index, transcript)
        })
        .collect()
}

fn ranked(
    candidates: &DiscoveryCandidates,
    index: &IndexTranscript,
    transcript: &SpeechTranscript,
) -> RankingSet {
    rank(
        candidates,
        index,
        transcript,
        RankingInputs {
            candidates: CANDIDATES_ID,
            index: INDEX_ID,
            transcript: TRANSCRIPT_ID,
        },
        Request::DEFAULT,
        IMPLEMENTATION,
    )
    .expect("the ranking runs")
}

#[test]
fn every_published_cohort_ranks_to_its_golden() {
    let bless = std::env::var_os("CLIPMILL_BLESS").is_some();
    let directory = repo().join("contracts/fixtures/ranking.set/valid");
    for (name, candidates, index, transcript) in recordings() {
        let produced = canonical(
            &serde_json::to_value(ranked(&candidates, &index, &transcript)).expect("serializes"),
        );
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
        assert_eq!(produced, expected, "ranking over {name} changed");
    }
}

/// What a results board is allowed to assume without opening the candidate set.
#[test]
fn every_set_holds_what_a_results_board_relies_on() {
    for (name, candidates, index, transcript) in recordings() {
        let set = ranked(&candidates, &index, &transcript);
        assert!(is_well_formed(&set), "{name}: the set is inconsistent");

        let lattices = candidates
            .candidates
            .iter()
            .map(|candidate| (candidate.id.as_str(), &candidate.boundary_lattice))
            .collect::<std::collections::BTreeMap<_, _>>();
        let mut clusters = BTreeSet::new();

        for entry in &set.cohort {
            // Every card names all eight axes, measured or not.
            assert_eq!(entry.factors.len(), 8, "{name}");
            for factor in &entry.factors {
                if factor.available {
                    assert!(factor.value.is_some() && factor.weight.is_some(), "{name}");
                } else {
                    // An absence carries no number and states its reason.
                    assert!(factor.value.is_none(), "{name}: an absence carries a value");
                    assert!(factor.weight.is_none(), "{name}: an absence is weighted");
                    assert!(factor.unavailable_reason.is_some(), "{name}");
                }
            }
            // The chosen boundary came out of that candidate's own lattice.
            let lattice = lattices
                .get(entry.candidate_id.as_str())
                .unwrap_or_else(|| panic!("{name}: a ranked candidate was never nominated"));
            assert!(
                lattice.starts.contains(&entry.boundary.chosen.start_ticks),
                "{name}"
            );
            assert!(
                lattice.ends.contains(&entry.boundary.chosen.end_ticks),
                "{name}"
            );
            assert_eq!(entry.boundary.terms.len(), 7, "{name}");
            // The runner-up never beats the winner.
            if let Some(alternative) = &entry.boundary.alternative {
                assert!(alternative.score <= entry.boundary.score, "{name}");
                assert_ne!(
                    (
                        alternative.interval.start_ticks,
                        alternative.interval.end_ticks
                    ),
                    (
                        entry.boundary.chosen.start_ticks,
                        entry.boundary.chosen.end_ticks
                    ),
                    "{name}: the alternative repeats the winner"
                );
            }
            assert!((0..=99).contains(&entry.display_score), "{name}");
        }

        // Selection never shows one moment twice.
        let cards = clipmill_discovery::ranking::cards(&set);
        for id in &set.selected {
            let entry = cards
                .get(id.as_str())
                .unwrap_or_else(|| panic!("{name}: a selected id is not in the cohort"));
            assert!(
                clusters.insert(entry.cluster_id.as_str()),
                "{name}: two clips from one cluster were selected"
            );
        }
    }
}

/// A set smaller than the one requested accounts for the difference. Padding it
/// would be the system recommending clips it does not believe in.
#[test]
fn a_short_set_says_why_rather_than_padding() {
    for (name, candidates, index, transcript) in recordings() {
        let set = ranked(&candidates, &index, &transcript);
        let accounted: u64 = set.shortfall.iter().map(|reason| reason.count.get()).sum();
        assert_eq!(
            clipmill_discovery::ranking::cards(&set).len(),
            set.cohort.len()
        );
        assert_eq!(
            set.selected.len() as u64 + accounted,
            set.requested.count.get(),
            "{name}: the shortfall does not add up"
        );
        for reason in &set.shortfall {
            assert!(
                matches!(
                    reason.reason,
                    contract::ShortfallReasonReason::CohortExhausted
                        | contract::ShortfallReasonReason::AllRemainingAreDuplicates
                        | contract::ShortfallReasonReason::AllRemainingBelowBar
                ),
                "{name}"
            );
        }
    }
}

#[test]
fn ranking_the_same_cohort_twice_produces_the_same_bytes() {
    for (name, candidates, index, transcript) in recordings() {
        let first =
            canonical(&serde_json::to_value(ranked(&candidates, &index, &transcript)).unwrap());
        let second =
            canonical(&serde_json::to_value(ranked(&candidates, &index, &transcript)).unwrap());
        assert_eq!(first, second, "{name} did not rank deterministically");
    }
}
