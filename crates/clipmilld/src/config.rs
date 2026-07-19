use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    time::Duration,
};

use directories::ProjectDirs;

use crate::DaemonError;

const DATA_DIR_ENV: &str = "CLIPMILL_DATA_DIR";
const SOCKET_ENV: &str = "CLIPMILL_SOCKET";
const ARTIFACT_GC_GRACE_ENV: &str = "CLIPMILL_ARTIFACT_GC_GRACE";
const FFPROBE_ENV: &str = "CLIPMILL_FFPROBE";
const DEFAULT_ARTIFACT_GC_GRACE: Duration = Duration::from_hours(168);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub paths: Paths,
    pub artifact_gc_grace: Duration,
    pub ffprobe: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Paths {
    pub data_dir: PathBuf,
    pub state_dir: PathBuf,
    pub backups_dir: PathBuf,
    pub artifacts_dir: PathBuf,
    pub run_dir: PathBuf,
    pub database: PathBuf,
    pub socket: PathBuf,
    pub lock: PathBuf,
    pub probe_scratch_dir: PathBuf,
}

impl Config {
    pub fn resolve(
        cli_data_dir: Option<PathBuf>,
        cli_socket: Option<PathBuf>,
    ) -> Result<Self, DaemonError> {
        Self::resolve_with_gc(cli_data_dir, cli_socket, None)
    }

    pub fn resolve_with_gc(
        cli_data_dir: Option<PathBuf>,
        cli_socket: Option<PathBuf>,
        cli_artifact_gc_grace: Option<Duration>,
    ) -> Result<Self, DaemonError> {
        Self::resolve_full(cli_data_dir, cli_socket, cli_artifact_gc_grace, None)
    }

    pub fn resolve_full(
        cli_data_dir: Option<PathBuf>,
        cli_socket: Option<PathBuf>,
        cli_artifact_gc_grace: Option<Duration>,
        cli_ffprobe: Option<PathBuf>,
    ) -> Result<Self, DaemonError> {
        let platform_default = ProjectDirs::from("dev", "clipmill", "ClipMill")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .ok_or(DaemonError::PlatformDataDirectory)?;
        Self::from_sources_with_gc(
            cli_data_dir,
            cli_socket,
            cli_artifact_gc_grace,
            env::var_os(DATA_DIR_ENV),
            env::var_os(SOCKET_ENV),
            env::var_os(ARTIFACT_GC_GRACE_ENV),
            cli_ffprobe,
            env::var_os(FFPROBE_ENV),
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
        Self::from_sources_with_gc(
            cli_data_dir,
            cli_socket,
            None,
            env_data_dir,
            env_socket,
            None,
            None,
            None,
            platform_default,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_sources_with_gc(
        cli_data_dir: Option<PathBuf>,
        cli_socket: Option<PathBuf>,
        cli_artifact_gc_grace: Option<Duration>,
        env_data_dir: Option<OsString>,
        env_socket: Option<OsString>,
        env_artifact_gc_grace: Option<OsString>,
        cli_ffprobe: Option<PathBuf>,
        env_ffprobe: Option<OsString>,
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
        let artifact_gc_grace = match cli_artifact_gc_grace {
            Some(value) => value,
            None => env_artifact_gc_grace
                .map(|value| parse_duration(&value))
                .transpose()?
                .unwrap_or(DEFAULT_ARTIFACT_GC_GRACE),
        };
        let socket = cli_socket
            .or_else(|| env_socket.map(PathBuf::from))
            .unwrap_or_else(|| run_dir.join("clipmilld.sock"));
        validate_socket(&socket)?;
        let ffprobe = cli_ffprobe
            .or_else(|| env_ffprobe.map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("ffprobe"));
        if ffprobe.as_os_str().is_empty() {
            return Err(DaemonError::InvalidPath("FFprobe path is empty"));
        }

        Ok(Self {
            paths: Paths {
                database: state_dir.join("clipmill.db"),
                backups_dir: state_dir.join("backups"),
                artifacts_dir: data_dir.join("artifacts"),
                lock: run_dir.join("daemon.lock"),
                probe_scratch_dir: state_dir.join("probe-scratch"),
                data_dir,
                state_dir,
                run_dir,
                socket,
            },
            artifact_gc_grace,
            ffprobe,
        })
    }
}

fn parse_duration(value: &OsString) -> Result<Duration, DaemonError> {
    let text = value
        .to_str()
        .ok_or_else(|| DaemonError::InvalidDuration("duration is not valid UTF-8".to_owned()))?;
    humantime::parse_duration(text).map_err(|error| DaemonError::InvalidDuration(error.to_string()))
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

    use std::{ffi::OsString, path::PathBuf, time::Duration};

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
        assert_eq!(
            config.paths.backups_dir,
            PathBuf::from("/default/data/state/backups")
        );
        assert_eq!(
            config.paths.artifacts_dir,
            PathBuf::from("/default/data/artifacts")
        );
        assert_eq!(
            config.paths.probe_scratch_dir,
            PathBuf::from("/default/data/state/probe-scratch")
        );
        assert_eq!(config.artifact_gc_grace, Duration::from_hours(168));
        assert_eq!(config.ffprobe, PathBuf::from("ffprobe"));
    }

    #[test]
    fn artifact_gc_grace_uses_cli_environment_default_precedence() {
        let cli = Config::from_sources_with_gc(
            None,
            None,
            Some(Duration::from_hours(2)),
            None,
            None,
            Some(OsString::from("1h")),
            None,
            None,
            PathBuf::from("/default/data"),
        )
        .expect("CLI grace");
        assert_eq!(cli.artifact_gc_grace, Duration::from_hours(2));

        let environment = Config::from_sources_with_gc(
            None,
            None,
            None,
            None,
            None,
            Some(OsString::from("3d")),
            None,
            None,
            PathBuf::from("/default/data"),
        )
        .expect("environment grace");
        assert_eq!(environment.artifact_gc_grace, Duration::from_hours(72));

        assert!(
            Config::from_sources_with_gc(
                None,
                None,
                None,
                None,
                None,
                Some(OsString::from("not-a-duration")),
                None,
                None,
                PathBuf::from("/default/data"),
            )
            .is_err()
        );
    }

    #[test]
    fn ffprobe_uses_cli_environment_default_precedence() {
        let cli = Config::from_sources_with_gc(
            None,
            None,
            None,
            None,
            None,
            None,
            Some(PathBuf::from("/cli/ffprobe")),
            Some(OsString::from("/env/ffprobe")),
            PathBuf::from("/default/data"),
        )
        .expect("CLI FFprobe");
        assert_eq!(cli.ffprobe, PathBuf::from("/cli/ffprobe"));

        let environment = Config::from_sources_with_gc(
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(OsString::from("/env/ffprobe")),
            PathBuf::from("/default/data"),
        )
        .expect("environment FFprobe");
        assert_eq!(environment.ffprobe, PathBuf::from("/env/ffprobe"));
    }
}
