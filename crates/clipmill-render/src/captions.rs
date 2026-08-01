//! Where the caption document becomes something the writers can render.
//!
//! The caption engine publishes one document holding two groupings of one token
//! array. This is the only place either of them turns into an Edit IR caption
//! track, and it is deliberately a projection rather than a derivation: no
//! decision is taken here that was not already taken upstream. Line breaks
//! arrive decided, cue windows arrive decided, and what this module does is
//! choose an intent, shift source time into program time, and copy.
//!
//! That matters because the alternative is the failure the book names by name.
//! If the renderer could re-segment, a preview and an export could disagree
//! about where a line broke; if it could re-group, a burn-in and a sidecar
//! could disagree about which words were said. Neither is reachable from here.
//!
//! Which intent a surface takes is not a preference. The burned-in track may
//! take the kinetic grouping. Every sidecar takes the accessibility grouping,
//! always, because a sidecar is what a deaf viewer is left with.

use clipmill_captions::{
    Animation, Border, Preset,
    presets::{self, Colour as PresetColour},
};
use clipmill_contracts::schemas::captions_cues::{
    CaptionCues, CorrectionOrigin, Cue, CueRegion, Intent as DocumentIntent,
};
use clipmill_edit_ir::{
    CaptionAnimation, CaptionCue, CaptionLine, CaptionRegion, CaptionTrack, CaptionWord,
};
use thiserror::Error;

use crate::profile::{CaptionStyle, Colour};

/// Which of the document's two groupings a surface is rendering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Intent {
    /// The conservative grouping. Every sidecar, without exception.
    Accessibility,
    /// The kinetic grouping, for the burned-in track only.
    BurnIn,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ProjectionError {
    #[error("caption style {0} names a preset that does not exist")]
    UnknownStyle(String),
    #[error("cue {0} names tokens the document does not contain")]
    DanglingCue(String),
    #[error("cue {0} has a line that names tokens outside it")]
    DanglingLine(String),
}

/// Turn one intent of a caption document into an Edit IR track.
///
/// `offset_ticks` is subtracted from every time, which is how a document in
/// source time becomes a track in program time. A cue that would land before
/// the program starts is dropped rather than clamped: a caption pinned to zero
/// because its words were cut away is a caption that says the wrong thing at
/// the wrong moment.
pub fn project(
    document: &CaptionCues,
    intent: Intent,
    style_ref: &str,
    offset_ticks: i64,
) -> Result<CaptionTrack, ProjectionError> {
    let preset = presets::preset(style_ref)
        .ok_or_else(|| ProjectionError::UnknownStyle(style_ref.to_owned()))?;
    let grouping: &DocumentIntent = match intent {
        Intent::Accessibility => &document.intents.accessibility,
        Intent::BurnIn => &document.intents.burn_in,
    };
    // A still preset renders every cue without a sweep, whichever grouping it
    // came from. The animation belongs to the look, not to the words.
    let animation = match preset.animation {
        Animation::None => CaptionAnimation::None,
        Animation::Karaoke => CaptionAnimation::Karaoke,
    };

    let mut cues = Vec::with_capacity(grouping.cues.len());
    for cue in &grouping.cues {
        let start = as_i64(cue.start_ticks) - offset_ticks;
        let end = as_i64(cue.end_ticks.get()) - offset_ticks;
        if end <= 0 {
            continue;
        }
        cues.push(CaptionCue {
            cue_id: cue.cue_id.to_string(),
            start_ticks: start.max(0),
            end_ticks: end,
            region: region_of(cue),
            anim: animation,
            lines: lines_of(document, cue, offset_ticks)?,
        });
    }
    Ok(CaptionTrack {
        style_ref: preset.style_ref.to_owned(),
        cues,
    })
}

fn region_of(cue: &Cue) -> CaptionRegion {
    match cue.region {
        CueRegion::LowerSafe => CaptionRegion::LowerSafe,
        CueRegion::UpperSafe => CaptionRegion::UpperSafe,
        CueRegion::Center => CaptionRegion::Center,
    }
}

/// The cue's lines, with the words the document says are on each.
///
/// The ranges are checked rather than trusted. A line naming a token outside
/// its cue would render text from a caption the viewer is not looking at, and
/// that is worth a refusal rather than a truncation.
fn lines_of(
    document: &CaptionCues,
    cue: &Cue,
    offset_ticks: i64,
) -> Result<Vec<CaptionLine>, ProjectionError> {
    let cue_first = as_usize(cue.first_token);
    let cue_end = cue_first + as_usize(cue.token_count.get());
    if cue_end > document.tokens.len() {
        return Err(ProjectionError::DanglingCue(cue.cue_id.to_string()));
    }

    let mut lines = Vec::with_capacity(cue.lines.len());
    for line in &cue.lines {
        let first = as_usize(line.first_token);
        let end = first + as_usize(line.token_count.get());
        if first < cue_first || end > cue_end {
            return Err(ProjectionError::DanglingLine(cue.cue_id.to_string()));
        }
        let words = document.tokens[first..end]
            .iter()
            .map(|token| CaptionWord {
                text: apply_corrections(document, token.index, token.text.to_string()),
                start_ticks: (as_i64(token.start_ticks) - offset_ticks).max(0),
                end_ticks: (as_i64(token.end_ticks.get()) - offset_ticks).max(1),
            })
            .collect();
        lines.push(CaptionLine { words });
    }
    Ok(lines)
}

/// The overlay, applied on top of what the recogniser said.
///
/// A user's correction outranks a re-transcription's for the same token. That
/// ordering is the whole point of keeping corrections as an overlay: a better
/// model may propose, and it may not erase a word somebody already fixed.
fn apply_corrections(document: &CaptionCues, token_index: u64, raw: String) -> String {
    let mut chosen: Option<&str> = None;
    let mut from_user = false;
    for correction in &document.corrections {
        if correction.token_index != token_index {
            continue;
        }
        let is_user = matches!(correction.origin, CorrectionOrigin::User);
        if is_user || !from_user {
            chosen = Some(correction.text.as_str());
            from_user = from_user || is_user;
        }
    }
    chosen.map_or(raw, str::to_owned)
}

impl CaptionStyle {
    /// The render's style for a caption preset.
    ///
    /// The two types are separate on purpose: the preset is what a person
    /// chooses and the style is what libass is told, and the alpha conventions
    /// alone are reason enough not to pretend they are the same struct.
    pub fn from_preset(preset: &Preset) -> Self {
        Self {
            style_ref: preset.style_ref.to_owned(),
            font_family: preset.font_family.to_owned(),
            font_size: preset.font_size,
            spoken: colour(preset.spoken),
            unspoken: colour(preset.unspoken),
            outline: colour(preset.outline),
            shadow: colour(preset.shadow),
            outline_width: preset.outline_width,
            shadow_depth: preset.shadow_depth,
            bold: preset.bold,
            boxed: matches!(preset.border, Border::Box),
            margin_horizontal: preset.margin_horizontal,
            margin_vertical: preset.margin_vertical,
        }
    }
}

/// A preset colour as ASS sees it. ASS alpha runs the other way: zero is
/// opaque, which is exactly the sort of detail that has to be converted in one
/// place or be wrong in several.
fn colour(from: PresetColour) -> Colour {
    Colour {
        red: from.red,
        green: from.green,
        blue: from.blue,
        transparency: 255 - from.alpha,
    }
}

fn as_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn as_usize(value: u64) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests;
