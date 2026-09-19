//! The one frame allocation used by soft cuts in both playback and export.

use clipmill_edit_ir::{EditDocument, LayoutState, TICKS_PER_SECOND};

use crate::FrameRate;

/// Hold the outgoing composition over live incoming frames. Within the
/// half-open interval, outgoing alpha is `(end - frame) / (end - first)`.
/// Neither source intervals nor program duration change.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewTransition {
    pub incoming_segment_id: String,
    pub outgoing_frame: i64,
    pub first_frame: i64,
    pub end_frame: i64,
}

pub(crate) fn transitions(document: &EditDocument, rate: FrameRate) -> Vec<PreviewTransition> {
    if document.video.transition_ticks <= 0 {
        return Vec::new();
    }
    let denominator = i128::from(TICKS_PER_SECOND) * i128::from(rate.den);
    let numerator = i128::from(document.video.transition_ticks.min(22_500)) * i128::from(rate.num);
    let requested = i64::try_from((numerator + denominator / 2) / denominator).unwrap_or(0);
    let requested = requested.min(rate.frame_at(22_500));
    let mut result = Vec::new();
    let mut start = 0;
    let mut previous = None;
    for segment in &document.video.segments {
        let end = start + segment.duration_ticks();
        let first_frame = rate.frame_ceil(start);
        let end_frame = rate.frame_ceil(end);
        start = end;
        if first_frame == end_frame {
            continue;
        }
        let frames = requested.min((end_frame - first_frame) / 2);
        if let Some(state) = previous
            && !(state == LayoutState::Fit && segment.layout.state == LayoutState::Fit)
            && frames >= 2
        {
            result.push(PreviewTransition {
                incoming_segment_id: segment.segment_id.clone(),
                outgoing_frame: first_frame - 1,
                first_frame,
                end_frame: first_frame + frames,
            });
        }
        previous = Some(segment.layout.state);
    }
    result
}
