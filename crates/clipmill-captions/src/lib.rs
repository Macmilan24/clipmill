//! Caption segmentation, intent profiles, and independent validation.
//!
//! Cue-local costs make segmentation an exact shortest-path problem over token
//! boundaries. Burn-in and accessibility profiles group the same tokens with
//! different timing and density targets, so their words remain identical.
//! Validation re-derives metrics from finished cues: a minimum-cost partition can
//! still violate a profile on dense speech.

mod document;
pub mod fonts;
pub mod lexicon;
pub mod presets;
pub mod profile;
pub mod segment;
pub mod validate;

pub use document::{DeriveError, DeriveRequest, Inputs, captionable_word, derive};
pub use fonts::{CaptionFont, DEFAULT_FONT, FONTS, font};
pub use lexicon::{Break, FILLER_LEXICON};
pub use presets::{Animation, Border, Colour, DEFAULT_STYLE_REF, PRESETS, Preset, preset};
pub use profile::{Direction, Profile, Profiles, TICKS_PER_SECOND};
pub use segment::{Cue, Line, SegmentError, Span, Token, Weights, segment};
pub use validate::{CueFacts, Violation, validate};
