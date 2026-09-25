//! Exercise adjacent source sections through the pinned FFmpeg, not just a
//! string comparison of the generated filter graph.
#![allow(clippy::expect_used, clippy::panic)]

use std::{path::PathBuf, process::Command};

use clipmill_edit_ir::{EditDocument, Layout, LayoutState, VideoSegment};
use clipmill_render::{LoudnessMeasurement, RenderProfile, SourceInput, compile};

const FINGERPRINT: &str = "sha256:2222222222222222222222222222222222222222222222222222222222222222";

#[test]
#[ignore = "requires the pinned FFmpeg sidecar"]
fn adjacent_sections_encode_with_one_decoder() {
    let ffmpeg = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.cache/bin/ffmpeg");
    let temp = tempfile::TempDir::new().expect("tempdir");
    let source = temp.path().join("source.mp4");
    let generated = Command::new(&ffmpeg)
        .args([
            "-y",
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=320x180:rate=30",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000",
            "-t",
            "2",
            "-c:v",
            "libx264",
            "-threads:v",
            "4",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
        ])
        .arg(&source)
        .output()
        .expect("generate fixture");
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );

    let mut document = EditDocument::default();
    document.video.segments = [("first", 0, 90_000), ("second", 90_000, 180_000)]
        .into_iter()
        .map(|(id, start, end)| VideoSegment {
            segment_id: id.to_owned(),
            source_fingerprint: FINGERPRINT.to_owned(),
            in_ticks: start,
            out_ticks: end,
            layout: Layout {
                state: LayoutState::Fit,
                crop_path: Vec::new(),
                secondary_crop_path: Vec::new(),
            },
        })
        .collect();
    document.captions.style_ref = RenderProfile::default().caption_style.style_ref;
    let mut profile = RenderProfile::default();
    profile.width = 270;
    profile.height = 480;
    profile.frame_rate.num = 30;
    profile.frame_rate.den = 1;
    let input = SourceInput {
        fingerprint: FINGERPRINT.to_owned(),
        path: source.to_string_lossy().into_owned(),
        width: 320,
        height: 180,
        has_audio: true,
        duration_ticks: 180_000,
        keyframe_ticks: vec![0],
    };
    let plan = compile(&document, &[input], &profile).expect("plan");
    let args = plan.encode_args(LoudnessMeasurement {
        input_lufs: -20.0,
        input_true_peak_dbtp: -4.0,
        input_range_lu: 7.0,
        input_threshold_lufs: -30.0,
        target_offset_lu: 0.0,
    });
    assert_eq!(args.iter().filter(|arg| *arg == "-i").count(), 1);
    let output = Command::new(&ffmpeg)
        .current_dir(temp.path())
        .arg("-y")
        .args(&args)
        .output()
        .expect("encode");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        temp.path()
            .join("clip.mp4")
            .metadata()
            .expect("output")
            .len()
            > 1_000
    );
}
