//! The caption engine.
//!
//! Captions are the most-read typography a creator ships, and they are read
//! over a moving picture by people who often cannot hear it. That is the whole
//! reason this is a crate and not a filter argument.
//!
//! Three things live here and each is separable.
//!
//! **The segmentation** is an exact optimization. Where to break a line is the
//! craft of captioning, and every consideration that goes into it — reading
//! speed, the quality of the break, not stranding an article, balanced lines,
//! how long a cue is held, never spanning a cut — is a cost on one candidate
//! cue. Costs that are local make the best partition a shortest path, and a
//! shortest path is exact.
//!
//! **The profiles** are the published numbers rather than invented ones, and
//! there are two of them at once. A burn-in may run hot, a few words at a time,
//! because that is the register a muted feed is read in. A sidecar may not,
//! ever. Both group the same tokens, so the two can differ in rhythm and can
//! never differ in words — which is the one divergence this product will not
//! host.
//!
//! **The validator** reads finished cues and re-derives every number from them.
//! The segmenter returning a minimum is not the same as the minimum being good
//! enough, and something that did not choose the cues has to be the one to say
//! so.

mod document;
pub mod lexicon;
pub mod presets;
pub mod profile;
pub mod segment;
pub mod validate;

pub use document::{DeriveError, DeriveRequest, Inputs, derive};
pub use lexicon::{Break, FILLER_LEXICON};
pub use presets::{Animation, Border, Colour, DEFAULT_STYLE_REF, PRESETS, Preset, preset};
pub use profile::{Direction, Profile, Profiles, TICKS_PER_SECOND};
pub use segment::{Cue, Line, SegmentError, Span, Token, Weights, segment};
pub use validate::{CueFacts, Violation, validate};
