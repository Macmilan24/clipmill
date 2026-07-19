//! Durable local daemon for ClipMill.

mod artifacts;
mod config;
#[cfg(unix)]
mod daemon;
mod db;
mod error;
#[cfg(unix)]
mod ipc;
mod jobs;
mod lock;
mod service;

pub use artifacts::{ArtifactCoordinator, ArtifactServiceError};
pub use config::{Config, Paths};
#[cfg(unix)]
pub use daemon::Daemon;
pub use error::DaemonError;
