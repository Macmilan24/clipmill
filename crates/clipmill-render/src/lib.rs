//! The render compiler: Edit IR in, deterministic execution graph out.
//!
//! Rendering is model-free by construction (book ch. 17). Everything this
//! crate does is arithmetic over the document: which source frames a segment
//! decodes, where the crop window sits on each output frame, which frame a
//! caption first appears on, what the encoder is told. No network, no model,
//! no clock, no filesystem — compilation is a pure function, which is what
//! lets the daemon key an artifact on the plan and treat a warm render as a
//! lookup rather than a re-encode.
//!
//! Two properties are worth stating plainly because everything else leans on
//! them:
//!
//! * **Frames are decided once.** Every timestamp the renderer emits comes
//!   from an integer frame index computed by [`FrameRate`], so a cue's first
//!   frame is a fact the preview can be compared against rather than the
//!   result of whichever rounding a call site reached for.
//! * **Nothing is re-decided downstream.** Line breaks come from the document
//!   and libass is configured never to re-wrap; the crop path is interpolated
//!   by one function whose emitted expression mirrors it branch for branch.
//!   A second implementation of either would be a parity bug with a head
//!   start.

mod graph;
mod manifest;
mod plan;
mod profile;
mod subtitles;
mod timing;

pub use graph::{DecodeSpan, FilterGraph, LOUDNORM_SLOT, crop_rect_at};
pub use manifest::{
    AiUseSummary, CaptionWindow, EngineIdentity, LoudnessReport, MeasuredLoudness, OutputFile,
    ProgramReport, ProgramSegment, RenderManifest, RightsAttestation,
    SCHEMA_VERSION as MANIFEST_SCHEMA_VERSION,
};
pub use plan::{
    ASS_FILE, CLIP_FILE, LoudnessMeasurement, MANIFEST_FILE, RenderError, RenderPlan, SRT_FILE,
    SegmentReport, SourceInput, VTT_FILE, compile,
};
pub use profile::{
    CaptionStyle, Colour, DEFAULT_STYLE_REF, FONT_FAMILY, FONTS_DIR, FrameRateSpec, LoudnessTarget,
    PROFILE_ID, RenderProfile,
};
pub use subtitles::{CueWindow, unrenderable_character};
pub use timing::{FrameRate, centis_to_ass, millis_to_srt, millis_to_vtt, ticks_to_seconds};
