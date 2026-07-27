//! What each proposer must and must not claim.
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::float_cmp)]

use crate::fixture::{self, SECOND};
use crate::prosody::Prosody;

use super::{
    Novelty, claim_language, insight_quote, is_question, narrative_arc, question_answer,
    specificity,
};

fn measured() -> (
    clipmill_contracts::schemas::index_transcript::IndexTranscript,
    Novelty,
    Prosody,
) {
    let interview = fixture::interview();
    let novelty = Novelty::measure(&interview.index);
    let prosody = Prosody::measure(&interview.index, None);
    (interview.index, novelty, prosody)
}

#[test]
fn a_topic_becomes_a_seed_with_an_opening_and_a_close() {
    let (index, novelty, _) = measured();
    let seeds = narrative_arc(&index, &novelty);
    assert_eq!(seeds.len(), 2);
    for seed in &seeds {
        let hook = seed.hook.as_ref().expect("an opening");
        let payoff = seed.payoff.as_ref().expect("a close");
        // The hook is the first sentence and the payoff is later, or the label
        // means nothing.
        assert!(payoff.index >= hook.index);
    }
    assert_eq!(seeds[0].hook.as_ref().unwrap().index, 0);
    assert_eq!(seeds[1].hook.as_ref().unwrap().index, 5);
}

/// A topic of one sentence opens and closes on the same words. Nominating it
/// as a mini-story would be nominating the sentence twice — once here and once
/// by the quote proposer.
#[test]
fn a_single_sentence_topic_is_not_an_arc() {
    let mut index = fixture::interview().index;
    index.topics = vec![fixture::topic(0, (0, 1), (0, 10 * SECOND), 0.0)];
    let novelty = Novelty::measure(&index);
    assert!(narrative_arc(&index, &novelty).is_empty());
}

/// The payoff is the most information-dense sentence of the closing third, not
/// simply the last one — a topic that trails off should not have the trailing
/// off labelled as its payoff.
#[test]
fn the_payoff_is_chosen_by_novelty_not_by_position() {
    let sentences = vec![
        fixture::sentence(
            0,
            0,
            (0, 6),
            (0, 3 * SECOND),
            "The renderer draws frames again.",
        ),
        fixture::sentence(
            1,
            1,
            (6, 6),
            (4 * SECOND, 7 * SECOND),
            "The renderer draws frames again.",
        ),
        fixture::sentence(
            2,
            2,
            (12, 7),
            (8 * SECOND, 11 * SECOND),
            "Deterministic encoding pins every profile setting exactly.",
        ),
        fixture::sentence(
            3,
            3,
            (19, 4),
            (12 * SECOND, 14 * SECOND),
            "The renderer draws frames.",
        ),
    ];
    let index = fixture::indexed(
        sentences,
        Vec::new(),
        vec![fixture::topic(0, (0, 4), (0, 14 * SECOND), 0.0)],
        Vec::new(),
        (0, 20 * SECOND),
    );
    let novelty = Novelty::measure(&index);
    let seeds = narrative_arc(&index, &novelty);
    // Sentence 2 is the novel one; the closing third starts at index 2, so it
    // is reachable and should beat the repetitive final sentence.
    assert_eq!(seeds[0].payoff.as_ref().unwrap().index, 2);
}

#[test]
fn a_question_mark_and_a_wh_opening_are_both_questions() {
    let asked = fixture::sentence(0, 0, (0, 4), (0, SECOND), "What makes it fast?");
    let inverted = fixture::sentence(0, 0, (0, 4), (0, SECOND), "Is that the reason");
    let stated = fixture::sentence(0, 0, (0, 4), (0, SECOND), "That is the reason.");
    assert!(is_question(&asked));
    assert!(is_question(&inverted));
    assert!(!is_question(&stated));
}

#[test]
fn a_question_and_its_answer_become_one_seed() {
    let (index, _, _) = measured();
    let seeds = question_answer(&index);
    assert_eq!(seeds.len(), 2, "two questions were asked");
    for seed in &seeds {
        let hook = seed.hook.as_ref().expect("the question");
        let payoff = seed.payoff.as_ref().expect("the answer");
        assert!(payoff.index > hook.index, "an answer follows its question");
        assert!(seed.evidence.len() >= 2);
    }
}

/// A question nobody answered is a setup with no payoff, and clipping it would
/// publish the setup.
#[test]
fn an_unanswered_question_is_not_nominated() {
    let index = fixture::indexed(
        vec![fixture::sentence(
            0,
            0,
            (0, 4),
            (0, 2 * SECOND),
            "What makes it fast?",
        )],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        (0, 10 * SECOND),
    );
    assert!(question_answer(&index).is_empty());
}

/// The answer window closes at a pause long enough that the subject changed.
#[test]
fn a_long_pause_ends_the_answer() {
    let sentences = vec![
        fixture::sentence(0, 0, (0, 4), (0, 2 * SECOND), "What makes it fast?"),
        fixture::sentence(
            1,
            1,
            (4, 5),
            (2 * SECOND, 4 * SECOND),
            "Pinned encoder settings do.",
        ),
        // Five seconds of quiet, well past the two-second window.
        fixture::sentence(
            2,
            2,
            (9, 5),
            (9 * SECOND, 11 * SECOND),
            "Anyway the weather turned.",
        ),
    ];
    let index = fixture::indexed(
        sentences,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        (0, 20 * SECOND),
    );
    let seeds = question_answer(&index);
    assert_eq!(seeds.len(), 1);
    assert_eq!(seeds[0].payoff.as_ref().unwrap().index, 1);
}

#[test]
fn specificity_counts_numbers_and_mid_sentence_capitals() {
    // The opening capital is grammatical, not a name, and is skipped.
    assert_eq!(specificity("The renderer draws frames"), 0.0);
    assert!(specificity("We pinned FFmpeg at 8.1.2") > 0.0);
    assert_eq!(specificity(""), 0.0);
}

#[test]
fn claim_language_reads_the_lexicon_and_saturates() {
    assert_eq!(claim_language("the renderer draws frames"), 0.0);
    assert!(claim_language("this is the key reason") > 0.0);
    assert_eq!(claim_language("never always must every wrong"), 1.0);
}

/// The bar is relative to this recording. An absolute one would nominate
/// everything in an emphatic recording and nothing in a flat one, which is a
/// statement about the microphone.
#[test]
fn quotes_are_the_sentences_that_stand_out_here() {
    let (index, novelty, prosody) = measured();
    let seeds = insight_quote(&index, &novelty, &prosody);
    assert!(!seeds.is_empty());
    assert!(
        seeds.len() < index.sentences.len(),
        "a proposer that nominates everything has nominated nothing"
    );
    for seed in &seeds {
        assert!(seed.score > 0.0 && seed.score <= 1.0);
        assert_eq!(seed.evidence.len(), 1);
    }
}

/// An interjection is not a quote, however novel its one word is.
#[test]
fn a_very_short_sentence_is_not_a_quote() {
    let index = fixture::indexed(
        vec![
            fixture::sentence(0, 0, (0, 2), (0, SECOND), "Absolutely not."),
            fixture::sentence(
                1,
                1,
                (2, 8),
                (2 * SECOND, 6 * SECOND),
                "Pinning the encoder profile is what keeps builds reproducible.",
            ),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        (0, 10 * SECOND),
    );
    let novelty = Novelty::measure(&index);
    let prosody = Prosody::measure(&index, None);
    for seed in insight_quote(&index, &novelty, &prosody) {
        assert_ne!(seed.evidence[0].index, 0, "an interjection was quoted");
    }
}

/// Both proposers that rank sentences use one novelty measure, so a payoff and
/// a quote cannot disagree about which sentence in a topic carries the most.
#[test]
fn novelty_is_one_measure_shared_by_the_proposers_that_need_it() {
    let (index, novelty, _) = measured();
    let best = (0..index.sentences.len())
        .max_by(|left, right| novelty.of(*left).total_cmp(&novelty.of(*right)))
        .expect("a sentence");
    assert!(novelty.of(best) > 0.0);
    assert_eq!(novelty.of(index.sentences.len() + 10), 0.0);
}
