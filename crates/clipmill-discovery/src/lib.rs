//! Finding the spans of a recording worth considering as clips.
//!
//! Discovery's job is width, not judgement (book ch. 15). It nominates; ranking
//! decides. The separation is what keeps re-ranking interactive: ranking may
//! reject, revise, or reorder a candidate, but it never has to *search*,
//! because every candidate arrives with a legal boundary lattice already
//! attached.
//!
//! Three guarantees, and they are the whole contract. Every candidate carries
//! evidence that walks back to words somebody measured (Rule 14.1). Every
//! candidate carries a lattice whose every point is legal. Every candidate
//! belongs to a cluster, so a near-duplicate is grouped rather than silently
//! dropped and the interface can always say why.
//!
//! What is *not* here is as deliberate. Three proposers out of the design's
//! ten, because the other seven read signals this phase does not measure. No
//! semantic embedding, so clustering is interval and evidence overlap. No
//! open-loop, identity, or rights terms in the legality predicate, because
//! nothing can evaluate them yet — and a term that always passes reads like a
//! term that was checked. Each of these limits is named in the document that
//! gets published, not just here.
//!
//! Nothing in this crate does any I/O.

mod boundary;
mod clustering;
#[cfg(test)]
pub(crate) mod fixture;
mod lattice;
#[cfg(test)]
mod planted;
mod proposers;
mod prosody;
pub mod ranking;
mod scorecard;

use std::collections::{BTreeMap, BTreeSet};

use clipmill_contracts::schemas::discovery_candidates as contract;
use clipmill_contracts::schemas::index_transcript::IndexTranscript;
use clipmill_contracts::schemas::media_loudness_envelope::MediaLoudnessEnvelope;
use clipmill_contracts::schemas::speech_transcript::SpeechTranscript;
use sha2::{Digest, Sha256};

pub use proposers::VERSION as PROPOSER_VERSION;
pub use ranking::{Inputs as RankingInputs, RankingError, Request, rank};
pub use scorecard::RUBRIC as SCORER_RUBRIC;

/// Who produced a candidate set.
pub const STAGE: &str = "discover-candidates";

#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    #[error("the evidence index was never analyzed, so there is nothing to search")]
    NotAnalyzed,
    #[error("the index, the transcript, and the loudness envelope describe different sources")]
    MismatchedSources,
    #[error("the requested clip length is empty: {min_ticks} to {max_ticks}")]
    EmptyDurationRange { min_ticks: u64, max_ticks: u64 },
    #[error("{field} is not a well-formed content address")]
    MalformedAddress { field: &'static str },
}

/// What the caller asked for, all of which reaches the artifact key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Parameters {
    /// The platform range candidates are expanded against.
    pub min_ticks: u64,
    pub max_ticks: u64,
    /// The fewest nominations each proposer keeps regardless of score.
    ///
    /// The exploration floor. A transcript-heavy recording scores well on the
    /// insight proposer and can crowd out a proposer with a different bias
    /// entirely; the floor is what stops the portfolio collapsing into one
    /// strategy on the material that strategy happens to suit.
    pub exploration_floor: u64,
}

impl Parameters {
    /// Fifteen seconds to three minutes is the platform range the design names
    /// (book ch. 15/16), and it is a range rather than fixed buckets because a
    /// clip's right length is a property of the moment.
    pub const DEFAULT: Self = Self {
        min_ticks: 15 * 90_000,
        max_ticks: 180 * 90_000,
        exploration_floor: 2,
    };
}

impl Default for Parameters {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// The artifacts a candidate set was searched from.
#[derive(Clone, Copy, Debug)]
pub struct Inputs<'a> {
    pub index: &'a str,
    pub transcript: &'a str,
    /// Absent for a source with no audio, in which case prosody measures
    /// nothing rather than contributing a default.
    pub loudness: Option<&'a str>,
}

/// Search the recording. Every candidate returned is legal, explained, and
/// grouped; none of them is chosen.
#[allow(
    clippy::too_many_lines,
    reason = "gather, run the mesh, expand, cluster — the order is the method"
)]
pub fn discover(
    index: &IndexTranscript,
    transcript: &SpeechTranscript,
    loudness: Option<&MediaLoudnessEnvelope>,
    inputs: Inputs<'_>,
    parameters: Parameters,
    implementation: &str,
) -> Result<contract::DiscoveryCandidates, DiscoveryError> {
    if !index.coverage.analyzed {
        return Err(DiscoveryError::NotAnalyzed);
    }
    if *index.source_fingerprint != *transcript.source_fingerprint
        || loudness
            .is_some_and(|envelope| *envelope.source_fingerprint != *index.source_fingerprint)
    {
        return Err(DiscoveryError::MismatchedSources);
    }
    if parameters.min_ticks > parameters.max_ticks {
        return Err(DiscoveryError::EmptyDurationRange {
            min_ticks: parameters.min_ticks,
            max_ticks: parameters.max_ticks,
        });
    }

    let (Some(min_ticks), Some(max_ticks)) = (
        std::num::NonZeroU64::new(parameters.min_ticks),
        std::num::NonZeroU64::new(parameters.max_ticks),
    ) else {
        // A zero-length clip is not a shorter clip; it is a request with no
        // answer, and the contract's own type says so.
        return Err(DiscoveryError::EmptyDurationRange {
            min_ticks: parameters.min_ticks,
            max_ticks: parameters.max_ticks,
        });
    };
    let duration = contract::DurationRange {
        min_ticks,
        max_ticks,
    };
    let coverage = (index.coverage.start_ticks, index.coverage.end_ticks);
    let boundaries = lattice::Boundaries::gather(index, &transcript.words);
    let novelty = proposers::Novelty::measure(index);
    let prosody = prosody::Prosody::measure(index, loudness);

    let mesh = [
        (
            proposers::identity(proposers::NARRATIVE, proposers::NARRATIVE_RUBRIC),
            proposers::narrative_arc(index, &novelty),
        ),
        (
            proposers::identity(proposers::INSIGHT, proposers::INSIGHT_RUBRIC),
            proposers::insight_quote(index, &novelty, &prosody),
        ),
        (
            proposers::identity(proposers::QUESTION, proposers::QUESTION_RUBRIC),
            proposers::question_answer(index),
        ),
    ];

    let mut runs = Vec::new();
    let mut candidates = Vec::new();
    for (proposer, seeds) in mesh {
        let seed_count = seeds.len();
        let mut kept = Vec::new();
        for seed in seeds {
            let Some(expanded) = boundaries.expand(seed.interval, &duration, coverage) else {
                continue;
            };
            kept.push((seed, expanded));
        }
        // The floor is a promise about width, so it is applied before anything
        // is thrown away and recorded when it changed the outcome.
        let floor = usize::try_from(parameters.exploration_floor).unwrap_or(usize::MAX);
        let floor_applied = (kept.len() < floor && kept.len() < seed_count)
            .then(|| std::num::NonZeroU64::new(as_u64(kept.len())))
            .flatten();

        // Two seeds from one proposer can expand to the same tightest
        // interval — adjacent sentences often do. That is one clip the
        // proposer found twice, not two clips, and publishing it twice would
        // put a duplicate id in the document and a phantom alternative in
        // front of a user. Merged rather than dropped, because the second
        // seed's evidence is part of why the interval is worth clipping.
        let mut merged: BTreeMap<String, contract::Candidate> = BTreeMap::new();
        for (seed, expanded) in &kept {
            let candidate = build(
                &proposer,
                seed,
                expanded,
                boundaries.mid_word_rejects,
                index,
            );
            match merged.entry(candidate.id.as_str().to_owned()) {
                std::collections::btree_map::Entry::Vacant(slot) => {
                    slot.insert(candidate);
                }
                std::collections::btree_map::Entry::Occupied(mut slot) => {
                    absorb(slot.get_mut(), candidate);
                }
            }
        }
        let kept_count = merged.len();
        candidates.extend(merged.into_values());
        runs.push(contract::ProposerRun {
            proposer,
            seeds: as_u64(seed_count),
            candidates: as_u64(kept_count),
            floor_applied,
        });
    }

    let grouped = candidates
        .iter()
        .map(|candidate| clustering::Grouped {
            id: candidate.id.as_str(),
            interval: (
                candidate.intervals[0].start_ticks,
                candidate.intervals[candidate.intervals.len() - 1].end_ticks,
            ),
            evidence: candidate
                .evidence
                .iter()
                .map(|reference| (kind_ordinal(reference.kind), reference.index))
                .collect(),
            score: candidate.prelim_score,
        })
        .collect::<Vec<_>>();
    let clusters = clustering::cluster(&grouped);
    let membership = clusters
        .iter()
        .flat_map(|cluster| {
            cluster
                .members
                .iter()
                .map(|member| (member.as_str().to_owned(), cluster.id.as_str().to_owned()))
        })
        .collect::<BTreeMap<_, _>>();
    for candidate in &mut candidates {
        if let Some(cluster) = membership.get(candidate.id.as_str()) {
            candidate.cluster_id = literal(cluster);
        }
    }
    candidates.sort_by(|left, right| {
        left.intervals[0]
            .start_ticks
            .cmp(&right.intervals[0].start_ticks)
            .then_with(|| left.id.as_str().cmp(right.id.as_str()))
    });

    Ok(contract::DiscoveryCandidates {
        schema_version: serde_json::json!("clipmill.discovery.candidates.v1"),
        source_fingerprint: parse(index.source_fingerprint.as_str(), "source_fingerprint")?,
        inputs: contract::DiscoveryCandidatesInputs {
            index_artifact_id: parse(inputs.index, "index_artifact_id")?,
            transcript_artifact_id: parse(inputs.transcript, "transcript_artifact_id")?,
            loudness_artifact_id: inputs
                .loudness
                .map(|id| parse(id, "loudness_artifact_id"))
                .transpose()?,
        },
        producer: contract::Producer {
            stage: parse(STAGE, "producer stage")?,
            implementation: parse(implementation, "producer implementation")?,
        },
        coverage: contract::Coverage {
            start_ticks: coverage.0,
            end_ticks: coverage.1,
            analyzed: true,
        },
        duration_target: duration,
        proposers: runs,
        candidates,
        clusters,
    })
}

/// One nomination, expanded and keyed.
fn build(
    proposer: &contract::Proposer,
    seed: &proposers::Seed,
    expanded: &lattice::Lattice,
    mid_word: u64,
    index: &IndexTranscript,
) -> contract::Candidate {
    let (start, end) = expanded.interval;
    let mut evidence = seed.evidence.clone();
    evidence.sort_by_key(|reference| (kind_ordinal(reference.kind), reference.index));
    evidence.dedup_by_key(|reference| (kind_ordinal(reference.kind), reference.index));

    // A candidate overlapping a span the transcript disowned is still a
    // candidate — the words were said — but ranking must be able to see that
    // its timing is not measured before it puts a cut inside one.
    let overlaps_interpolated = index
        .invalid_regions
        .iter()
        .any(|region| region.start_ticks < end && region.end_ticks > start);
    let exclusions = if overlaps_interpolated {
        vec![contract::Exclusion {
            reason: contract::ExclusionReason::InvalidRegion,
            detail: "part of this span has word timing the transcript does not vouch for"
                .parse()
                .ok(),
        }]
    } else {
        Vec::new()
    };

    let id = identity(
        "cand_",
        &[
            proposer.name.as_str().to_owned(),
            proposer.rubric.as_str().to_owned(),
            proposer.version.as_str().to_owned(),
            start.to_string(),
            end.to_string(),
        ],
    );
    contract::Candidate {
        id: literal(id.as_str()),
        intervals: vec![contract::Interval {
            start_ticks: start,
            end_ticks: end,
        }],
        proposer: proposer.clone(),
        evidence,
        roles: contract::CandidateRoles {
            hook: seed.hook.clone(),
            payoff: seed.payoff.clone(),
        },
        boundary_lattice: expanded.published(mid_word),
        // Fit is always legal at this phase, so nothing can be infeasible.
        layout_requirements: Vec::new(),
        // Overwritten once the clusters are known; a candidate is never
        // published without one.
        cluster_id: literal("cl_0000000000000000"),
        prelim_score: clamp_unit(seed.score),
        exclusions,
    }
}

/// Fold a second nomination of the same interval into the first.
///
/// The evidence is the union, because both seeds are reasons the interval is
/// worth clipping. The score is the higher of the two: a proposer that found
/// one clip by two routes is more confident in it, not less. Roles are kept
/// from whichever nomination had them, and the earlier one wins a tie so the
/// result does not depend on seed order.
fn absorb(into: &mut contract::Candidate, from: contract::Candidate) {
    into.evidence.extend(from.evidence);
    into.evidence
        .sort_by_key(|reference| (kind_ordinal(reference.kind), reference.index));
    into.evidence
        .dedup_by_key(|reference| (kind_ordinal(reference.kind), reference.index));
    if into.roles.hook.is_none() {
        into.roles.hook = from.roles.hook;
    }
    if into.roles.payoff.is_none() {
        into.roles.payoff = from.roles.payoff;
    }
    into.prelim_score = into.prelim_score.max(from.prelim_score);
    if into.exclusions.is_empty() {
        into.exclusions = from.exclusions;
    }
}

/// A short, content-derived name.
///
/// Derived rather than counted so that two runs over the same recording name
/// the same candidate, and so that a reordering upstream cannot rename one.
/// Sixteen hex digits: a candidate set is thousands of entries at most, and the
/// collision a longer name would prevent is one this scale never reaches.
fn identity(prefix: &str, parts: &[String]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"clipmill.discovery.v1\0");
    for part in parts {
        hasher.update(part.as_bytes());
        hasher.update(b"\0");
    }
    let digest = hasher.finalize();
    let mut name = String::from(prefix);
    for byte in &digest[..8] {
        // Two lowercase hex digits per byte, written directly: the pattern the
        // contract's newtype accepts, and one the formatter cannot drift from.
        const HEX: &[u8; 16] = b"0123456789abcdef";
        name.push(char::from(HEX[usize::from(byte >> 4)]));
        name.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    name
}

/// Content words of one sentence, shared by novelty and by nothing else that
/// would want a different definition.
fn tokens(text: &str) -> BTreeMap<String, u64> {
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    for raw in text.split_whitespace() {
        let token = raw
            .chars()
            .filter(|character| character.is_alphanumeric())
            .flat_map(char::to_lowercase)
            .collect::<String>();
        if token.chars().count() < 2 {
            continue;
        }
        *counts.entry(token).or_insert(0) += 1;
    }
    counts
}

fn kind_ordinal(kind: contract::EvidenceReferenceKind) -> u8 {
    match kind {
        contract::EvidenceReferenceKind::Utterance => 0,
        contract::EvidenceReferenceKind::Sentence => 1,
        contract::EvidenceReferenceKind::Topic => 2,
    }
}

fn clamp_unit(value: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else {
        published(value.clamp(0.0, 1.0))
    }
}

/// Six decimal places, applied to every float that reaches a published
/// document.
///
/// Two reasons, and the second is the load-bearing one. A score carrying
/// seventeen significant digits claims a precision no hand-set weight has. And
/// a document whose bytes depend on the last bits of a double is a document
/// three languages cannot agree about: Rust's JSON parser is only exact with
/// `float_roundtrip` enabled, and even where every reader is exact, an artifact
/// key computed over such a value is one nobody can reproduce by recomputing
/// the number a slightly different way. Six places is far more resolution than
/// any of these measurements has, and short decimals survive every parser.
pub(crate) fn published(value: f64) -> f64 {
    if value.is_finite() {
        (value * 1_000_000.0).round() / 1_000_000.0
    } else {
        0.0
    }
}

fn as_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

#[allow(
    clippy::cast_precision_loss,
    reason = "tick spans well inside f64's exact integer range"
)]
fn ticks_f64(value: u64) -> f64 {
    value as f64
}

#[allow(
    clippy::cast_precision_loss,
    reason = "counts and tick spans well inside f64's exact integer range"
)]
fn as_f64(value: usize) -> f64 {
    value as f64
}

/// A string the contract's newtype accepts.
///
/// Only for values this crate produced and therefore knows the shape of — a
/// proposer name, a derived identity. Anything arriving from outside goes
/// through `parse`, which has somewhere to put the failure.
fn literal<T: std::str::FromStr>(value: &str) -> T {
    value
        .parse()
        .unwrap_or_else(|_| unreachable!("this crate produced the value"))
}

fn parse<T: std::str::FromStr>(value: &str, field: &'static str) -> Result<T, DiscoveryError> {
    value
        .parse()
        .map_err(|_| DiscoveryError::MalformedAddress { field })
}

/// Every set is a set: no candidate id appears twice, and every one belongs to
/// exactly one cluster. Exported because the daemon asserts it before
/// publishing and the gate asserts it over real documents — a property worth
/// checking in both places is worth having one implementation of.
#[must_use]
pub fn is_well_formed(document: &contract::DiscoveryCandidates) -> bool {
    let ids = document
        .candidates
        .iter()
        .map(|candidate| candidate.id.as_str())
        .collect::<BTreeSet<_>>();
    if ids.len() != document.candidates.len() {
        return false;
    }
    let mut clustered = BTreeSet::new();
    for cluster in &document.clusters {
        for member in &cluster.members {
            if !clustered.insert(member.as_str()) || !ids.contains(member.as_str()) {
                return false;
            }
        }
        if !cluster
            .members
            .iter()
            .any(|member| member.as_str() == cluster.representative.as_str())
        {
            return false;
        }
    }
    clustered.len() == ids.len()
}

#[cfg(test)]
mod tests;
