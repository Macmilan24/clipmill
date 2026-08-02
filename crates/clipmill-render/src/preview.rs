//! What the editor's player must draw, computed by the thing that renders.
//!
//! The binding decision of this workstream is that there is **one interpreter**.
//! A preview is a claim about what the export will look like, and a claim like
//! that is worth nothing if the preview arrived at it independently — two
//! implementations of the same arithmetic are two answers waiting to differ on
//! a frame nobody checked, and the frame nobody checked is the one a creator
//! ships.
//!
//! So this module does no timing math of its own. The crop at a frame comes
//! from [`crate::crop_rect_at`], the frame a cue begins on comes from the same
//! `frame_ceil` the ASS writer uses, and the karaoke sweep comes from
//! [`crate::subtitles::sweep`] — the function the burned-in track is written
//! from. What is left here is sampling: walking the frames and asking.
//!
//! The renderer draws with libass and the player draws with the DOM, so pixels
//! will differ — antialiasing, hinting, subpixel positioning. That is expected
//! and is documented as a tolerance. What may never differ is **semantics**: a
//! different word, a different crop rectangle, a different frame for a cue.
//! `gate-editor` renders fixture documents and compares.

use clipmill_edit_ir::{CaptionAnimation, CaptionCue, CaptionRegion, EditDocument, LayoutState};

use crate::{
    graph::crop_rect_at,
    plan::RenderError,
    profile::RenderProfile,
    subtitles::{Sweep, sweep},
    timing::FrameRate,
};

/// A crop rectangle in source pixels, as the encoder will apply it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreviewCrop {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
}

/// One word, with where the highlight is while it is being said.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewWord {
    pub text: String,
    /// Centiseconds this word holds the highlight, from the sweep the burned-in
    /// track uses. Zero on a cue with no animation.
    pub hold_centis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewLine {
    pub words: Vec<PreviewWord>,
}

/// One caption, in frames rather than ticks.
///
/// Frames because that is the unit a divergence is measured in: a cue that
/// appears one frame early in the player and on time in the export is exactly
/// the kind of difference this plan exists to make impossible to miss.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewCue {
    pub cue_id: String,
    pub first_frame: i64,
    pub end_frame: i64,
    pub region: CaptionRegion,
    pub karaoke: bool,
    /// Before the first word is sung.
    pub lead_in_centis: i64,
    /// Already broken. The player must not re-wrap, for the same reason libass
    /// is told not to: the breaks were decided by the caption engine and a
    /// second opinion here is a preview that disagrees with the file.
    pub lines: Vec<PreviewLine>,
}

/// The gain curve, sampled to frames.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreviewGain {
    pub frame: i64,
    pub gain_db: f64,
}

/// Everything the player needs, and nothing it has to work out.
#[derive(Clone, Debug, PartialEq)]
pub struct PreviewPlan {
    pub rate: FrameRate,
    pub frame_count: i64,
    /// One entry per frame of the program. `None` where the layout is fit and
    /// the whole picture is shown — which is a different statement from a crop
    /// that happens to cover everything.
    pub crops: Vec<Option<PreviewCrop>>,
    pub cues: Vec<PreviewCue>,
    pub gain: Vec<PreviewGain>,
    /// The output frame the crops are fitted into.
    pub width: i64,
    pub height: i64,
}

/// Interpret a document against the proxy timeline.
///
/// Sampling every frame rather than handing over the keyframes is deliberate.
/// Keyframes would make the player interpolate, and interpolating is where the
/// two implementations would have to agree about rounding — which is precisely
/// the agreement that cannot be assumed. An integer rectangle per frame has
/// nothing left to disagree about.
pub fn preview_plan(
    document: &EditDocument,
    profile: &RenderProfile,
) -> Result<PreviewPlan, RenderError> {
    document.validate()?;
    let rate = profile.rate();
    let duration: i64 = document
        .video
        .segments
        .iter()
        .map(clipmill_edit_ir::VideoSegment::duration_ticks)
        .sum();
    if duration <= 0 {
        return Err(RenderError::EmptyProgram);
    }
    let frame_count = rate.frame_count(duration);

    Ok(PreviewPlan {
        rate,
        frame_count,
        crops: crops(document, rate, frame_count),
        cues: cues(document, rate),
        gain: document
            .audio
            .gain_curve
            .iter()
            .map(|point| PreviewGain {
                frame: rate.frame_ceil(point.t_ticks),
                gain_db: point.gain_db,
            })
            .collect(),
        width: profile.width,
        height: profile.height,
    })
}

/// The crop at every frame of the program.
///
/// Segments are laid end to end by their position in the list, so a program
/// frame is found by walking the segments — and the crop path inside one is
/// segment-local, which is why the offset is subtracted before asking.
fn crops(document: &EditDocument, rate: FrameRate, frame_count: i64) -> Vec<Option<PreviewCrop>> {
    let mut boundaries = Vec::with_capacity(document.video.segments.len());
    let mut at = 0_i64;
    for segment in &document.video.segments {
        let frames = rate.frame_count(segment.duration_ticks());
        boundaries.push((at, at + frames, segment));
        at += frames;
    }

    (0..frame_count)
        .map(|frame| {
            let (start, _, segment) = *boundaries
                .iter()
                .find(|(start, end, _)| frame >= *start && frame < *end)
                .or_else(|| boundaries.last())?;
            if matches!(segment.layout.state, LayoutState::Fit) {
                return None;
            }
            crop_rect_at(&segment.layout.crop_path, rate, frame - start).map(|rect| PreviewCrop {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            })
        })
        .collect()
}

/// The cues that are burned into the picture, in frames.
///
/// The burned-in list, not the reading one: the player is showing what a viewer
/// watching the export would see, and the sidecars are a different surface with
/// a different grouping.
fn cues(document: &EditDocument, rate: FrameRate) -> Vec<PreviewCue> {
    document
        .captions
        .burned()
        .iter()
        .map(|cue| {
            let first_frame = rate.frame_ceil(cue.start_ticks);
            let end_frame = rate.frame_ceil(cue.end_ticks);
            let karaoke = matches!(cue.anim, CaptionAnimation::Karaoke);
            let swept = if karaoke {
                sweep(
                    cue,
                    rate,
                    rate.frame_centis(first_frame),
                    rate.frame_centis(end_frame),
                )
            } else {
                Sweep {
                    lead_in_centis: 0,
                    holds_centis: Vec::new(),
                }
            };
            PreviewCue {
                cue_id: cue.cue_id.clone(),
                first_frame,
                end_frame,
                region: cue.region,
                karaoke,
                lead_in_centis: swept.lead_in_centis,
                lines: lines(cue, &swept),
            }
        })
        .collect()
}

fn lines(cue: &CaptionCue, swept: &Sweep) -> Vec<PreviewLine> {
    let mut index = 0_usize;
    cue.lines
        .iter()
        .map(|line| PreviewLine {
            words: line
                .words
                .iter()
                .map(|word| {
                    let hold = swept.holds_centis.get(index).copied().unwrap_or(0);
                    index += 1;
                    PreviewWord {
                        text: word.text.clone(),
                        hold_centis: hold,
                    }
                })
                .collect(),
        })
        .collect()
}

/// The text a viewer reads at a frame, or nothing.
///
/// The comparison the gate makes, expressed once so both sides of it mean the
/// same thing.
pub fn text_at(plan: &PreviewPlan, frame: i64) -> Option<String> {
    plan.cues
        .iter()
        .find(|cue| frame >= cue.first_frame && frame < cue.end_frame)
        .map(|cue| {
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
                .join(" ")
        })
}

#[cfg(test)]
mod tests;
