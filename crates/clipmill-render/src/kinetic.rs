//! Burned-in caption events for every way of marking the spoken word.
//!
//! One cue becomes one or more ASS dialogue events. With no highlight, or the
//! classic sweep, it is one event: the sweep is libass's own karaoke, which
//! paints each word as it is sung. The other marks — only the current word
//! coloured, a box behind it, a pop, an underline — cannot be said in one
//! event, so the cue is cut at each word's start into consecutive events that
//! carry the same text with a different word marked. Each event is a still
//! arrangement of the one line layout, which is why the words never move
//! except where a mark scales one on purpose.
//!
//! The box needs three layers, because libass draws an opaque box around a
//! run of text rather than behind a word: the look as it is, then the box
//! around only the marked word (every other run transparent), then the marked
//! word's text again on top of its box. All three set the same text in the
//! same font at the same size, so their glyphs land on the same pixels.
//!
//! Timing comes from [`crate::subtitles::sweep`], the one computation both
//! the burned-in track and the preview read.

use std::fmt::Write as _;

use clipmill_edit_ir::{CaptionCase, CaptionCue, CaptionPosition, CaptionRegion, HighlightStyle};

use crate::{
    profile::{CaptionStyle, Colour},
    subtitles::{burned_text, sweep},
    timing::{FrameRate, centis_to_ass},
};

/// Everything about a cue's surroundings its events need.
pub(crate) struct CueContext<'a> {
    pub style: &'a CaptionStyle,
    pub rate: FrameRate,
    pub text_case: CaptionCase,
    /// Where every cue sits unless one was placed on its own.
    pub position: Option<CaptionPosition>,
    /// The script resolution positions are stated against.
    pub play: (i64, i64),
}

/// The name of a region's style, and of its two helpers for the box mark.
pub(crate) fn region_style_name(region: CaptionRegion) -> &'static str {
    match region {
        CaptionRegion::LowerSafe => "lower_safe",
        CaptionRegion::UpperSafe => "upper_safe",
        CaptionRegion::Center => "center",
    }
}

/// The style of the box drawn behind a marked word.
pub(crate) const MARK_SUFFIX: &str = "_mark";
/// The style the marked word is redrawn in, on top of its box.
pub(crate) const WORD_SUFFIX: &str = "_word";

/// Every dialogue line one cue contributes, in drawing order.
pub(crate) fn cue_dialogues(
    cue: &CaptionCue,
    highlighted: bool,
    context: &CueContext<'_>,
) -> Vec<String> {
    let rate = context.rate;
    let start_centis = rate.frame_centis(rate.frame_ceil(cue.start_ticks));
    let end_centis = rate.frame_centis(rate.frame_ceil(cue.end_ticks));
    let region = region_style_name(cue.region);
    let place = position_tag(cue.position.or(context.position), context.play);
    let event = |layer: u32, from: i64, to: i64, style: &str, text: String| {
        format!(
            "Dialogue: {layer},{},{},{style},,0,0,0,,{place}{text}",
            centis_to_ass(from),
            centis_to_ass(to),
        )
    };
    if !highlighted {
        return vec![event(
            0,
            start_centis,
            end_centis,
            region,
            words_text(cue, context, None),
        )];
    }
    let swept = sweep(cue, rate, start_centis, end_centis);
    if context.style.highlight == HighlightStyle::Fill {
        return vec![event(
            0,
            start_centis,
            end_centis,
            region,
            karaoke_text(cue, context, &swept),
        )];
    }

    let mut lines = Vec::new();
    if swept.lead_in_centis > 0 {
        lines.push(event(
            0,
            start_centis,
            start_centis + swept.lead_in_centis,
            region,
            words_text(cue, context, None),
        ));
    }
    let mut at = start_centis + swept.lead_in_centis;
    for (index, hold) in swept.holds_centis.iter().enumerate() {
        let (from, to) = (at, at + hold);
        at = to;
        if to <= from {
            continue;
        }
        if context.style.highlight == HighlightStyle::Box {
            lines.push(event(0, from, to, region, words_text(cue, context, None)));
            lines.push(event(
                1,
                from,
                to,
                &format!("{region}{MARK_SUFFIX}"),
                only_word(cue, context, index, Visible::Box),
            ));
            lines.push(event(
                2,
                from,
                to,
                &format!("{region}{WORD_SUFFIX}"),
                only_word(cue, context, index, Visible::Text),
            ));
        } else {
            lines.push(event(
                0,
                from,
                to,
                region,
                words_text(cue, context, Some(index)),
            ));
        }
    }
    lines
}

/// `{\an5\pos(x,y)}` for a placed cue: its centre at thousandths of the frame.
fn position_tag(position: Option<CaptionPosition>, play: (i64, i64)) -> String {
    position.map_or_else(String::new, |position| {
        let x = i64::from(position.x) * play.0 / i64::from(CaptionPosition::FULL);
        let y = i64::from(position.y) * play.1 / i64::from(CaptionPosition::FULL);
        format!("{{\\an5\\pos({x},{y})}}")
    })
}

/// The colour a word rests in: the accent for a key word, the words colour
/// otherwise.
fn resting(word_emphasis: bool, style: &CaptionStyle) -> Colour {
    if word_emphasis {
        style.accent
    } else {
        style.unspoken
    }
}

/// The cue's words with at most one of them marked in the style's way.
fn words_text(cue: &CaptionCue, context: &CueContext<'_>, marked: Option<usize>) -> String {
    let style = context.style;
    let mut text = format!("{{\\1c{}}}", style.unspoken.to_ass_override());
    let mut index = 0_usize;
    for (line_index, line) in cue.lines.iter().enumerate() {
        if line_index > 0 {
            text.push_str("\\N");
        }
        for (word_index, word) in line.words.iter().enumerate() {
            if word_index > 0 {
                text.push(' ');
            }
            let shown = burned_text(&word.text, context.text_case);
            let rest = resting(word.emphasis, style);
            if marked == Some(index) {
                text.push_str(&marked_word(&shown, rest, style));
            } else if word.emphasis {
                let _ = write!(
                    text,
                    "{{\\1c{}}}{shown}{{\\1c{}}}",
                    rest.to_ass_override(),
                    style.unspoken.to_ass_override()
                );
            } else {
                text.push_str(&shown);
            }
            index += 1;
        }
    }
    text
}

/// One word as the style marks the word being spoken.
fn marked_word(shown: &str, rest: Colour, style: &CaptionStyle) -> String {
    let spoken = style.spoken.to_ass_override();
    let back = style.unspoken.to_ass_override();
    match style.highlight {
        HighlightStyle::Underline => {
            format!("{{\\u1\\1c{spoken}}}{shown}{{\\u0\\1c{back}}}")
        }
        // A quick swell and settle, measured from the word's own start: the
        // event begins when the word does.
        HighlightStyle::Pop => format!(
            "{{\\1c{spoken}\\fscx100\\fscy100\\t(0,70,\\fscx118\\fscy118)\\t(70,160,\\fscx108\\fscy108)}}{shown}{{\\fscx100\\fscy100\\1c{back}}}"
        ),
        // The box's word keeps its own colour; the box is the mark.
        HighlightStyle::Box => format!("{{\\1c{}}}{shown}{{\\1c{back}}}", rest.to_ass_override()),
        HighlightStyle::Word | HighlightStyle::Fill => {
            format!("{{\\1c{spoken}}}{shown}{{\\1c{back}}}")
        }
    }
}

/// Which part of the marked word a box layer shows.
#[derive(Clone, Copy)]
enum Visible {
    /// The box around it, from the mark style's opaque border.
    Box,
    /// Its text, redrawn over the box.
    Text,
}

/// The cue's text with every run invisible except one word's box or text.
fn only_word(
    cue: &CaptionCue,
    context: &CueContext<'_>,
    marked: usize,
    visible: Visible,
) -> String {
    const HIDDEN: &str = "\\1a&HFF&\\3a&HFF&\\4a&HFF&";
    let shown_tags = match visible {
        Visible::Box => "\\3a&H00&".to_owned(),
        Visible::Text => "\\1a&H00&\\3a&H00&\\4a&H00&".to_owned(),
    };
    let mut text = format!("{{{HIDDEN}}}");
    let mut index = 0_usize;
    for (line_index, line) in cue.lines.iter().enumerate() {
        if line_index > 0 {
            text.push_str("\\N");
        }
        for (word_index, word) in line.words.iter().enumerate() {
            if word_index > 0 {
                text.push(' ');
            }
            let shown = burned_text(&word.text, context.text_case);
            if index == marked {
                let colour = resting(word.emphasis, context.style).to_ass_override();
                let _ = write!(text, "{{{shown_tags}\\1c{colour}}}{shown}{{{HIDDEN}}}");
            } else {
                text.push_str(&shown);
            }
            index += 1;
        }
    }
    text
}

/// The classic sweep: libass paints each word as it is sung and it stays lit.
fn karaoke_text(
    cue: &CaptionCue,
    context: &CueContext<'_>,
    swept: &crate::subtitles::Sweep,
) -> String {
    let style = context.style;
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
            let shown = burned_text(&word.text, context.text_case);
            if word.emphasis {
                // A key word holds its accent through the sweep.
                let accent = style.accent.to_ass_override();
                pieces.push(format!(
                    "{{\\k{hold}\\1c{accent}\\2c{accent}}}{shown}{{\\1c{}\\2c{}}}",
                    style.spoken.to_ass_override(),
                    style.unspoken.to_ass_override(),
                ));
            } else {
                pieces.push(format!("{{\\k{hold}}}{shown}"));
            }
            index += 1;
        }
    }
    pieces.concat()
}
