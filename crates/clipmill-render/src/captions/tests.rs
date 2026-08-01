#![allow(clippy::expect_used, clippy::unwrap_used)]

use clipmill_captions::presets::{BOXED, CLEAN, CLEAN_STILL, MINIMAL};
use clipmill_contracts::schemas::captions_cues::CaptionCues;
use clipmill_edit_ir::{CaptionAnimation, CaptionRegion};

use super::{Intent, ProjectionError, project};
use crate::profile::{CaptionStyle, DEFAULT_STYLE_REF};

const FIXTURES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../contracts/fixtures/captions.cues/valid"
);

fn document(name: &str) -> CaptionCues {
    let path = format!("{FIXTURES}/{name}.json");
    let text = std::fs::read_to_string(&path).expect(&path);
    serde_json::from_str(&text).expect("a published caption document")
}

#[test]
fn both_intents_carry_the_same_words_in_a_different_rhythm() {
    // The one divergence this product will not host: a viewer reading the
    // burn-in and a viewer reading the sidecar must be reading the same
    // sentence, however differently it is paced.
    let cues = document("one_sentence_two_ways");

    let calm = project(&cues, Intent::Accessibility, CLEAN, 0).unwrap();
    let hot = project(&cues, Intent::BurnIn, CLEAN, 0).unwrap();

    let words = |track: &clipmill_edit_ir::CaptionTrack| {
        track
            .cues
            .iter()
            .flat_map(|cue| cue.lines.iter())
            .flat_map(|line| line.words.iter())
            .map(|word| word.text.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(words(&calm), words(&hot));
    assert!(hot.cues.len() > calm.cues.len(), "the burn-in runs hotter");
}

#[test]
fn stored_line_breaks_are_copied_and_never_recomputed() {
    let cues = document("one_sentence_two_ways");
    let track = project(&cues, Intent::Accessibility, CLEAN, 0).unwrap();

    let source = &cues.intents.accessibility.cues[0];
    assert_eq!(track.cues[0].lines.len(), source.lines.len());
    for (rendered, stored) in track.cues[0].lines.iter().zip(&source.lines) {
        assert_eq!(rendered.words.len() as u64, stored.token_count.get());
    }
}

#[test]
fn program_time_is_source_time_less_the_clip_offset() {
    let cues = document("one_sentence_two_ways");
    let offset = 30_000;

    let shifted = project(&cues, Intent::Accessibility, CLEAN, offset).unwrap();
    let unshifted = project(&cues, Intent::Accessibility, CLEAN, 0).unwrap();

    let first = &unshifted.cues[0];
    let moved = &shifted.cues[0];
    assert_eq!(moved.end_ticks, first.end_ticks - offset);
    // Clamped at the program's start rather than pushed negative.
    assert!(moved.start_ticks >= 0);
}

#[test]
fn a_cue_whose_words_were_cut_away_is_dropped_rather_than_pinned_to_zero() {
    let cues = document("one_sentence_two_ways");
    let past_everything = 10_000_000;

    let track = project(&cues, Intent::BurnIn, CLEAN, past_everything).unwrap();

    assert!(
        track.cues.is_empty(),
        "a caption pinned to zero would say the wrong thing at the wrong moment"
    );
}

#[test]
fn a_user_correction_outranks_a_re_transcription_of_the_same_word() {
    // The document corrects token 2 by hand and token 4 by re-transcription.
    let cues = document("corrected_by_hand");
    let track = project(&cues, Intent::Accessibility, CLEAN, 0).unwrap();

    let rendered: Vec<String> = track.cues[0].lines[0]
        .words
        .iter()
        .map(|word| word.text.clone())
        .collect();
    assert!(
        rendered.contains(&"Kubernetes".to_owned()),
        "the overlay must be applied: {rendered:?}"
    );
}

#[test]
fn a_still_preset_removes_the_sweep_from_whichever_grouping_it_renders() {
    let cues = document("one_sentence_two_ways");

    let swept = project(&cues, Intent::BurnIn, CLEAN, 0).unwrap();
    let still = project(&cues, Intent::BurnIn, CLEAN_STILL, 0).unwrap();

    assert!(
        swept
            .cues
            .iter()
            .all(|cue| cue.anim == CaptionAnimation::Karaoke)
    );
    assert!(
        still
            .cues
            .iter()
            .all(|cue| cue.anim == CaptionAnimation::None)
    );
    assert_eq!(swept.cues.len(), still.cues.len(), "same words, same cues");
}

#[test]
fn a_style_nothing_defines_is_refused_rather_than_defaulted() {
    let cues = document("one_sentence_two_ways");

    let error = project(
        &cues,
        Intent::Accessibility,
        "clipmill.captions.invented.v1",
        0,
    )
    .unwrap_err();

    assert!(matches!(error, ProjectionError::UnknownStyle(name) if name.contains("invented")));
}

#[test]
fn the_region_the_document_chose_is_the_region_that_renders() {
    let cues = document("one_sentence_two_ways");
    let track = project(&cues, Intent::Accessibility, MINIMAL, 0).unwrap();
    assert!(
        track
            .cues
            .iter()
            .all(|cue| cue.region == CaptionRegion::LowerSafe)
    );
}

#[test]
fn a_document_with_nothing_to_say_projects_to_an_empty_track() {
    let cues = document("nobody_spoke");
    let track = project(&cues, Intent::Accessibility, CLEAN, 0).unwrap();
    assert!(track.cues.is_empty());
    assert_eq!(track.style_ref, CLEAN);
}

#[test]
fn the_renders_default_style_is_a_preset_the_engine_actually_defines() {
    // The fallback inside `default_preset` exists only for a state this test
    // forbids, and something has to forbid it.
    assert!(clipmill_captions::preset(DEFAULT_STYLE_REF).is_some());
    let style = CaptionStyle::default_preset();
    assert_eq!(style.style_ref, DEFAULT_STYLE_REF);
    assert!(!style.boxed);
}

#[test]
fn the_boxed_preset_asks_for_a_plate_and_the_others_do_not() {
    let boxed = CaptionStyle::from_preset(clipmill_captions::preset(BOXED).unwrap());
    let clean = CaptionStyle::from_preset(clipmill_captions::preset(CLEAN).unwrap());
    assert!(boxed.boxed);
    assert!(!clean.boxed);
    // ASS alpha runs the other way, and the plate is deliberately not opaque
    // black: a hard rectangle over video reads as a broken player.
    assert!(boxed.outline.transparency > 0);
    assert_eq!(clean.outline.transparency, 0);
}
