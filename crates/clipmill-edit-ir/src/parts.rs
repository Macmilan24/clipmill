//! Group edits for a directed plan. Each continuous run is spliced with its
//! programme-time tracks, so corrected captions, cutaways and gain follow the picture.
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Range,
};

use crate::{
    CaptionCue, CommandError, EditCommand, EditDocument, Join, JoinKind, PartRef, Presentation,
    RemovedRange, VideoSegment,
};

fn part_groups(document: &EditDocument) -> Result<Vec<Range<usize>>, CommandError> {
    let mut groups: Vec<Range<usize>> = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, segment) in document.video.segments.iter().enumerate() {
        let id = &segment
            .part
            .as_ref()
            .ok_or(CommandError::InvalidPart)?
            .part_id;
        if let Some(last) = groups.last_mut()
            && document.video.segments[last.start]
                .part
                .as_ref()
                .is_some_and(|p| &p.part_id == id)
        {
            last.end = index + 1;
        } else {
            if !seen.insert(id) {
                return Err(CommandError::InvalidPart);
            }
            groups.push(index..index + 1);
        }
    }
    Ok(groups)
}

fn original_part<'a>(segments: &'a [VideoSegment], range: &RemovedRange) -> Option<&'a PartRef> {
    if let Some(id) = &range.part_id {
        return segments
            .iter()
            .filter_map(|s| s.part.as_ref())
            .find(|part| &part.part_id == id);
    }
    // Older documents carry only source times. Prefer the pair of adjoining
    // sections, which distinguishes a repeated hook from the tightened body.
    let matches = |s: &VideoSegment| s.source_fingerprint == range.source_fingerprint;
    segments
        .windows(2)
        .find(|pair| {
            matches(&pair[0])
                && matches(&pair[1])
                && pair[0].out_ticks == range.in_ticks
                && pair[1].in_ticks == range.out_ticks
                && pair[0].part.is_some()
                && pair[0].part == pair[1].part
        })
        .and_then(|pair| pair[0].part.as_ref())
        .or_else(|| {
            segments
                .iter()
                .find(|s| {
                    matches(s) && (s.out_ticks == range.in_ticks || s.in_ticks == range.out_ticks)
                })
                .and_then(|s| s.part.as_ref())
        })
}

pub(super) fn arrange(
    document: &mut EditDocument,
    id: &str,
    destination: Option<usize>,
) -> Result<EditCommand, CommandError> {
    let mut groups = part_groups(document)?;
    let old = groups
        .iter()
        .position(|range| {
            document.video.segments[range.start]
                .part
                .as_ref()
                .is_some_and(|p| p.part_id == id)
        })
        .ok_or(CommandError::InvalidPart)?;
    let inverse = EditCommand::capture(document);
    let moving = groups.remove(old);
    if let Some(to) = destination {
        if to > groups.len() {
            return Err(CommandError::InvalidPart);
        }
        if old == to {
            return Ok(inverse);
        }
        groups.insert(to, moving);
    } else if groups.is_empty() {
        return Err(CommandError::LastPart);
    }
    // There are at most four uninterrupted intervals after a move. Preserve
    // the exact grouping of everything inside each one.
    let mut runs: Vec<Range<usize>> = Vec::new();
    for group in groups {
        if let Some(last) = runs.last_mut()
            && last.end == group.start
        {
            last.end = group.end;
        } else {
            runs.push(group);
        }
    }
    let original = document.clone();
    clear_tracks(document);
    for run in runs {
        let begins = document.video.segments.len();
        append_interval(document, &original, run);
        // Retain the sound design of the incoming part and describe the new cut.
        if begins == 0 {
            document.video.segments[0].join = None;
        } else {
            let before = &document.video.segments[begins - 1];
            let incoming = &document.video.segments[begins];
            let kind = if before.source_fingerprint != incoming.source_fingerprint
                || incoming.in_ticks > before.out_ticks
            {
                JoinKind::Distant
            } else if incoming.in_ticks < before.out_ticks {
                JoinKind::Reorder
            } else {
                JoinKind::Adjacent
            };
            let join = document.video.segments[begins].join.get_or_insert(Join {
                kind,
                audio_crossfade_ticks: 0,
                room_tone: false,
                room_tone_range: None,
            });
            join.kind = kind;
        }
    }
    if destination.is_none()
        && let Some(plan) = &mut document.plan
    {
        plan.removed.retain(|range| {
            original_part(&original.video.segments, range).is_none_or(|p| p.part_id != id)
        });
    }
    Ok(inverse)
}

fn clear_tracks(document: &mut EditDocument) {
    document.video.segments.clear();
    document.video.cutaways.clear();
    document.captions.cues.clear();
    document.captions.burn_in.clear();
    document.audio.gain_curve.clear();
    document.overlays.clear();
}

/// An item crossing a newly separated part boundary has a fragment on each
/// side. Only the second fragment needs a different identity.
fn append_distinct<T>(list: &mut Vec<T>, incoming: Vec<T>, id: impl Fn(&mut T) -> &mut String) {
    let mut seen: BTreeSet<String> = list.iter_mut().map(|item| id(item).clone()).collect();
    for mut item in incoming {
        let key = id(&mut item);
        if !seen.insert(key.clone()) {
            let mut suffix = 1;
            loop {
                let candidate = format!("{key}~{suffix}");
                if seen.insert(candidate.clone()) {
                    *key = candidate;
                    break;
                }
                suffix += 1;
            }
        }
        list.push(item);
    }
}

fn append_interval(out: &mut EditDocument, source: &EditDocument, range: Range<usize>) {
    if range.is_empty() {
        return;
    }
    let original_end = source.program_duration_ticks();
    let starts = source.segment_program_starts();
    let start = starts[range.start];
    let end = starts.get(range.end).copied().unwrap_or(original_end);
    let mut piece = source.clone();
    if end < original_end {
        piece.splice_program_content(end, original_end - end, 0, original_end);
    }
    if start > 0 {
        piece.splice_program_content(0, start, 0, end);
    }
    let offset = out.program_duration_ticks();
    // A gain key on the last tick boundary belongs to the interval following it.
    piece
        .audio
        .gain_curve
        .retain(|p| p.t_ticks < end - start || end == original_end);
    if offset > 0 {
        piece.splice_program_content(0, 0, offset, end - start);
    }
    out.video
        .segments
        .extend_from_slice(&source.video.segments[range]);
    for presentation in [Presentation::Reading, Presentation::BurnIn] {
        append_distinct(
            out.captions.list_mut(presentation),
            std::mem::take(piece.captions.list_mut(presentation)),
            |cue| &mut cue.cue_id,
        );
    }
    if let Some(first) = piece.audio.gain_curve.first() {
        out.audio.gain_curve.retain(|p| p.t_ticks < first.t_ticks);
    }
    out.audio.gain_curve.append(&mut piece.audio.gain_curve);
    append_distinct(&mut out.overlays, piece.overlays, |item| {
        &mut item.overlay_id
    });
    append_distinct(&mut out.video.cutaways, piece.video.cutaways, |item| {
        &mut item.cutaway_id
    });
}

/// Newly exposed words may have played already in a cold open. Give that
/// occurrence one new ID shared by reading and burned-in captions.
fn restored_words(piece: &mut EditDocument, original: &EditDocument) {
    let mut taken: BTreeSet<String> = original
        .captions
        .cues
        .iter()
        .chain(&original.captions.burn_in)
        .flat_map(CaptionCue::words)
        .filter_map(|word| word.word_id.clone())
        .collect();
    let mut renamed = BTreeMap::new();
    for cue in piece
        .captions
        .cues
        .iter_mut()
        .chain(&mut piece.captions.burn_in)
    {
        for word in cue.lines.iter_mut().flat_map(|line| &mut line.words) {
            if let Some(id) = &mut word.word_id {
                let replacement = renamed.entry(id.clone()).or_insert_with(|| {
                    let mut candidate = id.clone();
                    while !taken.insert(candidate.clone()) {
                        candidate.push_str("~restored");
                    }
                    candidate
                });
                id.clone_from(replacement);
            }
        }
    }
}

pub(super) fn restore(
    document: &mut EditDocument,
    index: usize,
    sections: &[VideoSegment],
    reading: &[CaptionCue],
    burn_in: &[CaptionCue],
) -> Result<EditCommand, CommandError> {
    let unavailable = || CommandError::RemovedRangeUnavailable;
    let range = document
        .plan
        .as_ref()
        .and_then(|plan| plan.removed.get(index))
        .ok_or_else(unavailable)?;
    let owner = original_part(&document.video.segments, range).ok_or_else(unavailable)?;
    let group = part_groups(document)?
        .into_iter()
        .find(|group| document.video.segments[group.start].part.as_ref() == Some(owner))
        .ok_or_else(unavailable)?;
    let sections_before = &document.video.segments[group.clone()];
    if sections_before.iter().any(|s| {
        s.source_fingerprint != range.source_fingerprint
            || (s.in_ticks < range.out_ticks && s.out_ticks > range.in_ticks)
    }) {
        return Err(unavailable());
    }
    let slot = sections_before
        .iter()
        .position(|s| s.in_ticks >= range.out_ticks)
        .map_or(group.end, |n| group.start + n);
    let mut piece = EditDocument::default();
    let mut cursor = range.in_ticks;
    let mut ids: Vec<_> = document
        .video
        .segments
        .iter()
        .map(|s| s.segment_id.clone())
        .collect();
    for mut section in sections.iter().cloned() {
        if section.source_fingerprint != range.source_fingerprint
            || section.in_ticks != cursor
            || section.out_ticks <= cursor
        {
            return Err(unavailable());
        }
        cursor = section.out_ticks;
        section.part = Some(owner.clone());
        section.join = None;
        section.segment_id = EditDocument::derive_id(&ids, "restored");
        ids.push(section.segment_id.clone());
        piece.video.segments.push(section);
    }
    if cursor != range.out_ticks {
        return Err(unavailable());
    }
    piece.captions.cues = reading.to_vec();
    piece.captions.burn_in = burn_in.to_vec();
    restored_words(&mut piece, document);
    let inverse = EditCommand::capture(document);
    let original = document.clone();
    clear_tracks(document);
    append_interval(document, &original, 0..slot);
    append_interval(document, &piece, 0..sections.len());
    append_interval(document, &original, slot..original.video.segments.len());
    if let Some(next) = document.video.segments.get_mut(slot + sections.len())
        && next.join.is_some_and(|join| join.kind == JoinKind::Removed)
        && next.in_ticks == cursor
    {
        next.join = None;
    }
    if let Some(plan) = &mut document.plan {
        plan.removed.remove(index);
    }
    Ok(inverse)
}
