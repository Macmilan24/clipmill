//! The editorial model's side of an analysis (plan, Milestone 2).
//!
//! One model per run proposes moments and reviews them. It reads the
//! transcript in **windows** — overlapping pieces that never split a
//! sentence and together cover every sentence — and it answers in sentence
//! indexes, never timestamps. This crate is the arithmetic around that model:
//! the windows it reads (here), and in later steps the validation that turns
//! its proposals into candidates and the checks its judgments are held to.
//! No model runs in this crate, and nothing here does I/O; the daemon reads
//! the documents in and publishes the documents out.

pub mod review;
pub mod validate;
pub mod windows;

pub use windows::{Budget, Inputs, WindowsError, windows};

/// Who cuts the windows, recorded in the document.
pub const STAGE: &str = "editorial-windows";
