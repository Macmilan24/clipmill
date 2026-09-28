//! Installing one engine part with the app's pinned uv.
//!
//! An install is built in its own directory, named by the part's digest, and
//! becomes the part's install only when its pointer file names it: an install
//! interrupted at any step leaves a directory nothing points at, which the
//! next attempt removes. The steps:
//!
//! 1. the pinned Python, installed by uv into the engine's own directory and
//!    never taken from the machine;
//! 2. an environment on it;
//! 3. the part's third-party wheels, every one pinned by hash and none built
//!    from source;
//! 4. ClipMill's own wheels, from the app, each checked against its pin;
//! 5. its module imported once in an isolated interpreter, so a part that
//!    cannot start is found here rather than by a stalled analysis.
//!
//! Every command is a fixed argument list with no shell, its output goes to
//! the install log rather than a pipe nobody drains, and each has a deadline.

use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tokio::{process::Command, sync::watch};

use super::manifest::{self, Part};

const POINTER_SCHEMA: &str = "clipmill.engine.install.v1";
const PYTHON_DEADLINE: Duration = Duration::from_mins(15);
const PACKAGES_DEADLINE: Duration = Duration::from_mins(45);
const LOCAL_DEADLINE: Duration = Duration::from_mins(5);
const CHECK_DEADLINE: Duration = Duration::from_mins(3);
/// The log is started afresh once it passes this size.
const MAX_LOG_BYTES: u64 = 8 * 1024 * 1024;

/// Where an install puts things and what it runs.
#[derive(Clone, Debug)]
pub(crate) struct Installer {
    /// The app's pinned uv.
    pub uv: PathBuf,
    /// The engine's files inside the app: the manifest, wheels, requirements.
    pub shipped: PathBuf,
    /// The engine's writable directory in the data directory.
    pub engine_dir: PathBuf,
    /// Where install output is kept.
    pub log: PathBuf,
}

/// The file that makes an install the part's install.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Pointer {
    schema_version: String,
    pub digest: String,
    /// The install's directory under `envs/`.
    pub dir: String,
    pub installed_unix_millis: u64,
}

impl Installer {
    pub(crate) fn envs_dir(&self) -> PathBuf {
        self.engine_dir.join("envs")
    }

    fn pointer_path(&self, part: &str) -> PathBuf {
        self.envs_dir().join(format!("{part}.json"))
    }

    /// The part's current install, whatever version of the app made it.
    pub(crate) fn current(&self, part: &str) -> Option<(Pointer, PathBuf)> {
        let bytes = fs::read(self.pointer_path(part)).ok()?;
        let pointer: Pointer = serde_json::from_slice(&bytes).ok()?;
        if pointer.schema_version != POINTER_SCHEMA
            || !pointer.dir.starts_with(&format!("{part}-"))
            || pointer.dir.contains(['/', '\\'])
        {
            return None;
        }
        let dir = self.envs_dir().join(&pointer.dir);
        command_path(&dir, "python")
            .is_file()
            .then_some((pointer, dir))
    }

    /// Remove install directories no pointer names: interrupted installs, and
    /// installs a newer one replaced. Called before any worker is started, so
    /// nothing is running from them.
    pub(crate) fn remove_unreferenced(&self, parts: &[&str]) {
        let Ok(entries) = fs::read_dir(self.envs_dir()) else {
            return;
        };
        let kept: Vec<String> = parts
            .iter()
            .filter_map(|part| self.current(part).map(|(pointer, _)| pointer.dir))
            .collect();
        for entry in entries.filter_map(Result::ok) {
            let Ok(name) = entry.file_name().into_string() else {
                continue;
            };
            let is_install = entry.file_type().is_ok_and(|kind| kind.is_dir());
            if is_install && !kept.contains(&name) {
                if let Err(error) = fs::remove_dir_all(entry.path()) {
                    tracing::warn!(install = name, %error, "an old engine install could not be removed");
                } else {
                    tracing::info!(install = name, "removed an engine install nothing uses");
                }
            }
        }
    }

    /// Install `part`, reporting each step through `step`. Returns the new
    /// install's directory once its pointer names it.
    pub(crate) async fn install(
        &self,
        part: &Part,
        python: &str,
        step: &(dyn Fn(&str) + Send + Sync),
        cancel: &mut watch::Receiver<bool>,
    ) -> Result<PathBuf, String> {
        let digest = part.digest(python);
        let dir_name = format!("{}-{}", part.name, &digest[..12]);
        let dir = self.envs_dir().join(&dir_name);
        fs::create_dir_all(self.envs_dir())
            .map_err(|error| describe_io(&self.envs_dir(), &error))?;
        if dir.exists() {
            fs::remove_dir_all(&dir).map_err(|error| describe_io(&dir, &error))?;
        }
        self.start_log(&part.name);

        step(&format!("Getting Python {python}"));
        self.uv(["python", "install", python], PYTHON_DEADLINE, cancel)
            .await?;

        step("Preparing its environment");
        let mut venv: Vec<OsString> = vec!["venv".into(), dir.clone().into_os_string()];
        venv.extend(["--python", python, "--no-project"].map(OsString::from));
        self.uv(venv, LOCAL_DEADLINE, cancel).await?;
        let interpreter = command_path(&dir, "python");

        step("Downloading its packages");
        let requirements = manifest::resolve(&self.shipped, &part.requirements);
        let found = manifest::file_sha256(&requirements)
            .map_err(|error| describe_io(&requirements, &error))?;
        if found != part.requirements_sha256 {
            return Err("the app's package list for it has been changed since it was built; reinstall ClipMill".to_owned());
        }
        let mut install: Vec<OsString> = ["pip", "install", "--python"].map(OsString::from).into();
        install.push(interpreter.clone().into_os_string());
        install.extend(["--require-hashes", "--only-binary", ":all:", "-r"].map(OsString::from));
        install.push(requirements.into_os_string());
        self.uv(install, PACKAGES_DEADLINE, cancel).await?;

        step("Adding ClipMill's worker");
        let mut own: Vec<OsString> = ["pip", "install", "--python"].map(OsString::from).into();
        own.push(interpreter.clone().into_os_string());
        own.extend(["--no-deps", "--no-index", "--offline"].map(OsString::from));
        for wheel in &part.wheels {
            let path = manifest::resolve(&self.shipped, &wheel.file);
            let found = manifest::file_sha256(&path).map_err(|error| describe_io(&path, &error))?;
            if found != wheel.sha256 {
                return Err("a ClipMill package inside the app has been changed since it was built; reinstall ClipMill".to_owned());
            }
            own.push(path.into_os_string());
        }
        self.uv(own, LOCAL_DEADLINE, cancel).await?;

        step("Checking that it starts");
        let mut check = Command::new(&interpreter);
        check
            .args(["-I", "-c"])
            .arg(format!("import {}", part.module));
        self.run(check, CHECK_DEADLINE, cancel)
            .await
            .map_err(|message| format!("it was installed but does not start: {message}"))?;

        let pointer = Pointer {
            schema_version: POINTER_SCHEMA.to_owned(),
            digest,
            dir: dir_name,
            installed_unix_millis: unix_millis(),
        };
        let bytes = serde_json::to_vec_pretty(&pointer).map_err(|error| error.to_string())?;
        let path = self.pointer_path(&part.name);
        crate::library::write_private(&path, &bytes).map_err(|error| describe_io(&path, &error))?;
        Ok(dir)
    }

    /// Run the pinned uv with the engine's own directories and nothing of the
    /// machine's Python or uv configuration.
    async fn uv<I, S>(
        &self,
        arguments: I,
        deadline: Duration,
        cancel: &mut watch::Receiver<bool>,
    ) -> Result<(), String>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let mut command = Command::new(&self.uv);
        command
            .args(arguments.into_iter().map(Into::into))
            .args(["--no-progress", "--color", "never"])
            .env("UV_PYTHON_INSTALL_DIR", self.engine_dir.join("python"))
            .env("UV_CACHE_DIR", self.engine_dir.join("cache"))
            .env("UV_NO_CONFIG", "1")
            .env("UV_PYTHON_PREFERENCE", "only-managed")
            .env("UV_PYTHON_DOWNLOADS", "automatic");
        self.run(command, deadline, cancel).await
    }

    /// Run one step to completion, its output appended to the log.
    async fn run(
        &self,
        mut command: Command,
        deadline: Duration,
        cancel: &mut watch::Receiver<bool>,
    ) -> Result<(), String> {
        crate::platform::no_console_async(&mut command);
        for (key, _) in std::env::vars_os() {
            if affects_python(&key) {
                command.env_remove(&key);
            }
        }
        let log = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log)
            .map_err(|error| describe_io(&self.log, &error))?;
        let errors = log
            .try_clone()
            .map_err(|error| describe_io(&self.log, &error))?;
        command
            .env("PYTHONNOUSERSITE", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::from(log))
            .stderr(Stdio::from(errors))
            .kill_on_drop(true);
        let mut child = command
            .spawn()
            .map_err(|error| format!("the installer could not start: {error}"))?;
        let status = tokio::select! {
            waited = tokio::time::timeout(deadline, child.wait()) => {
                let Ok(status) = waited else {
                    let _ = child.kill().await;
                    return Err("it took too long; check the network connection and try again".to_owned());
                };
                status.map_err(|error| error.to_string())?
            },
            () = super::until_true(cancel) => {
                let _ = child.kill().await;
                return Err(CANCELLED.to_owned());
            }
        };
        if status.success() {
            Ok(())
        } else {
            Err(self.last_error())
        }
    }

    fn start_log(&self, part: &str) {
        if fs::metadata(&self.log).is_ok_and(|metadata| metadata.len() > MAX_LOG_BYTES) {
            let _ = fs::rename(&self.log, self.log.with_extension("log.1"));
        }
        if let Some(parent) = self.log.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log)
        {
            use std::io::Write as _;
            let _ = writeln!(file, "\n==> installing {part} at {}", unix_millis());
        }
    }

    /// The last thing the failed step said, as one line a screen can show.
    fn last_error(&self) -> String {
        let text = fs::read(&self.log)
            .map(|bytes| {
                let start = bytes.len().saturating_sub(16 * 1024);
                String::from_utf8_lossy(&bytes[start..]).into_owned()
            })
            .unwrap_or_default();
        let line = text
            .lines()
            .rev()
            .map(str::trim)
            .find(|line| {
                let lower = line.to_ascii_lowercase();
                !line.is_empty() && (lower.contains("error") || lower.contains("failed"))
            })
            .or_else(|| {
                text.lines()
                    .rev()
                    .map(str::trim)
                    .find(|line| !line.is_empty())
            })
            .unwrap_or("the installer stopped without saying why");
        let line: String = line.chars().take(240).collect();
        format!("{line} (the full output is in {})", self.log.display())
    }
}

/// Why an install stopped when the person stopped it.
pub(crate) const CANCELLED: &str = "stopped";

/// An executable in an install: `bin/<name>` on Unix, `Scripts\<name>.exe` on
/// Windows.
pub(crate) fn command_path(install: &Path, name: &str) -> PathBuf {
    if cfg!(windows) {
        install.join("Scripts").join(format!("{name}.exe"))
    } else {
        install.join("bin").join(name)
    }
}

/// Environment variables that would change which Python runs or what it
/// imports, or how uv behaves. Proxy settings are kept: they are how some
/// networks are reached at all.
pub(crate) fn affects_python(key: &std::ffi::OsStr) -> bool {
    let Some(key) = key.to_str() else {
        return false;
    };
    let upper = key.to_ascii_uppercase();
    upper.starts_with("PYTHON")
        || upper.starts_with("UV_")
        || upper.starts_with("PIP_")
        || matches!(
            upper.as_str(),
            "VIRTUAL_ENV" | "CONDA_PREFIX" | "CONDA_DEFAULT_ENV"
        )
}

fn describe_io(path: &Path, error: &std::io::Error) -> String {
    format!("{}: {error}", path.display())
}

fn unix_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn installer(root: &Path) -> Installer {
        Installer {
            uv: root.join("uv"),
            shipped: root.join("shipped"),
            engine_dir: root.join("engine"),
            log: root.join("logs").join("engine-install.log"),
        }
    }

    fn fake_install(installer: &Installer, part: &str, dir: &str) {
        let install = installer.envs_dir().join(dir);
        let python = command_path(&install, "python");
        fs::create_dir_all(python.parent().unwrap()).unwrap();
        fs::write(&python, b"").unwrap();
        let pointer = Pointer {
            schema_version: POINTER_SCHEMA.to_owned(),
            digest: "d".repeat(64),
            dir: dir.to_owned(),
            installed_unix_millis: 1,
        };
        fs::write(
            installer.envs_dir().join(format!("{part}.json")),
            serde_json::to_vec(&pointer).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn the_pointer_names_the_current_install() {
        let root = tempfile::tempdir().unwrap();
        let installer = installer(root.path());
        assert!(installer.current("vad").is_none());
        fake_install(&installer, "vad", "vad-aaaaaaaaaaaa");
        let (pointer, dir) = installer.current("vad").unwrap();
        assert_eq!(pointer.dir, "vad-aaaaaaaaaaaa");
        assert!(dir.ends_with("vad-aaaaaaaaaaaa"));
    }

    #[test]
    fn a_pointer_may_not_name_another_part_or_leave_the_engine() {
        let root = tempfile::tempdir().unwrap();
        let installer = installer(root.path());
        fake_install(&installer, "vad", "faces-aaaaaaaaaaaa");
        assert!(installer.current("vad").is_none());
        fake_install(&installer, "vad", "vad-/../../x");
        assert!(installer.current("vad").is_none());
    }

    #[test]
    fn installs_nothing_points_at_are_removed() {
        let root = tempfile::tempdir().unwrap();
        let installer = installer(root.path());
        fake_install(&installer, "vad", "vad-aaaaaaaaaaaa");
        fs::create_dir_all(installer.envs_dir().join("vad-bbbbbbbbbbbb")).unwrap();
        fs::create_dir_all(installer.envs_dir().join("faces-cccccccccccc")).unwrap();
        installer.remove_unreferenced(&["vad", "faces"]);
        let mut left: Vec<String> = fs::read_dir(installer.envs_dir())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect();
        left.sort();
        assert_eq!(left, ["vad-aaaaaaaaaaaa", "vad.json"]);
    }

    #[test]
    fn python_and_uv_settings_of_the_machine_are_not_inherited() {
        for key in [
            "PYTHONPATH",
            "PYTHONHOME",
            "UV_INDEX_URL",
            "PIP_INDEX_URL",
            "VIRTUAL_ENV",
        ] {
            assert!(affects_python(std::ffi::OsStr::new(key)), "{key}");
        }
        for key in ["HTTPS_PROXY", "HOME", "PATH", "SSL_CERT_FILE"] {
            assert!(!affects_python(std::ffi::OsStr::new(key)), "{key}");
        }
    }
}
