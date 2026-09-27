//! What the render compiler promises.
//!
//! The load-bearing test here is [`emitted_crop_expressions_mean_what_rust_computes`]:
//! the crop path is interpolated in Rust for the preview plan and by FFmpeg's
//! expression evaluator during the render, and chapter 17 makes a divergence
//! between those two a release-blocking bug. Rather than trust that the two
//! implementations agree, this file evaluates the emitted expression under
//! FFmpeg's own semantics and compares every frame.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use clipmill_edit_ir::{
    CaptionAnimation, CaptionCue, CaptionLine, CaptionRegion, CaptionWord, CropKeyframe, CropRect,
    EditCommand, EditDocument, FrameShape, GainPoint, Layout, LayoutState, VideoSegment,
};
use clipmill_render::{
    CLIP_FILE, LOUDNORM_SLOT, LoudnessMeasurement, RenderError, RenderProfile, SourceInput,
    compile, crop_rect_at,
};

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

fn segment(id: &str, in_ticks: i64, out_ticks: i64, layout: Layout) -> VideoSegment {
    VideoSegment {
        segment_id: id.to_owned(),
        source_fingerprint: SOURCE.to_owned(),
        in_ticks,
        out_ticks,
        layout,
    }
}

fn fit_document() -> EditDocument {
    let mut document = EditDocument::default();
    document.video.segments = vec![segment(
        "seg_1",
        180_000,
        540_000,
        Layout {
            secondary_crop_path: Vec::new(),
            state: LayoutState::Fit,
            crop_path: Vec::new(),
            ..Layout::default()
        },
    )];
    document.captions.style_ref = RenderProfile::default().caption_style.style_ref;
    document
}

fn cue(id: &str, start_frame: i64, end_frame: i64, words: &[(&str, i64, i64)]) -> CaptionCue {
    CaptionCue {
        cue_id: id.to_owned(),
        start_ticks: start_frame * FRAME_TICKS,
        end_ticks: end_frame * FRAME_TICKS,
        region: CaptionRegion::LowerSafe,
        anim: CaptionAnimation::Karaoke,
        lines: vec![CaptionLine {
            words: words
                .iter()
                .map(|(text, start, end)| CaptionWord {
                    text: (*text).to_owned(),
                    start_ticks: start * FRAME_TICKS,
                    end_ticks: end * FRAME_TICKS,
                    word_id: None,
                    emphasis: false,
                })
                .collect(),
        }],
        position: None,
    }
}

fn first_slice() -> EditDocument {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../contracts/fixtures/edit_ir/valid/first_slice.json"
    );
    let bytes = std::fs::read(path).expect("the first-slice fixture is published");
    EditDocument::from_canonical_json(&bytes).expect("the first slice is a valid edit document")
}

#[test]
fn the_first_slice_fixture_compiles_to_a_renderable_plan() {
    let document = first_slice();
    let plan = compile(&document, &[source()], &RenderProfile::default()).expect("compiles");

    // Two trims laid end to end: 4 s and 2 s at 30000/1001.
    assert_eq!(plan.spans.len(), 2);
    assert_eq!(plan.duration_ticks, 540_000);
    assert_eq!(plan.frame_count, 180);
    assert_eq!(plan.cue_windows.len(), 6);

    // Every cue window must sit inside the program and follow the last.
    let mut previous_end = 0;
    for window in &plan.cue_windows {
        assert!(window.first_frame >= previous_end, "cues overlap in frames");
        assert!(window.end_frame <= plan.frame_count);
        assert!(window.first_frame < window.end_frame);
        previous_end = window.end_frame;
    }
    assert_eq!(plan.cue_windows[0].text, "the first slice");
    assert_eq!(plan.cue_windows[5].text, "so preview and render agree");

    // Six cues in every caption surface.
    assert_eq!(plan.ass.matches("\nDialogue:").count(), 6);
    assert_eq!(plan.srt.matches(" --> ").count(), 6);
    assert_eq!(plan.vtt.matches(" --> ").count(), 6);
}

/// A second segment must not silently inherit the first segment's decode.
#[test]
fn each_segment_seeks_to_its_own_keyframe() {
    let plan = compile(&first_slice(), &[source()], &RenderProfile::default()).expect("compiles");
    let [open, close] = &plan.spans[..] else {
        panic!("expected two spans");
    };
    // in_ticks 180000 lands exactly on a keyframe; 900000 does too.
    assert_eq!(open.seek_ticks, 180_000);
    assert_eq!(open.trim_start_ticks, 0);
    assert_eq!(open.trim_end_ticks, 360_000);
    assert_eq!(close.seek_ticks, 900_000);
    assert_eq!(close.trim_start_ticks, 0);
    assert_eq!(close.trim_end_ticks, 180_000);
}

#[test]
fn a_seek_lands_on_the_keyframe_before_the_cut() {
    let mut document = fit_document();
    // 400000 sits between the keyframes at 360000 and 540000.
    document.video.segments[0].in_ticks = 400_000;
    document.video.segments[0].out_ticks = 500_000;
    let plan = compile(&document, &[source()], &RenderProfile::default()).expect("compiles");
    assert_eq!(plan.spans[0].seek_ticks, 360_000);
    assert_eq!(plan.spans[0].trim_start_ticks, 40_000);
    assert_eq!(plan.spans[0].trim_end_ticks, 140_000);
}

/// A source with no reference index is seeked from the start rather than
/// guessed at.
#[test]
fn a_source_without_keyframes_decodes_from_zero() {
    let mut input = source();
    input.keyframe_ticks.clear();
    let plan = compile(&fit_document(), &[input], &RenderProfile::default()).expect("compiles");
    assert_eq!(plan.spans[0].seek_ticks, 0);
    assert_eq!(plan.spans[0].trim_start_ticks, 180_000);
}

#[test]
fn compilation_is_a_pure_function_of_its_inputs() {
    let document = first_slice();
    let profile = RenderProfile::default();
    let first = compile(&document, &[source()], &profile).expect("compiles");
    let second = compile(&document, &[source()], &profile).expect("compiles");
    assert_eq!(first.graph, second.graph);
    assert_eq!(first.measurement_graph, second.measurement_graph);
    assert_eq!(first.ass, second.ass);
    assert_eq!(first.srt, second.srt);
    assert_eq!(first.vtt, second.vtt);
    assert_eq!(first.recipe_config(), second.recipe_config());
}

/// Rationale is explanation, and explanation must not move a pixel.
#[test]
fn rationale_changes_nothing_the_renderer_sees() {
    let profile = RenderProfile::default();
    let mut document = first_slice();
    let baseline = compile(&document, &[source()], &profile).expect("compiles");
    document.rationale = Some(clipmill_edit_ir::Rationale {
        candidate_id: Some("cand_other".to_owned()),
        decisions: vec!["a completely different justification".to_owned()],
    });
    let explained = compile(&document, &[source()], &profile).expect("compiles");
    assert_eq!(baseline.recipe_config(), explained.recipe_config());
    assert_eq!(baseline.ass, explained.ass);
}

#[test]
fn the_encode_pass_is_pinned_to_a_deterministic_profile() {
    let plan = compile(&first_slice(), &[source()], &RenderProfile::default()).expect("compiles");
    let args = plan.encode_args(LoudnessMeasurement {
        input_lufs: -19.5,
        input_true_peak_dbtp: -2.0,
        input_range_lu: 7.5,
        input_threshold_lufs: -29.5,
        target_offset_lu: 0.25,
    });
    let joined = args.join(" ");
    for expected in [
        "-threads:v 4",
        "-fflags +bitexact",
        "-flags:v +bitexact",
        "-flags:a +bitexact",
        "-map_metadata -1",
        "-crf 18",
        "-frames:v 180",
        "-r 30000/1001",
    ] {
        assert!(joined.contains(expected), "missing {expected} in {joined}");
    }
    assert!(args.last().is_some_and(|last| last == CLIP_FILE));
    assert!(
        !joined.contains(LOUDNORM_SLOT),
        "the measurement must be substituted into the graph"
    );
    assert!(joined.contains("measured_I=-19.500000"));
    assert!(joined.contains("loudnorm=I=-14:TP=-1:LRA=11"));
    // Disjoint source windows still need separate seek targets.
    assert_eq!(args.iter().filter(|arg| *arg == "-i").count(), 2);
    assert_eq!(args.iter().filter(|arg| *arg == "-ss").count(), 2);
}

#[test]
fn adjacent_sections_share_one_seek_and_decoder() {
    let mut document = fit_document();
    document.video.segments[0].out_ticks = 360_000;
    document.video.segments.push(segment(
        "seg_2",
        360_000,
        540_000,
        Layout {
            state: LayoutState::Fit,
            crop_path: Vec::new(),
            secondary_crop_path: Vec::new(),
            ..Layout::default()
        },
    ));
    let plan = compile(&document, &[source()], &RenderProfile::default()).expect("compiles");
    let args = plan.encode_args(LoudnessMeasurement {
        input_lufs: -19.5,
        input_true_peak_dbtp: -2.0,
        input_range_lu: 7.5,
        input_threshold_lufs: -29.5,
        target_offset_lu: 0.25,
    });
    assert_eq!(args.iter().filter(|arg| *arg == "-i").count(), 1);
    assert_eq!(args.iter().filter(|arg| *arg == "-ss").count(), 1);
    let graph = args
        .windows(2)
        .find(|pair| pair[0] == "-filter_complex")
        .expect("filter graph")[1]
        .as_str();
    assert!(graph.contains("[0:v]split=2[decode_v0][decode_v1]"));
    assert!(graph.contains("[0:a]asplit=2[decode_a0][decode_a1]"));
}

#[test]
fn spoken_word_highlight_is_independent_of_the_typography_preset() {
    let mut document = fit_document();
    document.captions.style_ref = "clipmill.captions.minimal.v1".to_owned();
    document.captions.cues = vec![cue("cue_1", 0, 60, &[("the", 0, 25), ("speaker", 25, 60)])];
    document.captions.cues[0].anim = CaptionAnimation::None;
    let profile = RenderProfile::default();
    document.captions.options.highlight_spoken_word = Some(true);
    let enabled = compile(&document, &[source()], &profile).expect("highlighted minimal compiles");
    let enabled_preview = clipmill_render::preview_plan(&document, &profile).expect("preview");
    assert!(enabled.ass.contains("{\\k"));
    assert!(enabled_preview.cues[0].karaoke);
    assert_ne!(
        enabled_preview.caption_style.spoken,
        enabled_preview.caption_style.unspoken
    );

    document.captions.options.highlight_spoken_word = Some(false);
    let disabled = compile(&document, &[source()], &profile).expect("plain minimal compiles");
    let disabled_preview = clipmill_render::preview_plan(&document, &profile).expect("preview");
    assert!(!disabled.ass.contains("{\\k"));
    assert!(!disabled_preview.cues[0].karaoke);
    assert_eq!(
        disabled_preview.caption_style.spoken,
        disabled_preview.caption_style.unspoken
    );
}

#[test]
fn the_measurement_pass_never_decodes_video() {
    let plan = compile(&first_slice(), &[source()], &RenderProfile::default()).expect("compiles");
    let args = plan.measurement_args();
    let joined = args.join(" ");
    assert!(joined.contains("print_format=json"));
    assert!(joined.contains("-f null"));
    assert!(!joined.contains(":v]"), "no video stream is mapped in");
    assert!(
        !joined.contains("subtitles="),
        "captions are not rasterised"
    );
    assert!(!joined.contains("[vout]"));
}

#[test]
fn measured_loudness_is_read_out_of_the_filter_report() {
    let stderr = r#"[Parsed_loudnorm_0 @ 0x7f8]
{
	"input_i" : "-19.53",
	"input_tp" : "-2.05",
	"input_lra" : "7.50",
	"input_thresh" : "-29.62",
	"output_i" : "-14.02",
	"target_offset" : "0.25"
}
"#;
    let measurement = LoudnessMeasurement::from_loudnorm_json(stderr).expect("parses");
    assert!((measurement.input_lufs - -19.53).abs() < 1e-9);
    assert!((measurement.input_true_peak_dbtp - -2.05).abs() < 1e-9);
    assert!((measurement.input_range_lu - 7.50).abs() < 1e-9);
    assert!((measurement.input_threshold_lufs - -29.62).abs() < 1e-9);
    assert!((measurement.target_offset_lu - 0.25).abs() < 1e-9);
    assert!(LoudnessMeasurement::from_loudnorm_json("no report here").is_none());
}

#[test]
fn a_silent_source_still_occupies_its_span() {
    let mut input = source();
    input.has_audio = false;
    let plan = compile(&fit_document(), &[input], &RenderProfile::default()).expect("compiles");
    assert!(
        plan.graph.graph.contains("anullsrc"),
        "a silent segment must be filled, not skipped: {}",
        plan.graph.graph
    );
    assert!(!plan.graph.graph.contains("[0:a]"));
}

/// The tail pad belongs to the program, never to a span.
///
/// `-frames:v` caps and does not pad, so the graph holds its last frame to let
/// the encoder reach the count the plan pinned — which matters for any source
/// not already at the render rate, where `fps` resampling yields fewer frames
/// than the span asks for.
///
/// Where it sits is the whole safety of it. A pad that stops on end-of-input
/// never stops, so one placed inside a span chain would hold that span's last
/// frame forever and `concat` would never reach the second span: a two-segment
/// render would hang rather than fail. After the concat there is nothing left
/// to starve, and `-frames:v` ends the stream.
#[test]
fn every_video_span_is_bounded_on_one_global_frame_grid_without_padding_audio() {
    let plan = compile(&first_slice(), &[source()], &RenderProfile::default()).expect("compile");
    assert_eq!(plan.graph.graph.matches("tpad=").count(), plan.spans.len());
    for span in &plan.spans {
        assert!(
            plan.graph
                .graph
                .contains(&format!("trim=end_frame={}", span.frame_count))
        );
    }
    assert!(plan.graph.graph.contains("concat=n=2:v=0:a=1[acat]"));
    assert!(plan.graph.graph.contains("concat=n=2:v=1:a=0[vcat]"));
}

#[test]
fn the_measurement_pass_carries_no_tail_pad() {
    let plan = compile(&fit_document(), &[source()], &RenderProfile::default()).expect("compiles");
    assert!(!plan.measurement_graph.graph.contains("tpad="));
}

#[test]
fn a_gain_curve_becomes_a_program_time_expression() {
    let mut document = fit_document();
    document.audio.gain_curve = vec![
        GainPoint {
            t_ticks: 0,
            gain_db: 0.0,
        },
        GainPoint {
            t_ticks: 180_000,
            gain_db: -6.0,
        },
    ];
    let plan = compile(&document, &[source()], &RenderProfile::default()).expect("compiles");
    assert!(plan.graph.graph.contains("volume=eval=frame"));
    assert!(plan.graph.graph.contains("0.0000"));
    assert!(plan.graph.graph.contains("-6.0000"));
    // No automation means no filter at all, rather than a no-op one.
    let without_automation =
        compile(&fit_document(), &[source()], &RenderProfile::default()).expect("compiles");
    assert!(!without_automation.graph.graph.contains("volume="));
}

#[test]
fn captions_burn_only_when_the_document_has_them() {
    let profile = RenderProfile::default();
    let plain = compile(&fit_document(), &[source()], &profile).expect("compiles");
    assert!(!plain.graph.graph.contains("subtitles="));
    let with_captions = compile(&first_slice(), &[source()], &profile).expect("compiles");
    assert!(
        with_captions
            .graph
            .graph
            .contains("subtitles=filename=clip.ass:fontsdir=fonts"),
        "libass must see exactly the staged font directory"
    );
}

#[test]
fn text_over_the_program_burns_in_the_caption_pass_even_without_captions() {
    use clipmill_edit_ir::{Overlay, OverlayContent, TextRole};
    let mut document = fit_document();
    document.overlays = vec![Overlay {
        overlay_id: "ovl_hook".to_owned(),
        start_ticks: 0,
        end_ticks: 180_000,
        content: OverlayContent::Text {
            text: "Why the second\nquestion wins".to_owned(),
            role: TextRole::Hook,
            x: 500,
            y: 140,
            size: 96,
            colour: "#FFFFFF".to_owned(),
            plate: Some("#E0245E".to_owned()),
        },
    }];
    let profile = RenderProfile::default();
    let plan = compile(&document, &[source()], &profile).expect("compiles");
    assert!(plan.graph.graph.contains("subtitles=filename=clip.ass"));
    assert!(
        plan.ass.contains("Style: text_plate,Inter,"),
        "{}",
        plan.ass
    );
    let dialogue = plan
        .ass
        .lines()
        .find(|line| line.contains("text_plate,,"))
        .expect("the hook is an event");
    assert_eq!(
        dialogue,
        "Dialogue: 10,0:00:00.00,0:00:02.00,text_plate,,0,0,0,,\
         {\\an5\\pos(540,268)\\fs96\\c&HFFFFFF&\\3c&H5E24E0&\\bord19\\shad0}\
         Why the second\\Nquestion wins"
    );
    // The preview draws the same script.
    let preview = clipmill_render::preview_plan(&document, &profile).expect("preview");
    assert!(preview.ass.contains(dialogue));

    // Left past the end of the program by an edit, it is refused by name.
    document.overlays[0].start_ticks = 400_000;
    document.overlays[0].end_ticks = 500_000;
    assert!(matches!(
        refuses(&document, &[source()]),
        RenderError::OverlayOutsideProgram(id) if id == "ovl_hook"
    ));
}

#[test]
fn an_emoji_is_laid_from_its_pinned_picture_under_the_captions() {
    use clipmill_edit_ir::{Overlay, OverlayContent};
    let mut document = first_slice();
    let emoji = |code: &str| Overlay {
        overlay_id: "ovl_money".to_owned(),
        start_ticks: 45_000,
        end_ticks: 153_000,
        content: OverlayContent::Emoji {
            emoji: code.to_owned(),
            x: 500,
            y: 640,
            size: 180,
        },
    };
    document.overlays = vec![emoji("1f4b0")];
    let profile = RenderProfile::default();
    let plan = compile(&document, &[source()], &profile).expect("compiles");
    let graph = &plan.graph.graph;
    // An even side of the short edge, centred where the document says, for
    // the frames of its span.
    assert!(
        graph.contains("movie=filename=emoji/emoji_u1f4b0.png,format=rgba,scale=194:194[emoji0]"),
        "{graph}"
    );
    assert!(
        graph.contains(
            "[emoji0]overlay=x=540-w/2:y=1228-h/2:eof_action=repeat:\
             enable='between(n\\,15\\,50)'"
        ),
        "{graph}"
    );
    // Under the captions, which burn over it; and never a caption event.
    let laid = graph.find("[emoji0]overlay").expect("laid");
    let burned = graph.find("subtitles=").expect("captions");
    assert!(laid < burned, "{graph}");
    assert!(!plan.ass.contains("ovl_money"));
    assert_eq!(
        clipmill_render::emoji_files(&document.overlays),
        ["emoji_u1f4b0.png"]
    );
    // The preview lists it, for the editor to draw and move.
    let preview = clipmill_render::preview_plan(&document, &profile).expect("preview");
    let shown = &preview.overlays[0];
    assert_eq!((shown.kind, shown.emoji.as_str()), ("emoji", "1f4b0"));

    // Alone, an emoji needs no caption pass: it is a picture.
    let mut alone = fit_document();
    alone.overlays = vec![emoji("1f4b0")];
    let plan = compile(&alone, &[source()], &profile).expect("compiles");
    assert!(plan.graph.graph.contains("[emoji0]overlay"));
    assert!(
        !plan.graph.graph.contains("subtitles="),
        "{}",
        plan.graph.graph
    );

    // A code the app does not offer has no picture to stage.
    document.overlays = vec![emoji("1f9a4")];
    assert!(matches!(
        refuses(&document, &[source()]),
        RenderError::UnknownEmoji(code) if code == "1f9a4"
    ));
}

fn picture_cutaway(start: i64, end: i64, push_in: bool) -> clipmill_edit_ir::Cutaway {
    clipmill_edit_ir::Cutaway {
        cutaway_id: "cut_chart".to_owned(),
        start_ticks: start,
        end_ticks: end,
        fit: clipmill_edit_ir::CutawayFit::Fill,
        content: clipmill_edit_ir::CutawayContent::Picture {
            asset: format!("sha256:{}", "3".repeat(64)),
            push_in,
        },
    }
}

fn with_picture(document: &mut EditDocument) {
    document.assets = vec![clipmill_edit_ir::Asset {
        hash: format!("sha256:{}", "3".repeat(64)),
        license: "royalty_free".to_owned(),
    }];
}

/// B-roll lies over the program's own picture and under everything drawn on
/// it: a still held for its frames, footage from one more decoder seeked to
/// a keyframe and read only as far as it shows.
#[test]
fn b_roll_is_laid_over_the_program_for_its_frames() {
    let mut document = fit_document();
    with_picture(&mut document);
    document.video.cutaways = vec![
        picture_cutaway(45_000, 135_000, false),
        clipmill_edit_ir::Cutaway {
            cutaway_id: "cut_room".to_owned(),
            start_ticks: 180_000,
            end_ticks: 270_000,
            fit: clipmill_edit_ir::CutawayFit::Fit,
            content: clipmill_edit_ir::CutawayContent::Footage {
                source_fingerprint: SOURCE.to_owned(),
                in_ticks: 900_000,
            },
        },
    ];
    let plan = compile(&document, &[source()], &RenderProfile::default()).expect("compiles");
    let graph = &plan.graph.graph;
    for piece in [
        format!(
            "movie=filename=cutaways/{},format=yuv420p[cut0raw]",
            "3".repeat(64)
        ),
        "[cut0raw]scale=1080:1920:force_original_aspect_ratio=increase,crop=1080:1920,\
         setsar=1[cut0]"
            .to_owned(),
        "[vcat][cut0]overlay=x=0:y=0:eof_action=repeat:enable='between(n\\,15\\,44)'".to_owned(),
        // The footage is the input after the section's own.
        "[1:v]trim=start=0.000000:end=1.000000,setpts=PTS-STARTPTS,fps=30000/1001:start_time=0,\
         tpad=stop_mode=clone:stop_duration=1,trim=end_frame=30,setpts=PTS-STARTPTS,\
         format=yuv420p[cut1raw]"
            .to_owned(),
        "[cut1rawbg]scale=1080:1920:force_original_aspect_ratio=increase,crop=1080:1920,\
         gblur=sigma=40"
            .to_owned(),
        "[cut1fitted]setpts=PTS-STARTPTS+60*1001/30000/TB[cut1]".to_owned(),
        "[vcut0][cut1]overlay=x=0:y=0:eof_action=pass:enable='between(n\\,60\\,89)'".to_owned(),
        "[vcut1]null[vout]".to_owned(),
    ] {
        assert!(graph.contains(&piece), "{piece}\n\n{graph}");
    }
    // Read from ten seconds in, for its second and a second's slack.
    let args = plan.encode_args(LoudnessMeasurement {
        input_lufs: -20.0,
        input_true_peak_dbtp: -3.0,
        input_range_lu: 5.0,
        input_threshold_lufs: -30.0,
        target_offset_lu: 0.0,
    });
    let footage = args
        .windows(6)
        .position(|window| {
            window
                == [
                    "-ss",
                    "10.000000",
                    "-t",
                    "2.000000",
                    "-i",
                    "/private/fixtures/source.mp4",
                ]
        })
        .expect("the footage's decoder");
    let filter = args
        .iter()
        .position(|arg| arg == "-filter_complex")
        .expect("graph");
    assert!(footage < filter);
    // The measurement never decodes it.
    assert!(!plan.measurement_graph.graph.contains("cut"));
    // The preview is told the same frames, to draw it over the picture.
    let preview =
        clipmill_render::preview_plan(&document, &RenderProfile::default()).expect("preview");
    let shown = preview
        .cutaways
        .iter()
        .map(|cutaway| {
            (
                cutaway.kind,
                cutaway.fit,
                cutaway.first_frame,
                cutaway.end_frame,
                cutaway.in_ticks,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        shown,
        [
            ("picture", "fill", 15, 45, 0),
            ("footage", "fit", 60, 90, 900_000)
        ]
    );
    assert_eq!(
        clipmill_render::cutaway_pictures(&document),
        [format!("sha256:{}", "3".repeat(64))]
    );
}

/// A picture that moves closer is composed at twice the size, so the move
/// lands on half pixels, and eased in over its frames.
#[test]
fn a_picture_that_pushes_in_is_composed_at_twice_the_size() {
    let mut document = fit_document();
    with_picture(&mut document);
    document.video.cutaways = vec![picture_cutaway(180_000, 270_000, true)];
    let plan = compile(&document, &[source()], &RenderProfile::default()).expect("compiles");
    assert!(
        plan.graph.graph.contains(
            "[cut0big]zoompan=z='1+0.08*on/29':x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)':d=30:\
             s=1080x1920:fps=30000/1001,setpts=PTS-STARTPTS+60*1001/30000/TB,format=yuv420p[cut0]"
        ),
        "{}",
        plan.graph.graph
    );
    assert!(plan.graph.graph.contains("crop=2160:3840"));
}

#[test]
fn b_roll_that_cannot_be_read_is_refused_by_name() {
    let footage = |in_ticks: i64, fingerprint: &str| clipmill_edit_ir::Cutaway {
        cutaway_id: "cut_room".to_owned(),
        start_ticks: 0,
        end_ticks: 90_000,
        fit: clipmill_edit_ir::CutawayFit::Fill,
        content: clipmill_edit_ir::CutawayContent::Footage {
            source_fingerprint: fingerprint.to_owned(),
            in_ticks,
        },
    };
    let mut document = fit_document();
    document.video.cutaways = vec![footage(1_300_000, SOURCE)];
    assert!(matches!(
        refuses(&document, &[source()]),
        RenderError::CutawayPastEndOfSource(id) if id == "cut_room"
    ));
    let elsewhere = format!("sha256:{}", "9".repeat(64));
    document.video.cutaways = vec![footage(0, &elsewhere)];
    assert!(matches!(
        refuses(&document, &[source()]),
        RenderError::UnresolvedSource(fingerprint) if fingerprint == elsewhere
    ));
    // Left past the end of the program, it is named.
    with_picture(&mut document);
    document.video.cutaways = vec![picture_cutaway(360_000, 450_000, false)];
    assert!(matches!(
        refuses(&document, &[source()]),
        RenderError::CutawayOutsideProgram(id) if id == "cut_chart"
    ));
}

// ---- Crop path parity -------------------------------------------------------

fn crop_document(path: Vec<CropKeyframe>) -> EditDocument {
    let mut document = EditDocument::default();
    document.video.segments = vec![segment(
        "seg_1",
        0,
        180_000,
        Layout {
            secondary_crop_path: Vec::new(),
            state: LayoutState::SpeakerFill,
            crop_path: path,
            ..Layout::default()
        },
    )];
    document
}

fn keyframe(frame: i64, x: i64, y: i64) -> CropKeyframe {
    CropKeyframe {
        t_ticks: frame * FRAME_TICKS,
        rect: CropRect {
            x,
            y,
            width: 608,
            height: 1_080,
        },
        easing: clipmill_edit_ir::CropEasing::Linear,
    }
}

/// The parity keystone. FFmpeg evaluates the emitted expression in doubles;
/// the preview plan calls `crop_rect_at`. If those ever disagree the crop is
/// on a different pixel in the preview than in the render, which chapter 17
/// makes release-blocking — so the agreement is checked frame by frame rather
/// than assumed from the shared source of the formula.
#[test]
fn emitted_crop_expressions_mean_what_rust_computes() {
    let paths = vec![
        vec![keyframe(0, 100, 0), keyframe(59, 500, 0)],
        vec![keyframe(0, 500, 0), keyframe(59, 100, 0)],
        vec![
            keyframe(10, 0, 0),
            keyframe(20, 333, 0),
            keyframe(50, 90, 0),
        ],
        vec![keyframe(0, 656, 0)],
        vec![
            keyframe(0, 100, 0),
            keyframe(7, 100, 0),
            keyframe(31, 400, 0),
            keyframe(59, 400, 0),
        ],
    ];
    let profile = RenderProfile::default();
    let rate = profile.rate();
    for path in paths {
        let document = crop_document(path.clone());
        let plan = compile(&document, &[source()], &profile).expect("compiles");
        let (x_expression, y_expression) = crop_expressions(&plan.graph.graph);
        for frame in 0..60 {
            let expected = crop_rect_at(&path, rate, frame).expect("a rect");
            assert_eq!(
                evaluate(&x_expression, frame),
                expected.x,
                "x diverged on frame {frame} for {x_expression}"
            );
            assert_eq!(
                evaluate(&y_expression, frame),
                expected.y,
                "y diverged on frame {frame}"
            );
        }
    }
}

/// A trim keeps the crop, and what it keeps is what the renderer draws.
///
/// The reproduction: a static crop is one keyframe at zero, and advancing
/// the head past it left a `speaker_fill` segment with no path, which the
/// renderer refused. Now the trimmed segment compiles, and — for a moving
/// path as well — the emitted expression at every remaining frame is the
/// crop the untrimmed path held at that source frame: exactly for a static
/// crop, and within one frame of motion for a moving one, since the
/// keyframe written at the new edge is an integer rectangle at a tick the
/// renderer rounds to a frame, and the path it stands in for passed that
/// edge between two frames.
#[test]
fn a_trimmed_speaker_fill_segment_still_compiles_to_the_crop_it_had() {
    let profile = RenderProfile::default();
    let rate = profile.rate();
    let paths = vec![
        vec![keyframe(0, 656, 0)],
        vec![keyframe(0, 100, 0), keyframe(59, 500, 0)],
        vec![
            keyframe(0, 100, 0),
            keyframe(7, 100, 0),
            keyframe(31, 400, 0),
            keyframe(59, 400, 0),
        ],
    ];
    for path in paths {
        let untrimmed = crop_document(path.clone());
        let mut document = untrimmed.clone();
        // Ten frames off the head, ten off the tail.
        let head = 10 * FRAME_TICKS;
        EditCommand::Trim {
            segment_id: "seg_1".to_owned(),
            in_ticks: head,
            out_ticks: 180_000 - head,
        }
        .apply(&mut document)
        .expect("the trim applies");
        assert!(
            !document.video.segments[0].layout.crop_path.is_empty(),
            "the crop survived the trim"
        );

        let plan = compile(&document, &[source()], &profile)
            .unwrap_or_else(|error| panic!("a trimmed segment renders: {error}"));
        let (x_expression, y_expression) = crop_expressions(&plan.graph.graph);
        // The fastest the untrimmed camera moved, per frame, plus the pixel
        // the edge rectangle rounded away.
        let tolerance = path
            .windows(2)
            .map(|pair| {
                let frames = ((pair[1].t_ticks - pair[0].t_ticks) / FRAME_TICKS).max(1);
                ((pair[1].rect.x - pair[0].rect.x).abs() + frames - 1) / frames
            })
            .max()
            .unwrap_or(0)
            + i64::from(path.len() > 1);
        for frame in 0..40 {
            let expected = crop_rect_at(&path, rate, frame + 10).expect("a rect");
            let drawn_x = evaluate(&x_expression, frame);
            assert!(
                (drawn_x - expected.x).abs() <= tolerance,
                "x drifted to {drawn_x} from {} on trimmed frame {frame} (source frame {}) for \
                 {x_expression}",
                expected.x,
                frame + 10
            );
            assert_eq!(evaluate(&y_expression, frame), expected.y);
        }
    }
}

/// Pull `x='…'` and `y='…'` back out of the compiled crop filter.
fn crop_expressions(graph: &str) -> (String, String) {
    let crop = graph
        .split("crop=")
        .nth(1)
        .expect("a crop filter in the graph");
    let mut quoted = crop.split('\'');
    let _before_x = quoted.next();
    let x = quoted.next().expect("an x expression").to_owned();
    let _between = quoted.next();
    let y = quoted.next().expect("a y expression").to_owned();
    (x, y)
}

/// Evaluate the restricted expression grammar the compiler emits, under
/// FFmpeg's semantics: double arithmetic, `floor` toward negative infinity,
/// `lt` yielding 1 or 0, `if` selecting on non-zero.
fn evaluate(expression: &str, frame: i64) -> i64 {
    let text: String = expression.replace("\\,", ",").chars().collect();
    let mut parser = Parser {
        bytes: text.as_bytes(),
        position: 0,
        frame: f64::from(i32::try_from(frame).expect("test frames are small")),
    };
    let value = parser.expression();
    assert_eq!(
        parser.position,
        parser.bytes.len(),
        "unparsed tail in {text}"
    );
    // FFmpeg feeds the expression's value to crop as an integer pixel offset.
    #[allow(clippy::cast_possible_truncation)]
    {
        value as i64
    }
}

struct Parser<'a> {
    bytes: &'a [u8],
    position: usize,
    frame: f64,
}

impl Parser<'_> {
    fn peek(&self) -> u8 {
        self.bytes.get(self.position).copied().unwrap_or(b'\0')
    }

    fn eat(&mut self, expected: u8) {
        assert_eq!(self.peek(), expected, "expected {}", expected as char);
        self.position += 1;
    }

    fn matches(&mut self, word: &str) -> bool {
        if self.bytes[self.position..].starts_with(word.as_bytes()) {
            self.position += word.len();
            return true;
        }
        false
    }

    fn expression(&mut self) -> f64 {
        let mut value = self.term();
        loop {
            match self.peek() {
                b'+' => {
                    self.position += 1;
                    value += self.term();
                }
                b'-' => {
                    self.position += 1;
                    value -= self.term();
                }
                _ => return value,
            }
        }
    }

    fn term(&mut self) -> f64 {
        let mut value = self.factor();
        loop {
            match self.peek() {
                b'*' => {
                    self.position += 1;
                    value *= self.factor();
                }
                b'/' => {
                    self.position += 1;
                    value /= self.factor();
                }
                _ => return value,
            }
        }
    }

    fn factor(&mut self) -> f64 {
        if self.matches("if(") {
            let condition = self.expression();
            self.eat(b',');
            let when_true = self.expression();
            self.eat(b',');
            let when_false = self.expression();
            self.eat(b')');
            return if condition == 0.0 {
                when_false
            } else {
                when_true
            };
        }
        if self.matches("lt(") {
            let left = self.expression();
            self.eat(b',');
            let right = self.expression();
            self.eat(b')');
            return f64::from(u8::from(left < right));
        }
        if self.matches("floor(") {
            let value = self.expression();
            self.eat(b')');
            return value.floor();
        }
        if self.matches("n") {
            return self.frame;
        }
        if self.peek() == b'(' {
            self.position += 1;
            let value = self.expression();
            self.eat(b')');
            return value;
        }
        let start = self.position;
        if self.peek() == b'-' {
            self.position += 1;
        }
        while self.peek().is_ascii_digit() || self.peek() == b'.' {
            self.position += 1;
        }
        std::str::from_utf8(&self.bytes[start..self.position])
            .expect("ascii")
            .parse()
            .expect("a number")
    }
}

// ---- Refusals ---------------------------------------------------------------

#[track_caller]
fn refuses(document: &EditDocument, sources: &[SourceInput]) -> RenderError {
    match compile(document, sources, &RenderProfile::default()) {
        Err(error) => error,
        Ok(_) => panic!("compilation should have been refused"),
    }
}

#[test]
fn an_empty_program_is_refused() {
    assert!(matches!(
        refuses(&EditDocument::default(), &[source()]),
        RenderError::EmptyProgram
    ));
}

/// A document authored against a longer cut of the same footage must be
/// refused with its reason, not encoded into a file that is quietly short.
#[test]
fn a_segment_past_the_end_of_its_source_is_refused() {
    let mut input = source();
    input.duration_ticks = 400_000;
    assert!(matches!(
        refuses(&first_slice(), &[input]),
        RenderError::SegmentPastEndOfSource(_)
    ));
}

/// A source whose observation states no duration is taken on trust: the
/// encoder's frame-count check is still there as the backstop.
#[test]
fn a_source_of_unstated_length_is_not_refused() {
    let mut input = source();
    input.duration_ticks = 0;
    assert!(compile(&first_slice(), &[input], &RenderProfile::default()).is_ok());
}

#[test]
fn an_unresolved_source_is_refused_rather_than_skipped() {
    assert!(matches!(
        refuses(&fit_document(), &[]),
        RenderError::UnresolvedSource(_)
    ));
}

#[test]
fn an_unknown_caption_style_is_refused_rather_than_defaulted() {
    let mut document = first_slice();
    document.captions.style_ref = "brand.captions.bold".to_owned();
    assert!(matches!(
        refuses(&document, &[source()]),
        RenderError::UnknownCaptionStyle(_)
    ));
}

#[test]
fn caption_markup_is_refused_rather_than_rewritten() {
    let mut document = fit_document();
    document.captions.cues = vec![cue("cue_1", 0, 30, &[("{drop}", 0, 30)])];
    assert!(matches!(
        refuses(&document, &[source()]),
        RenderError::UnrenderableCaptionText { .. }
    ));
}

#[test]
fn a_cue_past_the_end_of_the_program_is_refused() {
    let mut document = fit_document();
    // The program is 4 s; this cue starts well after it.
    document.captions.cues = vec![cue("cue_1", 200, 230, &[("late", 200, 230)])];
    assert!(matches!(
        refuses(&document, &[source()]),
        RenderError::CueOutsideProgram(_)
    ));
}

#[test]
fn speaker_fill_without_a_path_is_refused() {
    assert!(matches!(
        refuses(&crop_document(Vec::new()), &[source()]),
        RenderError::SpeakerFillWithoutCropPath(_)
    ));
    let preview =
        clipmill_render::preview_plan(&crop_document(Vec::new()), &RenderProfile::default())
            .expect("legacy document remains editable");
    assert!(preview.segments[0].framing_warning.contains("choose Fit"));
    assert!(preview.crops.iter().all(Option::is_none));
}

#[test]
fn a_zooming_crop_path_compiles_and_preview_follows_the_same_windows() {
    let mut path = vec![keyframe(0, 100, 0), keyframe(30, 100, 0)];
    path[1].rect.width = 540;
    path[1].rect.height = 960;
    let document = crop_document(path);
    let compiled =
        compile(&document, &[source()], &RenderProfile::default()).expect("zoom compiles");
    assert!(compiled.graph.graph.contains("eval=frame,crop="));
    let preview =
        clipmill_render::preview_plan(&document, &RenderProfile::default()).expect("preview");
    assert_eq!(preview.crops[0].expect("first crop").width, 608);
    assert_eq!(preview.crops[30].expect("zoomed crop").width, 540);
}

#[test]
fn an_eased_keyframe_uses_the_same_midpoint_in_preview_and_graph() {
    let mut path = vec![keyframe(0, 100, 0), keyframe(40, 500, 0)];
    path[0].easing = clipmill_edit_ir::CropEasing::EaseIn;
    let rate = RenderProfile::default().rate();
    assert_eq!(crop_rect_at(&path, rate, 20).expect("midpoint").x, 200);
    let compiled =
        compile(&crop_document(path), &[source()], &RenderProfile::default()).expect("compiles");
    assert!(compiled.graph.graph.contains("pow("));
}

#[test]
fn a_crop_window_of_the_wrong_shape_is_refused() {
    let mut path = vec![keyframe(0, 0, 0)];
    path[0].rect.width = 1_080;
    path[0].rect.height = 1_080;
    assert!(matches!(
        refuses(&crop_document(path), &[source()]),
        RenderError::CropAspectMismatch(_)
    ));
}

/// 608x1080 is half a pixel off exact 9:16 — the closest an integer rectangle
/// can get at that height — and must be accepted.
#[test]
fn a_crop_window_within_a_pixel_of_the_output_shape_is_accepted() {
    assert!(
        compile(
            &crop_document(vec![keyframe(0, 0, 0)]),
            &[source()],
            &RenderProfile::default(),
        )
        .is_ok()
    );
}

#[test]
fn a_crop_window_reaching_outside_the_frame_is_refused() {
    let path = vec![keyframe(0, 100, 0), keyframe(30, 1_900, 0)];
    assert!(matches!(
        refuses(&crop_document(path), &[source()]),
        RenderError::CropOutsideFrame(_)
    ));
}

#[test]
fn two_crop_keyframes_on_one_frame_are_refused() {
    let path = vec![
        CropKeyframe {
            t_ticks: 0,
            ..keyframe(0, 100, 0)
        },
        CropKeyframe {
            t_ticks: 1,
            ..keyframe(0, 200, 0)
        },
    ];
    assert!(matches!(
        refuses(&crop_document(path), &[source()]),
        RenderError::CropKeyframesTooDense(_)
    ));
}

#[test]
fn two_person_render_and_preview_use_both_independent_viewports() {
    let mut document = crop_document(vec![CropKeyframe {
        t_ticks: 0,
        rect: CropRect {
            x: 0,
            y: 140,
            width: 900,
            height: 800,
        },
        easing: clipmill_edit_ir::CropEasing::Linear,
    }]);
    let layout = &mut document.video.segments[0].layout;
    layout.state = LayoutState::TwoUp;
    layout.secondary_crop_path = vec![CropKeyframe {
        t_ticks: 0,
        rect: CropRect {
            x: 1000,
            y: 140,
            width: 900,
            height: 800,
        },
        easing: clipmill_edit_ir::CropEasing::Linear,
    }];
    let profile = RenderProfile::default();
    let plan = compile(&document, &[source()], &profile).expect("two-person composition compiles");
    assert!(plan.graph.graph.contains("vstack=inputs=2"));
    assert!(plan.graph.graph.contains("scale=1080:960"));
    let preview = clipmill_render::preview_plan(&document, &profile).expect("preview");
    assert_eq!(preview.crops[0].unwrap().x, 0);
    assert_eq!(preview.secondary_crops[0].unwrap().x, 1000);
    assert_eq!(preview.secondary_crops.len(), preview.crops.len());
    document.video.segments[0].layout.secondary_crop_path[0]
        .rect
        .x = 1500;
    assert!(matches!(
        compile(&document, &[source()], &profile),
        Err(RenderError::CropOutsideFrame(_))
    ));
}

fn still(x: i64, y: i64, width: i64, height: i64) -> Vec<CropKeyframe> {
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
}

#[test]
fn a_screen_over_a_face_shares_the_height_at_its_split() {
    // The recording's own 16:9 on top, 606 pixels of 1920; the face below.
    let mut document = crop_document(still(0, 2, 1_918, 1_076));
    let layout = &mut document.video.segments[0].layout;
    layout.state = LayoutState::TwoUp;
    layout.split = Some(316);
    layout.secondary_crop_path = still(1_000, 0, 888, 1_080);
    let profile = RenderProfile::default();
    let plan = compile(&document, &[source()], &profile).expect("a split two-up compiles");
    assert!(
        plan.graph.graph.contains("scale=1080:606"),
        "{}",
        plan.graph.graph
    );
    assert!(plan.graph.graph.contains("scale=1080:1314"));
    let preview = clipmill_render::preview_plan(&document, &profile).expect("preview");
    assert_eq!(preview.segments[0].layout, "two_up");
    assert_eq!(preview.segments[0].upper_height, 606);

    // The even split's portraits no longer fit this split's viewports.
    let layout = &mut document.video.segments[0].layout;
    layout.crop_path = still(0, 140, 900, 800);
    assert!(matches!(
        refuses(&document, &[source()]),
        RenderError::CropAspectMismatch(_)
    ));
}

#[test]
fn a_landscape_frame_splits_side_by_side_and_the_shape_sizes_the_render() {
    // Each half of the recording in its own half of a 16:9 frame.
    let mut document = crop_document(still(0, 0, 960, 1_080));
    document.video.shape = FrameShape::Landscape;
    let layout = &mut document.video.segments[0].layout;
    layout.state = LayoutState::TwoUp;
    layout.secondary_crop_path = still(960, 0, 960, 1_080);
    let profile = RenderProfile::for_output(
        FrameShape::Landscape,
        1_920,
        RenderProfile::default().frame_rate,
    )
    .expect("offered");
    let plan = compile(&document, &[source()], &profile).expect("side by side compiles");
    assert!(
        plan.graph.graph.contains("hstack=inputs=2"),
        "{}",
        plan.graph.graph
    );
    assert!(plan.graph.graph.contains("scale=960:1080"));
    let preview = clipmill_render::preview_plan(&document, &profile).expect("preview");
    assert_eq!((preview.width, preview.height), (1_920, 1_080));
    assert_eq!(preview.segments[0].upper_height, 960);

    // A portrait crop does not fill a landscape half.
    let mut portrait = document.clone();
    portrait.video.segments[0].layout.crop_path = still(0, 0, 608, 1_080);
    assert!(matches!(
        compile(&portrait, &[source()], &profile),
        Err(RenderError::CropAspectMismatch(_))
    ));
    // Nor does a 9:16 profile render a landscape clip.
    assert!(matches!(
        compile(&document, &[source()], &RenderProfile::default()),
        Err(RenderError::ShapeMismatch { .. })
    ));
    assert!(clipmill_render::preview_plan(&document, &RenderProfile::default()).is_err());
}

#[test]
fn a_fitted_picture_takes_its_colour_and_zoom() {
    let mut document = fit_document();
    let layout = &mut document.video.segments[0].layout;
    layout.background = Some(clipmill_edit_ir::FitBackground::Colour {
        colour: "#00FF00".to_owned(),
    });
    layout.zoom = Some(150);
    let plan = compile(&document, &[source()], &RenderProfile::default()).expect("compiles");
    let graph = &plan.graph.graph;
    assert!(graph.contains("color=0x00FF00@1:t=fill"), "{graph}");
    // 1920x1080 fits 1080 wide at 608 tall; half as large again is
    // 1620x912, and the frame keeps its middle 1080.
    assert!(graph.contains("scale=1620:912,crop=1080:912"), "{graph}");
    assert!(!graph.contains("gblur"));

    // An unzoomed, blurred fit compiles exactly as it always has.
    let unstyled = compile(&fit_document(), &[source()], &RenderProfile::default()).expect("fit");
    assert!(
        unstyled
            .graph
            .graph
            .contains("force_original_aspect_ratio=decrease")
    );
    assert!(unstyled.graph.graph.contains("gblur"));
}

#[test]
fn a_picture_in_picture_insets_its_square_in_the_chosen_corner() {
    let mut document = fit_document();
    let layout = &mut document.video.segments[0].layout;
    layout.state = LayoutState::PictureInPicture;
    layout.secondary_crop_path = still(1_100, 100, 800, 800);
    layout.inset = Some(clipmill_edit_ir::Inset {
        corner: clipmill_edit_ir::InsetCorner::BottomLeft,
        size: 400,
    });
    let profile = RenderProfile::default();
    let plan = compile(&document, &[source()], &profile).expect("compiles");
    let graph = &plan.graph.graph;
    // 40% of 1080 is 432, 4% in from the left and clear of the bottom block.
    assert!(graph.contains("scale=432:432"), "{graph}");
    assert!(graph.contains("overlay=x=42:y=988"), "{graph}");
    let preview = clipmill_render::preview_plan(&document, &profile).expect("preview");
    assert_eq!(preview.segments[0].inset, Some((42, 988, 432)));
    assert!(
        preview.crops[0].is_none(),
        "the full picture is the whole frame"
    );
    assert_eq!(preview.secondary_crops[0].unwrap().x, 1_100);

    // An inset that is not square is a framing mistake, not rounding.
    document.video.segments[0].layout.secondary_crop_path = still(1_100, 100, 800, 600);
    assert!(matches!(
        refuses(&document, &[source()]),
        RenderError::CropAspectMismatch(_)
    ));
}

#[test]
fn a_punch_in_draws_a_tighter_crop_in_render_and_preview_alike() {
    // A still crop, punched 150% closer from 0.5 s to 1.5 s of the section.
    let mut document = crop_document(still(656, 0, 608, 1_080));
    document.video.segments[0].layout.punches = vec![clipmill_edit_ir::Punch {
        start_ticks: 45_000,
        end_ticks: 135_000,
        zoom: 150,
    }];
    let profile = RenderProfile::default();
    let plan = compile(&document, &[source()], &profile).expect("a punched crop compiles");
    assert!(
        plan.graph.graph.contains("eval=frame"),
        "a crop that changes size is scaled frame by frame: {}",
        plan.graph.graph
    );
    let preview = clipmill_render::preview_plan(&document, &profile).expect("preview");
    // Frames 7, 30 and 52 at 29.97: a quarter, one and one and three
    // quarter seconds into the section.
    let at = |frame: usize| preview.crops[frame].expect("a crop");
    assert_eq!((at(7).width, at(7).height), (608, 1_080));
    let punched = at(30);
    assert_eq!((punched.width, punched.height), (404, 720));
    assert_eq!(
        (punched.x, punched.y),
        (758, 180),
        "about the crop's own centre"
    );
    assert_eq!((at(52).width, at(52).height), (608, 1_080));
}
