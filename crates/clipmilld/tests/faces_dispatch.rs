//! Exercise real face-task planning and authenticated dispatch without model inference.
//! Only the completed ingest is seeded: its frames and manifest are published through
//! the artifact coordinator. No downloaded weights, decoder, or Python worker is needed.
#![cfg(unix)]
#![allow(clippy::expect_used, clippy::panic)]

mod support;

use std::{
    collections::BTreeMap,
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    time::Duration,
};

use clipmill_artifacts::{
    ArtifactPath, ArtifactRecipe, NetworkPolicy, PrepareOutcome, Producer, RecipeSpec, Timebase,
};
use clipmill_contracts::proto::{
    ipc::v1::{
        DetectFacesPayloadV1, FacesStagePayloadV1, Job, JobState, Request, SubmitJobRequest,
        TaskState, request, response,
    },
    worker::v1::{
        CapabilityDescriptor, FailureClass, RegisterWorker, RegistrationChallenge, TaskLease,
        WorkRequest, WorkerRequest, WorkerResponse, worker_request, worker_response,
    },
};
use clipmill_core::{ArtifactId, JobId, ProjectId, Sha256Digest, SourceId, TaskId, WorkerId};
use clipmilld::{ArtifactCoordinator, Config, Daemon, DaemonError};
use ed25519_dalek::{Signer, SigningKey};
use prost::Message;
use rusqlite::{Connection, params};
use serde_json::{Map, json};
use sha2::{Digest, Sha256};
use support::{create, get_job, send, wait_until_ready, workspace_tempdir};
use tempfile::TempDir;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
    sync::oneshot,
    task::JoinHandle,
    time::timeout,
};

const FRAME: &[u8] = include_bytes!("../../../apps/desktop/src-tauri/icons/32x32.png");
// Bound the observation well below the daemon's 15-second lease expiry. These
// failures must be resolved while this authenticated connection is still open.
const DISPATCH_DEADLINE: Duration = Duration::from_secs(5);

struct Running {
    shutdown: oneshot::Sender<()>,
    task: JoinHandle<Result<(), DaemonError>>,
}
impl Running {
    async fn start(config: &Config) -> (Self, ArtifactCoordinator) {
        let daemon = Daemon::start(config.clone()).await.expect("daemon starts");
        let artifacts = daemon.artifact_coordinator();
        let (shutdown, stopped) = oneshot::channel();
        let task = tokio::spawn(daemon.serve_until(async {
            let _ = stopped.await;
        }));
        wait_until_ready(&config.paths.socket)
            .await
            .expect("daemon ready");
        (Self { shutdown, task }, artifacts)
    }

    async fn stop(self) {
        self.shutdown.send(()).expect("stop signal");
        self.task
            .await
            .expect("daemon joins")
            .expect("daemon stops");
    }
}

struct Fixture {
    _temp: TempDir,
    config: Config,
    project: ProjectId,
    source: SourceId,
    fingerprint: Sha256Digest,
    frames: ArtifactId,
    worker_id: String,
}

impl Fixture {
    async fn new(with_model_registry: bool) -> Self {
        let temp = workspace_tempdir();
        let mut config = Config::from_sources_with_gc(
            Some(temp.path().to_path_buf()),
            None,
            None,
            None,
            None,
            None,
            Some(temp.path().join("pinned-tools/ffprobe")),
            None,
            temp.path().to_path_buf(),
        )
        .expect("isolated dispatch config");
        config.models_dir = if with_model_registry {
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../models/registry")
        } else {
            temp.path().join("empty-model-registry")
        };
        std::fs::create_dir_all(&config.models_dir).expect("registry directory");
        let (daemon, artifacts) = Running::start(&config).await;
        let project = create(&config.paths.socket, "faces-project", "Faces dispatch")
            .await
            .expect("project")
            .project_id
            .parse()
            .expect("project id");
        let fingerprint = Sha256Digest::from_bytes(Sha256::digest(FRAME).into());
        let frames_json = json!({
            "schema_version":"clipmill.media.frames.v1", "source_fingerprint":format!("sha256:{fingerprint}"),
            "frame_rate":{"num":1,"den":1}, "frame_height":32,
            "coverage":{"start_ticks":0,"end_ticks":90_000},
            "frames":[{"file":"frame_00001.png","t_ticks":0}]
        }).to_string();
        let frames = publish(
            &artifacts,
            &project,
            fingerprint,
            "media.frames.v1",
            vec![],
            &[
                ("index.json", frames_json.as_bytes()),
                ("frame_00001.png", FRAME),
            ],
        )
        .await;
        let manifest_json = json!({
            "schema_version":"clipmill.media.ingest_manifest.v1",
            "source_fingerprint":format!("sha256:{fingerprint}"),
            "children":[{"kind":"media.frames.v1","artifact_id":frames.to_string()}]
        })
        .to_string();
        let manifest = publish(
            &artifacts,
            &project,
            fingerprint,
            "media.ingest_manifest.v1",
            vec![frames],
            &[("ingest-manifest.json", manifest_json.as_bytes())],
        )
        .await;
        drop(artifacts);
        daemon.stop().await;
        let source = SourceId::new();
        seed_completed_ingest(&config, &project, &source, fingerprint, manifest);
        // Trust is a startup snapshot, matching the real provisioning flow.
        let worker_id = WorkerId::new().to_string();
        let trust = config
            .paths
            .worker_trust_dir
            .join(format!("{worker_id}.pub"));
        std::fs::write(
            &trust,
            hex::encode(SigningKey::from_bytes(&[7; 32]).verifying_key().as_bytes()),
        )
        .expect("trust fixture key");
        std::fs::set_permissions(&trust, std::fs::Permissions::from_mode(0o600))
            .expect("private trust");
        Self {
            _temp: temp,
            config,
            project,
            source,
            fingerprint,
            frames,
            worker_id,
        }
    }

    async fn submit(&self) -> Job {
        let reply = send(
            &self.config.paths.socket,
            Request {
                request_id: "detect-faces".to_owned(),
                body: Some(request::Body::SubmitJob(SubmitJobRequest {
                    project_id: self.project.to_string(),
                    kind: "detect-faces".to_owned(),
                    payload: DetectFacesPayloadV1 {
                        key_version: "clipmill.detect-faces.v1".to_owned(),
                        source_id: self.source.to_string(),
                        detection: None,
                    }
                    .encode_to_vec(),
                })),
            },
        )
        .await
        .expect("submit response");
        match reply.body {
            Some(response::Body::SubmitJob(reply)) => reply.job.expect("planned faces job"),
            other => panic!("expected planned faces job, got {other:?}"),
        }
    }
}

async fn publish(
    artifacts: &ArtifactCoordinator,
    project: &ProjectId,
    fingerprint: Sha256Digest,
    kind: &str,
    inputs: Vec<ArtifactId>,
    files: &[(&str, &[u8])],
) -> ArtifactId {
    let recipe = ArtifactRecipe::try_from_spec(RecipeSpec {
        kind: kind.to_owned(),
        source_fingerprint: fingerprint,
        timebase: Timebase {
            num: 1,
            den: 90_000,
        },
        producer: Producer {
            stage: "ingest-fixture".to_owned(),
            implementation: "ingest-fixture@1".to_owned(),
            model_digest: None,
        },
        inputs,
        policy: NetworkPolicy::LocalLock,
        config: Map::new(),
        semantic_version: "1.0.0".to_owned(),
    })
    .expect("fixture recipe");
    let PrepareOutcome::Miss(staging) = artifacts.prepare(recipe).await.expect("prepare fixture")
    else {
        panic!("fresh fixture must be a cache miss")
    };
    let paths: Vec<ArtifactPath> = files
        .iter()
        .map(|(name, contents)| {
            let path = name.parse().expect("relative artifact path");
            let mut file = staging.create_file(&path).expect("fixture output");
            file.write_all(contents).expect("write fixture");
            file.sync_all().expect("sync fixture");
            path
        })
        .collect();
    artifacts
        .publish_project(
            project.clone(),
            staging.id().clone(),
            paths,
            BTreeMap::new(),
        )
        .await
        .expect("publish fixture")
        .artifact_id()
}

// Seed only the predecessor boundary, with the daemon stopped. The face job and
// its implementation string are always produced by the actual SubmitJob planner.
fn seed_completed_ingest(
    config: &Config,
    project: &ProjectId,
    source: &SourceId,
    fingerprint: Sha256Digest,
    manifest: ArtifactId,
) {
    let source_path = config.paths.data_dir.join("source.png");
    std::fs::write(&source_path, FRAME).expect("source fixture");
    let mut source_map: serde_json::Value = serde_json::from_str(include_str!(
        "../../../contracts/fixtures/source_map/valid/minimal.json"
    ))
    .expect("valid source map fixture");
    source_map["source_fingerprint"] = format!("sha256:{fingerprint}").into();
    let mut db = Connection::open(&config.paths.database).expect("stopped fixture database");
    let tx = db.transaction().expect("seed transaction");
    tx.execute(
        "INSERT INTO sources(source_id,project_id,source_fingerprint,source_map_json,created_unix_millis)
         VALUES(?1,?2,?3,?4,1)",
        params![source.to_string(), project.to_string(), format!("sha256:{fingerprint}"),
            serde_json::to_vec(&source_map).expect("source map bytes")],
    ).expect("fixture source");
    tx.execute(
        "INSERT INTO source_file_observations(source_id,absolute_path,byte_size,sample_sha256,
             device_id,inode,modified_unix_nanos) VALUES(?1,?2,?3,?4,0,0,0)",
        params![
            source.to_string(),
            source_path.to_string_lossy(),
            i64::try_from(FRAME.len()).expect("fixture byte size fits SQLite"),
            format!("sha256:{fingerprint}")
        ],
    )
    .expect("fixture source observation");
    let job = JobId::new().to_string();
    tx.execute(
        "INSERT INTO jobs(job_id,project_id,kind,payload,state,source_id,
            created_unix_millis,updated_unix_millis) VALUES(?1,?2,'ingest-source',X'',?3,?4,1,1)",
        params![
            job,
            project.to_string(),
            JobState::Succeeded as i32,
            source.to_string()
        ],
    )
    .expect("completed ingest job");
    tx.execute(
        "INSERT INTO tasks(task_id,job_id,ordinal,kind,input_kinds_json,output_kind,payload,
          input_key,state,cpu_threads,ram_bytes,accelerator_class,vram_bytes,disk_bytes,
          network_policy,thermal_class,determinism_class,checkpoint_support,preemption_cost,
          implementation,max_attempts,next_attempt_unix_millis,is_final,output_artifact_id,
          created_unix_millis,updated_unix_millis)
         VALUES(?1,?2,0,'ingest-manifest','[]','media.ingest_manifest.v1',X'',?3,?4,
          1,0,'',0,0,'local-lock','burst','deterministic',0,0,'ingest-fixture@1',1,0,1,?5,1,1)",
        params![
            TaskId::new().to_string(),
            job,
            "00".repeat(32),
            TaskState::Succeeded as i32,
            manifest.to_string()
        ],
    )
    .expect("completed ingest output");
    tx.commit().expect("commit ingest boundary");
}

fn signing_preimage(
    challenge: &RegistrationChallenge,
    descriptor: &CapabilityDescriptor,
) -> Vec<u8> {
    let mut bytes = b"clipmill.worker.registration.v1\0".to_vec();
    bytes.extend_from_slice(&challenge.nonce);
    bytes.push(0);
    for value in [
        &descriptor.worker_id,
        &descriptor.family,
        &descriptor.capabilities.join("\x1f"),
        &descriptor.protocol_version,
        &descriptor.backend,
    ] {
        bytes.extend_from_slice(value.as_bytes());
        bytes.push(0);
    }
    bytes.extend_from_slice(&descriptor.max_memory_bytes.to_be_bytes());
    bytes.extend_from_slice(&descriptor.cpu_threads.to_be_bytes());
    bytes.extend_from_slice(&descriptor.vram_bytes.to_be_bytes());
    bytes.extend_from_slice(&descriptor.public_key);
    bytes
}

async fn worker(fixture: &Fixture) -> UnixStream {
    let key = SigningKey::from_bytes(&[7; 32]);
    let mut stream = UnixStream::connect(&fixture.config.paths.worker_socket)
        .await
        .expect("worker socket");
    let worker_response::Body::Challenge(challenge) = read_worker(&mut stream).await else {
        panic!("registration challenge required")
    };
    let mut descriptor = CapabilityDescriptor {
        worker_id: fixture.worker_id.clone(),
        family: "faces".to_owned(),
        capabilities: vec!["detect-faces".to_owned()],
        protocol_version: "1.2".to_owned(),
        backend: "onnx-cpu".to_owned(),
        max_memory_bytes: 1024 * 1024 * 1024,
        cpu_threads: 2,
        vram_bytes: 0,
        public_key: key.verifying_key().to_bytes().to_vec(),
        signature: vec![],
    };
    descriptor.signature = key
        .sign(&signing_preimage(&challenge, &descriptor))
        .to_bytes()
        .to_vec();
    write_worker(
        &mut stream,
        worker_request::Body::Register(RegisterWorker {
            descriptor: Some(descriptor),
        }),
    )
    .await;
    match read_worker(&mut stream).await {
        worker_response::Body::RegistrationAck(ack) => assert!(ack.accepted, "{}", ack.detail),
        other => panic!("expected accepted registration, got {other:?}"),
    }
    stream
}

async fn write_worker(stream: &mut UnixStream, body: worker_request::Body) {
    stream
        .write_all(&WorkerRequest { body: Some(body) }.encode_length_delimited_to_vec())
        .await
        .expect("write worker frame");
}

async fn read_worker(stream: &mut UnixStream) -> worker_response::Body {
    timeout(DISPATCH_DEADLINE, async {
        let mut length = 0_usize;
        for shift in (0..=28).step_by(7) {
            let byte = stream.read_u8().await.expect("worker length byte");
            length |= usize::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                assert!(length <= 1024 * 1024, "bounded worker frame");
                let mut bytes = vec![0; length];
                stream.read_exact(&mut bytes).await.expect("worker payload");
                return WorkerResponse::decode(bytes.as_slice())
                    .expect("worker frame")
                    .body
                    .expect("worker body");
            }
        }
        panic!("oversized worker frame length")
    })
    .await
    .expect("worker responds before lease expiry")
}

async fn pull(stream: &mut UnixStream) -> worker_response::Body {
    write_worker(
        stream,
        worker_request::Body::WorkRequest(WorkRequest { max_wait_ms: 0 }),
    )
    .await;
    read_worker(stream).await
}

fn assert_bindings(fixture: &Fixture, job: &Job, lease: &TaskLease) {
    assert_eq!(lease.job_id, job.job_id);
    assert_eq!(lease.task_id, job.tasks[0].task_id);
    assert_eq!(lease.kind, "detect-faces");
    assert_eq!(lease.output_kind, "vision.face_track.v1");
    assert_eq!(lease.attempt, 1);
    assert_eq!(lease.input_artifact_ids, vec![fixture.frames.to_string()]);
    let payload = FacesStagePayloadV1::decode(lease.payload.as_slice()).expect("faces payload");
    assert_eq!(payload.key_version, "clipmill.faces-stage.v1");
    assert_eq!(payload.stage, "detect-faces");
    assert_eq!(
        payload.source_fingerprint,
        format!("sha256:{}", fixture.fingerprint)
    );
    assert!(lease.shared_buffer.is_some());
    assert!(Path::new(&lease.staging_dir).is_dir());
    assert_eq!(
        Path::new(&lease.artifact_root),
        fixture.config.paths.artifacts_dir
    );
    assert_eq!(lease.models.len(), 1);
    let model = &lease.models[0];
    assert_eq!(model.name, "yunet-face");
    assert_eq!(model.capability, "detect-faces");
    assert_eq!(
        PathBuf::from(&model.root),
        fixture.config.weights_dir.join("yunet-face")
    );
    assert!(model.digest.starts_with("sha256:"));
    assert_eq!(model.digest.len(), 71);
    assert_eq!(model.files.len(), 1);
    assert_eq!(
        model.files[0].path,
        "models/face_detection_yunet/face_detection_yunet_2023mar.onnx"
    );
    assert_eq!(
        model.files[0].sha256,
        "8f2383e4dd3cfbb4553ea8718107fc0423210dc964f9f4280604804ed2552fa4"
    );
    assert_eq!(model.files[0].bytes, 232_589);
    assert_eq!(lease.tools.len(), 1);
    assert_eq!(lease.tools[0].name, "ffmpeg");
    assert_eq!(
        PathBuf::from(&lease.tools[0].path),
        fixture.config.ffprobe.with_file_name("ffmpeg")
    );
    assert_eq!(lease.tools[0].bom, "ffmpeg-8.1.2-btb-n8.1.2");
}

#[tokio::test]
async fn planned_faces_task_dispatches_pinned_frames_model_and_decoder() {
    let fixture = Fixture::new(true).await;
    let (daemon, artifacts) = Running::start(&fixture.config).await;
    drop(artifacts);
    let job = fixture.submit().await;
    let mut stream = worker(&fixture).await;
    let worker_response::Body::TaskLease(lease) = pull(&mut stream).await else {
        panic!("real planned face task must dispatch with a recognized implementation")
    };
    assert_bindings(&fixture, &job, &lease);
    drop(stream);
    daemon.stop().await;
}

#[tokio::test]
async fn recipe_failure_is_terminal_with_diagnostic_before_lease_expiry() {
    let fixture = Fixture::new(false).await;
    let (daemon, artifacts) = Running::start(&fixture.config).await;
    drop(artifacts);
    let job = fixture.submit().await;
    let mut stream = worker(&fixture).await;
    assert!(matches!(
        pull(&mut stream).await,
        worker_response::Body::NoWork(_)
    ));
    let failed = get_job(&fixture.config.paths.socket, "recipe-failed", &job.job_id)
        .await
        .expect("failure immediately visible");
    assert_eq!(failed.state, JobState::Failed as i32);
    assert_eq!(failed.failure_class, FailureClass::Deterministic as i32);
    assert!(
        failed
            .failure_detail
            .starts_with("Cannot prepare detect-faces:")
    );
    assert!(failed.failure_detail.contains("yunet-face"));
    assert!(failed.failure_detail.contains("registry does not pin"));
    assert_eq!(failed.tasks[0].state, TaskState::Failed as i32);
    assert_eq!(failed.tasks[0].attempt, 1);
    assert_no_active_lease(&fixture.config, &job.job_id);
    drop(stream);
    daemon.stop().await;
}

fn assert_no_active_lease(config: &Config, job_id: &str) {
    let db = Connection::open(&config.paths.database).expect("read task leases");
    let active: i64 = db
        .query_row(
            "SELECT count(*) FROM task_leases l JOIN tasks t ON t.task_id=l.task_id
         WHERE t.job_id=?1 AND l.status=1",
            [job_id],
            |row| row.get(0),
        )
        .expect("lease count");
    assert_eq!(
        active, 0,
        "preparation failure must release its lease immediately"
    );
}

#[tokio::test]
async fn output_preparation_failure_is_retryable_with_diagnostic_before_lease_expiry() {
    let fixture = Fixture::new(true).await;
    let (daemon, artifacts) = Running::start(&fixture.config).await;
    drop(artifacts);
    let job = fixture.submit().await;
    let mut stream = worker(&fixture).await;
    let staging = fixture.config.paths.artifacts_dir.join("staging");
    let saved = fixture.config.paths.artifacts_dir.join("staging-saved");
    std::fs::rename(&staging, &saved).expect("move isolated staging directory");
    std::fs::write(&staging, b"intentionally block staging directory creation")
        .expect("block staging");
    let reply = pull(&mut stream).await;
    std::fs::remove_file(&staging).expect("remove staging blocker");
    std::fs::rename(&saved, &staging).expect("restore isolated staging directory");
    assert!(matches!(reply, worker_response::Body::NoWork(_)));
    let failed = get_job(&fixture.config.paths.socket, "output-failed", &job.job_id)
        .await
        .expect("retry immediately visible");
    assert_eq!(failed.tasks[0].state, TaskState::Retryable as i32);
    assert_eq!(failed.tasks[0].attempt, 1);
    let db = Connection::open(&fixture.config.paths.database).expect("read failure detail");
    let (class, detail): (i32, String) = db
        .query_row(
            "SELECT failure_class,failure_detail FROM tasks WHERE task_id=?1",
            [&failed.tasks[0].task_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("recorded output preparation failure");
    assert_eq!(class, FailureClass::Transient as i32);
    assert!(
        detail.starts_with("Cannot prepare detect-faces output:"),
        "{detail}"
    );
    assert_no_active_lease(&fixture.config, &job.job_id);
    drop(stream);
    daemon.stop().await;
}
