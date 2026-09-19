#![allow(clippy::unwrap_used)]

use clipmill_edit_ir::{
    CropKeyframe, CropRect, EditCommand, EditDocument, Layout, LayoutState, VideoSegment,
};
use clipmill_render::{RenderProfile, SourceInput, compile, preview_plan};

const FINGERPRINT: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn document(durations: &[i64]) -> EditDocument {
    let mut document = EditDocument::default();
    document.video.transition_ticks = 10_800;
    let mut at = 0;
    document.video.segments = durations
        .iter()
        .enumerate()
        .map(|(index, duration)| {
            let start = at;
            at += duration;
            VideoSegment {
                segment_id: format!("seg_{index}"),
                source_fingerprint: FINGERPRINT.to_owned(),
                in_ticks: start,
                out_ticks: at,
                layout: Layout {
                    state: LayoutState::SpeakerFill,
                    crop_path: vec![CropKeyframe {
                        t_ticks: 0,
                        rect: CropRect {
                            x: 100,
                            y: 0,
                            width: 540,
                            height: 960,
                        },
                    }],
                    secondary_crop_path: Vec::new(),
                },
            }
        })
        .collect();
    document
}

#[test]
fn soft_cuts_use_one_global_frame_allocation_and_bound_short_shots() {
    let document = document(&[46_800, 46_800, 9_000, 18_000]);
    let plan = preview_plan(&document, &RenderProfile::default()).unwrap();
    assert_eq!(plan.transition_ticks, 10_800);
    assert_eq!(plan.transitions.len(), 2);
    let first = &plan.transitions[0];
    assert_eq!(first.incoming_segment_id, "seg_1");
    assert_eq!(
        (first.outgoing_frame, first.first_frame, first.end_frame),
        (15, 16, 20)
    );
    // Three allocated frames cannot accommodate a blend of at least two
    // frames while reserving half of the shot as fully incoming footage.
    assert!(
        plan.transitions
            .iter()
            .all(|transition| transition.incoming_segment_id != "seg_2")
    );
    let last = &plan.transitions[1];
    assert_eq!(last.end_frame - last.first_frame, 3);
    for transition in &plan.transitions {
        let segment = plan
            .segments
            .iter()
            .find(|segment| segment.segment_id == transition.incoming_segment_id)
            .unwrap();
        assert!(transition.end_frame <= segment.end_frame);
        assert_eq!(transition.outgoing_frame + 1, transition.first_frame);
    }
}

#[test]
fn fitted_camera_cuts_stay_sharp_and_off_removes_all_blends() {
    let mut document = document(&[90_000, 90_000, 90_000]);
    document.video.segments[0].layout.state = LayoutState::Fit;
    document.video.segments[1].layout.state = LayoutState::Fit;
    let profile = RenderProfile::default();
    let plan = preview_plan(&document, &profile).unwrap();
    assert_eq!(plan.transitions.len(), 1);
    assert_eq!(plan.transitions[0].incoming_segment_id, "seg_2");
    document.video.transition_ticks = 0;
    assert!(
        preview_plan(&document, &profile)
            .unwrap()
            .transitions
            .is_empty()
    );
}

#[test]
fn trims_keep_the_control_and_recompute_only_surviving_boundaries() {
    let mut document = document(&[90_000, 90_000]);
    let before = document.to_canonical_json().unwrap();
    let undo = EditCommand::RippleDelete {
        start_ticks: 0,
        end_ticks: 90_000,
        reflow_edges: false,
    }
    .apply(&mut document)
    .unwrap();
    assert_eq!(document.video.transition_ticks, 10_800);
    assert!(
        preview_plan(&document, &RenderProfile::default())
            .unwrap()
            .transitions
            .is_empty()
    );
    undo.apply(&mut document).unwrap();
    assert_eq!(document.to_canonical_json().unwrap(), before);
}

#[test]
fn visual_blends_leave_program_audio_captions_and_source_windows_identical() {
    let mut document = document(&[46_800, 46_800]);
    let source = SourceInput {
        fingerprint: FINGERPRINT.to_owned(),
        path: "source.mp4".to_owned(),
        width: 1920,
        height: 1080,
        has_audio: true,
        duration_ticks: 180_000,
        keyframe_ticks: vec![0],
    };
    let profile = RenderProfile::default();
    let enabled = compile(&document, std::slice::from_ref(&source), &profile).unwrap();
    document.video.transition_ticks = 0;
    let disabled = compile(&document, &[source], &profile).unwrap();
    assert_eq!(enabled.frame_count, disabled.frame_count);
    assert_eq!(enabled.duration_ticks, disabled.duration_ticks);
    assert_eq!(enabled.spans, disabled.spans);
    assert_eq!(enabled.measurement_graph, disabled.measurement_graph);
    assert_eq!(enabled.ass, disabled.ass);
    assert_eq!(enabled.srt, disabled.srt);
    assert_eq!(enabled.vtt, disabled.vtt);
    assert!(
        enabled
            .graph
            .graph
            .contains("fade=t=out:start_frame=0:nb_frames=4:alpha=1")
    );
    assert!(!disabled.graph.graph.contains("cut_hold"));
    assert_ne!(enabled.recipe_config(), disabled.recipe_config());
}

#[test]
fn maximum_duration_never_exceeds_a_quarter_second() {
    let mut document = document(&[90_000, 90_000]);
    document.video.transition_ticks = 22_500;
    let profile = RenderProfile::default();
    let plan = preview_plan(&document, &profile).unwrap();
    assert_eq!(
        plan.transitions[0].end_frame - plan.transitions[0].first_frame,
        7
    );
}
