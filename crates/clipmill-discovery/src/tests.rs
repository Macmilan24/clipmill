//! The three guarantees, over a whole recording.
//!
//! Discovery promises width, legality, and explicability — not judgement. Each
//! of these tests is one of those promises, plus the refusals that keep a
//! candidate set from claiming more than it measured.
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::float_cmp)]

use clipmill_contracts::schemas::discovery_candidates as contract;

use super::{DiscoveryError, Inputs, Parameters, discover, is_well_formed};
use crate::fixture::{self, IMPLEMENTATION, INDEX_ID, TRANSCRIPT_ID};

fn searched() -> contract::DiscoveryCandidates {
    let interview = fixture::interview();
    discover(
        &interview.index,
        &interview.transcript,
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
fn the_whole_mesh_reports_even_when_a_proposer_finds_nothing() {
    let found = searched();
    let names = found
        .proposers
        .iter()
        .map(|run| run.proposer.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, ["narrative-arc", "insight-quote", "question-answer"]);
    // A proposer that nominated nothing is a fact about the recording. A
    // proposer missing from this list would be a fact about the build, and the
    // two must not look alike.
    for run in &found.proposers {
        assert!(run.candidates <= run.seeds);
    }
}

/// Rule 14.1. A candidate nobody can explain is a candidate ranking cannot
/// defend, so evidence is required by the contract and non-empty here.
#[test]
fn every_candidate_walks_back_to_the_index() {
    let interview = fixture::interview();
    let found = searched();
    assert!(!found.candidates.is_empty());
    for candidate in &found.candidates {
        assert!(!candidate.evidence.is_empty());
        for reference in &candidate.evidence {
            let at = usize::try_from(reference.index).unwrap();
            match reference.kind {
                contract::EvidenceReferenceKind::Sentence => {
                    assert!(at < interview.index.sentences.len());
                }
                contract::EvidenceReferenceKind::Topic => {
                    assert!(at < interview.index.topics.len());
                }
                contract::EvidenceReferenceKind::Utterance => {
                    assert!(at < interview.index.utterances.len());
                }
            }
        }
        for role in [&candidate.roles.hook, &candidate.roles.payoff]
            .into_iter()
            .flatten()
        {
            assert!(
                candidate
                    .evidence
                    .iter()
                    .any(|held| held.kind == role.kind && held.index == role.index),
                "a role names evidence the candidate does not carry"
            );
        }
    }
}

#[test]
fn every_published_interval_is_legal() {
    let found = searched();
    let min = found.duration_target.min_ticks.get();
    let max = found.duration_target.max_ticks.get();
    for candidate in &found.candidates {
        for interval in &candidate.intervals {
            let span = interval.end_ticks - interval.start_ticks;
            assert!(span >= min, "shorter than the requested minimum");
            assert!(span <= max, "longer than the requested maximum");
            assert!(interval.start_ticks >= found.coverage.start_ticks);
            assert!(interval.end_ticks <= found.coverage.end_ticks);
        }
        // The lattice contains the interval it settled on, or the interval was
        // chosen from somewhere the lattice does not admit.
        let lattice = &candidate.boundary_lattice;
        assert!(lattice.starts.contains(&candidate.intervals[0].start_ticks));
        assert!(
            lattice
                .ends
                .contains(&candidate.intervals[candidate.intervals.len() - 1].end_ticks)
        );
    }
}

#[test]
fn every_candidate_belongs_to_exactly_one_cluster() {
    let found = searched();
    let mut ids = found
        .candidates
        .iter()
        .map(|candidate| candidate.id.as_str())
        .collect::<Vec<_>>();
    let total = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), total, "a candidate id is published twice");
    assert!(is_well_formed(&found));
    for candidate in &found.candidates {
        assert!(
            found
                .clusters
                .iter()
                .any(|cluster| cluster.id.as_str() == candidate.cluster_id.as_str()),
            "a candidate names a cluster that is not published"
        );
    }
}

/// Discovery nominates; ranking decides. A candidate set that had already
/// chosen would be making the decision with less information than the stage
/// that has to live with it.
#[test]
fn discovery_does_not_choose() {
    let found = searched();
    // More than one nomination, and more than one legal boundary among them.
    assert!(found.candidates.len() > 1);
    assert!(
        found
            .candidates
            .iter()
            .any(|candidate| candidate.boundary_lattice.starts.len() > 1
                || candidate.boundary_lattice.ends.len() > 1)
    );
    // Nothing is marked as the winner: the only ordering is by position.
    let mut previous = 0;
    for candidate in &found.candidates {
        assert!(candidate.intervals[0].start_ticks >= previous);
        previous = candidate.intervals[0].start_ticks;
    }
}

/// The rubric is where the honesty lives, and it reaches the artifact key
/// through the document, so a proposer whose method changes cannot reuse a
/// cached candidate set.
#[test]
fn every_proposer_names_the_approximation_it_is_making() {
    let found = searched();
    let rubrics = found
        .proposers
        .iter()
        .map(|run| run.proposer.rubric.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        rubrics,
        [
            "topic-span-open-close.v1",
            "tfidf-specificity-claim-prosody.v1",
            "punctuation-and-wh-pattern.v1",
        ]
    );
    for run in &found.proposers {
        assert_ne!(
            run.proposer.rubric.as_str(),
            run.proposer.name.as_str(),
            "a rubric that repeats the name explains nothing"
        );
    }
}

#[test]
fn the_same_recording_searches_to_the_same_document_twice() {
    assert_eq!(
        serde_json::to_value(searched()).unwrap(),
        serde_json::to_value(searched()).unwrap()
    );
}

/// Two seeds from one proposer that expand to the same interval are one clip
/// found twice. Publishing both would put a duplicate id in the document and a
/// phantom alternative in front of a user; dropping the second would lose part
/// of why the interval is worth clipping.
#[test]
fn one_proposer_finding_a_clip_twice_publishes_it_once_with_both_reasons() {
    let interview = fixture::interview();
    let found = searched();
    let insight = found
        .candidates
        .iter()
        .filter(|candidate| candidate.proposer.name.as_str() == "insight-quote")
        .collect::<Vec<_>>();
    let reported = found
        .proposers
        .iter()
        .find(|run| run.proposer.name.as_str() == "insight-quote")
        .expect("the proposer ran");
    assert_eq!(insight.len(), usize::try_from(reported.candidates).unwrap());
    // The merge happened: this recording gives the quote proposer more seeds
    // than it gives distinct intervals.
    assert!(reported.seeds > reported.candidates);
    // And the survivor carries evidence from more than one of them.
    assert!(
        insight.iter().any(|candidate| candidate.evidence.len() > 1),
        "a merged candidate kept only one seed's evidence"
    );
    let _ = interview;
}

/// A candidate whose span overlaps timing the transcript disowned is still a
/// candidate — the words were said — but ranking must see that before it puts
/// a cut inside one.
#[test]
fn a_candidate_over_interpolated_timing_says_so() {
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
    let found = discover(
        &interview.index,
        &interview.transcript,
        None,
        Inputs {
            index: INDEX_ID,
            transcript: TRANSCRIPT_ID,
            loudness: None,
        },
        Parameters::DEFAULT,
        IMPLEMENTATION,
    )
    .expect("the search runs");
    assert!(
        found
            .candidates
            .iter()
            .all(|candidate| !candidate.exclusions.is_empty())
    );
}

#[test]
fn an_index_nobody_analyzed_is_refused_rather_than_searched() {
    let mut interview = fixture::interview();
    interview.index.coverage.analyzed = false;
    assert!(matches!(
        discover(
            &interview.index,
            &interview.transcript,
            None,
            Inputs {
                index: INDEX_ID,
                transcript: TRANSCRIPT_ID,
                loudness: None,
            },
            Parameters::DEFAULT,
            IMPLEMENTATION,
        ),
        Err(DiscoveryError::NotAnalyzed)
    ));
}

#[test]
fn documents_describing_different_sources_are_refused() {
    let interview = fixture::interview();
    let mut transcript = interview.transcript.clone();
    transcript.source_fingerprint =
        "sha256:0000000000000000000000000000000000000000000000000000000000000001"
            .parse()
            .expect("a digest");
    assert!(matches!(
        discover(
            &interview.index,
            &transcript,
            None,
            Inputs {
                index: INDEX_ID,
                transcript: TRANSCRIPT_ID,
                loudness: None,
            },
            Parameters::DEFAULT,
            IMPLEMENTATION,
        ),
        Err(DiscoveryError::MismatchedSources)
    ));
}

#[test]
fn an_empty_length_request_is_refused() {
    let interview = fixture::interview();
    for parameters in [
        Parameters {
            min_ticks: 100,
            max_ticks: 10,
            ..Parameters::DEFAULT
        },
        Parameters {
            min_ticks: 0,
            max_ticks: 0,
            ..Parameters::DEFAULT
        },
    ] {
        assert!(matches!(
            discover(
                &interview.index,
                &interview.transcript,
                None,
                Inputs {
                    index: INDEX_ID,
                    transcript: TRANSCRIPT_ID,
                    loudness: None,
                },
                parameters,
                IMPLEMENTATION,
            ),
            Err(DiscoveryError::EmptyDurationRange { .. })
        ));
    }
}

/// Asking for a different length is a different search, not a filter over this
/// one — and the document says which was asked for.
#[test]
fn the_requested_length_changes_the_answer_and_is_recorded() {
    let interview = fixture::interview();
    let short = discover(
        &interview.index,
        &interview.transcript,
        None,
        Inputs {
            index: INDEX_ID,
            transcript: TRANSCRIPT_ID,
            loudness: None,
        },
        Parameters {
            min_ticks: 5 * fixture::SECOND,
            max_ticks: 20 * fixture::SECOND,
            ..Parameters::DEFAULT
        },
        IMPLEMENTATION,
    )
    .expect("the search runs");
    assert_eq!(short.duration_target.min_ticks.get(), 5 * fixture::SECOND);
    assert_ne!(
        short.candidates.len(),
        searched().candidates.len(),
        "a different length found the same set"
    );
}

/// Identity is derived from the proposer and the interval, so a reordering
/// upstream cannot rename a candidate and two runs name the same one.
#[test]
fn candidate_identity_is_derived_rather_than_counted() {
    let found = searched();
    let again = searched();
    let names = found
        .candidates
        .iter()
        .map(|candidate| candidate.id.as_str())
        .collect::<Vec<_>>();
    let repeat = again
        .candidates
        .iter()
        .map(|candidate| candidate.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, repeat);
    for id in &names {
        assert!(id.starts_with("cand_"));
        assert_eq!(id.len(), "cand_".len() + 16);
    }
}
