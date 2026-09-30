//! Durable local daemon for ClipMill.

mod analysis;
mod artifacts;
mod assets;
mod captions;
mod collector;
mod config;
mod daemon;
mod db;
mod device;
mod discovery;
mod editorial;
pub mod endpoint;
mod engine;
mod error;
mod evidence;
mod export;
mod implementations;
mod inputs;
mod inspector;
mod ipc;
mod jobs;
mod library;
mod lock;
mod media;
mod models;
mod platform;
mod policy;
mod ranking;
mod recipes;
mod render;
mod selection;
mod service;
mod shell;
mod shm;
mod sources;
mod speech;
mod storage;
mod updates;
mod worker;
mod youtube_transport;

pub use artifacts::{ArtifactCoordinator, ArtifactServiceError};
pub use config::{Bundle, Config, Paths};
pub use daemon::{Daemon, EditLog};
pub use device::{
    DeviceProfileError, VerifiedDeviceProfile, verify_profile as verify_device_profile,
};
pub use error::DaemonError;
