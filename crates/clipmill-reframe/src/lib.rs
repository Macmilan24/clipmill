//! The virtual camera.
//!
//! Two decisions, kept apart because they fail differently. **Who to follow** is
//! a judgement over evidence that can be wrong in a way nobody sees — a
//! confident crop of the wrong person — so it is gated, and falling short of the
//! gate produces a fitted frame and a sentence saying why. **How to follow** is
//! arithmetic: a least-squares path whose damping terms exist to stop the camera
//! chasing detection flicker, solved as a banded system in microseconds so an
//! interactive nudge costs nothing.
//!
//! Nothing here writes anything. A solve returns a proposal; whether it becomes
//! part of an edit is the caller's decision, which is what makes re-solving free
//! and what keeps an accepted edit from being mutated by a re-run.

mod banded;
mod solver;
mod tracks;

pub use banded::BandedError;
pub use solver::{CropPath, Keyframe, SolveError, Weights, solve};
pub use tracks::{FitReason, Focus, FocusGate, resolve};

#[cfg(test)]
mod testing;
