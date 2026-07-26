//! Durable local daemon for ClipMill.

mod artifacts;
mod config;
#[cfg(unix)]
mod daemon;
mod db;
mod device;
mod error;
#[cfg(unix)]
mod ipc;
mod jobs;
mod lock;
mod media;
mod models;
mod recipes;
mod render;
mod service;
#[cfg(unix)]
mod shm;
mod sources;
#[cfg(unix)]
mod worker;

pub use artifacts::{ArtifactCoordinator, ArtifactServiceError};
pub use config::{Config, Paths};
#[cfg(unix)]
pub use daemon::{Daemon, EditLog};
pub use device::{
    DeviceProfileError, VerifiedDeviceProfile, verify_profile as verify_device_profile,
};
pub use error::DaemonError;
