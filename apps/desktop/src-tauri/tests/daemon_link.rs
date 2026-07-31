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
    sync::Arc,
    time::{Duration, Instant},
};

use clipmill_shell::{DaemonClient, DaemonSupervisor, MEDIA_SCHEME as SCHEME, MediaProtocol};

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

/// Generate a short A/V file with the pinned encoder.
///
/// Real media rather than a fixture on disk: the point of this drill is that the
/// daemon probed something, derived from it, and served the result — and a
/// checked-in file would only prove the last of those.
fn generate_media(path: &Path, seconds: u32) {
    let ffmpeg = repo_root().join(".cache/bin/ffmpeg");
    assert!(
        ffmpeg.is_file(),
        "pinned ffmpeg missing at {}; run ./tools/fetch-ffmpeg.sh",
        ffmpeg.display()
    );
    let status = Command::new(ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
        ])
        .arg(format!("testsrc2=size=320x180:rate=24:duration={seconds}"))
        .args(["-f", "lavfi", "-i"])
        .arg(format!(
            "sine=frequency=440:sample_rate=48000:duration={seconds}"
        ))
        .args([
            "-map",
            "0:v:0",
            "-map",
            "1:a:0",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-c:a",
            "aac",
            "-shortest",
        ])
        .arg(path)
        .status()
        .expect("run the pinned encoder");
    assert!(status.success(), "fixture generation failed");
}

/// Wait until a task publishing this kind has an address, and return it.
async fn published(
    client: &DaemonClient,
    job_id: &str,
    output_kind: &str,
    timeout: Duration,
) -> Option<String> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let job = client.get_job(job_id).await.expect("read the job");
        if let Some(task) = job
            .tasks
            .iter()
            .find(|task| task.output_kind == output_kind && !task.output_artifact_id.is_empty())
        {
            return Some(task.output_artifact_id.clone());
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    None
}

fn media_request(path: &str, range: Option<&str>) -> tauri::http::Request<Vec<u8>> {
    let mut builder = tauri::http::Request::builder().uri(format!("{SCHEME}://localhost{path}"));
    if let Some(value) = range {
        builder = builder.header(tauri::http::header::RANGE, value);
    }
    builder.body(Vec::new()).expect("a media request")
}

/// Serve one filmstrip tile whole, then a byte range out of it.
///
/// The range is the half that matters: a player seeking sends one, and a handler
/// that quietly answered with the beginning would play the wrong thing.
async fn stream_a_tile(
    client: &DaemonClient,
    protocol: &MediaProtocol,
    project: &str,
    filmstrip: &str,
) -> String {
    let inventory = client
        .resolve_media(project, filmstrip)
        .await
        .expect("resolve the filmstrip");
    let tile = inventory
        .files
        .first()
        .expect("the filmstrip named no tiles")
        .clone();
    assert_eq!(tile.media_type, "image/jpeg");

    let whole = protocol
        .serve(media_request(
            &format!("/{project}/{filmstrip}/{}", tile.path),
            None,
        ))
        .await;
    assert_eq!(whole.status(), 200);
    assert_eq!(whole.body().len() as u64, tile.bytes);

    let ranged = protocol
        .serve(media_request(
            &format!("/{project}/{filmstrip}/{}", tile.path),
            Some("bytes=1-8"),
        ))
        .await;
    assert_eq!(ranged.status(), 206);
    assert_eq!(ranged.body().len(), 8);
    assert_eq!(&ranged.body()[..], &whole.body()[1..9]);

    tile.path
}

/// The three refusals, against artifacts that genuinely exist.
///
/// Refusing something absent proves nothing; each of these is a real artifact
/// this daemon really published, denied for a policy reason rather than for want
/// of an object.
async fn assert_refusals(
    client: &DaemonClient,
    protocol: &MediaProtocol,
    project: &str,
    job_id: &str,
    filmstrip: &str,
    tile: &str,
) {
    // A kind nobody put on the media list. The 16 kHz audio exists — speech
    // reads it — and a renderer still may not have it by either door.
    let audio = published(client, job_id, "media.audio_16k.v1", Duration::from_mins(1))
        .await
        .expect("the run never published 16 kHz audio");
    let denied = protocol
        .serve(media_request(
            &format!("/{project}/{audio}/audio.wav"),
            None,
        ))
        .await;
    assert_eq!(denied.status(), 403, "an unlisted kind was streamed");
    assert!(
        client.read_document(project, &audio).await.is_err(),
        "an unlisted kind was served as a document"
    );

    // Another project's id, for an artifact that genuinely exists. Not found
    // rather than denied: a project learns nothing about another's artifacts.
    let other = client.create_project("bystander").await.expect("a project");
    assert!(
        client.read_document(&other, filmstrip).await.is_err(),
        "one project read another's artifact"
    );
    let cross = protocol
        .serve(media_request(&format!("/{other}/{filmstrip}/{tile}"), None))
        .await;
    assert_eq!(
        cross.status(),
        403,
        "one project streamed another's artifact"
    );

    // A file the artifact's own descriptor never named.
    let invented = protocol
        .serve(media_request(
            &format!("/{project}/{filmstrip}/invented.jpg"),
            None,
        ))
        .await;
    assert_eq!(invented.status(), 404, "a file nobody named was served");
}

/// The whole shell data plane, against a daemon that really ran the work.
///
/// This is the drill the screens sit on: import a file, watch the run move, read
/// a document it published, and stream a frame it derived. Nothing is stubbed —
/// the probe is FFprobe, the filmstrip is FFmpeg's, and the tile arrives through
/// the same protocol handler the WebView addresses.
///
/// It does not wait for the analysis to finish. The stages after ingest need
/// worker processes this job does not start, so what is asserted is everything
/// the shell needs before them, which is also everything this workstream built.
#[tokio::test]
#[ignore = "requires `cargo build --workspace` and `./tools/fetch-ffmpeg.sh`"]
async fn a_run_is_started_watched_read_and_streamed() {
    let directory = PathBuf::from(format!("/tmp/cm-pipeline-{}", std::process::id()));
    fs::create_dir_all(&directory).expect("test directory");
    let socket = directory.join("d.sock");
    let media_path = directory.join("smoke.mp4");
    generate_media(&media_path, 2);

    let daemon = DaemonUnderTest::start(&socket, &directory);
    let client = DaemonClient::new(socket.clone());
    assert!(
        wait_for_health(&client, Duration::from_secs(30)).await,
        "daemon never opened its socket"
    );

    // ---- Import, exactly as the New Project screen performs it. ----
    let project = client.create_project("pipeline").await.expect("a project");
    let registered = client
        .register_source(&project, media_path.to_str().expect("a UTF-8 path"))
        .await
        .expect("register the source");
    let source = registered.source.expect("the registered source");

    // The probe arrives with the registration, because the artifact carrying it
    // is not published until the run's first task — and the screen has to show a
    // duration before anyone commits to a run.
    let map: serde_json::Value =
        serde_json::from_str(&registered.source_map_json).expect("the probe parses");
    assert!(
        map["container"]["duration_ticks"].as_u64().unwrap_or(0) > 0,
        "the probe reported no duration"
    );

    let job = client
        .submit_analyze(
            &project,
            clipmill_contracts::proto::ipc::v1::AnalyzeSourcePayloadV1 {
                key_version: "clipmill.analyze-source.v1".to_owned(),
                source_id: source.source_id.clone(),
                language: String::new(),
                duration: Some(clipmill_contracts::proto::ipc::v1::ClipDurationV1 {
                    min_ticks: 15 * 90_000,
                    max_ticks: 60 * 90_000,
                }),
                count: 3,
                diversity_milli: 0,
            },
        )
        .await
        .expect("submit the analysis");

    // ---- Watch it move, and read what it derived. ----

    // The same probe again, this time as a published artifact read through the
    // document door the screens use.
    let source_map = published(
        &client,
        &job.job_id,
        "evidence.source_map.v1",
        Duration::from_mins(1),
    )
    .await
    .expect("the run never published a source map");
    let (kind, json) = client
        .read_document(&project, &source_map)
        .await
        .expect("read the source map");
    assert_eq!(kind, "evidence.source_map.v1");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&json).expect("the document parses")["container"]
            ["duration_ticks"],
        map["container"]["duration_ticks"],
        "the published probe disagrees with the one handed back at registration"
    );

    let filmstrip = published(
        &client,
        &job.job_id,
        "media.filmstrip.v1",
        Duration::from_mins(3),
    )
    .await
    .expect("the run never published a filmstrip");
    let ingest = published(
        &client,
        &job.job_id,
        "media.ingest_manifest.v1",
        Duration::from_mins(1),
    )
    .await
    .expect("the run never published an ingest manifest");
    let (kind, _json) = client
        .read_document(&project, &ingest)
        .await
        .expect("read the ingest manifest");
    assert_eq!(kind, "media.ingest_manifest.v1");

    // ---- Stream a tile, and confirm what the doors will not open. ----
    let supervisor = Arc::new(DaemonSupervisor::new(DaemonClient::new(socket.clone())));
    let protocol = MediaProtocol::new(Arc::clone(&supervisor), directory.join("artifacts"));
    let tile = stream_a_tile(&client, &protocol, &project, &filmstrip).await;
    assert_refusals(&client, &protocol, &project, &job.job_id, &filmstrip, &tile).await;

    drop(daemon);
}
