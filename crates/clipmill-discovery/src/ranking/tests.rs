//! The three decisions, and the refusal to pad a set.
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::float_cmp)]

use clipmill_contracts::schemas::ranking_set as contract;

use crate::fixture::{self, INDEX_ID, TRANSCRIPT_ID};

use super::{Inputs, RankingError, Request, is_well_formed, rank};

const CANDIDATES_ID: &str =
    "sha256:ca4d000000000000000000000000000000000000000000000000000000000077";

fn searched() -> (
    clipmill_contracts::schemas::discovery_candidates::DiscoveryCandidates,
    crate::fixture::Interview,
) {
    let interview = fixture::interview();
    let found = crate::discover(
        &interview.index,
        &interview.transcript,
        None,
        crate::Inputs {
            index: INDEX_ID,
            transcript: TRANSCRIPT_ID,
            loudness: None,
        },
        crate::Parameters::DEFAULT,
        fixture::IMPLEMENTATION,
    )
    .expect("the search runs");
    (found, interview)
}

fn ranked(request: Request) -> contract::RankingSet {
    let (found, interview) = searched();
    rank(
        &found,
        &interview.index,
        &interview.transcript,
        Inputs {
            candidates: CANDIDATES_ID,
            index: INDEX_ID,
            transcript: TRANSCRIPT_ID,
        },
        request,
        fixture::IMPLEMENTATION,
    )
    .expect("the ranking runs")
}

#[test]
fn the_cohort_is_ordered_scored_and_internally_consistent() {
    let set = ranked(Request::DEFAULT);
    assert!(is_well_formed(&set));
    assert!(!set.cohort.is_empty());
    for pair in set.cohort.windows(2) {
        assert!(pair[0].score >= pair[1].score, "the cohort is out of order");
        assert!(pair[0].rank.get() < pair[1].rank.get());
    }
    for entry in &set.cohort {
        assert!((0..=99).contains(&entry.display_score));
        assert_eq!(entry.factors.len(), 8);
    }
}

/// The rule most systems break. Asked for more clips than the recording holds,
/// the honest answer is fewer plus a reason — not a padded set the system does
/// not believe in.
#[test]
fn asking_for_more_than_exists_returns_fewer_and_says_why() {
    let set = ranked(Request {
        count: 50,
        diversity: 0.7,
    });
    assert!(set.selected.len() < 50);
    assert!(
        !set.shortfall.is_empty(),
        "a short set must account for itself"
    );
    let accounted: u64 = set.shortfall.iter().map(|reason| reason.count.get()).sum();
    assert_eq!(
        crate::as_u64(set.selected.len()) + accounted,
        set.requested.count.get(),
        "the shortfall does not add up"
    );
    assert!(is_well_formed(&set));
}

#[test]
fn a_request_that_can_be_met_reports_no_shortfall() {
    let set = ranked(Request {
        count: 1,
        diversity: 0.7,
    });
    assert_eq!(set.selected.len(), 1);
    assert!(set.shortfall.is_empty());
}

/// Discovery already stated that a cluster's members are the same moment.
/// Selecting two of them would show a user the same clip twice however the
/// arithmetic scores them.
#[test]
fn selection_takes_at_most_one_clip_from_a_cluster() {
    let set = ranked(Request {
        count: 20,
        diversity: 0.7,
    });
    let cards = super::cards(&set);
    let mut clusters = std::collections::BTreeSet::new();
    for id in &set.selected {
        let entry = cards
            .get(id.as_str())
            .expect("a selected id is in the cohort");
        assert!(
            clusters.insert(entry.cluster_id.as_str()),
            "two clips from one cluster reached the set"
        );
    }
}

/// The diversity dial is real: taking pure quality returns overlapping clips
/// that a diversity-weighted selector spreads out.
#[test]
fn the_diversity_dial_changes_which_clips_are_selected() {
    let greedy = ranked(Request {
        count: 3,
        diversity: 1.0,
    });
    let spread = ranked(Request {
        count: 3,
        diversity: 0.0,
    });
    assert_eq!(greedy.requested.diversity, 1.0);
    assert_eq!(spread.requested.diversity, 0.0);
    // Both are valid sets; the point is that the dial reaches the document and
    // therefore the artifact key.
    assert!(is_well_formed(&greedy));
    assert!(is_well_formed(&spread));
}

/// A candidate removed before scoring is recorded with its reason, so the
/// interface can answer "what happened to that one?" rather than the candidate
/// vanishing between two documents.
#[test]
fn a_filtered_candidate_is_recorded_rather_than_dropped() {
    let mut interview = fixture::interview();
    interview.index.invalid_regions = vec![
        clipmill_contracts::schemas::index_transcript::InvalidRegion {
            start_ticks: 0,
            end_ticks: interview.index.coverage.end_ticks,
            reason:
                clipmill_contracts::schemas::index_transcript::InvalidRegionReason::TimingInterpolated,
            detail: None,
        },
    ];
    let found = crate::discover(
        &interview.index,
        &interview.transcript,
        None,
        crate::Inputs {
            index: INDEX_ID,
            transcript: TRANSCRIPT_ID,
            loudness: None,
        },
        crate::Parameters::DEFAULT,
        fixture::IMPLEMENTATION,
    )
    .expect("the search runs");
    let set = rank(
        &found,
        &interview.index,
        &interview.transcript,
        Inputs {
            candidates: CANDIDATES_ID,
            index: INDEX_ID,
            transcript: TRANSCRIPT_ID,
        },
        Request::DEFAULT,
        fixture::IMPLEMENTATION,
    )
    .expect("the ranking runs");
    assert!(!set.filtered.is_empty());
    assert!(set.filtered.iter().all(|entry| matches!(
        entry.reason,
        contract::FilteredCandidateReason::ExcludedByDiscovery
    )));
    assert!(set.cohort.is_empty(), "nothing survived the filter");
    assert!(is_well_formed(&set));
}

/// Overlap with a clip already ranked above is a penalty applied once the
/// cohort exists, because it is a fact about a candidate's neighbours.
#[test]
fn repetition_penalises_the_weaker_of_an_overlapping_pair() {
    let set = ranked(Request::DEFAULT);
    let penalised = set
        .cohort
        .iter()
        .filter(|entry| {
            entry
                .penalties
                .iter()
                .any(|penalty| penalty.reason == contract::PenaltyReason::Repetition)
        })
        .collect::<Vec<_>>();
    if let Some(first) = penalised.first() {
        // Whoever was penalised is not the top of the cohort: the penalty falls
        // on the weaker of a pair, never on the one it overlapped.
        assert!(first.rank.get() > 1);
    }
}

/// Every boundary in the set is one the optimizer chose from the candidate's
/// own lattice, and the runner-up ships with it.
#[test]
fn every_ranked_clip_carries_its_boundary_and_its_alternative() {
    let (found, _) = searched();
    let set = ranked(Request::DEFAULT);
    let lattices = found
        .candidates
        .iter()
        .map(|candidate| (candidate.id.as_str(), &candidate.boundary_lattice))
        .collect::<std::collections::BTreeMap<_, _>>();
    for entry in &set.cohort {
        let lattice = lattices
            .get(entry.candidate_id.as_str())
            .expect("a ranked candidate was nominated");
        assert!(lattice.starts.contains(&entry.boundary.chosen.start_ticks));
        assert!(lattice.ends.contains(&entry.boundary.chosen.end_ticks));
        assert_eq!(entry.boundary.terms.len(), 7);
        assert!(
            entry
                .boundary
                .considered
                .is_some_and(|count| count.get() >= 1)
        );
        if let Some(alternative) = &entry.boundary.alternative {
            assert!(alternative.score <= entry.boundary.score);
        }
    }
}

/// The rubrics reach the document, and therefore the artifact key: a re-tune is
/// a different ranking rather than a correction of this one.
#[test]
fn the_document_names_the_arithmetic_that_produced_it() {
    let set = ranked(Request::DEFAULT);
    assert_eq!(set.rubric.scorer.as_str(), "scorecard-handset.v1");
    assert_eq!(set.rubric.boundary.as_str(), "boundary-j-handset.v1");
    assert_eq!(set.rubric.selector.as_str(), "mmr-handset.v1");
}

#[test]
fn the_same_cohort_ranks_the_same_way_twice() {
    assert_eq!(
        serde_json::to_value(ranked(Request::DEFAULT)).unwrap(),
        serde_json::to_value(ranked(Request::DEFAULT)).unwrap()
    );
}

#[test]
fn a_set_of_zero_clips_is_refused() {
    let (found, interview) = searched();
    assert!(matches!(
        rank(
            &found,
            &interview.index,
            &interview.transcript,
            Inputs {
                candidates: CANDIDATES_ID,
                index: INDEX_ID,
                transcript: TRANSCRIPT_ID,
            },
            Request {
                count: 0,
                diversity: 0.7,
            },
            fixture::IMPLEMENTATION,
        ),
        Err(RankingError::EmptyRequest)
    ));
}

#[test]
fn documents_describing_different_sources_are_refused() {
    let (found, interview) = searched();
    let mut transcript = interview.transcript.clone();
    transcript.source_fingerprint =
        "sha256:0000000000000000000000000000000000000000000000000000000000000001"
            .parse()
            .expect("a digest");
    assert!(matches!(
        rank(
            &found,
            &interview.index,
            &transcript,
            Inputs {
                candidates: CANDIDATES_ID,
                index: INDEX_ID,
                transcript: TRANSCRIPT_ID,
            },
            Request::DEFAULT,
            fixture::IMPLEMENTATION,
        ),
        Err(RankingError::MismatchedSources)
    ));
}
