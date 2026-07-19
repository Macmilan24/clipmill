//! Durable local daemon for ClipMill.

mod config;
#[cfg(unix)]
mod daemon;
mod db;
mod error;
#[cfg(unix)]
mod ipc;
mod lock;
mod service;

pub use config::{Config, Paths};
#[cfg(unix)]
pub use daemon::Daemon;
pub use error::DaemonError;
