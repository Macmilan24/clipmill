//! W11 ingest fan-out gate: one submit derives every media derivative, all
//! of them verify against their descriptors, a warm re-submit is a pure
//! cache identity, mutated sources fail deterministically, and a killed
//! daemon finishes the fan-out within the recovery SLO.
//!
//! Requires the pinned FFmpeg/FFprobe sidecars (`./tools/fetch-ffmpeg.sh`),
//! so every test is `#[ignore]` and driven by `just gate-ingest`.
#![cfg(unix)]
#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod support;

use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

use clipmill_artifacts::{ArtifactLease, ArtifactPath};
use clipmill_contracts::proto::{ipc::v1::JobState, worker::v1::FailureClass};
use clipmill_core::ArtifactId;
use clipmilld::{ArtifactCoordinator, Config, Daemon, DaemonError};
use serde_json::Value;
use tempfile::TempDir;
use tokio::{sync::oneshot, task::JoinHandle, time::sleep};

use support::{
    create, get_job, register_source, submit_ingest, wait_until_ready, workspace_tempdir,
};

fn workspace_tool(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(".cache/bin")
        .join(name)
        .canonicalize()
        .unwrap_or_else(|_| panic!("{name} is missing; run ./tools/fetch-ffmpeg.sh"))
}

fn config(temp: &TempDir) -> Config {
    Config::from_sources_with_gc(
        Some(temp.path().to_path_buf()),
        None,
        None,
        Some(OsString::from("/ignored/env")),
        None,
        None,
        Some(workspace_tool("ffprobe")),
        None,
        PathBuf::from("/ignored/default"),
    )
    .expect("ingest test config")
}

async fn running(
    config: Config,
) -> (
    PathBuf,
    ArtifactCoordinator,
    oneshot::Sender<()>,
    JoinHandle<Result<(), DaemonError>>,
) {
    let daemon = Daemon::start(config).await.expect("daemon starts");
    let socket = daemon.socket_path().to_path_buf();
    let artifacts = daemon.artifact_coordinator();
    let (shutdown, stopped) = oneshot::channel();
    let task = tokio::spawn(daemon.serve_until(async {
        let _result = stopped.await;
    }));
    (socket, artifacts, shutdown, task)
}

async fn stop(shutdown: oneshot::Sender<()>, task: JoinHandle<Result<(), DaemonError>>) {
    let _sent = shutdown.send(());
    task.await
        .expect("daemon task joins")
        .expect("daemon shuts down");
}

fn generate_av_media(ffmpeg: &Path, path: &Path, seconds: u32) {
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
        .expect("run pinned FFmpeg");
    assert!(status.success(), "A/V fixture generation failed");
}

fn generate_audio_media(ffmpeg: &Path, path: &Path) {
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
        .arg("sine=frequency=330:sample_rate=44100:duration=2")
        .args(["-c:a", "aac"])
        .arg(path)
        .status()
        .expect("run pinned FFmpeg");
    assert!(status.success(), "audio fixture generation failed");
}

async fn wait_for_job(
    socket: &Path,
    request_prefix: &str,
    job_id: &str,
    wanted: JobState,
    timeout: Duration,
) -> clipmill_contracts::proto::ipc::v1::Job {
    let deadline = Instant::now() + timeout;
    let mut attempt = 0_u64;
    loop {
        attempt += 1;
        let job = get_job(socket, &format!("{request_prefix}-{attempt}"), job_id)
            .await
            .expect("get job");
        if job.state == wanted as i32 {
            return job;
        }
        assert!(
            job.state != JobState::Failed as i32 || wanted == JobState::Failed,
            "job failed instead: {}",
            job.failure_detail
        );
        assert!(
            Instant::now() < deadline,
            "job did not reach {wanted:?} in time (last state {})",
            job.state
        );
        sleep(Duration::from_millis(100)).await;
    }
}

fn read_json(lease: &ArtifactLease, name: &str) -> Value {
    let path = name.parse::<ArtifactPath>().expect("artifact path");
    let mut file = lease.open_verified(&path).expect("open verified payload");
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).expect("read payload");
    serde_json::from_slice(&bytes).expect("payload JSON")
}

/// Open the fan-in manifest and return every child as kind → artifact id.
async fn manifest_children(
    artifacts: &ArtifactCoordinator,
    job: &clipmill_contracts::proto::ipc::v1::Job,
) -> BTreeMap<String, ArtifactId> {
    assert_eq!(
        job.output_artifact_ids.len(),
        1,
        "one rooted final artifact"
    );
    let manifest_id = job.output_artifact_ids[0]
        .parse::<ArtifactId>()
        .expect("manifest artifact id");
    let lease = artifacts.open(manifest_id).await.expect("open manifest");
    assert_eq!(lease.kind(), "media.ingest_manifest.v1");
    let manifest = read_json(&lease, "ingest-manifest.json");
    assert_eq!(
        manifest["schema_version"], "clipmill.media.ingest_manifest.v1",
        "manifest declares its schema"
    );
    manifest["children"]
        .as_array()
        .expect("children")
        .iter()
        .map(|child| {
            (
                child["kind"].as_str().expect("child kind").to_owned(),
                child["artifact_id"]
                    .as_str()
                    .expect("child id")
                    .parse::<ArtifactId>()
                    .expect("child artifact id"),
            )
        })
        .collect()
}

#[allow(clippy::too_many_lines)]
#[tokio::test]
#[ignore = "requires the pinned FFmpeg sidecars (./tools/fetch-ffmpeg.sh)"]
async fn full_ingest_derives_everything_verifies_and_caches() {
    let temp = workspace_tempdir();
    let media_path = temp.path().join("episode.mp4");
    generate_av_media(&workspace_tool("ffmpeg"), &media_path, 2);
    let (socket, artifacts, shutdown, task) = running(config(&temp)).await;
    wait_until_ready(&socket).await.expect("daemon ready");
    let project = create(&socket, "ingest-project", "Ingest")
        .await
        .expect("project");
    let registered = register_source(&socket, "ingest-register", &project.project_id, &media_path)
        .await
        .expect("register source");
    let source = registered.source.expect("source record");

    let submitted = submit_ingest(
        &socket,
        "ingest-submit",
        &project.project_id,
        &source.source_id,
    )
    .await
    .expect("submit ingest");
    assert_eq!(submitted.tasks.len(), 9, "full A/V fan-out plus manifest");
    let job = wait_for_job(
        &socket,
        "ingest-wait",
        &submitted.job_id,
        JobState::Succeeded,
        Duration::from_mins(2),
    )
    .await;

    let children = manifest_children(&artifacts, &job).await;
    let expected_kinds = [
        "media.proxy.v1",
        "media.audio_16k.v1",
        "media.audio_48k.v1",
        "media.loudness_envelope.v1",
        "media.reference_index.v1",
        "media.filmstrip.v1",
        "media.audio_peaks.v1",
        "media.frames.v1",
    ];
    for kind in expected_kinds {
        assert!(children.contains_key(kind), "manifest lists {kind}");
    }
    assert_eq!(children.len(), expected_kinds.len());

    // Every derivative re-verifies from its manifest and agrees on the
    // source fingerprint and a positive coverage interval.
    let proxy = artifacts
        .open(children["media.proxy.v1"])
        .await
        .expect("open proxy");
    let proxy_descriptor = read_json(&proxy, "proxy.json");
    assert_eq!(
        proxy_descriptor["source_fingerprint"],
        source.source_fingerprint
    );
    assert!(proxy_descriptor["video"]["width"].as_u64().expect("width") <= 1280);
    assert_eq!(proxy_descriptor["video"]["frame_rate"]["num"], 30_000);
    assert!(proxy_descriptor["duration_ticks"].as_i64().expect("ticks") > 0);
    let proxy_payload = "proxy.mp4".parse::<ArtifactPath>().expect("path");
    drop(
        proxy
            .open_verified(&proxy_payload)
            .expect("proxy payload verifies"),
    );

    let audio_16k = artifacts
        .open(children["media.audio_16k.v1"])
        .await
        .expect("open 16k");
    let descriptor = read_json(&audio_16k, "audio.json");
    assert_eq!(descriptor["sample_rate"], 16_000);
    assert_eq!(descriptor["channels"], 1);
    assert_eq!(descriptor["source_fingerprint"], source.source_fingerprint);

    let audio_48k = artifacts
        .open(children["media.audio_48k.v1"])
        .await
        .expect("open 48k");
    let descriptor = read_json(&audio_48k, "audio.json");
    assert_eq!(descriptor["sample_rate"], 48_000);
    assert_eq!(descriptor["channels"], 2);

    let loudness = artifacts
        .open(children["media.loudness_envelope.v1"])
        .await
        .expect("open loudness");
    let envelope = read_json(&loudness, "loudness.json");
    assert!(
        !envelope["points"].as_array().expect("points").is_empty(),
        "envelope carries sampled points"
    );
    let integrated = envelope["summary"]["integrated_lufs"]
        .as_f64()
        .expect("integrated loudness");
    assert!(integrated.is_finite() && integrated < 0.0);

    let reference = artifacts
        .open(children["media.reference_index.v1"])
        .await
        .expect("open reference index");
    let index = read_json(&reference, "reference-index.json");
    assert!(
        !index["video_keyframes"]
            .as_array()
            .expect("keyframes")
            .is_empty(),
        "reference index carries keyframes"
    );
    assert!(index["streams"].as_array().expect("streams").len() >= 2);

    let filmstrip = artifacts
        .open(children["media.filmstrip.v1"])
        .await
        .expect("open filmstrip");
    let strip = read_json(&filmstrip, "index.json");
    let tiles = strip["tiles"].as_array().expect("tiles");
    assert!(!tiles.is_empty());
    assert_eq!(tiles[0]["t_ticks"], 0);
    let first_tile = tiles[0]["file"].as_str().expect("tile file");
    drop(
        filmstrip
            .open_verified(&first_tile.parse::<ArtifactPath>().expect("tile path"))
            .expect("tile payload verifies"),
    );

    let frames = artifacts
        .open(children["media.frames.v1"])
        .await
        .expect("open frames");
    let frame_index = read_json(&frames, "index.json");
    assert!(
        frame_index["frames"].as_array().expect("frames").len() >= 6,
        "two seconds at six frames per second"
    );

    let peaks = artifacts
        .open(children["media.audio_peaks.v1"])
        .await
        .expect("open peaks");
    let peaks_document = read_json(&peaks, "peaks.json");
    let buckets = peaks_document["peaks"].as_array().expect("buckets");
    assert!(!buckets.is_empty());
    assert!(
        buckets.iter().any(|bucket| {
            bucket["max"].as_i64().unwrap_or(0) > bucket["min"].as_i64().unwrap_or(0)
        }),
        "a sine source has non-flat buckets"
    );

    // Warm re-ingest: a distinct job resolves to the exact same artifact
    // identities without re-deriving anything.
    let warm_started = Instant::now();
    let resubmitted = submit_ingest(
        &socket,
        "ingest-resubmit",
        &project.project_id,
        &source.source_id,
    )
    .await
    .expect("warm submit");
    let warm_job = wait_for_job(
        &socket,
        "ingest-warm-wait",
        &resubmitted.job_id,
        JobState::Succeeded,
        Duration::from_secs(30),
    )
    .await;
    let warm_elapsed = warm_started.elapsed();
    let warm_children = manifest_children(&artifacts, &warm_job).await;
    assert_eq!(
        warm_children, children,
        "cold and warm ingest agree on every artifact identity"
    );
    assert_eq!(warm_job.output_artifact_ids, job.output_artifact_ids);
    assert!(
        warm_elapsed < Duration::from_secs(10),
        "warm ingest was not a cache lookup ({warm_elapsed:?})"
    );

    stop(shutdown, task).await;
}

#[tokio::test]
#[ignore = "requires the pinned FFmpeg sidecars (./tools/fetch-ffmpeg.sh)"]
async fn audio_only_ingest_skips_video_derivatives() {
    let temp = workspace_tempdir();
    let media_path = temp.path().join("podcast.m4a");
    generate_audio_media(&workspace_tool("ffmpeg"), &media_path);
    let (socket, artifacts, shutdown, task) = running(config(&temp)).await;
    wait_until_ready(&socket).await.expect("daemon ready");
    let project = create(&socket, "audio-project", "Podcast")
        .await
        .expect("project");
    let registered = register_source(&socket, "audio-register", &project.project_id, &media_path)
        .await
        .expect("register source");
    let source = registered.source.expect("source record");
    let submitted = submit_ingest(
        &socket,
        "audio-submit",
        &project.project_id,
        &source.source_id,
    )
    .await
    .expect("submit ingest");
    let job = wait_for_job(
        &socket,
        "audio-wait",
        &submitted.job_id,
        JobState::Succeeded,
        Duration::from_mins(1),
    )
    .await;
    let children = manifest_children(&artifacts, &job).await;
    for kind in [
        "media.audio_16k.v1",
        "media.audio_48k.v1",
        "media.loudness_envelope.v1",
        "media.audio_peaks.v1",
        "media.reference_index.v1",
    ] {
        assert!(children.contains_key(kind), "audio ingest keeps {kind}");
    }
    for kind in ["media.proxy.v1", "media.filmstrip.v1", "media.frames.v1"] {
        assert!(
            !children.contains_key(kind),
            "audio ingest must not schedule {kind}"
        );
    }
    stop(shutdown, task).await;
}

#[tokio::test]
#[ignore = "requires the pinned FFmpeg sidecars (./tools/fetch-ffmpeg.sh)"]
async fn mutated_source_fails_deterministically() {
    let temp = workspace_tempdir();
    let media_path = temp.path().join("mutating.mp4");
    generate_av_media(&workspace_tool("ffmpeg"), &media_path, 2);
    let (socket, _artifacts, shutdown, task) = running(config(&temp)).await;
    wait_until_ready(&socket).await.expect("daemon ready");
    let project = create(&socket, "mutate-project", "Mutate")
        .await
        .expect("project");
    let registered = register_source(&socket, "mutate-register", &project.project_id, &media_path)
        .await
        .expect("register source");
    let source = registered.source.expect("source record");

    let mut corrupted = fs::read(&media_path).expect("read fixture");
    let middle = corrupted.len() / 2;
    corrupted[middle] ^= 0xff;
    fs::write(&media_path, &corrupted).expect("mutate fixture in place");

    let submitted = submit_ingest(
        &socket,
        "mutate-submit",
        &project.project_id,
        &source.source_id,
    )
    .await
    .expect("submit ingest");
    let job = wait_for_job(
        &socket,
        "mutate-wait",
        &submitted.job_id,
        JobState::Failed,
        Duration::from_mins(1),
    )
    .await;
    assert_eq!(
        job.failure_class,
        FailureClass::Deterministic as i32,
        "mutation is a deterministic refusal, not a retry loop"
    );
    assert!(
        job.failure_detail.contains("SOURCE_CHANGED"),
        "diagnostic names the mutation: {}",
        job.failure_detail
    );
    stop(shutdown, task).await;
}

/// Spawn the daemon binary with the pinned FFprobe sidecar, since the ingest
/// executors resolve FFmpeg as its sibling.
fn spawn_ingest_daemon(
    data_dir: &Path,
    socket: &Path,
    step_delay_ms: Option<u64>,
) -> std::process::Child {
    let mut command = Command::new(env!("CARGO_BIN_EXE_clipmilld"));
    command
        .arg("--data-dir")
        .arg(data_dir)
        .arg("--socket")
        .arg(socket)
        .env("RUST_LOG", "error")
        .env("CLIPMILL_FFPROBE", workspace_tool("ffprobe"))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::inherit());
    if let Some(delay) = step_delay_ms {
        command.env("CLIPMILL_W4_STEP_DELAY_MS", delay.to_string());
    }
    command.spawn().expect("spawn clipmilld")
}

#[tokio::test]
#[ignore = "requires the pinned FFmpeg sidecars (./tools/fetch-ffmpeg.sh)"]
async fn kill_mid_ingest_recovers_within_thirty_seconds() {
    use support::wait_for_exit;

    let temp = workspace_tempdir();
    let media_path = temp.path().join("long.mp4");
    generate_av_media(&workspace_tool("ffmpeg"), &media_path, 4);
    let data_dir = temp.path().join("data");
    fs::create_dir_all(&data_dir).expect("data dir");
    let socket = temp.path().join("clipmill.sock");

    let mut child = spawn_ingest_daemon(&data_dir, &socket, Some(1_500));
    wait_until_ready(&socket).await.expect("daemon ready");
    let project = create(&socket, "kill-project", "Kill")
        .await
        .expect("project");
    let registered = register_source(&socket, "kill-register", &project.project_id, &media_path)
        .await
        .expect("register source");
    let source = registered.source.expect("source record");
    let submitted = submit_ingest(
        &socket,
        "kill-submit",
        &project.project_id,
        &source.source_id,
    )
    .await
    .expect("submit ingest");

    // Let the fan-out get into flight, then kill without any grace.
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let job = get_job(&socket, "kill-peek", &submitted.job_id)
            .await
            .expect("get job");
        let running_tasks = job.tasks.iter().filter(|task| task.state == 3).count();
        if job.state == JobState::Running as i32 && running_tasks > 0 {
            break;
        }
        assert!(Instant::now() < deadline, "fan-out never started running");
        sleep(Duration::from_millis(50)).await;
    }
    child.kill().expect("SIGKILL daemon");
    let _status = wait_for_exit(&mut child).await;

    let restarted = Instant::now();
    let mut child = spawn_ingest_daemon(&data_dir, &socket, None);
    wait_until_ready(&socket).await.expect("daemon restarts");
    let job = wait_for_job(
        &socket,
        "kill-wait",
        &submitted.job_id,
        JobState::Succeeded,
        Duration::from_secs(30),
    )
    .await;
    assert!(
        restarted.elapsed() < Duration::from_secs(30),
        "recovery must finish the ingest within the SLO"
    );
    assert_eq!(job.output_artifact_ids.len(), 1);
    child.kill().expect("stop restarted daemon");
    let _status = wait_for_exit(&mut child).await;
}
