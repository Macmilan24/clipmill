use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Write,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use clipmill_artifacts::{
    ArtifactPath, ArtifactRecipe, NetworkPolicy, PrepareOutcome, Producer, RecipeSpec, Timebase,
};
use clipmill_contracts::proto::{
    ipc::v1::{self, DeviceProfilePayloadV1, JobState},
    worker::v1::{FailureClass, ProgressUnits},
};
use clipmill_core::{ArtifactId, JobId, LeaseId, ProjectId, Sha256Digest, TaskId};
use prost::Message;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use tokio::{
    sync::{Notify, broadcast, oneshot},
    task::{JoinHandle, JoinSet},
    time::{MissedTickBehavior, interval},
};

use crate::{
    artifacts::ArtifactHandle,
    db::{DbHandle, StoreError},
    device::{DeviceProfiler, VerifiedDeviceProfile, verify_profile},
    media::{self, MediaRunner, ProgressSlot},
    sources::{SourceInspector, SourceProbeError},
};

pub(crate) const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
pub(crate) const LEASE_TTL: Duration = Duration::from_secs(15);
pub(crate) const DEVICE_PROFILE_KEY_VERSION: &str = "clipmill.device-profile.v1";
pub(crate) const SYSTEM_PROJECT_ID: &str = "prj_00000000000000000000000000";
const SCHEDULER_TICK: Duration = Duration::from_millis(100);
const MAX_BUILTIN_TASKS: usize = 4;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResourceDeclaration {
    pub cpu_threads: u32,
    pub ram_bytes: u64,
    pub accelerator_class: String,
    pub vram_bytes: u64,
    pub disk_bytes: u64,
    pub network_policy: String,
    pub thermal_class: String,
    pub determinism_class: String,
    pub checkpoint_support: bool,
    pub preemption_cost: u32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ResourceCapacity {
    pub cpu_threads: u32,
    pub ram_bytes: u64,
    pub disk_bytes: u64,
    pub accelerator_mask: u32,
    pub vram_bytes: u64,
}

impl ResourceCapacity {
    #[cfg(test)]
    pub(crate) fn w4_builtin() -> Self {
        let cpu_threads = std::thread::available_parallelism()
            .map_or(1, std::num::NonZeroUsize::get)
            .min(MAX_BUILTIN_TASKS);
        Self {
            cpu_threads: u32::try_from(cpu_threads).unwrap_or(u32::MAX),
            ram_bytes: 512 * 1024 * 1024,
            disk_bytes: 512 * 1024 * 1024,
            accelerator_mask: 0,
            vram_bytes: 0,
        }
    }

    pub(crate) fn measured(logical_cores: u32, available_memory_bytes: u64) -> Self {
        let maximum_threads = u32::try_from(MAX_BUILTIN_TASKS).unwrap_or(u32::MAX);
        Self {
            cpu_threads: logical_cores.clamp(1, maximum_threads),
            ram_bytes: available_memory_bytes
                .saturating_mul(3)
                .checked_div(4)
                .unwrap_or(available_memory_bytes)
                .max(64 * 1024 * 1024),
            disk_bytes: 512 * 1024 * 1024,
            accelerator_mask: 0,
            vram_bytes: 0,
        }
    }

    pub(crate) fn with_available_backends(mut self, backends: &BTreeSet<String>) -> Self {
        self.accelerator_mask = backends
            .iter()
            .filter_map(|backend| accelerator_bit(backend))
            .fold(0, std::ops::BitOr::bitor);
        self
    }

    fn reserve(&mut self, resources: &ResourceDeclaration) -> bool {
        let accelerator_available = if resources.accelerator_class.is_empty() {
            resources.vram_bytes == 0
        } else {
            accelerator_bit(&resources.accelerator_class)
                .is_some_and(|bit| self.accelerator_mask & bit != 0)
                && resources.vram_bytes <= self.vram_bytes
        };
        if accelerator_available
            && resources.network_policy == "local-lock"
            && resources.cpu_threads <= self.cpu_threads
            && resources.ram_bytes <= self.ram_bytes
            && resources.disk_bytes <= self.disk_bytes
        {
            self.cpu_threads -= resources.cpu_threads;
            self.ram_bytes -= resources.ram_bytes;
            self.disk_bytes -= resources.disk_bytes;
            self.vram_bytes -= resources.vram_bytes;
            true
        } else {
            false
        }
    }

    fn release(&mut self, resources: &ResourceDeclaration) {
        self.cpu_threads = self.cpu_threads.saturating_add(resources.cpu_threads);
        self.ram_bytes = self.ram_bytes.saturating_add(resources.ram_bytes);
        self.disk_bytes = self.disk_bytes.saturating_add(resources.disk_bytes);
        self.vram_bytes = self.vram_bytes.saturating_add(resources.vram_bytes);
    }
}

pub(crate) fn accelerator_bit(backend: &str) -> Option<u32> {
    match backend {
        "videotoolbox" => Some(1 << 0),
        "vaapi" => Some(1 << 1),
        "cuda" => Some(1 << 2),
        "vulkan" => Some(1 << 3),
        "metal" => Some(1 << 4),
        _ => None,
    }
}

impl ResourceDeclaration {
    fn demo() -> Self {
        Self {
            cpu_threads: 1,
            ram_bytes: 1024 * 1024,
            accelerator_class: String::new(),
            vram_bytes: 0,
            disk_bytes: 1024 * 1024,
            network_policy: "local-lock".to_owned(),
            thermal_class: "light".to_owned(),
            determinism_class: "deterministic".to_owned(),
            checkpoint_support: false,
            preemption_cost: 1,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TaskSpec {
    pub task_id: String,
    pub ordinal: u32,
    pub kind: String,
    pub input_kinds: Vec<String>,
    pub output_kind: String,
    pub payload: Vec<u8>,
    pub dependencies: Vec<String>,
    pub resources: ResourceDeclaration,
    pub implementation: String,
    pub max_attempts: u32,
    pub is_final: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct JobPlan {
    pub job_id: String,
    pub project_id: String,
    pub kind: String,
    pub source_id: Option<String>,
    pub payload: Vec<u8>,
    pub created_unix_millis: u64,
    pub tasks: Vec<TaskSpec>,
}

impl JobPlan {
    pub(crate) fn demo(project_id: &ProjectId, payload: Vec<u8>, now: u64) -> Self {
        let job_id = JobId::new().to_string();
        let seed = TaskId::new().to_string();
        let left = TaskId::new().to_string();
        let right = TaskId::new().to_string();
        let join = TaskId::new().to_string();
        let task_payload = payload.clone();
        let task = |task_id: String,
                    ordinal: u32,
                    kind: &str,
                    input_kinds: Vec<&str>,
                    output_kind: &str,
                    dependencies: Vec<String>,
                    is_final: bool| TaskSpec {
            task_id,
            ordinal,
            kind: kind.to_owned(),
            input_kinds: input_kinds.into_iter().map(str::to_owned).collect(),
            output_kind: output_kind.to_owned(),
            payload: task_payload.clone(),
            dependencies,
            resources: ResourceDeclaration::demo(),
            implementation: "builtin-demo@1.0.0".to_owned(),
            max_attempts: 3,
            is_final,
        };
        Self {
            job_id,
            project_id: project_id.to_string(),
            kind: "demo-dag".to_owned(),
            source_id: None,
            payload,
            created_unix_millis: now,
            tasks: vec![
                task(
                    seed.clone(),
                    0,
                    "demo-seed",
                    Vec::new(),
                    "evidence.demo.seed.v1",
                    Vec::new(),
                    false,
                ),
                task(
                    left.clone(),
                    1,
                    "demo-left",
                    vec!["evidence.demo.seed.v1"],
                    "evidence.demo.left.v1",
                    vec![seed.clone()],
                    false,
                ),
                task(
                    right.clone(),
                    2,
                    "demo-right",
                    vec!["evidence.demo.seed.v1"],
                    "evidence.demo.right.v1",
                    vec![seed],
                    false,
                ),
                task(
                    join,
                    3,
                    "demo-join",
                    vec!["evidence.demo.left.v1", "evidence.demo.right.v1"],
                    "evidence.demo.final.v1",
                    vec![left, right],
                    true,
                ),
            ],
        }
    }

    pub(crate) fn probe_source(
        project_id: &ProjectId,
        source_id: String,
        payload: Vec<u8>,
        now: u64,
    ) -> Self {
        let job_id = JobId::new().to_string();
        Self {
            job_id,
            project_id: project_id.to_string(),
            kind: "probe-source".to_owned(),
            source_id: Some(source_id),
            payload: payload.clone(),
            created_unix_millis: now,
            tasks: vec![TaskSpec {
                task_id: TaskId::new().to_string(),
                ordinal: 0,
                kind: "probe-source".to_owned(),
                input_kinds: Vec::new(),
                output_kind: "evidence.source_map.v1".to_owned(),
                payload,
                dependencies: Vec::new(),
                resources: ResourceDeclaration {
                    cpu_threads: 1,
                    ram_bytes: 64 * 1024 * 1024,
                    accelerator_class: String::new(),
                    vram_bytes: 0,
                    disk_bytes: 32 * 1024 * 1024,
                    network_policy: "local-lock".to_owned(),
                    thermal_class: "light".to_owned(),
                    determinism_class: "deterministic".to_owned(),
                    checkpoint_support: false,
                    preemption_cost: 1,
                },
                implementation: "ffprobe-8.1.2+clipmill-map-v1".to_owned(),
                max_attempts: 3,
                is_final: true,
            }],
        }
    }

    /// The W11 ingest fan-out (book ch. 12): decode the source video once
    /// into the proxy, render the PCM diets straight from the source's audio,
    /// then hang every other derivative off an already-committed artifact.
    /// The single final task is the fan-in manifest whose recipe inputs are
    /// all children, because the job store roots exactly one artifact per job
    /// and garbage collection walks recipe inputs from the roots.
    pub(crate) fn ingest_source(
        project_id: &ProjectId,
        source_id: String,
        payload: Vec<u8>,
        has_video: bool,
        has_audio: bool,
        now: u64,
    ) -> Result<Self, &'static str> {
        if !has_video && !has_audio {
            return Err("source carries neither video nor audio");
        }
        let mut builder = IngestPlanBuilder::new(payload);
        let proxy = has_video.then(|| {
            builder.derivative(
                media::KIND_PROXY,
                "media.proxy.v1",
                "ffmpeg-8.1.2+clipmill-proxy-v1",
                ingest_resources(2, 256, 512),
                &[],
            )
        });
        let audio_16k = has_audio.then(|| {
            builder.derivative(
                media::KIND_AUDIO_16K,
                "media.audio_16k.v1",
                "ffmpeg-8.1.2+clipmill-audio-v1",
                ingest_resources(1, 64, 128),
                &[],
            )
        });
        let audio_48k = has_audio.then(|| {
            builder.derivative(
                media::KIND_AUDIO_48K,
                "media.audio_48k.v1",
                "ffmpeg-8.1.2+clipmill-audio-v1",
                ingest_resources(1, 64, 256),
                &[],
            )
        });
        builder.derivative(
            media::KIND_REFERENCE_INDEX,
            "media.reference_index.v1",
            "ffprobe-8.1.2+clipmill-refindex-v1",
            ingest_resources(1, 128, 64),
            &[],
        );
        if let Some(audio_48k) = &audio_48k {
            builder.derivative(
                media::KIND_LOUDNESS,
                "media.loudness_envelope.v1",
                "ffmpeg-8.1.2+clipmill-loudness-v1",
                ingest_resources(1, 64, 32),
                std::slice::from_ref(audio_48k),
            );
        }
        if let Some(audio_16k) = &audio_16k {
            builder.derivative(
                media::KIND_AUDIO_PEAKS,
                "media.audio_peaks.v1",
                "clipmill-audio-peaks@1.0.0",
                ingest_resources(1, 64, 32),
                std::slice::from_ref(audio_16k),
            );
        }
        if let Some(proxy) = &proxy {
            builder.derivative(
                media::KIND_FILMSTRIP,
                "media.filmstrip.v1",
                "ffmpeg-8.1.2+clipmill-filmstrip-v1",
                ingest_resources(1, 128, 128),
                std::slice::from_ref(proxy),
            );
            builder.derivative(
                media::KIND_FRAMES,
                "media.frames.v1",
                "ffmpeg-8.1.2+clipmill-frames-v1",
                ingest_resources(1, 128, 256),
                std::slice::from_ref(proxy),
            );
        }
        let (payload, tasks) = builder.finish_with_manifest();
        Ok(Self {
            job_id: JobId::new().to_string(),
            project_id: project_id.to_string(),
            kind: "ingest-source".to_owned(),
            source_id: Some(source_id),
            payload,
            created_unix_millis: now,
            tasks,
        })
    }

    pub(crate) fn device_profile(
        hardware_fingerprint: String,
        measurement_generation: u64,
        now: u64,
    ) -> Self {
        let payload = DeviceProfilePayloadV1 {
            key_version: DEVICE_PROFILE_KEY_VERSION.to_owned(),
            hardware_fingerprint,
            measurement_generation,
        }
        .encode_to_vec();
        Self {
            job_id: JobId::new().to_string(),
            project_id: SYSTEM_PROJECT_ID.to_owned(),
            kind: "device-profile".to_owned(),
            source_id: None,
            payload: payload.clone(),
            created_unix_millis: now,
            tasks: vec![TaskSpec {
                task_id: TaskId::new().to_string(),
                ordinal: 0,
                kind: "device-profile".to_owned(),
                input_kinds: Vec::new(),
                output_kind: "evidence.device_profile.v1".to_owned(),
                payload,
                dependencies: Vec::new(),
                resources: ResourceDeclaration {
                    cpu_threads: 1,
                    ram_bytes: 64 * 1024 * 1024,
                    accelerator_class: String::new(),
                    vram_bytes: 0,
                    disk_bytes: 32 * 1024 * 1024,
                    network_policy: "local-lock".to_owned(),
                    thermal_class: "light".to_owned(),
                    determinism_class: "generation-scoped".to_owned(),
                    checkpoint_support: false,
                    preemption_cost: 1,
                },
                implementation: "clipmill-device-profiler@1.0.0".to_owned(),
                max_attempts: 3,
                is_final: true,
            }],
        }
    }
}

fn ingest_resources(
    cpu_threads: u32,
    ram_mebibytes: u64,
    disk_mebibytes: u64,
) -> ResourceDeclaration {
    ResourceDeclaration {
        cpu_threads,
        ram_bytes: ram_mebibytes * 1024 * 1024,
        accelerator_class: String::new(),
        vram_bytes: 0,
        disk_bytes: disk_mebibytes * 1024 * 1024,
        network_policy: "local-lock".to_owned(),
        thermal_class: "sustained".to_owned(),
        determinism_class: "deterministic".to_owned(),
        checkpoint_support: false,
        preemption_cost: 2,
    }
}

/// A derivative already added to an ingest plan, usable as a dependency.
#[derive(Clone)]
struct DerivativeHandle {
    task_id: String,
    output_kind: String,
}

/// Accumulates the ingest fan-out and closes it with the fan-in manifest,
/// keeping ordinals dense and dependency/input-kind lists aligned (the plan
/// validator requires one input kind per dependency).
struct IngestPlanBuilder {
    payload: Vec<u8>,
    tasks: Vec<TaskSpec>,
    children: Vec<DerivativeHandle>,
}

impl IngestPlanBuilder {
    fn new(payload: Vec<u8>) -> Self {
        Self {
            payload,
            tasks: Vec::new(),
            children: Vec::new(),
        }
    }

    fn derivative(
        &mut self,
        kind: &str,
        output_kind: &str,
        implementation: &str,
        resources: ResourceDeclaration,
        dependencies: &[DerivativeHandle],
    ) -> DerivativeHandle {
        let handle = DerivativeHandle {
            task_id: TaskId::new().to_string(),
            output_kind: output_kind.to_owned(),
        };
        self.tasks.push(TaskSpec {
            task_id: handle.task_id.clone(),
            ordinal: u32::try_from(self.tasks.len()).unwrap_or(u32::MAX),
            kind: kind.to_owned(),
            input_kinds: dependencies
                .iter()
                .map(|dependency| dependency.output_kind.clone())
                .collect(),
            output_kind: output_kind.to_owned(),
            payload: self.payload.clone(),
            dependencies: dependencies
                .iter()
                .map(|dependency| dependency.task_id.clone())
                .collect(),
            resources,
            implementation: implementation.to_owned(),
            max_attempts: 3,
            is_final: false,
        });
        self.children.push(handle.clone());
        handle
    }

    fn finish_with_manifest(mut self) -> (Vec<u8>, Vec<TaskSpec>) {
        let manifest = TaskSpec {
            task_id: TaskId::new().to_string(),
            ordinal: u32::try_from(self.tasks.len()).unwrap_or(u32::MAX),
            kind: media::KIND_MANIFEST.to_owned(),
            input_kinds: self
                .children
                .iter()
                .map(|child| child.output_kind.clone())
                .collect(),
            output_kind: "media.ingest_manifest.v1".to_owned(),
            payload: self.payload.clone(),
            dependencies: self
                .children
                .iter()
                .map(|child| child.task_id.clone())
                .collect(),
            resources: ResourceDeclaration {
                cpu_threads: 1,
                ram_bytes: 32 * 1024 * 1024,
                accelerator_class: String::new(),
                vram_bytes: 0,
                disk_bytes: 8 * 1024 * 1024,
                network_policy: "local-lock".to_owned(),
                thermal_class: "light".to_owned(),
                determinism_class: "deterministic".to_owned(),
                checkpoint_support: false,
                preemption_cost: 1,
            },
            implementation: "clipmill-ingest-manifest@1.0.0".to_owned(),
            max_attempts: 3,
            is_final: true,
        };
        self.tasks.push(manifest);
        (self.payload, self.tasks)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TaskRecord {
    pub task_id: String,
    pub kind: String,
    pub state: i32,
    pub attempt: u32,
    pub max_attempts: u32,
    pub progress_unit: String,
    pub progress_done: u64,
    pub progress_total: u64,
    pub wait_reason: String,
    pub output_artifact_id: String,
}

impl From<TaskRecord> for v1::Task {
    fn from(value: TaskRecord) -> Self {
        let progress = (!value.progress_unit.is_empty()).then_some(ProgressUnits {
            unit: value.progress_unit,
            done: value.progress_done,
            total: value.progress_total,
        });
        Self {
            task_id: value.task_id,
            kind: value.kind,
            state: value.state,
            attempt: value.attempt,
            max_attempts: value.max_attempts,
            progress,
            wait_reason: value.wait_reason,
            output_artifact_id: value.output_artifact_id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct JobRecord {
    pub job_id: String,
    pub project_id: String,
    pub kind: String,
    pub state: i32,
    pub created_unix_millis: u64,
    pub updated_unix_millis: u64,
    pub tasks: Vec<TaskRecord>,
    pub output_artifact_ids: Vec<String>,
    pub failure_class: i32,
    pub failure_detail: String,
}

impl From<JobRecord> for v1::Job {
    fn from(value: JobRecord) -> Self {
        Self {
            job_id: value.job_id,
            project_id: value.project_id,
            kind: value.kind,
            state: value.state,
            created_unix_millis: value.created_unix_millis,
            updated_unix_millis: value.updated_unix_millis,
            tasks: value.tasks.into_iter().map(Into::into).collect(),
            output_artifact_ids: value.output_artifact_ids,
            failure_class: value.failure_class,
            failure_detail: value.failure_detail,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TaskEventRecord {
    pub event_id: u64,
    pub project_id: String,
    pub job_id: String,
    pub task_id: String,
    pub state: i32,
    pub attempt: u32,
    pub progress_unit: String,
    pub progress_done: u64,
    pub progress_total: u64,
    pub wait_reason: String,
    pub failure_class: i32,
    pub at_unix_millis: u64,
}

impl TaskEventRecord {
    #[must_use]
    pub(crate) fn as_proto(&self) -> v1::TaskEvent {
        let progress = (!self.progress_unit.is_empty()).then_some(ProgressUnits {
            unit: self.progress_unit.clone(),
            done: self.progress_done,
            total: self.progress_total,
        });
        v1::TaskEvent {
            job_id: self.job_id.clone(),
            task_id: self.task_id.clone(),
            state: self.state,
            progress,
            wait_reason: self.wait_reason.clone(),
            at_unix_millis: self.at_unix_millis,
            event_id: self.event_id,
            attempt: self.attempt,
            failure_class: self.failure_class,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct EventFilter {
    pub project_id: Option<String>,
    pub job_id: Option<String>,
}

impl EventFilter {
    #[must_use]
    pub(crate) fn matches(&self, event: &TaskEventRecord) -> bool {
        self.project_id
            .as_ref()
            .is_none_or(|project_id| project_id == &event.project_id)
            && self
                .job_id
                .as_ref()
                .is_none_or(|job_id| job_id == &event.job_id)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct EventHub {
    sender: broadcast::Sender<TaskEventRecord>,
}

impl EventHub {
    #[must_use]
    pub(crate) fn new() -> Self {
        let (sender, _receiver) = broadcast::channel(1024);
        Self { sender }
    }

    pub(crate) fn publish_all(&self, events: impl IntoIterator<Item = TaskEventRecord>) {
        for event in events {
            let _receivers = self.sender.send(event);
        }
    }

    pub(crate) fn subscribe(&self) -> broadcast::Receiver<TaskEventRecord> {
        self.sender.subscribe()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LeasedTask {
    pub project_id: String,
    pub job_id: String,
    pub source_id: Option<String>,
    pub task_id: String,
    pub lease_id: String,
    pub kind: String,
    pub output_kind: String,
    pub payload: Vec<u8>,
    pub implementation: String,
    pub attempt: u32,
    pub input_artifact_ids: Vec<ArtifactId>,
    pub resources: ResourceDeclaration,
}

#[derive(Clone, Debug)]
pub(crate) struct LeaseSelection {
    pub task: Option<LeasedTask>,
    pub events: Vec<TaskEventRecord>,
}

#[derive(Clone, Debug)]
pub(crate) struct LeaseRequest {
    pub lease_id: String,
    pub daemon_epoch: String,
    pub now_unix_millis: u64,
    pub expires_unix_millis: u64,
    pub capacity: ResourceCapacity,
    pub worker_id: String,
    pub capabilities: Vec<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct TaskCompletion {
    pub response: Vec<u8>,
    pub events: Vec<TaskEventRecord>,
}

#[derive(Debug)]
pub(crate) struct Scheduler {
    handle: SchedulerHandle,
    stopped: Option<oneshot::Sender<()>>,
    task: JoinHandle<()>,
}

#[derive(Clone, Debug)]
pub(crate) struct SchedulerHandle {
    notify: Arc<Notify>,
    capacity_update: Arc<Mutex<Option<ResourceCapacity>>>,
    capacity_limit: ResourceCapacity,
}

impl SchedulerHandle {
    pub(crate) fn notify(&self) {
        self.notify.notify_one();
    }

    pub(crate) fn apply_device_profile(&self, profile: &VerifiedDeviceProfile) {
        let measured =
            ResourceCapacity::measured(profile.logical_cores, profile.available_memory_bytes)
                .with_available_backends(&profile.available_backends);
        let capacity = ResourceCapacity {
            cpu_threads: measured
                .cpu_threads
                .min(self.capacity_limit.cpu_threads)
                .max(1),
            ram_bytes: measured.ram_bytes.min(self.capacity_limit.ram_bytes),
            disk_bytes: measured.disk_bytes.min(self.capacity_limit.disk_bytes),
            accelerator_mask: measured.accelerator_mask,
            vram_bytes: measured.vram_bytes,
        };
        if let Ok(mut pending) = self.capacity_update.lock() {
            *pending = Some(capacity);
        }
        self.notify.notify_one();
    }
}

impl Scheduler {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn start(
        database: DbHandle,
        artifacts: ArtifactHandle,
        events: EventHub,
        daemon_epoch: String,
        sources: SourceInspector,
        device_profiler: DeviceProfiler,
        media: MediaRunner,
        capacity: ResourceCapacity,
        builtin_fixture_executor: bool,
    ) -> Self {
        debug_assert!(LEASE_TTL >= HEARTBEAT_INTERVAL.saturating_mul(3));
        let notify = Arc::new(Notify::new());
        let capacity_update = Arc::new(Mutex::new(None));
        let handle = SchedulerHandle {
            notify: Arc::clone(&notify),
            capacity_update: Arc::clone(&capacity_update),
            capacity_limit: capacity,
        };
        let (stopped, stop) = oneshot::channel();
        let task = tokio::spawn(run_scheduler(
            database,
            artifacts,
            events,
            daemon_epoch,
            sources,
            device_profiler,
            media,
            capacity,
            capacity_update,
            builtin_fixture_executor,
            notify,
            stop,
        ));
        handle.notify();
        Self {
            handle,
            stopped: Some(stopped),
            task,
        }
    }

    #[must_use]
    pub(crate) fn handle(&self) -> SchedulerHandle {
        self.handle.clone()
    }

    pub(crate) async fn shutdown(mut self) {
        if let Some(stopped) = self.stopped.take() {
            let _sent = stopped.send(());
        }
        let _joined = self.task.await;
    }
}

#[allow(clippy::too_many_arguments)]
async fn run_scheduler(
    database: DbHandle,
    artifacts: ArtifactHandle,
    events: EventHub,
    daemon_epoch: String,
    sources: SourceInspector,
    device_profiler: DeviceProfiler,
    media: MediaRunner,
    capacity: ResourceCapacity,
    capacity_update: Arc<Mutex<Option<ResourceCapacity>>>,
    builtin_fixture_executor: bool,
    notify: Arc<Notify>,
    mut stop: oneshot::Receiver<()>,
) {
    let mut schedule = interval(SCHEDULER_TICK);
    schedule.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut running = JoinSet::new();
    let mut available_capacity = capacity;
    let mut builtin_capabilities = vec!["probe-source".to_owned(), "device-profile".to_owned()];
    builtin_capabilities.extend(media::INGEST_TASK_KINDS.map(str::to_owned));
    if builtin_fixture_executor {
        builtin_capabilities
            .extend(["demo-seed", "demo-left", "demo-right", "demo-join"].map(str::to_owned));
    }
    loop {
        tokio::select! {
            biased;
            _ = &mut stop => break,
            joined = running.join_next(), if !running.is_empty() => {
                match joined {
                    Some(Ok(resources)) => available_capacity.release(&resources),
                    Some(Err(error)) => {
                        tracing::warn!(%error, "built-in task executor stopped unexpectedly");
                    }
                    None => {}
                }
            }
            () = notify.notified() => {}
            _ = schedule.tick() => {}
        }

        if let Ok(expired) = database
            .expire_task_leases(now_millis(), &daemon_epoch)
            .await
        {
            events.publish_all(expired);
        }
        if running.is_empty()
            && let Ok(mut pending) = capacity_update.lock()
            && let Some(measured) = pending.take()
        {
            available_capacity = measured;
            tracing::info!(
                cpu_threads = measured.cpu_threads,
                ram_bytes = measured.ram_bytes,
                "scheduler applied verified device profile capacity"
            );
        }
        while running.len() < MAX_BUILTIN_TASKS && available_capacity.cpu_threads > 0 {
            let now = now_millis();
            let lease = LeaseId::new().to_string();
            let leased = database
                .lease_next_task(LeaseRequest {
                    lease_id: lease,
                    daemon_epoch: daemon_epoch.clone(),
                    now_unix_millis: now,
                    expires_unix_millis: now.saturating_add(duration_millis(LEASE_TTL)),
                    capacity: available_capacity,
                    worker_id: "builtin-fixture".to_owned(),
                    capabilities: builtin_capabilities.clone(),
                })
                .await;
            let Ok(selection) = leased else {
                break;
            };
            events.publish_all(selection.events);
            let Some(task) = selection.task else {
                break;
            };
            let resources = task.resources.clone();
            if !available_capacity.reserve(&resources) {
                tracing::error!(
                    task_id = task.task_id,
                    "database admitted an over-capacity task"
                );
                break;
            }
            let database = database.clone();
            let artifacts = artifacts.clone();
            let events = events.clone();
            let sources = sources.clone();
            let device_profiler = device_profiler.clone();
            let media = media.clone();
            let notify = Arc::clone(&notify);
            running.spawn(async move {
                execute_task(
                    database,
                    artifacts,
                    events,
                    sources,
                    device_profiler,
                    media,
                    task,
                )
                .await;
                notify.notify_one();
                resources
            });
        }
    }

    running.abort_all();
    while running.join_next().await.is_some() {}
}

#[allow(clippy::too_many_lines)]
async fn execute_task(
    database: DbHandle,
    artifacts: ArtifactHandle,
    events: EventHub,
    sources: SourceInspector,
    device_profiler: DeviceProfiler,
    media: MediaRunner,
    task: LeasedTask,
) {
    tracing::debug!(
        project_id = task.project_id,
        job_id = task.job_id,
        task_id = task.task_id,
        attempt = task.attempt,
        "executing built-in durable task"
    );
    let lease_id = task.lease_id.clone();
    let progress = ProgressSlot::default();
    let work = async {
        if let Ok(delay) = std::env::var("CLIPMILL_W4_STEP_DELAY_MS")
            && let Ok(delay) = delay.parse::<u64>()
        {
            tokio::time::sleep(Duration::from_millis(delay.min(30_000))).await;
        }
        match task.kind.as_str() {
            "probe-source" => execute_probe_artifact(&database, &artifacts, &sources, &task).await,
            "device-profile" => {
                execute_device_artifact(&database, &artifacts, &device_profiler, &task).await
            }
            kind if media::is_ingest_kind(kind) => {
                media::execute_ingest_task(
                    &database, &artifacts, &media, &sources, &task, &progress,
                )
                .await
            }
            _ => execute_demo_artifact(&artifacts, &task)
                .await
                .map_err(TaskExecutionError::transient),
        }
    };
    tokio::pin!(work);
    let mut heartbeat = interval(HEARTBEAT_INTERVAL);
    heartbeat.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let outcome = loop {
        tokio::select! {
            result = &mut work => break Some(result),
            _ = heartbeat.tick() => {
                let now = now_millis();
                if let Ok(task_events) = database
                    .heartbeat_task(
                        lease_id.clone(),
                        now,
                        now.saturating_add(duration_millis(LEASE_TTL)),
                        progress.take(),
                    )
                    .await
                {
                    events.publish_all(task_events);
                } else {
                    tracing::debug!(task_id = task.task_id, "task lease heartbeat was rejected");
                    break None;
                }
            }
        }
    };
    let Some(outcome) = outcome else {
        return;
    };
    match outcome {
        Ok(artifact_id) => {
            let response = artifact_id.to_string().into_bytes();
            let expected_response = response.clone();
            match database
                .complete_task(
                    lease_id.clone(),
                    artifact_id,
                    Sha256::digest(&response).into(),
                    response,
                    now_millis(),
                )
                .await
            {
                Ok(completion) => {
                    if completion.response == expected_response {
                        events.publish_all(completion.events);
                    } else {
                        tracing::error!(
                            task_id = task.task_id,
                            "durable task completion returned an inconsistent response"
                        );
                    }
                }
                Err(StoreError::Conflict | StoreError::NotFound) => {
                    tracing::debug!(task_id = task.task_id, "discarded stale task completion");
                }
                Err(error) => {
                    tracing::warn!(task_id = task.task_id, %error, "task completion failed");
                }
            }
        }
        Err(failure) => match database
            .fail_task(
                lease_id,
                failure.classification as i32,
                failure.detail,
                now_millis(),
            )
            .await
        {
            Ok(task_events) => events.publish_all(task_events),
            Err(error) => {
                tracing::warn!(task_id = task.task_id, %error, "task failure could not be persisted");
            }
        },
    }
}

#[derive(Debug)]
pub(crate) struct TaskExecutionError {
    classification: FailureClass,
    detail: String,
}

impl TaskExecutionError {
    pub(crate) fn transient(detail: String) -> Self {
        Self {
            classification: FailureClass::Transient,
            detail,
        }
    }

    pub(crate) fn deterministic(detail: impl Into<String>) -> Self {
        Self {
            classification: FailureClass::Deterministic,
            detail: detail.into(),
        }
    }
}

async fn execute_probe_artifact(
    database: &DbHandle,
    artifacts: &ArtifactHandle,
    sources: &SourceInspector,
    task: &LeasedTask,
) -> Result<ArtifactId, TaskExecutionError> {
    let source_id = task
        .source_id
        .as_ref()
        .ok_or_else(|| TaskExecutionError::deterministic("probe task omitted source id"))?;
    let source = database
        .get_source(source_id.clone())
        .await
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
    sources
        .verify(&source.observation)
        .await
        .map_err(|error| match error {
            SourceProbeError::SourceChanged => TaskExecutionError::deterministic("SOURCE_CHANGED"),
            SourceProbeError::InvalidPath(_) | SourceProbeError::ProbeFailed(_) => {
                TaskExecutionError::deterministic(error.to_string())
            }
            _ => TaskExecutionError::transient(error.to_string()),
        })?;
    let digest = source
        .source_fingerprint
        .strip_prefix("sha256:")
        .ok_or_else(|| TaskExecutionError::deterministic("source fingerprint is invalid"))?
        .parse::<Sha256Digest>()
        .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    let mut config = Map::new();
    config.insert(
        "ffmpeg_bom".to_owned(),
        Value::String(media::FFMPEG_BOM.to_owned()),
    );
    config.insert(
        "probe_algorithm".to_owned(),
        Value::String("clipmill.ffprobe.normalize.v1".to_owned()),
    );
    config.insert(
        "mapping_algorithm".to_owned(),
        Value::String("clipmill.source-map.mapping.v1".to_owned()),
    );
    let recipe = ArtifactRecipe::try_from_spec(RecipeSpec {
        kind: task.output_kind.clone(),
        source_fingerprint: digest,
        timebase: Timebase {
            num: 1,
            den: 90_000,
        },
        producer: Producer {
            stage: task.kind.clone(),
            implementation: task.implementation.clone(),
            model_digest: None,
        },
        inputs: Vec::new(),
        policy: NetworkPolicy::LocalLock,
        config,
        semantic_version: "clipmill.source_map.v1".to_owned(),
    })
    .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    match artifacts
        .prepare(recipe)
        .await
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?
    {
        PrepareOutcome::Hit(lease) => Ok(lease.artifact_id()),
        PrepareOutcome::InFlight { .. } => Err(TaskExecutionError::transient(
            "source-map artifact key is already in flight".to_owned(),
        )),
        PrepareOutcome::Miss(staging) => {
            let path = "source-map.json"
                .parse::<ArtifactPath>()
                .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
            let mut file = staging
                .create_file(&path)
                .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
            write_and_sync(&mut file, &source.source_map_json)
                .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
            drop(file);
            let lease = artifacts
                .commit(staging.id().clone(), vec![path], BTreeMap::new())
                .await
                .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
            Ok(lease.artifact_id())
        }
    }
}

#[allow(clippy::too_many_lines)]
async fn execute_device_artifact(
    database: &DbHandle,
    artifacts: &ArtifactHandle,
    profiler: &DeviceProfiler,
    task: &LeasedTask,
) -> Result<ArtifactId, TaskExecutionError> {
    let payload = DeviceProfilePayloadV1::decode(task.payload.as_slice())
        .map_err(|_| TaskExecutionError::deterministic("device profile payload is invalid"))?;
    if payload.key_version != DEVICE_PROFILE_KEY_VERSION || payload.measurement_generation == 0 {
        return Err(TaskExecutionError::deterministic(
            "device profile payload version or generation is invalid",
        ));
    }
    payload
        .hardware_fingerprint
        .parse::<ArtifactId>()
        .map_err(|_| TaskExecutionError::deterministic("device fingerprint is invalid"))?;
    let record = database
        .device_profile_for_job(task.job_id.clone())
        .await
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
    if record.hardware_fingerprint != payload.hardware_fingerprint
        || record.measurement_generation != payload.measurement_generation
    {
        return Err(TaskExecutionError::deterministic(
            "device job does not match its durable generation",
        ));
    }
    let profile_json = if let Some(profile_json) = record.profile_json {
        profile_json
    } else {
        let measured = profiler
            .measure(
                &payload.hardware_fingerprint,
                payload.measurement_generation,
            )
            .await
            .map_err(|error| match error {
                crate::device::DeviceProfileError::FingerprintMismatch => {
                    TaskExecutionError::deterministic("DEVICE_CHANGED")
                }
                _ => TaskExecutionError::transient(error.to_string()),
            })?;
        database
            .store_device_profile_json(task.job_id.clone(), measured, now_millis())
            .await
            .map_err(|error| TaskExecutionError::transient(error.to_string()))?
            .profile_json
            .ok_or_else(|| {
                TaskExecutionError::deterministic("stored device measurement is missing")
            })?
    };
    verify_profile(&profile_json, Some(&payload.hardware_fingerprint))
        .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    let digest = payload
        .hardware_fingerprint
        .strip_prefix("sha256:")
        .ok_or_else(|| TaskExecutionError::deterministic("device fingerprint is invalid"))?
        .parse::<Sha256Digest>()
        .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    let mut config = Map::new();
    config.insert(
        "measurement_generation".to_owned(),
        Value::Number(payload.measurement_generation.into()),
    );
    config.insert(
        "profile_algorithm".to_owned(),
        Value::String("clipmill.device-profile.measure.v1".to_owned()),
    );
    let recipe = ArtifactRecipe::try_from_spec(RecipeSpec {
        kind: task.output_kind.clone(),
        source_fingerprint: digest,
        timebase: Timebase {
            num: 1,
            den: 90_000,
        },
        producer: Producer {
            stage: task.kind.clone(),
            implementation: task.implementation.clone(),
            model_digest: None,
        },
        inputs: Vec::new(),
        policy: NetworkPolicy::LocalLock,
        config,
        semantic_version: "clipmill.device_profile.v1".to_owned(),
    })
    .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    match artifacts
        .prepare(recipe)
        .await
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?
    {
        PrepareOutcome::Hit(lease) => Ok(lease.artifact_id()),
        PrepareOutcome::InFlight { .. } => Err(TaskExecutionError::transient(
            "device profile artifact key is already in flight".to_owned(),
        )),
        PrepareOutcome::Miss(staging) => {
            let path = "profile.json"
                .parse::<ArtifactPath>()
                .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
            let mut file = staging
                .create_file(&path)
                .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
            write_and_sync(&mut file, profile_json.as_bytes())
                .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
            drop(file);
            let lease = artifacts
                .commit(staging.id().clone(), vec![path], BTreeMap::new())
                .await
                .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
            Ok(lease.artifact_id())
        }
    }
}

async fn execute_demo_artifact(
    artifacts: &ArtifactHandle,
    task: &LeasedTask,
) -> Result<ArtifactId, String> {
    let recipe = demo_recipe(task)?;
    match artifacts
        .prepare(recipe)
        .await
        .map_err(|error| error.to_string())?
    {
        PrepareOutcome::Hit(lease) => Ok(lease.artifact_id()),
        PrepareOutcome::InFlight { .. } => Err("artifact key is already in flight".to_owned()),
        PrepareOutcome::Miss(staging) => {
            let path = "result.json"
                .parse::<ArtifactPath>()
                .map_err(|error| error.to_string())?;
            let output = demo_output(task)?;
            let mut file = staging
                .create_file(&path)
                .map_err(|error| error.to_string())?;
            write_and_sync(&mut file, &output).map_err(|error| error.to_string())?;
            drop(file);
            let lease = artifacts
                .commit(staging.id().clone(), vec![path], BTreeMap::new())
                .await
                .map_err(|error| error.to_string())?;
            Ok(lease.artifact_id())
        }
    }
}

pub(crate) fn demo_recipe(task: &LeasedTask) -> Result<ArtifactRecipe, String> {
    let mut source_hasher = Sha256::new();
    source_hasher.update(b"clipmill.demo.source.v1\0");
    source_hasher.update(&task.payload);
    let source_fingerprint = Sha256Digest::from_bytes(source_hasher.finalize().into());
    let mut config = Map::new();
    config.insert("task_kind".to_owned(), Value::String(task.kind.clone()));
    ArtifactRecipe::try_from_spec(RecipeSpec {
        kind: task.output_kind.clone(),
        source_fingerprint,
        timebase: Timebase {
            num: 1,
            den: 90_000,
        },
        producer: Producer {
            stage: task.kind.clone(),
            implementation: task.implementation.clone(),
            model_digest: None,
        },
        inputs: task.input_artifact_ids.clone(),
        policy: NetworkPolicy::LocalLock,
        config,
        semantic_version: "1.0.0".to_owned(),
    })
    .map_err(|error| error.to_string())
}

pub(crate) fn demo_output(task: &LeasedTask) -> Result<Vec<u8>, String> {
    serde_json_canonicalizer::to_vec(&json!({
        "inputs": task.input_artifact_ids.iter().map(ToString::to_string).collect::<Vec<_>>(),
        "kind": task.kind,
        "payload_sha256": format!("sha256:{}", Sha256Digest::from_bytes(Sha256::digest(&task.payload).into())),
    }))
    .map_err(|error| error.to_string())
}

fn write_and_sync(file: &mut File, bytes: &[u8]) -> std::io::Result<()> {
    file.write_all(bytes)?;
    file.sync_all()
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

pub(crate) fn is_terminal_job(state: i32) -> bool {
    state == JobState::Succeeded as i32
        || state == JobState::Failed as i32
        || state == JobState::Cancelled as i32
}

#[cfg(test)]
mod resource_tests {
    use std::collections::BTreeSet;

    use super::{ResourceCapacity, ResourceDeclaration};

    #[test]
    fn measured_backend_availability_controls_accelerator_admission() {
        let backends = BTreeSet::from(["videotoolbox".to_owned()]);
        let mut capacity =
            ResourceCapacity::measured(4, 1024 * 1024 * 1024).with_available_backends(&backends);
        let mut resources = ResourceDeclaration::demo();
        resources.accelerator_class = "vaapi".to_owned();
        assert!(!capacity.reserve(&resources));
        resources.accelerator_class = "videotoolbox".to_owned();
        assert!(capacity.reserve(&resources));
        capacity.release(&resources);
        assert!(capacity.reserve(&resources));
    }
}
