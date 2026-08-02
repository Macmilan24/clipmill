//! The plan against the thing it is a plan for.
//!
//! Every test here compares the preview's answer to the render's own — not to a
//! stored expectation. A golden would catch a change; this catches a
//! *divergence*, which is the only failure the parity rule cares about.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used
)]

use clipmill_edit_ir::{
    CaptionAnimation, CaptionCue, CaptionLine, CaptionRegion, CaptionTrack, CaptionWord,
    CropKeyframe, CropRect, EditDocument, Layout, LayoutState, VideoSegment, VideoTrack,
};

use super::{preview_plan, text_at};
use crate::{RenderProfile, SourceInput, compile, crop_rect_at};

const FRAME_TICKS: i64 = 3_003;
const SOURCE: &str = "sha256:2222222222222222222222222222222222222222222222222222222222222222";

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

fn word(text: &str, from: i64, to: i64) -> CaptionWord {
    CaptionWord {
        text: text.to_owned(),
        start_ticks: from * FRAME_TICKS,
        end_ticks: to * FRAME_TICKS,
    }
}

fn document(state: LayoutState) -> EditDocument {
    EditDocument {
        video: VideoTrack {
            segments: vec![VideoSegment {
                segment_id: "seg_1".to_owned(),
                source_fingerprint: SOURCE.to_owned(),
                in_ticks: 0,
                out_ticks: 120 * FRAME_TICKS,
                layout: Layout {
                    state,
                    crop_path: vec![
                        CropKeyframe {
                            t_ticks: 0,
                            rect: CropRect {
                                x: 200,
                                y: 0,
                                width: 608,
                                height: 1_080,
                            },
                        },
                        CropKeyframe {
                            t_ticks: 90 * FRAME_TICKS,
                            rect: CropRect {
                                x: 800,
                                y: 0,
                                width: 608,
                                height: 1_080,
                            },
                        },
                    ],
                },
            }],
        },
        captions: CaptionTrack {
            style_ref: RenderProfile::default().caption_style.style_ref,
            cues: vec![CaptionCue {
                cue_id: "read_1".to_owned(),
                start_ticks: 10 * FRAME_TICKS,
                end_ticks: 100 * FRAME_TICKS,
                region: CaptionRegion::LowerSafe,
                anim: CaptionAnimation::Karaoke,
                lines: vec![CaptionLine {
                    words: vec![
                        word("the", 10, 40),
                        word("whole", 40, 70),
                        word("point", 70, 95),
                    ],
                }],
            }],
            burn_in: vec![
                CaptionCue {
                    cue_id: "hot_1".to_owned(),
                    start_ticks: 10 * FRAME_TICKS,
                    end_ticks: 55 * FRAME_TICKS,
                    region: CaptionRegion::LowerSafe,
                    anim: CaptionAnimation::Karaoke,
                    lines: vec![CaptionLine {
                        words: vec![word("the", 10, 40), word("whole", 40, 50)],
                    }],
                },
                CaptionCue {
                    cue_id: "hot_2".to_owned(),
                    start_ticks: 60 * FRAME_TICKS,
                    end_ticks: 100 * FRAME_TICKS,
                    region: CaptionRegion::LowerSafe,
                    anim: CaptionAnimation::Karaoke,
                    lines: vec![CaptionLine {
                        words: vec![word("point", 70, 95)],
                    }],
                },
            ],
        },
        ..EditDocument::default()
    }
}

#[test]
fn every_frame_carries_the_crop_the_renderer_would_apply() {
    // The claim in one line: the plan's rectangle and the render's rectangle
    // are the same rectangle, frame by frame, because they came from the same
    // function.
    let edit = document(LayoutState::SpeakerFill);
    let profile = RenderProfile::default();
    let plan = preview_plan(&edit, &profile).expect("a plan");
    let path = &edit.video.segments[0].layout.crop_path;

    assert_eq!(plan.crops.len() as i64, plan.frame_count);
    for frame in 0..plan.frame_count {
        let rendered = crop_rect_at(path, plan.rate, frame).expect("a rect");
        let previewed = plan.crops[frame as usize].expect("a crop");
        assert_eq!(
            (previewed.x, previewed.y, previewed.width, previewed.height),
            (rendered.x, rendered.y, rendered.width, rendered.height),
            "frame {frame} diverged",
        );
    }
}

#[test]
fn a_fitted_segment_says_so_rather_than_cropping_to_everything() {
    let plan =
        preview_plan(&document(LayoutState::Fit), &RenderProfile::default()).expect("a plan");
    assert!(plan.crops.iter().all(Option::is_none));
}

#[test]
fn the_cue_windows_are_the_frames_the_render_gate_measures() {
    let edit = document(LayoutState::Fit);
    let profile = RenderProfile::default();
    let plan = preview_plan(&edit, &profile).expect("a plan");
    let rendered = compile(&edit, &[source()], &profile).expect("a render plan");

    assert_eq!(plan.cues.len(), rendered.cue_windows.len());
    for (previewed, window) in plan.cues.iter().zip(&rendered.cue_windows) {
        assert_eq!(previewed.cue_id, window.cue_id);
        assert_eq!(previewed.first_frame, window.first_frame);
        assert_eq!(previewed.end_frame, window.end_frame);
    }
}

#[test]
fn the_words_a_viewer_reads_are_the_words_the_encoder_burns_in() {
    let edit = document(LayoutState::Fit);
    let profile = RenderProfile::default();
    let plan = preview_plan(&edit, &profile).expect("a plan");
    let rendered = compile(&edit, &[source()], &profile).expect("a render plan");

    for window in &rendered.cue_windows {
        // Sampled inside the window, which is where a viewer would be looking.
        let at = window.first_frame + (window.end_frame - window.first_frame) / 2;
        assert_eq!(
            text_at(&plan, at).as_deref(),
            Some(window.text.as_str()),
            "frame {at} shows different words",
        );
    }
}

#[test]
fn the_sweep_is_the_one_the_burned_in_track_was_written_with() {
    let edit = document(LayoutState::Fit);
    let profile = RenderProfile::default();
    let plan = preview_plan(&edit, &profile).expect("a plan");
    let rendered = compile(&edit, &[source()], &profile).expect("a render plan");

    // Every `\k` the ASS carries, in order.
    let burned: Vec<i64> = rendered
        .ass
        .lines()
        .filter(|line| line.starts_with("Dialogue:"))
        .flat_map(|line| {
            line.split("{\\k")
                .skip(1)
                .filter_map(|chunk| chunk.split('}').next())
                .filter_map(|value| value.parse::<i64>().ok())
                .collect::<Vec<_>>()
        })
        .collect();

    let mut previewed = Vec::new();
    for cue in &plan.cues {
        if cue.lead_in_centis > 0 {
            previewed.push(cue.lead_in_centis);
        }
        for line in &cue.lines {
            for word in &line.words {
                previewed.push(word.hold_centis);
            }
        }
    }

    assert_eq!(previewed, burned, "the highlight would advance differently");
}

#[test]
fn the_gain_curve_is_sampled_to_the_frames_it_will_be_applied_on() {
    let mut edit = document(LayoutState::Fit);
    edit.audio.gain_curve = vec![
        clipmill_edit_ir::GainPoint {
            t_ticks: 0,
            gain_db: 0.0,
        },
        clipmill_edit_ir::GainPoint {
            t_ticks: 30 * FRAME_TICKS,
            gain_db: -6.0,
        },
    ];
    let plan = preview_plan(&edit, &RenderProfile::default()).expect("a plan");

    assert_eq!(plan.gain.len(), 2);
    assert_eq!(plan.gain[1].frame, 30);
    assert!((plan.gain[1].gain_db + 6.0).abs() < f64::EPSILON);
}

#[test]
fn a_document_the_ir_refuses_produces_no_plan_at_all() {
    let mut edit = document(LayoutState::Fit);
    edit.video.segments.clear();
    assert!(preview_plan(&edit, &RenderProfile::default()).is_err());
}

#[test]
fn the_plan_describes_the_burned_in_grouping_and_not_the_reading_one() {
    let plan =
        preview_plan(&document(LayoutState::Fit), &RenderProfile::default()).expect("a plan");
    let ids: Vec<&str> = plan.cues.iter().map(|cue| cue.cue_id.as_str()).collect();
    assert_eq!(ids, vec!["hot_1", "hot_2"], "the player shows the picture");
}
