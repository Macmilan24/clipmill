//! What the caption engine published, arriving unchanged in the files a viewer
//! actually gets.
//!
//! Three surfaces are written from one caption track — the burned-in ASS, the
//! SubRip sidecar, and the WebVTT sidecar — and the design rule the book states
//! is that they can differ in how the words arrive and never in which words
//! they are. This test walks a published `captions.cues.v1` document all the
//! way through: projection into the Edit IR, compilation into a render plan,
//! and back out of the two sidecar formats by parsing them as a player would.
//!
//! Parsing the output rather than comparing it to a stored string is the point.
//! A golden file catches a change; parsing catches a change that is *wrong* —
//! a dropped word, a moved break, a timestamp that does not agree between the
//! two formats a player might pick from.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use clipmill_contracts::schemas::captions_cues::CaptionCues;
use clipmill_edit_ir::{EditDocument, Layout, LayoutState, VideoSegment};
use clipmill_render::{
    RenderProfile, SourceInput,
    captions::{Intent, project},
    compile,
};

const SOURCE: &str = "sha256:2222222222222222222222222222222222222222222222222222222222222222";
const FIXTURES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../contracts/fixtures/captions.cues/valid"
);

fn document(name: &str) -> CaptionCues {
    let path = format!("{FIXTURES}/{name}.json");
    let text = std::fs::read_to_string(&path).expect(&path);
    serde_json::from_str(&text).expect("a published caption document")
}

fn source() -> SourceInput {
    SourceInput {
        fingerprint: SOURCE.to_owned(),
        path: "/private/fixtures/source.mp4".to_owned(),
        width: 1_920,
        height: 1_080,
        has_audio: true,
        duration_ticks: 1_350_000,
        keyframe_ticks: vec![0, 180_000, 360_000, 540_000, 900_000],
    }
}

/// An edit whose caption track is the projection of `name` under `intent`.
fn edit(name: &str, intent: Intent) -> EditDocument {
    let cues = document(name);
    let mut edit = EditDocument::default();
    edit.video.segments = vec![VideoSegment {
        segment_id: "seg_1".to_owned(),
        source_fingerprint: SOURCE.to_owned(),
        in_ticks: 0,
        out_ticks: 900_000,
        layout: Layout {
            state: LayoutState::Fit,
            crop_path: Vec::new(),
        },
    }];
    edit.captions = project(&cues, intent, clipmill_captions::DEFAULT_STYLE_REF, 0)
        .expect("the document projects");
    edit
}

/// The cues a SubRip file states, as `(text lines, start, end)`.
fn parse_srt(text: &str) -> Vec<(Vec<String>, String, String)> {
    let mut cues = Vec::new();
    for block in text.split("\n\n") {
        let mut lines = block.lines();
        let Some(_ordinal) = lines.next() else {
            continue;
        };
        let Some(timing) = lines.next() else { continue };
        let Some((start, end)) = timing.split_once(" --> ") else {
            continue;
        };
        let body: Vec<String> = lines.map(str::to_owned).collect();
        if body.is_empty() {
            continue;
        }
        cues.push((body, start.to_owned(), end.to_owned()));
    }
    cues
}

/// The same for WebVTT, whose cues carry the document's own ids.
fn parse_vtt(text: &str) -> Vec<(Vec<String>, String, String)> {
    let mut cues = Vec::new();
    for block in text.split("\n\n").skip(1) {
        let mut lines = block.lines();
        let Some(_id) = lines.next() else { continue };
        let Some(timing) = lines.next() else { continue };
        let Some((start, end)) = timing.split_once(" --> ") else {
            continue;
        };
        let body: Vec<String> = lines.map(str::to_owned).collect();
        if body.is_empty() {
            continue;
        }
        cues.push((body, start.to_owned(), end.to_owned()));
    }
    cues
}

#[test]
fn the_sidecars_carry_the_documents_words_and_its_line_breaks() {
    let cues = document("one_sentence_two_ways");
    let edit = edit("one_sentence_two_ways", Intent::Accessibility);
    let plan = compile(&edit, &[source()], &RenderProfile::default()).expect("a render plan");

    let expected: Vec<Vec<String>> = cues
        .intents
        .accessibility
        .cues
        .iter()
        .map(|cue| {
            cue.lines
                .iter()
                .map(|line| {
                    let first = usize::try_from(line.first_token).unwrap();
                    let count = usize::try_from(line.token_count.get()).unwrap();
                    cues.tokens[first..first + count]
                        .iter()
                        .map(|token| token.text.to_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .collect()
        })
        .collect();

    let srt: Vec<Vec<String>> = parse_srt(&plan.srt).into_iter().map(|cue| cue.0).collect();
    let vtt: Vec<Vec<String>> = parse_vtt(&plan.vtt).into_iter().map(|cue| cue.0).collect();

    assert_eq!(srt, expected, "SubRip lost or moved something");
    assert_eq!(vtt, expected, "WebVTT lost or moved something");
}

#[test]
fn the_two_sidecar_formats_agree_about_every_cue_boundary() {
    // A player may take either. If they disagree, captions appear at different
    // moments depending on which file the platform chose.
    let edit = edit("one_sentence_two_ways", Intent::Accessibility);
    let plan = compile(&edit, &[source()], &RenderProfile::default()).expect("a render plan");

    let srt = parse_srt(&plan.srt);
    let vtt = parse_vtt(&plan.vtt);
    assert_eq!(srt.len(), vtt.len());
    for (from_srt, from_vtt) in srt.iter().zip(&vtt) {
        // SubRip writes a comma before the milliseconds and WebVTT a stop; the
        // instant either names must be the same instant.
        assert_eq!(from_srt.1.replace(',', "."), from_vtt.1);
        assert_eq!(from_srt.2.replace(',', "."), from_vtt.2);
    }
}

#[test]
fn a_sidecar_never_carries_the_burn_ins_markup() {
    let edit = edit("one_sentence_two_ways", Intent::Accessibility);
    let plan = compile(&edit, &[source()], &RenderProfile::default()).expect("a render plan");

    for sidecar in [&plan.srt, &plan.vtt] {
        assert!(!sidecar.contains("\\k"), "a sidecar is the reading profile");
        assert!(
            !sidecar.contains("\\N"),
            "line breaks are real newlines here"
        );
    }
    // And the burned-in track does carry it, or the sweep would not exist.
    assert!(plan.ass.contains("{\\k"));
    assert!(plan.ass.contains("WrapStyle: 2"), "libass must not re-wrap");
}

#[test]
fn the_burn_in_intent_reaches_the_render_as_more_and_shorter_cues() {
    let calm = compile(
        &edit("one_sentence_two_ways", Intent::Accessibility),
        &[source()],
        &RenderProfile::default(),
    )
    .expect("a render plan");
    let hot = compile(
        &edit("one_sentence_two_ways", Intent::BurnIn),
        &[source()],
        &RenderProfile::default(),
    )
    .expect("a render plan");

    assert!(hot.cue_windows.len() > calm.cue_windows.len());
    let words = |windows: &[clipmill_render::CueWindow]| {
        windows
            .iter()
            .flat_map(|window| window.text.split_whitespace().map(str::to_owned))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        words(&hot.cue_windows),
        words(&calm.cue_windows),
        "the two intents must never disagree about the words"
    );
}

#[test]
fn a_document_with_no_cues_renders_a_program_with_no_captions() {
    let edit = edit("nobody_spoke", Intent::Accessibility);
    let plan = compile(&edit, &[source()], &RenderProfile::default()).expect("a render plan");

    assert!(plan.cue_windows.is_empty());
    assert_eq!(plan.srt, "");
    assert_eq!(plan.vtt, "WEBVTT\n\n");
}
