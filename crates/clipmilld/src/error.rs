use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DaemonError {
    #[error("another ClipMill daemon already owns {0}")]
    AlreadyRunning(PathBuf),
    #[error("database actor stopped unexpectedly")]
    DatabaseActorStopped,
    #[error("database actor thread panicked")]
    DatabaseActorPanicked,
    #[error("artifact actor stopped unexpectedly")]
    ArtifactActorStopped,
    #[error("artifact actor thread panicked")]
    ArtifactActorPanicked,
    #[error("artifact store error: {0}")]
    Artifact(#[from] clipmill_artifacts::ArtifactError),
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("invalid path: {0}")]
    InvalidPath(&'static str),
    #[error("invalid artifact GC duration: {0}")]
    InvalidDuration(String),
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("IPC error: {0}")]
    Ipc(String),
    #[error("SQLite integrity check failed: {result}")]
    IntegrityCheckFailed { result: String },
    #[error("this platform does not expose an application data directory")]
    PlatformDataDirectory,
    #[error("socket path is occupied by a non-socket file: {0}")]
    SocketPathOccupied(PathBuf),
    #[error("SQLite {found} is below the required 3.51.3")]
    SqliteTooOld { found: String },
    #[error("SQLite refused WAL journal mode and returned {found}")]
    UnexpectedJournalMode { found: String },
    #[error("database schema version {found} is newer than supported version {supported}")]
    UnsupportedSchema { found: i64, supported: i64 },
    #[error("database application id {found} does not belong to ClipMill")]
    UnexpectedApplicationId { found: i64 },
}

impl DaemonError {
    pub(crate) fn io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
