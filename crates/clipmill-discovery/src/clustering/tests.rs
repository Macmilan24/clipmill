//! Grouping duplicates without deciding between them.
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::float_cmp)]

use std::collections::BTreeSet;

use super::{Grouped, cluster};

fn candidate<'a>(id: &'a str, interval: (u64, u64), evidence: &[u64], score: f64) -> Grouped<'a> {
    Grouped {
        id,
        interval,
        evidence: evidence
            .iter()
            .map(|at| (1u8, *at))
            .collect::<BTreeSet<_>>(),
        score,
    }
}

const A: &str = "cand_0000000000000001";
const B: &str = "cand_0000000000000002";
const C: &str = "cand_0000000000000003";

#[test]
fn a_candidate_that_duplicates_nothing_is_its_own_cluster() {
    let clusters = cluster(&[
        candidate(A, (0, 100), &[0], 0.5),
        candidate(B, (10_000, 10_100), &[9], 0.5),
    ]);
    assert_eq!(clusters.len(), 2);
    for group in &clusters {
        assert_eq!(group.members.len(), 1);
        assert_eq!(group.similarity, 1.0, "a cluster of one duplicates nothing");
        assert_eq!(group.representative.as_str(), group.members[0].as_str());
    }
}

#[test]
fn two_nominations_of_one_moment_land_in_one_cluster() {
    let clusters = cluster(&[
        candidate(A, (0, 1_000), &[0, 1, 2], 0.4),
        candidate(B, (0, 1_100), &[0, 1, 2], 0.9),
    ]);
    assert_eq!(clusters.len(), 1);
    assert_eq!(clusters[0].members.len(), 2);
    // The representative is the stronger nomination, not the first one seen.
    assert_eq!(clusters[0].representative.as_str(), B);
    assert!(clusters[0].similarity < 1.0);
}

/// Interval overlap alone would merge a quote with the unrelated remark that
/// happens to sit inside the same thirty seconds.
#[test]
fn sharing_a_span_is_not_enough_without_sharing_evidence() {
    let clusters = cluster(&[
        candidate(A, (0, 1_000), &[0], 0.5),
        candidate(B, (0, 1_000), &[50], 0.5),
    ]);
    assert_eq!(clusters.len(), 1, "identical spans are the same moment");

    // Half the span and no shared evidence is below the bar.
    let apart = cluster(&[
        candidate(A, (0, 1_000), &[0], 0.5),
        candidate(B, (500, 1_500), &[50], 0.5),
    ]);
    assert_eq!(apart.len(), 2);
}

/// Evidence overlap alone would split two nominations of the same moment that
/// happened to cite different units of it.
#[test]
fn sharing_evidence_survives_a_difference_in_span() {
    let clusters = cluster(&[
        candidate(A, (0, 1_000), &[0, 1], 0.5),
        candidate(B, (0, 1_200), &[0, 1], 0.5),
    ]);
    assert_eq!(clusters.len(), 1);
}

/// Single link: a topic span and the quote inside it are the same moment, and
/// so are the quote and a second quote it overlaps, even where the two quotes
/// share nothing directly.
#[test]
fn a_chain_of_overlaps_is_one_cluster() {
    let clusters = cluster(&[
        candidate(A, (0, 1_000), &[0, 1], 0.3),
        candidate(B, (0, 1_000), &[1, 2], 0.6),
        candidate(C, (0, 1_000), &[2, 3], 0.9),
    ]);
    assert_eq!(clusters.len(), 1);
    assert_eq!(clusters[0].members.len(), 3);
    assert_eq!(clusters[0].representative.as_str(), C);
}

/// Every candidate belongs to exactly one cluster: nothing is dropped, and
/// nothing is counted twice.
#[test]
fn the_clusters_partition_the_candidates() {
    let candidates = [
        candidate(A, (0, 1_000), &[0, 1], 0.3),
        candidate(B, (0, 1_100), &[0, 1], 0.6),
        candidate(C, (50_000, 51_000), &[40], 0.9),
    ];
    let clusters = cluster(&candidates);
    let members = clusters
        .iter()
        .flat_map(|group| group.members.iter().map(|id| id.as_str().to_owned()))
        .collect::<BTreeSet<_>>();
    assert_eq!(members.len(), candidates.len());
}

/// Identity is derived from the members, so it does not depend on which order
/// the proposers ran in.
#[test]
fn cluster_identity_follows_the_members_not_the_order() {
    let forward = cluster(&[
        candidate(A, (0, 1_000), &[0, 1], 0.3),
        candidate(B, (0, 1_100), &[0, 1], 0.6),
    ]);
    let backward = cluster(&[
        candidate(B, (0, 1_100), &[0, 1], 0.6),
        candidate(A, (0, 1_000), &[0, 1], 0.3),
    ]);
    assert_eq!(forward[0].id.as_str(), backward[0].id.as_str());
    assert_eq!(
        forward[0].representative.as_str(),
        backward[0].representative.as_str()
    );
}

#[test]
fn no_candidates_is_no_clusters() {
    assert!(cluster(&[]).is_empty());
}
