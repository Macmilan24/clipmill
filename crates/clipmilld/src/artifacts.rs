use std::{
    collections::BTreeMap,
    path::Path,
    thread,
    time::{Duration, Instant, SystemTime},
};

use clipmill_artifacts::{
    ArtifactError, ArtifactLease, ArtifactPath, ArtifactRecipe, ArtifactStore, GcReport,
    PrepareOutcome, RecoveryReport,
};
use clipmill_core::{ArtifactId, ProjectId, StagingId};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

use crate::{
    DaemonError,
    db::{DbHandle, StoreError},
};

const COMMAND_CAPACITY: usize = 64;

#[derive(Clone, Debug)]
struct ArtifactLogContext {
    artifact_id: Option<ArtifactId>,
    kind: String,
    stage: String,
}

fn latency_millis(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn log_prepare(
    context: &ArtifactLogContext,
    result: &Result<PrepareOutcome, ArtifactError>,
    started: Instant,
) {
    let artifact_id = context
        .artifact_id
        .map_or_else(|| "unavailable".to_owned(), |value| value.to_string());
    let latency_ms = latency_millis(started);
    if let Ok(outcome) = result {
        let cache_result = match outcome {
            PrepareOutcome::Hit(_) => "hit",
            PrepareOutcome::Miss(_) => "miss",
            PrepareOutcome::InFlight { .. } => "in-flight",
        };
        tracing::info!(
            artifact_id,
            kind = context.kind,
            stage = context.stage,
            operation = "prepare",
            latency_ms,
            cache_result,
            result = "ok",
            "artifact operation"
        );
    } else {
        tracing::warn!(
            artifact_id,
            kind = context.kind,
            stage = context.stage,
            operation = "prepare",
            latency_ms,
            cache_result = "error",
            result = "error",
            "artifact operation failed"
        );
    }
}

fn log_commit(
    context: Option<&ArtifactLogContext>,
    result: &Result<ArtifactLease, ArtifactError>,
    started: Instant,
) {
    let latency_ms = latency_millis(started);
    match result {
        Ok(lease) => tracing::info!(
            artifact_id = %lease.artifact_id(),
            kind = lease.kind(),
            stage = lease.stage(),
            operation = "commit",
            latency_ms,
            cache_result = "published",
            result = "ok",
            "artifact operation"
        ),
        Err(error) => {
            let artifact_id = context
                .and_then(|value| value.artifact_id)
                .map_or_else(|| "unavailable".to_owned(), |value| value.to_string());
            let kind = context.map_or("unavailable", |value| value.kind.as_str());
            let stage = context.map_or("unavailable", |value| value.stage.as_str());
            let cache_result = if matches!(error, ArtifactError::NonDeterministicOutput(_)) {
                "nondeterministic"
            } else {
                "error"
            };
            tracing::warn!(
                artifact_id,
                kind,
                stage,
                operation = "commit",
                latency_ms,
                cache_result,
                result = "error",
                "artifact operation failed"
            );
        }
    }
}

fn log_open(
    artifact_id: ArtifactId,
    result: &Result<ArtifactLease, ArtifactError>,
    started: Instant,
) {
    let latency_ms = latency_millis(started);
    if let Ok(lease) = result {
        tracing::info!(
            artifact_id = %artifact_id,
            kind = lease.kind(),
            stage = lease.stage(),
            operation = "open",
            latency_ms,
            cache_result = "hit",
            result = "ok",
            "artifact operation"
        );
    } else {
        tracing::warn!(
            artifact_id = %artifact_id,
            kind = "unavailable",
            stage = "unavailable",
            operation = "open",
            latency_ms,
            cache_result = "error",
            result = "error",
            "artifact operation failed"
        );
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ArtifactHandle {
    sender: mpsc::Sender<Command>,
}

#[derive(Debug)]
pub(crate) struct ArtifactActor {
    handle: ArtifactHandle,
    thread: Option<thread::JoinHandle<()>>,
}

impl ArtifactActor {
    pub(crate) fn start(path: &Path) -> Result<(Self, RecoveryReport), DaemonError> {
        let (sender, mut receiver) = mpsc::channel(COMMAND_CAPACITY);
        let (ready_sender, ready_receiver) = std::sync::mpsc::sync_channel(1);
        let store_path = path.to_path_buf();
        let actor_thread = thread::Builder::new()
            .name("clipmill-artifacts".to_owned())
            .spawn(move || match ArtifactStore::initialize(&store_path) {
                Ok((mut store, recovery)) => {
                    let _ready = ready_sender.send(Ok(recovery));
                    let mut staging_context = BTreeMap::new();
                    while let Some(command) = receiver.blocking_recv() {
                        match command {
                            Command::Prepare { recipe, reply } => {
                                let started = Instant::now();
                                let context = ArtifactLogContext {
                                    artifact_id: recipe.artifact_id().ok(),
                                    kind: recipe.kind().to_owned(),
                                    stage: recipe.producer().stage.clone(),
                                };
                                let result = store.prepare(recipe);
                                if let Ok(PrepareOutcome::Miss(staging)) = &result {
                                    staging_context.insert(staging.id().clone(), context.clone());
                                }
                                log_prepare(&context, &result, started);
                                let _reply = reply.send(result);
                            }
                            Command::Commit {
                                staging_id,
                                paths,
                                quality,
                                reply,
                            } => {
                                let started = Instant::now();
                                let context = staging_context.remove(&staging_id);
                                let result = store.commit(&staging_id, paths, quality);
                                log_commit(context.as_ref(), &result, started);
                                let _reply = reply.send(result);
                            }
                            Command::Abandon { staging_id, reply } => {
                                staging_context.remove(&staging_id);
                                let _reply = reply.send(store.abandon(&staging_id));
                            }
                            Command::Open { artifact_id, reply } => {
                                let started = Instant::now();
                                let result = store.open(artifact_id);
                                log_open(artifact_id, &result, started);
                                let _reply = reply.send(result);
                            }
                            Command::Collect {
                                roots,
                                now,
                                grace,
                                reply,
                            } => {
                                let _reply = reply.send(store.collect_garbage(roots, now, grace));
                            }
                            Command::Shutdown { reply } => {
                                let _reply = reply.send(());
                                break;
                            }
                        }
                    }
                }
                Err(error) => {
                    let _ready = ready_sender.send(Err(error));
                }
            })
            .map_err(|source| DaemonError::io(path, source))?;

        match ready_receiver.recv() {
            Ok(Ok(recovery)) => Ok((
                Self {
                    handle: ArtifactHandle { sender },
                    thread: Some(actor_thread),
                },
                recovery,
            )),
            Ok(Err(error)) => {
                let _joined = actor_thread.join();
                Err(error.into())
            }
            Err(_) => {
                let _joined = actor_thread.join();
                Err(DaemonError::ArtifactActorStopped)
            }
        }
    }

    pub(crate) fn handle(&self) -> ArtifactHandle {
        self.handle.clone()
    }

    pub(crate) async fn shutdown(mut self) -> Result<(), DaemonError> {
        let (reply, received) = oneshot::channel();
        self.handle
            .sender
            .send(Command::Shutdown { reply })
            .await
            .map_err(|_| DaemonError::ArtifactActorStopped)?;
        received
            .await
            .map_err(|_| DaemonError::ArtifactActorStopped)?;
        if let Some(actor_thread) = self.thread.take() {
            tokio::task::spawn_blocking(move || actor_thread.join())
                .await
                .map_err(|_| DaemonError::ArtifactActorPanicked)?
                .map_err(|_| DaemonError::ArtifactActorPanicked)?;
        }
        Ok(())
    }
}

impl ArtifactHandle {
    pub(crate) async fn prepare(
        &self,
        recipe: ArtifactRecipe,
    ) -> Result<PrepareOutcome, ArtifactServiceError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::Prepare { recipe, reply })
            .await
            .map_err(|_| ArtifactServiceError::Stopped)?;
        received
            .await
            .map_err(|_| ArtifactServiceError::Stopped)?
            .map_err(Into::into)
    }

    pub(crate) async fn commit(
        &self,
        staging_id: StagingId,
        paths: Vec<ArtifactPath>,
        quality: BTreeMap<String, f64>,
    ) -> Result<ArtifactLease, ArtifactServiceError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::Commit {
                staging_id,
                paths,
                quality,
                reply,
            })
            .await
            .map_err(|_| ArtifactServiceError::Stopped)?;
        received
            .await
            .map_err(|_| ArtifactServiceError::Stopped)?
            .map_err(Into::into)
    }

    pub(crate) async fn abandon(
        &self,
        staging_id: StagingId,
    ) -> Result<bool, ArtifactServiceError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::Abandon { staging_id, reply })
            .await
            .map_err(|_| ArtifactServiceError::Stopped)?;
        received
            .await
            .map_err(|_| ArtifactServiceError::Stopped)?
            .map_err(Into::into)
    }

    pub(crate) async fn open(
        &self,
        artifact_id: ArtifactId,
    ) -> Result<ArtifactLease, ArtifactServiceError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::Open { artifact_id, reply })
            .await
            .map_err(|_| ArtifactServiceError::Stopped)?;
        received
            .await
            .map_err(|_| ArtifactServiceError::Stopped)?
            .map_err(Into::into)
    }

    pub(crate) async fn collect(
        &self,
        roots: Vec<ArtifactId>,
        now: SystemTime,
        grace: Duration,
    ) -> Result<GcReport, ArtifactServiceError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::Collect {
                roots,
                now,
                grace,
                reply,
            })
            .await
            .map_err(|_| ArtifactServiceError::Stopped)?;
        received
            .await
            .map_err(|_| ArtifactServiceError::Stopped)?
            .map_err(Into::into)
    }
}

/// In-process daemon interface used by the scheduler and lifecycle tests.
#[derive(Clone, Debug)]
pub struct ArtifactCoordinator {
    artifacts: ArtifactHandle,
    database: DbHandle,
}

impl ArtifactCoordinator {
    pub(crate) fn new(artifacts: ArtifactHandle, database: DbHandle) -> Self {
        Self {
            artifacts,
            database,
        }
    }

    pub async fn prepare(
        &self,
        recipe: ArtifactRecipe,
    ) -> Result<PrepareOutcome, ArtifactServiceError> {
        self.artifacts.prepare(recipe).await
    }

    pub async fn open(
        &self,
        artifact_id: ArtifactId,
    ) -> Result<ArtifactLease, ArtifactServiceError> {
        self.artifacts.open(artifact_id).await
    }

    /// Return the distinct artifact roots currently published by projects.
    ///
    /// This is an in-process scheduler/lifecycle interface; it is deliberately
    /// not exposed through the W3 protobuf control API.
    pub async fn artifact_roots(&self) -> Result<Vec<ArtifactId>, ArtifactServiceError> {
        self.database
            .list_artifact_roots()
            .await
            .map_err(ArtifactServiceError::from)
    }

    /// Commit the immutable object first, then publish its project root in
    /// SQLite. The returned success is the durability acknowledgement.
    pub async fn publish_project(
        &self,
        project_id: ProjectId,
        staging_id: StagingId,
        paths: Vec<ArtifactPath>,
        quality: BTreeMap<String, f64>,
    ) -> Result<ArtifactLease, ArtifactServiceError> {
        let lease = self.artifacts.commit(staging_id, paths, quality).await?;
        self.database
            .attach_artifact_root(project_id, lease.artifact_id())
            .await
            .map_err(ArtifactServiceError::from)?;
        Ok(lease)
    }

    /// Attach an already committed cache hit to another project without
    /// duplicating payload bytes.
    pub async fn attach_existing_project(
        &self,
        project_id: ProjectId,
        artifact_id: ArtifactId,
    ) -> Result<ArtifactLease, ArtifactServiceError> {
        let lease = self.artifacts.open(artifact_id).await?;
        self.database
            .attach_artifact_root(project_id, artifact_id)
            .await
            .map_err(ArtifactServiceError::from)?;
        Ok(lease)
    }
}

#[derive(Debug, Error)]
pub enum ArtifactServiceError {
    #[error("artifact actor stopped")]
    Stopped,
    #[error("artifact operation failed: {0}")]
    Artifact(#[from] ArtifactError),
    #[error("database operation failed: {0}")]
    Database(String),
}

impl From<StoreError> for ArtifactServiceError {
    fn from(value: StoreError) -> Self {
        Self::Database(value.to_string())
    }
}

#[derive(Debug)]
enum Command {
    Prepare {
        recipe: ArtifactRecipe,
        reply: oneshot::Sender<Result<PrepareOutcome, ArtifactError>>,
    },
    Commit {
        staging_id: StagingId,
        paths: Vec<ArtifactPath>,
        quality: BTreeMap<String, f64>,
        reply: oneshot::Sender<Result<ArtifactLease, ArtifactError>>,
    },
    Abandon {
        staging_id: StagingId,
        reply: oneshot::Sender<Result<bool, ArtifactError>>,
    },
    Open {
        artifact_id: ArtifactId,
        reply: oneshot::Sender<Result<ArtifactLease, ArtifactError>>,
    },
    Collect {
        roots: Vec<ArtifactId>,
        now: SystemTime,
        grace: Duration,
        reply: oneshot::Sender<Result<GcReport, ArtifactError>>,
    },
    Shutdown {
        reply: oneshot::Sender<()>,
    },
}
