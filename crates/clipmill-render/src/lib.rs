//! Compile Edit IR into a deterministic render execution graph.
//!
//! Compilation performs no I/O, model inference, or clock reads. Source frames,
//! crops, captions, and encoder settings are derived from the inputs for caching.
//! All emitted timestamps come from integer frame indices computed by
//! [`FrameRate`]. Line breaks are preserved with libass re-wrapping disabled;
//! crop expressions mirror the same interpolation used by preview.

pub mod captions;
mod graph;
mod manifest;
mod plan;
pub mod preview;
mod profile;
mod subtitles;
mod timing;
mod transitions;

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
pub use preview::{
    PreviewCrop, PreviewCue, PreviewGain, PreviewLine, PreviewPlan, PreviewWord, preview_plan,
    text_at,
};
pub use profile::{
    CaptionStyle, Colour, DEFAULT_STYLE_REF, FONT_FAMILY, FONTS_DIR, FrameRateSpec, LoudnessTarget,
    PROFILE_ID, RenderProfile,
};
pub use subtitles::{CueWindow, Sweep, unrenderable_character};
pub use timing::{FrameRate, centis_to_ass, millis_to_srt, millis_to_vtt, ticks_to_seconds};
pub use transitions::PreviewTransition;
