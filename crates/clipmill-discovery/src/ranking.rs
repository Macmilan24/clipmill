//! Which clips to show, in what order, cut where.
//!
//! Three decisions discovery deliberately left open, taken here (book ch. 16):
//! what each candidate is worth, where its boundaries actually fall, and which
//! subset of the cohort a user should see. Each is *shown* rather than
//! asserted — the score is a card with named axes and the evidence behind them,
//! the chosen boundary ships beside the runner-up it beat, and a set smaller
//! than the one requested says why.
//!
//! That last one is the rule most systems break. Asked for ten clips from a
//! recording holding four good moments, the honest answer is four and a reason,
//! not ten with six the system does not believe in. Padding a set is how a tool
//! teaches a user to stop trusting its ordering.
//!
//! Nothing here does any I/O.

use std::collections::{BTreeMap, BTreeSet};

use clipmill_contracts::schemas::discovery_candidates::DiscoveryCandidates;
use clipmill_contracts::schemas::index_transcript::IndexTranscript;
use clipmill_contracts::schemas::ranking_set as contract;
use clipmill_contracts::schemas::speech_transcript::SpeechTranscript;

use crate::{boundary, scorecard};

/// Who produced a ranked set.
pub const STAGE: &str = "rank-candidates";
/// Hand-set, and named so.
pub const SELECTOR_RUBRIC: &str = "mmr-handset.v1";

#[derive(Debug, thiserror::Error)]
pub enum RankingError {
    #[error("the candidate set was never searched, so there is nothing to rank")]
    NotAnalyzed,
    #[error("the candidate set, the index, and the transcript describe different sources")]
    MismatchedSources,
    #[error("a set of zero clips was requested")]
    EmptyRequest,
    #[error("{field} is not a well-formed content address")]
    MalformedAddress { field: &'static str },
}

/// What the caller asked for, all of which reaches the artifact key.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Request {
    /// How many clips to select.
    pub count: u64,
    /// The maximal-marginal-relevance trade-off: one takes the best clips
    /// whatever their overlap, zero takes the most different ones whatever
    /// their quality.
    pub diversity: f64,
}

impl Request {
    /// Ten clips at a mild diversity preference. Ten because that is what a
    /// results board holds without scrolling; 0.7 because a user who asked for
    /// ten wants ten *different* moments, and a pure-quality selector returns
    /// the same moment cut five ways.
    pub const DEFAULT: Self = Self {
        count: 10,
        diversity: 0.7,
    };
}

impl Default for Request {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// The artifacts a ranked set was computed from.
#[derive(Clone, Copy, Debug)]
pub struct Inputs<'a> {
    pub candidates: &'a str,
    pub index: &'a str,
    pub transcript: &'a str,
}

/// Score, cut, and select. Every number returned can be traced to something
/// that happened.
#[allow(
    clippy::too_many_lines,
    reason = "filter, cut, score, rank, select — the order is the cascade"
)]
pub fn rank(
    candidates: &DiscoveryCandidates,
    index: &IndexTranscript,
    transcript: &SpeechTranscript,
    inputs: Inputs<'_>,
    request: Request,
    implementation: &str,
) -> Result<contract::RankingSet, RankingError> {
    if !candidates.coverage.analyzed || !index.coverage.analyzed {
        return Err(RankingError::NotAnalyzed);
    }
    if *candidates.source_fingerprint != *index.source_fingerprint
        || *candidates.source_fingerprint != *transcript.source_fingerprint
    {
        return Err(RankingError::MismatchedSources);
    }
    let Some(count) = std::num::NonZeroU64::new(request.count) else {
        return Err(RankingError::EmptyRequest);
    };

    let duration = (
        candidates.duration_target.min_ticks.get(),
        candidates.duration_target.max_ticks.get(),
    );
    let context = boundary::Context {
        sentences: &index.sentences,
        utterances: &index.utterances,
        edges: &index.edges,
    };
    let evidence = scorecard::Evidence {
        index,
        words: &transcript.words,
        // Prosody is what the craft axis reads, and the loudness envelope is
        // what discovery had. Its presence there is the honest signal for
        // whether this source has audio worth measuring.
        has_audio: candidates.inputs.loudness_artifact_id.is_some(),
    };

    // Stage one: the filters. A candidate discovery already excluded, or whose
    // lattice holds nothing legal, is removed *with its reason recorded* — so
    // the interface can answer "what happened to that one?" rather than the
    // candidate vanishing between two documents.
    let mut filtered = Vec::new();
    let mut scored = Vec::new();
    for candidate in &candidates.candidates {
        if !candidate.exclusions.is_empty() {
            filtered.push(contract::FilteredCandidate {
                candidate_id: crate::literal(candidate.id.as_str()),
                reason: contract::FilteredCandidateReason::ExcludedByDiscovery,
                detail: candidate
                    .exclusions
                    .first()
                    .and_then(|exclusion| exclusion.detail.as_ref())
                    .and_then(|detail| detail.as_str().parse().ok()),
            });
            continue;
        }
        let Some(choice) = boundary::optimize(
            &candidate.boundary_lattice.starts,
            &candidate.boundary_lattice.ends,
            duration,
            &context,
        ) else {
            filtered.push(contract::FilteredCandidate {
                candidate_id: crate::literal(candidate.id.as_str()),
                reason: contract::FilteredCandidateReason::NoLegalBoundary,
                detail: "no pair in the lattice satisfies the requested clip length"
                    .parse()
                    .ok(),
            });
            continue;
        };
        let card = scorecard::measure(candidate, choice.chosen, &evidence);
        scored.push((candidate, choice, card));
    }

    // Repetition is a fact about a candidate's neighbours, so it is applied
    // once the cohort exists. Ordered by raw score first, so the penalty falls
    // on the weaker of an overlapping pair rather than on whichever the
    // proposers happened to emit second.
    scored.sort_by(|left, right| {
        right
            .2
            .score
            .total_cmp(&left.2.score)
            .then_with(|| left.0.id.as_str().cmp(right.0.id.as_str()))
    });
    let mut kept: Vec<(u64, u64)> = Vec::new();
    for (_, choice, card) in &mut scored {
        let overlap = kept
            .iter()
            .map(|earlier| intersection_over_union(*earlier, choice.chosen))
            .fold(0.0, f64::max);
        if overlap > 0.0 {
            let value = crate::published(0.4 * overlap);
            card.score = crate::published(card.score - value);
            card.penalties.push(contract::Penalty {
                reason: contract::PenaltyReason::Repetition,
                value,
                detail: "this span overlaps a clip already ranked above it"
                    .parse()
                    .ok(),
            });
        }
        kept.push(choice.chosen);
    }
    // Re-ordered, because the penalty just moved some of them.
    scored.sort_by(|left, right| {
        right
            .2
            .score
            .total_cmp(&left.2.score)
            .then_with(|| left.0.id.as_str().cmp(right.0.id.as_str()))
    });

    let raw = scored
        .iter()
        .map(|(_, _, card)| card.score)
        .collect::<Vec<_>>();
    let percentiles = scorecard::percentiles(&raw);
    let cohort = scored
        .iter()
        .enumerate()
        .map(|(position, (candidate, choice, card))| contract::Ranked {
            candidate_id: crate::literal(candidate.id.as_str()),
            rank: std::num::NonZeroU64::new(crate::as_u64(position + 1))
                .unwrap_or(std::num::NonZeroU64::MIN),
            display_score: i64::from(percentiles.get(&position).copied().unwrap_or(0)),
            score: card.score,
            factors: card.factors.clone(),
            uncertainty: card.uncertainty.clone(),
            penalties: card.penalties.clone(),
            boundary: contract::Boundary {
                chosen: contract::Interval {
                    start_ticks: choice.chosen.0,
                    end_ticks: choice.chosen.1,
                },
                score: choice.score,
                terms: choice.terms.clone(),
                alternative: choice.alternative.map(|(interval, score)| {
                    contract::BoundaryAlternative {
                        interval: contract::Interval {
                            start_ticks: interval.0,
                            end_ticks: interval.1,
                        },
                        score,
                    }
                }),
                considered: std::num::NonZeroU64::new(choice.considered),
            },
            cluster_id: crate::literal(candidate.cluster_id.as_str()),
        })
        .collect::<Vec<_>>();

    let (selected, shortfall) = select(&cohort, &scored, request, count);

    Ok(contract::RankingSet {
        schema_version: serde_json::json!("clipmill.ranking.set.v1"),
        source_fingerprint: parse(candidates.source_fingerprint.as_str(), "source_fingerprint")?,
        inputs: contract::RankingSetInputs {
            candidates_artifact_id: parse(inputs.candidates, "candidates_artifact_id")?,
            index_artifact_id: parse(inputs.index, "index_artifact_id")?,
            transcript_artifact_id: parse(inputs.transcript, "transcript_artifact_id")?,
        },
        producer: contract::Producer {
            stage: parse(STAGE, "producer stage")?,
            implementation: parse(implementation, "producer implementation")?,
        },
        rubric: contract::RankingSetRubric {
            scorer: parse(scorecard::RUBRIC, "scorer rubric")?,
            boundary: parse(boundary::RUBRIC, "boundary rubric")?,
            selector: parse(SELECTOR_RUBRIC, "selector rubric")?,
        },
        requested: contract::RankingSetRequested {
            count,
            diversity: crate::clamp_unit(request.diversity),
        },
        cohort,
        selected,
        shortfall,
        filtered,
    })
}

type Scored<'a> = (
    &'a clipmill_contracts::schemas::discovery_candidates::Candidate,
    boundary::Choice,
    scorecard::Card,
);

/// Maximal marginal relevance, and an honest account of what it could not fill.
///
/// At each step the selector takes the candidate maximising
/// `λ·q(c) − (1−λ)·max sim(c, chosen)`, so a clip has to be both good and
/// different from what is already in the set. The alternative — take the top
/// N by score — returns the same moment cut five ways whenever the mesh works
/// well, which is precisely when it should not.
fn select(
    cohort: &[contract::Ranked],
    scored: &[Scored<'_>],
    request: Request,
    count: std::num::NonZeroU64,
) -> (
    Vec<contract::RankingSetSelectedItem>,
    Vec<contract::ShortfallReason>,
) {
    let wanted = usize::try_from(count.get()).unwrap_or(usize::MAX);
    let lambda = crate::clamp_unit(request.diversity);
    let mut chosen: Vec<usize> = Vec::new();
    let mut chosen_clusters: BTreeSet<&str> = BTreeSet::new();
    let mut duplicates = 0u64;

    while chosen.len() < wanted {
        let mut best: Option<(usize, f64)> = None;
        for position in 0..cohort.len() {
            if chosen.contains(&position) {
                continue;
            }
            // A cluster is discovery's own statement that two nominations are
            // the same moment. Taking a second member is showing a user the
            // same clip twice however the arithmetic scores it.
            if chosen_clusters.contains(cohort[position].cluster_id.as_str()) {
                continue;
            }
            let similarity = chosen
                .iter()
                .map(|earlier| {
                    intersection_over_union(scored[*earlier].1.chosen, scored[position].1.chosen)
                })
                .fold(0.0, f64::max);
            let marginal = lambda * scored[position].2.score - (1.0 - lambda) * similarity;
            // Exact, for the same reason the boundary tiebreak is: the
            // earlier candidate wins a genuine tie, and a tolerance would make
            // that depend on arithmetic nobody chose.
            let wins = match best {
                None => true,
                Some((earlier, value)) => match marginal.total_cmp(&value) {
                    std::cmp::Ordering::Greater => true,
                    std::cmp::Ordering::Equal => position < earlier,
                    std::cmp::Ordering::Less => false,
                },
            };
            if wins {
                best = Some((position, marginal));
            }
        }
        let Some((position, _)) = best else {
            break;
        };
        chosen_clusters.insert(cohort[position].cluster_id.as_str());
        chosen.push(position);
    }
    duplicates += crate::as_u64(
        cohort
            .iter()
            .filter(|entry| {
                chosen_clusters.contains(entry.cluster_id.as_str())
                    && !chosen
                        .iter()
                        .any(|position| cohort[*position].candidate_id == entry.candidate_id)
            })
            .count(),
    );

    let selected = chosen
        .iter()
        .map(|position| crate::literal(cohort[*position].candidate_id.as_str()))
        .collect::<Vec<_>>();

    // The shortfall, accounted for rather than hidden. A set of four where ten
    // were asked for is a real answer; a set of ten where the last six are
    // clips the system does not believe in is not.
    let mut shortfall = Vec::new();
    let missing = wanted.saturating_sub(chosen.len());
    if missing > 0 {
        let from_duplicates = usize::try_from(duplicates).unwrap_or(0).min(missing);
        if let Some(value) = std::num::NonZeroU64::new(crate::as_u64(from_duplicates)) {
            shortfall.push(contract::ShortfallReason {
                reason: contract::ShortfallReasonReason::AllRemainingAreDuplicates,
                count: value,
                detail: "every remaining candidate belongs to a cluster already represented"
                    .parse()
                    .ok(),
            });
        }
        if let Some(value) = std::num::NonZeroU64::new(crate::as_u64(missing - from_duplicates)) {
            shortfall.push(contract::ShortfallReason {
                reason: contract::ShortfallReasonReason::CohortExhausted,
                count: value,
                detail: "the recording holds no further candidate to select"
                    .parse()
                    .ok(),
            });
        }
    }
    (selected, shortfall)
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

fn parse<T: std::str::FromStr>(value: &str, field: &'static str) -> Result<T, RankingError> {
    value
        .parse()
        .map_err(|_| RankingError::MalformedAddress { field })
}

/// The set is internally consistent: every selected id is in the cohort, every
/// cohort entry is ranked exactly once, and the shortfall accounts for the
/// difference between what was asked for and what was returned. Exported
/// because the daemon checks it before publishing and the gate checks it over
/// real documents.
#[must_use]
pub fn is_well_formed(document: &contract::RankingSet) -> bool {
    let ids = document
        .cohort
        .iter()
        .map(|entry| entry.candidate_id.as_str())
        .collect::<BTreeSet<_>>();
    if ids.len() != document.cohort.len() {
        return false;
    }
    for (position, entry) in document.cohort.iter().enumerate() {
        if entry.rank.get() != crate::as_u64(position + 1) {
            return false;
        }
    }
    let mut seen = BTreeSet::new();
    for id in &document.selected {
        if !ids.contains(id.as_str()) || !seen.insert(id.as_str()) {
            return false;
        }
    }
    let accounted: u64 = document
        .shortfall
        .iter()
        .map(|reason| reason.count.get())
        .sum();
    let requested = document.requested.count.get();
    let returned = crate::as_u64(document.selected.len());
    returned + accounted == requested || (returned == requested && accounted == 0)
}

/// The score cards, keyed by candidate, for a consumer that wants one.
#[must_use]
pub fn cards(document: &contract::RankingSet) -> BTreeMap<&str, &contract::Ranked> {
    document
        .cohort
        .iter()
        .map(|entry| (entry.candidate_id.as_str(), entry))
        .collect()
}

#[cfg(test)]
mod tests;
