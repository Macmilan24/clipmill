//! The cue sets this engine is expected to produce, and the two rules it may
//! never break.
//!
//! Goldens exist here because captioning fails silently. A segmenter that
//! quietly starts breaking one word earlier produces captions that still look
//! like captions, and nothing in a unit test of the cost function would notice.
//! Writing the cues down means a change to the weights has to be argued for
//! rather than merged.
//!
//! The two rules are checked on top and are not goldens: **zero reading-speed
//! violations in the accessibility grouping**, because that grouping is what
//! every sidecar is written from, and **no cue over a cut in a silence**,
//! because a caption that survives a change of picture reads as a glitch.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use clipmill_captions::{
    CueFacts, Profile, Span, Token, Weights,
    lexicon::{break_after, normalize, orphans_if_last},
    segment, validate,
};

/// A tenth of a second.
const TENTH: i64 = 9_000;

/// Speech written down by hand: each word takes as long as it has letters, at
/// roughly a syllable a tenth, with a beat after a sentence ends.
fn spoken(text: &str) -> Vec<Token> {
    let mut at = 0_i64;
    let mut tokens = Vec::new();
    for word in text.split_whitespace() {
        let letters = i64::try_from(word.chars().count()).unwrap_or(4);
        let length = (letters / 2).max(1);
        let normalized = normalize(word);
        tokens.push(Token {
            text: word.to_owned(),
            start_ticks: at * TENTH,
            end_ticks: (at + length) * TENTH,
            filler: false,
            emphasis: false,
            break_after: break_after(word),
            orphans: orphans_if_last(&normalized),
            normalized,
        });
        at += length + 1;
        if word.ends_with(['.', '?', '!']) {
            at += 3;
        }
    }
    tokens
}

fn span(tokens: &[Token]) -> Span {
    Span {
        start_ticks: 0,
        end_ticks: tokens.last().map_or(TENTH, |last| last.end_ticks) + 30 * TENTH,
    }
}

/// The cues as a reader sees them, one string a line, cues separated by `|`.
fn readable(tokens: &[Token], cues: &[segment::Cue]) -> Vec<String> {
    cues.iter()
        .map(|cue| {
            cue.lines
                .iter()
                .map(|line| {
                    tokens[line.first_token..line.first_token + line.token_count]
                        .iter()
                        .map(|token| token.text.as_str())
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .collect::<Vec<_>>()
                .join(" / ")
        })
        .collect()
}

fn facts<'a>(
    cues: &'a [segment::Cue],
    widths: &'a [Vec<usize>],
    ids: &'a [String],
) -> Vec<CueFacts<'a>> {
    cues.iter()
        .enumerate()
        .map(|(index, cue)| CueFacts {
            cue_id: &ids[index],
            start_ticks: cue.start_ticks,
            end_ticks: cue.end_ticks,
            lines: &widths[index],
        })
        .collect()
}

fn checkable(cues: &[segment::Cue]) -> (Vec<Vec<usize>>, Vec<String>) {
    let widths = cues
        .iter()
        .map(|cue| cue.lines.iter().map(|line| line.characters).collect())
        .collect();
    let ids = (0..cues.len())
        .map(|index| format!("cue_{}", index + 1))
        .collect();
    (widths, ids)
}

const PARAGRAPH: &str = "Captions are the most read typography a creator will ever ship. \
They are rendered sixty times a second over a moving image, and they deserve an engine \
rather than a filter. The break is the craft, and the break is also arithmetic.";

#[test]
fn the_sidecar_grouping_has_no_reading_speed_violations_at_all() {
    // The contract the accessibility export is held to. Not "few", not
    // "acceptable": none.
    let tokens = spoken(PARAGRAPH);
    let at = span(&tokens);
    let profile = Profile::ACCESSIBILITY_EN;

    let cues = segment(&tokens, &[], at, profile, Weights::default()).unwrap();
    let (widths, ids) = checkable(&cues);
    let found = validate(&facts(&cues, &widths, &ids), profile, &[]);

    let reading: Vec<String> = found
        .iter()
        .map(|item| format!("{}: {}", item.cue_id(), item.message()))
        .collect();
    assert!(
        reading.is_empty(),
        "the sidecar profile was not met: {reading:#?}"
    );
}

#[test]
fn no_cue_survives_a_cut_that_falls_in_a_silence() {
    let tokens = spoken(PARAGRAPH);
    let at = span(&tokens);
    let profile = Profile::ACCESSIBILITY_EN;
    // Cuts placed in four of the silences between words.
    let shot_cuts: Vec<i64> = [4_usize, 11, 19, 26]
        .iter()
        .filter_map(|index| {
            let before = tokens.get(*index)?;
            let after = tokens.get(index + 1)?;
            Some(i64::midpoint(before.end_ticks, after.start_ticks))
        })
        .collect();

    let cues = segment(&tokens, &shot_cuts, at, profile, Weights::default()).unwrap();
    let (widths, ids) = checkable(&cues);
    let found = validate(&facts(&cues, &widths, &ids), profile, &shot_cuts);

    let spanning: Vec<String> = found
        .iter()
        .filter(|item| matches!(item, clipmill_captions::Violation::SpansCut { .. }))
        .map(|item| format!("{}: {}", item.cue_id(), item.message()))
        .collect();
    assert!(spanning.is_empty(), "a cue outlived a cut: {spanning:#?}");
}

#[test]
fn the_sidecar_cues_are_the_ones_written_down() {
    let tokens = spoken(PARAGRAPH);
    let at = span(&tokens);
    let cues = segment(
        &tokens,
        &[],
        at,
        Profile::ACCESSIBILITY_EN,
        Weights::default(),
    )
    .unwrap();

    assert_eq!(
        readable(&tokens, &cues),
        vec![
            // Every line break lands at a phrase edge and every cue ends on
            // punctuation, which is what the break-quality term is buying.
            "Captions are the most read / typography a creator will ever ship.",
            "They are rendered sixty times / a second over a moving image,",
            "and they deserve an engine / rather than a filter.",
            "The break is the craft, / and the break is also arithmetic.",
        ]
    );
}

#[test]
fn the_burn_in_cues_are_the_ones_written_down() {
    let tokens = spoken("The break is the craft, and the break is also arithmetic.");
    let at = span(&tokens);
    let cues = segment(&tokens, &[], at, Profile::BURN_IN_EN, Weights::default()).unwrap();

    assert_eq!(
        readable(&tokens, &cues),
        vec![
            "The break",
            "is the craft,",
            "and the break is",
            "also arithmetic."
        ]
    );
}

#[test]
fn both_groupings_hold_every_word_exactly_once() {
    let tokens = spoken(PARAGRAPH);
    let at = span(&tokens);

    for profile in [Profile::ACCESSIBILITY_EN, Profile::BURN_IN_EN] {
        let cues = segment(&tokens, &[], at, profile, Weights::default()).unwrap();
        let mut next = 0_usize;
        for cue in &cues {
            assert_eq!(cue.first_token, next);
            next += cue.token_count;
        }
        assert_eq!(next, tokens.len(), "a word went missing");
    }
}
