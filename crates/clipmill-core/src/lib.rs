//! Core primitives shared by every ClipMill crate.

mod digest;
mod id;

pub use digest::{ArtifactId, DigestError, Sha256Digest};
pub use id::{
    EditDocId, EditDocIdError, IdError, JobId, JobIdError, LeaseId, LeaseIdError, ProjectId,
    SourceId, SourceIdError, StagingId, StagingIdError, TaskEventCursor, TaskEventCursorError,
    TaskId, TaskIdError, WorkerId, WorkerIdError,
};
