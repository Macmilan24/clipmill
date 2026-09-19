//! Actual encoded pixels, not a second model of the emitted filter expression.
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use clipmill_edit_ir::{CropKeyframe, CropRect, EditDocument, Layout, LayoutState, VideoSegment};
use clipmill_render::{
    FrameRateSpec, LoudnessMeasurement, RenderProfile, SourceInput, compile, preview_plan,
};
use std::{
    path::Path,
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

fn ffmpeg(binary: &Path, cwd: &Path, args: &[String]) -> Output {
    let output = Command::new(binary)
        .current_dir(cwd)
        .args(["-hide_banner", "-nostdin", "-y"])
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
#[ignore = "requires pinned FFmpeg; decodes changing footage, blend weights, final frame count and identical audio"]
#[allow(clippy::too_many_lines)]
fn soft_cuts_match_preview_weights_without_shortening_or_retiming_the_clip() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let binary = root.join(".cache/bin/ffmpeg");
    assert!(binary.is_file());
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let work = std::env::temp_dir().join(format!("clipmill-soft-cut-{nonce}"));
    std::fs::create_dir(&work).unwrap();
    ffmpeg(
        &binary,
        &work,
        &args(&[
            "-f",
            "lavfi",
            "-i",
            "nullsrc=s=160x160:r=30:d=2,geq=lum='if(lt(N,30),40+N,180+N-30)':cb='if(lt(N,30),110,145)':cr='if(lt(N,30),140,110)'",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=2",
            "-c:v",
            "ffv1",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "pcm_s16le",
            "source.mkv",
        ]),
    );
    let fingerprint = format!("sha256:{}", "1".repeat(64));
    let path = |x, y, width, height| {
        vec![CropKeyframe {
            t_ticks: 0,
            rect: CropRect {
                x,
                y,
                width,
                height,
            },
        }]
    };
    let mut document = EditDocument::default();
    document.video.segments = vec![
        VideoSegment {
            segment_id: "face".to_owned(),
            source_fingerprint: fingerprint.clone(),
            in_ticks: 0,
            out_ticks: 90_000,
            layout: Layout {
                state: LayoutState::SpeakerFill,
                crop_path: path(0, 0, 90, 160),
                secondary_crop_path: Vec::new(),
            },
        },
        VideoSegment {
            segment_id: "pair".to_owned(),
            source_fingerprint: fingerprint.clone(),
            in_ticks: 90_000,
            out_ticks: 180_000,
            layout: Layout {
                state: LayoutState::TwoUp,
                crop_path: path(0, 0, 90, 80),
                secondary_crop_path: path(70, 80, 90, 80),
            },
        },
    ];
    let source = SourceInput {
        fingerprint,
        path: work.join("source.mkv").to_string_lossy().into_owned(),
        width: 160,
        height: 160,
        has_audio: true,
        duration_ticks: 180_000,
        keyframe_ticks: vec![0],
    };
    for (num, den) in [(30, 1), (30_000, 1_001)] {
        let profile = RenderProfile {
            width: 90,
            height: 160,
            crf: 1,
            preset: "ultrafast".to_owned(),
            frame_rate: FrameRateSpec { num, den },
            ..RenderProfile::default()
        };
        let mut outputs = Vec::new();
        for duration in [0, 10_800] {
            document.video.transition_ticks = duration;
            let plan = compile(&document, std::slice::from_ref(&source), &profile).unwrap();
            let measurement = ffmpeg(&binary, &work, &plan.measurement_args());
            let measurement = LoudnessMeasurement::from_loudnorm_json(&String::from_utf8_lossy(
                &measurement.stderr,
            ))
            .unwrap();
            ffmpeg(&binary, &work, &plan.encode_args(measurement));
            let pixels = ffmpeg(
                &binary,
                &work,
                &args(&[
                    "-i",
                    "clip.mp4",
                    "-vf",
                    "scale=1:1,format=rgb24",
                    "-f",
                    "rawvideo",
                    "-pix_fmt",
                    "rgb24",
                    "-",
                ]),
            )
            .stdout;
            let audio = ffmpeg(
                &binary,
                &work,
                &args(&[
                    "-i",
                    "clip.mp4",
                    "-vn",
                    "-f",
                    "f32le",
                    "-acodec",
                    "pcm_f32le",
                    "-",
                ]),
            )
            .stdout;
            assert_eq!(pixels.len(), usize::try_from(plan.frame_count).unwrap() * 3);
            outputs.push((pixels, audio));
        }
        let preview = preview_plan(&document, &profile).unwrap();
        let transition = &preview.transitions[0];
        let (baseline, base_audio) = &outputs[0];
        let (blended, blend_audio) = &outputs[1];
        assert_eq!(blended.len(), baseline.len());
        assert_eq!(
            base_audio, blend_audio,
            "visual transitions must not alter encoded audio"
        );
        let first = usize::try_from(transition.first_frame).unwrap();
        let end = usize::try_from(transition.end_frame).unwrap();
        let outgoing_at = usize::try_from(transition.outgoing_frame).unwrap() * 3;
        assert!(
            baseline[outgoing_at] > baseline[outgoing_at + 2] + 30
                && baseline[first * 3 + 2] > baseline[first * 3] + 30,
            "fixture must have a real camera-cut contrast"
        );
        for (sample, actual) in blended.iter().enumerate() {
            let frame = sample / 3;
            let channel = sample % 3;
            let outgoing = baseline[outgoing_at + channel];
            let expected = if (first..end).contains(&frame) {
                let count = u32::try_from(end - first).unwrap();
                let remaining = u32::try_from(end - frame).unwrap();
                let alpha = f64::from(remaining) / f64::from(count);
                f64::from(outgoing) * alpha + f64::from(baseline[sample]) * (1.0 - alpha)
            } else {
                f64::from(baseline[sample])
            };
            assert!(
                (f64::from(*actual) - expected).abs() <= 4.0,
                "{num}/{den} frame{frame} channel{channel}: decoded{actual}, expected{expected:.2}, transition{transition:?}"
            );
        }
        // Outgoing footage changes every frame; matching its single final
        // value during the blend proves a hold, not a replay or outside handle.
        assert_eq!(
            &blended[first * 3..first * 3 + 3],
            &baseline[outgoing_at..outgoing_at + 3]
        );
        println!(
            "{num}/{den}: {} frames, RGB matches preview alpha interval{first}..{end}, unchanged audio, live incoming tail",
            blended.len() / 3
        );
    }
    std::fs::remove_dir_all(work).unwrap();
}
