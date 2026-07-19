//! Deterministic filesystem content-addressed storage for ClipMill artifacts.
//!
//! Callers provide a semantic recipe, write only inside the returned private
//! staging directory, and ask the store to publish an exact declared file set.
//! A committed directory appears atomically and is immutable thereafter.

mod manifest;
mod path;
mod recipe;
mod store;

pub use path::{ArtifactPath, ArtifactPathError};
pub use recipe::{ArtifactRecipe, NetworkPolicy, Producer, RecipeError, RecipeSpec, Timebase};
pub use store::{
    ArtifactError, ArtifactLease, ArtifactStore, CacheLookup, CacheMissReason, GcReport,
    PrepareOutcome, RecoveryReport, StagingArea, StorePaths,
};
