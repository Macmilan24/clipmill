//! Audio envelopes applied at the exact programme cut, shared by preview and export.

use clipmill_edit_ir::{EditDocument, Join};

/// Length of each quarter-sine ramp, clamped to the section it plays over.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct JoinEdges {
    pub fade_in: i64,
    pub fade_out: i64,
    pub room_tone_start: Option<i64>,
}

pub(crate) fn edges(document: &EditDocument) -> Vec<JoinEdges> {
    let segments = &document.video.segments;
    let mut edges = vec![JoinEdges::default(); segments.len()];
    for index in 1..segments.len() {
        let Some(join) = segments[index].join else {
            continue;
        };
        let half = join.blend_ticks().clamp(0, Join::MAX_CROSSFADE_TICKS) / 2;
        let outgoing = half.min(segments[index - 1].duration_ticks() / 2);
        let incoming = half.min(segments[index].duration_ticks() / 2);
        edges[index - 1].fade_out = outgoing;
        edges[index].fade_in = incoming;
        if join.room_tone
            && outgoing > 0
            && incoming > 0
            && let Some([start, end]) = join.room_tone_range
            && start >= 0
            && end.saturating_sub(start) >= outgoing + incoming
        {
            edges[index].room_tone_start = Some(start);
        }
    }
    edges
}

/// One recording excerpt placed across the dip in the programme's speech.
#[derive(Clone, Debug)]
pub(crate) struct RoomTone {
    pub source: String,
    pub source_start: i64,
    pub program_start: i64,
    pub rise: i64,
    pub fall: i64,
}

pub(crate) fn room_tones(
    document: &EditDocument,
    edges: &[JoinEdges],
    sources: &[crate::SourceInput],
) -> Vec<RoomTone> {
    let starts = document.segment_program_starts();
    edges
        .windows(2)
        .enumerate()
        .filter_map(|(before, pair)| {
            let source_start = pair[1].room_tone_start?;
            let index = before + 1;
            let source = &document.video.segments[index].source_fingerprint;
            let (rise, fall) = (pair[0].fade_out, pair[1].fade_in);
            let input = sources.iter().find(|input| input.fingerprint == *source)?;
            if !input.has_audio
                || (input.duration_ticks > 0
                    && source_start.saturating_add(rise + fall) > input.duration_ticks)
            {
                return None;
            }
            Some(RoomTone {
                source: source.clone(),
                source_start,
                program_start: starts[index] - rise,
                rise,
                fall,
            })
        })
        .collect()
}
