//! Captions derived afresh for a whole edited program.
//!
//! A clip's captions are derived once, when it is directed, from the
//! transcript and the rules of that moment. A later transcript — an analysis
//! run again — or later rules — a repaired segmenter — reach a clip made
//! before them only by deriving again. Each section is derived over its own
//! span, as a clip of one section is, and its cues are moved to where the
//! section plays in the program.

use std::collections::HashSet;

use clipmill_contracts::schemas::{
    evidence_shots::EvidenceShots, index_transcript::IndexTranscript,
    speech_transcript::SpeechTranscript,
};
use clipmill_edit_ir::{CaptionCue, EditDocument};

use crate::{Boundary, DirectError, captions_for_span};

/// Both presentations' cues for the whole program, reading then burned in,
/// in program time.
///
/// A section edge that cuts through a word — which a newer transcript can do
/// to an edit made on an older one — keeps that word out of the captions
/// rather than refusing the refresh. Burned-in cues keep the lane the clip's
/// captions were in.
pub fn captions_for_program(
    transcript: &SpeechTranscript,
    index: Option<&IndexTranscript>,
    shots: Option<&EvidenceShots>,
    document: &EditDocument,
) -> Result<(Vec<CaptionCue>, Vec<CaptionCue>), DirectError> {
    let first = i64::try_from(transcript.coverage.start_ticks).unwrap_or(i64::MAX);
    let last = i64::try_from(transcript.coverage.end_ticks).unwrap_or(i64::MAX);
    let lane = document.captions.burned().first().map(|cue| cue.region);
    let several = document.video.segments.len() > 1;
    let mut reading: Vec<CaptionCue> = Vec::new();
    let mut burn_in: Vec<CaptionCue> = Vec::new();
    let mut used: HashSet<String> = HashSet::new();
    let starts = document.segment_program_starts();
    for (position, (segment, program_start)) in
        document.video.segments.iter().zip(starts).enumerate()
    {
        let Some(span) = whole_words_within(
            transcript,
            Boundary {
                start_ticks: segment.in_ticks.max(first),
                end_ticks: segment.out_ticks.min(last),
            },
        ) else {
            continue;
        };
        let track =
            captions_for_span(transcript, index, shots, span, &document.captions.style_ref)?;
        let offset = program_start + (span.start_ticks - segment.in_ticks);
        // A word two sections both play would otherwise carry one id twice.
        let repeated: HashSet<String> = track
            .cues
            .iter()
            .flat_map(CaptionCue::words)
            .filter_map(|word| word.word_id.clone())
            .filter(|id| used.contains(id))
            .collect();
        let placed = |mut cue: CaptionCue| {
            cue.start_ticks += offset;
            cue.end_ticks += offset;
            if several {
                cue.cue_id = format!("s{}_{}", position + 1, cue.cue_id);
            }
            for line in &mut cue.lines {
                for word in &mut line.words {
                    word.start_ticks += offset;
                    word.end_ticks += offset;
                    if let Some(id) = word.word_id.as_mut()
                        && repeated.contains(id.as_str())
                    {
                        *id = format!("{id}~s{}", position + 1);
                    }
                }
            }
            cue
        };
        used.extend(
            track
                .cues
                .iter()
                .flat_map(CaptionCue::words)
                .filter_map(|word| word.word_id.clone()),
        );
        reading.extend(track.cues.into_iter().map(placed));
        burn_in.extend(track.burn_in.into_iter().map(placed).map(|mut cue| {
            if let Some(region) = lane {
                cue.region = region;
            }
            cue
        }));
    }
    Ok((reading, burn_in))
}

/// The span with each edge moved inward past any word it cuts through, or
/// `None` when no whole word is left inside it.
fn whole_words_within(transcript: &SpeechTranscript, mut span: Boundary) -> Option<Boundary> {
    // Each pass moves an edge strictly inward or stops, and an edge can pass
    // each word once, so the word count bounds the passes.
    for _ in 0..=transcript.words.len() {
        let mut moved = false;
        for word in &transcript.words {
            let (a, b) = (
                i64::try_from(word.start_ticks).unwrap_or(i64::MAX),
                i64::try_from(word.end_ticks).unwrap_or(i64::MAX),
            );
            let (from, to) = (a.min(b), a.max(b));
            if from < span.start_ticks && span.start_ticks < to {
                span.start_ticks = to;
                moved = true;
            }
            if from < span.end_ticks && span.end_ticks < to {
                span.end_ticks = from;
                moved = true;
            }
        }
        if !moved {
            break;
        }
    }
    (span.end_ticks > span.start_ticks).then_some(span)
}
