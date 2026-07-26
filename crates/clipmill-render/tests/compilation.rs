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
    EditDocument, GainPoint, Layout, LayoutState, VideoSegment,
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
            state: LayoutState::Fit,
            crop_path: Vec::new(),
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
                })
                .collect(),
        }],
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
        "-threads 1",
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
    // One input per span, each pre-seeked to its own keyframe.
    assert_eq!(args.iter().filter(|arg| *arg == "-i").count(), 2);
    assert_eq!(args.iter().filter(|arg| *arg == "-ss").count(), 2);
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

// ---- Crop path parity -------------------------------------------------------

fn crop_document(path: Vec<CropKeyframe>) -> EditDocument {
    let mut document = EditDocument::default();
    document.video.segments = vec![segment(
        "seg_1",
        0,
        180_000,
        Layout {
            state: LayoutState::SpeakerFill,
            crop_path: path,
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
}

#[test]
fn a_zooming_crop_path_is_refused_with_its_reason() {
    let mut path = vec![keyframe(0, 100, 0), keyframe(30, 100, 0)];
    path[1].rect.width = 700;
    path[1].rect.height = 1_244;
    assert!(matches!(
        refuses(&crop_document(path), &[source()]),
        RenderError::ZoomingCropPath(_)
    ));
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
