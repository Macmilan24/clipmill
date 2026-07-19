use std::{
    collections::BTreeMap,
    path::Path,
    thread,
    time::{Duration, SystemTime},
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
                    while let Some(command) = receiver.blocking_recv() {
                        match command {
                            Command::Prepare { recipe, reply } => {
                                let _reply = reply.send(store.prepare(recipe));
                            }
                            Command::Commit {
                                staging_id,
                                paths,
                                quality,
                                reply,
                            } => {
                                let _reply = reply.send(store.commit(&staging_id, paths, quality));
                            }
                            Command::Open { artifact_id, reply } => {
                                let _reply = reply.send(store.open(artifact_id));
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
