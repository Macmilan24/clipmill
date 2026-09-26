use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    document::{
        CaptionCue, CaptionOptions, CaptionPosition, CaptionRegion, CropEasing, CropKeyframe,
        CropRect, DocumentError, EditDocument, FitBackground, FrameShape, GainPoint, Inset,
        LayoutState, Overlay, Presentation, Punch, VideoSegment, crop_along_keyframes,
        retime_punches, split_punches,
    },
    reflow,
};

/// One typed, serializable edit. Applying a command returns the command that
/// undoes it, so undo/redo and durable replay share a single mechanism
/// instead of being two implementations that can drift apart.
///
/// Every inverse is exact: `apply(cmd)` followed by `apply(cmd.inverse)`
/// restores the document byte-for-byte in its canonical form. Commands that
/// can destroy material invert to [`EditCommand::RestoreArrangement`], which
/// carries the prior arrangement rather than trying to reconstruct it.
///
/// Commands never read a clock or a random source: replaying a log must mint
/// exactly the identifiers it minted live.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum EditCommand {
    /// Hold the outgoing composition briefly over the incoming picture.
    /// This changes no source interval, audio, caption or program timing.
    SetTransition {
        duration_ticks: i64,
    },
    /// Replace a section's punches: the moments its followed crop moves in.
    SetPunches {
        segment_id: String,
        punches: Vec<Punch>,
    },
    /// Deliver the clip in another frame shape. Only the shape: the crops
    /// that fit it are replaced by commands of their own, batched with this.
    SetFrameShape {
        shape: FrameShape,
    },
    /// Lay something over the program: on top of the others, or at `at` in
    /// their stack, which is how removing one is undone in its own place.
    AddOverlay {
        overlay: Overlay,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        at: Option<usize>,
    },
    RemoveOverlay {
        overlay_id: String,
    },
    /// Change an overlay in place: its span, text, look or position.
    SetOverlay {
        overlay: Overlay,
    },
    /// Name the clip, or clear its name with `None`.
    SetTitle {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
    },
    /// Change the clip's named caption preset without rewriting its words.
    SetCaptionStyle {
        style_ref: String,
    },
    /// Clip-wide type, colour, and case adjustments over the preset.
    SetCaptionOptions {
        options: CaptionOptions,
    },
    /// Move a segment's source window. Program-anchored content after the
    /// segment follows; content stranded in a shortened tail is removed.
    Trim {
        segment_id: String,
        in_ticks: i64,
        out_ticks: i64,
    },
    /// Grow a boundary segment and add cues derived from only the newly
    /// exposed words. Existing cue IDs and corrections remain untouched.
    ExtendSegment {
        segment_id: String,
        in_ticks: i64,
        out_ticks: i64,
        reading_cues: Vec<CaptionCue>,
        burn_in_cues: Vec<CaptionCue>,
    },
    /// Remove a program-time span and close the gap.
    RippleDelete {
        start_ticks: i64,
        end_ticks: i64,
        /// New whole-program trims repair caption fragments at the kept edge.
        /// Absent in older logs: replay must preserve their original grouping.
        #[serde(default, skip_serializing_if = "is_false")]
        reflow_edges: bool,
    },
    /// Restore a previously captured arrangement. This is the inverse of
    /// every command that can destroy material.
    RestoreArrangement {
        segments: Vec<VideoSegment>,
        cues: Vec<CaptionCue>,
        gain_curve: Vec<GainPoint>,
        /// The burned-in cues as they were. Absent in a log written before
        /// trims touched that list, and then left as it stands — which is
        /// what those trims had done to it.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        burn_in: Option<Vec<CaptionCue>>,
        /// The overlays as they were. Absent in a log written before there
        /// were any, and then left as they stand.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        overlays: Option<Vec<Overlay>>,
    },
    SetLayout {
        segment_id: String,
        state: LayoutState,
    },
    /// Exchange the upper and lower camera paths in a two-person section.
    SwapPortraits {
        segment_id: String,
    },
    /// How a section's layout is drawn: the split between two viewports,
    /// what fills around a fitted picture and how far it is zoomed, and where
    /// a picture-in-picture inset sits. All four at once, so the inverse is
    /// one command and a style can be copied to every section as a batch.
    SetLayoutStyle {
        segment_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        split: Option<u16>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        background: Option<FitBackground>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zoom: Option<u16>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        inset: Option<Inset>,
    },
    /// Make two independently framed sections at a source-time cut.
    SplitSegment {
        segment_id: String,
        at_ticks: i64,
        new_segment_id: String,
    },
    /// Insert or replace a crop keyframe at a segment-local tick.
    SetCropKeyframe {
        segment_id: String,
        t_ticks: i64,
        rect: CropRect,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        easing: Option<CropEasing>,
    },
    RemoveCropKeyframe {
        segment_id: String,
        t_ticks: i64,
    },
    /// Replace a solved path atomically, removing obsolete manual keyframes.
    ReplaceCropPath {
        segment_id: String,
        path: Vec<CropKeyframe>,
    },
    /// Adjust the lower portrait without changing the upper portrait.
    SetSecondaryCropKeyframe {
        segment_id: String,
        t_ticks: i64,
        rect: CropRect,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        easing: Option<CropEasing>,
    },
    RemoveSecondaryCropKeyframe {
        segment_id: String,
        t_ticks: i64,
    },
    /// Replace the lower portrait's solved path, as a re-solve of a
    /// two-person layout does beside `ReplaceCropPath` for the upper one.
    ReplaceSecondaryCropPath {
        segment_id: String,
        path: Vec<CropKeyframe>,
    },
    /// Correct one word's text without disturbing its timing, addressed by
    /// cue and position. The word's other occurrence — the same word in the
    /// other presentation — is corrected with it when the word carries an id.
    EditCaptionText {
        cue_id: String,
        word_index: usize,
        text: String,
        #[serde(default, skip_serializing_if = "is_reading")]
        presentation: Presentation,
    },
    /// Correct one word's text wherever it appears. The correction belongs
    /// to the word, and the word is in both presentations; a cue and an
    /// index name one grouping only, and not the one the player shows.
    SetWordText {
        word_id: String,
        text: String,
    },
    /// Hide a spoken word in both caption presentations without cutting audio.
    RemoveCaptionWord {
        cue_id: String,
        word_index: usize,
        #[serde(default, skip_serializing_if = "is_reading")]
        presentation: Presentation,
    },
    SetCueRegion {
        cue_id: String,
        region: CaptionRegion,
        #[serde(default, skip_serializing_if = "is_reading")]
        presentation: Presentation,
    },
    /// Place one cue by hand, or hand it back to its region with `None`.
    SetCuePosition {
        cue_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        position: Option<CaptionPosition>,
        #[serde(default, skip_serializing_if = "is_reading")]
        presentation: Presentation,
    },
    /// Mark or unmark a key word, wherever it appears. Emphasis belongs to
    /// the word, like a correction, so both presentations carry it.
    SetWordEmphasis {
        word_id: String,
        emphasis: bool,
    },
    /// Regroup the on-screen captions to show at most this many words at
    /// once. Only the burned-in list changes; the sidecar grouping stays.
    RegroupOnScreen {
        max_words: u32,
    },
    /// Remove words that are not speech — a recognizer's dash, a silence
    /// marker, a bracketed annotation — from both presentations.
    DropNonSpeechWords {},
    /// Replace one presentation's cues whole.
    ///
    /// A regrouping or a re-derivation is computed where the segmenter lives
    /// and carried here in full, so replaying the log needs nothing but the
    /// log. The inverse carries the list it replaced.
    ReplaceCues {
        cues: Vec<CaptionCue>,
        #[serde(default, skip_serializing_if = "is_reading")]
        presentation: Presentation,
    },
    /// Move a display window while every spoken word keeps its timing.
    SetCueTiming {
        cue_id: String,
        start_ticks: i64,
        end_ticks: i64,
        #[serde(default, skip_serializing_if = "is_reading")]
        presentation: Presentation,
    },
    /// Re-flow a cue's words into lines. Line breaks are stored, never
    /// recomputed at render time.
    SetCueLines {
        cue_id: String,
        line_word_counts: Vec<usize>,
        #[serde(default, skip_serializing_if = "is_reading")]
        presentation: Presentation,
    },
    /// Split a cue at a word boundary. The caller names the new cue so that
    /// replay reproduces the same identifier.
    SplitCue {
        cue_id: String,
        at_word_index: usize,
        new_cue_id: String,
        #[serde(default, skip_serializing_if = "is_reading")]
        presentation: Presentation,
    },
    MergeCues {
        first_cue_id: String,
        second_cue_id: String,
        #[serde(default, skip_serializing_if = "is_reading")]
        presentation: Presentation,
    },
    SetGain {
        t_ticks: i64,
        gain_db: f64,
    },
    RemoveGainPoint {
        t_ticks: i64,
    },
    /// Apply several commands as one undoable step.
    Batch {
        commands: Vec<EditCommand>,
    },
}

/// The reading presentation is the default, and a command that names it is
/// written as it always was, so a log replays byte for byte.
#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde's skip_serializing_if hands the field by reference"
)]
fn is_reading(presentation: &Presentation) -> bool {
    *presentation == Presentation::Reading
}

#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde's skip_serializing_if hands the field by reference"
)]
fn is_false(value: &bool) -> bool {
    !*value
}

impl EditCommand {
    /// Apply this command, returning the command that undoes it.
    ///
    /// The document is only modified on success: work happens on a clone that
    /// is validated before it replaces the caller's document, so a rejected
    /// command can never leave a half-applied edit behind.
    pub fn apply(&self, document: &mut EditDocument) -> Result<Self, CommandError> {
        let mut working = document.clone();
        let inverse = self.apply_in_place(&mut working)?;
        working.validate()?;
        *document = working;
        Ok(inverse)
    }

    #[allow(clippy::too_many_lines)]
    fn apply_in_place(&self, document: &mut EditDocument) -> Result<Self, CommandError> {
        match self {
            Self::SetTransition { duration_ticks } => {
                let previous =
                    std::mem::replace(&mut document.video.transition_ticks, *duration_ticks);
                Ok(Self::SetTransition {
                    duration_ticks: previous,
                })
            }
            Self::SetPunches {
                segment_id,
                punches,
            } => {
                let index = document.segment_index(segment_id)?;
                let segment = document
                    .video
                    .segments
                    .get_mut(index)
                    .ok_or_else(|| DocumentError::UnknownSegment(segment_id.clone()))?;
                let previous = std::mem::replace(&mut segment.layout.punches, punches.clone());
                Ok(Self::SetPunches {
                    segment_id: segment_id.clone(),
                    punches: previous,
                })
            }
            Self::SetFrameShape { shape } => {
                let previous = std::mem::replace(&mut document.video.shape, *shape);
                Ok(Self::SetFrameShape { shape: previous })
            }
            Self::AddOverlay { overlay, at } => {
                if document
                    .overlays
                    .iter()
                    .any(|existing| existing.overlay_id == overlay.overlay_id)
                {
                    return Err(CommandError::OverlayAlreadyExists(
                        overlay.overlay_id.clone(),
                    ));
                }
                let position = at.unwrap_or(document.overlays.len());
                if position > document.overlays.len() {
                    return Err(CommandError::OverlayOutOfStack(position));
                }
                document.overlays.insert(position, overlay.clone());
                Ok(Self::RemoveOverlay {
                    overlay_id: overlay.overlay_id.clone(),
                })
            }
            Self::RemoveOverlay { overlay_id } => {
                let position = document.overlay_index(overlay_id)?;
                let overlay = document.overlays.remove(position);
                Ok(Self::AddOverlay {
                    overlay,
                    at: Some(position),
                })
            }
            Self::SetOverlay { overlay } => {
                let position = document.overlay_index(&overlay.overlay_id)?;
                let previous = std::mem::replace(&mut document.overlays[position], overlay.clone());
                Ok(Self::SetOverlay { overlay: previous })
            }
            Self::SetTitle { title } => {
                let previous = std::mem::replace(
                    &mut document.title,
                    title.as_ref().map(|title| title.trim().to_owned()),
                );
                Ok(Self::SetTitle { title: previous })
            }
            Self::SetCaptionStyle { style_ref } => {
                if clipmill_captions::preset(style_ref).is_none() {
                    return Err(CommandError::UnknownCaptionStyle(style_ref.clone()));
                }
                let previous =
                    std::mem::replace(&mut document.captions.style_ref, style_ref.clone());
                Ok(Self::SetCaptionStyle {
                    style_ref: previous,
                })
            }
            Self::SetCaptionOptions { options } => {
                let previous = std::mem::replace(&mut document.captions.options, options.clone());
                Ok(Self::SetCaptionOptions { options: previous })
            }
            Self::Trim {
                segment_id,
                in_ticks,
                out_ticks,
            } => Self::apply_trim(document, segment_id, *in_ticks, *out_ticks),
            Self::ExtendSegment {
                segment_id,
                in_ticks,
                out_ticks,
                reading_cues,
                burn_in_cues,
            } => Self::extend_segment(
                document,
                segment_id,
                *in_ticks,
                *out_ticks,
                reading_cues,
                burn_in_cues,
            ),
            Self::RippleDelete {
                start_ticks,
                end_ticks,
                reflow_edges,
            } => Self::apply_ripple_delete(document, *start_ticks, *end_ticks, *reflow_edges),
            Self::RestoreArrangement {
                segments,
                cues,
                gain_curve,
                burn_in,
                overlays,
            } => {
                let inverse = Self::capture(document);
                document.video.segments.clone_from(segments);
                document.captions.cues.clone_from(cues);
                if let Some(burn_in) = burn_in {
                    document.captions.burn_in.clone_from(burn_in);
                }
                document.audio.gain_curve.clone_from(gain_curve);
                if let Some(overlays) = overlays {
                    document.overlays.clone_from(overlays);
                }
                Ok(inverse)
            }
            Self::SetLayout { segment_id, state } => {
                let index = document.segment_index(segment_id)?;
                let segment = document
                    .video
                    .segments
                    .get_mut(index)
                    .ok_or_else(|| DocumentError::UnknownSegment(segment_id.clone()))?;
                let previous = segment.layout.state;
                segment.layout.state = *state;
                Ok(Self::SetLayout {
                    segment_id: segment_id.clone(),
                    state: previous,
                })
            }
            Self::SetLayoutStyle {
                segment_id,
                split,
                background,
                zoom,
                inset,
            } => {
                let index = document.segment_index(segment_id)?;
                let layout = &mut document.video.segments[index].layout;
                Ok(Self::SetLayoutStyle {
                    segment_id: segment_id.clone(),
                    split: std::mem::replace(&mut layout.split, *split),
                    background: std::mem::replace(&mut layout.background, background.clone()),
                    zoom: std::mem::replace(&mut layout.zoom, *zoom),
                    inset: std::mem::replace(&mut layout.inset, *inset),
                })
            }
            Self::SwapPortraits { segment_id } => {
                let index = document.segment_index(segment_id)?;
                let layout = &mut document.video.segments[index].layout;
                if layout.state != LayoutState::TwoUp {
                    return Err(CommandError::NotTwoUp);
                }
                std::mem::swap(&mut layout.crop_path, &mut layout.secondary_crop_path);
                Ok(Self::SwapPortraits {
                    segment_id: segment_id.clone(),
                })
            }
            Self::SplitSegment {
                segment_id,
                at_ticks,
                new_segment_id,
            } => Self::split_segment(document, segment_id, *at_ticks, new_segment_id),
            Self::ReplaceCropPath { segment_id, path } => {
                let index = document.segment_index(segment_id)?;
                let previous = std::mem::replace(
                    &mut document.video.segments[index].layout.crop_path,
                    path.clone(),
                );
                Ok(Self::ReplaceCropPath {
                    segment_id: segment_id.clone(),
                    path: previous,
                })
            }
            Self::ReplaceSecondaryCropPath { segment_id, path } => {
                let index = document.segment_index(segment_id)?;
                let previous = std::mem::replace(
                    &mut document.video.segments[index].layout.secondary_crop_path,
                    path.clone(),
                );
                Ok(Self::ReplaceSecondaryCropPath {
                    segment_id: segment_id.clone(),
                    path: previous,
                })
            }
            Self::SetCropKeyframe {
                segment_id,
                t_ticks,
                rect,
                easing,
            } => Self::crop_command(document, segment_id, *t_ticks, Some(*rect), *easing, false),
            Self::RemoveCropKeyframe {
                segment_id,
                t_ticks,
            } => Self::crop_command(document, segment_id, *t_ticks, None, None, false),
            Self::SetSecondaryCropKeyframe {
                segment_id,
                t_ticks,
                rect,
                easing,
            } => Self::crop_command(document, segment_id, *t_ticks, Some(*rect), *easing, true),
            Self::RemoveSecondaryCropKeyframe {
                segment_id,
                t_ticks,
            } => Self::crop_command(document, segment_id, *t_ticks, None, None, true),
            Self::EditCaptionText {
                cue_id,
                word_index,
                text,
                presentation,
            } => {
                let index = document.cue_index(*presentation, cue_id)?;
                let cue = document
                    .captions
                    .list(*presentation)
                    .get(index)
                    .ok_or_else(|| DocumentError::UnknownCue(cue_id.clone()))?;
                let word = cue
                    .words()
                    .nth(*word_index)
                    .ok_or(CommandError::NoSuchWord(*word_index))?;
                let previous = if let Some(word_id) = word.word_id.clone() {
                    // The word is known by name: the correction reaches its
                    // twin in the other presentation too.
                    document.set_word_text(&word_id, text)?
                } else {
                    // A document that predates word ids: this occurrence only.
                    let cue = document
                        .captions
                        .list_mut(*presentation)
                        .get_mut(index)
                        .ok_or_else(|| DocumentError::UnknownCue(cue_id.clone()))?;
                    let word = cue
                        .lines
                        .iter_mut()
                        .flat_map(|line| line.words.iter_mut())
                        .nth(*word_index)
                        .ok_or(CommandError::NoSuchWord(*word_index))?;
                    std::mem::replace(&mut word.text, text.clone())
                };
                Ok(Self::EditCaptionText {
                    cue_id: cue_id.clone(),
                    word_index: *word_index,
                    text: previous,
                    presentation: *presentation,
                })
            }
            Self::SetWordText { word_id, text } => {
                let previous = document.set_word_text(word_id, text)?;
                Ok(Self::SetWordText {
                    word_id: word_id.clone(),
                    text: previous,
                })
            }
            Self::RemoveCaptionWord {
                cue_id,
                word_index,
                presentation,
            } => Self::remove_caption_word(document, *presentation, cue_id, *word_index),
            Self::SetCueRegion {
                cue_id,
                region,
                presentation,
            } => {
                let index = document.cue_index(*presentation, cue_id)?;
                let cue = &mut document.captions.list_mut(*presentation)[index];
                let previous = std::mem::replace(&mut cue.region, *region);
                Ok(Self::SetCueRegion {
                    cue_id: cue_id.clone(),
                    region: previous,
                    presentation: *presentation,
                })
            }
            Self::SetCuePosition {
                cue_id,
                position,
                presentation,
            } => {
                let index = document.cue_index(*presentation, cue_id)?;
                let cue = &mut document.captions.list_mut(*presentation)[index];
                let previous = std::mem::replace(&mut cue.position, *position);
                Ok(Self::SetCuePosition {
                    cue_id: cue_id.clone(),
                    position: previous,
                    presentation: *presentation,
                })
            }
            Self::SetWordEmphasis { word_id, emphasis } => {
                let mut previous = None;
                for word in document.captions.words_mut() {
                    if word.word_id.as_deref() == Some(word_id.as_str()) {
                        previous.get_or_insert(word.emphasis);
                        word.emphasis = *emphasis;
                    }
                }
                let previous =
                    previous.ok_or_else(|| DocumentError::UnknownWord(word_id.clone()))?;
                Ok(Self::SetWordEmphasis {
                    word_id: word_id.clone(),
                    emphasis: previous,
                })
            }
            Self::RegroupOnScreen { max_words } => {
                if !(1..=8).contains(max_words) {
                    return Err(CommandError::InvalidWordsOnScreen);
                }
                let previous_cues = document.captions.burn_in.clone();
                let previous_options = document.captions.options.clone();
                let regrouped = crate::regroup::regroup(
                    document.captions.burned(),
                    usize::try_from(*max_words).unwrap_or(1),
                );
                document.captions.burn_in = regrouped;
                document.captions.options.words_on_screen = Some(*max_words);
                Ok(Self::Batch {
                    commands: vec![
                        Self::SetCaptionOptions {
                            options: previous_options,
                        },
                        Self::ReplaceCues {
                            cues: previous_cues,
                            presentation: Presentation::BurnIn,
                        },
                    ],
                })
            }
            Self::DropNonSpeechWords {} => {
                let previous_reading = document.captions.cues.clone();
                let previous_burn_in = document.captions.burn_in.clone();
                for presentation in [Presentation::Reading, Presentation::BurnIn] {
                    let cues = document.captions.list_mut(presentation);
                    for cue in cues.iter_mut() {
                        for line in &mut cue.lines {
                            line.words
                                .retain(|word| clipmill_captions::captionable_word(&word.text));
                        }
                        cue.lines.retain(|line| !line.words.is_empty());
                    }
                    cues.retain(|cue| !cue.lines.is_empty());
                }
                Ok(Self::Batch {
                    commands: vec![
                        Self::ReplaceCues {
                            cues: previous_reading,
                            presentation: Presentation::Reading,
                        },
                        Self::ReplaceCues {
                            cues: previous_burn_in,
                            presentation: Presentation::BurnIn,
                        },
                    ],
                })
            }
            Self::ReplaceCues { cues, presentation } => {
                let previous =
                    std::mem::replace(document.captions.list_mut(*presentation), cues.clone());
                Ok(Self::ReplaceCues {
                    cues: previous,
                    presentation: *presentation,
                })
            }
            Self::SetCueTiming {
                cue_id,
                start_ticks,
                end_ticks,
                presentation,
            } => {
                if *start_ticks < 0
                    || *end_ticks <= *start_ticks
                    || *end_ticks > document.program_duration_ticks()
                {
                    return Err(CommandError::CaptionTimingOutsideProgram);
                }
                let index = document.cue_index(*presentation, cue_id)?;
                let cues = document.captions.list(*presentation);
                if (index > 0 && cues[index - 1].end_ticks > *start_ticks)
                    || cues
                        .get(index + 1)
                        .is_some_and(|next| next.start_ticks < *end_ticks)
                {
                    return Err(CommandError::CaptionTimingOverlaps);
                }
                if cues[index]
                    .words()
                    .any(|word| word.start_ticks < *start_ticks || word.end_ticks > *end_ticks)
                {
                    return Err(CommandError::CaptionTimingExcludesWords);
                }
                let cue = &mut document.captions.list_mut(*presentation)[index];
                let inverse = Self::SetCueTiming {
                    cue_id: cue_id.clone(),
                    start_ticks: cue.start_ticks,
                    end_ticks: cue.end_ticks,
                    presentation: *presentation,
                };
                cue.start_ticks = *start_ticks;
                cue.end_ticks = *end_ticks;
                Ok(inverse)
            }
            Self::SetCueLines {
                cue_id,
                line_word_counts,
                presentation,
            } => {
                let index = document.cue_index(*presentation, cue_id)?;
                let cue = document
                    .captions
                    .list_mut(*presentation)
                    .get_mut(index)
                    .ok_or_else(|| DocumentError::UnknownCue(cue_id.clone()))?;
                let previous = cue.line_word_counts();
                cue.reflow(line_word_counts)?;
                Ok(Self::SetCueLines {
                    cue_id: cue_id.clone(),
                    line_word_counts: previous,
                    presentation: *presentation,
                })
            }
            Self::SplitCue {
                cue_id,
                at_word_index,
                new_cue_id,
                presentation,
            } => Self::apply_split_cue(document, *presentation, cue_id, *at_word_index, new_cue_id),
            Self::MergeCues {
                first_cue_id,
                second_cue_id,
                presentation,
            } => Self::apply_merge_cues(document, *presentation, first_cue_id, second_cue_id),
            Self::SetGain { t_ticks, gain_db } => {
                let curve = &mut document.audio.gain_curve;
                match curve.binary_search_by_key(t_ticks, |point| point.t_ticks) {
                    Ok(position) => {
                        let previous = curve[position].gain_db;
                        curve[position].gain_db = *gain_db;
                        Ok(Self::SetGain {
                            t_ticks: *t_ticks,
                            gain_db: previous,
                        })
                    }
                    Err(position) => {
                        curve.insert(
                            position,
                            GainPoint {
                                t_ticks: *t_ticks,
                                gain_db: *gain_db,
                            },
                        );
                        Ok(Self::RemoveGainPoint { t_ticks: *t_ticks })
                    }
                }
            }
            Self::RemoveGainPoint { t_ticks } => {
                let curve = &mut document.audio.gain_curve;
                let position = curve
                    .binary_search_by_key(t_ticks, |point| point.t_ticks)
                    .map_err(|_| CommandError::NoGainPoint(*t_ticks))?;
                let removed = curve.remove(position);
                Ok(Self::SetGain {
                    t_ticks: *t_ticks,
                    gain_db: removed.gain_db,
                })
            }
            Self::Batch { commands } => {
                let mut inverses = Vec::with_capacity(commands.len());
                for command in commands {
                    inverses.push(command.apply_in_place(document)?);
                }
                inverses.reverse();
                Ok(Self::Batch { commands: inverses })
            }
        }
    }

    /// Capture the current arrangement as the command that would restore it.
    pub fn capture(document: &EditDocument) -> Self {
        Self::RestoreArrangement {
            segments: document.video.segments.clone(),
            cues: document.captions.cues.clone(),
            gain_curve: document.audio.gain_curve.clone(),
            burn_in: Some(document.captions.burn_in.clone()),
            // Only once there is something to restore, so a log of a clip
            // with none reads as it always has.
            overlays: (!document.overlays.is_empty()).then(|| document.overlays.clone()),
        }
    }

    fn split_segment(
        document: &mut EditDocument,
        segment_id: &str,
        at_ticks: i64,
        new_segment_id: &str,
    ) -> Result<Self, CommandError> {
        if new_segment_id.is_empty()
            || document
                .video
                .segments
                .iter()
                .any(|item| item.segment_id == new_segment_id)
        {
            return Err(CommandError::SegmentAlreadyExists(
                new_segment_id.to_owned(),
            ));
        }
        let index = document.segment_index(segment_id)?;
        let original = document.video.segments[index].clone();
        if at_ticks <= original.in_ticks || at_ticks >= original.out_ticks {
            return Err(CommandError::SplitOutsideSegment);
        }
        let inverse = Self::capture(document);
        let offset = at_ticks - original.in_ticks;
        let (head_crop, tail_crop) = Self::split_crop_path(&original.layout.crop_path, offset);
        let (head_secondary, tail_secondary) =
            Self::split_crop_path(&original.layout.secondary_crop_path, offset);
        let mut head = original.clone();
        head.out_ticks = at_ticks;
        let (head_punches, tail_punches) = split_punches(&original.layout.punches, offset);
        head.layout.crop_path = head_crop;
        head.layout.secondary_crop_path = head_secondary;
        head.layout.punches = head_punches;
        let mut tail = original;
        new_segment_id.clone_into(&mut tail.segment_id);
        tail.in_ticks = at_ticks;
        tail.layout.crop_path = tail_crop;
        tail.layout.secondary_crop_path = tail_secondary;
        tail.layout.punches = tail_punches;
        document.video.segments.splice(index..=index, [head, tail]);
        Ok(inverse)
    }

    fn extend_segment(
        document: &mut EditDocument,
        segment_id: &str,
        in_ticks: i64,
        out_ticks: i64,
        reading_cues: &[CaptionCue],
        burn_in_cues: &[CaptionCue],
    ) -> Result<Self, CommandError> {
        let index = document.segment_index(segment_id)?;
        let original = document.video.segments[index].clone();
        let head = index == 0
            && in_ticks >= 0
            && in_ticks < original.in_ticks
            && out_ticks == original.out_ticks;
        let tail = index + 1 == document.video.segments.len()
            && in_ticks == original.in_ticks
            && out_ticks > original.out_ticks;
        if !head && !tail {
            return Err(CommandError::ExtensionMustGrowEdge);
        }
        let delta = if head {
            original.in_ticks - in_ticks
        } else {
            out_ticks - original.out_ticks
        };
        let original_duration = document.program_duration_ticks();
        let inverse = Self::capture(document);
        if head {
            let layout = &mut document.video.segments[index].layout;
            for punch in &mut layout.punches {
                punch.start_ticks += delta;
                punch.end_ticks += delta;
            }
            for path in [&mut layout.crop_path, &mut layout.secondary_crop_path] {
                if let Some(first) = path.first().copied() {
                    for key in path.iter_mut() {
                        key.t_ticks += delta;
                    }
                    path.insert(
                        0,
                        CropKeyframe {
                            t_ticks: 0,
                            rect: first.rect,
                            easing: CropEasing::default(),
                        },
                    );
                }
            }
            document.splice_program_content(0, 0, delta, original_duration);
            document.video.segments[index].in_ticks = in_ticks;
        } else {
            document.video.segments[index].out_ticks = out_ticks;
        }
        let offset = if head { 0 } else { original_duration };
        for (presentation, incoming) in [
            (Presentation::Reading, reading_cues),
            (Presentation::BurnIn, burn_in_cues),
        ] {
            let adjacent = document.captions.list(presentation);
            let region = if head {
                adjacent.first()
            } else {
                adjacent.last()
            }
            .map_or(CaptionRegion::LowerSafe, |cue| cue.region);
            let mut added = incoming.to_vec();
            for cue in &mut added {
                cue.region = region;
                cue.start_ticks += offset;
                cue.end_ticks += offset;
                for word in cue.lines.iter_mut().flat_map(|line| &mut line.words) {
                    word.start_ticks += offset;
                    word.end_ticks += offset;
                }
            }
            let cues = document.captions.list_mut(presentation);
            if head {
                added.extend(std::mem::take(cues));
                *cues = added;
            } else {
                cues.extend(added);
            }
        }
        Ok(inverse)
    }

    fn split_crop_path(path: &[CropKeyframe], at: i64) -> (Vec<CropKeyframe>, Vec<CropKeyframe>) {
        if path.is_empty() {
            return (Vec::new(), Vec::new());
        }
        let Some(boundary) = crop_along_keyframes(path, at) else {
            return (Vec::new(), Vec::new());
        };
        let mut head: Vec<_> = path
            .iter()
            .filter(|key| key.t_ticks < at)
            .copied()
            .collect();
        head.push(CropKeyframe {
            t_ticks: at,
            rect: boundary,
            easing: CropEasing::Linear,
        });
        let mut tail = vec![CropKeyframe {
            t_ticks: 0,
            rect: boundary,
            easing: CropEasing::Linear,
        }];
        tail.extend(
            path.iter()
                .filter(|key| key.t_ticks > at)
                .map(|key| CropKeyframe {
                    t_ticks: key.t_ticks - at,
                    rect: key.rect,
                    easing: key.easing,
                }),
        );
        (head, tail)
    }

    /// Move a segment's source window, and everything anchored to it.
    ///
    /// The head and the tail are two different edits and are spliced as two.
    /// Advancing the in point removes program time at the segment's *start*:
    /// the words said there are gone and everything after them moves earlier.
    /// Moving the out point changes program time at its *end*. Treating any
    /// shortening as a tail deletion, as this once did, kept the original
    /// opening caption over footage that now began a second later and cut
    /// the caption that was actually still playing.
    fn crop_command(
        document: &mut EditDocument,
        segment_id: &str,
        t_ticks: i64,
        rect: Option<CropRect>,
        easing: Option<CropEasing>,
        secondary: bool,
    ) -> Result<Self, CommandError> {
        let index = document.segment_index(segment_id)?;
        let segment = document
            .video
            .segments
            .get_mut(index)
            .ok_or_else(|| DocumentError::UnknownSegment(segment_id.to_owned()))?;
        let path = if secondary {
            &mut segment.layout.secondary_crop_path
        } else {
            &mut segment.layout.crop_path
        };
        let set = |rect, easing| {
            if secondary {
                Self::SetSecondaryCropKeyframe {
                    segment_id: segment_id.to_owned(),
                    t_ticks,
                    rect,
                    easing,
                }
            } else {
                Self::SetCropKeyframe {
                    segment_id: segment_id.to_owned(),
                    t_ticks,
                    rect,
                    easing,
                }
            }
        };
        let remove = || {
            if secondary {
                Self::RemoveSecondaryCropKeyframe {
                    segment_id: segment_id.to_owned(),
                    t_ticks,
                }
            } else {
                Self::RemoveCropKeyframe {
                    segment_id: segment_id.to_owned(),
                    t_ticks,
                }
            }
        };
        match (
            path.binary_search_by_key(&t_ticks, |keyframe| keyframe.t_ticks),
            rect,
        ) {
            (Ok(position), Some(rect)) => {
                let previous = path[position].rect;
                let previous_easing = path[position].easing;
                path[position].rect = rect;
                if let Some(easing) = easing {
                    path[position].easing = easing;
                }
                Ok(set(previous, Some(previous_easing)))
            }
            (Err(position), Some(rect)) => {
                path.insert(
                    position,
                    CropKeyframe {
                        t_ticks,
                        rect,
                        easing: easing.unwrap_or_default(),
                    },
                );
                Ok(remove())
            }
            (Ok(position), None) => {
                let previous = path.remove(position);
                Ok(set(previous.rect, Some(previous.easing)))
            }
            (Err(_), None) => Err(CommandError::NoCropKeyframe(t_ticks)),
        }
    }

    fn apply_trim(
        document: &mut EditDocument,
        segment_id: &str,
        in_ticks: i64,
        out_ticks: i64,
    ) -> Result<Self, CommandError> {
        let prior = Self::capture(document);
        let (old_in, old_out) = Self::trim_in_place(document, segment_id, in_ticks, out_ticks)?;
        // The narrow inverse — the old window — is the log's preferred undo,
        // but it is only an undo if trimming back reproduces the arrangement
        // exactly. Cutting keeps the crop and the gain the material had at
        // the new edges as points of their own, and words a cut removed do
        // not come back with the window, so the inverse is checked rather
        // than assumed: the narrow one where it restores every byte, the
        // whole prior arrangement where it would not.
        let mut check = document.clone();
        let narrow_restores = Self::trim_in_place(&mut check, segment_id, old_in, old_out).is_ok()
            && Self::capture(&check) == prior;
        if narrow_restores {
            Ok(Self::Trim {
                segment_id: segment_id.to_owned(),
                in_ticks: old_in,
                out_ticks: old_out,
            })
        } else {
            Ok(prior)
        }
    }

    /// Move the window and everything anchored to it; the old window back.
    fn trim_in_place(
        document: &mut EditDocument,
        segment_id: &str,
        in_ticks: i64,
        out_ticks: i64,
    ) -> Result<(i64, i64), CommandError> {
        if out_ticks <= in_ticks || in_ticks < 0 {
            return Err(CommandError::EmptyRange);
        }
        let index = document.segment_index(segment_id)?;
        let starts = document.segment_program_starts();
        let program_start = *starts
            .get(index)
            .ok_or_else(|| DocumentError::UnknownSegment(segment_id.to_owned()))?;
        let segment = document
            .video
            .segments
            .get_mut(index)
            .ok_or_else(|| DocumentError::UnknownSegment(segment_id.to_owned()))?;
        let old_in = segment.in_ticks;
        let old_out = segment.out_ticks;
        let old_duration = segment.duration_ticks();
        let new_duration = out_ticks.saturating_sub(in_ticks);
        segment.in_ticks = in_ticks;
        segment.out_ticks = out_ticks;
        segment.layout.crop_path = EditDocument::retime_crop_path(
            &segment.layout.crop_path,
            old_in,
            in_ticks,
            new_duration,
        );
        segment.layout.secondary_crop_path = EditDocument::retime_crop_path(
            &segment.layout.secondary_crop_path,
            old_in,
            in_ticks,
            new_duration,
        );
        segment.layout.punches =
            retime_punches(&segment.layout.punches, old_in, in_ticks, new_duration);
        // How many words each cue had, so a cue the cut fell inside can be
        // told from one it merely moved.
        let word_counts = Self::caption_word_counts(document);

        // The tail first, in the old program coordinates, so the head's
        // shift does not move the tail before it is cut.
        let old_end = program_start.saturating_add(old_duration);
        // Where the whole program ended before this trim, in the coordinates
        // each splice works in: the segments after this one still follow.
        let program_end = document.program_duration_ticks() - new_duration + old_duration;
        let tail_delta = out_ticks.saturating_sub(old_out);
        match tail_delta.signum() {
            -1 => {
                let cut_from = old_end.saturating_add(tail_delta);
                document.splice_program_content(cut_from, -tail_delta, 0, program_end);
            }
            1 => {
                document.splice_program_content(old_end, 0, tail_delta, program_end);
            }
            _ => {}
        }
        let head_delta = in_ticks.saturating_sub(old_in);
        let program_end = program_end.saturating_add(tail_delta);
        match head_delta.signum() {
            1 => {
                document.splice_program_content(program_start, head_delta, 0, program_end);
            }
            -1 => {
                document.splice_program_content(program_start, 0, -head_delta, program_end);
            }
            _ => {}
        }
        Self::reflow_shortened_edges(document, &word_counts, head_delta > 0, tail_delta < 0);
        Ok((old_in, old_out))
    }

    fn caption_word_counts(document: &EditDocument) -> Vec<(Presentation, String, usize)> {
        [Presentation::Reading, Presentation::BurnIn]
            .into_iter()
            .flat_map(|presentation| {
                document
                    .captions
                    .list(presentation)
                    .iter()
                    .map(move |cue| (presentation, cue.cue_id.clone(), cue.word_count()))
            })
            .collect()
    }

    /// Repair only a surviving cue that lost words at a trimmed program edge.
    /// Removing an entire cue must not regroup its untouched neighbour.
    fn reflow_shortened_edges(
        document: &mut EditDocument,
        word_counts: &[(Presentation, String, usize)],
        head: bool,
        tail: bool,
    ) {
        for presentation in [Presentation::Reading, Presentation::BurnIn] {
            let was = |cue: &CaptionCue| {
                word_counts.iter().any(|(list, id, count)| {
                    *list == presentation && *id == cue.cue_id && *count > cue.word_count()
                })
            };
            let cues = document.captions.list_mut(presentation);
            if head && cues.first().is_some_and(was) {
                reflow::reflow_fragment(cues, presentation, reflow::Edge::Head);
            }
            if tail && cues.last().is_some_and(was) {
                reflow::reflow_fragment(cues, presentation, reflow::Edge::Tail);
            }
        }
    }

    /// A section's crops and punches after its source window changed from
    /// `was`'s to its own: each kept where the new window still plays it and
    /// counted from its new start.
    fn retime_layout(segment: &mut VideoSegment, was: &VideoSegment) {
        let (new_in, duration) = (segment.in_ticks, segment.duration_ticks());
        segment.layout.crop_path =
            EditDocument::retime_crop_path(&was.layout.crop_path, was.in_ticks, new_in, duration);
        segment.layout.secondary_crop_path = EditDocument::retime_crop_path(
            &was.layout.secondary_crop_path,
            was.in_ticks,
            new_in,
            duration,
        );
        segment.layout.punches =
            retime_punches(&was.layout.punches, was.in_ticks, new_in, duration);
    }

    fn apply_ripple_delete(
        document: &mut EditDocument,
        start_ticks: i64,
        end_ticks: i64,
        reflow_edges: bool,
    ) -> Result<Self, CommandError> {
        if end_ticks <= start_ticks || start_ticks < 0 {
            return Err(CommandError::EmptyRange);
        }
        let inverse = Self::capture(document);
        let word_counts = reflow_edges.then(|| Self::caption_word_counts(document));
        let span = end_ticks.saturating_sub(start_ticks);
        let starts = document.segment_program_starts();
        let existing_ids = document
            .video
            .segments
            .iter()
            .map(|segment| segment.segment_id.clone())
            .collect::<Vec<_>>();
        let mut kept: Vec<VideoSegment> = Vec::with_capacity(document.video.segments.len() + 1);
        for (index, segment) in document.video.segments.iter().enumerate() {
            let program_start = starts.get(index).copied().unwrap_or(0);
            let program_end = program_start.saturating_add(segment.duration_ticks());
            if program_end <= start_ticks || program_start >= end_ticks {
                kept.push(segment.clone());
                continue;
            }
            if program_start >= start_ticks && program_end <= end_ticks {
                continue;
            }
            if program_start < start_ticks && program_end > end_ticks {
                let head_out = segment
                    .in_ticks
                    .saturating_add(start_ticks.saturating_sub(program_start));
                let tail_in = segment
                    .in_ticks
                    .saturating_add(end_ticks.saturating_sub(program_start));
                let mut head = segment.clone();
                head.out_ticks = head_out;
                Self::retime_layout(&mut head, segment);
                let mut tail = segment.clone();
                tail.segment_id = EditDocument::derive_id(&existing_ids, &segment.segment_id);
                tail.in_ticks = tail_in;
                Self::retime_layout(&mut tail, segment);
                kept.push(head);
                kept.push(tail);
                continue;
            }
            let mut trimmed = segment.clone();
            if program_start < start_ticks {
                trimmed.out_ticks = segment
                    .in_ticks
                    .saturating_add(start_ticks.saturating_sub(program_start));
            } else {
                trimmed.in_ticks = segment
                    .in_ticks
                    .saturating_add(end_ticks.saturating_sub(program_start));
            }
            Self::retime_layout(&mut trimmed, segment);
            kept.push(trimmed);
        }
        let program_end = document.program_duration_ticks();
        document.video.segments = kept;
        document.splice_program_content(start_ticks, span, 0, program_end);
        // The editor's whole-program head/tail controls use ripple deletion
        // across shot segments. They need the same caption repair as Trim.
        if let Some(word_counts) = word_counts {
            Self::reflow_shortened_edges(
                document,
                &word_counts,
                start_ticks == 0,
                end_ticks >= program_end,
            );
        }
        Ok(inverse)
    }

    fn apply_split_cue(
        document: &mut EditDocument,
        presentation: Presentation,
        cue_id: &str,
        at_word_index: usize,
        new_cue_id: &str,
    ) -> Result<Self, CommandError> {
        if new_cue_id.is_empty() {
            return Err(DocumentError::EmptyIdentifier.into());
        }
        if document
            .captions
            .list(presentation)
            .iter()
            .any(|cue| cue.cue_id == new_cue_id)
        {
            return Err(CommandError::CueAlreadyExists(new_cue_id.to_owned()));
        }
        let index = document.cue_index(presentation, cue_id)?;
        let cue = document
            .captions
            .list(presentation)
            .get(index)
            .ok_or_else(|| DocumentError::UnknownCue(cue_id.to_owned()))?
            .clone();
        let words = cue.words().cloned().collect::<Vec<_>>();
        if at_word_index == 0 || at_word_index >= words.len() {
            return Err(CommandError::SplitOutsideCue(at_word_index));
        }
        let original_counts = cue.line_word_counts();
        let (head_words, tail_words) = words.split_at(at_word_index);
        let mut head = cue.clone();
        head.end_ticks = head_words
            .last()
            .map_or(cue.end_ticks, |word| word.end_ticks);
        head.lines = vec![crate::document::CaptionLine {
            words: head_words.to_vec(),
        }];
        let mut tail = cue;
        new_cue_id.clone_into(&mut tail.cue_id);
        tail.start_ticks = tail_words
            .first()
            .map_or(tail.start_ticks, |word| word.start_ticks);
        tail.lines = Vec::new();
        tail.lines.push(crate::document::CaptionLine {
            words: tail_words.to_vec(),
        });
        document
            .captions
            .list_mut(presentation)
            .splice(index..=index, [head, tail]);
        Ok(Self::Batch {
            commands: vec![
                Self::MergeCues {
                    first_cue_id: cue_id.to_owned(),
                    second_cue_id: new_cue_id.to_owned(),
                    presentation,
                },
                Self::SetCueLines {
                    cue_id: cue_id.to_owned(),
                    line_word_counts: original_counts,
                    presentation,
                },
            ],
        })
    }

    fn remove_caption_word(
        document: &mut EditDocument,
        presentation: Presentation,
        cue_id: &str,
        word_index: usize,
    ) -> Result<Self, CommandError> {
        let index = document.cue_index(presentation, cue_id)?;
        let word_id = document.captions.list(presentation)[index]
            .words()
            .nth(word_index)
            .ok_or(CommandError::NoSuchWord(word_index))?
            .word_id
            .clone();
        let inverse = Self::capture(document);
        for list in [Presentation::Reading, Presentation::BurnIn] {
            let cues = document.captions.list_mut(list);
            for cue in cues.iter_mut() {
                let mut position = 0;
                for line in &mut cue.lines {
                    line.words.retain(|word| {
                        let remove = word_id.as_ref().map_or(
                            list == presentation && cue.cue_id == cue_id && position == word_index,
                            |id| word.word_id.as_ref() == Some(id),
                        );
                        position += 1;
                        !remove
                    });
                }
                cue.lines.retain(|line| !line.words.is_empty());
            }
            cues.retain(|cue| !cue.lines.is_empty());
        }
        Ok(inverse)
    }

    fn apply_merge_cues(
        document: &mut EditDocument,
        presentation: Presentation,
        first_cue_id: &str,
        second_cue_id: &str,
    ) -> Result<Self, CommandError> {
        let first_index = document.cue_index(presentation, first_cue_id)?;
        let second_index = document.cue_index(presentation, second_cue_id)?;
        if second_index != first_index.saturating_add(1) {
            return Err(CommandError::CuesNotAdjacent);
        }
        let cues = document.captions.list_mut(presentation);
        let second = cues.remove(second_index);
        let first = cues
            .get_mut(first_index)
            .ok_or_else(|| DocumentError::UnknownCue(first_cue_id.to_owned()))?;
        let first_counts = first.line_word_counts();
        let second_counts = second.line_word_counts();
        let split_at = first.word_count();
        first.end_ticks = second.end_ticks;
        first.lines.extend(second.lines);
        Ok(Self::Batch {
            commands: vec![
                Self::SplitCue {
                    cue_id: first_cue_id.to_owned(),
                    at_word_index: split_at,
                    new_cue_id: second_cue_id.to_owned(),
                    presentation,
                },
                Self::SetCueLines {
                    cue_id: first_cue_id.to_owned(),
                    line_word_counts: first_counts,
                    presentation,
                },
                Self::SetCueLines {
                    cue_id: second_cue_id.to_owned(),
                    line_word_counts: second_counts,
                    presentation,
                },
            ],
        })
    }

    pub fn from_canonical_json(bytes: &[u8]) -> Result<Self, CommandError> {
        serde_json::from_slice(bytes).map_err(|error| CommandError::Json(error.to_string()))
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, CommandError> {
        serde_json_canonicalizer::to_vec(self)
            .map_err(|error| CommandError::Json(error.to_string()))
    }
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("speaker switch is available only in a two-person section")]
    NotTwoUp,
    #[error("unknown caption preset {0}")]
    UnknownCaptionStyle(String),
    #[error("captions can show between one and eight words at a time")]
    InvalidWordsOnScreen,
    #[error("the new section id {0} already exists")]
    SegmentAlreadyExists(String),
    #[error("split must be inside the selected section")]
    SplitOutsideSegment,
    #[error("only the beginning or end of a clip can be extended")]
    ExtensionMustGrowEdge,
    #[error(transparent)]
    Document(#[from] DocumentError),
    #[error("time ranges must be non-empty and start at or after zero")]
    EmptyRange,
    #[error("no crop keyframe at tick {0}")]
    NoCropKeyframe(i64),
    #[error("no gain point at tick {0}")]
    NoGainPoint(i64),
    #[error("no word at index {0}")]
    NoSuchWord(usize),
    #[error("a cue can only be split between two of its words, not at index {0}")]
    SplitOutsideCue(usize),
    #[error("cue {0} already exists")]
    CueAlreadyExists(String),
    #[error("overlay {0} already exists")]
    OverlayAlreadyExists(String),
    #[error("an overlay cannot go at place {0} in the stack")]
    OverlayOutOfStack(usize),
    #[error("only adjacent cues can be merged")]
    CuesNotAdjacent,
    #[error("caption timing must stay inside the clip")]
    CaptionTimingOutsideProgram,
    #[error(
        "caption timing overlaps a neighbouring caption; shorten the display window or merge the captions"
    )]
    CaptionTimingOverlaps,
    #[error(
        "caption timing must include all its spoken words; extend the display window instead of cutting through speech"
    )]
    CaptionTimingExcludesWords,
    #[error("edit command is not valid JSON: {0}")]
    Json(String),
}
