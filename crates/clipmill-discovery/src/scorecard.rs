//! What a clip is worth, decomposed into axes a user can argue with.
//!
//! ```text
//! q(c) = f(H, F, V, P, N, E, A, R) − U − C
//! ```
//!
//! Hook, flow, value, prompt relevance, novelty, evidence, craft, feasibility;
//! minus uncertainty and penalties (book ch. 16). The displayed number is a
//! 0–99 percentile *within this recording's cohort* — an editorial index, not a
//! probability, and not comparable across recordings. It is named
//! `display_score` in the contract for that reason.
//!
//! Three of the eight axes cannot be measured at this phase, and each is
//! reported `available: false` with a stated reason rather than scored:
//!
//! - **prompt relevance** has no prompt — prompt mode is a later proposer
//! - **craft** needs the loudness envelope, and a source with no audio has none
//! - **feasibility** is `1.0` and *is* available, because fit is always legal
//!   and that is a real measurement rather than a missing one
//!
//! The distinction matters more than it looks. A factor scored zero reads as a
//! measurement of badness; a factor given a neutral default reads as a
//! measurement at all. Both would let an axis nobody evaluated move a number a
//! user is asked to trust, and the whole point of decomposing the score is that
//! each part of it can be traced to something that happened.

use std::collections::BTreeMap;

use clipmill_contracts::schemas::discovery_candidates as discovery;
use clipmill_contracts::schemas::index_transcript as index;
use clipmill_contracts::schemas::ranking_set as contract;
use clipmill_contracts::schemas::speech_transcript as transcript;

/// Hand-set weights, calibrated against nothing yet, and named so.
pub const RUBRIC: &str = "scorecard-handset.v1";

const W_HOOK: f64 = 1.4;
const W_FLOW: f64 = 1.1;
const W_VALUE: f64 = 1.0;
const W_PROMPT: f64 = 1.3;
const W_NOVELTY: f64 = 0.9;
const W_EVIDENCE: f64 = 0.8;
const W_CRAFT: f64 = 0.5;
const W_FEASIBILITY: f64 = 0.4;

/// Above this, the interface says "needs review" rather than "promising".
const NEEDS_REVIEW: f64 = 0.5;
/// Below this, "strong".
const STRONG: f64 = 0.2;

/// Everything the card is measured against.
pub(crate) struct Evidence<'a> {
    pub index: &'a index::IndexTranscript,
    pub words: &'a [transcript::Word],
    pub has_audio: bool,
}

/// One scored candidate, before the cohort turns raw scores into percentiles.
#[derive(Clone, Debug)]
pub(crate) struct Card {
    pub score: f64,
    pub factors: Vec<contract::Factor>,
    pub uncertainty: contract::Uncertainty,
    pub penalties: Vec<contract::Penalty>,
}

/// Measure one candidate over the interval the boundary optimizer chose.
///
/// The interval matters: discovery's candidate carries the tightest legal span,
/// and ranking has since chosen a different one. Scoring the old span would
/// describe a clip nobody is going to see.
#[allow(
    clippy::too_many_lines,
    reason = "eight axes in the order the design lists them; splitting hides it"
)]
pub(crate) fn measure(
    candidate: &discovery::Candidate,
    interval: (u64, u64),
    evidence: &Evidence<'_>,
) -> Card {
    let (start, end) = interval;
    let inside = evidence
        .index
        .sentences
        .iter()
        .enumerate()
        .filter(|(_, sentence)| sentence.start_ticks < end && sentence.end_ticks > start)
        .collect::<Vec<_>>();

    let mut card = Builder::default();

    // Hook: the sentence a viewer hears first, and whether it says something.
    let opening = inside.first().map(|(at, sentence)| (*at, *sentence));
    let hook = opening.map_or(0.0, |(_, sentence)| {
        let punctuated = f64::from(u8::from(matches!(
            sentence.terminator,
            index::SentenceTerminator::Punctuation
        )));
        let claims = f64::from(u8::from(!crate::boundary::opens_unresolved(
            sentence.text.as_str(),
        )));
        0.5 * claims + 0.3 * punctuated + 0.2 * (sentence.words_per_minute / 200.0).min(1.0)
    });
    card.measured(
        contract::FactorName::Hook,
        hook,
        W_HOOK,
        opening.map(|(at, _)| at).into_iter().collect(),
    );

    // Flow: whole sentences rather than fragments, and speech that keeps going.
    let whole = inside
        .iter()
        .filter(|(_, sentence)| sentence.start_ticks >= start && sentence.end_ticks <= end)
        .count();
    let flow = if inside.is_empty() {
        0.0
    } else {
        crate::as_f64(whole) / crate::as_f64(inside.len())
    };
    card.measured(contract::FactorName::Flow, flow, W_FLOW, Vec::new());

    // Value: how much of this stretch is a speaker taking a position, by the
    // same lexicon the quote proposer used. A proxy, and a shallow one.
    let value = if inside.is_empty() {
        0.0
    } else {
        inside
            .iter()
            .map(|(_, sentence)| crate::proposers::claim_language(sentence.text.as_str()))
            .sum::<f64>()
            / crate::as_f64(inside.len())
    };
    card.measured(
        contract::FactorName::Value,
        value,
        W_VALUE,
        inside.iter().map(|(at, _)| *at).collect(),
    );

    // Prompt relevance: there is no prompt. Prompt mode is a later proposer,
    // and scoring this axis against nothing would be inventing a measurement.
    card.unavailable(
        contract::FactorName::PromptRelevance,
        "no prompt was given; prompt retrieval is not one of this phase's proposers",
    );
    let _ = W_PROMPT;

    // Novelty: vocabulary this stretch uses that the rest of the recording does
    // not, from the same measure discovery ranked sentences by.
    let novelty = crate::proposers::Novelty::measure(evidence.index);
    let novel = if inside.is_empty() {
        0.0
    } else {
        inside.iter().map(|(at, _)| novelty.of(*at)).sum::<f64>() / crate::as_f64(inside.len())
    };
    card.measured(contract::FactorName::Novelty, novel, W_NOVELTY, Vec::new());

    // Evidence strength: how much of the span has word timing somebody
    // measured, and how confident the recognizer was about the words.
    let (aligned, confidence) = word_support(evidence.words, start, end);
    card.measured(
        contract::FactorName::Evidence,
        0.5 * aligned + 0.5 * confidence,
        W_EVIDENCE,
        Vec::new(),
    );

    // Craft: needs audio. Everything else the design lists — framing, motion,
    // reaction shots — needs vision this phase does not run, so what is
    // measured is delivery consistency and nothing more, and the factor is
    // absent entirely on a source with no audio.
    if evidence.has_audio {
        let rates = inside
            .iter()
            .map(|(_, sentence)| sentence.words_per_minute)
            .collect::<Vec<_>>();
        card.measured(
            contract::FactorName::Craft,
            steadiness(&rates),
            W_CRAFT,
            Vec::new(),
        );
    } else {
        card.unavailable(
            contract::FactorName::Craft,
            "this source carries no audio, so delivery was not measured",
        );
    }

    // Feasibility: available, and always one. Fit is legal for every clip, so
    // nothing can be infeasible — a real measurement with a constant answer,
    // which is different from an axis nobody evaluated.
    card.measured(
        contract::FactorName::Feasibility,
        1.0,
        W_FEASIBILITY,
        Vec::new(),
    );

    let (uncertainty, warnings) = uncertainty(candidate, evidence, start, end, aligned);
    let penalties = penalties(candidate, opening.map(|(_, sentence)| sentence));
    let score =
        card.score - uncertainty - penalties.iter().map(|penalty| penalty.value).sum::<f64>();

    Card {
        score,
        factors: card.factors,
        uncertainty: contract::Uncertainty {
            value: crate::clamp_unit(uncertainty),
            band: if uncertainty >= NEEDS_REVIEW {
                contract::UncertaintyBand::NeedsReview
            } else if uncertainty <= STRONG {
                contract::UncertaintyBand::Strong
            } else {
                contract::UncertaintyBand::Promising
            },
            warnings,
        },
        penalties,
    }
}

/// Accumulates the card so that a measured axis and an unmeasured one are
/// added through the same door — the difference between them is one field, and
/// keeping the two paths adjacent is what stops a future axis quietly
/// defaulting to zero.
#[derive(Default)]
struct Builder {
    factors: Vec<contract::Factor>,
    score: f64,
}

impl Builder {
    fn measured(&mut self, name: contract::FactorName, value: f64, weight: f64, cites: Vec<usize>) {
        let value = crate::clamp_unit(value);
        self.score += weight * value;
        self.factors.push(contract::Factor {
            name,
            available: true,
            value: Some(value),
            weight: Some(weight),
            unavailable_reason: None,
            evidence: cites.into_iter().map(sentence_reference).collect(),
        });
    }

    /// Contributes nothing to the score. Not zero, which would read as a
    /// measurement of badness; not a neutral default, which would read as a
    /// measurement at all.
    fn unavailable(&mut self, name: contract::FactorName, reason: &str) {
        self.factors.push(contract::Factor {
            name,
            available: false,
            value: None,
            weight: None,
            unavailable_reason: reason.parse().ok(),
            evidence: Vec::new(),
        });
    }
}

fn sentence_reference(at: usize) -> contract::EvidenceReference {
    contract::EvidenceReference {
        kind: contract::EvidenceReferenceKind::Sentence,
        index: crate::as_u64(at),
    }
}

/// The share of a span whose words were aligned rather than interpolated, and
/// the recognizer's own confidence across it.
fn word_support(words: &[transcript::Word], start: u64, end: u64) -> (f64, f64) {
    let inside = words
        .iter()
        .filter(|word| word.start_ticks < end && word.end_ticks > start)
        .collect::<Vec<_>>();
    if inside.is_empty() {
        return (0.0, 0.0);
    }
    let measured = inside
        .iter()
        .filter(|word| matches!(word.timing, transcript::WordTiming::Aligned))
        .count();
    let confidences = inside
        .iter()
        .map(|word| word.confidence.p10)
        .collect::<Vec<_>>();
    // The pessimistic quantile, not the median: ranking decides whether a quote
    // is safe to put on screen, and that question is about the worst words in
    // it rather than the typical ones.
    let (_, p10) = clipmill_evidence::confidence::distribution(&confidences);
    (crate::as_f64(measured) / crate::as_f64(inside.len()), p10)
}

/// One minus the normalized spread of a set of rates. A speaker whose pace is
/// even reads as composed; one whose pace lurches reads as unedited.
fn steadiness(rates: &[f64]) -> f64 {
    if rates.len() < 2 {
        return 0.5;
    }
    let mean = rates.iter().sum::<f64>() / crate::as_f64(rates.len());
    if mean <= 0.0 {
        return 0.0;
    }
    let spread =
        rates.iter().map(|rate| (rate - mean).abs()).sum::<f64>() / crate::as_f64(rates.len());
    crate::clamp_unit(1.0 - spread / mean)
}

/// How much this card should be trusted, and the specific reasons why not.
fn uncertainty(
    candidate: &discovery::Candidate,
    evidence: &Evidence<'_>,
    start: u64,
    end: u64,
    aligned: f64,
) -> (f64, Vec<contract::UncertaintyWarningsItem>) {
    let mut value = 0.0;
    let mut warnings = Vec::new();

    if aligned < 1.0 {
        value += 0.3 * (1.0 - aligned);
        warnings.push(crate::literal(
            "word timing here was interpolated rather than measured",
        ));
    }
    if evidence
        .index
        .invalid_regions
        .iter()
        .any(|region| region.start_ticks < end && region.end_ticks > start)
    {
        value += 0.2;
        warnings.push(crate::literal(
            "part of this span lies in a region the transcript does not vouch for",
        ));
    }
    // A lattice with one legal pair gave the optimizer no choice, so its
    // boundary is a consequence of the recording rather than a decision.
    if candidate.boundary_lattice.starts.len() == 1 && candidate.boundary_lattice.ends.len() == 1 {
        value += 0.15;
        warnings.push(crate::literal(
            "the lattice offered one legal boundary, so none was chosen",
        ));
    }
    // Three of eight axes unmeasured is a card built on less than the design
    // intends, and a user comparing it against a fully measured one should be
    // told.
    if !evidence.has_audio {
        value += 0.1;
        warnings.push(crate::literal(
            "no audio, so delivery contributed nothing to this score",
        ));
    }
    (value.min(1.0), warnings)
}

/// What is subtracted, and why. Repetition is applied by the cohort rather than
/// here, because it is a fact about a candidate's neighbours.
fn penalties(
    candidate: &discovery::Candidate,
    opening: Option<&index::Sentence>,
) -> Vec<contract::Penalty> {
    let mut penalties = Vec::new();
    if opening.is_some_and(|sentence| crate::boundary::opens_unresolved(sentence.text.as_str())) {
        penalties.push(contract::Penalty {
            reason: contract::PenaltyReason::ContextDebt,
            value: 0.25,
            detail: "the clip opens on a word referring to something the viewer has not seen"
                .parse()
                .ok(),
        });
    }
    // `rights_risk` cannot fire: there is no rights ledger at this phase. It is
    // in the contract so the shape does not change when there is one, and
    // absent here rather than present at zero.
    let _ = candidate;
    penalties
}

/// Turn raw scores into the 0–99 the interface shows.
///
/// A percentile within the cohort, which is what makes it an editorial index:
/// the best clip in a weak recording still shows well, because the question a
/// user is asking is "which of these should I look at" rather than "is this
/// good in absolute terms". Ties share a percentile, and the raw score travels
/// beside it so two candidates a rounding merged can still be told apart.
pub(crate) fn percentiles(scores: &[f64]) -> BTreeMap<usize, u8> {
    let mut order = (0..scores.len()).collect::<Vec<_>>();
    order.sort_by(|left, right| scores[*left].total_cmp(&scores[*right]));
    let mut ranks = BTreeMap::new();
    let total = scores.len();
    let mut position = 0;
    while position < total {
        // Every candidate with this score gets the same percentile.
        let mut tie = position + 1;
        // Exact: candidates that scored identically share a percentile, and a
        // tolerance would merge two that did not.
        while tie < total
            && scores[order[tie]].total_cmp(&scores[order[position]]) == std::cmp::Ordering::Equal
        {
            tie += 1;
        }
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a percentile, clamped into a byte by construction"
        )]
        let percentile = if total == 1 {
            99
        } else {
            (crate::as_f64(position) / crate::as_f64(total - 1) * 99.0).round() as u8
        };
        for slot in &order[position..tie] {
            ranks.insert(*slot, percentile.min(99));
        }
        position = tie;
    }
    ranks
}

#[cfg(test)]
mod tests;
