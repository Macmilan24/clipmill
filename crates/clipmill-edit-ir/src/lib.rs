//! The Edit IR: one versioned, non-destructive document that preview,
//! render, and (later) NLE export all read (book ch. 17).
//!
//! Two rules make the rest of the system possible. First, **no subsystem
//! renders, previews, or exports from any other representation** — a feature
//! that "just needs to draw one overlay directly" is a parity bug with a head
//! start. Second, editing is **command-based**: every change is a typed,
//! serializable operation that returns its exact inverse, so undo/redo and
//! durable replay are the same mechanism rather than two that can disagree.
//!
//! Time is integer ticks at 1/90000 throughout (decision D06). Program time
//! is never stored: a segment's program position is the sum of the durations
//! before it, so a trim cannot leave a stale cached offset behind.

mod command;
mod document;

pub use command::{CommandError, EditCommand};
pub use document::{
    Asset, AudioTrack, CaptionAnimation, CaptionCue, CaptionLine, CaptionRegion, CaptionTrack,
    CaptionWord, CropKeyframe, CropRect, DocumentError, EditDocument, GainPoint, IR_VERSION,
    Layout, LayoutState, Rationale, TICKS_PER_SECOND, Timebase, VideoSegment, VideoTrack,
};
