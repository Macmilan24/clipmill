//! Select a focus subject and propose a smoothed virtual-camera path.
//!
//! Insufficient focus evidence yields `Fit` with a reason. Accepted tracks drive a
//! banded least-squares solve whose damping suppresses detection jitter. Solving
//! returns a proposal without mutating an accepted edit or performing I/O.

mod banded;
mod solver;
mod tracks;

pub use banded::BandedError;
pub use solver::{CropPath, FrameGeometry, Keyframe, SolveError, Weights, solve, solve_in_frame};
pub use tracks::{FitReason, Focus, FocusGate, SpeakerGate, resolve, resolve_pair, speaker};

#[cfg(test)]
mod testing;
