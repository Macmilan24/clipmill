//! Text laid over the program, drawn by libass in the captions' own pass.
//!
//! An overlay is set in the clip's caption font, so the render stages no
//! other face, and it is timed on the same frame grid as a cue. Its position
//! is its centre, as a share of the frame, so it sits in the same place at
//! every output size and in every shape.

use clipmill_edit_ir::{Overlay, OverlayContent};

use crate::{
    profile::{CaptionStyle, Colour},
    timing::{FrameRate, centis_to_ass},
};

/// The style of a text drawn with an outline.
const OUTLINED: &str = "text_outline";
/// The style of a text on an opaque plate: libass fills a box behind each
/// line in the outline colour.
const PLATED: &str = "text_plate";
/// Overlays draw above every caption layer.
const LAYER: u32 = 10;

/// The two styles text overlays draw with, in the caption font.
pub(crate) fn style_lines(style: &CaptionStyle) -> [String; 2] {
    let line = |name: &str, border: u32| {
        format!(
            "Style: {name},{font},72,&H00FFFFFF,&H00FFFFFF,&H00000000,&H00000000,1,0,0,0,\
             100,100,0,0,{border},4,0,5,0,0,0,1",
            font = style.font_family,
        )
    };
    [line(OUTLINED, 1), line(PLATED, 3)]
}

/// One dialogue line per overlay, in stacking order, on the frame grid.
pub(crate) fn dialogues(overlays: &[Overlay], rate: FrameRate, play: (i64, i64)) -> Vec<String> {
    overlays
        .iter()
        .filter_map(|overlay| {
            let OverlayContent::Text {
                text,
                x,
                y,
                size,
                colour,
                plate,
                ..
            } = &overlay.content
            else {
                return None;
            };
            let colour = Colour::from_hex(colour).unwrap_or(Colour::opaque(0xFF, 0xFF, 0xFF));
            let plate = plate.as_deref().and_then(Colour::from_hex);
            let (style, border, edge) = match plate {
                // A plate is padded around the words, in the plate's colour.
                Some(plate) => (PLATED, (u32::from(*size) / 5).max(8), plate),
                None => (
                    OUTLINED,
                    (u32::from(*size) / 16).max(2),
                    Colour::opaque(0, 0, 0),
                ),
            };
            Some(format!(
                "Dialogue: {LAYER},{start},{end},{style},,0,0,0,,\
                 {{\\an5\\pos({px},{py})\\fs{size}\\c{fill}\\3c{edge}\\bord{border}\\shad0}}{text}",
                start = centis_to_ass(rate.frame_centis(rate.frame_ceil(overlay.start_ticks))),
                end = centis_to_ass(rate.frame_centis(rate.frame_ceil(overlay.end_ticks))),
                px = play.0 * i64::from(*x) / 1_000,
                py = play.1 * i64::from(*y) / 1_000,
                fill = colour.to_ass_override(),
                edge = edge.to_ass_override(),
                text = text.replace('\n', "\\N"),
            ))
        })
        .collect()
}

/// Where the emoji a clip uses are staged, beside the captions.
pub const EMOJI_DIR: &str = "emoji";

/// Each emoji over the picture, from its staged picture, at its size and
/// place, on the frames its span covers.
pub(crate) fn emoji_chains(
    overlays: &[Overlay],
    rate: FrameRate,
    (width, height): (i64, i64),
    input: &str,
) -> Result<(Vec<String>, String), crate::RenderError> {
    let mut chains = Vec::new();
    let mut picture = input.to_owned();
    for (index, overlay) in overlays.iter().enumerate() {
        let OverlayContent::Emoji { emoji, x, y, size } = &overlay.content else {
            continue;
        };
        let found = clipmill_captions::emoji(emoji)
            .ok_or_else(|| crate::RenderError::UnknownEmoji(emoji.clone()))?;
        let side = ((width.min(height) * i64::from(*size) / 1_000) & !1).max(2);
        let (first, end) = (
            rate.frame_ceil(overlay.start_ticks),
            rate.frame_ceil(overlay.end_ticks),
        );
        let label = format!("[vemoji{index}]");
        chains.push(format!(
            "movie=filename={EMOJI_DIR}/{file},format=rgba,scale={side}:{side}[emoji{index}]",
            file = found.file(),
        ));
        chains.push(format!(
            "{picture}[emoji{index}]overlay=x={cx}-w/2:y={cy}-h/2:eof_action=repeat:\
             enable='between(n\\,{first}\\,{last})',format=yuv420p{label}",
            cx = width * i64::from(*x) / 1_000,
            cy = height * i64::from(*y) / 1_000,
            last = (end - 1).max(first),
        ));
        picture = label;
    }
    Ok((chains, picture))
}

/// The emoji a document shows, once each, for the render to stage.
pub fn emoji_files(overlays: &[Overlay]) -> Vec<String> {
    let mut files: Vec<String> = overlays
        .iter()
        .filter_map(|overlay| match &overlay.content {
            OverlayContent::Emoji { emoji, .. } => {
                clipmill_captions::emoji(emoji).map(clipmill_captions::Emoji::file)
            }
            OverlayContent::Text { .. } => None,
        })
        .collect();
    files.sort();
    files.dedup();
    files
}
