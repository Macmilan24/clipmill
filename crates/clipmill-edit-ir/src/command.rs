use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::document::{
    CaptionCue, CropKeyframe, CropRect, DocumentError, EditDocument, GainPoint, LayoutState,
    VideoSegment,
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
    },
    /// Restore a previously captured arrangement. This is the inverse of
    /// every command that can destroy material.
    RestoreArrangement {
        segments: Vec<VideoSegment>,
        cues: Vec<CaptionCue>,
        gain_curve: Vec<GainPoint>,
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
    /// Correct one word's text without disturbing its timing.
    EditCaptionText {
        cue_id: String,
        word_index: usize,
        text: String,
    },
    /// Re-flow a cue's words into lines. Line breaks are stored, never
    /// recomputed at render time.
    SetCueLines {
        cue_id: String,
        line_word_counts: Vec<usize>,
    },
    /// Split a cue at a word boundary. The caller names the new cue so that
    /// replay reproduces the same identifier.
    SplitCue {
        cue_id: String,
        at_word_index: usize,
        new_cue_id: String,
    },
    MergeCues {
        first_cue_id: String,
        second_cue_id: String,
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
            Self::Trim {
                segment_id,
                in_ticks,
                out_ticks,
            } => Self::apply_trim(document, segment_id, *in_ticks, *out_ticks),
            Self::RippleDelete {
                start_ticks,
                end_ticks,
            } => Self::apply_ripple_delete(document, *start_ticks, *end_ticks),
            Self::RestoreArrangement {
                segments,
                cues,
                gain_curve,
            } => {
                let inverse = Self::capture(document);
                document.video.segments.clone_from(segments);
                document.captions.cues.clone_from(cues);
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
            Self::SetCropKeyframe {
                segment_id,
                t_ticks,
                rect,
            } => {
                let index = document.segment_index(segment_id)?;
                let segment = document
                    .video
                    .segments
                    .get_mut(index)
                    .ok_or_else(|| DocumentError::UnknownSegment(segment_id.clone()))?;
                let path = &mut segment.layout.crop_path;
                match path.binary_search_by_key(t_ticks, |keyframe| keyframe.t_ticks) {
                    Ok(position) => {
                        let previous = path[position].rect;
                        path[position].rect = *rect;
                        Ok(Self::SetCropKeyframe {
                            segment_id: segment_id.clone(),
                            t_ticks: *t_ticks,
                            rect: previous,
                        })
                    }
                    Err(position) => {
                        path.insert(
                            position,
                            CropKeyframe {
                                t_ticks: *t_ticks,
                                rect: *rect,
                            },
                        );
                        Ok(Self::RemoveCropKeyframe {
                            segment_id: segment_id.clone(),
                            t_ticks: *t_ticks,
                        })
                    }
                }
            }
            Self::RemoveCropKeyframe {
                segment_id,
                t_ticks,
            } => {
                let index = document.segment_index(segment_id)?;
                let segment = document
                    .video
                    .segments
                    .get_mut(index)
                    .ok_or_else(|| DocumentError::UnknownSegment(segment_id.clone()))?;
                let path = &mut segment.layout.crop_path;
                let position = path
                    .binary_search_by_key(t_ticks, |keyframe| keyframe.t_ticks)
                    .map_err(|_| CommandError::NoCropKeyframe(*t_ticks))?;
                let removed = path.remove(position);
                Ok(Self::SetCropKeyframe {
                    segment_id: segment_id.clone(),
                    t_ticks: *t_ticks,
                    rect: removed.rect,
                })
            }
            Self::EditCaptionText {
                cue_id,
                word_index,
                text,
            } => {
                let index = document.cue_index(cue_id)?;
                let cue = document
                    .captions
                    .cues
                    .get_mut(index)
                    .ok_or_else(|| DocumentError::UnknownCue(cue_id.clone()))?;
                let mut cursor = 0_usize;
                for line in &mut cue.lines {
                    for word in &mut line.words {
                        if cursor == *word_index {
                            let previous = std::mem::replace(&mut word.text, text.clone());
                            return Ok(Self::EditCaptionText {
                                cue_id: cue_id.clone(),
                                word_index: *word_index,
                                text: previous,
                            });
                        }
                        cursor += 1;
                    }
                }
                Err(CommandError::NoSuchWord(*word_index))
            }
            Self::SetCueLines {
                cue_id,
                line_word_counts,
            } => {
                let index = document.cue_index(cue_id)?;
                let cue = document
                    .captions
                    .cues
                    .get_mut(index)
                    .ok_or_else(|| DocumentError::UnknownCue(cue_id.clone()))?;
                let previous = cue.line_word_counts();
                cue.reflow(line_word_counts)?;
                Ok(Self::SetCueLines {
                    cue_id: cue_id.clone(),
                    line_word_counts: previous,
                })
            }
            Self::SplitCue {
                cue_id,
                at_word_index,
                new_cue_id,
            } => Self::apply_split_cue(document, cue_id, *at_word_index, new_cue_id),
            Self::MergeCues {
                first_cue_id,
                second_cue_id,
            } => Self::apply_merge_cues(document, first_cue_id, second_cue_id),
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
        }
    }

    fn apply_trim(
        document: &mut EditDocument,
        segment_id: &str,
        in_ticks: i64,
        out_ticks: i64,
    ) -> Result<Self, CommandError> {
        if out_ticks <= in_ticks || in_ticks < 0 {
            return Err(CommandError::EmptyRange);
        }
        let prior = Self::capture(document);
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
        let keyframes_before = segment.layout.crop_path.len();
        segment.in_ticks = in_ticks;
        segment.out_ticks = out_ticks;
        segment.layout.crop_path = EditDocument::retime_crop_path(
            &segment.layout.crop_path,
            old_in,
            in_ticks,
            new_duration,
        );
        let keyframes_lost = segment.layout.crop_path.len() != keyframes_before;
        let old_end = program_start.saturating_add(old_duration);
        let new_end = program_start.saturating_add(new_duration);
        let content_lost = if new_duration < old_duration {
            document.splice_program_content(new_end, old_end.saturating_sub(new_end), 0)
        } else {
            document.splice_program_content(old_end, 0, new_end.saturating_sub(old_end))
        };
        if keyframes_lost || content_lost {
            // Shortening a segment can strand captions in the tail it gave up
            // and push crop keyframes outside the new window. Restoring the
            // whole prior arrangement is the only exact undo once material is
            // gone; the narrow `Trim` inverse is kept for the common case
            // where nothing was lost.
            return Ok(prior);
        }
        Ok(Self::Trim {
            segment_id: segment_id.to_owned(),
            in_ticks: old_in,
            out_ticks: old_out,
        })
    }

    fn apply_ripple_delete(
        document: &mut EditDocument,
        start_ticks: i64,
        end_ticks: i64,
    ) -> Result<Self, CommandError> {
        if end_ticks <= start_ticks || start_ticks < 0 {
            return Err(CommandError::EmptyRange);
        }
        let inverse = Self::capture(document);
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
                let mut tail = segment.clone();
                tail.segment_id = EditDocument::derive_id(&existing_ids, &segment.segment_id);
                tail.in_ticks = tail_in;
                tail.layout.crop_path = EditDocument::retime_crop_path(
                    &segment.layout.crop_path,
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
            kept.push(trimmed);
        }
        document.video.segments = kept;
        document.splice_program_content(start_ticks, span, 0);
        Ok(inverse)
    }

    fn apply_split_cue(
        document: &mut EditDocument,
        cue_id: &str,
        at_word_index: usize,
        new_cue_id: &str,
    ) -> Result<Self, CommandError> {
        if new_cue_id.is_empty() {
            return Err(DocumentError::EmptyIdentifier.into());
        }
        if document
            .captions
            .cues
            .iter()
            .any(|cue| cue.cue_id == new_cue_id)
        {
            return Err(CommandError::CueAlreadyExists(new_cue_id.to_owned()));
        }
        let index = document.cue_index(cue_id)?;
        let cue = document
            .captions
            .cues
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
        document.captions.cues.splice(index..=index, [head, tail]);
        Ok(Self::Batch {
            commands: vec![
                Self::MergeCues {
                    first_cue_id: cue_id.to_owned(),
                    second_cue_id: new_cue_id.to_owned(),
                },
                Self::SetCueLines {
                    cue_id: cue_id.to_owned(),
                    line_word_counts: original_counts,
                },
            ],
        })
    }

    fn apply_merge_cues(
        document: &mut EditDocument,
        first_cue_id: &str,
        second_cue_id: &str,
    ) -> Result<Self, CommandError> {
        let first_index = document.cue_index(first_cue_id)?;
        let second_index = document.cue_index(second_cue_id)?;
        if second_index != first_index.saturating_add(1) {
            return Err(CommandError::CuesNotAdjacent);
        }
        let second = document.captions.cues.remove(second_index);
        let first = document
            .captions
            .cues
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
                },
                Self::SetCueLines {
                    cue_id: first_cue_id.to_owned(),
                    line_word_counts: first_counts,
                },
                Self::SetCueLines {
                    cue_id: second_cue_id.to_owned(),
                    line_word_counts: second_counts,
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
