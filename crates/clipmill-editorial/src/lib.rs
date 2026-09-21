//! Editorial windows and validation around model proposals and reviews.
//!
//! One model per run reads overlapping windows that preserve sentence boundaries
//! and cover every sentence. Responses cite sentence indexes, never timestamps.
//! This crate performs no inference or I/O; the daemon supplies and publishes the
//! documents.

pub mod review;
pub mod validate;
pub mod windows;

pub use windows::{Budget, Inputs, WindowsError, windows};

/// Who cuts the windows, recorded in the document.
pub const STAGE: &str = "editorial-windows";
