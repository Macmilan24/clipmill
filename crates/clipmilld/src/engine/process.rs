//! Keeping each installed part's worker running.
//!
//! One task per part holds at most one process. It starts the worker from the
//! part's current install, and when the process stops on its own it waits,
//! longer each time it stopped again soon, and starts it once more. A new
//! install replaces the running process straight away; stopping the daemon
//! asks every worker to finish (SIGTERM, which each worker handles by
//! releasing its lease), and only kills one that has not gone within a grace
//! period. The daemon's lease recovery covers whatever a killed worker held.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Stdio,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use tokio::{
    process::{Child, Command},
    sync::watch,
};

use super::install::{affects_python, command_path};

/// A process that ran this long before stopping is started again at once.
const STEADY: Duration = Duration::from_mins(5);
const FIRST_RETRY: Duration = Duration::from_secs(1);
const LONGEST_RETRY: Duration = Duration::from_mins(1);
/// How long a worker has to leave after being asked.
const GRACE: Duration = Duration::from_secs(10);
/// A worker's log is started afresh once it passes this size.
const MAX_LOG_BYTES: u64 = 8 * 1024 * 1024;

/// What a part's process is doing, for the engine listing.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) enum ProcessState {
    #[default]
    Stopped,
    Starting,
    Running,
    /// It stopped on its own and will be started again.
    Waiting {
        reason: String,
    },
}

impl ProcessState {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Waiting { .. } => "waiting",
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Watch {
    pub state: ProcessState,
    pub restarts: u32,
}

/// Everything a worker process is started with.
#[derive(Clone, Debug)]
pub(crate) struct Launch {
    pub part: String,
    pub command: String,
    pub identity: PathBuf,
    pub worker_socket: PathBuf,
    pub shm_socket: PathBuf,
    pub log: PathBuf,
}

/// Keep `launch` running from whichever install `installs` names, until
/// `stop` turns true.
pub(crate) async fn keep_running(
    launch: Launch,
    mut installs: watch::Receiver<Option<PathBuf>>,
    mut stop: watch::Receiver<bool>,
    watch: Arc<Mutex<Watch>>,
) {
    let mut quick_stops: u32 = 0;
    loop {
        if *stop.borrow() {
            break;
        }
        let install = installs.borrow_and_update().clone();
        let Some(install) = install else {
            set(&watch, ProcessState::Stopped, false);
            tokio::select! {
                changed = installs.changed() => if changed.is_err() { break },
                _ = stop.changed() => break,
            }
            continue;
        };

        set(&watch, ProcessState::Starting, false);
        let started = Instant::now();
        let mut child = match spawn(&launch, &install) {
            Ok(child) => child,
            Err(error) => {
                tracing::warn!(part = launch.part, %error, "an engine worker could not be started");
                quick_stops = quick_stops.saturating_add(1);
                let reason = format!("it could not be started: {error}");
                if !wait(
                    &watch,
                    reason,
                    retry_after(quick_stops),
                    &mut installs,
                    &mut stop,
                )
                .await
                {
                    break;
                }
                continue;
            }
        };
        set(&watch, ProcessState::Running, false);
        tracing::info!(part = launch.part, install = %install.display(), "engine worker started");

        let exited = tokio::select! {
            status = child.wait() => Some(status),
            changed = installs.changed() => {
                finish(&launch.part, &mut child).await;
                if changed.is_err() { break }
                quick_stops = 0;
                continue;
            }
            _ = stop.changed() => {
                finish(&launch.part, &mut child).await;
                break;
            }
        };
        let reason = match exited {
            Some(Ok(status)) => format!("it stopped ({status})"),
            Some(Err(error)) => format!("it could not be watched: {error}"),
            None => "it stopped".to_owned(),
        };
        tracing::warn!(
            part = launch.part,
            reason,
            "an engine worker stopped on its own"
        );
        quick_stops = if started.elapsed() >= STEADY {
            1
        } else {
            quick_stops.saturating_add(1)
        };
        if !wait(
            &watch,
            reason,
            retry_after(quick_stops),
            &mut installs,
            &mut stop,
        )
        .await
        {
            break;
        }
    }
    set(&watch, ProcessState::Stopped, false);
}

/// The pause before starting again after `stops` quick stops in a row.
fn retry_after(stops: u32) -> Duration {
    let doubled = FIRST_RETRY.saturating_mul(2_u32.saturating_pow(stops.saturating_sub(1)));
    doubled.min(LONGEST_RETRY)
}

/// Wait out the pause, counting the restart. False when the daemon is
/// stopping.
async fn wait(
    watch: &Arc<Mutex<Watch>>,
    reason: String,
    pause: Duration,
    installs: &mut watch::Receiver<Option<PathBuf>>,
    stop: &mut watch::Receiver<bool>,
) -> bool {
    set(watch, ProcessState::Waiting { reason }, true);
    tokio::select! {
        () = tokio::time::sleep(pause) => true,
        changed = installs.changed() => changed.is_ok(),
        _ = stop.changed() => false,
    }
}

fn set(watch: &Arc<Mutex<Watch>>, state: ProcessState, restarted: bool) {
    let mut held = watch
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    held.state = state;
    if restarted {
        held.restarts = held.restarts.saturating_add(1);
    }
}

fn spawn(launch: &Launch, install: &Path) -> std::io::Result<Child> {
    let program = command_path(install, &launch.command);
    let log = open_log(&launch.log)?;
    let errors = log.try_clone()?;
    let mut command = Command::new(&program);
    crate::platform::no_console_async(&mut command);
    for (key, _) in std::env::vars_os() {
        if affects_python(&key) {
            command.env_remove(&key);
        }
    }
    command
        .arg("--identity")
        .arg(&launch.identity)
        .arg("--worker-socket")
        .arg(&launch.worker_socket)
        .arg("--shm-socket")
        .arg(&launch.shm_socket)
        .env("PYTHONUNBUFFERED", "1")
        .env("PYTHONNOUSERSITE", "1")
        .env("CLIPMILL_WORKER_LOG", "INFO")
        // Weights reach a worker as paths on its lease. A library that would
        // look a model up on the network instead is told it may not.
        .env("HF_HUB_OFFLINE", "1")
        .env("TRANSFORMERS_OFFLINE", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(errors))
        .kill_on_drop(true);
    command.spawn()
}

fn open_log(path: &Path) -> std::io::Result<fs::File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    if fs::metadata(path).is_ok_and(|metadata| metadata.len() > MAX_LOG_BYTES) {
        let _ = fs::rename(path, path.with_extension("log.1"));
    }
    fs::OpenOptions::new().create(true).append(true).open(path)
}

/// Ask the worker to leave, and make it leave if it does not.
async fn finish(part: &str, child: &mut Child) {
    let asked = ask_to_stop(child);
    if !asked || tokio::time::timeout(GRACE, child.wait()).await.is_err() {
        tracing::warn!(
            part,
            "an engine worker did not stop when asked; stopping it"
        );
        let _ = child.kill().await;
    }
}

/// Whether the worker was asked; where no request exists (Windows, which has
/// no signal for a process without a window) it is ended at once instead,
/// and lease recovery covers anything it held.
#[cfg(unix)]
fn ask_to_stop(child: &Child) -> bool {
    use nix::{
        sys::signal::{Signal, kill},
        unistd::Pid,
    };
    child
        .id()
        .and_then(|id| i32::try_from(id).ok())
        .is_some_and(|pid| kill(Pid::from_raw(pid), Signal::SIGTERM).is_ok())
}

#[cfg(not(unix))]
fn ask_to_stop(_child: &Child) -> bool {
    false
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    #[test]
    fn restarts_wait_longer_each_time_up_to_a_minute() {
        assert_eq!(retry_after(1), Duration::from_secs(1));
        assert_eq!(retry_after(2), Duration::from_secs(2));
        assert_eq!(retry_after(4), Duration::from_secs(8));
        assert_eq!(retry_after(7), Duration::from_mins(1));
        assert_eq!(retry_after(40), Duration::from_mins(1));
    }

    /// A stand-in worker: a shell script at the install's command path.
    #[cfg(unix)]
    fn install_with(root: &Path, name: &str, script: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt as _;
        let install = root.join(name);
        let program = command_path(&install, "clipmill-worker-test");
        fs::create_dir_all(program.parent().unwrap()).unwrap();
        fs::write(&program, format!("#!/bin/sh\n{script}\n")).unwrap();
        fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();
        install
    }

    #[cfg(unix)]
    fn launch(root: &Path) -> Launch {
        Launch {
            part: "test".to_owned(),
            command: "clipmill-worker-test".to_owned(),
            identity: root.join("identity.json"),
            worker_socket: root.join("workers.sock"),
            shm_socket: root.join("shm.sock"),
            log: root.join("logs").join("test.log"),
        }
    }

    #[cfg(unix)]
    async fn until(watch: &Arc<Mutex<Watch>>, wanted: impl Fn(&Watch) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            if wanted(&watch.lock().unwrap()) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        panic!("the process never reached the state asked for");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn a_worker_is_started_with_its_identity_and_stopped_when_asked() {
        let root = tempfile::tempdir().unwrap();
        let install = install_with(
            root.path(),
            "one",
            "trap 'echo asked; exit 0' TERM; echo \"$@\"; while :; do sleep 0.05; done",
        );
        let (installs_tx, installs) = watch::channel(Some(install));
        let (stop_tx, stop) = watch::channel(false);
        let state = Arc::new(Mutex::new(Watch::default()));
        let task = tokio::spawn(keep_running(
            launch(root.path()),
            installs,
            stop,
            Arc::clone(&state),
        ));
        until(&state, |held| held.state == ProcessState::Running).await;
        // Stop it only once the stand-in has printed its arguments, which it
        // does after setting its trap: a signal that arrives earlier ends a
        // shell that never got to say anything.
        let log_path = root.path().join("logs/test.log");
        let deadline = Instant::now() + Duration::from_secs(10);
        while !fs::read_to_string(&log_path)
            .unwrap_or_default()
            .contains("--identity")
        {
            assert!(
                Instant::now() < deadline,
                "the stand-in worker never started"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
        stop_tx.send(true).unwrap();
        task.await.unwrap();
        drop(installs_tx);
        let log = fs::read_to_string(&log_path).unwrap();
        assert!(log.contains("--identity"), "{log}");
        assert!(log.contains("--worker-socket"), "{log}");
        assert!(log.contains("asked"), "SIGTERM reached the worker: {log}");
        assert_eq!(state.lock().unwrap().state, ProcessState::Stopped);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn a_worker_that_stops_on_its_own_is_started_again() {
        let root = tempfile::tempdir().unwrap();
        let install = install_with(root.path(), "one", "echo ran; exit 3");
        let (_installs_tx, installs) = watch::channel(Some(install));
        let (stop_tx, stop) = watch::channel(false);
        let state = Arc::new(Mutex::new(Watch::default()));
        let task = tokio::spawn(keep_running(
            launch(root.path()),
            installs,
            stop,
            Arc::clone(&state),
        ));
        until(&state, |held| held.restarts >= 2).await;
        stop_tx.send(true).unwrap();
        task.await.unwrap();
        let runs = fs::read_to_string(root.path().join("logs/test.log"))
            .unwrap()
            .matches("ran")
            .count();
        assert!(runs >= 2, "started {runs} times");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn a_new_install_replaces_the_running_worker() {
        let root = tempfile::tempdir().unwrap();
        let first = install_with(
            root.path(),
            "first",
            "echo first; trap 'exit 0' TERM; while :; do sleep 0.05; done",
        );
        let second = install_with(
            root.path(),
            "second",
            "echo second; trap 'exit 0' TERM; while :; do sleep 0.05; done",
        );
        let (installs_tx, installs) = watch::channel(Some(first));
        let (stop_tx, stop) = watch::channel(false);
        let state = Arc::new(Mutex::new(Watch::default()));
        let task = tokio::spawn(keep_running(
            launch(root.path()),
            installs,
            stop,
            Arc::clone(&state),
        ));
        until(&state, |held| held.state == ProcessState::Running).await;
        installs_tx.send(Some(second)).unwrap();
        let log_path = root.path().join("logs/test.log");
        let deadline = Instant::now() + Duration::from_secs(10);
        while !fs::read_to_string(&log_path)
            .unwrap_or_default()
            .contains("second")
        {
            assert!(Instant::now() < deadline, "the new install never started");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        stop_tx.send(true).unwrap();
        task.await.unwrap();
        assert_eq!(
            state.lock().unwrap().restarts,
            0,
            "a replacement is not a crash"
        );
    }
}
