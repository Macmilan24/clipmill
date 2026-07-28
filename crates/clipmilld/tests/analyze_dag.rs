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
//! a stated reason and the shot detector is the worker under test. A full analysis
//! of a recording with speech needs the three pinned speech models and a worker
//! fleet no drill currently starts; that is W26's harness, and the speech chain's
//! own end-to-end coverage lives in `gate-speech`.
//!
//! Requires the pinned FFmpeg sidecars and the shots worker environment, so every
//! test is `#[ignore]` and driven by `just gate-ranking`.
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

fn spawn_shots_worker(data_dir: &Path, identity: &Path) -> Child {
    let executable = workspace().join("workers/shots/.venv/bin/clipmill-worker-shots");
    assert!(
        executable.is_file(),
        "shots worker environment is missing; run `uv sync --project workers/shots`"
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
        .expect("spawn shots worker")
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
    serde_json::from_slice(&raw).expect("the manifest is JSON")
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
#[ignore = "requires pinned FFmpeg and the shots worker environment"]
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
    let identity = temp.path().join("worker.json");
    provision_worker(&data_dir, &identity);
    let mut daemon = Reaped(spawn_analyze_daemon(&data_dir, &socket));
    wait_until_ready(&socket).await.expect("daemon ready");
    let mut worker = Reaped(spawn_shots_worker(&data_dir, &identity));

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
        ]),
        "silent footage produces the probe, the ingest, and the shot cuts"
    );

    // The skip list, which is the whole reason it exists: seven stages absent
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
        7,
        "four speech stages and three that read one"
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

    worker.kill();
    daemon.kill();
}

#[tokio::test]
#[ignore = "requires pinned FFmpeg and the shots worker environment"]
async fn a_daemon_killed_mid_analysis_finishes_within_the_recovery_slo() {
    let temp = workspace_tempdir();
    let data_dir = temp.path().join("data");
    fs::create_dir_all(&data_dir).expect("data dir");
    let socket = temp.path().join("d.sock");
    let media = temp.path().join("silent.mp4");
    generate_silent_video(&media, 6);

    // Before the daemon starts: it reads its trust map once, so a worker
    // provisioned afterwards is one it has never heard of.
    let identity = temp.path().join("worker.json");
    provision_worker(&data_dir, &identity);
    let mut daemon = Reaped(spawn_analyze_daemon(&data_dir, &socket));
    wait_until_ready(&socket).await.expect("daemon ready");
    let mut worker = Reaped(spawn_shots_worker(&data_dir, &identity));

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
    worker.kill();
    daemon.kill();

    let recovery_started = Instant::now();
    let mut daemon = Reaped(spawn_analyze_daemon(&data_dir, &socket));
    wait_until_ready(&socket).await.expect("daemon restarts");
    let mut worker = Reaped(spawn_shots_worker(&data_dir, &identity));

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

    worker.kill();
    daemon.kill();
}

/// Everything the fan-in names must be an artifact that is actually in the store,
/// because a shell walks this document to find what a project has.
#[tokio::test]
#[ignore = "requires pinned FFmpeg and the shots worker environment"]
async fn every_address_the_manifest_names_is_readable_from_the_store() {
    let temp = workspace_tempdir();
    let data_dir = temp.path().join("data");
    fs::create_dir_all(&data_dir).expect("data dir");
    let socket = temp.path().join("d.sock");
    let media = temp.path().join("silent.mp4");
    generate_silent_video(&media, 3);

    // Before the daemon starts: it reads its trust map once, so a worker
    // provisioned afterwards is one it has never heard of.
    let identity = temp.path().join("worker.json");
    provision_worker(&data_dir, &identity);
    let mut daemon = Reaped(spawn_analyze_daemon(&data_dir, &socket));
    wait_until_ready(&socket).await.expect("daemon ready");
    let mut worker = Reaped(spawn_shots_worker(&data_dir, &identity));

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

    worker.kill();
    daemon.kill();
}
