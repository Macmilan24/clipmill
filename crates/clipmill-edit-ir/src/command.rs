use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    document::{
        CaptionCue, CropKeyframe, CropRect, DocumentError, EditDocument, GainPoint, LayoutState,
        Presentation, VideoSegment,
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
    /// Move a segment's source window. Program-anchored content after the
    /// segment follows; content stranded in a shortened tail is removed.
    Trim {
        segment_id: String,
        in_ticks: i64,
        out_ticks: i64,
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
    },
    SetLayout {
        segment_id: String,
        state: LayoutState,
    },
    /// Insert or replace a crop keyframe at a segment-local tick.
    SetCropKeyframe {
        segment_id: String,
        t_ticks: i64,
        rect: CropRect,
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
    },
    RemoveSecondaryCropKeyframe {
        segment_id: String,
        t_ticks: i64,
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
            Self::Trim {
                segment_id,
                in_ticks,
                out_ticks,
            } => Self::apply_trim(document, segment_id, *in_ticks, *out_ticks),
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
            } => {
                let inverse = Self::capture(document);
                document.video.segments.clone_from(segments);
                document.captions.cues.clone_from(cues);
                if let Some(burn_in) = burn_in {
                    document.captions.burn_in.clone_from(burn_in);
                }
                document.audio.gain_curve.clone_from(gain_curve);
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
            Self::SetCropKeyframe {
                segment_id,
                t_ticks,
                rect,
            } => Self::crop_command(document, segment_id, *t_ticks, Some(*rect), false),
            Self::RemoveCropKeyframe {
                segment_id,
                t_ticks,
            } => Self::crop_command(document, segment_id, *t_ticks, None, false),
            Self::SetSecondaryCropKeyframe {
                segment_id,
                t_ticks,
                rect,
            } => Self::crop_command(document, segment_id, *t_ticks, Some(*rect), true),
            Self::RemoveSecondaryCropKeyframe {
                segment_id,
                t_ticks,
            } => Self::crop_command(document, segment_id, *t_ticks, None, true),
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
        }
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
        let set = |rect| {
            if secondary {
                Self::SetSecondaryCropKeyframe {
                    segment_id: segment_id.to_owned(),
                    t_ticks,
                    rect,
                }
            } else {
                Self::SetCropKeyframe {
                    segment_id: segment_id.to_owned(),
                    t_ticks,
                    rect,
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
                path[position].rect = rect;
                Ok(set(previous))
            }
            (Err(position), Some(rect)) => {
                path.insert(position, CropKeyframe { t_ticks, rect });
                Ok(remove())
            }
            (Ok(position), None) => Ok(set(path.remove(position).rect)),
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
                head.layout.crop_path = EditDocument::retime_crop_path(
                    &segment.layout.crop_path,
                    segment.in_ticks,
                    segment.in_ticks,
                    head.duration_ticks(),
                );
                head.layout.secondary_crop_path = EditDocument::retime_crop_path(
                    &segment.layout.secondary_crop_path,
                    segment.in_ticks,
                    segment.in_ticks,
                    head.duration_ticks(),
                );
                let mut tail = segment.clone();
                tail.segment_id = EditDocument::derive_id(&existing_ids, &segment.segment_id);
                tail.in_ticks = tail_in;
                tail.layout.crop_path = EditDocument::retime_crop_path(
                    &segment.layout.crop_path,
                    segment.in_ticks,
                    tail_in,
                    tail.duration_ticks(),
                );
                tail.layout.secondary_crop_path = EditDocument::retime_crop_path(
                    &segment.layout.secondary_crop_path,
                    segment.in_ticks,
                    tail_in,
                    tail.duration_ticks(),
                );
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
            trimmed.layout.crop_path = EditDocument::retime_crop_path(
                &segment.layout.crop_path,
                segment.in_ticks,
                trimmed.in_ticks,
                trimmed.duration_ticks(),
            );
            trimmed.layout.secondary_crop_path = EditDocument::retime_crop_path(
                &segment.layout.secondary_crop_path,
                segment.in_ticks,
                trimmed.in_ticks,
                trimmed.duration_ticks(),
            );
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
    #[error("only adjacent cues can be merged")]
    CuesNotAdjacent,
    #[error("edit command is not valid JSON: {0}")]
    Json(String),
}
