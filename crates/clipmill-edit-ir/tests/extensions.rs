//! An extension adds only new source speech and preserves authored words.
#![allow(clippy::expect_used)]

use clipmill_edit_ir::{
    CaptionAnimation, CaptionCue, CaptionLine, CaptionRegion, CaptionWord, CropKeyframe, CropRect,
    EditCommand, EditDocument, GainPoint, Layout, LayoutState, VideoSegment,
};

const SOURCE: &str = "sha256:1111111111111111111111111111111111111111111111111111111111111111";

fn cue(id: &str, word_id: &str, text: &str, start: i64, end: i64) -> CaptionCue {
    CaptionCue {
        cue_id: id.to_owned(),
        start_ticks: start,
        end_ticks: end,
        region: CaptionRegion::UpperSafe,
        anim: CaptionAnimation::Karaoke,
        lines: vec![CaptionLine {
            words: vec![CaptionWord {
                text: text.to_owned(),
                start_ticks: start,
                end_ticks: end,
                word_id: Some(word_id.to_owned()),
            }],
        }],
    }
}

fn document() -> EditDocument {
    let mut document = EditDocument::default();
    document.video.segments = vec![VideoSegment {
        segment_id: "section".to_owned(),
        source_fingerprint: SOURCE.to_owned(),
        in_ticks: 90_000,
        out_ticks: 180_000,
        layout: Layout {
            state: LayoutState::SpeakerFill,
            crop_path: vec![CropKeyframe {
                t_ticks: 0,
                rect: CropRect {
                    x: 100,
                    y: 0,
                    width: 608,
                    height: 1080,
                },
                easing: clipmill_edit_ir::CropEasing::Linear,
            }],
            secondary_crop_path: Vec::new(),
        },
    }];
    document.captions.cues = vec![cue("authored", "source-1", "Corrected", 5_000, 50_000)];
    document.captions.burn_in = vec![cue("burned", "source-1", "Corrected", 5_000, 50_000)];
    document.audio.gain_curve = vec![GainPoint {
        t_ticks: 15_000,
        gain_db: 2.0,
    }];
    document.validate().expect("valid starting document");
    document
}

#[test]
fn extending_the_head_shifts_authored_content_without_rederiving_it() {
    let original = document();
    let mut edited = original.clone();
    let added = cue("new", "source-0", "Earlier", 2_000, 32_000);
    let inverse = EditCommand::ExtendSegment {
        segment_id: "section".to_owned(),
        in_ticks: 45_000,
        out_ticks: 180_000,
        reading_cues: vec![added.clone()],
        burn_in_cues: vec![added],
    }
    .apply(&mut edited)
    .expect("extend head");
    assert_eq!(edited.video.segments[0].in_ticks, 45_000);
    assert_eq!(
        edited.captions.cues[0]
            .words()
            .next()
            .expect("new word")
            .text,
        "Earlier"
    );
    let corrected = &edited.captions.cues[1];
    assert_eq!(
        corrected.words().next().expect("old word").text,
        "Corrected"
    );
    assert_eq!(corrected.start_ticks, 50_000);
    assert_eq!(edited.audio.gain_curve[0].t_ticks, 60_000);
    assert_eq!(edited.video.segments[0].layout.crop_path[1].t_ticks, 45_000);
    inverse.apply(&mut edited).expect("undo");
    assert_eq!(edited, original);
}

#[test]
fn extending_the_tail_appends_cues_and_undoes_exactly() {
    let original = document();
    let mut edited = original.clone();
    let added = cue("new", "source-2", "Later", 3_000, 28_000);
    let inverse = EditCommand::ExtendSegment {
        segment_id: "section".to_owned(),
        in_ticks: 90_000,
        out_ticks: 225_000,
        reading_cues: vec![added.clone()],
        burn_in_cues: vec![added],
    }
    .apply(&mut edited)
    .expect("extend tail");
    assert_eq!(
        edited.captions.cues[0]
            .words()
            .next()
            .expect("old word")
            .text,
        "Corrected"
    );
    assert_eq!(edited.captions.cues[1].start_ticks, 93_000);
    assert_eq!(edited.captions.cues[1].region, CaptionRegion::UpperSafe);
    inverse.apply(&mut edited).expect("undo");
    assert_eq!(edited, original);
}

#[test]
fn an_extension_cannot_replace_an_existing_window() {
    let original = document();
    let mut edited = original.clone();
    assert!(
        EditCommand::ExtendSegment {
            segment_id: "section".to_owned(),
            in_ticks: 100_000,
            out_ticks: 225_000,
            reading_cues: Vec::new(),
            burn_in_cues: Vec::new(),
        }
        .apply(&mut edited)
        .is_err()
    );
    assert_eq!(edited, original);
}
