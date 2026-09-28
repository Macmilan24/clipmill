//! Compile Edit IR into a deterministic render execution graph.
//!
//! Compilation performs no I/O, model inference, or clock reads. Source frames,
//! crops, captions, and encoder settings are derived from the inputs for caching.
//! All emitted timestamps come from integer frame indices computed by
//! [`FrameRate`]. Line breaks are preserved with libass re-wrapping disabled;
//! crop expressions mirror the same interpolation used by preview.

pub mod captions;
mod cutaways;
mod graph;
mod kinetic;
mod manifest;
mod music;
mod overlays;
mod plan;
pub mod preview;
mod profile;
mod subtitles;
mod timing;
mod transitions;

pub use cutaways::{CUTAWAY_DIR, FootageInput, cutaway_picture_file, cutaway_pictures};
pub use graph::{
    DecodeSpan, FilterGraph, LOGO_FILE, LOUDNORM_SLOT, LogoPlace, crop_rect_at, logo_place,
};
pub use manifest::{
    AiUseSummary, AssetRight, CaptionWindow, EngineIdentity, LoudnessReport, MeasuredLoudness,
    OutputFile, ProgramReport, ProgramSegment, RenderManifest, RightsAttestation,
    SCHEMA_VERSION as MANIFEST_SCHEMA_VERSION,
};
pub use music::{MUSIC_FILE, music_envelope};
pub use overlays::{EMOJI_DIR, emoji_files};
pub use plan::{
    ASS_FILE, CLIP_FILE, LoudnessMeasurement, MANIFEST_FILE, RenderError, RenderPlan, SRT_FILE,
    SegmentReport, SourceInput, VTT_FILE, compile, largest_upscale,
};
pub use preview::{
    PreviewCrop, PreviewCue, PreviewCutaway, PreviewGain, PreviewLine, PreviewLogo, PreviewMusic,
    PreviewOverlay, PreviewPlan, PreviewProgress, PreviewWord, caption_ass, preview_plan, text_at,
};
pub use profile::{
    CaptionStyle, Colour, DEFAULT_STYLE_REF, DESIGN_HEIGHT, FONT_FAMILY, FONTS_DIR, FrameRateSpec,
    LoudnessTarget, OUTPUT_HEIGHTS, PROFILE_ID, RenderProfile, design_resolution,
};
pub use subtitles::{CueWindow, Sweep, unrenderable_character};
pub use timing::{FrameRate, centis_to_ass, millis_to_srt, millis_to_vtt, ticks_to_seconds};
pub use transitions::PreviewTransition;
