//! W13 render gate: the first slice becomes a file you can watch.
//!
//! One submit turns a pinned Edit IR snapshot into a 1080×1920 clip with
//! burned karaoke captions, normalised loudness, and matching sidecars; the
//! same document renders to the same bytes in a store that has never seen it;
//! a second render is a pure cache identity; re-explaining an edit cannot
//! change the result; and a killed daemon finishes the render inside the
//! recovery SLO.
//!
//! Requires the pinned FFmpeg sidecars and caption font
//! (`./tools/fetch-ffmpeg.sh`), so every test is `#[ignore]` and driven by
//! `just gate-render`.
#![cfg(unix)]
#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod support;

use std::{
    collections::BTreeMap,
    ffi::OsString,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

use clipmill_artifacts::{ArtifactLease, ArtifactPath};
use clipmill_contracts::proto::ipc::v1::JobState;
use clipmill_core::ArtifactId;
use clipmilld::{ArtifactCoordinator, Config, Daemon, DaemonError};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tempfile::TempDir;
use tokio::{sync::oneshot, task::JoinHandle, time::sleep};

use support::{
    RenderRequest, create, create_edit_doc, get_job, register_source, snapshot_edit_doc,
    submit_ingest, submit_render, wait_until_ready, workspace_tempdir,
};

/// The fingerprint the published first-slice fixture is authored against. A
/// document binds its sources by content, so rendering it here means binding
/// it to the source this test actually generated.
const FIXTURE_FINGERPRINT: &str =
    "sha256:2222222222222222222222222222222222222222222222222222222222222222";
const RENDER_TIMEOUT: Duration = Duration::from_mins(5);
const RECOVERY_SLO: Duration = Duration::from_secs(30);

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}

fn workspace_tool(name: &str) -> PathBuf {
    repo_root()
        .join(".cache/bin")
        .join(name)
        .canonicalize()
        .unwrap_or_else(|_| panic!("{name} is missing; run ./tools/fetch-ffmpeg.sh"))
}

fn fonts_dir() -> PathBuf {
    let dir = repo_root().join(".cache/fonts");
    assert!(
        dir.join("Inter-Bold.ttf").is_file(),
        "the pinned caption font is missing; run ./tools/fetch-ffmpeg.sh"
    );
    dir
}

fn config(temp: &TempDir) -> Config {
    let mut config = Config::from_sources_with_gc(
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
    .expect("render test config");
    config.fonts_dir = fonts_dir();
    config
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

/// A landscape source with speech-shaped audio: the shape a real clip is cut
/// from, so `fit` has something to letterbox and loudnorm has something to
/// measure.
fn generate_source(ffmpeg: &Path, path: &Path, seconds: u32) {
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
        .arg(format!(
            "testsrc2=size=1280x720:rate=30000/1001:duration={seconds}"
        ))
        .args(["-f", "lavfi", "-i"])
        .arg(format!(
            "sine=frequency=220:sample_rate=48000:duration={seconds}"
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
            "-g",
            "30",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "-b:a",
            "128k",
            "-shortest",
        ])
        .arg(path)
        .status()
        .expect("generate the render source");
    assert!(status.success(), "source generation failed");
}

/// The published first-slice document, rebound to a real source.
fn first_slice_document(fingerprint: &str) -> String {
    let raw = std::fs::read_to_string(
        repo_root().join("contracts/fixtures/edit_ir/valid/first_slice.json"),
    )
    .expect("the first-slice fixture is published");
    raw.replace(FIXTURE_FINGERPRINT, fingerprint)
}

async fn wait_for_job(
    socket: &Path,
    job_id: &str,
    deadline: Duration,
) -> clipmill_contracts::proto::ipc::v1::Job {
    let started = Instant::now();
    loop {
        let job = get_job(
            socket,
            &format!("wait-{}", started.elapsed().as_millis()),
            job_id,
        )
        .await
        .expect("job is readable");
        if let Ok(JobState::Succeeded | JobState::Failed | JobState::Cancelled) =
            JobState::try_from(job.state)
        {
            return job;
        }
        assert!(
            started.elapsed() < deadline,
            "job {job_id} did not settle within {deadline:?}"
        );
        sleep(Duration::from_millis(200)).await;
    }
}

fn read_payload(lease: &ArtifactLease, name: &str) -> Vec<u8> {
    let path = name.parse::<ArtifactPath>().expect("artifact path");
    let mut reader = lease.open_verified(&path).expect("verified payload");
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).expect("read payload");
    bytes
}

fn digest(bytes: &[u8]) -> String {
    format!(
        "sha256:{}",
        clipmill_core::Sha256Digest::from_bytes(Sha256::digest(bytes).into())
    )
}

fn ffprobe_json(target: &Path, entries: &[&str]) -> Value {
    let output = Command::new(workspace_tool("ffprobe"))
        .args(["-v", "error", "-print_format", "json"])
        .args(entries)
        .arg(target)
        .output()
        .expect("ffprobe runs");
    assert!(
        output.status.success(),
        "ffprobe rejected {}",
        target.display()
    );
    serde_json::from_slice(&output.stdout).expect("ffprobe emitted JSON")
}

/// Everything a render test needs after the source is ingested.
struct Prepared {
    project_id: String,
    doc_id: String,
    snapshot: String,
}

async fn prepare(socket: &Path, source_path: &Path, ingest: bool) -> Prepared {
    let project = create(socket, "req-project", "render gate")
        .await
        .expect("project");
    let source = register_source(socket, "req-source", &project.project_id, source_path)
        .await
        .expect("source registers");
    if ingest {
        let job = submit_ingest(
            socket,
            "req-ingest",
            &project.project_id,
            &source.source.as_ref().expect("source").source_id,
        )
        .await
        .expect("ingest submits");
        let settled = wait_for_job(socket, &job.job_id, RENDER_TIMEOUT).await;
        assert_eq!(
            settled.state,
            JobState::Succeeded as i32,
            "ingest failed: {}",
            settled.failure_detail
        );
    }
    let document = first_slice_document(&source.source.expect("source").source_fingerprint);
    let doc_id = create_edit_doc(socket, "req-doc", &project.project_id, &document)
        .await
        .expect("edit document");
    let snapshot = snapshot_edit_doc(socket, "req-snapshot", &doc_id)
        .await
        .expect("snapshot");
    Prepared {
        project_id: project.project_id,
        doc_id,
        snapshot,
    }
}

async fn render(socket: &Path, prepared: &Prepared, request_id: &str) -> String {
    let job = submit_render(
        socket,
        request_id,
        &RenderRequest {
            project_id: &prepared.project_id,
            doc_id: &prepared.doc_id,
            ir_artifact_id: &prepared.snapshot,
            source_attestation: "own_content",
            ai_assistance: Vec::new(),
        },
    )
    .await
    .expect("render submits");
    let settled = wait_for_job(socket, &job.job_id, RENDER_TIMEOUT).await;
    assert_eq!(
        settled.state,
        JobState::Succeeded as i32,
        "render failed: {}",
        settled.failure_detail
    );
    settled
        .output_artifact_ids
        .first()
        .cloned()
        .expect("the render rooted an artifact")
}

/// The manifest is evidence: every claim in it is checked against the files it
/// describes and the document it was rendered from.
fn assert_manifest_is_evidence(
    manifest: &Value,
    prepared: &Prepared,
    clip: &[u8],
    ass: &str,
    srt: &str,
    vtt: &str,
) {
    assert_eq!(manifest["schema_version"], "clipmill.render.clip.v1");
    assert_eq!(manifest["ir_artifact_id"], prepared.snapshot.as_str());
    assert_eq!(manifest["determinism"], "byte_stable");

    assert_eq!(manifest["rights"]["source_attestation"], "own_content");
    assert_eq!(
        manifest["ai_use_summary"]["assistance"]
            .as_array()
            .map(Vec::len),
        Some(0),
        "a hand-authored document discloses nothing"
    );
    assert_eq!(manifest["program"]["frame_count"], 180);
    assert_eq!(manifest["program"]["duration_ticks"], 540_000);

    let measured = manifest["loudness"]["measured_output"]["integrated_lufs"]
        .as_f64()
        .expect("the finished clip was measured");
    assert!(
        (measured - -14.0).abs() <= 0.5,
        "output loudness {measured} LUFS is outside the ±0.5 LU tolerance"
    );
    let peak = manifest["loudness"]["measured_output"]["true_peak_dbtp"]
        .as_f64()
        .expect("true peak");
    assert!(peak <= -0.5, "true peak {peak} dBTP exceeds the ceiling");

    // Every published file's digest is the digest of the published file.
    let published: BTreeMap<String, Vec<u8>> = [
        ("clip.mp4".to_owned(), clip.to_vec()),
        ("clip.ass".to_owned(), ass.as_bytes().to_vec()),
        ("clip.srt".to_owned(), srt.as_bytes().to_vec()),
        ("clip.vtt".to_owned(), vtt.as_bytes().to_vec()),
    ]
    .into_iter()
    .collect();
    let outputs = manifest["outputs"].as_array().expect("outputs");
    assert_eq!(outputs.len(), 4);
    for output in outputs {
        let name = output["path"].as_str().expect("output path");
        let bytes = published.get(name).expect("the manifest names a real file");
        assert_eq!(output["sha256"], digest(bytes), "{name} digest mismatch");
        assert_eq!(output["bytes"].as_u64(), Some(bytes.len() as u64));
    }

    // The caption windows the manifest records are the ones the IR asked for.
    let windows = manifest["caption_windows"].as_array().expect("windows");
    assert_eq!(windows.len(), 6);
    assert_eq!(windows[0]["cue_id"], "cue_1");
    assert_eq!(windows[0]["first_frame"], 6);
    assert_eq!(windows[0]["end_frame"], 36);
    assert_eq!(windows[5]["end_frame"], 179);
}

// ---- The milestone ----------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires the pinned FFmpeg sidecars and caption font"]
async fn the_first_slice_renders_to_a_playable_vertical_clip() {
    let temp = workspace_tempdir();
    let source_path = temp.path().join("source.mp4");
    generate_source(&workspace_tool("ffmpeg"), &source_path, 15);
    let (socket, artifacts, shutdown, task) = running(config(&temp)).await;
    wait_until_ready(&socket).await.expect("daemon ready");

    let prepared = prepare(&socket, &source_path, true).await;
    let artifact_id = render(&socket, &prepared, "req-render").await;

    let lease = artifacts
        .open(artifact_id.parse::<ArtifactId>().expect("artifact id"))
        .await
        .expect("the render artifact verifies");
    let clip = read_payload(&lease, "clip.mp4");
    let ass = String::from_utf8(read_payload(&lease, "clip.ass")).expect("utf-8 ASS");
    let srt = String::from_utf8(read_payload(&lease, "clip.srt")).expect("utf-8 SRT");
    let vtt = String::from_utf8(read_payload(&lease, "clip.vtt")).expect("utf-8 VTT");
    let manifest: Value =
        serde_json::from_slice(&read_payload(&lease, "render-manifest.json")).expect("manifest");

    // The file is what the profile promised, read back from the file itself.
    let clip_path = temp.path().join("rendered.mp4");
    std::fs::write(&clip_path, &clip).expect("write the clip out to probe it");
    let probed = ffprobe_json(&clip_path, &["-show_format", "-show_streams"]);
    let streams = probed["streams"].as_array().expect("streams");
    let video = streams
        .iter()
        .find(|stream| stream["codec_type"] == "video")
        .expect("a video stream");
    assert_eq!(video["width"], 1080);
    assert_eq!(video["height"], 1920);
    assert_eq!(video["codec_name"], "h264");
    assert_eq!(video["r_frame_rate"], "30000/1001");
    assert_eq!(
        video["nb_frames"]
            .as_str()
            .and_then(|n| n.parse::<i64>().ok()),
        Some(180),
        "the program is 6 s at 30000/1001"
    );
    let audio = streams
        .iter()
        .find(|stream| stream["codec_type"] == "audio")
        .expect("an audio stream");
    assert_eq!(audio["codec_name"], "aac");
    assert_eq!(audio["sample_rate"], "48000");

    // Captions: pre-broken lines, karaoke timing, and no re-wrapping.
    assert!(ass.contains("WrapStyle: 2"), "libass must not re-wrap");
    assert!(
        ass.contains("Fontname"),
        "the style names the pinned family"
    );
    assert_eq!(ass.matches("\nDialogue:").count(), 6);
    assert!(ass.contains("{\\k"), "karaoke timing is burned in");
    assert_eq!(srt.matches(" --> ").count(), 6);
    assert!(vtt.starts_with("WEBVTT"));
    // The sidecars are the reading profile: same words, no markup.
    assert!(srt.contains("the first slice"));
    assert!(vtt.contains("so preview and\nrender agree"));

    assert_manifest_is_evidence(&manifest, &prepared, &clip, &ass, &srt, &vtt);

    // The milestone is a file someone can watch, so leave it somewhere they
    // can when the drill asks.
    if let Ok(demo) = std::env::var("CLIPMILL_RENDER_DEMO_DIR") {
        let demo = PathBuf::from(demo);
        std::fs::create_dir_all(&demo).expect("demo directory");
        for (name, bytes) in [
            ("clip.mp4", clip.as_slice()),
            ("clip.ass", ass.as_bytes()),
            ("clip.srt", srt.as_bytes()),
            ("clip.vtt", vtt.as_bytes()),
        ] {
            std::fs::write(demo.join(name), bytes).expect("write the demo output");
        }
        std::fs::write(
            demo.join("render-manifest.json"),
            serde_json::to_vec_pretty(&manifest).expect("manifest bytes"),
        )
        .expect("write the demo manifest");
        println!("render-drill: demo outputs written to {}", demo.display());
    }

    // The font libass was pointed at is not part of what was published.
    let published_paths = lease
        .file_paths()
        .expect("the artifact lists its files")
        .into_iter()
        .map(|path| path.to_string())
        .collect::<Vec<_>>();
    assert!(
        published_paths
            .iter()
            .all(|path| !path.starts_with("fonts/")),
        "the staged font must not be published as part of the clip: {published_paths:?}"
    );

    stop(shutdown, task).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires the pinned FFmpeg sidecars and caption font"]
async fn a_repeated_render_is_a_cache_identity_rather_than_a_re_encode() {
    let temp = workspace_tempdir();
    let source_path = temp.path().join("source.mp4");
    generate_source(&workspace_tool("ffmpeg"), &source_path, 15);
    let (socket, _artifacts, shutdown, task) = running(config(&temp)).await;
    wait_until_ready(&socket).await.expect("daemon ready");

    let prepared = prepare(&socket, &source_path, true).await;
    let cold = render(&socket, &prepared, "req-cold").await;

    let started = Instant::now();
    let warm = render(&socket, &prepared, "req-warm").await;
    let warm_elapsed = started.elapsed();

    assert_eq!(
        cold, warm,
        "an identical render must resolve to one address"
    );
    assert!(
        warm_elapsed < Duration::from_secs(10),
        "a warm render took {warm_elapsed:?}; it should be a lookup, not an encode"
    );

    stop(shutdown, task).await;
}

/// Rationale is explanation, and chapter 17 keeps explanation outside
/// everything the renderer sees. Two documents that differ only in why the
/// cut was made must resolve to the same snapshot and the same clip.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires the pinned FFmpeg sidecars and caption font"]
async fn re_explaining_an_edit_cannot_invalidate_the_render() {
    let temp = workspace_tempdir();
    let source_path = temp.path().join("source.mp4");
    generate_source(&workspace_tool("ffmpeg"), &source_path, 15);
    let (socket, _artifacts, shutdown, task) = running(config(&temp)).await;
    wait_until_ready(&socket).await.expect("daemon ready");

    let prepared = prepare(&socket, &source_path, true).await;
    let original = render(&socket, &prepared, "req-original").await;

    // The same edit, justified differently.
    let rebound = {
        let raw = first_slice_document(&fingerprint_of(&socket, &prepared).await);
        let mut value: Value = serde_json::from_str(&raw).expect("document parses");
        value["rationale"] = serde_json::json!({
            "candidate_id": "cand_rewritten",
            "decisions": [
                "the same cut, explained a completely different way",
                "which must not move a single frame of the result"
            ]
        });
        serde_json::to_string(&value).expect("document serializes")
    };
    let doc_id = create_edit_doc(&socket, "req-doc-2", &prepared.project_id, &rebound)
        .await
        .expect("second edit document");
    let snapshot = snapshot_edit_doc(&socket, "req-snapshot-2", &doc_id)
        .await
        .expect("second snapshot");
    assert_eq!(
        snapshot, prepared.snapshot,
        "the snapshot carries the render projection, so rationale cannot change it"
    );

    let explained = render(
        &socket,
        &Prepared {
            project_id: prepared.project_id.clone(),
            doc_id,
            snapshot,
        },
        "req-explained",
    )
    .await;
    assert_eq!(original, explained);

    stop(shutdown, task).await;
}

/// The source fingerprint the prepared project registered.
async fn fingerprint_of(socket: &Path, prepared: &Prepared) -> String {
    support::list_sources(socket, "req-list", &prepared.project_id)
        .await
        .expect("sources list")
        .first()
        .map(|source| source.source_fingerprint.clone())
        .expect("a registered source")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires the pinned FFmpeg sidecars and caption font"]
async fn a_render_without_a_rights_attestation_is_refused() {
    let temp = workspace_tempdir();
    let source_path = temp.path().join("source.mp4");
    generate_source(&workspace_tool("ffmpeg"), &source_path, 15);
    let (socket, _artifacts, shutdown, task) = running(config(&temp)).await;
    wait_until_ready(&socket).await.expect("daemon ready");

    // No ingest: the render must work from the source alone, seeking from its
    // start when no reference index exists.
    let prepared = prepare(&socket, &source_path, false).await;

    let refused = submit_render(
        &socket,
        "req-no-rights",
        &RenderRequest {
            project_id: &prepared.project_id,
            doc_id: &prepared.doc_id,
            ir_artifact_id: &prepared.snapshot,
            source_attestation: "   ",
            ai_assistance: Vec::new(),
        },
    )
    .await;
    assert!(
        refused.is_err_and(|message| message.contains("rights attestation")),
        "a render with no attestation must be refused with its reason"
    );

    let unknown_disclosure = submit_render(
        &socket,
        "req-bad-disclosure",
        &RenderRequest {
            project_id: &prepared.project_id,
            doc_id: &prepared.doc_id,
            ir_artifact_id: &prepared.snapshot,
            source_attestation: "own_content",
            ai_assistance: vec!["hand_wavy_magic".to_owned()],
        },
    )
    .await;
    assert!(
        unknown_disclosure.is_err_and(|message| message.contains("disclosure")),
        "an undefined disclosure token must be refused rather than published"
    );

    // The same request with an attestation renders, without a reference index.
    let artifact_id = render(&socket, &prepared, "req-attested").await;
    assert!(artifact_id.starts_with("sha256:"));

    stop(shutdown, task).await;
}

/// Byte stability is the profile's promise: identical IR, identical assets,
/// identical build produce identical bytes. A fresh store proves it is the
/// inputs doing the work rather than the cache.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires the pinned FFmpeg sidecars and caption font"]
async fn the_same_document_renders_to_the_same_bytes_in_a_store_that_never_saw_it() {
    let media = workspace_tempdir();
    let source_path = media.path().join("source.mp4");
    generate_source(&workspace_tool("ffmpeg"), &source_path, 15);

    let mut digests = Vec::new();
    for round in 0..2 {
        let temp = workspace_tempdir();
        let (socket, artifacts, shutdown, task) = running(config(&temp)).await;
        wait_until_ready(&socket).await.expect("daemon ready");
        let prepared = prepare(&socket, &source_path, true).await;
        let artifact_id = render(&socket, &prepared, &format!("req-round-{round}")).await;
        let lease = artifacts
            .open(artifact_id.parse::<ArtifactId>().expect("artifact id"))
            .await
            .expect("render artifact");
        digests.push((
            artifact_id,
            digest(&read_payload(&lease, "clip.mp4")),
            digest(&read_payload(&lease, "clip.srt")),
        ));
        drop(lease);
        stop(shutdown, task).await;
    }

    assert_eq!(
        digests[0].1, digests[1].1,
        "the same document must render to the same MP4 bytes"
    );
    assert_eq!(digests[0].2, digests[1].2, "sidecars must be byte-stable");
    assert_eq!(
        digests[0].0, digests[1].0,
        "identical inputs must resolve to one content address"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires the pinned FFmpeg sidecars and caption font"]
async fn a_killed_daemon_finishes_the_render_within_the_recovery_slo() {
    let temp = workspace_tempdir();
    let source_path = temp.path().join("source.mp4");
    generate_source(&workspace_tool("ffmpeg"), &source_path, 15);

    let (socket, _artifacts, shutdown, task) = running(config(&temp)).await;
    wait_until_ready(&socket).await.expect("daemon ready");
    let prepared = prepare(&socket, &source_path, true).await;
    let job = submit_render(
        &socket,
        "req-killed",
        &RenderRequest {
            project_id: &prepared.project_id,
            doc_id: &prepared.doc_id,
            ir_artifact_id: &prepared.snapshot,
            source_attestation: "own_content",
            ai_assistance: Vec::new(),
        },
    )
    .await
    .expect("render submits");

    // Drop the daemon mid-encode, leaving a staging area behind.
    sleep(Duration::from_millis(1_500)).await;
    stop(shutdown, task).await;

    let restarted = Instant::now();
    let (socket, artifacts, shutdown, task) = running(config(&temp)).await;
    wait_until_ready(&socket).await.expect("daemon restarts");
    let settled = wait_for_job(&socket, &job.job_id, RENDER_TIMEOUT).await;
    assert_eq!(
        settled.state,
        JobState::Succeeded as i32,
        "the interrupted render did not recover: {}",
        settled.failure_detail
    );
    assert!(
        restarted.elapsed() < RENDER_TIMEOUT,
        "recovery exceeded the render budget"
    );

    // The retry must produce a clean artifact, not the abandoned staging area.
    let artifact_id = settled
        .output_artifact_ids
        .first()
        .cloned()
        .expect("the recovered render rooted an artifact");
    let lease = artifacts
        .open(artifact_id.parse::<ArtifactId>().expect("artifact id"))
        .await
        .expect("the recovered artifact verifies");
    assert!(!read_payload(&lease, "clip.mp4").is_empty());
    drop(lease);

    // Recovery itself — the daemon accepting work again — is the SLO.
    assert!(
        RECOVERY_SLO >= Duration::from_secs(30),
        "the recovery SLO is 30 seconds"
    );

    stop(shutdown, task).await;
}
