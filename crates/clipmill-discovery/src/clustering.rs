//! Grouping near-duplicates, without deciding between them.
//!
//! Three proposers searching one recording will nominate the same moment more
//! than once — that is the mesh working, not a bug. What must not happen is the
//! same clip reaching a user three times, or two of the three being dropped
//! silently so nobody can ask why.
//!
//! So candidates are clustered and every cluster keeps all its members. Ranking
//! sends the representative forward and the interface can still answer "why was
//! this considered a duplicate?" with the alternatives in hand (book ch. 15).
//! A diversity decision that cannot be shown is a diversity decision nobody can
//! argue with.
//!
//! Similarity is two measures rather than an embedding, because there is no
//! embedding model at this phase and a single measure gets each half wrong:
//! interval overlap alone merges a question with the unrelated remark that
//! follows it in the same thirty seconds, and evidence overlap alone splits two
//! nominations of the same moment that happened to cite different units of it.

use std::collections::{BTreeMap, BTreeSet};

use clipmill_contracts::schemas::discovery_candidates as contract;

/// Above this, two nominations are the same moment.
///
/// Set where a shared question-and-answer clusters with the quote pulled out of
/// its answer, and a topic span does not cluster with the next topic merely
/// because a long clip reaches into it.
const SIMILAR: f64 = 0.5;

/// What one candidate looks like to the clusterer.
pub(crate) struct Grouped<'a> {
    pub id: &'a str,
    pub interval: (u64, u64),
    pub evidence: BTreeSet<(u8, u64)>,
    pub score: f64,
}

/// Cluster the candidates and return the groups, ordered by identity.
///
/// Single-link agglomeration over the similarity above. Single-link because the
/// question being asked is "does this duplicate anything already in the group",
/// not "does it duplicate all of them": a long topic span and the quote inside
/// it are the same moment even though the quote and a second quote at the far
/// end of the topic are not.
pub(crate) fn cluster(candidates: &[Grouped<'_>]) -> Vec<contract::Cluster> {
    let mut parent = (0..candidates.len()).collect::<Vec<_>>();
    let mut weakest: BTreeMap<usize, f64> = BTreeMap::new();
    for left in 0..candidates.len() {
        for right in left + 1..candidates.len() {
            let score = similarity(&candidates[left], &candidates[right]);
            if score < SIMILAR {
                continue;
            }
            let (a, b) = (find(&mut parent, left), find(&mut parent, right));
            if a != b {
                parent[a] = b;
            }
            let root = find(&mut parent, b);
            let entry = weakest.entry(root).or_insert(1.0);
            *entry = entry.min(score);
        }
    }

    let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for position in 0..candidates.len() {
        let root = find(&mut parent, position);
        groups.entry(root).or_default().push(position);
    }
    // The weakest link was recorded against whichever root happened to win a
    // union, and roots move as the passes go on. Re-resolve them once at the
    // end so the number lands on the group it actually describes.
    let resolved = weakest
        .into_iter()
        .map(|(root, score)| (find(&mut parent, root), score))
        .fold(
            BTreeMap::new(),
            |mut merged: BTreeMap<usize, f64>, (root, score)| {
                let entry = merged.entry(root).or_insert(1.0);
                *entry = entry.min(score);
                merged
            },
        );

    let mut clusters = groups
        .into_iter()
        .map(|(root, members)| {
            let mut ids = members
                .iter()
                .map(|position| candidates[*position].id.to_owned())
                .collect::<Vec<_>>();
            ids.sort();
            // Highest preliminary score, ties broken by id so the choice does
            // not depend on which proposer ran first.
            let representative = members
                .iter()
                .max_by(|left, right| {
                    candidates[**left]
                        .score
                        .total_cmp(&candidates[**right].score)
                        .then_with(|| candidates[**right].id.cmp(candidates[**left].id))
                })
                .map(|position| candidates[*position].id.to_owned())
                .unwrap_or_default();
            let similarity = if ids.len() == 1 {
                1.0
            } else {
                resolved.get(&root).copied().unwrap_or(SIMILAR)
            };
            contract::Cluster {
                id: crate::literal(crate::identity("cl_", &ids).as_str()),
                representative: crate::literal(&representative),
                members: ids.iter().map(|id| crate::literal(id)).collect(),
                similarity,
            }
        })
        .collect::<Vec<_>>();
    clusters.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
    clusters
}

/// Interval overlap and evidence overlap, weighted evenly.
///
/// Neither dominates: two nominations of one moment usually agree on both, and
/// where they disagree the disagreement is the information — a quote inside a
/// topic shares evidence without sharing much span, and two adjacent topics
/// share span without sharing evidence.
fn similarity(left: &Grouped<'_>, right: &Grouped<'_>) -> f64 {
    0.5 * intersection_over_union(left.interval, right.interval)
        + 0.5 * jaccard(&left.evidence, &right.evidence)
}

fn intersection_over_union(left: (u64, u64), right: (u64, u64)) -> f64 {
    let low = left.0.max(right.0);
    let high = left.1.min(right.1);
    if high <= low {
        return 0.0;
    }
    let union = left.1.max(right.1) - left.0.min(right.0);
    if union == 0 {
        return 0.0;
    }
    crate::ticks_f64(high - low) / crate::ticks_f64(union)
}

fn jaccard(left: &BTreeSet<(u8, u64)>, right: &BTreeSet<(u8, u64)>) -> f64 {
    if left.is_empty() && right.is_empty() {
        return 0.0;
    }
    let shared = left.intersection(right).count();
    let total = left.union(right).count();
    if total == 0 {
        return 0.0;
    }
    crate::as_f64(shared) / crate::as_f64(total)
}

fn find(parent: &mut [usize], mut at: usize) -> usize {
    while parent[at] != at {
        parent[at] = parent[parent[at]];
        at = parent[at];
    }
    at
}

#[cfg(test)]
mod tests;
