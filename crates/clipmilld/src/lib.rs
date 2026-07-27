//! Durable local daemon for ClipMill.

mod artifacts;
mod config;
#[cfg(unix)]
mod daemon;
mod db;
mod device;
mod discovery;
mod error;
mod evidence;
mod implementations;
mod inputs;
#[cfg(unix)]
mod ipc;
mod jobs;
mod lock;
mod media;
mod models;
mod ranking;
mod recipes;
mod render;
mod selection;
mod service;
#[cfg(unix)]
mod shm;
mod sources;
mod speech;
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
