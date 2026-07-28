//! The card, and the difference between a zero and an absence.
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::float_cmp)]

use clipmill_contracts::schemas::ranking_set as contract;

use crate::fixture::{self, SECOND};

use super::{Evidence, measure, percentiles, steadiness};

fn card(has_audio: bool) -> super::Card {
    let interview = fixture::interview();
    let found = crate::discover(
        &interview.index,
        &interview.transcript,
        None,
        crate::Inputs {
            index: fixture::INDEX_ID,
            transcript: fixture::TRANSCRIPT_ID,
            loudness: None,
        },
        crate::Parameters::DEFAULT,
        fixture::IMPLEMENTATION,
    )
    .expect("the search runs");
    let candidate = found.candidates.first().expect("a candidate");
    let interval = (
        candidate.intervals[0].start_ticks,
        candidate.intervals[0].end_ticks,
    );
    measure(
        candidate,
        interval,
        &Evidence {
            index: &interview.index,
            words: &interview.transcript.words,
            has_audio,
        },
    )
}

fn factor(card: &super::Card, name: contract::FactorName) -> &contract::Factor {
    card.factors
        .iter()
        .find(|factor| factor.name == name)
        .expect("the card carries every axis")
}

/// All eight axes are always present. A card that grew one and a card that lost
/// one must be distinguishable documents, which they are not if a missing axis
/// is simply absent from the list.
#[test]
fn the_card_names_all_eight_axes_whether_or_not_it_measured_them() {
    let card = card(true);
    assert_eq!(card.factors.len(), 8);
    for name in [
        contract::FactorName::Hook,
        contract::FactorName::Flow,
        contract::FactorName::Value,
        contract::FactorName::PromptRelevance,
        contract::FactorName::Novelty,
        contract::FactorName::Evidence,
        contract::FactorName::Craft,
        contract::FactorName::Feasibility,
    ] {
        let _ = factor(&card, name);
    }
}

/// The distinction the whole module is built around. An axis nobody measured
/// contributes nothing and says why — it does not score zero, which would read
/// as a measurement of badness.
#[test]
fn an_unmeasured_axis_says_so_rather_than_scoring_zero() {
    let card = card(true);
    let prompt = factor(&card, contract::FactorName::PromptRelevance);
    assert!(!prompt.available);
    assert!(prompt.value.is_none(), "an absence must not carry a value");
    assert!(prompt.weight.is_none(), "an absence must not be weighted");
    assert!(
        prompt
            .unavailable_reason
            .as_ref()
            .is_some_and(|reason| reason.as_str().contains("no prompt")),
        "an absence must state its reason"
    );
}

/// Feasibility is one, and *available*: fit is always legal, which is a real
/// measurement with a constant answer rather than an axis nobody evaluated.
#[test]
fn a_constant_measurement_is_still_a_measurement() {
    let card = card(true);
    let feasibility = factor(&card, contract::FactorName::Feasibility);
    assert!(feasibility.available);
    assert_eq!(feasibility.value, Some(1.0));
}

/// Audio is what the craft axis reads. Without it the axis is absent, and the
/// score is lower only because one weighted term is gone — not because craft
/// was scored badly.
#[test]
fn without_audio_craft_is_absent_rather_than_poor() {
    let heard = card(true);
    let silent = card(false);
    assert!(factor(&heard, contract::FactorName::Craft).available);
    let craft = factor(&silent, contract::FactorName::Craft);
    assert!(!craft.available);
    assert!(craft.value.is_none());
    assert!(
        craft
            .unavailable_reason
            .as_ref()
            .is_some_and(|reason| reason.as_str().contains("no audio"))
    );
    // And the card says the score rests on less than it should.
    assert!(
        silent
            .uncertainty
            .warnings
            .iter()
            .any(|warning| warning.as_str().contains("no audio"))
    );
}

#[test]
fn a_measured_axis_carries_its_weight_and_its_evidence() {
    let card = card(true);
    let hook = factor(&card, contract::FactorName::Hook);
    assert!(hook.available);
    assert!(hook.weight.is_some_and(|weight| weight > 0.0));
    assert!(
        !hook.evidence.is_empty(),
        "a user asking why the hook is strong should get the sentence"
    );
}

/// Uncertainty is translated into words rather than shading a hidden number,
/// and every band it can report is reachable.
#[test]
fn uncertainty_bands_say_how_much_to_trust_the_card() {
    let confident = card(true);
    assert!(matches!(
        confident.uncertainty.band,
        contract::UncertaintyBand::Strong | contract::UncertaintyBand::Promising
    ));
    assert!((0.0..=1.0).contains(&confident.uncertainty.value));
    let silent = card(false);
    assert!(silent.uncertainty.value > confident.uncertainty.value);
}

#[test]
fn steadiness_reads_the_spread_not_the_pace() {
    // A fast even speaker and a slow even speaker are equally composed.
    assert!(steadiness(&[100.0, 100.0, 100.0]) > 0.9);
    assert!(steadiness(&[220.0, 220.0, 220.0]) > 0.9);
    // A lurching one is not.
    assert!(steadiness(&[40.0, 300.0, 60.0]) < 0.5);
    // One sentence is not enough to tell, and says so with a middling answer
    // rather than a confident one.
    assert_eq!(steadiness(&[150.0]), 0.5);
}

/// The displayed number is a percentile within the cohort — an editorial index,
/// not a probability. The best clip in a weak recording still shows well,
/// because the question a user is asking is which of these to look at.
#[test]
fn percentiles_rank_within_the_cohort_and_share_ties() {
    let ranks = percentiles(&[0.1, 0.9, 0.5]);
    assert_eq!(ranks[&1], 99, "the best of the cohort tops out");
    assert_eq!(ranks[&0], 0, "the worst bottoms out");
    assert_eq!(ranks[&2], 50);

    // A single candidate is the whole cohort, so it is the best of it.
    assert_eq!(percentiles(&[0.3])[&0], 99);

    // Equal scores share a percentile rather than being ordered arbitrarily.
    let tied = percentiles(&[0.5, 0.5, 0.9]);
    assert_eq!(tied[&0], tied[&1]);
    assert!(tied[&2] > tied[&0]);
}

/// A weak recording's best clip and a strong recording's best clip both show 99.
/// That is the point of a percentile, and the reason the raw score travels
/// beside it.
#[test]
fn the_display_score_is_not_comparable_across_recordings() {
    assert_eq!(percentiles(&[0.01, 0.02])[&1], 99);
    assert_eq!(percentiles(&[8.0, 9.0])[&1], 99);
}

/// A clip over interpolated timing is scored, but the card says the timing
/// under it was not measured.
#[test]
fn interpolated_timing_lowers_confidence_and_names_itself() {
    let mut interview = fixture::interview();
    for word in &mut interview.transcript.words {
        word.timing = clipmill_contracts::schemas::speech_transcript::WordTiming::Interpolated;
    }
    let found = crate::discover(
        &interview.index,
        &interview.transcript,
        None,
        crate::Inputs {
            index: fixture::INDEX_ID,
            transcript: fixture::TRANSCRIPT_ID,
            loudness: None,
        },
        crate::Parameters::DEFAULT,
        fixture::IMPLEMENTATION,
    )
    .expect("the search runs");
    let candidate = found.candidates.first().expect("a candidate");
    let card = measure(
        candidate,
        (
            candidate.intervals[0].start_ticks,
            candidate.intervals[0].end_ticks,
        ),
        &Evidence {
            index: &interview.index,
            words: &interview.transcript.words,
            has_audio: true,
        },
    );
    assert!(
        card.uncertainty
            .warnings
            .iter()
            .any(|warning| warning.as_str().contains("interpolated"))
    );
}

#[test]
fn a_clip_opening_on_a_dangling_word_is_penalised_and_told_why() {
    let sentences = vec![
        fixture::sentence(0, 0, (0, 6), (0, 20 * SECOND), "So that is the reason."),
        fixture::sentence(
            1,
            1,
            (6, 6),
            (20 * SECOND, 40 * SECOND),
            "Pricing rewards confidence.",
        ),
    ];
    let index = fixture::indexed(
        sentences,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        (0, 60 * SECOND),
    );
    let transcript = fixture::spoken(Vec::new(), (0, 60 * SECOND));
    let found = crate::discover(
        &index,
        &transcript,
        None,
        crate::Inputs {
            index: fixture::INDEX_ID,
            transcript: fixture::TRANSCRIPT_ID,
            loudness: None,
        },
        crate::Parameters::DEFAULT,
        fixture::IMPLEMENTATION,
    )
    .expect("the search runs");
    let Some(candidate) = found.candidates.first() else {
        return;
    };
    let card = measure(
        candidate,
        (0, 40 * SECOND),
        &Evidence {
            index: &index,
            words: &transcript.words,
            has_audio: true,
        },
    );
    assert!(
        card.penalties
            .iter()
            .any(|penalty| penalty.reason == contract::PenaltyReason::ContextDebt),
        "a clip opening on 'So' owes the viewer context"
    );
}
