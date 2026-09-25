//! Write ASS, SubRip, and WebVTT from caption IR using shared frame timing.
//!
//! Line breaks come from the document; `WrapStyle: 2` prevents libass re-wrapping.
//! Every timestamp derives from a frame index, never floating-point seconds.
//! Burn-in uses kinetic cues when present and reading cues otherwise. Sidecars
//! always use reading cues, with no kinetic fallback.

use clipmill_edit_ir::{CaptionAnimation, CaptionCase, CaptionCue, CaptionRegion, CaptionTrack};

pub(crate) fn burned_text(text: &str, text_case: CaptionCase) -> String {
    match text_case {
        CaptionCase::Original => text.to_owned(),
        CaptionCase::Upper => text.to_uppercase(),
        CaptionCase::Lower => text.to_lowercase(),
    }
}

use crate::{
    profile::{CaptionStyle, RenderProfile},
    timing::{FrameRate, centis_to_ass, millis_to_srt, millis_to_vtt},
};

/// The frames a cue occupies in the rendered program: `[first_frame,
/// end_frame)`. The render gate compares these against the decoded output, so
/// they are part of the plan rather than a comment about it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CueWindow {
    pub cue_id: String,
    pub first_frame: i64,
    pub end_frame: i64,
    pub text: String,
}

/// Characters that would be read as ASS markup. Captions are refused rather
/// than silently rewritten: a caption that does not say what the user typed is
/// worse than a render that explains why it stopped.
pub fn unrenderable_character(text: &str) -> Option<char> {
    text.chars()
        .find(|character| matches!(character, '{' | '}' | '\\') || character.is_control())
}

/// The frames the burned-in cues occupy, which is what the render gate compares
/// against decoded output — so it follows the burn-in list, not the reading one.
pub(crate) fn cue_windows(track: &CaptionTrack, rate: FrameRate) -> Vec<CueWindow> {
    track
        .burned()
        .iter()
        .map(|cue| CueWindow {
            cue_id: cue.cue_id.clone(),
            first_frame: rate.frame_ceil(cue.start_ticks),
            end_frame: rate.frame_ceil(cue.end_ticks),
            text: burned_text(&plain_text(cue, " "), track.options.text_case),
        })
        .collect()
}

/// The cue's words as a reader sees them, lines joined by `separator`.
fn plain_text(cue: &CaptionCue, separator: &str) -> String {
    cue.lines
        .iter()
        .map(|line| {
            line.words
                .iter()
                .map(|word| word.text.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join(separator)
}

pub(crate) fn write_ass(track: &CaptionTrack, profile: &RenderProfile) -> String {
    let rate = profile.rate();
    let style = &profile.caption_style;
    let (play_x, play_y) = crate::profile::design_resolution(profile.width, profile.height);
    let mut lines = vec![
        "[Script Info]".to_owned(),
        "ScriptType: v4.00+".to_owned(),
        // Styles are authored at the design height and libass scales them to
        // whatever the output is, so every output size keeps one proportion
        // and a preview at proxy resolution scales by one factor.
        format!("PlayResX: {play_x}"),
        format!("PlayResY: {play_y}"),
        // 2: no automatic wrapping. The document already decided the breaks.
        "WrapStyle: 2".to_owned(),
        "ScaledBorderAndShadow: yes".to_owned(),
        "YCbCr Matrix: TV.709".to_owned(),
        String::new(),
        "[V4+ Styles]".to_owned(),
        "Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, \
         BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, \
         BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding"
            .to_owned(),
    ];
    lines.extend(
        [
            CaptionRegion::LowerSafe,
            CaptionRegion::UpperSafe,
            CaptionRegion::Center,
        ]
        .into_iter()
        .map(|region| style_line(style, region)),
    );
    lines.push(String::new());
    lines.push("[Events]".to_owned());
    lines.push(
        "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text"
            .to_owned(),
    );
    for cue in track.burned() {
        let start_centis = rate.frame_centis(rate.frame_ceil(cue.start_ticks));
        let end_centis = rate.frame_centis(rate.frame_ceil(cue.end_ticks));
        lines.push(format!(
            "Dialogue: 0,{},{},{},,0,0,0,,{}",
            centis_to_ass(start_centis),
            centis_to_ass(end_centis),
            region_style_name(cue.region),
            dialogue_text(
                cue,
                rate,
                start_centis,
                end_centis,
                track.options.text_case,
                if highlight_enabled(cue, track.options.highlight_spoken_word) {
                    Plain::Swept
                } else {
                    Plain::Words(style.unspoken)
                },
            ),
        ));
    }
    lines.push(String::new());
    lines.join("\n")
}

pub(crate) fn highlight_enabled(cue: &CaptionCue, override_value: Option<bool>) -> bool {
    override_value.unwrap_or(matches!(cue.anim, CaptionAnimation::Karaoke))
}

/// Where a cue's highlight is at every moment it is on screen.
///
/// The sweep is computed **once, here**, and both the burned-in track and the
/// preview plan read it. That is the whole reason it is a function rather than
/// a loop inside the ASS writer: a preview that computed its own karaoke timing
/// would be a second implementation of the same arithmetic, and two
/// implementations of the same arithmetic are two answers waiting to differ on
/// a frame nobody checked.
///
/// Durations are centiseconds from the dialogue's own start, and each word
/// holds the highlight until the next word begins, with a short floor when
/// several guessed starts collapse on one frame. Holds still sum to the cue's
/// length with no accumulated drift.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sweep {
    /// Before the first word is sung. Zero when speech starts with the cue.
    pub lead_in_centis: i64,
    /// How long each word holds the highlight, in cue order across all lines.
    pub holds_centis: Vec<i64>,
}

pub(crate) fn sweep(
    cue: &CaptionCue,
    rate: FrameRate,
    start_centis: i64,
    end_centis: i64,
) -> Sweep {
    let words = cue.words().collect::<Vec<_>>();
    if words.is_empty() {
        return Sweep {
            lead_in_centis: 0,
            holds_centis: Vec::new(),
        };
    }
    let total = (end_centis - start_centis).max(0);
    // An aligner may put several words on one tick. Reserve about two output
    // frames for each word by borrowing from the long holds around that run.
    // When the cue is too short to fit that floor, divide its available time
    // evenly rather than creating an impossible boundary beyond its end.
    let two_frames = (rate.frame_centis(2) - rate.frame_centis(0)).max(1);
    let floor = two_frames.min(total / i64::try_from(words.len()).unwrap_or(1).max(1));
    let mut starts = Vec::with_capacity(words.len());
    for (index, word) in words.iter().enumerate() {
        let desired =
            (rate.frame_centis(rate.frame_ceil(word.start_ticks)) - start_centis).clamp(0, total);
        let lower = starts
            .last()
            .copied()
            .map_or(0, |previous| previous + floor);
        let remaining = i64::try_from(words.len() - index).unwrap_or(1);
        let upper = total.saturating_sub(remaining.saturating_mul(floor));
        starts.push(desired.clamp(lower, upper));
    }
    let holds = starts
        .iter()
        .enumerate()
        .map(|(index, &start)| {
            let end = starts.get(index + 1).copied().unwrap_or(total);
            (end - start).max(0)
        })
        .collect();
    Sweep {
        lead_in_centis: starts[0],
        holds_centis: holds,
    }
}

/// How a cue's words are coloured: swept word by word as they are spoken, or
/// all in the one colour the look calls its words.
#[derive(Clone, Copy)]
enum Plain {
    Swept,
    Words(crate::profile::Colour),
}

/// A cue's text with karaoke timing, when the cue asks for it.
///
/// Without a sweep libass draws text in the style's primary colour, which is
/// the *spoken* colour — so a cue with no highlight would take the highlight
/// colour and ignore the words colour. The words colour is set on the line
/// instead, and the two swatches mean the same thing whatever the look.
fn dialogue_text(
    cue: &CaptionCue,
    rate: FrameRate,
    start_centis: i64,
    end_centis: i64,
    text_case: CaptionCase,
    colouring: Plain,
) -> String {
    if let Plain::Words(colour) = colouring {
        let text = cue
            .lines
            .iter()
            .map(|line| {
                line.words
                    .iter()
                    .map(|word| burned_text(&word.text, text_case))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>()
            .join("\\N");
        return format!("{{\\1c{}}}{text}", colour.to_ass_override());
    }
    let swept = sweep(cue, rate, start_centis, end_centis);
    let mut pieces = Vec::new();
    if swept.lead_in_centis > 0 {
        pieces.push(format!("{{\\k{}}}", swept.lead_in_centis));
    }
    let mut index = 0_usize;
    for (line_index, line) in cue.lines.iter().enumerate() {
        if line_index > 0 {
            pieces.push("\\N".to_owned());
        }
        for (word_index, word) in line.words.iter().enumerate() {
            if word_index > 0 {
                pieces.push(" ".to_owned());
            }
            let hold = swept.holds_centis.get(index).copied().unwrap_or(0);
            pieces.push(format!(
                "{{\\k{hold}}}{}",
                burned_text(&word.text, text_case)
            ));
            index += 1;
        }
    }
    pieces.concat()
}

fn region_style_name(region: CaptionRegion) -> &'static str {
    match region {
        CaptionRegion::LowerSafe => "lower_safe",
        CaptionRegion::UpperSafe => "upper_safe",
        CaptionRegion::Center => "center",
    }
}

fn region_alignment(region: CaptionRegion) -> u32 {
    match region {
        CaptionRegion::LowerSafe => 2,
        CaptionRegion::Center => 5,
        CaptionRegion::UpperSafe => 8,
    }
}

fn style_line(style: &CaptionStyle, region: CaptionRegion) -> String {
    format!(
        "Style: {name},{font},{size},{spoken},{unspoken},{outline},{shadow},{bold},0,0,0,\
         100,100,0,0,{border},{outline_width},{shadow_depth},{alignment},{margin_h},{margin_h},\
         {margin_v},1",
        name = region_style_name(region),
        // 1 draws an outline and a drop shadow; 3 fills an opaque plate behind
        // the line, using the outline colour as the plate.
        border = if style.boxed { 3 } else { 1 },
        font = style.font_family,
        size = style.font_size,
        // With `\k`, text is drawn in the secondary colour until it is sung
        // and in the primary colour afterwards.
        spoken = style.spoken.to_ass(),
        unspoken = style.unspoken.to_ass(),
        outline = style.outline.to_ass(),
        shadow = style.shadow.to_ass(),
        bold = i32::from(style.bold),
        outline_width = style.outline_width,
        shadow_depth = style.shadow_depth,
        alignment = region_alignment(region),
        margin_h = style.margin_horizontal,
        margin_v = if matches!(region, CaptionRegion::Center) {
            0
        } else {
            style.margin_vertical
        },
    )
}

pub(crate) fn write_srt(track: &CaptionTrack, rate: FrameRate) -> String {
    track
        .cues
        .iter()
        .enumerate()
        .map(|(ordinal, cue)| {
            format!(
                "{}\n{} --> {}\n{}\n\n",
                ordinal + 1,
                millis_to_srt(rate.frame_millis(rate.frame_ceil(cue.start_ticks))),
                millis_to_srt(rate.frame_millis(rate.frame_ceil(cue.end_ticks))),
                plain_text(cue, "\n"),
            )
        })
        .collect::<Vec<_>>()
        .concat()
}

pub(crate) fn write_vtt(track: &CaptionTrack, rate: FrameRate) -> String {
    let cues = track.cues.iter().map(|cue| {
        format!(
            "{}\n{} --> {}\n{}\n\n",
            cue.cue_id,
            millis_to_vtt(rate.frame_millis(rate.frame_ceil(cue.start_ticks))),
            millis_to_vtt(rate.frame_millis(rate.frame_ceil(cue.end_ticks))),
            escape_vtt(&plain_text(cue, "\n")),
        )
    });
    std::iter::once("WEBVTT\n\n".to_owned())
        .chain(cues)
        .collect()
}

/// WebVTT payloads are parsed as markup, so the three characters that would
/// open a cue span are escaped.
fn escape_vtt(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use clipmill_edit_ir::{
        CaptionAnimation, CaptionCue, CaptionLine, CaptionRegion, CaptionTrack, CaptionWord,
    };

    use super::{cue_windows, sweep, unrenderable_character, write_ass, write_srt, write_vtt};
    use crate::{profile::RenderProfile, timing::FrameRate};

    const FRAME_TICKS: i64 = 3_003;
    const RATE: FrameRate = FrameRate::NTSC_30;

    fn word(text: &str, start_frame: i64, end_frame: i64) -> CaptionWord {
        CaptionWord {
            text: text.to_owned(),
            start_ticks: start_frame * FRAME_TICKS,
            end_ticks: end_frame * FRAME_TICKS,
            word_id: None,
        }
    }

    fn track() -> CaptionTrack {
        CaptionTrack {
            style_ref: crate::profile::DEFAULT_STYLE_REF.to_owned(),
            options: clipmill_edit_ir::CaptionOptions::default(),
            cues: vec![CaptionCue {
                cue_id: "cue_1".to_owned(),
                start_ticks: 30 * FRAME_TICKS,
                end_ticks: 90 * FRAME_TICKS,
                region: CaptionRegion::LowerSafe,
                anim: CaptionAnimation::Karaoke,
                lines: vec![
                    CaptionLine {
                        words: vec![word("the", 30, 40), word("whole", 40, 55)],
                    },
                    CaptionLine {
                        words: vec![word("point", 60, 75)],
                    },
                ],
            }],
            burn_in: Vec::new(),
        }
    }

    #[test]
    fn karaoke_durations_sum_to_the_cue_length() {
        let ass = write_ass(&track(), &RenderProfile::default());
        let dialogue = ass
            .lines()
            .find(|line| line.starts_with("Dialogue:"))
            .expect("a dialogue line");
        let total: i64 = dialogue
            .split("{\\k")
            .skip(1)
            .filter_map(|chunk| chunk.split('}').next())
            .filter_map(|value| value.parse::<i64>().ok())
            .sum();
        let start = RATE.frame_centis(30);
        let end = RATE.frame_centis(90);
        assert_eq!(
            total,
            end - start,
            "karaoke sweep must span exactly the cue: {dialogue}"
        );
    }

    #[test]
    fn a_word_holds_the_highlight_until_the_next_one_starts() {
        let ass = write_ass(&track(), &RenderProfile::default());
        let dialogue = ass
            .lines()
            .find(|line| line.starts_with("Dialogue:"))
            .expect("a dialogue line");
        // "whole" starts at frame 40 and "point" at frame 60, so the sweep
        // must hold "whole" across the 5-frame gap after it ends.
        let expected = RATE.frame_centis(60) - RATE.frame_centis(40);
        assert!(
            dialogue.contains(&format!("{{\\k{expected}}}whole")),
            "expected whole to hold for {expected} centiseconds: {dialogue}"
        );
    }

    #[test]
    fn collapsed_word_starts_each_get_visible_highlight_time() {
        let mut caption = track().cues[0].clone();
        caption.start_ticks = 0;
        caption.end_ticks = 90 * FRAME_TICKS;
        for word in caption.lines.iter_mut().flat_map(|line| &mut line.words) {
            word.start_ticks = 30 * FRAME_TICKS;
            word.end_ticks = 30 * FRAME_TICKS;
        }
        let measured = sweep(&caption, RATE, 0, RATE.frame_centis(90));
        let floor = RATE.frame_centis(2) - RATE.frame_centis(0);
        assert!(
            measured.holds_centis.iter().all(|hold| *hold >= floor),
            "{measured:?}"
        );
        assert_eq!(
            measured.lead_in_centis + measured.holds_centis.iter().sum::<i64>(),
            RATE.frame_centis(90)
        );
    }

    #[test]
    fn stored_line_breaks_survive_into_every_output() {
        let profile = RenderProfile::default();
        let ass = write_ass(&track(), &profile);
        assert!(ass.contains("WrapStyle: 2"), "libass must not re-wrap");
        assert!(ass.contains("\\Npoint") || ass.contains("\\N{\\k"));
        let srt = write_srt(&track(), RATE);
        assert!(srt.contains("the whole\npoint"));
        let vtt = write_vtt(&track(), RATE);
        assert!(vtt.contains("the whole\npoint"));
    }

    /// The same words, grouped kinetically: three one-word cues.
    fn kinetic() -> Vec<CaptionCue> {
        [("the", 30, 40), ("whole", 40, 55), ("point", 60, 75)]
            .into_iter()
            .enumerate()
            .map(|(index, (text, from, to))| CaptionCue {
                cue_id: format!("hot_{}", index + 1),
                start_ticks: from * FRAME_TICKS,
                end_ticks: to * FRAME_TICKS,
                region: CaptionRegion::LowerSafe,
                anim: CaptionAnimation::Karaoke,
                lines: vec![CaptionLine {
                    words: vec![word(text, from, to)],
                }],
            })
            .collect()
    }

    #[test]
    fn the_burn_in_takes_the_kinetic_grouping_and_the_sidecars_never_do() {
        // The asymmetry the second track exists for. A burn-in that fell back
        // to the reading cues is merely conservative; a sidecar that picked up
        // the kinetic ones is the divergence between what a viewer reads and
        // what a deaf viewer reads.
        let mut both = track();
        both.burn_in = kinetic();

        let ass = write_ass(&both, &RenderProfile::default());
        let srt = write_srt(&both, RATE);
        let vtt = write_vtt(&both, RATE);

        assert_eq!(
            ass.lines()
                .filter(|line| line.starts_with("Dialogue:"))
                .count(),
            3,
            "the picture gets the kinetic cues"
        );
        assert!(
            srt.contains("the whole\npoint"),
            "the reader gets the reading cues"
        );
        assert!(vtt.contains("the whole\npoint"));
        assert!(!srt.contains("hot_"), "no sidecar may carry a kinetic cue");
    }

    #[test]
    fn without_a_kinetic_grouping_the_reading_cues_are_what_gets_burned_in() {
        // Every document written before the second track existed behaves this
        // way, and must keep behaving this way.
        let ass = write_ass(&track(), &RenderProfile::default());
        assert_eq!(
            ass.lines()
                .filter(|line| line.starts_with("Dialogue:"))
                .count(),
            1
        );
        assert_eq!(cue_windows(&track(), RATE).len(), 1);
    }

    #[test]
    fn the_gate_measures_the_frames_that_are_actually_burned_in() {
        let mut both = track();
        both.burn_in = kinetic();
        let windows = cue_windows(&both, RATE);
        assert_eq!(windows.len(), 3, "the render gate follows the picture");
        assert_eq!(windows[0].text, "the");
    }

    #[test]
    fn sidecars_carry_no_karaoke_markup() {
        let srt = write_srt(&track(), RATE);
        let vtt = write_vtt(&track(), RATE);
        for sidecar in [&srt, &vtt] {
            assert!(!sidecar.contains("\\k"), "sidecars are the reading profile");
        }
        assert!(srt.starts_with("1\n00:00:01,001 --> 00:00:03,003\n"));
        assert!(vtt.starts_with("WEBVTT\n\ncue_1\n00:00:01.001 --> 00:00:03.003\n"));
    }

    #[test]
    fn cue_windows_name_the_frames_the_gate_checks() {
        let windows = cue_windows(&track(), RATE);
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].first_frame, 30);
        assert_eq!(windows[0].end_frame, 90);
        assert_eq!(windows[0].text, "the whole point");
    }

    #[test]
    fn markup_characters_are_reported_rather_than_rewritten() {
        assert_eq!(unrenderable_character("plain words"), None);
        assert_eq!(unrenderable_character("an {override}"), Some('{'));
        assert_eq!(unrenderable_character("a\\break"), Some('\\'));
        assert_eq!(unrenderable_character("line\nbreak"), Some('\n'));
    }

    #[test]
    fn webvtt_payloads_escape_their_markup() {
        let mut cues = track();
        cues.cues[0].lines[0].words[0].text = "a<b&c".to_owned();
        let vtt = write_vtt(&cues, RATE);
        assert!(vtt.contains("a&lt;b&amp;c"));
    }

    #[test]
    fn every_region_has_a_style_with_its_own_anchor() {
        let ass = write_ass(&track(), &RenderProfile::default());
        assert!(ass.contains("Style: lower_safe,Inter,84,"));
        assert!(ass.contains("Style: upper_safe,"));
        assert!(ass.contains("Style: center,"));
        // Bottom-anchored, top-anchored, middle-anchored.
        assert!(ass.contains(",2,90,90,260,1"));
        assert!(ass.contains(",8,90,90,260,1"));
        assert!(ass.contains(",5,90,90,0,1"));
    }
}
