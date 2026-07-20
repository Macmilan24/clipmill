#![cfg(unix)]
#![allow(clippy::expect_used, clippy::panic)]

mod support;

use std::{
    path::Path,
    process::{Child, Command, ExitStatus, Stdio},
    time::{Duration, Instant},
};

use clipmill_contracts::proto::ipc::v1::{Job, JobState, TaskState};
use sha2::{Digest, Sha256};
use support::{
    cancel_job, create, get_job, signal_terminate, submit_demo, wait_for_exit, wait_until_ready,
    workspace_tempdir,
};
use tokio::time::sleep;

#[tokio::test]
#[ignore = "run by tools/drills/worker-drill.sh"]
async fn authenticated_echo_worker_completes_dag_and_replays_lost_ack() {
    let temp = workspace_tempdir();
    let data_dir = temp.path().join("data");
    let control_socket = temp.path().join("control.sock");
    let identity = temp.path().join("echo-worker.json");
    let untrusted_data_dir = temp.path().join("untrusted-data");
    let untrusted_identity = temp.path().join("untrusted-worker.json");
    provision_worker(&data_dir, &identity);
    provision_worker(&untrusted_data_dir, &untrusted_identity);

    let mut daemon = ChildGuard::new(spawn_production_daemon_with_options(
        &data_dir,
        &control_socket,
        true,
    ));
    wait_until_ready(&control_socket)
        .await
        .expect("production daemon ready");
    assert_private_socket(&data_dir.join("run/clipmill-workers.sock"));
    assert_private_socket(&data_dir.join("run/clipmill-shm.sock"));

    let mut untrusted = ChildGuard::new(spawn_echo_worker(&data_dir, &untrusted_identity, None));
    let project = create(&control_socket, "worker-project", "Worker recovery")
        .await
        .expect("project");
    let submitted = submit_demo(
        &control_socket,
        "worker-demo",
        &project.project_id,
        b"authenticated-shared-memory",
    )
    .await
    .expect("submit external demo job");
    sleep(Duration::from_millis(500)).await;
    let unauthenticated = get_job(
        &control_socket,
        "untrusted-worker-observation",
        &submitted.job_id,
    )
    .await
    .expect("read job while only an untrusted worker connects");
    assert!(
        unauthenticated
            .tasks
            .iter()
            .all(|task| task.state == TaskState::Planned as i32)
    );
    untrusted.terminate().await.expect("untrusted worker exits");

    let mut worker = ChildGuard::new(spawn_echo_worker(&data_dir, &identity, None));
    let completed = wait_for_job_state(
        &control_socket,
        &submitted.job_id,
        JobState::Succeeded,
        Duration::from_secs(30),
    )
    .await;
    assert_eq!(completed.tasks.len(), 4);
    assert!(completed.tasks.iter().all(|task| {
        task.state == TaskState::Succeeded as i32
            && task.attempt == 1
            && !task.output_artifact_id.is_empty()
    }));
    assert_eq!(completed.output_artifact_ids.len(), 1);

    worker.terminate().await.expect("echo worker exits");
    daemon.terminate().await.expect("daemon exits");
    assert!(!control_socket.exists());
    assert!(!data_dir.join("run/clipmill-workers.sock").exists());
    assert!(!data_dir.join("run/clipmill-shm.sock").exists());
}

#[tokio::test]
#[ignore = "run by tools/drills/worker-drill.sh"]
async fn worker_and_daemon_death_recover_without_partial_outputs() {
    let temp = workspace_tempdir();
    let data_dir = temp.path().join("recovery-data");
    let control_socket = temp.path().join("recovery.sock");
    let identity = temp.path().join("recovery-worker.json");
    provision_worker(&data_dir, &identity);

    let mut daemon = ChildGuard::new(spawn_production_daemon(&data_dir, &control_socket));
    wait_until_ready(&control_socket)
        .await
        .expect("daemon ready");
    let project = create(&control_socket, "recovery-project", "Recovery")
        .await
        .expect("project");

    let mut killed_worker = ChildGuard::new(spawn_echo_worker(&data_dir, &identity, Some(10_000)));
    let worker_death_job = submit_demo(
        &control_socket,
        "worker-death-submit",
        &project.project_id,
        b"worker-death",
    )
    .await
    .expect("worker-death job");
    wait_for_running_task(&control_socket, &worker_death_job.job_id).await;
    killed_worker.kill().await.expect("hard-kill worker");
    wait_for_empty_staging(&data_dir, Duration::from_secs(5)).await;

    let mut worker = ChildGuard::new(spawn_echo_worker(&data_dir, &identity, None));
    let recovered = wait_for_job_state(
        &control_socket,
        &worker_death_job.job_id,
        JobState::Succeeded,
        Duration::from_secs(35),
    )
    .await;
    assert!(recovered.tasks.iter().any(|task| task.attempt >= 2));
    verify_job_outputs(&data_dir, &recovered);

    worker.terminate().await.expect("replacement worker exits");
    let mut cancelling = ChildGuard::new(spawn_echo_worker(&data_dir, &identity, Some(10_000)));
    let cancellation_job = submit_demo(
        &control_socket,
        "cancellation-submit",
        &project.project_id,
        b"cooperative-cancellation",
    )
    .await
    .expect("cancellation job");
    wait_for_running_task(&control_socket, &cancellation_job.job_id).await;
    let cancelled = cancel_job(
        &control_socket,
        "cancel-running-worker-job",
        &cancellation_job.job_id,
    )
    .await
    .expect("cancel running worker job");
    assert_eq!(cancelled.state, JobState::Cancelled as i32);
    wait_for_empty_staging(&data_dir, Duration::from_secs(7)).await;
    cancelling
        .terminate()
        .await
        .expect("cancelled worker exits");

    let mut reconnecting = ChildGuard::new(spawn_echo_worker(&data_dir, &identity, Some(10_000)));
    let daemon_death_job = submit_demo(
        &control_socket,
        "daemon-death-submit",
        &project.project_id,
        b"daemon-death",
    )
    .await
    .expect("daemon-death job");
    wait_for_running_task(&control_socket, &daemon_death_job.job_id).await;
    daemon.kill().await.expect("hard-kill daemon");

    daemon = ChildGuard::new(spawn_production_daemon(&data_dir, &control_socket));
    wait_until_ready(&control_socket)
        .await
        .expect("restarted daemon ready");
    let restarted = wait_for_job_state(
        &control_socket,
        &daemon_death_job.job_id,
        JobState::Succeeded,
        Duration::from_secs(30),
    )
    .await;
    verify_job_outputs(&data_dir, &restarted);
    wait_for_empty_staging(&data_dir, Duration::from_secs(5)).await;

    reconnecting
        .terminate()
        .await
        .expect("reconnected worker exits");
    daemon.terminate().await.expect("restarted daemon exits");
}

fn provision_worker(data_dir: &Path, identity: &Path) {
    let status = Command::new(env!("CARGO_BIN_EXE_clipmill-worker-keygen"))
        .arg("--data-dir")
        .arg(data_dir)
        .arg("--identity")
        .arg(identity)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("run worker key generator");
    assert!(status.success(), "worker key generator failed: {status}");
}

fn spawn_production_daemon(data_dir: &Path, control_socket: &Path) -> Child {
    spawn_production_daemon_with_options(data_dir, control_socket, false)
}

fn spawn_production_daemon_with_options(
    data_dir: &Path,
    control_socket: &Path,
    drop_completion_ack_once: bool,
) -> Child {
    let rust_log = std::env::var("CLIPMILL_TEST_DAEMON_LOG").unwrap_or_else(|_| "error".to_owned());
    let mut command = Command::new(env!("CARGO_BIN_EXE_clipmilld"));
    command
        .arg("--data-dir")
        .arg(data_dir)
        .arg("--socket")
        .arg(control_socket)
        .env("RUST_LOG", rust_log)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    if drop_completion_ack_once {
        command.env("CLIPMILL_TEST_DROP_COMPLETION_ACK_ONCE", "1");
    }
    command.spawn().expect("spawn production daemon")
}

fn spawn_echo_worker(data_dir: &Path, identity: &Path, delay_once_ms: Option<u64>) -> Child {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root");
    let executable = workspace.join("workers/echo/.venv/bin/clipmill-worker-echo");
    assert!(
        executable.is_file(),
        "echo worker environment is missing; run `uv sync --project workers/echo`"
    );
    let mut command = Command::new(executable);
    command
        .arg("--identity")
        .arg(identity)
        .arg("--data-dir")
        .arg(data_dir)
        .env("CLIPMILL_WORKER_LOG", "WARNING")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    if let Some(delay_ms) = delay_once_ms {
        command.env("CLIPMILL_ECHO_DELAY_ONCE_MS", delay_ms.to_string());
    }
    command.spawn().expect("spawn standalone echo worker")
}

async fn wait_for_job_state(
    socket: &Path,
    job_id: &str,
    expected: JobState,
    deadline: Duration,
) -> Job {
    let started = Instant::now();
    let mut attempt = 0_u64;
    while started.elapsed() < deadline {
        let job = get_job(socket, &format!("worker-poll-{attempt}"), job_id)
            .await
            .expect("poll worker job");
        if job.state == expected as i32 {
            return job;
        }
        if matches!(
            job.state(),
            JobState::Succeeded | JobState::Failed | JobState::Cancelled
        ) {
            panic!(
                "job reached unexpected terminal state {:?}: {}",
                job.state(),
                job.failure_detail
            );
        }
        attempt = attempt.saturating_add(1);
        sleep(Duration::from_millis(50)).await;
    }
    panic!("job {job_id} did not reach {expected:?} within {deadline:?}");
}

async fn wait_for_running_task(socket: &Path, job_id: &str) -> Job {
    let started = Instant::now();
    let mut attempt = 0_u64;
    while started.elapsed() < Duration::from_secs(10) {
        let job = get_job(socket, &format!("running-poll-{attempt}"), job_id)
            .await
            .expect("poll running job");
        if job
            .tasks
            .iter()
            .any(|task| task.state == TaskState::Running as i32)
        {
            return job;
        }
        attempt = attempt.saturating_add(1);
        sleep(Duration::from_millis(25)).await;
    }
    panic!("job {job_id} did not acquire a worker lease");
}

async fn wait_for_empty_staging(data_dir: &Path, deadline: Duration) {
    let staging = data_dir.join("artifacts/staging");
    let started = Instant::now();
    while started.elapsed() < deadline {
        let empty = std::fs::read_dir(&staging)
            .expect("staging directory")
            .next()
            .is_none();
        if empty {
            return;
        }
        sleep(Duration::from_millis(25)).await;
    }
    panic!("worker staging was not revoked within {deadline:?}");
}

fn verify_job_outputs(data_dir: &Path, job: &Job) {
    for task in &job.tasks {
        assert_eq!(task.state, TaskState::Succeeded as i32);
        verify_artifact(data_dir, &task.output_artifact_id);
    }
    for artifact_id in &job.output_artifact_ids {
        verify_artifact(data_dir, artifact_id);
    }
}

fn verify_artifact(data_dir: &Path, artifact_id: &str) {
    let digest = artifact_id
        .strip_prefix("sha256:")
        .expect("artifact digest prefix");
    assert_eq!(digest.len(), 64);
    let object = data_dir
        .join("artifacts/objects/sha256")
        .join(&digest[..2])
        .join(digest);
    let manifest_path = object.join("manifest.json");
    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&manifest_path).expect("read committed artifact manifest"),
    )
    .expect("parse committed artifact manifest");
    assert_eq!(manifest["artifact_id"].as_str(), Some(artifact_id));
    let files = manifest["files"].as_array().expect("manifest files");
    assert!(!files.is_empty());
    for file in files {
        let relative = file["path"].as_str().expect("manifest file path");
        let payload = std::fs::read(object.join(relative)).expect("read artifact payload");
        assert_eq!(
            u64::try_from(payload.len()).expect("payload length"),
            file["bytes"].as_u64().expect("manifest byte size")
        );
        assert_eq!(
            format!("sha256:{}", hex::encode(Sha256::digest(&payload))),
            file["sha256"].as_str().expect("manifest file digest")
        );
    }
}

fn assert_private_socket(path: &Path) {
    use std::os::unix::fs::{FileTypeExt, PermissionsExt};

    let metadata = std::fs::symlink_metadata(path).expect("socket metadata");
    assert!(metadata.file_type().is_socket());
    assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
}

#[derive(Debug)]
struct ChildGuard {
    child: Option<Child>,
}

impl ChildGuard {
    fn new(child: Child) -> Self {
        Self { child: Some(child) }
    }

    async fn terminate(&mut self) -> Result<ExitStatus, String> {
        let child = self
            .child
            .as_mut()
            .ok_or_else(|| "child exited".to_owned())?;
        signal_terminate(child)?;
        let status = wait_for_exit(child).await?;
        self.child = None;
        Ok(status)
    }

    async fn kill(&mut self) -> Result<ExitStatus, String> {
        let child = self
            .child
            .as_mut()
            .ok_or_else(|| "child exited".to_owned())?;
        child.kill().map_err(|error| error.to_string())?;
        let status = wait_for_exit(child).await?;
        self.child = None;
        Ok(status)
    }

    #[allow(dead_code)]
    fn id(&self) -> u32 {
        self.child.as_ref().map_or(0, Child::id)
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _result = child.kill();
            let _status = child.wait();
        }
    }
}
