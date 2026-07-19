use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;

use crate::DaemonError;

const DATA_DIR_ENV: &str = "CLIPMILL_DATA_DIR";
const SOCKET_ENV: &str = "CLIPMILL_SOCKET";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub paths: Paths,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Paths {
    pub data_dir: PathBuf,
    pub state_dir: PathBuf,
    pub run_dir: PathBuf,
    pub database: PathBuf,
    pub socket: PathBuf,
    pub lock: PathBuf,
}

impl Config {
    pub fn resolve(
        cli_data_dir: Option<PathBuf>,
        cli_socket: Option<PathBuf>,
    ) -> Result<Self, DaemonError> {
        let platform_default = ProjectDirs::from("dev", "clipmill", "ClipMill")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .ok_or(DaemonError::PlatformDataDirectory)?;
        Self::from_sources(
            cli_data_dir,
            cli_socket,
            env::var_os(DATA_DIR_ENV),
            env::var_os(SOCKET_ENV),
            platform_default,
        )
    }

    pub fn from_sources(
        cli_data_dir: Option<PathBuf>,
        cli_socket: Option<PathBuf>,
        env_data_dir: Option<OsString>,
        env_socket: Option<OsString>,
        platform_default: PathBuf,
    ) -> Result<Self, DaemonError> {
        let data_dir = cli_data_dir
            .or_else(|| env_data_dir.map(PathBuf::from))
            .unwrap_or(platform_default);
        if data_dir.as_os_str().is_empty() {
            return Err(DaemonError::InvalidPath("data directory is empty"));
        }

        let state_dir = data_dir.join("state");
        let run_dir = data_dir.join("run");
        let socket = cli_socket
            .or_else(|| env_socket.map(PathBuf::from))
            .unwrap_or_else(|| run_dir.join("clipmilld.sock"));
        validate_socket(&socket)?;

        Ok(Self {
            paths: Paths {
                database: state_dir.join("clipmill.db"),
                lock: run_dir.join("daemon.lock"),
                data_dir,
                state_dir,
                run_dir,
                socket,
            },
        })
    }
}

fn validate_socket(path: &Path) -> Result<(), DaemonError> {
    if path.as_os_str().is_empty() {
        return Err(DaemonError::InvalidPath("socket path is empty"));
    }
    if path.file_name().is_none() {
        return Err(DaemonError::InvalidPath(
            "socket path must include a file name",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use std::{ffi::OsString, path::PathBuf};

    use super::Config;

    #[test]
    fn command_line_precedes_environment_and_defaults() {
        let config = Config::from_sources(
            Some(PathBuf::from("/cli/data")),
            Some(PathBuf::from("/cli/socket")),
            Some(OsString::from("/env/data")),
            Some(OsString::from("/env/socket")),
            PathBuf::from("/default/data"),
        )
        .expect("valid config");

        assert_eq!(config.paths.data_dir, PathBuf::from("/cli/data"));
        assert_eq!(config.paths.socket, PathBuf::from("/cli/socket"));
    }

    #[test]
    fn environment_precedes_platform_default() {
        let config = Config::from_sources(
            None,
            None,
            Some(OsString::from("/env/data")),
            Some(OsString::from("/env/socket")),
            PathBuf::from("/default/data"),
        )
        .expect("valid config");

        assert_eq!(config.paths.data_dir, PathBuf::from("/env/data"));
        assert_eq!(config.paths.socket, PathBuf::from("/env/socket"));
    }

    #[test]
    fn default_socket_follows_the_selected_environment_data_directory() {
        let config = Config::from_sources(
            None,
            None,
            Some(OsString::from("/env/data")),
            None,
            PathBuf::from("/default/data"),
        )
        .expect("valid config");

        assert_eq!(config.paths.data_dir, PathBuf::from("/env/data"));
        assert_eq!(
            config.paths.socket,
            PathBuf::from("/env/data/run/clipmilld.sock")
        );
    }

    #[test]
    fn default_socket_lives_under_run_directory() {
        let config = Config::from_sources(None, None, None, None, PathBuf::from("/default/data"))
            .expect("valid config");

        assert_eq!(
            config.paths.socket,
            PathBuf::from("/default/data/run/clipmilld.sock")
        );
    }
}
