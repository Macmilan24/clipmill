//! Core primitives shared by every ClipMill crate.

mod digest;
mod id;

pub use digest::{ArtifactId, DigestError, Sha256Digest};
pub use id::{IdError, ProjectId, StagingId, StagingIdError};
