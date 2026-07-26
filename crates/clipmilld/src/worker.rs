use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    os::unix::fs::{FileTypeExt, PermissionsExt},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use clipmill_artifacts::{ArtifactPath, PrepareOutcome};
use clipmill_contracts::proto::worker::v1::{
    CapabilityDescriptor, Complete, CompletionAck, Decline, FailureClass, Heartbeat, HeartbeatAck,
    LeaseAcceptance, NoWork, ProtocolError, RegisterWorker, RegistrationAck, RegistrationChallenge,
    StagedOutput, TaskLease, TaskOutcome, WorkerRequest, WorkerResponse, worker_request,
    worker_response,
};
use clipmill_core::{LeaseId, StagingId, WorkerId};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use prost::Message;
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
    time::{sleep, timeout},
};

use crate::{
    artifacts::ArtifactHandle,
    db::{DbHandle, StoreError},
    jobs::{
        EventHub, HEARTBEAT_INTERVAL, LEASE_TTL, LeaseRequest, LeasedTask, ResourceCapacity,
        SchedulerHandle,
    },
    models::ModelRegistry,
    recipes,
    shm::ShmBroker,
};

const MAX_FRAME_BYTES: usize = 4 * 1024 * 1024;
const READ_TIMEOUT: Duration = Duration::from_secs(30);
const WRITE_TIMEOUT: Duration = Duration::from_secs(10);
const CURRENT_PROTOCOL: &str = "1.2";
const PREVIOUS_PROTOCOL: &str = "1.1";

/// Runtimes a worker may declare, and the accelerator class each one needs.
///
/// The names are runtimes rather than chips, because that is what a worker
/// actually knows about itself. Mapping to an accelerator class is the
/// daemon's job, and doing it in one place is what stops "mlx" and "metal"
/// from drifting into two different meanings.
const BACKENDS: [(&str, BackendRequirement); 6] = [
    ("cpu", BackendRequirement::CpuOnly),
    ("onnx-cpu", BackendRequirement::CpuOnly),
    ("mlx", BackendRequirement::Accelerator("metal")),
    ("coreml", BackendRequirement::Accelerator("metal")),
    // Protocol 1.1 workers named the chip, not the runtime.
    ("metal", BackendRequirement::Accelerator("metal")),
    ("cuda", BackendRequirement::Accelerator("cuda")),
];

/// What a backend needs the machine to have.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BackendRequirement {
    CpuOnly,
    /// The accelerator class the verified device profile must report.
    Accelerator(&'static str),
}

fn backend_requirement(backend: &str) -> Option<BackendRequirement> {
    BACKENDS
        .iter()
        .find(|(name, _)| *name == backend)
        .map(|(_, requirement)| *requirement)
}
const NO_WORK_MAX_WAIT: Duration = Duration::from_secs(1);
const NO_WORK_RETRY: Duration = Duration::from_millis(250);
const SIGNING_DOMAIN: &[u8] = b"clipmill.worker.registration.v1\0";

#[derive(Clone, Debug)]
pub(crate) struct WorkerService {
    database: DbHandle,
    artifacts: ArtifactHandle,
    events: EventHub,
    scheduler: SchedulerHandle,
    daemon_epoch: String,
    trust: Arc<BTreeMap<String, [u8; 32]>>,
    active_workers: Arc<Mutex<BTreeSet<String>>>,
    shm: ShmBroker,
    models: Arc<ModelRegistry>,
    artifact_root: Arc<PathBuf>,
    /// Where the pinned weights were installed. Handed to workers on the
    /// lease; never part of any artifact key.
    weights_root: Arc<PathBuf>,
    accepting_work: Arc<AtomicBool>,
    drop_completion_ack_once: Arc<AtomicBool>,
}

impl WorkerService {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        database: DbHandle,
        artifacts: ArtifactHandle,
        events: EventHub,
        scheduler: SchedulerHandle,
        daemon_epoch: String,
        trust_dir: &Path,
        shm: ShmBroker,
        models: Arc<ModelRegistry>,
        artifact_root: PathBuf,
        weights_root: PathBuf,
    ) -> Result<Self, WorkerError> {
        Ok(Self {
            database,
            artifacts,
            events,
            scheduler,
            daemon_epoch,
            trust: Arc::new(load_trust(trust_dir)?),
            active_workers: Arc::new(Mutex::new(BTreeSet::new())),
            shm,
            models,
            artifact_root: Arc::new(artifact_root),
            weights_root: Arc::new(weights_root),
            accepting_work: Arc::new(AtomicBool::new(true)),
            drop_completion_ack_once: Arc::new(AtomicBool::new(
                std::env::var_os("CLIPMILL_TEST_DROP_COMPLETION_ACK_ONCE")
                    .is_some_and(|value| value == "1"),
            )),
        })
    }

    pub(crate) fn stop_scheduling(&self) {
        self.accepting_work.store(false, Ordering::Release);
    }

    pub(crate) fn shm_broker(&self) -> ShmBroker {
        self.shm.clone()
    }

    pub(crate) async fn handle_connection(
        &self,
        mut stream: UnixStream,
    ) -> Result<(), WorkerError> {
        let challenge = challenge()?;
        write_response(
            &mut stream,
            &WorkerResponse {
                body: Some(worker_response::Body::Challenge(challenge.clone())),
            },
        )
        .await?;
        let request = read_request(&mut stream).await?;
        let Some(worker_request::Body::Register(RegisterWorker {
            descriptor: Some(descriptor),
        })) = request.body
        else {
            return Err(WorkerError::Protocol("registration must follow challenge"));
        };
        validate_registration(&descriptor, &challenge, &self.trust)?;
        let worker_id = descriptor.worker_id.clone();
        let _active = ActiveWorker::acquire(Arc::clone(&self.active_workers), worker_id.clone())?;
        let session_id = ulid::Ulid::new().to_string();
        write_response(
            &mut stream,
            &WorkerResponse {
                body: Some(worker_response::Body::RegistrationAck(RegistrationAck {
                    accepted: true,
                    session_id,
                    detail: String::new(),
                })),
            },
        )
        .await?;
        tracing::info!(worker_id, family = descriptor.family, "worker registered");
        self.run_registered(&mut stream, &descriptor).await
    }

    async fn run_registered(
        &self,
        stream: &mut UnixStream,
        descriptor: &CapabilityDescriptor,
    ) -> Result<(), WorkerError> {
        let mut active: Option<ActiveLease> = None;
        let result = loop {
            let request = match read_request(stream).await {
                Ok(request) => request,
                Err(error) => break Err(error),
            };
            let Some(body) = request.body else {
                break Err(WorkerError::Protocol("worker request body is required"));
            };
            let response = match body {
                worker_request::Body::Register(_) => {
                    Err(WorkerError::Protocol("worker is already registered"))
                }
                worker_request::Body::WorkRequest(request) => {
                    if active.is_some() {
                        Err(WorkerError::Protocol(
                            "worker requested another task while a lease is active",
                        ))
                    } else {
                        self.pull_task(descriptor, request.max_wait_ms, &mut active)
                            .await
                    }
                }
                worker_request::Body::LeaseAcceptance(acceptance) => {
                    self.accept_lease(acceptance, &mut active).await
                }
                worker_request::Body::Heartbeat(heartbeat) => {
                    self.heartbeat(heartbeat, active.as_ref()).await
                }
                worker_request::Body::Complete(completion) => {
                    self.complete(completion, &mut active, &descriptor.worker_id)
                        .await
                }
                worker_request::Body::Decline(decline) => self.decline(decline, &mut active).await,
            };
            match response {
                Ok(response) => {
                    if matches!(
                        &response.body,
                        Some(worker_response::Body::CompletionAck(_))
                    ) && self.drop_completion_ack_once.swap(false, Ordering::AcqRel)
                    {
                        tracing::warn!(
                            operation = "completion-ack",
                            "injecting response loss after durable worker completion"
                        );
                        break Err(WorkerError::InjectedResponseLoss);
                    }
                    if let Err(error) = write_response(stream, &response).await {
                        break Err(error);
                    }
                }
                Err(error) => {
                    let response = protocol_error(&error);
                    let _write = write_response(stream, &response).await;
                    break Err(error);
                }
            }
        };
        if let Some(lease) = active.take() {
            self.cleanup_lease(&lease).await;
        }
        result
    }

    fn admitted_capacity(
        &self,
        descriptor: &CapabilityDescriptor,
    ) -> Result<ResourceCapacity, &'static str> {
        admit_capacity(descriptor, self.scheduler.machine_capacity())
    }

    #[allow(clippy::too_many_lines)]
    async fn pull_task(
        &self,
        descriptor: &CapabilityDescriptor,
        requested_wait_ms: u64,
        active: &mut Option<ActiveLease>,
    ) -> Result<WorkerResponse, WorkerError> {
        if !self.accepting_work.load(Ordering::Acquire) {
            return Ok(WorkerResponse {
                body: Some(worker_response::Body::NoWork(NoWork {
                    retry_after_ms: duration_millis(NO_WORK_RETRY),
                })),
            });
        }
        let wait = Duration::from_millis(requested_wait_ms).min(NO_WORK_MAX_WAIT);
        if !wait.is_zero() {
            sleep(wait).await;
        }
        let capacity = match self.admitted_capacity(descriptor) {
            Ok(capacity) => capacity,
            Err(detail) => {
                tracing::info!(
                    worker_id = descriptor.worker_id,
                    backend = descriptor.backend,
                    detail,
                    "worker declined: declared resources exceed the verified device"
                );
                return Ok(WorkerResponse {
                    body: Some(worker_response::Body::NoWork(NoWork {
                        retry_after_ms: duration_millis(NO_WORK_RETRY),
                    })),
                });
            }
        };
        for _cache_attempt in 0..8 {
            let now = now_millis();
            let lease_id = LeaseId::new().to_string();
            let selection = self
                .database
                .lease_next_task(LeaseRequest {
                    lease_id,
                    daemon_epoch: self.daemon_epoch.clone(),
                    now_unix_millis: now,
                    expires_unix_millis: now.saturating_add(duration_millis(LEASE_TTL)),
                    capacity,
                    worker_id: descriptor.worker_id.clone(),
                    capabilities: descriptor.capabilities.clone(),
                })
                .await?;
            self.events.publish_all(selection.events);
            let Some(task) = selection.task else {
                return Ok(WorkerResponse {
                    body: Some(worker_response::Body::NoWork(NoWork {
                        retry_after_ms: duration_millis(NO_WORK_RETRY),
                    })),
                });
            };
            let recipe = recipes::worker_recipe(&task, &self.models)
                .map_err(|error| WorkerError::Artifact(error.to_string()))?;
            match self.artifacts.prepare(recipe).await? {
                PrepareOutcome::Hit(artifact) => {
                    self.complete_cache_hit(&task, artifact.artifact_id())
                        .await?;
                    self.scheduler.notify();
                }
                PrepareOutcome::InFlight { .. } => {
                    let events = self
                        .database
                        .fail_task(
                            task.lease_id.clone(),
                            FailureClass::Transient as i32,
                            "artifact key is already in flight".to_owned(),
                            now_millis(),
                        )
                        .await?;
                    self.events.publish_all(events);
                    self.scheduler.notify();
                }
                PrepareOutcome::Miss(staging) => {
                    let shared_buffer = match self.shm.create(&task.lease_id, &task.payload) {
                        Ok(descriptor) => descriptor,
                        Err(error) => {
                            let _abandoned = self.artifacts.abandon(staging.id().clone()).await;
                            let events = self
                                .database
                                .fail_task(
                                    task.lease_id.clone(),
                                    FailureClass::Transient as i32,
                                    error.to_string(),
                                    now_millis(),
                                )
                                .await?;
                            self.events.publish_all(events);
                            return Err(error.into());
                        }
                    };
                    let lease = TaskLease {
                        task_id: task.task_id.clone(),
                        lease_id: task.lease_id.clone(),
                        kind: task.kind.clone(),
                        payload: task.payload.clone(),
                        heartbeat_interval_ms: duration_millis(HEARTBEAT_INTERVAL),
                        lease_ttl_ms: duration_millis(LEASE_TTL),
                        staging_id: staging.id().to_string(),
                        staging_dir: staging.path().to_string_lossy().into_owned(),
                        input_artifact_ids: task
                            .input_artifact_ids
                            .iter()
                            .map(ToString::to_string)
                            .collect(),
                        shared_buffer: Some(shared_buffer),
                        output_kind: task.output_kind.clone(),
                        attempt: task.attempt,
                        artifact_root: self.artifact_root.to_string_lossy().into_owned(),
                        // Only what this stage was registered to run. A worker
                        // handed every pinned model would be free to choose,
                        // and the artifact key would still name the one the
                        // registry expected.
                        models: recipes::lookup(&task.kind)
                            .and_then(|recipe| recipe.model)
                            .and_then(|name| self.models.binding(name, &self.weights_root))
                            .into_iter()
                            .collect(),
                    };
                    *active = Some(ActiveLease {
                        task,
                        staging_id: staging.id().clone(),
                        staging_dir: staging.path().to_path_buf(),
                        accepted: false,
                    });
                    return Ok(WorkerResponse {
                        body: Some(worker_response::Body::TaskLease(lease)),
                    });
                }
            }
        }
        Ok(WorkerResponse {
            body: Some(worker_response::Body::NoWork(NoWork {
                retry_after_ms: duration_millis(NO_WORK_RETRY),
            })),
        })
    }

    async fn complete_cache_hit(
        &self,
        task: &LeasedTask,
        artifact_id: clipmill_core::ArtifactId,
    ) -> Result<(), WorkerError> {
        let ack = WorkerResponse {
            body: Some(worker_response::Body::CompletionAck(CompletionAck {
                lease_id: task.lease_id.clone(),
                accepted: true,
                artifact_ids: vec![artifact_id.to_string()],
                detail: "cache-hit".to_owned(),
            })),
        };
        let bytes = ack.encode_to_vec();
        let hash: [u8; 32] = Sha256::digest(&bytes).into();
        let completion = self
            .database
            .complete_task(
                task.lease_id.clone(),
                artifact_id,
                hash,
                bytes,
                now_millis(),
            )
            .await?;
        self.events.publish_all(completion.events);
        Ok(())
    }

    async fn accept_lease(
        &self,
        acceptance: LeaseAcceptance,
        active: &mut Option<ActiveLease>,
    ) -> Result<WorkerResponse, WorkerError> {
        let lease = active.as_mut().ok_or(WorkerError::StaleLease)?;
        if acceptance.lease_id != lease.task.lease_id {
            return Err(WorkerError::StaleLease);
        }
        if !acceptance.accepted {
            let detail = bounded_detail(&acceptance.detail);
            let events = self
                .database
                .fail_task(
                    lease.task.lease_id.clone(),
                    FailureClass::Transient as i32,
                    format!("worker declined lease: {detail}"),
                    now_millis(),
                )
                .await?;
            self.events.publish_all(events);
            self.cleanup_lease(lease).await;
            *active = None;
            self.scheduler.notify();
            return Ok(WorkerResponse {
                body: Some(worker_response::Body::NoWork(NoWork {
                    retry_after_ms: duration_millis(NO_WORK_RETRY),
                })),
            });
        }
        lease.accepted = true;
        Ok(WorkerResponse {
            body: Some(worker_response::Body::HeartbeatAck(HeartbeatAck {
                lease_id: lease.task.lease_id.clone(),
                accepted: true,
                cancelled: false,
                expires_unix_millis: now_millis().saturating_add(duration_millis(LEASE_TTL)),
            })),
        })
    }

    async fn heartbeat(
        &self,
        heartbeat: Heartbeat,
        active: Option<&ActiveLease>,
    ) -> Result<WorkerResponse, WorkerError> {
        let lease = active.ok_or(WorkerError::StaleLease)?;
        if !lease.accepted || heartbeat.lease_id != lease.task.lease_id {
            return Err(WorkerError::StaleLease);
        }
        let now = now_millis();
        match self
            .database
            .heartbeat_task(
                heartbeat.lease_id.clone(),
                now,
                now.saturating_add(duration_millis(LEASE_TTL)),
                heartbeat.progress,
            )
            .await
        {
            Ok(events) => {
                self.events.publish_all(events);
                Ok(WorkerResponse {
                    body: Some(worker_response::Body::HeartbeatAck(HeartbeatAck {
                        lease_id: heartbeat.lease_id,
                        accepted: true,
                        cancelled: false,
                        expires_unix_millis: now.saturating_add(duration_millis(LEASE_TTL)),
                    })),
                })
            }
            Err(StoreError::Conflict | StoreError::NotFound) => Ok(WorkerResponse {
                body: Some(worker_response::Body::Cancel(
                    clipmill_contracts::proto::worker::v1::CancelLease {
                        lease_id: heartbeat.lease_id,
                        reason: "lease cancelled or expired".to_owned(),
                    },
                )),
            }),
            Err(error) => Err(error.into()),
        }
    }

    async fn complete(
        &self,
        completion: Complete,
        active: &mut Option<ActiveLease>,
        worker_id: &str,
    ) -> Result<WorkerResponse, WorkerError> {
        completion
            .lease_id
            .parse::<LeaseId>()
            .map_err(|_| WorkerError::StaleLease)?;
        let completion_hash: [u8; 32] = Sha256::digest(completion.encode_to_vec()).into();
        if let Some(response) = self
            .database
            .replay_task_completion(
                completion.lease_id.clone(),
                worker_id.to_owned(),
                completion_hash,
            )
            .await?
        {
            return WorkerResponse::decode(response.as_slice())
                .map_err(|error| WorkerError::Decode(error.to_string()));
        }
        let lease = active.as_ref().ok_or(WorkerError::StaleLease)?;
        if !lease.accepted || completion.lease_id != lease.task.lease_id {
            return Err(WorkerError::StaleLease);
        }
        let outcome = completion.outcome();
        if outcome != TaskOutcome::Succeeded {
            let failure = match completion.failure_class() {
                FailureClass::Unspecified => FailureClass::Transient,
                value => value,
            };
            let ack = WorkerResponse {
                body: Some(worker_response::Body::CompletionAck(CompletionAck {
                    lease_id: completion.lease_id.clone(),
                    accepted: true,
                    artifact_ids: Vec::new(),
                    detail: "failure recorded".to_owned(),
                })),
            };
            let encoded_ack = ack.encode_to_vec();
            let durable = self
                .database
                .complete_failed_task(
                    completion.lease_id.clone(),
                    failure as i32,
                    bounded_detail(&completion.detail),
                    completion_hash,
                    encoded_ack,
                    now_millis(),
                )
                .await?;
            self.events.publish_all(durable.events);
            let lease = active.take().ok_or(WorkerError::StaleLease)?;
            self.cleanup_lease(&lease).await;
            self.scheduler.notify();
            return WorkerResponse::decode(durable.response.as_slice())
                .map_err(|error| WorkerError::Decode(error.to_string()));
        }
        if !completion.artifact_ids.is_empty() {
            return Err(WorkerError::Protocol(
                "workers cannot assign artifact identifiers",
            ));
        }
        validate_staged_outputs(lease, &completion.staged_outputs)?;
        let paths = completion
            .staged_outputs
            .iter()
            .map(|output| output.relative_path.parse::<ArtifactPath>())
            .collect::<Result<Vec<_>, _>>()?;
        let artifact = self
            .artifacts
            .commit(lease.staging_id.clone(), paths, BTreeMap::new())
            .await?;
        let ack = WorkerResponse {
            body: Some(worker_response::Body::CompletionAck(CompletionAck {
                lease_id: completion.lease_id.clone(),
                accepted: true,
                artifact_ids: vec![artifact.artifact_id().to_string()],
                detail: String::new(),
            })),
        };
        let encoded_ack = ack.encode_to_vec();
        let durable = self
            .database
            .complete_task(
                completion.lease_id,
                artifact.artifact_id(),
                completion_hash,
                encoded_ack,
                now_millis(),
            )
            .await?;
        self.events.publish_all(durable.events);
        let completed = active.take().ok_or(WorkerError::StaleLease)?;
        self.shm.revoke_lease(&completed.task.lease_id);
        self.scheduler.notify();
        WorkerResponse::decode(durable.response.as_slice())
            .map_err(|error| WorkerError::Decode(error.to_string()))
    }

    async fn decline(
        &self,
        decline: Decline,
        active: &mut Option<ActiveLease>,
    ) -> Result<WorkerResponse, WorkerError> {
        let lease = active.as_ref().ok_or(WorkerError::StaleLease)?;
        if decline.task_id != lease.task.task_id {
            return Err(WorkerError::StaleLease);
        }
        let events = self
            .database
            .fail_task(
                lease.task.lease_id.clone(),
                FailureClass::Transient as i32,
                format!("worker declined task: {}", bounded_detail(&decline.detail)),
                now_millis(),
            )
            .await?;
        self.events.publish_all(events);
        let lease = active.take().ok_or(WorkerError::StaleLease)?;
        self.cleanup_lease(&lease).await;
        self.scheduler.notify();
        Ok(WorkerResponse {
            body: Some(worker_response::Body::NoWork(NoWork {
                retry_after_ms: duration_millis(NO_WORK_RETRY),
            })),
        })
    }

    async fn cleanup_lease(&self, lease: &ActiveLease) {
        self.shm.revoke_lease(&lease.task.lease_id);
        if let Err(error) = self.artifacts.abandon(lease.staging_id.clone()).await {
            tracing::warn!(lease_id = lease.task.lease_id, %error, "cannot abandon worker staging");
        }
    }
}

#[derive(Debug)]
struct ActiveLease {
    task: LeasedTask,
    staging_id: StagingId,
    staging_dir: PathBuf,
    accepted: bool,
}

#[derive(Debug)]
struct ActiveWorker {
    workers: Arc<Mutex<BTreeSet<String>>>,
    worker_id: String,
}

impl ActiveWorker {
    fn acquire(
        workers: Arc<Mutex<BTreeSet<String>>>,
        worker_id: String,
    ) -> Result<Self, WorkerError> {
        let mut active = workers.lock().map_err(|_| WorkerError::Stopped)?;
        if !active.insert(worker_id.clone()) {
            return Err(WorkerError::DuplicateWorker);
        }
        drop(active);
        Ok(Self { workers, worker_id })
    }
}

impl Drop for ActiveWorker {
    fn drop(&mut self) {
        if let Ok(mut active) = self.workers.lock() {
            active.remove(&self.worker_id);
        }
    }
}

fn load_trust(path: &Path) -> Result<BTreeMap<String, [u8; 32]>, WorkerError> {
    let mut keys = BTreeMap::new();
    let mut public_keys = BTreeSet::new();
    for entry in fs::read_dir(path).map_err(WorkerError::Io)? {
        let entry = entry.map_err(WorkerError::Io)?;
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        let Some(worker_id) = file_name.strip_suffix(".pub") else {
            continue;
        };
        let worker_id = worker_id
            .parse::<WorkerId>()
            .map_err(|_| WorkerError::InvalidTrustKey)?
            .to_string();
        let metadata = fs::symlink_metadata(entry.path()).map_err(WorkerError::Io)?;
        if metadata.file_type().is_symlink()
            || !metadata.file_type().is_file()
            || metadata.len() > 256
            || metadata.permissions().mode() & 0o077 != 0
        {
            return Err(WorkerError::InvalidTrustKey);
        }
        let text = fs::read_to_string(entry.path()).map_err(WorkerError::Io)?;
        let bytes = hex::decode(text.trim()).map_err(|_| WorkerError::InvalidTrustKey)?;
        let key: [u8; 32] = bytes.try_into().map_err(|_| WorkerError::InvalidTrustKey)?;
        if !public_keys.insert(key) || keys.insert(worker_id, key).is_some() {
            return Err(WorkerError::InvalidTrustKey);
        }
    }
    Ok(keys)
}

fn challenge() -> Result<RegistrationChallenge, WorkerError> {
    let mut nonce = [0_u8; 32];
    getrandom::fill(&mut nonce).map_err(|error| WorkerError::Random(error.to_string()))?;
    Ok(RegistrationChallenge {
        nonce: nonce.to_vec(),
        supported_protocol_versions: vec![
            CURRENT_PROTOCOL.to_owned(),
            PREVIOUS_PROTOCOL.to_owned(),
        ],
        issued_unix_millis: now_millis(),
    })
}

fn validate_registration(
    descriptor: &CapabilityDescriptor,
    challenge: &RegistrationChallenge,
    trust: &BTreeMap<String, [u8; 32]>,
) -> Result<(), WorkerError> {
    descriptor
        .worker_id
        .parse::<WorkerId>()
        .map_err(|_| WorkerError::InvalidDescriptor)?;
    if !matches!(
        descriptor.protocol_version.as_str(),
        CURRENT_PROTOCOL | PREVIOUS_PROTOCOL
    ) || !valid_word(&descriptor.family)
        || backend_requirement(&descriptor.backend).is_none()
        || descriptor.max_memory_bytes == 0
        || descriptor.max_memory_bytes > i64::MAX as u64
        || descriptor.vram_bytes > i64::MAX as u64
        || descriptor.capabilities.is_empty()
        || descriptor.capabilities.len() > 64
    {
        return Err(WorkerError::InvalidDescriptor);
    }
    // A 1.1 worker declares no thread count and is taken at its word as one.
    // A 1.2 worker that declares an absurd one is refused rather than left to
    // starve every other stage on the machine.
    if descriptor.protocol_version == CURRENT_PROTOCOL
        && (descriptor.cpu_threads == 0 || descriptor.cpu_threads > 1024)
    {
        return Err(WorkerError::InvalidDescriptor);
    }
    if descriptor.protocol_version == PREVIOUS_PROTOCOL
        && (descriptor.cpu_threads != 0 || descriptor.vram_bytes != 0)
    {
        // Those fields are not covered by a 1.1 signature, so honouring them
        // would let an attacker widen a worker's budget without resigning.
        return Err(WorkerError::InvalidDescriptor);
    }
    let mut previous = None;
    for capability in &descriptor.capabilities {
        if !valid_word(capability) || previous.is_some_and(|value: &String| value >= capability) {
            return Err(WorkerError::InvalidDescriptor);
        }
        previous = Some(capability);
    }
    let public: [u8; 32] = descriptor
        .public_key
        .as_slice()
        .try_into()
        .map_err(|_| WorkerError::InvalidDescriptor)?;
    if trust.get(&descriptor.worker_id) != Some(&public) {
        return Err(WorkerError::UntrustedWorker);
    }
    let signature =
        Signature::from_slice(&descriptor.signature).map_err(|_| WorkerError::InvalidSignature)?;
    let key = VerifyingKey::from_bytes(&public).map_err(|_| WorkerError::InvalidSignature)?;
    key.verify(&registration_preimage(challenge, descriptor), &signature)
        .map_err(|_| WorkerError::InvalidSignature)
}

pub(crate) fn registration_preimage(
    challenge: &RegistrationChallenge,
    descriptor: &CapabilityDescriptor,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(SIGNING_DOMAIN);
    bytes.extend_from_slice(&challenge.nonce);
    bytes.push(0);
    for field in [
        descriptor.worker_id.as_bytes(),
        descriptor.family.as_bytes(),
    ] {
        bytes.extend_from_slice(field);
        bytes.push(0);
    }
    bytes.extend_from_slice(descriptor.capabilities.join("\x1f").as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(descriptor.protocol_version.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(descriptor.backend.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(&descriptor.max_memory_bytes.to_be_bytes());
    // Protocol 1.2 added declared resources. They join the preimage so a
    // worker cannot widen its own budget after signing; 1.1 signatures keep
    // their original shape, which is what lets both versions be served.
    if descriptor.protocol_version == CURRENT_PROTOCOL {
        bytes.extend_from_slice(&descriptor.cpu_threads.to_be_bytes());
        bytes.extend_from_slice(&descriptor.vram_bytes.to_be_bytes());
    }
    bytes.extend_from_slice(&descriptor.public_key);
    bytes
}

/// What a worker may actually reserve.
///
/// The declaration is a request, not a fact: it is clamped by what the
/// verified device profile measured, so a worker cannot claim hardware this
/// machine never had. On unified memory the same bytes back both system and
/// video allocations, so counting VRAM separately would reserve them twice
/// and starve the machine at half load.
fn admit_capacity(
    descriptor: &CapabilityDescriptor,
    machine: ResourceCapacity,
) -> Result<ResourceCapacity, &'static str> {
    let requirement =
        backend_requirement(&descriptor.backend).ok_or("backend is not one this daemon serves")?;
    let accelerator_mask = match requirement {
        BackendRequirement::CpuOnly => 0,
        BackendRequirement::Accelerator(class) => {
            let bit = crate::jobs::accelerator_bit(class)
                .ok_or("backend maps to no accelerator class")?;
            if machine.accelerator_mask & bit == 0 {
                return Err("the verified device profile measured no such accelerator");
            }
            bit
        }
    };
    let cpu_threads = descriptor.cpu_threads.max(1);
    if cpu_threads > machine.cpu_threads {
        return Err("declared threads exceed the machine");
    }
    if descriptor.max_memory_bytes > machine.ram_bytes {
        return Err("declared memory exceeds the machine");
    }
    // Unified memory: one budget, and it was already counted as RAM.
    let vram_bytes = if machine.vram_bytes == 0 {
        0
    } else if descriptor.vram_bytes > machine.vram_bytes {
        return Err("declared video memory exceeds the machine");
    } else {
        descriptor.vram_bytes
    };
    Ok(ResourceCapacity {
        cpu_threads,
        ram_bytes: descriptor.max_memory_bytes,
        disk_bytes: machine.disk_bytes,
        accelerator_mask,
        vram_bytes,
    })
}

fn valid_word(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || (index > 0 && matches!(byte, b'-' | b'_'))
        })
}

fn validate_staged_outputs(
    lease: &ActiveLease,
    outputs: &[StagedOutput],
) -> Result<(), WorkerError> {
    if outputs.is_empty() || outputs.len() > 64 {
        return Err(WorkerError::InvalidStaging);
    }
    let mut seen = BTreeSet::new();
    for output in outputs {
        let path = output
            .relative_path
            .parse::<ArtifactPath>()
            .map_err(|_| WorkerError::InvalidStaging)?;
        if !seen.insert(path.clone())
            || output.sha256.len() != 64
            || output
                .sha256
                .bytes()
                .any(|byte| !byte.is_ascii_digit() && !(b'a'..=b'f').contains(&byte))
        {
            return Err(WorkerError::InvalidStaging);
        }
        let disk_path = lease.staging_dir.join(path.as_path());
        let metadata = fs::symlink_metadata(&disk_path).map_err(WorkerError::Io)?;
        if metadata.file_type().is_symlink()
            || !metadata.file_type().is_file()
            || metadata.file_type().is_socket()
            || metadata.len() != output.byte_size
        {
            return Err(WorkerError::InvalidStaging);
        }
        let mut file = fs::File::open(disk_path).map_err(WorkerError::Io)?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0_u8; 64 * 1024];
        loop {
            let read = file.read(&mut buffer).map_err(WorkerError::Io)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        if hex::encode(hasher.finalize()) != output.sha256 {
            return Err(WorkerError::InvalidStaging);
        }
    }
    Ok(())
}

async fn read_request(stream: &mut UnixStream) -> Result<WorkerRequest, WorkerError> {
    let length = timeout(READ_TIMEOUT, read_varint(stream))
        .await
        .map_err(|_| WorkerError::Timeout)??;
    let length = usize::try_from(length).map_err(|_| WorkerError::OversizedFrame)?;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err(WorkerError::OversizedFrame);
    }
    let mut bytes = vec![0_u8; length];
    timeout(READ_TIMEOUT, stream.read_exact(&mut bytes))
        .await
        .map_err(|_| WorkerError::Timeout)??;
    WorkerRequest::decode(bytes.as_slice()).map_err(|error| WorkerError::Decode(error.to_string()))
}

async fn write_response(
    stream: &mut UnixStream,
    response: &WorkerResponse,
) -> Result<(), WorkerError> {
    let bytes = response.encode_length_delimited_to_vec();
    if bytes.len() > MAX_FRAME_BYTES {
        return Err(WorkerError::OversizedFrame);
    }
    timeout(WRITE_TIMEOUT, stream.write_all(&bytes))
        .await
        .map_err(|_| WorkerError::Timeout)??;
    Ok(())
}

async fn read_varint(stream: &mut UnixStream) -> Result<u64, WorkerError> {
    let mut value = 0_u64;
    for index in 0..10 {
        let byte = stream.read_u8().await?;
        if index == 9 && byte > 1 {
            return Err(WorkerError::MalformedFrame);
        }
        value |= u64::from(byte & 0x7f) << (index * 7);
        if byte & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err(WorkerError::MalformedFrame)
}

fn protocol_error(error: &WorkerError) -> WorkerResponse {
    let code = error.code();
    let detail = if matches!(
        code,
        "DUPLICATE_WORKER"
            | "INVALID_ARGUMENT"
            | "UNAUTHENTICATED"
            | "STAGED_OUTPUT_MISMATCH"
            | "FRAME_TOO_LARGE"
            | "STALE_LEASE"
    ) {
        error.to_string()
    } else {
        "internal worker service failure".to_owned()
    };
    WorkerResponse {
        body: Some(worker_response::Body::Error(ProtocolError {
            code: code.to_owned(),
            detail,
        })),
    }
}

fn bounded_detail(detail: &str) -> String {
    detail
        .chars()
        .filter(|character| !character.is_control())
        .take(512)
        .collect()
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
        })
}

fn duration_millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

#[derive(Debug, Error)]
pub(crate) enum WorkerError {
    #[error("artifact operation failed: {0}")]
    Artifact(String),
    #[error("artifact path is invalid: {0}")]
    ArtifactPath(#[from] clipmill_artifacts::ArtifactPathError),
    #[error("worker protocol decode failed: {0}")]
    Decode(String),
    #[error("a worker with this ID is already active")]
    DuplicateWorker,
    #[error("worker descriptor is invalid")]
    InvalidDescriptor,
    #[error("worker signature is invalid")]
    InvalidSignature,
    #[error("test-injected worker completion response loss")]
    InjectedResponseLoss,
    #[error("worker staged output does not match its declaration")]
    InvalidStaging,
    #[error("worker trust entry is invalid")]
    InvalidTrustKey,
    #[error("worker I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("worker frame is malformed")]
    MalformedFrame,
    #[error("worker frame exceeds the 4 MiB limit")]
    OversizedFrame,
    #[error("worker protocol error: {0}")]
    Protocol(&'static str),
    #[error("cannot generate worker challenge: {0}")]
    Random(String),
    #[error("task lease is stale, cancelled, or belongs to another connection")]
    StaleLease,
    #[error("worker registry stopped")]
    Stopped,
    #[error("worker request timed out")]
    Timeout,
    #[error("worker public key is not trusted")]
    UntrustedWorker,
    #[error(transparent)]
    ArtifactService(#[from] crate::artifacts::ArtifactServiceError),
    #[error(transparent)]
    Database(#[from] StoreError),
    #[error(transparent)]
    SharedMemory(#[from] crate::shm::ShmError),
}

impl WorkerError {
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::DuplicateWorker => "DUPLICATE_WORKER",
            Self::InvalidDescriptor | Self::Protocol(_) => "INVALID_ARGUMENT",
            Self::InvalidSignature | Self::UntrustedWorker => "UNAUTHENTICATED",
            Self::InvalidStaging => "STAGED_OUTPUT_MISMATCH",
            Self::OversizedFrame => "FRAME_TOO_LARGE",
            Self::StaleLease => "STALE_LEASE",
            _ => "INTERNAL",
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use std::{
        collections::{BTreeMap, BTreeSet},
        sync::{Arc, Mutex},
    };

    use clipmill_contracts::proto::worker::v1::{CapabilityDescriptor, RegistrationChallenge};
    use ed25519_dalek::{Signer, SigningKey};
    use prost::Message;
    use tokio::io::AsyncWriteExt;

    use super::{
        ActiveWorker, CURRENT_PROTOCOL, MAX_FRAME_BYTES, PREVIOUS_PROTOCOL, ResourceCapacity,
        WorkerError, admit_capacity, read_request, registration_preimage, valid_word,
        validate_registration,
    };

    fn signed_descriptor(
        protocol_version: &str,
    ) -> (CapabilityDescriptor, RegistrationChallenge, [u8; 32]) {
        let signing = SigningKey::from_bytes(&[7; 32]);
        let challenge = RegistrationChallenge {
            nonce: vec![8; 32],
            supported_protocol_versions: vec![
                CURRENT_PROTOCOL.to_owned(),
                PREVIOUS_PROTOCOL.to_owned(),
            ],
            issued_unix_millis: 1,
        };
        let mut descriptor = CapabilityDescriptor {
            worker_id: clipmill_core::WorkerId::new().to_string(),
            family: "echo".to_owned(),
            capabilities: vec!["demo-left".to_owned(), "demo-right".to_owned()],
            protocol_version: protocol_version.to_owned(),
            backend: "cpu".to_owned(),
            max_memory_bytes: 1024,
            public_key: signing.verifying_key().to_bytes().to_vec(),
            signature: Vec::new(),
            // Only 1.2 signs these, so a 1.1 descriptor must leave them unset.
            cpu_threads: u32::from(protocol_version == CURRENT_PROTOCOL),
            vram_bytes: 0,
        };
        descriptor.signature = signing
            .sign(&registration_preimage(&challenge, &descriptor))
            .to_bytes()
            .to_vec();
        (descriptor, challenge, signing.verifying_key().to_bytes())
    }

    #[test]
    fn registration_preimage_is_order_sensitive_and_excludes_signature() {
        let challenge = RegistrationChallenge {
            nonce: vec![7; 32],
            supported_protocol_versions: vec![CURRENT_PROTOCOL.to_owned()],
            issued_unix_millis: 1,
        };
        let mut descriptor = CapabilityDescriptor {
            worker_id: "wrk_01J00000000000000000000000".to_owned(),
            family: "echo".to_owned(),
            capabilities: vec!["demo-left".to_owned(), "demo-right".to_owned()],
            protocol_version: CURRENT_PROTOCOL.to_owned(),
            backend: "cpu".to_owned(),
            max_memory_bytes: 1024,
            public_key: vec![3; 32],
            signature: vec![4; 64],
            cpu_threads: 1,
            vram_bytes: 0,
        };
        let first = registration_preimage(&challenge, &descriptor);
        descriptor.signature = vec![9; 64];
        assert_eq!(first, registration_preimage(&challenge, &descriptor));
        descriptor.capabilities.reverse();
        assert_ne!(first, registration_preimage(&challenge, &descriptor));
    }

    fn machine(threads: u32, ram: u64, mask: u32, vram: u64) -> ResourceCapacity {
        ResourceCapacity {
            cpu_threads: threads,
            ram_bytes: ram,
            disk_bytes: 512 * 1024 * 1024,
            accelerator_mask: mask,
            vram_bytes: vram,
        }
    }

    fn declaring(backend: &str, threads: u32, ram: u64, vram: u64) -> CapabilityDescriptor {
        CapabilityDescriptor {
            worker_id: "wrk_01J00000000000000000000000".to_owned(),
            family: "asr".to_owned(),
            capabilities: vec!["demo-left".to_owned()],
            protocol_version: CURRENT_PROTOCOL.to_owned(),
            backend: backend.to_owned(),
            max_memory_bytes: ram,
            public_key: vec![3; 32],
            signature: vec![4; 64],
            cpu_threads: threads,
            vram_bytes: vram,
        }
    }

    const METAL: u32 = 1 << 4;

    /// A worker declaring an accelerator this machine never measured is
    /// declined, not admitted and left to fail at load time.
    #[test]
    fn a_worker_cannot_claim_hardware_the_device_never_measured() {
        let cpu_only = machine(8, 16 << 30, 0, 0);
        assert!(admit_capacity(&declaring("mlx", 2, 4 << 30, 0), cpu_only).is_err());
        assert!(admit_capacity(&declaring("cuda", 2, 4 << 30, 0), cpu_only).is_err());
        // A CPU runtime needs no accelerator and is admitted on the same box.
        let admitted = admit_capacity(&declaring("onnx-cpu", 2, 4 << 30, 0), cpu_only)
            .expect("a cpu runtime fits a cpu machine");
        assert_eq!(admitted.accelerator_mask, 0);
        assert_eq!(admitted.cpu_threads, 2);
    }

    #[test]
    fn an_accelerated_worker_is_admitted_where_the_accelerator_exists() {
        let apple = machine(10, 32 << 30, METAL, 0);
        let admitted =
            admit_capacity(&declaring("mlx", 4, 8 << 30, 0), apple).expect("metal is available");
        assert_eq!(admitted.accelerator_mask, METAL);
        assert_eq!(admitted.ram_bytes, 8 << 30);
    }

    /// On unified memory the same bytes back both allocations, so counting
    /// VRAM again would reserve them twice and starve the machine at half load.
    #[test]
    fn unified_memory_is_budgeted_once() {
        let apple = machine(10, 32 << 30, METAL, 0);
        let admitted = admit_capacity(&declaring("mlx", 4, 8 << 30, 6 << 30), apple)
            .expect("unified memory is one budget");
        assert_eq!(admitted.vram_bytes, 0, "VRAM was already counted as RAM");

        // A discrete card is a separate budget and is checked on its own.
        let discrete = machine(16, 64 << 30, 1 << 2, 8 << 30);
        let admitted = admit_capacity(&declaring("cuda", 4, 8 << 30, 6 << 30), discrete)
            .expect("the card holds it");
        assert_eq!(admitted.vram_bytes, 6 << 30);
        assert!(admit_capacity(&declaring("cuda", 4, 8 << 30, 12 << 30), discrete).is_err());
    }

    #[test]
    fn over_declared_resources_are_refused_rather_than_clamped() {
        let small = machine(2, 4 << 30, 0, 0);
        assert!(admit_capacity(&declaring("cpu", 8, 1 << 30, 0), small).is_err());
        assert!(admit_capacity(&declaring("cpu", 1, 32 << 30, 0), small).is_err());
        assert!(admit_capacity(&declaring("wishful-thinking", 1, 1 << 30, 0), small).is_err());
    }

    /// Declared resources joined the signature in 1.2. A 1.1 descriptor that
    /// carries them is refused, because its signature does not cover them and
    /// honouring them would let a worker widen its own budget after signing.
    #[test]
    fn declared_resources_are_only_honoured_where_the_signature_covers_them() {
        let challenge = RegistrationChallenge {
            nonce: vec![7; 32],
            supported_protocol_versions: vec![CURRENT_PROTOCOL.to_owned()],
            issued_unix_millis: 1,
        };
        let mut descriptor = declaring("cpu", 4, 1 << 30, 0);
        let baseline = registration_preimage(&challenge, &descriptor);
        descriptor.cpu_threads = 8;
        assert_ne!(
            baseline,
            registration_preimage(&challenge, &descriptor),
            "1.2 must sign its declared threads"
        );

        let mut legacy = declaring("cpu", 0, 1 << 30, 0);
        legacy.protocol_version = PREVIOUS_PROTOCOL.to_owned();
        let legacy_baseline = registration_preimage(&challenge, &legacy);
        legacy.cpu_threads = 8;
        assert_eq!(
            legacy_baseline,
            registration_preimage(&challenge, &legacy),
            "1.1 signatures keep their original shape"
        );

        let trust = BTreeMap::new();
        let mut smuggled = declaring("cpu", 4, 1 << 30, 0);
        smuggled.protocol_version = PREVIOUS_PROTOCOL.to_owned();
        assert!(
            matches!(
                validate_registration(&smuggled, &challenge, &trust),
                Err(WorkerError::InvalidDescriptor)
            ),
            "a 1.1 worker may not declare unsigned resources"
        );
    }

    #[test]
    fn capability_words_are_portable() {
        assert!(valid_word("demo-seed"));
        assert!(!valid_word("Demo"));
        assert!(!valid_word("demo,seed"));
        assert!(!valid_word("-demo"));
    }

    #[test]
    fn current_and_previous_protocols_require_fresh_trusted_signatures() {
        // The daemon serves the current minor and the one before it; 1.0
        // fell out of that window when declared resources arrived in 1.2.
        for protocol in [CURRENT_PROTOCOL, PREVIOUS_PROTOCOL] {
            let (descriptor, challenge, public) = signed_descriptor(protocol);
            let trust = BTreeMap::from([(descriptor.worker_id.clone(), public)]);
            validate_registration(&descriptor, &challenge, &trust).expect("trusted registration");

            let mut replayed_challenge = challenge.clone();
            replayed_challenge.nonce[0] ^= 1;
            assert!(matches!(
                validate_registration(&descriptor, &replayed_challenge, &trust),
                Err(WorkerError::InvalidSignature)
            ));
            assert!(matches!(
                validate_registration(&descriptor, &challenge, &BTreeMap::new()),
                Err(WorkerError::UntrustedWorker)
            ));
            let wrong_identity =
                BTreeMap::from([(clipmill_core::WorkerId::new().to_string(), public)]);
            assert!(matches!(
                validate_registration(&descriptor, &challenge, &wrong_identity),
                Err(WorkerError::UntrustedWorker)
            ));
        }
    }

    #[test]
    fn invalid_capabilities_protocols_and_duplicate_workers_are_rejected() {
        let (mut descriptor, challenge, public) = signed_descriptor("1.1");
        let trust = BTreeMap::from([(descriptor.worker_id.clone(), public)]);
        descriptor.capabilities.reverse();
        assert!(matches!(
            validate_registration(&descriptor, &challenge, &trust),
            Err(WorkerError::InvalidDescriptor)
        ));

        let (mut descriptor, challenge, public) = signed_descriptor("2.0");
        let trust = BTreeMap::from([(descriptor.worker_id.clone(), public)]);
        assert!(matches!(
            validate_registration(&descriptor, &challenge, &trust),
            Err(WorkerError::InvalidDescriptor)
        ));
        descriptor.protocol_version = "1.1".to_owned();

        let workers = Arc::new(Mutex::new(BTreeSet::new()));
        let active = ActiveWorker::acquire(Arc::clone(&workers), descriptor.worker_id.clone())
            .expect("first worker");
        assert!(matches!(
            ActiveWorker::acquire(Arc::clone(&workers), descriptor.worker_id.clone()),
            Err(WorkerError::DuplicateWorker)
        ));
        drop(active);
        ActiveWorker::acquire(workers, descriptor.worker_id).expect("worker ID released");
    }

    #[tokio::test]
    async fn worker_framing_handles_fragmentation_and_rejects_bad_lengths() {
        let request = clipmill_contracts::proto::worker::v1::WorkerRequest {
            body: Some(
                clipmill_contracts::proto::worker::v1::worker_request::Body::WorkRequest(
                    clipmill_contracts::proto::worker::v1::WorkRequest { max_wait_ms: 10 },
                ),
            ),
        };
        let encoded = request.encode_length_delimited_to_vec();
        let (mut sender, mut receiver) = tokio::net::UnixStream::pair().expect("socket pair");
        let writer = tokio::spawn(async move {
            for byte in encoded {
                sender.write_all(&[byte]).await.expect("fragment write");
                tokio::task::yield_now().await;
            }
        });
        assert_eq!(
            read_request(&mut receiver).await.expect("fragmented frame"),
            request
        );
        writer.await.expect("fragment writer");

        let (mut sender, mut receiver) = tokio::net::UnixStream::pair().expect("socket pair");
        sender
            .write_all(&encode_varint_for_test(
                u64::try_from(MAX_FRAME_BYTES + 1).expect("frame limit"),
            ))
            .await
            .expect("oversized prefix");
        assert!(matches!(
            read_request(&mut receiver).await,
            Err(WorkerError::OversizedFrame)
        ));

        let (mut sender, mut receiver) = tokio::net::UnixStream::pair().expect("socket pair");
        sender
            .write_all(&[0x80; 10])
            .await
            .expect("malformed prefix");
        assert!(matches!(
            read_request(&mut receiver).await,
            Err(WorkerError::MalformedFrame)
        ));

        let (mut sender, mut receiver) = tokio::net::UnixStream::pair().expect("socket pair");
        sender.write_all(&[0]).await.expect("zero prefix");
        assert!(matches!(
            read_request(&mut receiver).await,
            Err(WorkerError::OversizedFrame)
        ));
    }

    fn encode_varint_for_test(mut value: u64) -> Vec<u8> {
        let mut encoded = Vec::new();
        while value >= 0x80 {
            encoded.push(u8::try_from(value & 0x7f).expect("seven bits") | 0x80);
            value >>= 7;
        }
        encoded.push(u8::try_from(value).expect("final varint byte"));
        encoded
    }
}
