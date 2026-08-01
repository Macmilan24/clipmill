#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::panic,
    clippy::unwrap_used
)]

use super::{Cue, Line, Span, Token, Weights, cue, layout, lowest_start, solve};
use crate::lexicon::{break_after, normalize, orphans_if_last};
use crate::profile::Profile;

/// A tenth of a second, which is roughly one syllable and a convenient unit for
/// writing speech down by hand.
const TENTH: i64 = 9_000;

fn token(text: &str, start: i64, end: i64) -> Token {
    let normalized = normalize(text);
    Token {
        text: text.to_owned(),
        start_ticks: start * TENTH,
        end_ticks: end * TENTH,
        filler: false,
        emphasis: false,
        break_after: break_after(text),
        orphans: orphans_if_last(&normalized),
        normalized,
    }
}

/// A sentence spoken at an even pace, three tenths of a second a word.
fn spoken(text: &str, from: i64) -> Vec<Token> {
    text.split_whitespace()
        .enumerate()
        .map(|(index, word)| {
            let start = from + index as i64 * 3;
            token(word, start, start + 2)
        })
        .collect()
}

fn span(tokens: &[Token]) -> Span {
    Span {
        start_ticks: 0,
        end_ticks: tokens.last().map_or(TENTH, |last| last.end_ticks) + 20 * TENTH,
    }
}

/// Every possible segmentation of `tokens`, scored with the same cue cost the
/// dynamic program uses.
fn brute_force(
    tokens: &[Token],
    shot_cuts: &[i64],
    at: Span,
    profile: Profile,
    weights: Weights,
) -> Option<f64> {
    let count = tokens.len();
    assert!(
        count <= 14,
        "brute force is exponential; keep the run small"
    );
    let mut best: Option<f64> = None;
    for mask in 0_u32..(1 << count.saturating_sub(1)) {
        let mut boundaries = vec![0_usize];
        for position in 1..count {
            if mask & (1 << (position - 1)) != 0 {
                boundaries.push(position);
            }
        }
        boundaries.push(count);

        let mut total = 0.0;
        let mut legal = true;
        for window in boundaries.windows(2) {
            if let Some((cost, _)) = cue(
                tokens, shot_cuts, at, profile, weights, window[0], window[1],
            ) {
                total += cost;
            } else {
                legal = false;
                break;
            }
        }
        if legal && best.is_none_or(|current| total < current) {
            best = Some(total);
        }
    }
    best
}

#[test]
fn the_dynamic_program_finds_the_same_optimum_brute_force_does() {
    // The claim the whole module rests on. A greedy segmenter agrees with this
    // on most inputs, which is exactly why the disagreement has to be measured
    // rather than eyeballed.
    let profile = Profile::ACCESSIBILITY_EN;
    let weights = Weights::default();
    for text in [
        "the quick brown fox jumps over the lazy dog today",
        "we shipped it, and then everything broke at once.",
        "a very long word segmentation problem needs an exact answer here",
        "yes. no. maybe. it depends on what you mean by that.",
    ] {
        let tokens = spoken(text, 0);
        let at = span(&tokens);
        let (_, dynamic) = solve(&tokens, &[], at, profile, weights).unwrap();
        let exhaustive = brute_force(&tokens, &[], at, profile, weights).unwrap();
        assert!(
            (dynamic - exhaustive).abs() < 1e-9,
            "dynamic {dynamic} vs exhaustive {exhaustive} for {text:?}"
        );
    }
}

#[test]
fn the_optimum_is_still_the_optimum_when_a_cut_forbids_some_of_it() {
    let profile = Profile::ACCESSIBILITY_EN;
    let weights = Weights::default();
    let tokens = spoken("we shipped it and then everything broke at once", 0);
    let at = span(&tokens);
    // In the silence after "then", which no cue may span.
    let shot_cuts = vec![tokens[3].end_ticks + TENTH / 2];

    let (cues, dynamic) = solve(&tokens, &shot_cuts, at, profile, weights).unwrap();
    let exhaustive = brute_force(&tokens, &shot_cuts, at, profile, weights).unwrap();

    assert!((dynamic - exhaustive).abs() < 1e-9);
    assert!(
        cues.iter()
            .any(|item| item.first_token + item.token_count == 4),
        "a cue must end at the cut: {cues:?}"
    );
}

#[test]
fn no_cue_spans_a_cut_that_falls_in_a_silence() {
    let profile = Profile::ACCESSIBILITY_EN;
    let tokens = spoken("one two three four five six seven eight", 0);
    let at = span(&tokens);
    let shot_cuts: Vec<i64> = (0..4)
        .map(|index| tokens[index * 2].end_ticks + TENTH / 2)
        .collect();

    let (cues, _) = solve(&tokens, &shot_cuts, at, profile, Weights::default()).unwrap();

    for item in &cues {
        let inside = shot_cuts.iter().filter(|cut| {
            **cut > tokens[item.first_token].start_ticks
                && **cut < tokens[item.first_token + item.token_count - 1].end_ticks
        });
        assert_eq!(inside.count(), 0, "cue {item:?} spans a cut");
    }
}

#[test]
fn a_line_never_exceeds_the_character_ceiling() {
    let profile = Profile::ACCESSIBILITY_EN;
    let tokens = spoken(
        "captioning is the most read typography a creator will ever ship and it \
         deserves an engine rather than a filter applied at the end",
        0,
    );
    let at = span(&tokens);

    let (cues, _) = solve(&tokens, &[], at, profile, Weights::default()).unwrap();

    assert!(!cues.is_empty());
    for item in &cues {
        assert!(item.lines.len() <= profile.max_lines);
        for line in &item.lines {
            assert!(
                line.characters <= profile.max_line_characters,
                "line of {} characters: {item:?}",
                line.characters
            );
        }
    }
}

#[test]
fn a_line_break_does_not_orphan_an_article_when_it_has_a_choice() {
    // "of" ends the top line only if every alternative is worse; with a run
    // this even, one exists.
    let profile = Profile::ACCESSIBILITY_EN;
    let run: Vec<Token> = spoken("the whole point of the exercise was speed", 0);
    let (lines, _) = layout(&run, 0, profile, Weights::default()).unwrap();

    if lines.len() == 2 {
        let last = &run[lines[0].token_count - 1];
        assert!(!last.orphans, "top line ends on {:?}", last.text);
    }
}

#[test]
fn a_cue_is_held_long_enough_to_read_when_there_is_room_to_hold_it() {
    let profile = Profile::ACCESSIBILITY_EN;
    // Four quick words, then a long silence: the cue may stay up.
    let tokens = vec![
        token("that", 0, 1),
        token("is", 1, 2),
        token("the", 2, 3),
        token("point.", 3, 4),
    ];
    let at = Span {
        start_ticks: 0,
        end_ticks: 200 * TENTH,
    };

    let (cues, _) = solve(&tokens, &[], at, profile, Weights::default()).unwrap();

    assert_eq!(cues.len(), 1);
    let held = cues[0].end_ticks - cues[0].start_ticks;
    assert!(
        held >= profile.min_duration_ticks,
        "held for {held} ticks, floor is {}",
        profile.min_duration_ticks
    );
    assert!(cues[0].reading_rate_cps <= profile.reading_rate_cps);
}

#[test]
fn a_cue_never_disappears_while_its_last_word_is_still_being_said() {
    let profile = Profile::ACCESSIBILITY_EN;
    // A long word followed immediately by the next cue's first word.
    let tokens = vec![
        token("extraordinarily", 0, 20),
        token("fast", 20, 21),
        token("now", 21, 22),
    ];
    let at = span(&tokens);

    let (cues, _) = solve(&tokens, &[], at, profile, Weights::default()).unwrap();

    for item in &cues {
        let last = &tokens[item.first_token + item.token_count - 1];
        assert!(
            item.end_ticks >= last.end_ticks,
            "cue ended at {} but {:?} runs to {}",
            item.end_ticks,
            last.text,
            last.end_ticks
        );
    }
}

#[test]
fn the_same_words_segment_the_same_way_twice() {
    let profile = Profile::ACCESSIBILITY_EN;
    let tokens = spoken(
        "determinism is not incidental here it is the whole point",
        0,
    );
    let at = span(&tokens);

    let first = solve(&tokens, &[], at, profile, Weights::default()).unwrap();
    let second = solve(&tokens, &[], at, profile, Weights::default()).unwrap();

    assert_eq!(first.0, second.0);
    assert!((first.1 - second.1).abs() < f64::EPSILON);
}

#[test]
fn the_burn_in_grouping_runs_hotter_than_the_one_the_sidecar_gets() {
    // The property that matters, and the one that does not depend on how long
    // the words happen to be: both intents hold every word, and the kinetic one
    // gets through them in more, smaller, single-line cues.
    let tokens = spoken("this is what a kinetic caption actually looks like", 0);
    let at = span(&tokens);

    let (hot, _) = solve(&tokens, &[], at, Profile::BURN_IN_EN, Weights::default()).unwrap();
    let (calm, _) = solve(
        &tokens,
        &[],
        at,
        Profile::ACCESSIBILITY_EN,
        Weights::default(),
    )
    .unwrap();

    assert!(
        hot.len() > calm.len(),
        "burn-in produced {} cues and the sidecar {}",
        hot.len(),
        calm.len()
    );
    for item in &hot {
        assert_eq!(item.lines.len(), 1, "a kinetic cue is one line: {item:?}");
        assert!(item.characters <= Profile::BURN_IN_EN.max_line_characters);
    }
    let words = |cues: &[Cue]| cues.iter().map(|item| item.token_count).sum::<usize>();
    assert_eq!(words(&hot), tokens.len());
    assert_eq!(words(&calm), tokens.len());
}

#[test]
fn a_word_wider_than_a_line_still_gets_a_cue() {
    let profile = Profile::ACCESSIBILITY_EN;
    let tokens = vec![token(&"x".repeat(60), 0, 10), token("after.", 10, 12)];
    let at = span(&tokens);

    let (cues, _) = solve(&tokens, &[], at, profile, Weights::default()).unwrap();

    assert!(!cues.is_empty(), "a long word must not make a run unusable");
    assert_eq!(cues[0].token_count, 1);
}

#[test]
fn a_third_line_is_refused_rather_than_approximated() {
    let mut profile = Profile::ACCESSIBILITY_EN;
    profile.max_lines = 3;
    let tokens = spoken("one two three", 0);

    let error = solve(&tokens, &[], span(&tokens), profile, Weights::default()).unwrap_err();

    assert_eq!(error, super::SegmentError::TooManyLines(3));
}

#[test]
fn no_words_is_no_cues_rather_than_an_error() {
    let at = Span {
        start_ticks: 0,
        end_ticks: 100 * TENTH,
    };
    let (cues, cost) = solve(&[], &[], at, Profile::ACCESSIBILITY_EN, Weights::default()).unwrap();
    assert!(cues.is_empty());
    assert!(cost.abs() < f64::EPSILON);
}

#[test]
fn the_candidate_window_always_admits_a_single_token() {
    let tokens = vec![token(&"y".repeat(200), 0, 5), token("next", 5, 6)];
    assert_eq!(lowest_start(&tokens, 1, Profile::ACCESSIBILITY_EN), 0);
    assert_eq!(lowest_start(&tokens, 2, Profile::ACCESSIBILITY_EN), 1);
}

#[test]
fn every_token_lands_in_exactly_one_cue_and_one_line() {
    let profile = Profile::ACCESSIBILITY_EN;
    let tokens = spoken(
        "a caption track that lost a word would be worse than one that never ran",
        0,
    );
    let at = span(&tokens);

    let (cues, _) = solve(&tokens, &[], at, profile, Weights::default()).unwrap();

    let mut seen = 0_usize;
    for item in &cues {
        assert_eq!(item.first_token, seen, "cues must be contiguous: {cues:?}");
        let in_lines: usize = item.lines.iter().map(|line| line.token_count).sum();
        assert_eq!(in_lines, item.token_count, "lines must hold the whole cue");
        let mut at_token = item.first_token;
        for line in &item.lines {
            assert_eq!(line.first_token, at_token);
            at_token += line.token_count;
        }
        seen += item.token_count;
    }
    assert_eq!(seen, tokens.len(), "every word must be captioned");
}

#[test]
fn a_line_is_measured_the_way_it_renders() {
    let run = spoken("two words", 0);
    let (lines, _) = layout(&run, 7, Profile::ACCESSIBILITY_EN, Weights::default()).unwrap();
    assert_eq!(
        lines,
        vec![Line {
            first_token: 7,
            token_count: 2,
            characters: "two words".chars().count(),
        }]
    );
}

#[test]
fn a_cue_reports_the_rate_it_was_actually_held_at() {
    let profile = Profile::ACCESSIBILITY_EN;
    let tokens = spoken("short and sweet.", 0);
    let at = span(&tokens);
    let (cost, built) = cue(&tokens, &[], at, profile, Weights::default(), 0, 3).unwrap();

    let seconds = (built.end_ticks - built.start_ticks) as f64 / 90_000.0;
    let expected = built.characters as f64 / seconds;
    assert!((built.reading_rate_cps - expected).abs() < 1e-9);
    assert!(cost.is_finite());
    assert_eq!(
        built,
        Cue {
            first_token: 0,
            token_count: 3,
            ..built.clone()
        }
    );
}
