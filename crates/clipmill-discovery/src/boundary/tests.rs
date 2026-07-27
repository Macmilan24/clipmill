//! J against brute force, and the speech rules as preferences rather than laws.
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::float_cmp)]

use crate::fixture::{self, SECOND};

use super::{Context, judge, opens_unresolved, optimize};

fn context(index: &clipmill_contracts::schemas::index_transcript::IndexTranscript) -> Context<'_> {
    Context {
        sentences: &index.sentences,
        utterances: &index.utterances,
        edges: &index.edges,
    }
}

/// The property that matters: exhaustive means exhaustive. Whatever the terms
/// weigh, the winner must be the argmax over every legal pair.
#[test]
fn the_chosen_pair_is_the_argmax_over_the_whole_lattice() {
    let interview = fixture::interview();
    let context = context(&interview.index);
    let starts = (0..12).map(|n| n * 3 * SECOND).collect::<Vec<_>>();
    let ends = (6..20).map(|n| n * 3 * SECOND).collect::<Vec<_>>();
    let duration = (15 * SECOND, 45 * SECOND);
    let choice = optimize(&starts, &ends, duration, &context).expect("a choice");

    let mut best = f64::NEG_INFINITY;
    let mut legal = 0u64;
    for start in &starts {
        for end in &ends {
            if start >= end {
                continue;
            }
            let span = end - start;
            if span < duration.0 || span > duration.1 {
                continue;
            }
            legal += 1;
            best = best.max(judge(*start, *end, &context).total());
        }
    }
    assert_eq!(choice.considered, legal, "not every legal pair was scored");
    assert_eq!(choice.score, best, "the winner is not the argmax");
}

/// The runner-up ships beside the winner because the optimizer's second choice
/// is frequently the editor's first.
#[test]
fn the_runner_up_is_the_second_best_and_is_not_the_winner() {
    let interview = fixture::interview();
    let context = context(&interview.index);
    let starts = (0..8).map(|n| n * 4 * SECOND).collect::<Vec<_>>();
    let ends = (5..16).map(|n| n * 4 * SECOND).collect::<Vec<_>>();
    let duration = (15 * SECOND, 60 * SECOND);
    let choice = optimize(&starts, &ends, duration, &context).expect("a choice");
    let (interval, score) = choice.alternative.expect("a runner-up");
    assert_ne!(interval, choice.chosen);
    assert!(score <= choice.score);

    // Nothing legal scores between the two.
    for start in &starts {
        for end in &ends {
            if start >= end {
                continue;
            }
            let span = end - start;
            if span < duration.0 || span > duration.1 {
                continue;
            }
            let here = judge(*start, *end, &context).total();
            assert!(
                here <= score || (*start, *end) == choice.chosen,
                "a pair scores between the winner and the runner-up"
            );
        }
    }
}

/// One legal pair is no choice at all, and the contract lets the alternative be
/// absent rather than repeating the winner.
#[test]
fn a_lattice_with_one_legal_pair_offers_no_alternative() {
    let interview = fixture::interview();
    let context = context(&interview.index);
    let choice =
        optimize(&[0], &[20 * SECOND], (15 * SECOND, 25 * SECOND), &context).expect("a choice");
    assert_eq!(choice.considered, 1);
    assert!(choice.alternative.is_none());
}

#[test]
fn nothing_legal_is_none_rather_than_a_bad_answer() {
    let interview = fixture::interview();
    let context = context(&interview.index);
    // Every pair here is far shorter than the minimum.
    assert!(
        optimize(
            &[0, SECOND],
            &[2 * SECOND],
            (15 * SECOND, 60 * SECOND),
            &context
        )
        .is_none()
    );
}

/// Cutting through a sentence is the failure a viewer notices first, so it
/// carries the heaviest weight — and the term reports it plainly.
#[test]
fn a_boundary_inside_a_sentence_scores_worse_than_one_between_them() {
    let interview = fixture::interview();
    let context = context(&interview.index);
    let first = &interview.index.sentences[0];
    let second = &interview.index.sentences[1];
    let clean = judge(first.start_ticks, second.end_ticks, &context);
    let torn = judge(first.start_ticks + SECOND / 2, second.end_ticks, &context);
    assert!(torn.total() < clean.total());
    assert!(torn.abrupt > clean.abrupt);
}

/// "Never open on an unresolvable pronoun" is a preference a strong enough
/// hook can outweigh — which is exactly what the design's worked example does
/// when it prefers a cold open. A hard filter would forbid that.
#[test]
fn context_debt_is_a_penalty_rather_than_a_prohibition() {
    assert!(opens_unresolved("And that is why it works"));
    assert!(opens_unresolved("They never tell you this."));
    assert!(!opens_unresolved("Charging less is lying to yourself."));

    let sentences = vec![
        fixture::sentence(0, 0, (0, 6), (0, 6 * SECOND), "So that is the reason."),
        fixture::sentence(
            1,
            1,
            (6, 6),
            (6 * SECOND, 12 * SECOND),
            "Pricing psychology rewards confidence.",
        ),
        fixture::sentence(
            2,
            2,
            (12, 6),
            (12 * SECOND, 18 * SECOND),
            "Charging less is lying to yourself.",
        ),
        fixture::sentence(
            3,
            3,
            (18, 6),
            (18 * SECOND, 24 * SECOND),
            "Buyers read the discount as doubt.",
        ),
    ];
    let index = fixture::indexed(
        sentences,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        (0, 30 * SECOND),
    );
    let context = context(&index);
    let indebted = judge(0, 18 * SECOND, &context);
    let clean = judge(12 * SECOND, 30 * SECOND, &context);
    assert!(indebted.context_debt > 0.0);
    assert_eq!(clean.context_debt, 0.0);
    // The debt is subtracted, not disqualifying: the indebted span still has a
    // real score rather than being removed from the search.
    assert!(indebted.total() > f64::NEG_INFINITY);
}

/// Ending on a full stop is a closed thought; ending where the recording ran
/// out is the weakest of the three.
#[test]
fn the_payoff_term_reads_how_the_sentence_ended() {
    let interview = fixture::interview();
    let context = context(&interview.index);
    let punctuated = &interview.index.sentences[0];
    let closed = judge(punctuated.start_ticks, punctuated.end_ticks, &context);
    assert!(closed.payoff > 0.0);
}

/// A span holding no sentence is silence. Legal, almost certainly unwanted, and
/// the terms say so rather than a filter removing it.
#[test]
fn a_span_of_pure_silence_is_all_dead_air() {
    let sentences = vec![fixture::sentence(
        0,
        0,
        (0, 4),
        (0, 4 * SECOND),
        "Pricing psychology rewards confidence.",
    )];
    let index = fixture::indexed(
        sentences,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        (0, 60 * SECOND),
    );
    let context = context(&index);
    let quiet = judge(30 * SECOND, 50 * SECOND, &context);
    assert_eq!(quiet.deadair, 1.0);
    assert!(quiet.total() < 0.0);
}

#[test]
fn the_published_terms_carry_their_sign() {
    let interview = fixture::interview();
    let context = context(&interview.index);
    let terms = judge(0, 20 * SECOND, &context).published();
    assert_eq!(terms.len(), 7);
    for term in &terms {
        assert!((0.0..=1.0).contains(&term.value));
    }
    // Four reward, three penalise, and a reader does not have to know which.
    assert_eq!(terms.iter().filter(|term| term.weight > 0.0).count(), 4);
    assert_eq!(terms.iter().filter(|term| term.weight < 0.0).count(), 3);
}

#[test]
fn the_same_lattice_optimizes_the_same_way_twice() {
    let interview = fixture::interview();
    let context = context(&interview.index);
    let starts = (0..6).map(|n| n * 4 * SECOND).collect::<Vec<_>>();
    let ends = (5..12).map(|n| n * 4 * SECOND).collect::<Vec<_>>();
    let once = optimize(&starts, &ends, (15 * SECOND, 45 * SECOND), &context).expect("a choice");
    let twice = optimize(&starts, &ends, (15 * SECOND, 45 * SECOND), &context).expect("a choice");
    assert_eq!(once.chosen, twice.chosen);
    assert_eq!(once.score, twice.score);
    assert_eq!(once.alternative, twice.alternative);
}
