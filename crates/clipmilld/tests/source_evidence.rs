#![cfg(unix)]
#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod support;

use std::{
    ffi::OsString,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

use clipmill_artifacts::ArtifactPath;
use clipmill_contracts::{
    proto::ipc::v1::{JobState, RegisterSourceRequest, Request, request, response},
    schemas::source_map::SourceMap,
};
use clipmill_core::ArtifactId;
use clipmilld::{Config, Daemon, DaemonError};
use tempfile::TempDir;
use tokio::{sync::oneshot, task::JoinHandle, time::sleep};

use support::{
    create, get_job, get_source, list_sources, register_source, send,
    send_without_reading_response, submit_probe, workspace_tempdir,
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
    .expect("source test config")
}

async fn running(
    config: Config,
) -> (
    PathBuf,
    clipmilld::ArtifactCoordinator,
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

fn generate_media(ffmpeg: &Path, path: &Path) {
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
        .arg("testsrc=size=160x90:rate=30000/1001:duration=1")
        .args(["-f", "lavfi", "-i"])
        .arg("sine=frequency=440:sample_rate=48000:duration=1")
        .args([
            "-map",
            "0:v:0",
            "-map",
            "1:a:0",
            "-c:v",
            "ffv1",
            "-c:a",
            "pcm_s16le",
        ])
        .arg(path)
        .status()
        .expect("run pinned FFmpeg");
    assert!(status.success(), "fixture generation failed");
}

fn generate_vfr_media(ffmpeg: &Path, path: &Path) {
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
        .arg("testsrc=size=160x90:rate=24:duration=0.5")
        .args(["-f", "lavfi", "-i"])
        .arg("testsrc=size=160x90:rate=30:duration=0.5")
        .args([
            "-filter_complex",
            "[0:v][1:v]concat=n=2:v=1:a=0[v]",
            "-map",
            "[v]",
            "-fps_mode",
            "vfr",
            "-c:v",
            "ffv1",
        ])
        .arg(path)
        .status()
        .expect("generate VFR fixture");
    assert!(status.success(), "VFR fixture generation failed");
}

fn generate_rotation_media(ffmpeg: &Path, path: &Path) {
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
        .arg("testsrc=size=160x90:rate=30:duration=1")
        .args(["-metadata:s:v:0", "rotate=90", "-c:v", "ffv1"])
        .arg(path)
        .status()
        .expect("generate rotation fixture");
    assert!(status.success(), "rotation fixture generation failed");
}

fn generate_audio_offset_media(ffmpeg: &Path, path: &Path) {
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
        .arg("testsrc=size=160x90:rate=30:duration=1")
        .args(["-f", "lavfi", "-i"])
        .arg("sine=frequency=440:duration=0.75")
        .args(["-itsoffset", "0.25", "-f", "lavfi", "-i"])
        .arg("sine=frequency=880:duration=0.75")
        .args([
            "-map",
            "0:v",
            "-map",
            "1:a",
            "-map",
            "2:a",
            "-c:v",
            "ffv1",
            "-c:a",
            "pcm_s16le",
        ])
        .arg(path)
        .status()
        .expect("generate audio-offset fixture");
    assert!(status.success(), "audio-offset fixture generation failed");
}

async fn verified_source_map(
    artifacts: &clipmilld::ArtifactCoordinator,
    value: &str,
) -> serde_json::Value {
    let artifact_id = value.parse::<ArtifactId>().expect("artifact id");
    let lease = artifacts.open(artifact_id).await.expect("open source map");
    let artifact_path = "source-map.json"
        .parse::<ArtifactPath>()
        .expect("artifact path");
    let mut file = lease
        .open_verified(&artifact_path)
        .expect("verified source map");
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).expect("read source map");
    let typed: SourceMap = serde_json::from_slice(&bytes).expect("typed source-map contract");
    assert!(
        typed.mapping.is_some(),
        "new W5 output includes exact mapping"
    );
    serde_json::from_slice(&bytes).expect("source-map JSON")
}

async fn wait_terminal(socket: &Path, job_id: &str) -> clipmill_contracts::proto::ipc::v1::Job {
    for attempt in 0..300 {
        let job = get_job(socket, &format!("poll-{job_id}-{attempt}"), job_id)
            .await
            .expect("poll job");
        if matches!(
            job.state(),
            JobState::Succeeded | JobState::Failed | JobState::Cancelled
        ) {
            return job;
        }
        sleep(Duration::from_millis(100)).await;
    }
    panic!("probe job did not become terminal in thirty seconds")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "run through tools/drills/media-drill.sh with pinned FFmpeg"]
#[allow(clippy::too_many_lines)]
async fn registered_sources_cache_probe_publish_and_detect_mutation() {
    let temp = workspace_tempdir();
    let media = temp.path().join("source.mkv");
    let vfr_media = temp.path().join("vfr.mkv");
    let rotation_media = temp.path().join("rotation.mkv");
    let audio_offset_media = temp.path().join("audio-offset.mkv");
    let ffmpeg = workspace_tool("ffmpeg");
    generate_media(&ffmpeg, &media);
    generate_vfr_media(&ffmpeg, &vfr_media);
    generate_rotation_media(&ffmpeg, &rotation_media);
    generate_audio_offset_media(&ffmpeg, &audio_offset_media);
    let daemon_config = config(&temp);
    let (socket, artifacts, shutdown, task) = running(daemon_config.clone()).await;
    let project = create(&socket, "media-project", "Media evidence")
        .await
        .expect("create project");

    let registration_request = Request {
        request_id: "register-source-1".to_owned(),
        body: Some(request::Body::RegisterSource(RegisterSourceRequest {
            project_id: project.project_id.clone(),
            absolute_path: media.to_string_lossy().into_owned(),
        })),
    };
    send_without_reading_response(&socket, registration_request.clone())
        .await
        .expect("register source and lose response");
    let first = match send(&socket, registration_request)
        .await
        .expect("retry source registration")
        .body
    {
        Some(response::Body::RegisterSource(value)) => value,
        other => panic!("unexpected registration retry: {other:?}"),
    };
    assert!(!first.observation_cache_hit);
    let source = first.source.expect("registered source");
    assert!(source.source_id.starts_with("src_"));
    assert!(source.source_fingerprint.starts_with("sha256:"));

    let cache_started = Instant::now();
    let cached = register_source(&socket, "register-source-2", &project.project_id, &media)
        .await
        .expect("unchanged registration cache hit");
    assert!(
        cache_started.elapsed() < Duration::from_secs(1),
        "unchanged local observation should be returned in under one second"
    );
    assert!(cached.observation_cache_hit);
    assert_eq!(
        cached.source.expect("cached source").source_id,
        source.source_id
    );
    assert_eq!(
        list_sources(&socket, "list-sources", &project.project_id)
            .await
            .expect("list sources")
            .len(),
        1
    );

    let submitted = submit_probe(
        &socket,
        "submit-probe-1",
        &project.project_id,
        &source.source_id,
    )
    .await
    .expect("submit probe");
    let completed = wait_terminal(&socket, &submitted.job_id).await;
    assert_eq!(
        completed.state(),
        JobState::Succeeded,
        "probe failed: {}",
        completed.failure_detail
    );
    assert_eq!(completed.output_artifact_ids.len(), 1);
    let artifact_id = completed.output_artifact_ids[0]
        .parse::<ArtifactId>()
        .expect("artifact id");
    verified_source_map(&artifacts, &completed.output_artifact_ids[0]).await;

    let rooted = get_source(&socket, "get-rooted-source", &source.source_id)
        .await
        .expect("get rooted source");
    assert_eq!(rooted.source_map_artifact_id, artifact_id.to_string());
    let warm = submit_probe(
        &socket,
        "submit-probe-2",
        &project.project_id,
        &source.source_id,
    )
    .await
    .expect("submit warm probe");
    let warm = wait_terminal(&socket, &warm.job_id).await;
    assert_eq!(warm.state(), JobState::Succeeded);
    assert_eq!(warm.output_artifact_ids, completed.output_artifact_ids);

    for (ordinal, (path, assertion)) in [
        (&vfr_media, "vfr"),
        (&rotation_media, "rotation"),
        (&audio_offset_media, "audio-offset"),
    ]
    .into_iter()
    .enumerate()
    {
        let registered = register_source(
            &socket,
            &format!("register-conformance-{ordinal}"),
            &project.project_id,
            path,
        )
        .await
        .expect("register conformance source")
        .source
        .expect("conformance source");
        let job = submit_probe(
            &socket,
            &format!("submit-conformance-{ordinal}"),
            &project.project_id,
            &registered.source_id,
        )
        .await
        .expect("submit conformance probe");
        let job = wait_terminal(&socket, &job.job_id).await;
        assert_eq!(job.state(), JobState::Succeeded, "{assertion} probe failed");
        let map = verified_source_map(&artifacts, &job.output_artifact_ids[0]).await;
        match assertion {
            "vfr" => assert_eq!(map["streams"][0]["video"]["vfr"], true),
            "rotation" => {
                assert_eq!(map["container"]["rotation_deg"], 90);
                assert_eq!(map["streams"][0]["video"]["display_width"], 90);
                assert_eq!(map["streams"][0]["video"]["display_height"], 160);
            }
            "audio-offset" => {
                let offset = map["streams"]
                    .as_array()
                    .expect("streams")
                    .iter()
                    .find(|stream| stream["index"] == 2)
                    .expect("offset audio stream")["start_offset_ticks"]
                    .as_i64()
                    .expect("audio offset ticks");
                assert_eq!(offset, 22_500);
            }
            _ => unreachable!(),
        }
    }

    let mut changed = fs::OpenOptions::new()
        .append(true)
        .open(&media)
        .expect("open source for mutation");
    changed.write_all(b"changed").expect("mutate source");
    changed.sync_all().expect("sync mutation");
    drop(changed);
    let changed_job = submit_probe(
        &socket,
        "submit-probe-changed",
        &project.project_id,
        &source.source_id,
    )
    .await
    .expect("submit changed source");
    let changed_job = wait_terminal(&socket, &changed_job.job_id).await;
    assert_eq!(changed_job.state(), JobState::Failed);
    assert_eq!(changed_job.failure_detail, "SOURCE_CHANGED");

    stop(shutdown, task).await;
    let (socket, _artifacts, shutdown, task) = running(daemon_config).await;
    let persisted = get_source(&socket, "get-after-restart", &source.source_id)
        .await
        .expect("source survives restart");
    assert_eq!(persisted.source_map_artifact_id, artifact_id.to_string());
    stop(shutdown, task).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "run through tools/drills/media-drill.sh with pinned FFmpeg"]
async fn hostile_paths_and_malformed_media_are_rejected() {
    let temp = workspace_tempdir();
    let (socket, _artifacts, shutdown, task) = running(config(&temp)).await;
    let project = create(&socket, "path-project", "Path rules")
        .await
        .expect("create project");
    let malformed = temp.path().join("malformed.mkv");
    fs::write(&malformed, b"not media").expect("malformed fixture");
    assert!(
        register_source(&socket, "malformed", &project.project_id, &malformed,)
            .await
            .is_err()
    );

    let symlink = temp.path().join("source-link.mkv");
    std::os::unix::fs::symlink(&malformed, &symlink).expect("source symlink");
    assert!(
        register_source(&socket, "symlink", &project.project_id, &symlink)
            .await
            .is_err()
    );
    assert!(
        register_source(&socket, "directory", &project.project_id, temp.path())
            .await
            .is_err()
    );
    let url = send(
        &socket,
        Request {
            request_id: "url".to_owned(),
            body: Some(request::Body::RegisterSource(RegisterSourceRequest {
                project_id: project.project_id,
                absolute_path: "https://example.com/video.mp4".to_owned(),
            })),
        },
    )
    .await
    .expect("URL rejection response");
    assert!(matches!(url.body, Some(response::Body::Error(error)) if error.code == 1));
    stop(shutdown, task).await;
}
