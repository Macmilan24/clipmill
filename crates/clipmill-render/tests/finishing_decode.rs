//! Pixel and loudness verification against the actual encoder, kept separate
//! from pure compiler tests so normal checks do not require binary tools.
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use clipmill_edit_ir::{CropKeyframe, CropRect, EditDocument, Layout, LayoutState, VideoSegment};
use clipmill_render::{LoudnessMeasurement, RenderProfile, SourceInput, compile};
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
        .expect("start pinned FFmpeg");
    assert!(
        output.status.success(),
        "FFmpeg failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "one explicit end-to-end encoder scenario"
)]
#[ignore = "requires the pinned .cache/bin/ffmpeg; renders and decodes actual pixels and audio"]
fn two_up_decodes_both_people_and_meets_existing_audio_targets() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository");
    let binary = root.join(".cache/bin/ffmpeg");
    assert!(
        binary.is_file(),
        "fetch pinned FFmpeg before this explicit gate"
    );
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let work = std::env::temp_dir().join(format!("clipmill-finishing-{nonce}"));
    std::fs::create_dir(&work).expect("scratch directory");
    ffmpeg(
        &binary,
        &work,
        &args(&[
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=1920x1080:r=30:d=4,drawbox=x=960:y=0:w=960:h=1080:color=blue:t=fill",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=4",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-threads",
            "1",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "source.mp4",
        ]),
    );
    let fingerprint = format!("sha256:{}", "1".repeat(64));
    let crop = |x| {
        vec![CropKeyframe {
            t_ticks: 0,
            rect: CropRect {
                x,
                y: 140,
                width: 900,
                height: 800,
            },
            easing: clipmill_edit_ir::CropEasing::Linear,
        }]
    };
    let mut document = EditDocument::default();
    document.video.segments = vec![VideoSegment {
        segment_id: "two-portraits".to_owned(),
        source_fingerprint: fingerprint.clone(),
        in_ticks: 0,
        out_ticks: 4 * 90_000,
        layout: Layout {
            state: LayoutState::TwoUp,
            crop_path: crop(0),
            secondary_crop_path: crop(1000),
            ..Layout::default()
        },
    }];
    let source = SourceInput {
        fingerprint,
        path: work.join("source.mp4").to_string_lossy().into_owned(),
        width: 1920,
        height: 1080,
        has_audio: true,
        duration_ticks: 4 * 90_000,
        keyframe_ticks: vec![0],
    };
    let plan = compile(&document, &[source], &RenderProfile::default()).expect("compile");
    let measured = ffmpeg(&binary, &work, &plan.measurement_args());
    let measurement =
        LoudnessMeasurement::from_loudnorm_json(&String::from_utf8_lossy(&measured.stderr))
            .expect("measured loudness");
    ffmpeg(&binary, &work, &plan.encode_args(measurement));
    let decoded = ffmpeg(
        &binary,
        &work,
        &args(&[
            "-ss",
            "1",
            "-i",
            "clip.mp4",
            "-frames:v",
            "1",
            "-vf",
            "scale=2:4",
            "-pix_fmt",
            "rgb24",
            "-f",
            "rawvideo",
            "pipe:1",
        ]),
    );
    assert_eq!(decoded.stdout.len(), 24);
    let upper = &decoded.stdout[0..3];
    let lower = &decoded.stdout[18..21];
    assert!(
        upper[0] > 180 && upper[2] < 70,
        "upper viewport must show the red source person: {upper:?}"
    );
    assert!(
        lower[2] > 180 && lower[0] < 70,
        "lower viewport must show the blue source person: {lower:?}"
    );
    let audio = ffmpeg(
        &binary,
        &work,
        &args(&[
            "-i",
            "clip.mp4",
            "-vn",
            "-af",
            "loudnorm=I=-14:TP=-1:LRA=11:print_format=json",
            "-f",
            "null",
            "-",
        ]),
    );
    let mastered = LoudnessMeasurement::from_loudnorm_json(&String::from_utf8_lossy(&audio.stderr))
        .expect("decode mastered audio");
    assert!(
        (mastered.input_lufs + 14.0).abs() < 0.6,
        "integrated loudness {}",
        mastered.input_lufs
    );
    assert!(
        mastered.input_true_peak_dbtp <= -0.8,
        "true peak {}",
        mastered.input_true_peak_dbtp
    );
    eprintln!(
        "Decoded two-up and measured audio: {} LUFS, {} dBTP; artifact {}",
        mastered.input_lufs,
        mastered.input_true_peak_dbtp,
        work.join("clip.mp4").display()
    );
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "actual many-shot timestamp, caption and audio falsification gate"
)]
#[ignore = "requires pinned FFmpeg and font; checks 25fps source against fractional output rate"]
fn many_shots_keep_the_tail_caption_and_audio_on_one_program_clock() {
    use clipmill_edit_ir::{CaptionAnimation, CaptionCue, CaptionLine, CaptionRegion, CaptionWord};
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("root");
    let binary = root.join(".cache/bin/ffmpeg");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let work = std::env::temp_dir().join(format!("clipmill-shot-clock-{nonce}"));
    std::fs::create_dir_all(work.join("fonts")).expect("scratch");
    std::fs::copy(
        root.join(".cache/fonts/Inter-Bold.ttf"),
        work.join("fonts/Inter-Bold.ttf"),
    )
    .expect("pinned font");
    // Luma encodes the original 25fps frame number. A pulse near the tail makes
    // inserted audio padding measurable instead of trusting duration metadata.
    ffmpeg(
        &binary,
        &work,
        &args(&[
            "-f",
            "lavfi",
            "-i",
            "nullsrc=s=320x180:r=25:d=10.4,geq=lum='16+N*0.6':cb=128:cr=128",
            "-f",
            "lavfi",
            "-i",
            "aevalsrc=if(between(t\\,9.9\\,10.1)\\,0.5*sin(2*PI*1000*t)\\,0):s=48000:d=10.4",
            "-c:v",
            "libx264",
            "-crf",
            "0",
            "-preset",
            "ultrafast",
            "-threads",
            "1",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "source.mp4",
        ]),
    );
    let fingerprint = format!("sha256:{}", "2".repeat(64));
    let mut document = EditDocument::default();
    document.video.segments = (0..20)
        .map(|index| VideoSegment {
            segment_id: format!("shot-{index}"),
            source_fingerprint: fingerprint.clone(),
            in_ticks: index * 46_800,
            out_ticks: (index + 1) * 46_800,
            layout: Layout {
                state: LayoutState::SpeakerFill,
                crop_path: vec![CropKeyframe {
                    t_ticks: 0,
                    rect: CropRect {
                        x: 0,
                        y: 0,
                        width: 90,
                        height: 160,
                    },
                    easing: clipmill_edit_ir::CropEasing::Linear,
                }],
                secondary_crop_path: Vec::new(),
                ..Layout::default()
            },
        })
        .collect();
    document.captions.cues = vec![CaptionCue {
        cue_id: "last-beat".to_owned(),
        start_ticks: 891_000,
        end_ticks: 927_000,
        region: CaptionRegion::LowerSafe,
        anim: CaptionAnimation::None,
        lines: vec![CaptionLine {
            words: vec![CaptionWord {
                text: "FINAL".to_owned(),
                start_ticks: 891_000,
                end_ticks: 927_000,
                word_id: None,
                emphasis: false,
            }],
        }],
        position: None,
    }];
    let source = SourceInput {
        fingerprint,
        path: work.join("source.mp4").to_string_lossy().into_owned(),
        width: 320,
        height: 180,
        has_audio: true,
        duration_ticks: 936_000,
        keyframe_ticks: vec![0],
    };
    let mut profile = RenderProfile {
        width: 180,
        height: 320,
        ..RenderProfile::default()
    };
    document.captions.style_ref = profile.caption_style.style_ref.clone();
    profile.caption_style.font_size = 14;
    profile.caption_style.margin_horizontal = 15;
    profile.caption_style.margin_vertical = 43;
    profile.caption_style.outline_width = 1;
    profile.caption_style.spoken = clipmill_render::Colour::opaque(255, 255, 255);
    let plan = compile(&document, &[source], &profile).expect("compile");
    assert_eq!(plan.frame_count, 312);
    assert_eq!(plan.spans.iter().map(|s| s.frame_count).sum::<i64>(), 312);
    let preview = clipmill_render::preview_plan(&document, &profile).expect("preview");
    assert_eq!(preview.segments.last().unwrap().end_frame, 312);
    std::fs::write(work.join("clip.ass"), &plan.ass).expect("captions");
    let first = ffmpeg(&binary, &work, &plan.measurement_args());
    let measurement =
        LoudnessMeasurement::from_loudnorm_json(&String::from_utf8_lossy(&first.stderr))
            .expect("measurement");
    ffmpeg(&binary, &work, &plan.encode_args(measurement));
    let pixels = ffmpeg(
        &binary,
        &work,
        &args(&[
            "-i",
            "clip.mp4",
            "-an",
            "-vf",
            "crop=90:80:0:0,scale=1:1,format=gray",
            "-f",
            "rawvideo",
            "pipe:1",
        ]),
    );
    assert_eq!(pixels.stdout.len(), 312, "every allocated frame decodes");
    let last = *pixels.stdout.last().unwrap();
    assert!(
        (178..=183).contains(&last),
        "tail must contain original frame259, not an earlier shot: {last}"
    );
    let audio = ffmpeg(
        &binary,
        &work,
        &args(&[
            "-i", "clip.mp4", "-vn", "-ac", "1", "-ar", "48000", "-f", "f32le", "pipe:1",
        ]),
    );
    let onset = audio
        .stdout
        .chunks_exact(4)
        .position(|sample| f32::from_le_bytes(sample.try_into().unwrap()).abs() > 0.02)
        .expect("tail pulse remains audible");
    #[allow(clippy::cast_precision_loss, reason = "ten-second fixture")]
    let onset_seconds = onset as f64 / 48_000.0;
    assert!(
        (onset_seconds - 9.9).abs() < 0.04,
        "audio acquired padding at shot cuts: pulse={onset_seconds}"
    );
    for (at, has_caption) in [("9.8", false), ("9.95", true)] {
        let pixels = ffmpeg(
            &binary,
            &work,
            &args(&[
                "-ss",
                at,
                "-i",
                "clip.mp4",
                "-frames:v",
                "1",
                "-vf",
                "crop=180:80:0:220,format=gray",
                "-f",
                "rawvideo",
                "pipe:1",
            ]),
        );
        assert_eq!(
            pixels.stdout.iter().any(|pixel| *pixel > 235),
            has_caption,
            "decoded caption timing at {at}s"
        );
    }
    eprintln!(
        "25fps/20-shot gate: 312 decoded frames, tail luma={last}, audio pulse={onset_seconds:.4}s, caption on time; {}",
        work.join("clip.mp4").display()
    );
}

fn gate_workspace(name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("root");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let work = std::env::temp_dir().join(format!("clipmill-{name}-{nonce}"));
    std::fs::create_dir_all(&work).expect("scratch");
    (root, work)
}

fn encode_fixture(
    binary: &Path,
    work: &Path,
    document: &EditDocument,
    source: SourceInput,
) -> clipmill_render::RenderPlan {
    let profile = RenderProfile {
        width: 90,
        height: 160,
        ..RenderProfile::default()
    };
    let plan = compile(document, &[source], &profile).expect("fixture compilation");
    let measured = ffmpeg(binary, work, &plan.measurement_args());
    let measurement =
        LoudnessMeasurement::from_loudnorm_json(&String::from_utf8_lossy(&measured.stderr))
            .expect("loudness measurement");
    ffmpeg(binary, work, &plan.encode_args(measurement));
    plan
}

fn probe_json(binary: &Path, file: &Path, show: &[&str]) -> serde_json::Value {
    let result = Command::new(binary)
        .args(["-v", "error", "-of", "json"])
        .args(show)
        .arg(file)
        .output()
        .expect("probe");
    assert!(result.status.success());
    serde_json::from_slice(&result.stdout).expect("probe JSON")
}

#[test]
#[ignore = "requires pinned FFmpeg; exercises CFR/VFR timestamps against decoded luma"]
#[allow(
    clippy::too_many_lines,
    reason = "bounded source-rate matrix with decoded source-time assertions"
)]
fn supported_source_rates_and_vfr_keep_source_time_through_shot_cuts() {
    let (root, work) = gate_workspace("source-rates");
    let binary = root.join(".cache/bin/ffmpeg");
    let probe = root.join(".cache/bin/ffprobe");
    for (name, rate, vfr) in [
        ("25", "25", false),
        ("2997", "30000/1001", false),
        ("30", "30", false),
        ("60", "60", false),
        ("vfr", "60", true),
    ] {
        let case = work.join(name);
        std::fs::create_dir(&case).expect("case");
        // Gray level records presentation TIME, not input frame ordinal. An
        // incorrectly normalized VFR source therefore visibly drifts.
        let filter = format!(
            "nullsrc=s=160x180:r={rate}:d=1.5,geq=lum='16+100*T':cb=128:cr=128{}",
            if vfr {
                ",select='if(lt(t,0.5),not(mod(n,2)),not(mod(n,3)))'"
            } else {
                ""
            }
        );
        ffmpeg(
            &binary,
            &case,
            &args(&[
                "-f",
                "lavfi",
                "-i",
                &filter,
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:sample_rate=48000:duration=1.5",
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast",
                "-crf",
                "0",
                "-threads",
                "1",
                "-fps_mode",
                "vfr",
                "-c:a",
                "aac",
                "source.mp4",
            ]),
        );
        if vfr {
            let metadata = probe_json(
                &probe,
                &case.join("source.mp4"),
                &[
                    "-select_streams",
                    "v:0",
                    "-show_frames",
                    "-show_entries",
                    "frame=pts_time",
                ],
            );
            let times: Vec<f64> = metadata["frames"]
                .as_array()
                .unwrap()
                .iter()
                .map(|frame| frame["pts_time"].as_str().unwrap().parse().unwrap())
                .collect();
            let deltas: Vec<f64> = times.windows(2).map(|pair| pair[1] - pair[0]).collect();
            assert!(
                deltas.iter().any(|delta| *delta > 0.049)
                    && deltas.iter().any(|delta| *delta < 0.034),
                "fixture really has irregular presentation intervals"
            );
        }
        let fingerprint = format!("sha256:{}", "3".repeat(64));
        let mut document = EditDocument::default();
        document.video.segments = [(18_000, 65_970), (65_970, 110_700)]
            .into_iter()
            .enumerate()
            .map(|(index, (start, end))| VideoSegment {
                segment_id: format!("shot-{index}"),
                source_fingerprint: fingerprint.clone(),
                in_ticks: start,
                out_ticks: end,
                layout: Layout {
                    state: LayoutState::SpeakerFill,
                    crop_path: vec![CropKeyframe {
                        t_ticks: 0,
                        rect: CropRect {
                            x: 0,
                            y: 0,
                            width: 90,
                            height: 160,
                        },
                        easing: clipmill_edit_ir::CropEasing::Linear,
                    }],
                    secondary_crop_path: Vec::new(),
                    ..Layout::default()
                },
            })
            .collect();
        let plan = encode_fixture(
            &binary,
            &case,
            &document,
            SourceInput {
                fingerprint,
                path: case.join("source.mp4").to_string_lossy().into_owned(),
                width: 160,
                height: 180,
                has_audio: true,
                duration_ticks: 135_000,
                keyframe_ticks: vec![0],
            },
        );
        assert_eq!(plan.frame_count, 31);
        let decoded = ffmpeg(
            &binary,
            &case,
            &args(&[
                "-i",
                "clip.mp4",
                "-an",
                "-vf",
                "scale=1:1,format=gray",
                "-f",
                "rawvideo",
                "pipe:1",
            ]),
        );
        assert_eq!(
            decoded.stdout.len(),
            31,
            "{name}: the final partial frame remains present"
        );
        for frame in [0_u32, 12, 24, 30] {
            let source_seconds = 0.2 + f64::from(frame) * 1001.0 / 30_000.0;
            let expected = source_seconds * 100.0 * 255.0 / 219.0;
            let actual = f64::from(decoded.stdout[usize::try_from(frame).unwrap()]);
            assert!(
                (actual - expected).abs() < 8.0,
                "{name}: output frame {frame} has source-time luma {actual}, expected {expected:.2}"
            );
        }
        eprintln!(
            "Source {name}: 31 decoded frames, source time/crop/remainder verified; {}",
            case.join("clip.mp4").display()
        );
    }
}

#[test]
#[ignore = "requires pinned FFmpeg; validates display-space crops for rotation metadata"]
#[allow(
    clippy::too_many_lines,
    reason = "actual rotated files decoded before display-space crop assertion"
)]
fn rotation_metadata_is_applied_before_display_space_cropping() {
    let (root, work) = gate_workspace("rotation");
    let binary = root.join(".cache/bin/ffmpeg");
    let probe = root.join(".cache/bin/ffprobe");
    ffmpeg(
        &binary,
        &work,
        &args(&[
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=320x180:r=30:d=1.1,drawbox=x=160:y=0:w=160:h=180:color=blue:t=fill",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=1.1",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-threads",
            "1",
            "-c:a",
            "aac",
            "original.mp4",
        ]),
    );
    for angle in [90, 180, 270] {
        let case = work.join(angle.to_string());
        std::fs::create_dir(&case).expect("case");
        ffmpeg(
            &binary,
            &case,
            &args(&[
                "-display_rotation:v:0",
                &angle.to_string(),
                "-i",
                work.join("original.mp4").to_str().unwrap(),
                "-c",
                "copy",
                "source.mp4",
            ]),
        );
        let info = probe_json(
            &probe,
            &case.join("source.mp4"),
            &["-select_streams", "v:0", "-show_streams"],
        );
        assert!(
            !info["streams"][0]["side_data_list"]
                .as_array()
                .unwrap()
                .is_empty(),
            "fixture includes a display matrix"
        );
        let (width, height, x, y) = if angle == 180 {
            (320, 180, 0, 10)
        } else {
            (180, 320, 45, 0)
        };
        let fingerprint = format!("sha256:{}", "4".repeat(64));
        let mut document = EditDocument::default();
        document.video.segments = vec![VideoSegment {
            segment_id: "rotated".to_owned(),
            source_fingerprint: fingerprint.clone(),
            in_ticks: 9_000,
            out_ticks: 99_000,
            layout: Layout {
                state: LayoutState::SpeakerFill,
                crop_path: vec![CropKeyframe {
                    t_ticks: 0,
                    rect: CropRect {
                        x,
                        y,
                        width: 90,
                        height: 160,
                    },
                    easing: clipmill_edit_ir::CropEasing::Linear,
                }],
                secondary_crop_path: Vec::new(),
                ..Layout::default()
            },
        }];
        let plan = encode_fixture(
            &binary,
            &case,
            &document,
            SourceInput {
                fingerprint,
                path: case.join("source.mp4").to_string_lossy().into_owned(),
                width,
                height,
                has_audio: true,
                duration_ticks: 99_000,
                keyframe_ticks: vec![0],
            },
        );
        let decoded = ffmpeg(
            &binary,
            &case,
            &args(&[
                "-i",
                "clip.mp4",
                "-an",
                "-vf",
                "scale=1:1",
                "-pix_fmt",
                "rgb24",
                "-f",
                "rawvideo",
                "pipe:1",
            ]),
        );
        assert_eq!(
            decoded.stdout.len(),
            usize::try_from(plan.frame_count).unwrap() * 3
        );
        for pixel in decoded.stdout.chunks_exact(3) {
            let (dominant, other) = if angle == 270 {
                (pixel[0], pixel[2])
            } else {
                (pixel[2], pixel[0])
            };
            assert!(
                dominant > 180 && other < 70,
                "rotation {angle}: crop picked wrong display-space half: {pixel:?}"
            );
        }
        let output = probe_json(
            &probe,
            &case.join("clip.mp4"),
            &["-select_streams", "v:0", "-show_streams"],
        );
        assert_eq!(output["streams"][0]["width"], 90);
        assert_eq!(output["streams"][0]["height"], 160);
        assert!(
            output["streams"][0]["side_data_list"]
                .as_array()
                .is_none_or(|list| list
                    .iter()
                    .all(|item| item["rotation"].as_i64().unwrap_or(0) == 0)),
            "rotation must be baked in, not applied again by players"
        );
        eprintln!(
            "Rotation {angle}: decoded crop/display dimensions verified; {}",
            case.join("clip.mp4").display()
        );
    }
}

#[test]
#[ignore = "requires pinned FFmpeg; verifies audio-less sources remain deliverable"]
fn a_source_without_audio_encodes_silence_instead_of_invalid_loudness() {
    let (root, work) = gate_workspace("silent-source");
    let binary = root.join(".cache/bin/ffmpeg");
    ffmpeg(
        &binary,
        &work,
        &args(&[
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=160x180:r=30:d=1",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-threads",
            "1",
            "source.mp4",
        ]),
    );
    let fingerprint = format!("sha256:{}", "5".repeat(64));
    let mut document = EditDocument::default();
    document.video.segments = vec![VideoSegment {
        segment_id: "silent".to_owned(),
        source_fingerprint: fingerprint.clone(),
        in_ticks: 0,
        out_ticks: 90_000,
        layout: Layout {
            state: LayoutState::SpeakerFill,
            crop_path: vec![CropKeyframe {
                t_ticks: 0,
                rect: CropRect {
                    x: 0,
                    y: 0,
                    width: 90,
                    height: 160,
                },
                easing: clipmill_edit_ir::CropEasing::Linear,
            }],
            secondary_crop_path: Vec::new(),
            ..Layout::default()
        },
    }];
    encode_fixture(
        &binary,
        &work,
        &document,
        SourceInput {
            fingerprint,
            path: work.join("source.mp4").to_string_lossy().into_owned(),
            width: 160,
            height: 180,
            has_audio: false,
            duration_ticks: 90_000,
            keyframe_ticks: vec![0],
        },
    );
    let audio = ffmpeg(
        &binary,
        &work,
        &args(&[
            "-i", "clip.mp4", "-vn", "-ac", "1", "-ar", "48000", "-f", "f32le", "pipe:1",
        ]),
    );
    assert!(audio.stdout.len() >= 48_000 * 4);
    assert!(
        audio
            .stdout
            .chunks_exact(4)
            .all(|value| f32::from_le_bytes(value.try_into().unwrap()).abs() < 0.00001)
    );
}

#[test]
#[ignore = "requires pinned FFmpeg; integrated loudness is unavailable for a 200ms edit"]
fn a_short_audible_span_keeps_finite_nonzero_audio() {
    let (root, work) = gate_workspace("short-audio");
    let binary = root.join(".cache/bin/ffmpeg");
    ffmpeg(
        &binary,
        &work,
        &args(&[
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=160x180:r=30:d=1",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=1",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-threads",
            "1",
            "-c:a",
            "aac",
            "source.mp4",
        ]),
    );
    let fingerprint = format!("sha256:{}", "6".repeat(64));
    let mut document = EditDocument::default();
    document.video.segments = vec![VideoSegment {
        segment_id: "brief".to_owned(),
        source_fingerprint: fingerprint.clone(),
        in_ticks: 0,
        out_ticks: 18_000,
        layout: Layout {
            state: LayoutState::SpeakerFill,
            crop_path: vec![CropKeyframe {
                t_ticks: 0,
                rect: CropRect {
                    x: 0,
                    y: 0,
                    width: 90,
                    height: 160,
                },
                easing: clipmill_edit_ir::CropEasing::Linear,
            }],
            secondary_crop_path: Vec::new(),
            ..Layout::default()
        },
    }];
    encode_fixture(
        &binary,
        &work,
        &document,
        SourceInput {
            fingerprint,
            path: work.join("source.mp4").to_string_lossy().into_owned(),
            width: 160,
            height: 180,
            has_audio: true,
            duration_ticks: 90_000,
            keyframe_ticks: vec![0],
        },
    );
    let decoded = ffmpeg(
        &binary,
        &work,
        &args(&[
            "-i", "clip.mp4", "-vn", "-ac", "1", "-ar", "48000", "-f", "f32le", "pipe:1",
        ]),
    );
    let samples: Vec<f32> = decoded
        .stdout
        .chunks_exact(4)
        .map(|sample| f32::from_le_bytes(sample.try_into().unwrap()))
        .collect();
    assert!(samples.iter().all(|sample| sample.is_finite()));
    assert!(
        samples.iter().any(|sample| sample.abs() > 0.01),
        "a short sound must not turn into silence"
    );
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "one explicit end-to-end encoder scenario"
)]
#[ignore = "requires the pinned .cache/bin/ffmpeg; renders and decodes actual pixels"]
fn a_split_a_coloured_zoomed_fit_and_an_inset_draw_what_they_say() {
    use clipmill_edit_ir::{FitBackground, Inset, InsetCorner};
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository");
    let binary = root.join(".cache/bin/ffmpeg");
    assert!(binary.is_file(), "fetch pinned FFmpeg before this gate");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let work = std::env::temp_dir().join(format!("clipmill-layouts-{nonce}"));
    std::fs::create_dir(&work).expect("scratch directory");
    // Red on the left half, blue on the right.
    ffmpeg(
        &binary,
        &work,
        &args(&[
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=1920x1080:r=30:d=3,drawbox=x=960:y=0:w=960:h=1080:color=blue:t=fill",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=3",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-threads",
            "1",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "source.mp4",
        ]),
    );
    let fingerprint = format!("sha256:{}", "1".repeat(64));
    let still = |x, y, width, height| {
        vec![CropKeyframe {
            t_ticks: 0,
            rect: CropRect {
                x,
                y,
                width,
                height,
            },
            easing: clipmill_edit_ir::CropEasing::Linear,
        }]
    };
    let section = |id: &str, second: i64, layout: Layout| VideoSegment {
        segment_id: id.to_owned(),
        source_fingerprint: fingerprint.clone(),
        in_ticks: second * 90_000,
        out_ticks: (second + 1) * 90_000,
        layout,
    };
    let mut document = EditDocument::default();
    document.video.transition_ticks = 0;
    document.video.segments = vec![
        // The whole recording in the top 606 rows, the blue half below.
        section(
            "screen",
            0,
            Layout {
                state: LayoutState::TwoUp,
                crop_path: still(0, 2, 1_918, 1_076),
                secondary_crop_path: still(1_000, 0, 888, 1_080),
                split: Some(316),
                ..Layout::default()
            },
        ),
        // Fitted on green, half as large again.
        section(
            "fitted",
            1,
            Layout {
                state: LayoutState::Fit,
                background: Some(FitBackground::Colour {
                    colour: "#00FF00".to_owned(),
                }),
                zoom: Some(150),
                ..Layout::default()
            },
        ),
        // The whole frame, with the blue half inset bottom left.
        section(
            "inset",
            2,
            Layout {
                state: LayoutState::PictureInPicture,
                secondary_crop_path: still(1_100, 100, 800, 800),
                inset: Some(Inset {
                    corner: InsetCorner::BottomLeft,
                    size: 400,
                }),
                ..Layout::default()
            },
        ),
    ];
    let source = SourceInput {
        fingerprint,
        path: work.join("source.mp4").to_string_lossy().into_owned(),
        width: 1920,
        height: 1080,
        has_audio: true,
        duration_ticks: 3 * 90_000,
        keyframe_ticks: vec![0],
    };
    let plan = compile(&document, &[source], &RenderProfile::default()).expect("compile");
    let measured = ffmpeg(&binary, &work, &plan.measurement_args());
    let measurement =
        LoudnessMeasurement::from_loudnorm_json(&String::from_utf8_lossy(&measured.stderr))
            .expect("measured loudness");
    ffmpeg(&binary, &work, &plan.encode_args(measurement));
    let pixel = |seconds: &str, x: u32, y: u32| {
        let decoded = ffmpeg(
            &binary,
            &work,
            &args(&[
                "-ss",
                seconds,
                "-i",
                "clip.mp4",
                "-frames:v",
                "1",
                "-vf",
                &format!("crop=2:2:{x}:{y},scale=1:1"),
                "-pix_fmt",
                "rgb24",
                "-f",
                "rawvideo",
                "pipe:1",
            ]),
        );
        assert_eq!(decoded.stdout.len(), 3);
        [decoded.stdout[0], decoded.stdout[1], decoded.stdout[2]]
    };
    let red = |[r, g, b]: [u8; 3]| r > 180 && g < 80 && b < 80;
    let blue = |[r, g, b]: [u8; 3]| b > 180 && r < 80 && g < 80;
    let green = |[r, g, b]: [u8; 3]| g > 180 && r < 80 && b < 80;
    let at = |seconds, x, y| pixel(seconds, x, y);

    // Split: the whole recording on top, both halves; the face below.
    assert!(red(at("0.5", 100, 300)), "{:?}", at("0.5", 100, 300));
    assert!(blue(at("0.5", 1_000, 300)), "{:?}", at("0.5", 1_000, 300));
    assert!(blue(at("0.5", 540, 1_500)), "{:?}", at("0.5", 540, 1_500));
    // Fit: green above the zoomed picture, which lost its outer sides.
    assert!(green(at("1.5", 540, 100)), "{:?}", at("1.5", 540, 100));
    assert!(red(at("1.5", 100, 960)), "{:?}", at("1.5", 100, 960));
    assert!(blue(at("1.5", 1_000, 960)), "{:?}", at("1.5", 1_000, 960));
    // Inset: blue in the corner square, over the fitted frame's red half.
    assert!(blue(at("2.5", 258, 1_204)), "{:?}", at("2.5", 258, 1_204));
    assert!(red(at("2.5", 258, 900)), "{:?}", at("2.5", 258, 900));
    eprintln!(
        "Layouts decoded; artifact {}",
        work.join("clip.mp4").display()
    );
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "two end-to-end renders, each decoded at the pixels that tell"
)]
#[ignore = "requires the pinned .cache/bin/ffmpeg; renders and decodes actual pixels"]
fn a_landscape_split_and_a_square_fit_render_in_their_own_frames() {
    use clipmill_edit_ir::{FitBackground, FrameShape};
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository");
    let binary = root.join(".cache/bin/ffmpeg");
    assert!(binary.is_file(), "fetch pinned FFmpeg before this gate");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let work = std::env::temp_dir().join(format!("clipmill-shapes-{nonce}"));
    std::fs::create_dir(&work).expect("scratch directory");
    // Red on the left half, blue on the right.
    ffmpeg(
        &binary,
        &work,
        &args(&[
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=1920x1080:r=30:d=2,drawbox=x=960:y=0:w=960:h=1080:color=blue:t=fill",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=2",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-threads",
            "1",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "source.mp4",
        ]),
    );
    let fingerprint = format!("sha256:{}", "1".repeat(64));
    let still = |x, width| {
        vec![CropKeyframe {
            t_ticks: 0,
            rect: CropRect {
                x,
                y: 0,
                width,
                height: 1_080,
            },
            easing: clipmill_edit_ir::CropEasing::Linear,
        }]
    };
    let source = SourceInput {
        fingerprint: fingerprint.clone(),
        path: work.join("source.mp4").to_string_lossy().into_owned(),
        width: 1920,
        height: 1080,
        has_audio: true,
        duration_ticks: 2 * 90_000,
        keyframe_ticks: vec![0],
    };
    let render = |shape: FrameShape, layout: Layout, name: &str| {
        let mut document = EditDocument::default();
        document.video.shape = shape;
        document.video.transition_ticks = 0;
        document.video.segments = vec![VideoSegment {
            segment_id: "seg_1".to_owned(),
            source_fingerprint: fingerprint.clone(),
            in_ticks: 0,
            out_ticks: 2 * 90_000,
            layout,
        }];
        let profile = RenderProfile::for_output(shape, 1_920, RenderProfile::default().frame_rate)
            .expect("offered");
        let plan = compile(&document, std::slice::from_ref(&source), &profile).expect("compile");
        let measured = ffmpeg(&binary, &work, &plan.measurement_args());
        let measurement =
            LoudnessMeasurement::from_loudnorm_json(&String::from_utf8_lossy(&measured.stderr))
                .expect("measured loudness");
        ffmpeg(&binary, &work, &plan.encode_args(measurement));
        std::fs::rename(work.join("clip.mp4"), work.join(name)).expect("keep the render");
    };
    let size = |name: &str| {
        let probed = Command::new(&binary)
            .current_dir(&work)
            .args(["-hide_banner", "-nostdin", "-i", name])
            .output()
            .expect("probe");
        let text = String::from_utf8_lossy(&probed.stderr).into_owned();
        text.lines()
            .find(|line| line.contains("Video:"))
            .and_then(|line| {
                line.split([',', ' '])
                    .find(|word| {
                        word.split_once('x').is_some_and(|(width, height)| {
                            width.parse::<u32>().is_ok() && height.parse::<u32>().is_ok()
                        })
                    })
                    .map(str::to_owned)
            })
            .unwrap_or_default()
    };
    let pixel = |name: &str, x: u32, y: u32| {
        let decoded = ffmpeg(
            &binary,
            &work,
            &args(&[
                "-ss",
                "1",
                "-i",
                name,
                "-frames:v",
                "1",
                "-vf",
                &format!("crop=2:2:{x}:{y},scale=1:1"),
                "-pix_fmt",
                "rgb24",
                "-f",
                "rawvideo",
                "pipe:1",
            ]),
        );
        assert_eq!(decoded.stdout.len(), 3);
        [decoded.stdout[0], decoded.stdout[1], decoded.stdout[2]]
    };
    let red = |[r, g, b]: [u8; 3]| r > 180 && g < 80 && b < 80;
    let blue = |[r, g, b]: [u8; 3]| b > 180 && r < 80 && g < 80;
    let green = |[r, g, b]: [u8; 3]| g > 180 && r < 80 && b < 80;

    // Landscape, split side by side: the blue half on the left, red on the right.
    render(
        FrameShape::Landscape,
        Layout {
            state: LayoutState::TwoUp,
            crop_path: still(960, 960),
            secondary_crop_path: still(0, 960),
            ..Layout::default()
        },
        "landscape.mp4",
    );
    assert_eq!(size("landscape.mp4"), "1920x1080");
    let at = |x, y| pixel("landscape.mp4", x, y);
    assert!(blue(at(200, 540)), "{:?}", at(200, 540));
    assert!(red(at(1_700, 540)), "{:?}", at(1_700, 540));

    // Square, fitted on green: bands above and below the whole picture.
    render(
        FrameShape::Square,
        Layout {
            state: LayoutState::Fit,
            background: Some(FitBackground::Colour {
                colour: "#00FF00".to_owned(),
            }),
            ..Layout::default()
        },
        "square.mp4",
    );
    assert_eq!(size("square.mp4"), "1080x1080");
    let at = |x, y| pixel("square.mp4", x, y);
    assert!(green(at(540, 60)), "{:?}", at(540, 60));
    assert!(red(at(200, 540)), "{:?}", at(200, 540));
    assert!(blue(at(900, 540)), "{:?}", at(900, 540));
    assert!(green(at(540, 1_020)), "{:?}", at(540, 1_020));
    eprintln!("Shapes decoded; renders in {}", work.display());
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "one explicit end-to-end encoder scenario"
)]
#[ignore = "requires the pinned .cache/bin/ffmpeg and fonts; renders and decodes actual pixels"]
fn a_text_on_its_plate_is_drawn_where_the_document_puts_it() {
    use clipmill_edit_ir::{FitBackground, Overlay, OverlayContent, TextRole};
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository");
    let binary = root.join(".cache/bin/ffmpeg");
    assert!(binary.is_file(), "fetch pinned FFmpeg before this gate");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let work = std::env::temp_dir().join(format!("clipmill-text-{nonce}"));
    std::fs::create_dir_all(work.join("fonts")).expect("scratch directory");
    std::fs::copy(
        root.join(".cache/fonts/Inter-Bold.ttf"),
        work.join("fonts/Inter-Bold.ttf"),
    )
    .expect("pinned font");
    // Red on the left half, blue on the right.
    ffmpeg(
        &binary,
        &work,
        &args(&[
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=1920x1080:r=30:d=2,drawbox=x=960:y=0:w=960:h=1080:color=blue:t=fill",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=2",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-threads",
            "1",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "source.mp4",
        ]),
    );
    let fingerprint = format!("sha256:{}", "1".repeat(64));
    let mut document = EditDocument::default();
    document.video.transition_ticks = 0;
    document.video.segments = vec![VideoSegment {
        segment_id: "seg_1".to_owned(),
        source_fingerprint: fingerprint.clone(),
        in_ticks: 0,
        out_ticks: 2 * 90_000,
        layout: Layout {
            state: LayoutState::Fit,
            background: Some(FitBackground::Colour {
                colour: "#000000".to_owned(),
            }),
            ..Layout::default()
        },
    }];
    // A thin letter on a green plate, in the middle of the frame.
    document.overlays = vec![Overlay {
        overlay_id: "ovl_hook".to_owned(),
        start_ticks: 0,
        end_ticks: 2 * 90_000,
        content: OverlayContent::Text {
            text: "I".to_owned(),
            role: TextRole::Hook,
            x: 500,
            y: 500,
            size: 120,
            colour: "#FFFFFF".to_owned(),
            plate: Some("#00FF00".to_owned()),
        },
    }];
    document.captions.style_ref = RenderProfile::default().caption_style.style_ref;
    let source = SourceInput {
        fingerprint,
        path: work.join("source.mp4").to_string_lossy().into_owned(),
        width: 1920,
        height: 1080,
        has_audio: true,
        duration_ticks: 2 * 90_000,
        keyframe_ticks: vec![0],
    };
    let plan = compile(&document, &[source], &RenderProfile::default()).expect("compile");
    std::fs::write(work.join("clip.ass"), &plan.ass).expect("script");
    let measured = ffmpeg(&binary, &work, &plan.measurement_args());
    let measurement =
        LoudnessMeasurement::from_loudnorm_json(&String::from_utf8_lossy(&measured.stderr))
            .expect("measured loudness");
    ffmpeg(&binary, &work, &plan.encode_args(measurement));
    let pixel = |x: u32, y: u32| {
        let decoded = ffmpeg(
            &binary,
            &work,
            &args(&[
                "-ss",
                "1",
                "-i",
                "clip.mp4",
                "-frames:v",
                "1",
                "-vf",
                &format!("crop=2:2:{x}:{y},scale=1:1"),
                "-pix_fmt",
                "rgb24",
                "-f",
                "rawvideo",
                "pipe:1",
            ]),
        );
        assert_eq!(decoded.stdout.len(), 3);
        [decoded.stdout[0], decoded.stdout[1], decoded.stdout[2]]
    };
    let white = |[r, g, b]: [u8; 3]| r > 200 && g > 200 && b > 200;
    let green = |[r, g, b]: [u8; 3]| g > 180 && r < 80 && b < 80;
    let red = |[r, g, b]: [u8; 3]| r > 180 && g < 80 && b < 80;
    // The letter's stroke at the centre, its plate just beside it, and the
    // picture carrying on beyond the plate.
    assert!(white(pixel(539, 959)), "{:?}", pixel(539, 959));
    assert!(green(pixel(505, 959)), "{:?}", pixel(505, 959));
    assert!(red(pixel(200, 959)), "{:?}", pixel(200, 959));
    eprintln!("Text decoded; render in {}", work.display());
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "one explicit end-to-end encoder scenario"
)]
#[ignore = "requires the pinned .cache/bin/ffmpeg; renders and decodes actual pixels"]
fn a_punch_in_widens_the_picture_for_its_span_only() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository");
    let binary = root.join(".cache/bin/ffmpeg");
    assert!(binary.is_file(), "fetch pinned FFmpeg before this gate");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let work = std::env::temp_dir().join(format!("clipmill-punch-{nonce}"));
    std::fs::create_dir(&work).expect("scratch directory");
    // A white bar 40 pixels wide down the middle of a red frame.
    ffmpeg(
        &binary,
        &work,
        &args(&[
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=1920x1080:r=30:d=2,drawbox=x=940:y=0:w=40:h=1080:color=white:t=fill",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=2",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-threads",
            "1",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "source.mp4",
        ]),
    );
    let fingerprint = format!("sha256:{}", "1".repeat(64));
    let mut document = EditDocument::default();
    document.video.transition_ticks = 0;
    document.video.segments = vec![VideoSegment {
        segment_id: "seg_1".to_owned(),
        source_fingerprint: fingerprint.clone(),
        in_ticks: 0,
        out_ticks: 2 * 90_000,
        layout: Layout {
            state: LayoutState::SpeakerFill,
            crop_path: vec![CropKeyframe {
                t_ticks: 0,
                rect: CropRect {
                    x: 656,
                    y: 0,
                    width: 608,
                    height: 1_080,
                },
                easing: clipmill_edit_ir::CropEasing::Linear,
            }],
            punches: vec![clipmill_edit_ir::Punch {
                start_ticks: 45_000,
                end_ticks: 135_000,
                zoom: 150,
            }],
            ..Layout::default()
        },
    }];
    let source = SourceInput {
        fingerprint,
        path: work.join("source.mp4").to_string_lossy().into_owned(),
        width: 1920,
        height: 1080,
        has_audio: true,
        duration_ticks: 2 * 90_000,
        keyframe_ticks: vec![0],
    };
    let plan = compile(&document, &[source], &RenderProfile::default()).expect("compile");
    let measured = ffmpeg(&binary, &work, &plan.measurement_args());
    let measurement =
        LoudnessMeasurement::from_loudnorm_json(&String::from_utf8_lossy(&measured.stderr))
            .expect("measured loudness");
    ffmpeg(&binary, &work, &plan.encode_args(measurement));
    let pixel = |seconds: &str, x: u32| {
        let decoded = ffmpeg(
            &binary,
            &work,
            &args(&[
                "-ss",
                seconds,
                "-i",
                "clip.mp4",
                "-frames:v",
                "1",
                "-vf",
                &format!("crop=2:2:{x}:959,scale=1:1"),
                "-pix_fmt",
                "rgb24",
                "-f",
                "rawvideo",
                "pipe:1",
            ]),
        );
        assert_eq!(decoded.stdout.len(), 3);
        [decoded.stdout[0], decoded.stdout[1], decoded.stdout[2]]
    };
    let white = |[r, g, b]: [u8; 3]| r > 200 && g > 200 && b > 200;
    let red = |[r, g, b]: [u8; 3]| r > 180 && g < 80 && b < 80;
    // Before the punch the bar spans 504 to 576 of the output's width; during
    // it, 487 to 593; after it, back to 504 to 576.
    assert!(white(pixel("0.25", 540)), "{:?}", pixel("0.25", 540));
    assert!(red(pixel("0.25", 587)), "{:?}", pixel("0.25", 587));
    assert!(white(pixel("1.0", 587)), "{:?}", pixel("1.0", 587));
    assert!(white(pixel("1.0", 492)), "{:?}", pixel("1.0", 492));
    assert!(red(pixel("1.75", 587)), "{:?}", pixel("1.75", 587));
    eprintln!("Punch decoded; render in {}", work.display());
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "one explicit end-to-end encoder scenario"
)]
#[ignore = "requires the pinned .cache/bin/ffmpeg; renders and decodes actual pixels"]
fn a_progress_bar_fills_along_its_edge_as_the_clip_plays() {
    use clipmill_edit_ir::{BarEdge, Brand, ProgressBar};
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository");
    let binary = root.join(".cache/bin/ffmpeg");
    assert!(binary.is_file(), "fetch pinned FFmpeg before this gate");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let work = std::env::temp_dir().join(format!("clipmill-bar-{nonce}"));
    std::fs::create_dir(&work).expect("scratch directory");
    ffmpeg(
        &binary,
        &work,
        &args(&[
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=1920x1080:r=30:d=2",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=2",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-threads",
            "1",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "source.mp4",
        ]),
    );
    let fingerprint = format!("sha256:{}", "1".repeat(64));
    let mut document = EditDocument::default();
    document.video.transition_ticks = 0;
    document.video.segments = vec![VideoSegment {
        segment_id: "seg_1".to_owned(),
        source_fingerprint: fingerprint.clone(),
        in_ticks: 0,
        out_ticks: 2 * 90_000,
        layout: Layout {
            state: LayoutState::SpeakerFill,
            crop_path: vec![CropKeyframe {
                t_ticks: 0,
                rect: CropRect {
                    x: 656,
                    y: 0,
                    width: 608,
                    height: 1_080,
                },
                easing: clipmill_edit_ir::CropEasing::Linear,
            }],
            ..Layout::default()
        },
    }];
    document.brand = Some(Brand {
        progress: Some(ProgressBar {
            colour: "#00FF00".to_owned(),
            edge: BarEdge::Bottom,
            thickness: 24,
        }),
        logo: None,
    });
    let source = SourceInput {
        fingerprint,
        path: work.join("source.mp4").to_string_lossy().into_owned(),
        width: 1920,
        height: 1080,
        has_audio: true,
        duration_ticks: 2 * 90_000,
        keyframe_ticks: vec![0],
    };
    let plan = compile(&document, &[source], &RenderProfile::default()).expect("compile");
    let measured = ffmpeg(&binary, &work, &plan.measurement_args());
    let measurement =
        LoudnessMeasurement::from_loudnorm_json(&String::from_utf8_lossy(&measured.stderr))
            .expect("measured loudness");
    ffmpeg(&binary, &work, &plan.encode_args(measurement));
    let pixel = |seconds: &str, x: u32, y: u32| {
        let decoded = ffmpeg(
            &binary,
            &work,
            &args(&[
                "-ss",
                seconds,
                "-i",
                "clip.mp4",
                "-frames:v",
                "1",
                "-vf",
                &format!("crop=2:2:{x}:{y},scale=1:1"),
                "-pix_fmt",
                "rgb24",
                "-f",
                "rawvideo",
                "pipe:1",
            ]),
        );
        assert_eq!(decoded.stdout.len(), 3);
        [decoded.stdout[0], decoded.stdout[1], decoded.stdout[2]]
    };
    let green = |[r, g, b]: [u8; 3]| g > 180 && r < 80 && b < 80;
    let red = |[r, g, b]: [u8; 3]| r > 180 && g < 80 && b < 80;
    // A quarter of the way in, the bar reaches a quarter of the way across.
    assert!(
        green(pixel("0.5", 100, 1_908)),
        "{:?}",
        pixel("0.5", 100, 1_908)
    );
    assert!(
        red(pixel("0.5", 900, 1_908)),
        "{:?}",
        pixel("0.5", 900, 1_908)
    );
    // Near the end it is nearly across, and it never rises above its edge.
    assert!(
        green(pixel("1.9", 900, 1_908)),
        "{:?}",
        pixel("1.9", 900, 1_908)
    );
    assert!(
        red(pixel("1.9", 900, 1_880)),
        "{:?}",
        pixel("1.9", 900, 1_880)
    );
    eprintln!("Progress bar decoded; render in {}", work.display());
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "one explicit end-to-end encoder scenario"
)]
#[ignore = "requires the pinned .cache/bin/ffmpeg; renders and decodes actual pixels"]
fn a_logo_sits_in_its_corner_at_its_size_and_opacity() {
    use clipmill_edit_ir::{Asset, Brand, InsetCorner, Logo};
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository");
    let binary = root.join(".cache/bin/ffmpeg");
    assert!(binary.is_file(), "fetch pinned FFmpeg before this gate");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let work = std::env::temp_dir().join(format!("clipmill-logo-{nonce}"));
    std::fs::create_dir(&work).expect("scratch directory");
    ffmpeg(
        &binary,
        &work,
        &args(&[
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=1920x1080:r=30:d=2",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=2",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-threads",
            "1",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "source.mp4",
        ]),
    );
    // A green picture twice as wide as it is tall, staged as the render
    // stages the asset: under the one name, without an extension.
    ffmpeg(
        &binary,
        &work,
        &args(&[
            "-f",
            "lavfi",
            "-i",
            "color=c=0x00FF00:s=200x100:d=1",
            "-frames:v",
            "1",
            "logo.png",
        ]),
    );
    std::fs::rename(work.join("logo.png"), work.join(clipmill_render::LOGO_FILE))
        .expect("staged logo");
    let fingerprint = format!("sha256:{}", "1".repeat(64));
    let logo_hash = format!("sha256:{}", "2".repeat(64));
    let mut document = EditDocument::default();
    document.video.transition_ticks = 0;
    document.video.segments = vec![VideoSegment {
        segment_id: "seg_1".to_owned(),
        source_fingerprint: fingerprint.clone(),
        in_ticks: 0,
        out_ticks: 2 * 90_000,
        layout: Layout {
            state: LayoutState::SpeakerFill,
            crop_path: vec![CropKeyframe {
                t_ticks: 0,
                rect: CropRect {
                    x: 656,
                    y: 0,
                    width: 608,
                    height: 1_080,
                },
                easing: clipmill_edit_ir::CropEasing::Linear,
            }],
            ..Layout::default()
        },
    }];
    document.assets = vec![Asset {
        hash: logo_hash.clone(),
        license: "own_content".to_owned(),
    }];
    document.brand = Some(Brand {
        progress: None,
        logo: Some(Logo {
            asset: logo_hash,
            corner: InsetCorner::TopRight,
            size: 200,
            opacity: 100,
        }),
    });
    let source = SourceInput {
        fingerprint,
        path: work.join("source.mp4").to_string_lossy().into_owned(),
        width: 1920,
        height: 1080,
        has_audio: true,
        duration_ticks: 2 * 90_000,
        keyframe_ticks: vec![0],
    };
    let plan = compile(&document, &[source], &RenderProfile::default()).expect("compile");
    let measured = ffmpeg(&binary, &work, &plan.measurement_args());
    let measurement =
        LoudnessMeasurement::from_loudnorm_json(&String::from_utf8_lossy(&measured.stderr))
            .expect("measured loudness");
    ffmpeg(&binary, &work, &plan.encode_args(measurement));
    let pixel = |x: u32, y: u32| {
        let decoded = ffmpeg(
            &binary,
            &work,
            &args(&[
                "-ss",
                "1",
                "-i",
                "clip.mp4",
                "-frames:v",
                "1",
                "-vf",
                &format!("crop=2:2:{x}:{y},scale=1:1"),
                "-pix_fmt",
                "rgb24",
                "-f",
                "rawvideo",
                "pipe:1",
            ]),
        );
        assert_eq!(decoded.stdout.len(), 3);
        [decoded.stdout[0], decoded.stdout[1], decoded.stdout[2]]
    };
    let green = |[r, g, b]: [u8; 3]| g > 180 && r < 80 && b < 80;
    let red = |[r, g, b]: [u8; 3]| r > 180 && g < 80 && b < 80;
    // 216 by 108 pixels, 42 in from the right edge and 172 down from the top.
    assert!(green(pixel(930, 226)), "{:?}", pixel(930, 226));
    assert!(green(pixel(826, 176)), "{:?}", pixel(826, 176));
    assert!(red(pixel(930, 300)), "{:?}", pixel(930, 300));
    assert!(red(pixel(1_050, 226)), "{:?}", pixel(1_050, 226));
    eprintln!("Logo decoded; render in {}", work.display());
}
