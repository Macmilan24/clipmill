//! The processing engine of a packaged ClipMill: its model workers, installed
//! on this computer when the person asks and kept running by the daemon.
//!
//! A development checkout builds its workers with `just setup` and starts them
//! with `just workers`; nothing here runs there. A packaged app instead ships
//! the engine's packages and a pinned uv, and this module does what those two
//! commands did: it installs each part into the data directory (downloading
//! the pinned Python and the third-party wheels, which is a network operation
//! the person starts and the Local Lock counts), and it starts, watches and
//! stops one worker process per installed part.
//!
//! Keys come first. The worker service reads the keys it trusts once, when
//! the daemon starts, so [`Engine::prepare`] enrols every part this computer
//! runs before that read, whether or not the part is installed yet.

mod identity;
mod install;
mod manifest;
mod process;

use std::{
    collections::{BTreeMap, VecDeque},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use clipmill_contracts::proto::ipc::v1::{EnginePartV1, EngineResponse};
use tokio::sync::{Notify, watch};

use self::{
    install::{CANCELLED, Installer},
    manifest::{Manifest, Part},
    process::{Launch, Watch},
};
use crate::policy::LocalLockPolicy;

/// Where the engine lives and what its workers connect to.
#[derive(Clone, Debug)]
pub(crate) struct EnginePaths {
    /// The app's pinned uv.
    pub uv: PathBuf,
    /// `<resources>/engine`: the manifest and the packages the app carries.
    pub shipped: PathBuf,
    /// `<data>/engine`: Python, installs and uv's cache.
    pub engine_dir: PathBuf,
    /// `<data>/logs`.
    pub logs_dir: PathBuf,
    pub identity_dir: PathBuf,
    pub trust_dir: PathBuf,
    pub worker_socket: PathBuf,
    pub shm_socket: PathBuf,
}

/// Why a request to the engine was refused. One sentence a screen can show.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Refusal {
    NotFound(String),
    Unavailable(String),
}

impl Refusal {
    pub(crate) fn message(&self) -> &str {
        match self {
            Self::NotFound(message) | Self::Unavailable(message) => message,
        }
    }
}

/// The latest attempt at installing one part.
#[derive(Clone, Debug)]
enum Attempt {
    Queued,
    Installing { step: String },
    Failed { message: String },
}

#[derive(Debug, Default)]
struct State {
    attempts: BTreeMap<String, Attempt>,
    queue: VecDeque<String>,
    active: Option<watch::Sender<bool>>,
    /// The editorial check: what it is doing or why it failed, and the
    /// machine, model and install it was tried for, so it is tried once.
    proof: Option<String>,
    proof_tried: Option<String>,
}

/// One part this computer runs, with what keeps its worker going.
#[derive(Debug)]
struct Running {
    part: Part,
    /// The install its worker runs from; replaced after a new install.
    install: watch::Sender<Option<PathBuf>>,
    watch: Arc<Mutex<Watch>>,
    log: PathBuf,
}

#[derive(Debug)]
pub(crate) struct Engine {
    manifest: Result<Manifest, String>,
    installer: Installer,
    paths: EnginePaths,
    policy: Arc<LocalLockPolicy>,
    parts: Vec<Running>,
    state: Mutex<State>,
    wake: Notify,
    stop: watch::Sender<bool>,
    stopping: AtomicBool,
    tasks: Mutex<Vec<tokio::task::JoinHandle<()>>>,
}

impl Engine {
    /// Read the app's engine and enrol a key for every part this computer
    /// runs. Nothing starts yet: see [`Engine::start`].
    pub(crate) fn prepare(paths: EnginePaths, policy: Arc<LocalLockPolicy>) -> Arc<Self> {
        let manifest = Manifest::load(&paths.shipped);
        if let Err(problem) = &manifest {
            tracing::warn!(problem, "the app's processing engine cannot be installed");
        }
        let installer = Installer {
            uv: paths.uv.clone(),
            shipped: paths.shipped.clone(),
            engine_dir: paths.engine_dir.clone(),
            log: paths.logs_dir.join("engine-install.log"),
        };
        let mut parts = Vec::new();
        if let Ok(manifest) = &manifest {
            for part in manifest.parts_here() {
                if part.is_worker()
                    && let Err(problem) =
                        identity::enrol(&paths.identity_dir, &paths.trust_dir, &part.name)
                {
                    tracing::warn!(part = part.name, problem, "an engine worker has no key");
                    continue;
                }
                let current = installer.current(&part.name).map(|(_, dir)| dir);
                parts.push(Running {
                    part: part.clone(),
                    install: watch::channel(current).0,
                    watch: Arc::new(Mutex::new(Watch::default())),
                    log: paths
                        .logs_dir
                        .join("workers")
                        .join(format!("{}.log", part.name)),
                });
            }
        }
        let (stop, _) = watch::channel(false);
        Arc::new(Self {
            manifest,
            installer,
            paths,
            policy,
            parts,
            state: Mutex::new(State::default()),
            wake: Notify::new(),
            stop,
            stopping: AtomicBool::new(false),
            tasks: Mutex::new(Vec::new()),
        })
    }

    /// Start every installed part's worker and the install runner. Call once
    /// the worker socket is listening. Must be called inside a Tokio runtime.
    pub(crate) fn start(self: &Arc<Self>) {
        let names: Vec<&str> = self
            .parts
            .iter()
            .map(|running| running.part.name.as_str())
            .collect();
        self.installer.remove_unreferenced(&names);
        let mut tasks = self
            .tasks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for running in &self.parts {
            if !running.part.is_worker() {
                self.write_launcher(running);
                continue;
            }
            let launch = Launch {
                part: running.part.name.clone(),
                command: running.part.command.clone(),
                identity: identity::identity_path(&self.paths.identity_dir, &running.part.name),
                worker_socket: self.paths.worker_socket.clone(),
                shm_socket: self.paths.shm_socket.clone(),
                log: running.log.clone(),
            };
            tasks.push(tokio::spawn(process::keep_running(
                launch,
                running.install.subscribe(),
                self.stop.subscribe(),
                Arc::clone(&running.watch),
            )));
        }
        tasks.push(tokio::spawn(Arc::clone(self).run_installs()));
    }

    /// Stop installing and stop every worker, waiting for them to leave.
    pub(crate) async fn stop(&self) {
        self.stopping.store(true, Ordering::SeqCst);
        {
            let mut state = self.lock();
            state.queue.clear();
            if let Some(cancel) = &state.active {
                let _ = cancel.send(true);
            }
        }
        let _ = self.stop.send(true);
        self.wake.notify_one();
        let tasks: Vec<_> = self
            .tasks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .drain(..)
            .collect();
        for task in tasks {
            if tokio::time::timeout(Duration::from_secs(15), task)
                .await
                .is_err()
            {
                tracing::warn!("an engine task did not stop in time");
            }
        }
    }

    /// Install `names`, or every part here that is not installed at its
    /// current version when `names` is empty. The installs run one after
    /// another in the background; the listing says how far they got.
    pub(crate) fn install(&self, names: &[String]) -> Result<(), Refusal> {
        if let Err(problem) = &self.manifest {
            return Err(Refusal::Unavailable(problem.clone()));
        }
        let python = self.python();
        let wanted: Vec<&Running> = if names.is_empty() {
            self.parts
                .iter()
                .filter(|running| self.install_state(running, &python) != "installed")
                .collect()
        } else {
            names
                .iter()
                .map(|name| {
                    self.parts
                        .iter()
                        .find(|running| &running.part.name == name)
                        .ok_or_else(|| {
                            Refusal::NotFound(format!("this computer runs no engine part {name:?}"))
                        })
                })
                .collect::<Result<_, _>>()?
        };
        let mut state = self.lock();
        for running in wanted {
            let name = &running.part.name;
            let busy = matches!(
                state.attempts.get(name),
                Some(Attempt::Queued | Attempt::Installing { .. })
            );
            if !busy {
                state.attempts.insert(name.clone(), Attempt::Queued);
                state.queue.push_back(name.clone());
            }
        }
        drop(state);
        self.wake.notify_one();
        Ok(())
    }

    /// Stop the install in progress and forget the queue. Parts already
    /// installed stay installed.
    pub(crate) fn cancel(&self) {
        let mut state = self.lock();
        for name in state.queue.drain(..).collect::<Vec<_>>() {
            state.attempts.remove(&name);
        }
        if let Some(cancel) = &state.active {
            let _ = cancel.send(true);
        }
    }

    /// The engine as it stands now.
    pub(crate) fn list(&self) -> EngineResponse {
        let attempts = self.lock().attempts.clone();
        EngineResponse {
            managed: true,
            parts: self
                .parts
                .iter()
                .map(|running| self.describe(running, &attempts, true))
                .collect(),
            python_version: self.python(),
            unavailable: self.manifest.as_ref().err().cloned().unwrap_or_default(),
        }
    }

    /// The part whose worker family is `family`, when the engine runs it.
    /// Its size is not measured: readiness asks often, and never shows it.
    pub(crate) fn part_for_family(&self, family: &str) -> Option<EnginePartV1> {
        let attempts = self.lock().attempts.clone();
        self.parts
            .iter()
            .find(|running| running.part.family == family)
            .map(|running| self.describe(running, &attempts, false))
    }

    fn describe(
        &self,
        running: &Running,
        attempts: &BTreeMap<String, Attempt>,
        measure: bool,
    ) -> EnginePartV1 {
        let python = self.python();
        let (state, detail) = match attempts.get(&running.part.name) {
            Some(Attempt::Queued) => ("queued".to_owned(), String::new()),
            Some(Attempt::Installing { step }) => ("installing".to_owned(), step.clone()),
            Some(Attempt::Failed { message }) => ("failed".to_owned(), message.clone()),
            None => (
                self.install_state(running, &python).to_owned(),
                String::new(),
            ),
        };
        let watch = running
            .watch
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let detail = match (&watch.state, detail.is_empty()) {
            (process::ProcessState::Waiting { reason }, true) => reason.clone(),
            _ if detail.is_empty() && running.part.family == EDITORIAL_FAMILY => {
                self.lock().proof.clone().unwrap_or_default()
            }
            _ => detail,
        };
        let installed_bytes = if measure {
            self.installer
                .current(&running.part.name)
                .map_or(0, |(_, dir)| crate::library::tree_bytes(&dir))
        } else {
            0
        };
        EnginePartV1 {
            name: running.part.name.clone(),
            title: running.part.title.clone(),
            family: running.part.family.clone(),
            state,
            detail,
            installed_bytes,
            download_bytes: running.part.download_bytes_here(),
            process: watch.state.name().to_owned(),
            restarts: watch.restarts,
            log_path: running.log.display().to_string(),
            tool: !running.part.is_worker(),
        }
    }

    /// What the daemon runs for the tool part `name`: a launcher that starts
    /// its current install, or says it is not installed yet. A shell script
    /// on Unix, a batch file on Windows.
    pub(crate) fn launcher(&self, name: &str) -> PathBuf {
        let launchers = self.paths.engine_dir.join("launchers");
        if cfg!(windows) {
            launchers.join(format!("{name}.cmd"))
        } else {
            launchers.join(name)
        }
    }

    /// Write the launcher for a tool part, pointing at its current install.
    /// It is a fixed script that quotes the one path the engine chose;
    /// without an install it answers with the importer's own setup error, so
    /// the screen says what to do.
    fn write_launcher(&self, running: &Running) {
        let path = self.launcher(&running.part.name);
        let python = self
            .installer
            .current(&running.part.name)
            .map(|(_, dir)| install::command_path(&dir, "python"));
        let script = launcher_script(&running.part.title, &running.part.module, python.as_deref());
        let written = path
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|()| crate::library::write_private(&path, script.as_bytes()))
            .and_then(|()| make_executable(&path));
        if let Err(error) = written {
            tracing::warn!(part = running.part.name, %error, "a tool's launcher could not be written");
        }
    }

    /// Turns true when the daemon stops, for loops that serve the engine.
    pub(crate) fn stopping(&self) -> watch::Receiver<bool> {
        self.stop.subscribe()
    }

    /// The editorial component's install, when it is installed at the
    /// version this app ships: one from an earlier app may lack the check.
    pub(crate) fn editorial_install(&self) -> Option<(String, PathBuf)> {
        let running = self
            .parts
            .iter()
            .find(|running| running.part.family == EDITORIAL_FAMILY)?;
        let (pointer, dir) = self.installer.current(&running.part.name)?;
        (pointer.digest == running.part.digest(&self.python())).then_some((pointer.digest, dir))
    }

    /// Whether the editorial check was already tried for `key` (machine,
    /// model and install): it runs once per combination, and a failure stays
    /// on screen until one of them changes.
    pub(crate) fn proof_tried(&self, key: &str) -> bool {
        self.lock().proof_tried.as_deref() == Some(key)
    }

    /// Record that `key` was tried without running the check: a receipt that
    /// already matched was applied.
    pub(crate) fn mark_proof_tried(&self, key: &str) {
        self.lock().proof_tried = Some(key.to_owned());
    }

    /// Generate once with the editorial model `binding` names, in the
    /// editorial component's own environment, and let it write `receipt`.
    /// Its output goes to the logs; the part shows what it is doing.
    pub(crate) async fn prove_editorial(
        &self,
        key: &str,
        binding: &[u8],
        receipt: &std::path::Path,
        fingerprint: &str,
    ) -> Result<(), String> {
        {
            let mut state = self.lock();
            state.proof_tried = Some(key.to_owned());
            state.proof = Some("Checking that the editorial model runs on this Mac".to_owned());
        }
        let outcome = self
            .run_editorial_check(binding, receipt, fingerprint)
            .await;
        self.lock().proof = match &outcome {
            Ok(()) => None,
            Err(message) => Some(format!("The editorial model could not run here: {message}")),
        };
        outcome
    }

    async fn run_editorial_check(
        &self,
        binding: &[u8],
        receipt: &std::path::Path,
        fingerprint: &str,
    ) -> Result<(), String> {
        let (_, install) = self
            .editorial_install()
            .ok_or("the editorial component is not installed")?;
        let binding_path = self.paths.engine_dir.join("editorial-binding.pb");
        crate::library::write_private(&binding_path, binding)
            .map_err(|error| format!("{}: {error}", binding_path.display()))?;
        let log_path = self.paths.logs_dir.join("editorial-check.log");
        if let Some(parent) = log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|error| format!("{}: {error}", log_path.display()))?;
        let errors = log
            .try_clone()
            .map_err(|error| format!("{}: {error}", log_path.display()))?;
        let mut command = tokio::process::Command::new(install::command_path(&install, "python"));
        crate::platform::no_console_async(&mut command);
        for (key, _) in std::env::vars_os() {
            if install::affects_python(&key) {
                command.env_remove(&key);
            }
        }
        command
            .args([
                "-I",
                "-m",
                "clipmill_worker_editorial.runtime_check",
                "--binding",
            ])
            .arg(&binding_path)
            .arg("--receipt")
            .arg(receipt)
            .arg("--fingerprint")
            .arg(fingerprint)
            .env("PYTHONUNBUFFERED", "1")
            .env("HF_HUB_OFFLINE", "1")
            .env("TRANSFORMERS_OFFLINE", "1")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::from(log))
            .stderr(std::process::Stdio::from(errors))
            .kill_on_drop(true);
        let mut child = command
            .spawn()
            .map_err(|error| format!("the check could not start: {error}"))?;
        let mut stop = self.stop.subscribe();
        let status = tokio::select! {
            waited = tokio::time::timeout(EDITORIAL_CHECK_DEADLINE, child.wait()) => {
                let Ok(status) = waited else {
                    let _ = child.kill().await;
                    return Err("it did not answer within 20 minutes".to_owned());
                };
                status.map_err(|error| error.to_string())?
            }
            () = until_true(&mut stop) => {
                let _ = child.kill().await;
                return Err(install::CANCELLED.to_owned());
            }
        };
        let _ = std::fs::remove_file(&binding_path);
        if status.success() {
            Ok(())
        } else {
            Err(format!("see {} ({status})", log_path.display()))
        }
    }

    fn python(&self) -> String {
        self.manifest
            .as_ref()
            .map(|manifest| manifest.python.clone())
            .unwrap_or_default()
    }

    /// missing, installed or outdated, from what is on disk.
    fn install_state(&self, running: &Running, python: &str) -> &'static str {
        match self.installer.current(&running.part.name) {
            None => "missing",
            Some((pointer, _)) if pointer.digest == running.part.digest(python) => "installed",
            Some(_) => "outdated",
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    async fn run_installs(self: Arc<Self>) {
        loop {
            if self.stopping.load(Ordering::SeqCst) {
                return;
            }
            let next = {
                let mut state = self.lock();
                state.queue.pop_front().map(|name| {
                    let (cancel, cancelled) = watch::channel(false);
                    state.active = Some(cancel);
                    state.attempts.insert(
                        name.clone(),
                        Attempt::Installing {
                            step: "Starting".to_owned(),
                        },
                    );
                    (name, cancelled)
                })
            };
            let Some((name, mut cancelled)) = next else {
                let mut stop = self.stop.subscribe();
                tokio::select! {
                    () = self.wake.notified() => {}
                    () = until_true(&mut stop) => return,
                }
                continue;
            };
            let outcome = self.install_one(&name, &mut cancelled).await;
            let mut state = self.lock();
            state.active = None;
            match outcome {
                Ok(()) => {
                    state.attempts.remove(&name);
                }
                Err(message) if message == CANCELLED => {
                    state.attempts.remove(&name);
                }
                Err(message) => {
                    tracing::warn!(part = name, message, "an engine part failed to install");
                    state.attempts.insert(name, Attempt::Failed { message });
                }
            }
        }
    }

    async fn install_one(
        &self,
        name: &str,
        cancelled: &mut watch::Receiver<bool>,
    ) -> Result<(), String> {
        let Some(running) = self.parts.iter().find(|running| running.part.name == name) else {
            return Err(format!("this computer runs no engine part {name:?}"));
        };
        // Python and the third-party wheels are downloaded: counted by the
        // Local Lock like every network operation the person starts.
        self.policy.note_task_start("engine-install");
        let python = self.python();
        let step = |step: &str| {
            let mut state = self.lock();
            if let Some(Attempt::Installing { step: current }) = state.attempts.get_mut(name) {
                step.clone_into(current);
            }
        };
        let dir = self
            .installer
            .install(&running.part, &python, &step, cancelled)
            .await?;
        tracing::info!(part = name, install = %dir.display(), "engine part installed");
        // The worker moves to the new install; its old one is removed the
        // next time the daemon starts, when nothing runs from it.
        running.install.send_replace(Some(dir));
        if !running.part.is_worker() {
            self.write_launcher(running);
        }
        Ok(())
    }
}

/// The launcher for a tool whose module is `module`, run by `python` when it
/// is installed. Without an install it answers with the importer's own setup
/// error, so the screen says what to do.
#[cfg(unix)]
fn launcher_script(title: &str, module: &str, python: Option<&std::path::Path>) -> String {
    match python {
        Some(python) => format!(
            "#!/bin/sh\n# Written by the ClipMill daemon: runs {title}.\nexec {} -I -m {module} \"$@\"\n",
            shell_quote(&python.display().to_string()),
        ),
        None => format!(
            "#!/bin/sh\n# Written by the ClipMill daemon: {title} is not installed.\nprintf '%s\\n' {}\nexit 1\n",
            shell_quote(&setup_required(title)),
        ),
    }
}

/// The batch-file launcher. Inside double quotes a batch file still expands
/// `%`, so it is doubled; a Windows path cannot hold a double quote. The
/// message leaves out every character the batch parser treats specially.
#[cfg(windows)]
fn launcher_script(title: &str, module: &str, python: Option<&std::path::Path>) -> String {
    if let Some(python) = python {
        format!(
            "@echo off\r\n\"{}\" -I -m {module} %*\r\n",
            python.display().to_string().replace('%', "%%"),
        )
    } else {
        let title: String = title
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == ' ' || *c == '-')
            .collect();
        format!(
            "@echo off\r\necho {}\r\nexit /b 1\r\n",
            setup_required(&title)
        )
    }
}

/// The importer's setup error, as the line a tool prints.
fn setup_required(title: &str) -> String {
    format!(
        "{{\"event\":\"error\",\"code\":\"setup_required\",\"message\":\"{title} is not installed yet. Install it under Components in Models.\"}}"
    )
}

/// `value` as one single-quoted shell word.
#[cfg(unix)]
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(unix)]
fn make_executable(path: &std::path::Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
#[allow(clippy::unnecessary_wraps)]
fn make_executable(_path: &std::path::Path) -> std::io::Result<()> {
    Ok(())
}

/// The worker family of the editorial component, whose model must prove it
/// runs on a Mac before the daemon admits its Metal worker.
pub(crate) const EDITORIAL_FAMILY: &str = "editorial";
/// Loading a 6 GB model and answering once; generous for a slow disk.
const EDITORIAL_CHECK_DEADLINE: Duration = Duration::from_mins(20);

/// Resolve once `flag` is true, or once nothing can set it any more. The
/// borrow `wait_for` returns is dropped inside, so the caller's future stays
/// `Send`.
pub(crate) async fn until_true(flag: &mut watch::Receiver<bool>) {
    let _ = flag.wait_for(|value| *value).await;
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::fs;

    use super::*;

    fn paths(root: &std::path::Path) -> EnginePaths {
        EnginePaths {
            uv: root.join("uv"),
            shipped: root.join("resources").join("engine"),
            engine_dir: root.join("data").join("engine"),
            logs_dir: root.join("data").join("logs"),
            identity_dir: root.join("data").join("state").join("worker-identity"),
            trust_dir: root.join("data").join("state").join("worker-trust"),
            worker_socket: root.join("w.sock"),
            shm_socket: root.join("s.sock"),
        }
    }

    fn ship(root: &std::path::Path, parts: &[(&str, &str)]) {
        let shipped = root.join("resources").join("engine");
        fs::create_dir_all(&shipped).unwrap();
        let digest = "a".repeat(64);
        let parts: Vec<String> = parts
            .iter()
            .map(|(name, platform)| {
                format!(
                    r#"{{"name":"{name}","title":"Part {name}","family":"fam-{name}","command":"clipmill-worker-{name}","module":"clipmill_worker_{name}","platforms":["{platform}"],"requirements":"requirements/{name}.txt","requirements_sha256":"{digest}","wheels":[{{"file":"wheels/{name}.whl","sha256":"{digest}"}}],"download_bytes":{{"{platform}":1234}}}}"#
                )
            })
            .collect();
        fs::write(
            shipped.join("engine.json"),
            format!(
                r#"{{"schema_version":"clipmill.engine.v1","python":"3.12.13","parts":[{}]}}"#,
                parts.join(",")
            ),
        )
        .unwrap();
    }

    #[test]
    fn only_the_parts_this_computer_runs_are_listed_and_enrolled() {
        let root = tempfile::tempdir().unwrap();
        ship(
            root.path(),
            &[("one", manifest::platform()), ("two", "elsewhere")],
        );
        let engine = Engine::prepare(paths(root.path()), Arc::new(LocalLockPolicy::new()));
        let listing = engine.list();
        assert!(listing.managed);
        assert_eq!(listing.python_version, "3.12.13");
        assert_eq!(listing.parts.len(), 1);
        let part = &listing.parts[0];
        assert_eq!(
            (part.name.as_str(), part.state.as_str()),
            ("one", "missing")
        );
        assert_eq!(part.download_bytes, 1234);
        assert_eq!(part.process, "stopped");
        let trusted = fs::read_dir(root.path().join("data/state/worker-trust"))
            .unwrap()
            .count();
        assert_eq!(trusted, 1, "a key for the one part here");
    }

    #[tokio::test]
    async fn a_tool_gets_no_key_and_a_launcher_instead_of_a_process() {
        let root = tempfile::tempdir().unwrap();
        ship(root.path(), &[("one", manifest::platform())]);
        // Make "one" a tool.
        let path = root.path().join("resources/engine/engine.json");
        let text = fs::read_to_string(&path)
            .unwrap()
            .replace(r#""name":"one","#, r#""name":"one","kind":"tool","#);
        fs::write(&path, text).unwrap();
        let engine = Engine::prepare(paths(root.path()), Arc::new(LocalLockPolicy::new()));
        let trusted =
            fs::read_dir(root.path().join("data/state/worker-trust")).map_or(0, Iterator::count);
        assert_eq!(trusted, 0, "a tool never connects as a worker");
        engine.start();
        let launcher = fs::read_to_string(engine.launcher("one")).unwrap();
        assert!(launcher.contains("setup_required"), "{launcher}");
        assert!(
            launcher.contains("Part one is not installed yet"),
            "{launcher}"
        );
        let part = engine.list().parts.remove(0);
        assert!(part.tool);
        assert_eq!(part.process, "stopped");
        engine.stop().await;
    }

    #[test]
    fn a_launcher_quotes_the_install_it_runs() {
        assert_eq!(shell_quote("/a b/c"), "'/a b/c'");
        assert_eq!(shell_quote("it's"), r"'it'\''s'");
    }

    #[test]
    fn an_app_without_an_engine_says_why_and_refuses_to_install() {
        let root = tempfile::tempdir().unwrap();
        let engine = Engine::prepare(paths(root.path()), Arc::new(LocalLockPolicy::new()));
        let listing = engine.list();
        assert!(listing.parts.is_empty());
        assert!(!listing.unavailable.is_empty());
        assert!(matches!(engine.install(&[]), Err(Refusal::Unavailable(_))));
    }

    #[test]
    fn asking_for_a_part_this_computer_does_not_run_is_refused_by_name() {
        let root = tempfile::tempdir().unwrap();
        ship(root.path(), &[("one", manifest::platform())]);
        let engine = Engine::prepare(paths(root.path()), Arc::new(LocalLockPolicy::new()));
        let refused = engine.install(&["two".to_owned()]).unwrap_err();
        assert!(refused.message().contains("two"), "{refused:?}");
    }

    #[tokio::test]
    async fn an_install_that_cannot_start_fails_with_a_reason_and_counts_as_network_use() {
        let root = tempfile::tempdir().unwrap();
        ship(root.path(), &[("one", manifest::platform())]);
        let policy = Arc::new(LocalLockPolicy::new());
        let engine = Engine::prepare(paths(root.path()), Arc::clone(&policy));
        engine.start();
        engine.install(&[]).unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        loop {
            let part = engine.list().parts.remove(0);
            if part.state == "failed" {
                assert!(part.detail.contains("could not start"), "{}", part.detail);
                break;
            }
            assert!(std::time::Instant::now() < deadline, "still {}", part.state);
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert_eq!(policy.status().egress_attempts, 1);
        engine.stop().await;
    }
}
