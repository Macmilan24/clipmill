//! Expansion: from a seed to every legal interval around it.
//!
//! A proposer says "something worth clipping happens here". It does not say
//! where the clip starts, and it should not: the boundary that reads best
//! depends on speech rules, duration, and what the viewer sees, and the stage
//! that knows those things is ranking. So a seed becomes a *lattice* — every
//! place a clip could legally start, every place it could legally end — and
//! ranking searches it (book ch. 15).
//!
//! Keeping the whole lattice rather than choosing is the point. Discovery has
//! less information than the stage that has to live with the choice, and a
//! pre-chosen boundary is a decision made by whoever had the least reason to
//! make it.
//!
//! Legality is `Φ`. Two of its terms are real here and the rest are absent
//! rather than always-passing: a start that lands inside a word is structurally
//! illegal and never enters the lattice at all, and a pair outside the
//! requested duration range or outside coverage is rejected. The design also
//! names open loops, identity discontinuity, rights exclusions, and layout
//! infeasibility. None of those can be measured at this phase — there is no
//! open-loop detection, no diarization, no rights ledger, and the fit layout is
//! always legal — so they are omitted. A term recorded as never firing reads
//! like a term that was checked.

use std::collections::{BTreeMap, BTreeSet};

use clipmill_contracts::schemas::discovery_candidates as contract;
use clipmill_contracts::schemas::index_transcript as index;
use clipmill_contracts::schemas::speech_transcript as transcript;

/// How far either side of a seed the expander looks for a boundary.
///
/// Bounded by the longest clip anyone can ask for: a start further back than
/// that could never pair with an end after the seed and still be legal, so
/// gathering it would only make the lattice bigger without making it wider.
fn window(duration: &contract::DurationRange) -> u64 {
    duration.max_ticks.get()
}

/// The candidate boundaries a recording offers, gathered once.
///
/// Built for the whole recording rather than per seed, because every seed
/// draws from the same three sources and rebuilding the set per proposer would
/// be the same work done nine times with a chance of disagreeing.
#[derive(Clone, Debug)]
pub(crate) struct Boundaries {
    /// Ticks a clip may start on, ascending and distinct.
    starts: Vec<u64>,
    /// Ticks a clip may end on.
    ends: Vec<u64>,
    /// How many candidate points the structural term removed.
    pub(crate) mid_word_rejects: u64,
}

impl Boundaries {
    /// Gather every boundary the index and the transcript offer.
    ///
    /// Sentence and utterance edges come from the index and are word-aligned by
    /// construction. Silence spans contribute both edges, because a clip may
    /// begin anywhere in a silence and beginning at its far edge is what avoids
    /// a leading gap. Shot cuts are the only source that can land inside a
    /// word — they were measured against pixels, which know nothing about
    /// speech — so they are the only ones the structural test can reject.
    pub(crate) fn gather(document: &index::IndexTranscript, words: &[transcript::Word]) -> Self {
        let mut starts = BTreeSet::new();
        let mut ends = BTreeSet::new();
        for sentence in &document.sentences {
            starts.insert(sentence.start_ticks);
            ends.insert(sentence.end_ticks);
        }
        for utterance in &document.utterances {
            starts.insert(utterance.start_ticks);
            ends.insert(utterance.end_ticks);
        }
        let mut mid_word_rejects = 0;
        for edge in &document.edges {
            // A silence contributes both of its edges to both sets: a clip may
            // begin anywhere inside one, and beginning at the far edge is what
            // avoids a leading gap. A shot cut contributes one instant, and is
            // the only source that can land inside a word — it was measured
            // against pixels, which know nothing about speech.
            let cut = matches!(edge.kind, index::EdgeKind::ShotCut);
            for at in [edge.start_ticks, edge.end_ticks] {
                if cut && inside_a_word(words, at) {
                    mid_word_rejects += 1;
                    continue;
                }
                starts.insert(at);
                ends.insert(at);
            }
        }
        // The recording's own bounds are legal boundaries: a clip may start at
        // the beginning and end at the end, whatever else was detected.
        starts.insert(document.coverage.start_ticks);
        ends.insert(document.coverage.end_ticks);

        Self {
            starts: starts.into_iter().collect(),
            ends: ends.into_iter().collect(),
            // A shot cut carries the same tick as its start and its end, so
            // the loop above tested it twice. Reporting it once is what a
            // reader means by "this cut was rejected".
            mid_word_rejects: mid_word_rejects / 2,
        }
    }

    /// The lattice around one seed, and what `Φ` removed building it.
    ///
    /// Returns `None` when nothing legal survives, which is a fact about the
    /// recording rather than a failure: a seed inside a stretch shorter than
    /// the shortest requested clip has nowhere legal to grow to.
    pub(crate) fn expand(
        &self,
        seed: (u64, u64),
        duration: &contract::DurationRange,
        coverage: (u64, u64),
    ) -> Option<Lattice> {
        let (low, high) = seed;
        let reach = window(duration);
        let starts = self
            .starts
            .iter()
            .copied()
            .filter(|at| *at <= low && low.saturating_sub(*at) <= reach && *at >= coverage.0)
            .collect::<Vec<_>>();
        let ends = self
            .ends
            .iter()
            .copied()
            .filter(|at| *at >= high && at.saturating_sub(high) <= reach && *at <= coverage.1)
            .collect::<Vec<_>>();
        if starts.is_empty() || ends.is_empty() {
            return None;
        }

        // Every pair is tested, and the rejections are counted by reason rather
        // than listed: the pairs are the product of the two sets, so recording
        // each one would make the artifact quadratic in the recording's length
        // to say something a consumer can recompute from the bounds.
        let mut rejects: BTreeMap<contract::PhiRejectReason, u64> = BTreeMap::new();
        let mut legal_starts = BTreeSet::new();
        let mut legal_ends = BTreeSet::new();
        let mut best: Option<(u64, u64)> = None;
        for start in &starts {
            for end in &ends {
                match legality(*start, *end, duration, coverage) {
                    Ok(()) => {
                        legal_starts.insert(*start);
                        legal_ends.insert(*end);
                        // The tightest legal interval containing the seed: the
                        // one that adds least to what the proposer pointed at.
                        let span = end - start;
                        if best.is_none_or(|(from, to)| span < to - from) {
                            best = Some((*start, *end));
                        }
                    }
                    Err(reason) => *rejects.entry(reason).or_insert(0) += 1,
                }
            }
        }
        let (start, end) = best?;
        Some(Lattice {
            interval: (start, end),
            starts: legal_starts.into_iter().collect(),
            ends: legal_ends.into_iter().collect(),
            rejects,
        })
    }
}

/// One seed's legal boundaries, plus the interval it settled on.
#[derive(Clone, Debug)]
pub(crate) struct Lattice {
    /// The tightest legal interval containing the seed. A starting point for
    /// ranking's search, not a decision.
    pub(crate) interval: (u64, u64),
    pub(crate) starts: Vec<u64>,
    pub(crate) ends: Vec<u64>,
    rejects: BTreeMap<contract::PhiRejectReason, u64>,
}

impl Lattice {
    /// The published form, with the structural rejections folded in.
    pub(crate) fn published(&self, mid_word: u64) -> contract::BoundaryLattice {
        let mut rejects = self
            .rejects
            .iter()
            .filter_map(|(reason, count)| {
                Some(contract::PhiReject {
                    reason: *reason,
                    count: std::num::NonZeroU64::new(*count)?,
                })
            })
            .collect::<Vec<_>>();
        if let Some(count) = std::num::NonZeroU64::new(mid_word) {
            rejects.push(contract::PhiReject {
                reason: contract::PhiRejectReason::MidWord,
                count,
            });
        }
        rejects.sort_by_key(|reject| format!("{:?}", reject.reason));
        contract::BoundaryLattice {
            starts: self.starts.clone(),
            ends: self.ends.clone(),
            phi_rejects: rejects,
        }
    }
}

/// `Φ`, for one pair. The terms that can be measured at this phase.
fn legality(
    start: u64,
    end: u64,
    duration: &contract::DurationRange,
    coverage: (u64, u64),
) -> Result<(), contract::PhiRejectReason> {
    if start < coverage.0 || end > coverage.1 || start >= end {
        return Err(contract::PhiRejectReason::OutsideCoverage);
    }
    let span = end - start;
    if span < duration.min_ticks.get() {
        return Err(contract::PhiRejectReason::TooShort);
    }
    if span > duration.max_ticks.get() {
        return Err(contract::PhiRejectReason::TooLong);
    }
    Ok(())
}

/// Whether a tick falls strictly inside a word.
///
/// Strictly: a tick exactly on a word's start or end is a boundary, not an
/// interruption. Words are ordered and non-overlapping, so this is a binary
/// search rather than a scan — a recording with fifty thousand words and a
/// thousand shot cuts would otherwise be fifty million comparisons for a
/// question with a logarithmic answer.
fn inside_a_word(words: &[transcript::Word], at: u64) -> bool {
    let found = words.partition_point(|word| word.start_ticks <= at);
    found
        .checked_sub(1)
        .and_then(|position| words.get(position))
        .is_some_and(|word| at > word.start_ticks && at < word.end_ticks)
}

#[cfg(test)]
mod tests;
