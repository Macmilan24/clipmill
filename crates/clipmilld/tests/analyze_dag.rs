//! W19 analyze gate: the whole pipeline as one job.
//!
//! Everything the plan-level tests in `jobs::analyze_tests` cannot see, because
//! it only exists once a real store and a real worker are involved:
//!
//!   The plan is accepted. Four standalone stage jobs could not be submitted at
//!   all before this workstream — they named an input kind with no dependency
//!   behind it, which the validator rejects — so "the store took it" is a real
//!   assertion rather than a formality.
//!
//!   Declared inputs reach a worker's lease. An address that travelled only in
//!   the stage payload named an artifact the worker was forbidden to open, and no
//!   test could catch that without a worker actually opening one.
//!
//!   The fan-in publishes a document that validates against its own schema, names
//!   every stage that ran, and accounts for the ones that did not.
//!
//!   Cold and warm agree. A second analysis of the same source resolves to the
//!   same artifact identities rather than deriving a second copy of each answer.
//!
//!   A killed daemon finishes the DAG inside the 30-second recovery SLO.
//!
//! The source here carries video and no audio, so the speech half is skipped with
//! a stated reason and the shot and face detectors are the workers under test.
//! Speech analysis is covered by `gate-speech` and the full fleet by `gate-lock-phase1`.
//!
//! Requires pinned FFmpeg, `YuNet` weights, and both visual worker environments,
//! so every test is `#[ignore]` and driven by `just gate-ranking`.
#![cfg(unix)]
#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod support;

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

use clipmill_artifacts::ArtifactPath;
use clipmill_contracts::proto::ipc::v1::{Job, JobState};
use clipmill_core::ArtifactId;
use serde_json::Value;
use tokio::time::sleep;

use support::{
    create, get_job, register_source, submit_analyze, submit_probe, wait_until_ready,
    workspace_tempdir,
};

/// The SLO the plan states for recovery after a kill, anywhere in the DAG.
const RECOVERY_SLO: Duration = Duration::from_secs(30);
/// Generous, because this runs a real decode on whatever machine CI gave us.
const COMPLETION_TIMEOUT: Duration = Duration::from_mins(3);

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn workspace_tool(name: &str) -> PathBuf {
    workspace()
        .join(".cache/bin")
        .join(name)
        .canonicalize()
        .unwrap_or_else(|_| panic!("{name} is missing; run ./tools/fetch-ffmpeg.sh"))
}

/// Silent footage: video with no audio track at all, which is what makes the
/// speech half of the analysis a stated skip rather than a failure.
fn generate_silent_video(path: &Path, seconds: u32) {
    let status = Command::new(workspace_tool("ffmpeg"))
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
        .args([
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-pix_fmt",
            "yuv420p",
        ])
        .arg(path)
        .status()
        .expect("run pinned FFmpeg");
    assert!(status.success(), "silent video fixture generation failed");
}

/// The daemon under test, told where the pinned prober is.
///
/// Its own spawn rather than the shared one because an analysis begins with a
/// probe: a daemon that has to find FFprobe on the PATH would produce a source
/// map from whatever build the machine happens to carry, and the fan-out is
/// shaped from that map.
fn spawn_analyze_daemon(data_dir: &Path, socket: &Path) -> Child {
    Command::new(env!("CARGO_BIN_EXE_clipmilld"))
        .arg("--data-dir")
        .arg(data_dir)
        .arg("--socket")
        .arg(socket)
        .env(
            "RUST_LOG",
            std::env::var("CLIPMILL_TEST_DAEMON_LOG").unwrap_or_else(|_| "error".to_owned()),
        )
        .env("CLIPMILL_FFPROBE", workspace_tool("ffprobe"))
        .env("CLIPMILL_MODELS_DIR", workspace().join("models/registry"))
        .env("CLIPMILL_WEIGHTS_DIR", workspace().join(".cache/models"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn clipmilld")
}

fn provision_worker(data_dir: &Path, identity: &Path) {
    let status = Command::new(env!("CARGO_BIN_EXE_clipmill-worker-keygen"))
        .arg("--data-dir")
        .arg(data_dir)
        .arg("--identity")
        .arg(identity)
        .stdin(Stdio::null())
        .stderr(Stdio::inherit())
        .status()
        .expect("run worker key generator");
    assert!(status.success(), "worker key generator failed: {status}");
}

fn spawn_visual_worker(kind: &str, data_dir: &Path, identity: &Path) -> Child {
    let executable = workspace().join(format!("workers/{kind}/.venv/bin/clipmill-worker-{kind}"));
    assert!(
        executable.is_file(),
        "{kind} worker environment is missing; run `uv sync --project workers/{kind}`"
    );
    Command::new(executable)
        .arg("--identity")
        .arg(identity)
        .arg("--data-dir")
        .arg(data_dir)
        .env("CLIPMILL_WORKER_LOG", "WARNING")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn visual worker")
}

fn provision_visual_workers(data_dir: &Path, identities_dir: &Path) -> [PathBuf; 2] {
    ["shots", "faces"].map(|kind| {
        let identity = identities_dir.join(format!("worker-{kind}.json"));
        provision_worker(data_dir, &identity);
        identity
    })
}

struct VisualWorkers([Reaped; 2]);

impl VisualWorkers {
    fn spawn(data_dir: &Path, identities: &[PathBuf; 2]) -> Self {
        Self(std::array::from_fn(|index| {
            Reaped(spawn_visual_worker(
                ["shots", "faces"][index],
                data_dir,
                &identities[index],
            ))
        }))
    }

    fn kill(&mut self) {
        for worker in &mut self.0 {
            worker.kill();
        }
    }
}

/// A child killed when the test leaves, so a failure does not leak a daemon.
struct Reaped(Child);

impl Reaped {
    fn kill(&mut self) {
        let _killed = self.0.kill();
        let _waited = self.0.wait();
    }
}

impl Drop for Reaped {
    fn drop(&mut self) {
        self.kill();
    }
}

async fn wait_for_job(socket: &Path, prefix: &str, job_id: &str, timeout: Duration) -> Job {
    let deadline = Instant::now() + timeout;
    let mut attempt = 0_u64;
    loop {
        attempt += 1;
        let job = get_job(socket, &format!("{prefix}-{attempt}"), job_id)
            .await
            .expect("get job");
        if job.state == JobState::Succeeded as i32 {
            return job;
        }
        assert!(
            job.state != JobState::Failed as i32,
            "analyze failed: {}",
            job.failure_detail
        );
        assert!(
            Instant::now() < deadline,
            "analyze did not finish in time (last state {}, tasks {:?})",
            job.state,
            job.tasks
                .iter()
                .map(|task| (task.kind.as_str(), task.state, task.wait_reason.as_str()))
                .collect::<Vec<_>>()
        );
        sleep(Duration::from_millis(200)).await;
    }
}

/// The rooted manifest, read out of the store the daemon wrote it to.
fn read_manifest(data_dir: &Path, job: &Job) -> Value {
    assert_eq!(
        job.output_artifact_ids.len(),
        1,
        "an analysis roots exactly one artifact"
    );
    let artifact_id = job.output_artifact_ids[0]
        .parse::<ArtifactId>()
        .expect("manifest artifact id");
    let digest = artifact_id.to_string();
    let digest = digest.strip_prefix("sha256:").expect("an address");
    let object = data_dir
        .join("artifacts/objects/sha256")
        .join(&digest[..2])
        .join(digest);
    let raw = fs::read(object.join("analysis.json")).expect("read the analysis manifest");
    let manifest = serde_json::from_slice(&raw).expect("the manifest is JSON");
    assert_measured_face_evidence(data_dir, &manifest);
    manifest
}

fn artifact_document(data_dir: &Path, address: &str, file: &str) -> Value {
    let _: ArtifactId = address.parse().expect("artifact address");
    let digest = address.strip_prefix("sha256:").expect("digest");
    let object = data_dir
        .join("artifacts/objects/sha256")
        .join(&digest[..2])
        .join(digest);
    serde_json::from_slice(&fs::read(object.join(file)).expect("published document"))
        .expect("document JSON")
}

fn assert_measured_face_evidence(data_dir: &Path, manifest: &Value) {
    let stages = stage_addresses(manifest);
    let address = stages
        .get("vision.face_track.v1")
        .expect("face detection is rooted");
    let faces = artifact_document(data_dir, address, "faces.json");
    let artifact = artifact_document(data_dir, address, "manifest.json");
    let frames = artifact_document(
        data_dir,
        faces["frames_artifact_id"]
            .as_str()
            .expect("exact frames input"),
        "index.json",
    );
    let examined = faces["coverage"]["frames_examined"]
        .as_u64()
        .expect("frame count");
    assert!(
        examined > 0,
        "an empty result must still examine the source frames"
    );
    assert_eq!(
        examined,
        u64::try_from(frames["frames"].as_array().expect("sampled frames").len())
            .expect("frame count")
    );
    assert_eq!(faces["coverage"]["analyzed"], true);
    assert_eq!(faces["source_fingerprint"], manifest["source_fingerprint"]);
    assert_eq!(faces["producer"]["stage"], "detect-faces");
    // The recipe records the registry bundle identity, including revision and
    // filenames, rather than the hash of the single ONNX file. Re-pinning the
    // model deliberately changes this expected identity.
    assert_eq!(
        artifact["producer"]["model_digest"],
        "sha256:0ca8930cd602aab4b920c70f8ad8caa59bda9da938cfb46dd0e878d8102cb027"
    );
}

fn stage_addresses(manifest: &Value) -> BTreeMap<String, String> {
    manifest["stages"]
        .as_array()
        .expect("stages")
        .iter()
        .map(|stage| {
            (
                stage["kind"].as_str().expect("kind").to_owned(),
                stage["artifact_id"].as_str().expect("address").to_owned(),
            )
        })
        .collect()
}

/// Set up a project, a source, and a probed source map — the precondition an
/// analysis needs before its fan-out can be shaped.
async fn probed_source(socket: &Path, media: &Path) -> (String, String) {
    let project = create(socket, "req-project", "analyze")
        .await
        .expect("create project");
    let registered = register_source(socket, "req-source", &project.project_id, media)
        .await
        .expect("register source");
    let source_id = registered.source.expect("a registered source").source_id;
    let probe = submit_probe(socket, "req-probe", &project.project_id, &source_id)
        .await
        .expect("submit probe");
    wait_for_job(socket, "req-probe-poll", &probe.job_id, COMPLETION_TIMEOUT).await;
    (project.project_id, source_id)
}

#[tokio::test]
#[ignore = "requires pinned FFmpeg, YuNet, and both visual worker environments"]
#[allow(clippy::too_many_lines)]
async fn analyze_runs_the_whole_pipeline_and_agrees_with_itself_when_warm() {
    let temp = workspace_tempdir();
    let data_dir = temp.path().join("data");
    fs::create_dir_all(&data_dir).expect("data dir");
    let socket = temp.path().join("d.sock");
    let media = temp.path().join("silent.mp4");
    generate_silent_video(&media, 4);

    // Before the daemon starts: it reads its trust map once, so a worker
    // provisioned afterwards is one it has never heard of.
    let identities = provision_visual_workers(&data_dir, temp.path());
    let mut daemon = Reaped(spawn_analyze_daemon(&data_dir, &socket));
    wait_until_ready(&socket).await.expect("daemon ready");
    let mut workers = VisualWorkers::spawn(&data_dir, &identities);

    let (project_id, source_id) = probed_source(&socket, &media).await;

    // Cold. Submitting at all is the first assertion: a plan whose stages named
    // input kinds without dependencies behind them would be refused here.
    let cold = submit_analyze(&socket, "req-cold", &project_id, &source_id)
        .await
        .expect("the store accepts an analyze plan");
    let cold = wait_for_job(&socket, "req-cold-poll", &cold.job_id, COMPLETION_TIMEOUT).await;

    let manifest = read_manifest(&data_dir, &cold);
    assert_eq!(
        manifest["schema_version"], "clipmill.analysis.manifest.v1",
        "the fan-in declares its schema"
    );
    let addresses = stage_addresses(&manifest);
    assert_eq!(
        addresses.keys().cloned().collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "evidence.source_map.v1".to_owned(),
            "media.ingest_manifest.v1".to_owned(),
            "evidence.shots.v1".to_owned(),
            "vision.face_track.v1".to_owned(),
        ]),
        "silent footage produces probe, ingest, shot cuts, and measured face evidence"
    );

    // The skip list, which is the whole reason it exists: eight stages absent
    // because this recording has no audio, each saying so.
    let skipped = manifest["skipped"]
        .as_array()
        .expect("a silent source skips the speech half")
        .iter()
        .map(|stage| {
            (
                stage["kind"].as_str().expect("kind").to_owned(),
                stage["reason"].as_str().expect("reason").to_owned(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        skipped.len(),
        8,
        "four speech stages and four that read one"
    );
    assert!(skipped.iter().all(|(_, reason)| reason == "no_audio"));

    // Coverage comes from the stage that measured the recording, not from the
    // container's duration.
    let coverage = &manifest["coverage"];
    assert!(coverage["analyzed"].as_bool().expect("analyzed"));
    assert!(
        coverage["end_ticks"].as_u64().expect("end")
            > coverage["start_ticks"].as_u64().expect("start")
    );

    // Warm. Every stage resolves to the address it already has, including the
    // one a worker produced — which is the property that makes an analysis over
    // an already-ingested source cost nothing.
    let warm = submit_analyze(&socket, "req-warm", &project_id, &source_id)
        .await
        .expect("submit a second analysis");
    let warm = wait_for_job(&socket, "req-warm-poll", &warm.job_id, COMPLETION_TIMEOUT).await;
    assert_eq!(
        warm.output_artifact_ids, cold.output_artifact_ids,
        "a warm analysis roots the same manifest rather than a second copy"
    );
    assert_eq!(
        stage_addresses(&read_manifest(&data_dir, &warm)),
        addresses,
        "a warm analysis names the same artifact for every stage"
    );

    workers.kill();
    daemon.kill();
}

#[tokio::test]
#[ignore = "requires pinned FFmpeg, YuNet, and both visual worker environments"]
async fn a_daemon_killed_mid_analysis_finishes_within_the_recovery_slo() {
    let temp = workspace_tempdir();
    let data_dir = temp.path().join("data");
    fs::create_dir_all(&data_dir).expect("data dir");
    let socket = temp.path().join("d.sock");
    let media = temp.path().join("silent.mp4");
    generate_silent_video(&media, 6);

    // Before the daemon starts: it reads its trust map once, so a worker
    // provisioned afterwards is one it has never heard of.
    let identities = provision_visual_workers(&data_dir, temp.path());
    let mut daemon = Reaped(spawn_analyze_daemon(&data_dir, &socket));
    wait_until_ready(&socket).await.expect("daemon ready");
    let mut workers = VisualWorkers::spawn(&data_dir, &identities);

    let (project_id, source_id) = probed_source(&socket, &media).await;
    let submitted = submit_analyze(&socket, "req-kill", &project_id, &source_id)
        .await
        .expect("submit analyze");

    // Killed partway through, wherever the DAG happens to be. The point is that
    // no stage is left half-published: a task either committed its artifact or
    // it did not, and the second daemon re-runs whichever did not.
    let job = get_job(&socket, "req-kill-peek", &submitted.job_id)
        .await
        .expect("get job");
    assert_ne!(
        job.state,
        JobState::Succeeded as i32,
        "killed too late to test"
    );
    workers.kill();
    daemon.kill();

    let recovery_started = Instant::now();
    let mut daemon = Reaped(spawn_analyze_daemon(&data_dir, &socket));
    wait_until_ready(&socket).await.expect("daemon restarts");
    let mut workers = VisualWorkers::spawn(&data_dir, &identities);

    let finished = wait_for_job(
        &socket,
        "req-kill-poll",
        &submitted.job_id,
        COMPLETION_TIMEOUT,
    )
    .await;
    let elapsed = recovery_started.elapsed();
    assert!(
        elapsed < RECOVERY_SLO,
        "recovery took {elapsed:?}, over the {RECOVERY_SLO:?} SLO"
    );
    assert_eq!(finished.output_artifact_ids.len(), 1);
    let manifest = read_manifest(&data_dir, &finished);
    assert_eq!(manifest["schema_version"], "clipmill.analysis.manifest.v1");

    workers.kill();
    daemon.kill();
}

/// Everything the fan-in names must be an artifact that is actually in the store,
/// because a shell walks this document to find what a project has.
#[tokio::test]
#[ignore = "requires pinned FFmpeg, YuNet, and both visual worker environments"]
async fn every_address_the_manifest_names_is_readable_from_the_store() {
    let temp = workspace_tempdir();
    let data_dir = temp.path().join("data");
    fs::create_dir_all(&data_dir).expect("data dir");
    let socket = temp.path().join("d.sock");
    let media = temp.path().join("silent.mp4");
    generate_silent_video(&media, 3);

    // Before the daemon starts: it reads its trust map once, so a worker
    // provisioned afterwards is one it has never heard of.
    let identities = provision_visual_workers(&data_dir, temp.path());
    let mut daemon = Reaped(spawn_analyze_daemon(&data_dir, &socket));
    wait_until_ready(&socket).await.expect("daemon ready");
    let mut workers = VisualWorkers::spawn(&data_dir, &identities);

    let (project_id, source_id) = probed_source(&socket, &media).await;
    let job = submit_analyze(&socket, "req-walk", &project_id, &source_id)
        .await
        .expect("submit analyze");
    let job = wait_for_job(&socket, "req-walk-poll", &job.job_id, COMPLETION_TIMEOUT).await;

    for (kind, address) in stage_addresses(&read_manifest(&data_dir, &job)) {
        let digest = address.strip_prefix("sha256:").expect("an address");
        let object = data_dir
            .join("artifacts/objects/sha256")
            .join(&digest[..2])
            .join(digest);
        let manifest_path = object.join("manifest.json");
        let raw = fs::read(&manifest_path)
            .unwrap_or_else(|error| panic!("{kind} at {address} is not in the store: {error}"));
        let artifact: Value = serde_json::from_slice(&raw).expect("artifact manifest JSON");
        assert_eq!(
            artifact["kind"], kind,
            "the analysis names {address} as a {kind}, and the store disagrees"
        );
        // Every file the artifact declares is present and its digest matches, so
        // walking the manifest lands on bytes rather than on a promise.
        for file in artifact["files"].as_array().expect("files") {
            let name = file["path"].as_str().expect("path");
            let path = name.parse::<ArtifactPath>().expect("a valid artifact path");
            let mut bytes = Vec::new();
            fs::File::open(object.join(path.as_str()))
                .unwrap_or_else(|error| panic!("{kind} is missing {name}: {error}"))
                .read_to_end(&mut bytes)
                .expect("read the payload");
            let digest = <sha2::Sha256 as sha2::Digest>::digest(&bytes);
            assert_eq!(
                file["sha256"],
                format!("sha256:{}", hex::encode(digest)),
                "{kind}/{name} does not match"
            );
        }
    }

    workers.kill();
    daemon.kill();
}
