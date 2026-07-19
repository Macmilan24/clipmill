use std::{
    collections::BTreeMap,
    fs::File,
    io::Write,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use clipmill_artifacts::{
    ArtifactPath, ArtifactRecipe, NetworkPolicy, PrepareOutcome, Producer, RecipeSpec, Timebase,
};
use clipmill_contracts::proto::{
    ipc::v1::{self, JobState},
    worker::v1::{FailureClass, ProgressUnits},
};
use clipmill_core::{ArtifactId, JobId, LeaseId, ProjectId, Sha256Digest, TaskId};
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
};

pub(crate) const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
pub(crate) const LEASE_TTL: Duration = Duration::from_secs(15);
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
}

impl ResourceCapacity {
    pub(crate) fn w4_builtin() -> Self {
        let cpu_threads = std::thread::available_parallelism()
            .map_or(1, std::num::NonZeroUsize::get)
            .min(MAX_BUILTIN_TASKS);
        Self {
            cpu_threads: u32::try_from(cpu_threads).unwrap_or(u32::MAX),
            ram_bytes: 512 * 1024 * 1024,
            disk_bytes: 512 * 1024 * 1024,
        }
    }

    fn reserve(&mut self, resources: &ResourceDeclaration) -> bool {
        if resources.accelerator_class.is_empty()
            && resources.vram_bytes == 0
            && resources.network_policy == "local-lock"
            && resources.cpu_threads <= self.cpu_threads
            && resources.ram_bytes <= self.ram_bytes
            && resources.disk_bytes <= self.disk_bytes
        {
            self.cpu_threads -= resources.cpu_threads;
            self.ram_bytes -= resources.ram_bytes;
            self.disk_bytes -= resources.disk_bytes;
            true
        } else {
            false
        }
    }

    fn release(&mut self, resources: &ResourceDeclaration) {
        self.cpu_threads = self.cpu_threads.saturating_add(resources.cpu_threads);
        self.ram_bytes = self.ram_bytes.saturating_add(resources.ram_bytes);
        self.disk_bytes = self.disk_bytes.saturating_add(resources.disk_bytes);
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
}

impl SchedulerHandle {
    pub(crate) fn notify(&self) {
        self.notify.notify_one();
    }
}

impl Scheduler {
    pub(crate) fn start(
        database: DbHandle,
        artifacts: ArtifactHandle,
        events: EventHub,
        daemon_epoch: String,
    ) -> Self {
        debug_assert!(LEASE_TTL >= HEARTBEAT_INTERVAL.saturating_mul(3));
        let notify = Arc::new(Notify::new());
        let handle = SchedulerHandle {
            notify: Arc::clone(&notify),
        };
        let (stopped, stop) = oneshot::channel();
        let task = tokio::spawn(run_scheduler(
            database,
            artifacts,
            events,
            daemon_epoch,
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

async fn run_scheduler(
    database: DbHandle,
    artifacts: ArtifactHandle,
    events: EventHub,
    daemon_epoch: String,
    notify: Arc<Notify>,
    mut stop: oneshot::Receiver<()>,
) {
    let mut schedule = interval(SCHEDULER_TICK);
    schedule.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut running = JoinSet::new();
    let mut available_capacity = ResourceCapacity::w4_builtin();
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
        while running.len() < MAX_BUILTIN_TASKS && available_capacity.cpu_threads > 0 {
            let now = now_millis();
            let lease = LeaseId::new().to_string();
            let leased = database
                .lease_next_task(
                    lease,
                    daemon_epoch.clone(),
                    now,
                    now.saturating_add(duration_millis(LEASE_TTL)),
                    available_capacity,
                )
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
            let notify = Arc::clone(&notify);
            running.spawn(async move {
                execute_task(database, artifacts, events, task).await;
                notify.notify_one();
                resources
            });
        }
    }

    running.abort_all();
    while running.join_next().await.is_some() {}
}

async fn execute_task(
    database: DbHandle,
    artifacts: ArtifactHandle,
    events: EventHub,
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
    let work = async {
        if let Ok(delay) = std::env::var("CLIPMILL_W4_STEP_DELAY_MS")
            && let Ok(delay) = delay.parse::<u64>()
        {
            tokio::time::sleep(Duration::from_millis(delay.min(30_000))).await;
        }
        execute_demo_artifact(&artifacts, &task).await
    };
    tokio::pin!(work);
    let mut heartbeat = interval(HEARTBEAT_INTERVAL);
    heartbeat.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let outcome = loop {
        tokio::select! {
            result = &mut work => break Some(result),
            _ = heartbeat.tick() => {
                let now = now_millis();
                if database
                    .heartbeat_task(
                        lease_id.clone(),
                        now,
                        now.saturating_add(duration_millis(LEASE_TTL)),
                    )
                    .await
                    .is_err()
                {
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
        Err(detail) => match database
            .fail_task(
                lease_id,
                FailureClass::Transient as i32,
                detail,
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

async fn execute_demo_artifact(
    artifacts: &ArtifactHandle,
    task: &LeasedTask,
) -> Result<ArtifactId, String> {
    let mut source_hasher = Sha256::new();
    source_hasher.update(b"clipmill.demo.source.v1\0");
    source_hasher.update(&task.payload);
    let source_fingerprint = Sha256Digest::from_bytes(source_hasher.finalize().into());
    let mut config = Map::new();
    config.insert("task_kind".to_owned(), Value::String(task.kind.clone()));
    let recipe = ArtifactRecipe::try_from_spec(RecipeSpec {
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
    .map_err(|error| error.to_string())?;
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
            let output = serde_json_canonicalizer::to_vec(&json!({
                "inputs": task.input_artifact_ids.iter().map(ToString::to_string).collect::<Vec<_>>(),
                "kind": task.kind,
                "payload_sha256": format!("sha256:{}", Sha256Digest::from_bytes(Sha256::digest(&task.payload).into())),
            }))
            .map_err(|error| error.to_string())?;
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
