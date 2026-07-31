//! End-to-end proof of the shell's half of the milestone demo.
//!
//! Starts a real `clipmilld`, reads real measured hardware over the real
//! socket, then SIGKILLs the daemon and checks the shell reports the loss
//! instead of serving a stale answer. The renderer is not involved: this is
//! the data path the Models screen sits on.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command},
    time::{Duration, Instant},
};

use clipmill_shell::DaemonClient;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repository root")
}

/// `target/debug/clipmilld`, resolved from this test binary's own location.
fn daemon_binary() -> PathBuf {
    let mut path = std::env::current_exe().expect("test executable path");
    path.pop(); // deps/
    path.pop(); // debug/
    path.join("clipmilld")
}

struct DaemonUnderTest {
    child: Child,
    directory: PathBuf,
}

impl DaemonUnderTest {
    fn start(socket: &Path, directory: &Path) -> Self {
        let binary = daemon_binary();
        assert!(
            binary.is_file(),
            "clipmilld is not built at {}; run `cargo build --workspace` first",
            binary.display()
        );
        let ffprobe = repo_root().join(".cache/bin/ffprobe");
        assert!(
            ffprobe.is_file(),
            "pinned ffprobe missing at {}; run `just setup`",
            ffprobe.display()
        );

        let child = Command::new(binary)
            .arg("--data-dir")
            .arg(directory)
            .arg("--socket")
            .arg(socket)
            .arg("--ffprobe")
            .arg(ffprobe)
            .spawn()
            .expect("spawn clipmilld");

        Self {
            child,
            directory: directory.to_path_buf(),
        }
    }
}

impl Drop for DaemonUnderTest {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_dir_all(&self.directory);
    }
}

async fn wait_for_health(client: &DaemonClient, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if client.health().await.is_ok() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    false
}

#[tokio::test]
// Needs a built clipmilld and the pinned ffprobe, which the plain `rust` job
// does not fetch. The `shell-link` CI job provides both and runs it with
// `--ignored`; keeping it out of `cargo test --workspace` avoids a failure
// that would only mean "this machine has no sidecars".
#[ignore = "requires `cargo build --workspace` and `./tools/fetch-ffmpeg.sh`"]
async fn shell_reads_measured_hardware_and_notices_a_killed_daemon() {
    // Unix socket paths are length-limited, so stay out of deep temp trees.
    let directory = PathBuf::from(format!("/tmp/cm-shell-{}", std::process::id()));
    fs::create_dir_all(&directory).expect("test directory");
    let socket = directory.join("d.sock");

    let mut daemon = DaemonUnderTest::start(&socket, &directory);
    let client = DaemonClient::new(socket.clone());

    assert!(
        wait_for_health(&client, Duration::from_secs(30)).await,
        "daemon never opened its socket"
    );

    // ---- what the sidebar badge and top bar bind to ----
    let health = client.health().await.expect("health");
    assert!(
        !health.daemon_version.is_empty(),
        "daemon reported no version"
    );
    assert!(health.local_lock, "Local Lock should be on in Phase 0");

    // ---- what the Models screen renders ----
    let profile = client
        .device_profile(false)
        .await
        .expect("device profile should be measurable on this machine");
    assert!(
        profile.artifact_id.starts_with("sha256:"),
        "profile must be content-addressed, got {}",
        profile.artifact_id
    );

    let document: serde_json::Value =
        serde_json::from_str(&profile.profile_json).expect("profile is valid JSON");
    assert_eq!(
        document["schema_version"], "clipmill.device_profile.v1",
        "profile must carry its schema version"
    );
    assert!(
        document["cpu"]["model"]
            .as_str()
            .is_some_and(|m| !m.is_empty()),
        "profile must name a real CPU"
    );
    assert!(
        document["memory"]["total_bytes"].as_u64().unwrap_or(0) > 0,
        "profile must report memory"
    );

    // The cache must return the identical artifact rather than remeasuring.
    let again = client.device_profile(false).await.expect("cached profile");
    assert_eq!(
        again.artifact_id, profile.artifact_id,
        "an unchanged machine must reuse its profile artifact"
    );

    // ---- kill -9 while the app is "open" ----
    daemon.child.kill().expect("kill daemon");
    daemon.child.wait().expect("reap daemon");

    let deadline = Instant::now() + Duration::from_secs(10);
    let mut observed_loss = false;
    while Instant::now() < deadline {
        if client.health().await.is_err() {
            observed_loss = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(
        observed_loss,
        "the shell kept reporting a healthy daemon after it was killed"
    );
}

/// The stream the Analysis Progress screen binds to.
///
/// Three things it has to do, and only a live daemon can show any of them: push
/// events as tasks move, hand out a cursor that means something, and replay from
/// that cursor so a shell that was away does not show a finished stage as still
/// running. The demo DAG stands in for the analyze DAG here — it is four tasks
/// with real transitions, and it needs no media or models.
#[tokio::test]
#[ignore = "requires `cargo build --workspace` and `./tools/fetch-ffmpeg.sh`"]
async fn task_events_stream_live_and_replay_from_a_cursor() {
    use std::sync::{Arc, Mutex};

    let directory = PathBuf::from(format!("/tmp/cm-events-{}", std::process::id()));
    fs::create_dir_all(&directory).expect("test directory");
    let socket = directory.join("d.sock");

    let daemon = DaemonUnderTest::start(&socket, &directory);
    let client = DaemonClient::new(socket.clone());
    assert!(
        wait_for_health(&client, Duration::from_secs(30)).await,
        "daemon never opened its socket"
    );

    // Subscribe before submitting, which is the order the shell uses: a screen
    // that subscribed after starting a job would miss its first transitions.
    let live = Arc::new(Mutex::new(Vec::<(u64, String)>::new()));
    let collected = Arc::clone(&live);
    let streaming = DaemonClient::new(socket.clone());
    let follower = tokio::spawn(async move {
        let _result = streaming
            .stream_task_events(0, move |event| {
                collected
                    .lock()
                    .expect("event list")
                    .push((event.event_id, event.task_id));
            })
            .await;
    });
    tokio::time::sleep(Duration::from_millis(300)).await;

    let project = client
        .create_project("events")
        .await
        .expect("create a project");
    client
        .submit_demo(&project, b"task-event-stream")
        .await
        .expect("submit the demo DAG");

    // Four tasks, each passing through several states, so waiting for a handful
    // of events is waiting for the pipeline to actually move.
    let deadline = Instant::now() + Duration::from_secs(30);
    while live.lock().expect("event list").len() < 4 && Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let seen = live.lock().expect("event list").clone();
    assert!(
        seen.len() >= 4,
        "expected the demo DAG's transitions, saw {}",
        seen.len()
    );
    // The cursor is global and monotonic: a shell that keeps the highest one it
    // saw can ask for everything after it and get exactly that.
    let mut ordered = seen.iter().map(|(id, _)| *id).collect::<Vec<_>>();
    let sorted = {
        let mut copy = ordered.clone();
        copy.sort_unstable();
        copy
    };
    assert_eq!(ordered, sorted, "event ids arrived out of order");
    ordered.dedup();
    assert_eq!(ordered.len(), seen.len(), "an event id repeated");

    // Replay: a second subscription from a mid-stream cursor sees the rest and
    // nothing before it. This is what a reconnecting shell does.
    let midpoint = seen[seen.len() / 2].0;
    let replayed = Arc::new(Mutex::new(Vec::<u64>::new()));
    let collected = Arc::clone(&replayed);
    let replaying = DaemonClient::new(socket.clone());
    let replay = tokio::spawn(async move {
        let _result = replaying
            .stream_task_events(midpoint, move |event| {
                collected.lock().expect("replay list").push(event.event_id);
            })
            .await;
    });
    let deadline = Instant::now() + Duration::from_secs(15);
    while replayed.lock().expect("replay list").is_empty() && Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let after = replayed.lock().expect("replay list").clone();
    assert!(!after.is_empty(), "replay from a cursor returned nothing");
    assert!(
        after.iter().all(|id| *id > midpoint),
        "replay returned events at or before the cursor: {after:?}"
    );

    follower.abort();
    replay.abort();
    drop(daemon);
}
