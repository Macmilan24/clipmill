use std::{
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use clipmill_contracts::proto::ipc::v1::{
    CreateProjectResponse, DeleteProjectResponse, Project, Response, response,
};
use clipmill_core::{ArtifactId, ProjectId};
use prost::Message;
use rusqlite::{
    Connection, OpenFlags, OptionalExtension, Transaction, TransactionBehavior, backup::Backup,
    params,
};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};
use ulid::Ulid;

use crate::DaemonError;
use crate::jobs::{
    EventFilter, JobPlan, JobRecord, LeaseRequest, LeaseSelection, TaskCompletion, TaskEventRecord,
};

mod job_store;
pub(crate) use job_store::MutationResult;
mod source_store;
pub(crate) use source_store::SourceRecord;
mod device_store;
pub(crate) use device_store::{BeginDeviceProfile, DeviceProfileRecord, DeviceProfileState};
mod edit_store;
pub(crate) use edit_store::{EditCommandRecord, EditDocRecord};
mod decision_store;
pub(crate) use decision_store::{Decision, DecisionRecord};

const APPLICATION_ID: i64 = 0x434C_504D; // "CLPM"
const SCHEMA_VERSION: i64 = 9;
const SQLITE_MIN_VERSION: i32 = 3_051_003;
const COMMAND_CAPACITY: usize = 128;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectRecord {
    pub project_id: String,
    pub name: String,
    pub created_unix_millis: u64,
}

impl From<ProjectRecord> for Project {
    fn from(record: ProjectRecord) -> Self {
        Self {
            project_id: record.project_id,
            name: record.name,
            created_unix_millis: record.created_unix_millis,
        }
    }
}

#[derive(Debug, Error)]
pub(crate) enum StoreError {
    #[error("request id was already used with a different request body")]
    Conflict,
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("database contains invalid data: {0}")]
    InvalidData(&'static str),
    #[error("requested record was not found")]
    NotFound,
    #[error("database actor stopped")]
    Stopped,
}

#[derive(Clone, Debug)]
pub(crate) struct DbHandle {
    sender: mpsc::Sender<Command>,
}

#[derive(Debug)]
pub(crate) struct DbActor {
    handle: DbHandle,
    thread: Option<thread::JoinHandle<()>>,
}

impl DbActor {
    #[allow(clippy::too_many_lines)]
    pub(crate) fn start(path: &Path, backups_dir: &Path) -> Result<Self, DaemonError> {
        let (sender, mut receiver) = mpsc::channel(COMMAND_CAPACITY);
        let (ready_sender, ready_receiver) = std::sync::mpsc::sync_channel(1);
        let database_path = path.to_path_buf();
        let database_backups = backups_dir.to_path_buf();
        let actor_thread = thread::Builder::new()
            .name("clipmill-sqlite".to_owned())
            .spawn(move || {
                let connection = open_database(&database_path, &database_backups);
                match connection {
                    Ok(mut connection) => {
                        let _result = ready_sender.send(Ok(()));
                        while let Some(command) = receiver.blocking_recv() {
                            match command {
                                Command::Create {
                                    request_id,
                                    request_hash,
                                    project,
                                    reply,
                                } => {
                                    let _result = reply.send(create_project(
                                        &mut connection,
                                        &request_id,
                                        &request_hash,
                                        &project,
                                    ));
                                }
                                Command::Delete {
                                    request_id,
                                    request_hash,
                                    project_id,
                                    completed_unix_millis,
                                    reply,
                                } => {
                                    let _result = reply.send(delete_project(
                                        &mut connection,
                                        &request_id,
                                        &request_hash,
                                        &project_id,
                                        completed_unix_millis,
                                    ));
                                }
                                Command::Get { project_id, reply } => {
                                    let _result = reply.send(get_project(&connection, &project_id));
                                }
                                Command::List { reply } => {
                                    let _result = reply.send(list_projects(&connection));
                                }
                                Command::AttachArtifactRoot {
                                    project_id,
                                    artifact_id,
                                    reply,
                                } => {
                                    let _result = reply.send(attach_artifact_root(
                                        &mut connection,
                                        &project_id,
                                        &artifact_id,
                                    ));
                                }
                                Command::ListArtifactRoots { reply } => {
                                    let _result = reply.send(list_artifact_roots(&connection));
                                }
                                Command::ArtifactIsProjectOutput {
                                    project_id,
                                    artifact_id,
                                    reply,
                                } => {
                                    let _result =
                                        reply.send(job_store::artifact_is_project_output(
                                            &connection,
                                            &project_id,
                                            &artifact_id,
                                        ));
                                }
                                Command::SubmitJob {
                                    request_id,
                                    request_hash,
                                    plan,
                                    reply,
                                } => {
                                    let _result = reply.send(job_store::submit_job(
                                        &mut connection,
                                        &request_id,
                                        &request_hash,
                                        &plan,
                                    ));
                                }
                                Command::GetJob { job_id, reply } => {
                                    let _result =
                                        reply.send(job_store::get_job(&connection, &job_id));
                                }
                                Command::ListJobs { project_id, reply } => {
                                    let _result =
                                        reply.send(job_store::list_jobs(&connection, &project_id));
                                }
                                Command::SetClipDecision {
                                    project_id,
                                    source_id,
                                    candidate_id,
                                    decision,
                                    now_unix_millis,
                                    reply,
                                } => {
                                    let _result = reply.send(
                                        decision_store::set(
                                            &connection,
                                            &project_id,
                                            &source_id,
                                            &candidate_id,
                                            decision,
                                            now_unix_millis,
                                        )
                                        .map_err(StoreError::from),
                                    );
                                }
                                Command::ListEditDocs { project_id, reply } => {
                                    let _result = reply
                                        .send(edit_store::list_edit_docs(&connection, &project_id));
                                }
                                Command::ListClipDecisions {
                                    project_id,
                                    source_id,
                                    reply,
                                } => {
                                    let _result = reply.send(
                                        decision_store::list(&connection, &project_id, &source_id)
                                            .map_err(StoreError::from),
                                    );
                                }
                                Command::CancelJob {
                                    request_id,
                                    request_hash,
                                    job_id,
                                    completed_unix_millis,
                                    reply,
                                } => {
                                    let _result = reply.send(job_store::cancel_job(
                                        &mut connection,
                                        &request_id,
                                        &request_hash,
                                        &job_id,
                                        completed_unix_millis,
                                    ));
                                }
                                Command::RecoverJobs {
                                    daemon_epoch,
                                    recovered_unix_millis,
                                    reply,
                                } => {
                                    let _result = reply.send(job_store::recover_jobs(
                                        &mut connection,
                                        &daemon_epoch,
                                        recovered_unix_millis,
                                    ));
                                }
                                Command::LeaseNextTask { request, reply } => {
                                    let _result =
                                        reply.send(job_store::lease_next_task_for_worker(
                                            &mut connection,
                                            &request,
                                        ));
                                }
                                Command::HeartbeatTask {
                                    lease_id,
                                    now_unix_millis,
                                    expires_unix_millis,
                                    progress,
                                    reply,
                                } => {
                                    let _result =
                                        reply.send(job_store::heartbeat_task_with_progress(
                                            &mut connection,
                                            &lease_id,
                                            now_unix_millis,
                                            expires_unix_millis,
                                            progress.as_ref(),
                                        ));
                                }
                                Command::ReplayTaskCompletion {
                                    lease_id,
                                    worker_id,
                                    completion_hash,
                                    reply,
                                } => {
                                    let _result = reply.send(job_store::replay_task_completion(
                                        &connection,
                                        &lease_id,
                                        &worker_id,
                                        &completion_hash,
                                    ));
                                }
                                Command::CompleteTask {
                                    lease_id,
                                    artifact_id,
                                    completion_hash,
                                    completion_response,
                                    completed_unix_millis,
                                    reply,
                                } => {
                                    let _result = reply.send(job_store::complete_task(
                                        &mut connection,
                                        &lease_id,
                                        artifact_id,
                                        &completion_hash,
                                        &completion_response,
                                        completed_unix_millis,
                                    ));
                                }
                                Command::FailTask {
                                    lease_id,
                                    failure_class,
                                    detail,
                                    failed_unix_millis,
                                    reply,
                                } => {
                                    let _result = reply.send(job_store::fail_task(
                                        &mut connection,
                                        &lease_id,
                                        failure_class,
                                        &detail,
                                        failed_unix_millis,
                                    ));
                                }
                                Command::CompleteFailedTask {
                                    lease_id,
                                    failure_class,
                                    detail,
                                    completion_hash,
                                    completion_response,
                                    completed_unix_millis,
                                    reply,
                                } => {
                                    let _result = reply.send(job_store::complete_failed_task(
                                        &mut connection,
                                        &lease_id,
                                        failure_class,
                                        &detail,
                                        &completion_hash,
                                        &completion_response,
                                        completed_unix_millis,
                                    ));
                                }
                                Command::ExpireTaskLeases {
                                    now_unix_millis,
                                    daemon_epoch,
                                    reply,
                                } => {
                                    let _result = reply.send(job_store::expire_task_leases(
                                        &mut connection,
                                        now_unix_millis,
                                        &daemon_epoch,
                                    ));
                                }
                                Command::CurrentEventId { reply } => {
                                    let _result =
                                        reply.send(job_store::current_event_id(&connection));
                                }
                                Command::ListEvents {
                                    after_event_id,
                                    filter,
                                    reply,
                                } => {
                                    let _result = reply.send(job_store::list_events(
                                        &connection,
                                        after_event_id,
                                        &filter,
                                    ));
                                }
                                Command::FindSourceObservation {
                                    project_id,
                                    observation,
                                    reply,
                                } => {
                                    let _result = reply.send(source_store::find_observation(
                                        &connection,
                                        &project_id,
                                        &observation,
                                    ));
                                }
                                Command::RegisterSource {
                                    request_id,
                                    request_hash,
                                    project_id,
                                    source_id,
                                    inspection,
                                    created_unix_millis,
                                    reply,
                                } => {
                                    let _result = reply.send(source_store::register_source(
                                        &mut connection,
                                        &request_id,
                                        &request_hash,
                                        &project_id,
                                        &source_id,
                                        &inspection,
                                        created_unix_millis,
                                    ));
                                }
                                Command::RememberSourceHit {
                                    request_id,
                                    request_hash,
                                    source,
                                    completed_unix_millis,
                                    reply,
                                } => {
                                    let _result =
                                        reply.send(source_store::remember_observation_hit(
                                            &mut connection,
                                            &request_id,
                                            &request_hash,
                                            &source,
                                            completed_unix_millis,
                                        ));
                                }
                                Command::GetSource { source_id, reply } => {
                                    let _result = reply
                                        .send(source_store::get_source(&connection, &source_id));
                                }
                                Command::ListSources { project_id, reply } => {
                                    let _result = reply
                                        .send(source_store::list_sources(&connection, &project_id));
                                }
                                Command::LatestSourceJobArtifact {
                                    source_id,
                                    kind,
                                    reply,
                                } => {
                                    let _result =
                                        reply.send(job_store::latest_source_job_artifact(
                                            &connection,
                                            &source_id,
                                            &kind,
                                        ));
                                }
                                Command::CreateEditDoc {
                                    request_id,
                                    request_hash,
                                    project_id,
                                    document_json,
                                    now_unix_millis,
                                    reply,
                                } => {
                                    let _result = reply.send(edit_store::create_edit_doc(
                                        &mut connection,
                                        &request_id,
                                        &request_hash,
                                        &project_id,
                                        &document_json,
                                        now_unix_millis,
                                    ));
                                }
                                Command::ApplyEdit {
                                    request_id,
                                    request_hash,
                                    doc_id,
                                    expected_revision,
                                    command_json,
                                    now_unix_millis,
                                    reply,
                                } => {
                                    let _result = reply.send(edit_store::apply_edit_command(
                                        &mut connection,
                                        &request_id,
                                        &request_hash,
                                        &doc_id,
                                        expected_revision,
                                        &command_json,
                                        now_unix_millis,
                                    ));
                                }
                                Command::GetEditDoc { doc_id, reply } => {
                                    let _result =
                                        reply.send(edit_store::get_edit_doc(&connection, &doc_id));
                                }
                                Command::GetEditLog { doc_id, reply } => {
                                    let _result =
                                        reply.send(edit_store::get_edit_log(&connection, &doc_id));
                                }
                                Command::BeginDeviceProfile {
                                    request_id,
                                    request_hash,
                                    hardware_fingerprint,
                                    remeasure,
                                    now_unix_millis,
                                    reply,
                                } => {
                                    let _result = reply.send(device_store::begin_device_profile(
                                        &mut connection,
                                        &request_id,
                                        &request_hash,
                                        &hardware_fingerprint,
                                        remeasure,
                                        now_unix_millis,
                                    ));
                                }
                                Command::DeviceProfileForJob { job_id, reply } => {
                                    let _result = reply
                                        .send(device_store::profile_for_job(&connection, &job_id));
                                }
                                Command::CurrentDeviceProfile {
                                    hardware_fingerprint,
                                    reply,
                                } => {
                                    let _result = reply.send(device_store::current_profile(
                                        &connection,
                                        &hardware_fingerprint,
                                    ));
                                }
                                Command::StoreDeviceProfileJson {
                                    job_id,
                                    profile_json,
                                    now_unix_millis,
                                    reply,
                                } => {
                                    let _result = reply.send(device_store::store_profile_json(
                                        &mut connection,
                                        &job_id,
                                        &profile_json,
                                        now_unix_millis,
                                    ));
                                }
                                Command::FinishDeviceProfileRequest {
                                    request_id,
                                    request_hash,
                                    artifact_id,
                                    response,
                                    now_unix_millis,
                                    reply,
                                } => {
                                    let _result = reply.send(device_store::finish_device_request(
                                        &mut connection,
                                        &request_id,
                                        &request_hash,
                                        artifact_id,
                                        &response,
                                        now_unix_millis,
                                    ));
                                }
                                Command::Shutdown { reply } => {
                                    let _result = reply.send(());
                                    break;
                                }
                            }
                        }
                    }
                    Err(error) => {
                        let _result = ready_sender.send(Err(error));
                    }
                }
            })
            .map_err(|source| DaemonError::io(path, source))?;

        match ready_receiver.recv() {
            Ok(Ok(())) => Ok(Self {
                handle: DbHandle { sender },
                thread: Some(actor_thread),
            }),
            Ok(Err(error)) => {
                let _result = actor_thread.join();
                Err(error)
            }
            Err(_) => {
                let _result = actor_thread.join();
                Err(DaemonError::DatabaseActorStopped)
            }
        }
    }

    pub(crate) fn handle(&self) -> DbHandle {
        self.handle.clone()
    }

    pub(crate) async fn shutdown(mut self) -> Result<(), DaemonError> {
        let (reply, received) = oneshot::channel();
        self.handle
            .sender
            .send(Command::Shutdown { reply })
            .await
            .map_err(|_| DaemonError::DatabaseActorStopped)?;
        received
            .await
            .map_err(|_| DaemonError::DatabaseActorStopped)?;

        if let Some(actor_thread) = self.thread.take() {
            tokio::task::spawn_blocking(move || actor_thread.join())
                .await
                .map_err(|_| DaemonError::DatabaseActorPanicked)?
                .map_err(|_| DaemonError::DatabaseActorPanicked)?;
        }
        Ok(())
    }
}

impl DbHandle {
    pub(crate) async fn create_project(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        project: ProjectRecord,
    ) -> Result<Vec<u8>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::Create {
                request_id,
                request_hash,
                project,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn delete_project(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        project_id: String,
        completed_unix_millis: u64,
    ) -> Result<Vec<u8>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::Delete {
                request_id,
                request_hash,
                project_id,
                completed_unix_millis,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn get_project(
        &self,
        project_id: String,
    ) -> Result<ProjectRecord, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::Get { project_id, reply })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn list_projects(&self) -> Result<Vec<ProjectRecord>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::List { reply })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn attach_artifact_root(
        &self,
        project_id: ProjectId,
        artifact_id: ArtifactId,
    ) -> Result<(), StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::AttachArtifactRoot {
                project_id: project_id.to_string(),
                artifact_id: artifact_id.to_string(),
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    /// Whether the project may be shown this artifact.
    pub(crate) async fn artifact_is_project_output(
        &self,
        project_id: String,
        artifact_id: String,
    ) -> Result<bool, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::ArtifactIsProjectOutput {
                project_id,
                artifact_id,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn list_artifact_roots(&self) -> Result<Vec<ArtifactId>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::ListArtifactRoots { reply })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn submit_job(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        plan: JobPlan,
    ) -> Result<MutationResult, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::SubmitJob {
                request_id,
                request_hash,
                plan,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn get_job(&self, job_id: String) -> Result<JobRecord, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::GetJob { job_id, reply })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn list_jobs(&self, project_id: String) -> Result<Vec<JobRecord>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::ListJobs { project_id, reply })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    /// Record what somebody decided about a clip. Durable before it is
    /// acknowledged, which is the whole reason it is not renderer state.
    pub(crate) async fn set_clip_decision(
        &self,
        project_id: String,
        source_id: String,
        candidate_id: String,
        decision: Decision,
        now_unix_millis: u64,
    ) -> Result<(), StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::SetClipDecision {
                project_id,
                source_id,
                candidate_id,
                decision,
                now_unix_millis,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn list_clip_decisions(
        &self,
        project_id: String,
        source_id: String,
    ) -> Result<Vec<DecisionRecord>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::ListClipDecisions {
                project_id,
                source_id,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn list_edit_docs(
        &self,
        project_id: String,
    ) -> Result<Vec<EditDocRecord>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::ListEditDocs { project_id, reply })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn cancel_job(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        job_id: String,
        completed_unix_millis: u64,
    ) -> Result<MutationResult, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::CancelJob {
                request_id,
                request_hash,
                job_id,
                completed_unix_millis,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn recover_jobs(
        &self,
        daemon_epoch: String,
        recovered_unix_millis: u64,
    ) -> Result<Vec<TaskEventRecord>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::RecoverJobs {
                daemon_epoch,
                recovered_unix_millis,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn lease_next_task(
        &self,
        request: LeaseRequest,
    ) -> Result<LeaseSelection, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::LeaseNextTask { request, reply })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn heartbeat_task(
        &self,
        lease_id: String,
        now_unix_millis: u64,
        expires_unix_millis: u64,
        progress: Option<clipmill_contracts::proto::worker::v1::ProgressUnits>,
    ) -> Result<Vec<TaskEventRecord>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::HeartbeatTask {
                lease_id,
                now_unix_millis,
                expires_unix_millis,
                progress,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn replay_task_completion(
        &self,
        lease_id: String,
        worker_id: String,
        completion_hash: [u8; 32],
    ) -> Result<Option<Vec<u8>>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::ReplayTaskCompletion {
                lease_id,
                worker_id,
                completion_hash,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn complete_task(
        &self,
        lease_id: String,
        artifact_id: ArtifactId,
        completion_hash: [u8; 32],
        completion_response: Vec<u8>,
        completed_unix_millis: u64,
    ) -> Result<TaskCompletion, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::CompleteTask {
                lease_id,
                artifact_id,
                completion_hash,
                completion_response,
                completed_unix_millis,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn fail_task(
        &self,
        lease_id: String,
        failure_class: i32,
        detail: String,
        failed_unix_millis: u64,
    ) -> Result<Vec<TaskEventRecord>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::FailTask {
                lease_id,
                failure_class,
                detail,
                failed_unix_millis,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn complete_failed_task(
        &self,
        lease_id: String,
        failure_class: i32,
        detail: String,
        completion_hash: [u8; 32],
        completion_response: Vec<u8>,
        completed_unix_millis: u64,
    ) -> Result<TaskCompletion, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::CompleteFailedTask {
                lease_id,
                failure_class,
                detail,
                completion_hash,
                completion_response,
                completed_unix_millis,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn expire_task_leases(
        &self,
        now_unix_millis: u64,
        daemon_epoch: &str,
    ) -> Result<Vec<TaskEventRecord>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::ExpireTaskLeases {
                now_unix_millis,
                daemon_epoch: daemon_epoch.to_owned(),
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn current_event_id(&self) -> Result<u64, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::CurrentEventId { reply })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn list_events(
        &self,
        after_event_id: u64,
        filter: EventFilter,
    ) -> Result<Vec<TaskEventRecord>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::ListEvents {
                after_event_id,
                filter,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn find_source_observation(
        &self,
        project_id: String,
        observation: crate::sources::FileObservation,
    ) -> Result<Option<SourceRecord>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::FindSourceObservation {
                project_id,
                observation,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn register_source(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        project_id: String,
        source_id: String,
        inspection: crate::sources::InspectedSource,
        created_unix_millis: u64,
    ) -> Result<Vec<u8>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::RegisterSource {
                request_id,
                request_hash,
                project_id,
                source_id,
                inspection,
                created_unix_millis,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn get_source(&self, source_id: String) -> Result<SourceRecord, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::GetSource { source_id, reply })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn create_edit_doc(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        project_id: String,
        document_json: String,
        now_unix_millis: u64,
    ) -> Result<Vec<u8>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::CreateEditDoc {
                request_id,
                request_hash,
                project_id,
                document_json,
                now_unix_millis,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn apply_edit_command(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        doc_id: String,
        expected_revision: u64,
        command_json: String,
        now_unix_millis: u64,
    ) -> Result<Vec<u8>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::ApplyEdit {
                request_id,
                request_hash,
                doc_id,
                expected_revision,
                command_json,
                now_unix_millis,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn get_edit_doc(&self, doc_id: String) -> Result<EditDocRecord, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::GetEditDoc { doc_id, reply })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn get_edit_log(
        &self,
        doc_id: String,
    ) -> Result<(String, Vec<EditCommandRecord>), StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::GetEditLog { doc_id, reply })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn remember_source_hit(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        source: SourceRecord,
        completed_unix_millis: u64,
    ) -> Result<Vec<u8>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::RememberSourceHit {
                request_id,
                request_hash,
                source,
                completed_unix_millis,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn list_sources(
        &self,
        project_id: String,
    ) -> Result<Vec<SourceRecord>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::ListSources { project_id, reply })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    /// Final artifact of the newest succeeded job of `kind` for a source.
    pub(crate) async fn latest_source_job_artifact(
        &self,
        source_id: String,
        kind: String,
    ) -> Result<Option<String>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::LatestSourceJobArtifact {
                source_id,
                kind,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn begin_device_profile(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        hardware_fingerprint: String,
        remeasure: bool,
        now_unix_millis: u64,
    ) -> Result<BeginDeviceProfile, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::BeginDeviceProfile {
                request_id,
                request_hash,
                hardware_fingerprint,
                remeasure,
                now_unix_millis,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn device_profile_for_job(
        &self,
        job_id: String,
    ) -> Result<DeviceProfileRecord, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::DeviceProfileForJob { job_id, reply })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn current_device_profile(
        &self,
        hardware_fingerprint: String,
    ) -> Result<Option<DeviceProfileRecord>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::CurrentDeviceProfile {
                hardware_fingerprint,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn store_device_profile_json(
        &self,
        job_id: String,
        profile_json: String,
        now_unix_millis: u64,
    ) -> Result<DeviceProfileRecord, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::StoreDeviceProfileJson {
                job_id,
                profile_json,
                now_unix_millis,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn finish_device_profile_request(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        artifact_id: ArtifactId,
        response: Vec<u8>,
        now_unix_millis: u64,
    ) -> Result<Vec<u8>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::FinishDeviceProfileRequest {
                request_id,
                request_hash,
                artifact_id,
                response,
                now_unix_millis,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }
}

#[derive(Debug)]
enum Command {
    CreateEditDoc {
        request_id: String,
        request_hash: [u8; 32],
        project_id: String,
        document_json: String,
        now_unix_millis: u64,
        reply: oneshot::Sender<Result<Vec<u8>, StoreError>>,
    },
    ApplyEdit {
        request_id: String,
        request_hash: [u8; 32],
        doc_id: String,
        expected_revision: u64,
        command_json: String,
        now_unix_millis: u64,
        reply: oneshot::Sender<Result<Vec<u8>, StoreError>>,
    },
    GetEditDoc {
        doc_id: String,
        reply: oneshot::Sender<Result<EditDocRecord, StoreError>>,
    },
    GetEditLog {
        doc_id: String,
        reply: oneshot::Sender<Result<(String, Vec<EditCommandRecord>), StoreError>>,
    },
    Create {
        request_id: String,
        request_hash: [u8; 32],
        project: ProjectRecord,
        reply: oneshot::Sender<Result<Vec<u8>, StoreError>>,
    },
    Delete {
        request_id: String,
        request_hash: [u8; 32],
        project_id: String,
        completed_unix_millis: u64,
        reply: oneshot::Sender<Result<Vec<u8>, StoreError>>,
    },
    Get {
        project_id: String,
        reply: oneshot::Sender<Result<ProjectRecord, StoreError>>,
    },
    List {
        reply: oneshot::Sender<Result<Vec<ProjectRecord>, StoreError>>,
    },
    AttachArtifactRoot {
        project_id: String,
        artifact_id: String,
        reply: oneshot::Sender<Result<(), StoreError>>,
    },
    ArtifactIsProjectOutput {
        project_id: String,
        artifact_id: String,
        reply: oneshot::Sender<Result<bool, StoreError>>,
    },
    ListArtifactRoots {
        reply: oneshot::Sender<Result<Vec<ArtifactId>, StoreError>>,
    },
    SubmitJob {
        request_id: String,
        request_hash: [u8; 32],
        plan: JobPlan,
        reply: oneshot::Sender<Result<MutationResult, StoreError>>,
    },
    GetJob {
        job_id: String,
        reply: oneshot::Sender<Result<JobRecord, StoreError>>,
    },
    ListJobs {
        project_id: String,
        reply: oneshot::Sender<Result<Vec<JobRecord>, StoreError>>,
    },
    SetClipDecision {
        project_id: String,
        source_id: String,
        candidate_id: String,
        decision: Decision,
        now_unix_millis: u64,
        reply: oneshot::Sender<Result<(), StoreError>>,
    },
    ListClipDecisions {
        project_id: String,
        source_id: String,
        reply: oneshot::Sender<Result<Vec<DecisionRecord>, StoreError>>,
    },
    ListEditDocs {
        project_id: String,
        reply: oneshot::Sender<Result<Vec<EditDocRecord>, StoreError>>,
    },
    CancelJob {
        request_id: String,
        request_hash: [u8; 32],
        job_id: String,
        completed_unix_millis: u64,
        reply: oneshot::Sender<Result<MutationResult, StoreError>>,
    },
    RecoverJobs {
        daemon_epoch: String,
        recovered_unix_millis: u64,
        reply: oneshot::Sender<Result<Vec<TaskEventRecord>, StoreError>>,
    },
    LeaseNextTask {
        request: LeaseRequest,
        reply: oneshot::Sender<Result<LeaseSelection, StoreError>>,
    },
    HeartbeatTask {
        lease_id: String,
        now_unix_millis: u64,
        expires_unix_millis: u64,
        progress: Option<clipmill_contracts::proto::worker::v1::ProgressUnits>,
        reply: oneshot::Sender<Result<Vec<TaskEventRecord>, StoreError>>,
    },
    ReplayTaskCompletion {
        lease_id: String,
        worker_id: String,
        completion_hash: [u8; 32],
        reply: oneshot::Sender<Result<Option<Vec<u8>>, StoreError>>,
    },
    CompleteTask {
        lease_id: String,
        artifact_id: ArtifactId,
        completion_hash: [u8; 32],
        completion_response: Vec<u8>,
        completed_unix_millis: u64,
        reply: oneshot::Sender<Result<TaskCompletion, StoreError>>,
    },
    FailTask {
        lease_id: String,
        failure_class: i32,
        detail: String,
        failed_unix_millis: u64,
        reply: oneshot::Sender<Result<Vec<TaskEventRecord>, StoreError>>,
    },
    CompleteFailedTask {
        lease_id: String,
        failure_class: i32,
        detail: String,
        completion_hash: [u8; 32],
        completion_response: Vec<u8>,
        completed_unix_millis: u64,
        reply: oneshot::Sender<Result<TaskCompletion, StoreError>>,
    },
    ExpireTaskLeases {
        now_unix_millis: u64,
        daemon_epoch: String,
        reply: oneshot::Sender<Result<Vec<TaskEventRecord>, StoreError>>,
    },
    CurrentEventId {
        reply: oneshot::Sender<Result<u64, StoreError>>,
    },
    ListEvents {
        after_event_id: u64,
        filter: EventFilter,
        reply: oneshot::Sender<Result<Vec<TaskEventRecord>, StoreError>>,
    },
    FindSourceObservation {
        project_id: String,
        observation: crate::sources::FileObservation,
        reply: oneshot::Sender<Result<Option<SourceRecord>, StoreError>>,
    },
    RegisterSource {
        request_id: String,
        request_hash: [u8; 32],
        project_id: String,
        source_id: String,
        inspection: crate::sources::InspectedSource,
        created_unix_millis: u64,
        reply: oneshot::Sender<Result<Vec<u8>, StoreError>>,
    },
    RememberSourceHit {
        request_id: String,
        request_hash: [u8; 32],
        source: SourceRecord,
        completed_unix_millis: u64,
        reply: oneshot::Sender<Result<Vec<u8>, StoreError>>,
    },
    GetSource {
        source_id: String,
        reply: oneshot::Sender<Result<SourceRecord, StoreError>>,
    },
    ListSources {
        project_id: String,
        reply: oneshot::Sender<Result<Vec<SourceRecord>, StoreError>>,
    },
    LatestSourceJobArtifact {
        source_id: String,
        kind: String,
        reply: oneshot::Sender<Result<Option<String>, StoreError>>,
    },
    BeginDeviceProfile {
        request_id: String,
        request_hash: [u8; 32],
        hardware_fingerprint: String,
        remeasure: bool,
        now_unix_millis: u64,
        reply: oneshot::Sender<Result<BeginDeviceProfile, StoreError>>,
    },
    DeviceProfileForJob {
        job_id: String,
        reply: oneshot::Sender<Result<DeviceProfileRecord, StoreError>>,
    },
    CurrentDeviceProfile {
        hardware_fingerprint: String,
        reply: oneshot::Sender<Result<Option<DeviceProfileRecord>, StoreError>>,
    },
    StoreDeviceProfileJson {
        job_id: String,
        profile_json: String,
        now_unix_millis: u64,
        reply: oneshot::Sender<Result<DeviceProfileRecord, StoreError>>,
    },
    FinishDeviceProfileRequest {
        request_id: String,
        request_hash: [u8; 32],
        artifact_id: ArtifactId,
        response: Vec<u8>,
        now_unix_millis: u64,
        reply: oneshot::Sender<Result<Vec<u8>, StoreError>>,
    },
    Shutdown {
        reply: oneshot::Sender<()>,
    },
}

fn open_database(path: &Path, backups_dir: &Path) -> Result<Connection, DaemonError> {
    let found_version = rusqlite::version_number();
    enforce_sqlite_version(found_version, rusqlite::version())?;

    create_private_database_file(path)?;
    let mut connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    connection.busy_timeout(Duration::from_secs(5))?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;

    let journal_mode: String =
        connection.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
    if !journal_mode.eq_ignore_ascii_case("wal") {
        return Err(DaemonError::UnexpectedJournalMode {
            found: journal_mode,
        });
    }
    connection.execute_batch("PRAGMA synchronous = FULL;")?;

    let check: String = connection.query_row("PRAGMA quick_check(1)", [], |row| row.get(0))?;
    enforce_integrity_check(&check)?;
    migrate(&mut connection, backups_dir)?;
    let check: String = connection.query_row("PRAGMA quick_check(1)", [], |row| row.get(0))?;
    enforce_integrity_check(&check)?;
    Ok(connection)
}

fn enforce_sqlite_version(found: i32, label: &str) -> Result<(), DaemonError> {
    if found < SQLITE_MIN_VERSION {
        return Err(DaemonError::SqliteTooOld {
            found: label.to_owned(),
        });
    }
    Ok(())
}

fn enforce_integrity_check(result: &str) -> Result<(), DaemonError> {
    if result != "ok" {
        return Err(DaemonError::IntegrityCheckFailed {
            result: result.to_owned(),
        });
    }
    Ok(())
}

fn create_private_database_file(path: &Path) -> Result<(), DaemonError> {
    #[cfg(unix)]
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

    if !path.exists() {
        let mut options = OpenOptions::new();
        options.create_new(true).read(true).write(true);
        #[cfg(unix)]
        options.mode(0o600);
        options
            .open(path)
            .map_err(|source| DaemonError::io(path, source))?;
    }
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .map_err(|source| DaemonError::io(path, source))?;
    Ok(())
}

const CREATE_V1_TABLES: &str = "
    CREATE TABLE projects (
        project_id TEXT PRIMARY KEY
            CHECK(length(project_id) = 30 AND substr(project_id, 1, 4) = 'prj_'),
        name TEXT NOT NULL
            CHECK(length(name) BETWEEN 1 AND 200),
        created_unix_millis INTEGER NOT NULL
            CHECK(created_unix_millis >= 0)
    ) STRICT;

    CREATE TABLE request_dedup (
        request_id TEXT PRIMARY KEY
            CHECK(length(request_id) BETWEEN 1 AND 128),
        request_hash BLOB NOT NULL
            CHECK(length(request_hash) = 32),
        response_blob BLOB NOT NULL,
        completed_unix_millis INTEGER NOT NULL
            CHECK(completed_unix_millis >= 0)
    ) STRICT;
";

const CREATE_V2_TABLES: &str = "
    CREATE TABLE project_artifact_roots (
        project_id TEXT NOT NULL
            REFERENCES projects(project_id) ON DELETE CASCADE,
        artifact_id TEXT NOT NULL
            CHECK(
                length(artifact_id) = 71
                AND substr(artifact_id, 1, 7) = 'sha256:'
                AND substr(artifact_id, 8) NOT GLOB '*[^0-9a-f]*'
            ),
        PRIMARY KEY(project_id, artifact_id)
    ) STRICT, WITHOUT ROWID;

    CREATE INDEX project_artifact_roots_by_artifact
        ON project_artifact_roots(artifact_id);
";

fn migrate(connection: &mut Connection, backups_dir: &Path) -> Result<(), DaemonError> {
    let application_id: i64 =
        connection.query_row("PRAGMA application_id", [], |row| row.get(0))?;
    if application_id != 0 && application_id != APPLICATION_ID {
        return Err(DaemonError::UnexpectedApplicationId {
            found: application_id,
        });
    }

    let version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if version > SCHEMA_VERSION {
        return Err(DaemonError::UnsupportedSchema {
            found: version,
            supported: SCHEMA_VERSION,
        });
    }
    if version == 0 {
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute_batch(CREATE_V1_TABLES)?;
        transaction.execute_batch(CREATE_V2_TABLES)?;
        transaction.execute_batch(job_store::CREATE_V3_TABLES)?;
        transaction.execute_batch(source_store::CREATE_V4_TABLES)?;
        transaction.execute_batch(device_store::CREATE_V5_TABLES)?;
        transaction.execute_batch(edit_store::CREATE_V6_TABLES)?;
        transaction.execute_batch(job_store::CREATE_V7_TABLES)?;
        transaction.execute_batch(job_store::CREATE_V8_TABLES)?;
        transaction.execute_batch(decision_store::CREATE_V9_TABLES)?;
        transaction
            .execute_batch("PRAGMA application_id = 1129074765; PRAGMA user_version = 9;")?;
        transaction.commit()?;
    } else if version < SCHEMA_VERSION {
        create_schema_backup(connection, backups_dir, version, SCHEMA_VERSION)?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        if version < 2 {
            transaction.execute_batch(CREATE_V2_TABLES)?;
        }
        if version < 3 {
            transaction.execute_batch(job_store::CREATE_V3_TABLES)?;
        }
        if version < 4 {
            transaction.execute_batch(source_store::CREATE_V4_TABLES)?;
        }
        if version < 5 {
            transaction.execute_batch(device_store::CREATE_V5_TABLES)?;
        }
        if version < 6 {
            transaction.execute_batch(edit_store::CREATE_V6_TABLES)?;
        }
        if version < 7 {
            transaction.execute_batch(job_store::CREATE_V7_TABLES)?;
        }
        if version < 8 {
            transaction.execute_batch(job_store::CREATE_V8_TABLES)?;
        }
        if version < 9 {
            transaction.execute_batch(decision_store::CREATE_V9_TABLES)?;
        }
        transaction.execute_batch("PRAGMA user_version = 9;")?;
        transaction.commit()?;
    }
    Ok(())
}

fn create_schema_backup(
    source: &Connection,
    backups_dir: &Path,
    from_version: i64,
    to_version: i64,
) -> Result<PathBuf, DaemonError> {
    create_private_directory(backups_dir)?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| DaemonError::InvalidDuration(error.to_string()))?
        .as_millis();
    let base = format!(
        "clipmill-v{from_version}-to-v{to_version}-{timestamp}-{}",
        Ulid::new()
    );
    let temporary = backups_dir.join(format!("{base}.db.tmp"));
    let final_path = backups_dir.join(format!("{base}.db"));
    create_private_database_file(&temporary)?;

    let backup_result = (|| -> Result<(), DaemonError> {
        let mut destination = Connection::open_with_flags(
            &temporary,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        {
            let backup = Backup::new(source, &mut destination)?;
            backup.run_to_completion(128, Duration::from_millis(5), None)?;
        }
        let check: String = destination.query_row("PRAGMA quick_check(1)", [], |row| row.get(0))?;
        enforce_integrity_check(&check)?;
        drop(destination);
        File::open(&temporary)
            .and_then(|file| file.sync_all())
            .map_err(|source| DaemonError::io(&temporary, source))?;
        fs::rename(&temporary, &final_path)
            .map_err(|source| DaemonError::io(&final_path, source))?;
        set_private_file_permissions(&final_path)?;
        sync_directory(backups_dir)
    })();

    if backup_result.is_err() {
        let _cleanup = fs::remove_file(&temporary);
    }
    backup_result?;
    Ok(final_path)
}

fn create_private_directory(path: &Path) -> Result<(), DaemonError> {
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    fs::create_dir_all(path).map_err(|source| DaemonError::io(path, source))?;
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|source| DaemonError::io(path, source))?;
    Ok(())
}

fn set_private_file_permissions(path: &Path) -> Result<(), DaemonError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|source| DaemonError::io(path, source))?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), DaemonError> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|source| DaemonError::io(path, source))
}

fn create_project(
    connection: &mut Connection,
    request_id: &str,
    request_hash: &[u8; 32],
    project: &ProjectRecord,
) -> Result<Vec<u8>, StoreError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(response) = replay(&transaction, request_id, request_hash)? {
        transaction.commit()?;
        return Ok(response);
    }

    let created = i64::try_from(project.created_unix_millis)
        .map_err(|_| StoreError::InvalidData("project timestamp exceeds SQLite integer range"))?;
    transaction.execute(
        "INSERT INTO projects(project_id, name, created_unix_millis) VALUES (?1, ?2, ?3)",
        params![project.project_id, project.name, created],
    )?;

    let response = Response {
        request_id: request_id.to_owned(),
        body: Some(response::Body::CreateProject(CreateProjectResponse {
            project: Some(project.clone().into()),
        })),
    }
    .encode_to_vec();
    remember(
        &transaction,
        request_id,
        request_hash,
        &response,
        project.created_unix_millis,
    )?;
    transaction.commit()?;
    Ok(response)
}

fn delete_project(
    connection: &mut Connection,
    request_id: &str,
    request_hash: &[u8; 32],
    project_id: &str,
    completed_unix_millis: u64,
) -> Result<Vec<u8>, StoreError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(response) = replay(&transaction, request_id, request_hash)? {
        transaction.commit()?;
        return Ok(response);
    }

    let deleted = transaction.execute(
        "DELETE FROM projects WHERE project_id = ?1 AND is_system = 0",
        [project_id],
    )?;
    if deleted == 0 {
        return Err(StoreError::NotFound);
    }

    let response = Response {
        request_id: request_id.to_owned(),
        body: Some(response::Body::DeleteProject(DeleteProjectResponse {})),
    }
    .encode_to_vec();
    remember(
        &transaction,
        request_id,
        request_hash,
        &response,
        completed_unix_millis,
    )?;
    transaction.commit()?;
    Ok(response)
}

fn replay(
    transaction: &Transaction<'_>,
    request_id: &str,
    request_hash: &[u8; 32],
) -> Result<Option<Vec<u8>>, StoreError> {
    let cached: Option<(Vec<u8>, Vec<u8>)> = transaction
        .query_row(
            "SELECT request_hash, response_blob FROM request_dedup WHERE request_id = ?1",
            [request_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    match cached {
        Some((cached_hash, response)) if cached_hash == request_hash => Ok(Some(response)),
        Some(_) => Err(StoreError::Conflict),
        None => Ok(None),
    }
}

fn remember(
    transaction: &Transaction<'_>,
    request_id: &str,
    request_hash: &[u8; 32],
    response: &[u8],
    completed_unix_millis: u64,
) -> Result<(), StoreError> {
    let completed = i64::try_from(completed_unix_millis)
        .map_err(|_| StoreError::InvalidData("request timestamp exceeds SQLite integer range"))?;
    transaction.execute(
        "INSERT INTO request_dedup(request_id, request_hash, response_blob, completed_unix_millis)
         VALUES (?1, ?2, ?3, ?4)",
        params![request_id, request_hash.as_slice(), response, completed],
    )?;
    Ok(())
}

fn get_project(connection: &Connection, project_id: &str) -> Result<ProjectRecord, StoreError> {
    connection
        .query_row(
            "SELECT project_id, name, created_unix_millis FROM projects
             WHERE project_id = ?1 AND is_system = 0",
            [project_id],
            project_from_row,
        )
        .optional()?
        .ok_or(StoreError::NotFound)
}

fn list_projects(connection: &Connection) -> Result<Vec<ProjectRecord>, StoreError> {
    let mut statement = connection.prepare(
        "SELECT project_id, name, created_unix_millis
         FROM projects
         WHERE is_system = 0
         ORDER BY created_unix_millis DESC, project_id DESC",
    )?;
    let rows = statement.query_map([], project_from_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn attach_artifact_root(
    connection: &mut Connection,
    project_id: &str,
    artifact_id: &str,
) -> Result<(), StoreError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let project_exists: bool = transaction.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM projects WHERE project_id = ?1 AND is_system = 0
         )",
        [project_id],
        |row| row.get(0),
    )?;
    if !project_exists {
        return Err(StoreError::NotFound);
    }
    transaction.execute(
        "INSERT OR IGNORE INTO project_artifact_roots(project_id, artifact_id) VALUES (?1, ?2)",
        params![project_id, artifact_id],
    )?;
    transaction.commit()?;
    Ok(())
}

fn list_artifact_roots(connection: &Connection) -> Result<Vec<ArtifactId>, StoreError> {
    let mut statement = connection.prepare(
        "SELECT artifact_id FROM project_artifact_roots
         UNION
         SELECT artifact_id FROM task_artifact_roots
         UNION
         SELECT artifact_id FROM source_artifact_roots
         UNION
         SELECT artifact_id FROM system_artifact_roots
         ORDER BY artifact_id ASC",
    )?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    let encoded = rows.collect::<Result<Vec<_>, _>>()?;
    encoded
        .into_iter()
        .map(|value| {
            value
                .parse()
                .map_err(|_| StoreError::InvalidData("artifact root id is invalid"))
        })
        .collect()
}

fn project_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProjectRecord> {
    let created: i64 = row.get(2)?;
    let created_unix_millis = u64::try_from(created).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            2,
            rusqlite::types::Type::Integer,
            Box::new(error),
        )
    })?;
    Ok(ProjectRecord {
        project_id: row.get(0)?,
        name: row.get(1)?,
        created_unix_millis,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

    use std::{collections::BTreeSet, fs, path::Path};

    use clipmill_contracts::proto::{
        ipc::v1::{JobState, TaskState},
        worker::v1::FailureClass,
    };
    use clipmill_core::{ArtifactId, LeaseId, ProjectId, Sha256Digest, SourceId};
    use prost::Message;
    use rusqlite::{Connection, OpenFlags, params};
    use tempfile::TempDir;

    use super::{
        CREATE_V1_TABLES, CREATE_V2_TABLES, Decision, ProjectRecord, SCHEMA_VERSION,
        SQLITE_MIN_VERSION, StoreError, attach_artifact_root, create_project, decision_store,
        delete_project, device_store, edit_store, enforce_integrity_check, enforce_sqlite_version,
        get_project, job_store, list_artifact_roots, list_projects, open_database, source_store,
    };
    use crate::{
        DaemonError,
        jobs::{JobPlan, ResourceCapacity},
        sources::{FileObservation, InspectedSource},
    };

    fn database(temp: &TempDir) -> (std::path::PathBuf, Connection) {
        let path = temp.path().join("clipmill.db");
        let connection =
            open_database(&path, &temp.path().join("backups")).expect("database opens");
        (path, connection)
    }

    fn project(id: &str, name: &str, at: u64) -> ProjectRecord {
        ProjectRecord {
            project_id: id.to_owned(),
            name: name.to_owned(),
            created_unix_millis: at,
        }
    }

    /// The command log is the document's history of record: replaying it over
    /// the initial document must land on exactly the live bytes, or an
    /// archive restored from the log would silently differ from what the user
    /// last saw.
    #[test]
    fn replaying_the_command_log_reproduces_the_live_document() {
        use clipmill_edit_ir::{EditCommand, EditDocument};

        let temp = TempDir::new().expect("tempdir");
        let (_path, mut connection) = database(&temp);
        create_project(
            &mut connection,
            "create-project",
            &[1; 32],
            &project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAV", "Edits", 1),
        )
        .expect("project");

        let created = edit_store::create_edit_doc(
            &mut connection,
            "create-doc",
            &[2; 32],
            "prj_01ARZ3NDEKTSV4RRFFQ69G5FAV",
            &sample_edit_document(),
            10,
        )
        .expect("create edit doc");
        let doc_id = created_doc_id(&created);

        let commands = [
            EditCommand::SetLayout {
                segment_id: "seg_a".to_owned(),
                state: clipmill_edit_ir::LayoutState::SpeakerFill,
            },
            EditCommand::EditCaptionText {
                cue_id: "cue_a".to_owned(),
                word_index: 1,
                text: "TWO".to_owned(),
            },
            EditCommand::Trim {
                segment_id: "seg_a".to_owned(),
                in_ticks: 90_000,
                out_ticks: 900_000,
            },
            EditCommand::SetGain {
                t_ticks: 0,
                gain_db: -2.5,
            },
        ];
        for (index, command) in commands.iter().enumerate() {
            let json =
                String::from_utf8(command.to_canonical_json().expect("serialize")).expect("utf-8");
            edit_store::apply_edit_command(
                &mut connection,
                &format!("apply-{index}"),
                &[u8::try_from(index).unwrap_or(0) + 8; 32],
                &doc_id,
                index as u64,
                &json,
                20 + index as u64,
            )
            .expect("apply command");
        }

        let live = edit_store::get_edit_doc(&connection, &doc_id).expect("live document");
        assert_eq!(live.revision, 4);
        let (initial_json, log) = edit_store::get_edit_log(&connection, &doc_id).expect("log");
        assert_eq!(log.len(), 4);
        assert_eq!(
            log.iter().map(|entry| entry.revision).collect::<Vec<_>>(),
            vec![1, 2, 3, 4]
        );

        let mut replayed =
            EditDocument::from_canonical_json(initial_json.as_bytes()).expect("initial document");
        for entry in &log {
            let command = EditCommand::from_canonical_json(entry.command_json.as_bytes())
                .expect("logged command parses");
            command
                .apply(&mut replayed)
                .expect("logged command applies");
        }
        assert_eq!(
            String::from_utf8(replayed.to_canonical_json().expect("canonical")).expect("utf-8"),
            live.document_json,
            "replaying the durable log must reproduce the live document byte for byte"
        );

        // Undoing with the stored inverse walks back to the previous state.
        let last = log.last().expect("last entry");
        let inverse = EditCommand::from_canonical_json(last.inverse_json.as_bytes())
            .expect("stored inverse parses");
        inverse.apply(&mut replayed).expect("inverse applies");
        assert!(
            replayed.audio.gain_curve.is_empty(),
            "the stored inverse must undo the change it was recorded for"
        );
    }

    /// Editing a revision the client has not seen is a conflict, not a
    /// silent rebase that discards whichever edit lost the race.
    #[test]
    fn applying_against_a_stale_revision_conflicts() {
        let temp = TempDir::new().expect("tempdir");
        let (_path, mut connection) = database(&temp);
        create_project(
            &mut connection,
            "create-project",
            &[1; 32],
            &project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAV", "Edits", 1),
        )
        .expect("project");
        let created = edit_store::create_edit_doc(
            &mut connection,
            "create-doc",
            &[2; 32],
            "prj_01ARZ3NDEKTSV4RRFFQ69G5FAV",
            "",
            10,
        )
        .expect("create edit doc");
        let doc_id = created_doc_id(&created);
        let command =
            serde_json::json!({"op": "set_gain", "t_ticks": 0, "gain_db": -3.0}).to_string();
        edit_store::apply_edit_command(
            &mut connection,
            "apply-0",
            &[3; 32],
            &doc_id,
            0,
            &command,
            20,
        )
        .expect("first apply");
        let stale = edit_store::apply_edit_command(
            &mut connection,
            "apply-1",
            &[4; 32],
            &doc_id,
            0,
            &command,
            21,
        );
        assert!(matches!(stale, Err(StoreError::Conflict)));
        let replayed = edit_store::apply_edit_command(
            &mut connection,
            "apply-0",
            &[3; 32],
            &doc_id,
            0,
            &command,
            22,
        )
        .expect("retrying the same request replays its response");
        assert_eq!(
            edit_store::get_edit_doc(&connection, &doc_id)
                .expect("doc")
                .revision,
            1,
            "a replayed request must not apply the command twice"
        );
        assert!(!replayed.is_empty());
    }

    fn sample_edit_document() -> String {
        let fingerprint = format!("sha256:{}", "ab".repeat(32));
        serde_json::json!({
            "version": "ir/1",
            "timebase": {"num": 1, "den": 90000},
            "video": {"segments": [{
                "segment_id": "seg_a",
                "source_fingerprint": fingerprint,
                "in_ticks": 0,
                "out_ticks": 900_000,
                "layout": {"state": "fit"},
            }]},
            "captions": {"style_ref": "clean", "cues": [{
                "cue_id": "cue_a",
                "start_ticks": 0,
                "end_ticks": 90_000,
                "region": "lower_safe",
                "anim": "karaoke",
                "lines": [{"words": [
                    {"text": "one", "start_ticks": 0, "end_ticks": 45_000},
                    {"text": "two", "start_ticks": 45_000, "end_ticks": 90_000},
                ]}],
            }]},
            "audio": {"target_lufs": -14.0, "true_peak_dbtp": -1.0},
        })
        .to_string()
    }

    fn created_doc_id(response: &[u8]) -> String {
        let decoded =
            clipmill_contracts::proto::ipc::v1::Response::decode(response).expect("decode");
        match decoded.body {
            Some(clipmill_contracts::proto::ipc::v1::response::Body::CreateEditDoc(created)) => {
                created.doc.expect("doc").doc_id
            }
            _ => panic!("unexpected create response"),
        }
    }

    #[test]
    fn database_is_migrated_and_configured() {
        let temp = TempDir::new().expect("tempdir");
        let (path, connection) = database(&temp);
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("version");
        let mode: String = connection
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .expect("journal mode");
        let foreign_keys: i64 = connection
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("foreign keys");
        let synchronous: i64 = connection
            .query_row("PRAGMA synchronous", [], |row| row.get(0))
            .expect("synchronous mode");
        let busy_timeout: i64 = connection
            .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
            .expect("busy timeout");
        let strict_projects: i64 = connection
            .query_row(
                "SELECT strict FROM pragma_table_list WHERE name = 'projects'",
                [],
                |row| row.get(0),
            )
            .expect("projects table mode");
        let strict_roots: i64 = connection
            .query_row(
                "SELECT strict FROM pragma_table_list WHERE name = 'project_artifact_roots'",
                [],
                |row| row.get(0),
            )
            .expect("artifact roots table mode");
        let strict_sources: i64 = connection
            .query_row(
                "SELECT strict FROM pragma_table_list WHERE name = 'sources'",
                [],
                |row| row.get(0),
            )
            .expect("sources table mode");
        let strict_device_profiles: i64 = connection
            .query_row(
                "SELECT strict FROM pragma_table_list
                 WHERE name = 'device_profile_generations'",
                [],
                |row| row.get(0),
            )
            .expect("device profiles table mode");
        let quick_check: String = connection
            .query_row("PRAGMA quick_check(1)", [], |row| row.get(0))
            .expect("quick check");
        assert_eq!(version, SCHEMA_VERSION);
        assert_eq!(mode, "wal");
        assert_eq!(foreign_keys, 1);
        assert_eq!(synchronous, 2);
        assert_eq!(busy_timeout, 5_000);
        assert_eq!(strict_projects, 1);
        assert_eq!(strict_roots, 1);
        assert_eq!(strict_sources, 1);
        assert_eq!(strict_device_profiles, 1);
        assert_eq!(quick_check, "ok");
        let backup_count = fs::read_dir(temp.path().join("backups"))
            .map(Iterator::count)
            .unwrap_or_default();
        assert_eq!(backup_count, 0);
        drop(connection);
        open_database(&path, &temp.path().join("backups")).expect("repeat startup succeeds");
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn system_project_is_private_and_device_requests_join_durably() {
        let temp = TempDir::new().expect("tempdir");
        let (_path, mut connection) = database(&temp);
        assert!(list_projects(&connection).expect("projects").is_empty());
        assert!(matches!(
            get_project(&connection, crate::jobs::SYSTEM_PROJECT_ID),
            Err(StoreError::NotFound)
        ));

        let fingerprint = format!("sha256:{}", "0".repeat(64));
        let started = device_store::begin_device_profile(
            &mut connection,
            "device-request",
            &[1; 32],
            &fingerprint,
            false,
            10,
        )
        .expect("begin profile");
        let device_store::BeginDeviceProfile::Profile { record, events } = started else {
            panic!("new request must create a profile job");
        };
        assert_eq!(record.measurement_generation, 1);
        assert_eq!(events.len(), 1);

        let joined = device_store::begin_device_profile(
            &mut connection,
            "device-request-join",
            &[2; 32],
            &fingerprint,
            true,
            11,
        )
        .expect("join profile");
        let device_store::BeginDeviceProfile::Profile {
            record: joined_record,
            events: joined_events,
        } = joined
        else {
            panic!("join should remain pending");
        };
        assert_eq!(joined_record.job_id, record.job_id);
        assert!(joined_events.is_empty());
        assert!(matches!(
            device_store::begin_device_profile(
                &mut connection,
                "device-request",
                &[9; 32],
                &fingerprint,
                false,
                12,
            ),
            Err(StoreError::Conflict)
        ));

        device_store::store_profile_json(&mut connection, &record.job_id, "{}", 20)
            .expect("store measured profile");
        let lease_id = LeaseId::new().to_string();
        let leased = job_store::lease_next_task(
            &mut connection,
            &lease_id,
            "01ARZ3NDEKTSV4RRFFQ69G5FAV",
            21,
            15_021,
            ResourceCapacity::w4_builtin(),
        )
        .expect("lease device task");
        assert_eq!(leased.task.expect("task").kind, "device-profile");
        let artifact_id = format!("sha256:{}", "1".repeat(64))
            .parse::<ArtifactId>()
            .expect("artifact id");
        job_store::complete_task(
            &mut connection,
            &lease_id,
            artifact_id,
            &[3; 32],
            b"artifact-response",
            30,
        )
        .expect("complete profile");
        let roots = list_artifact_roots(&connection).expect("roots");
        assert_eq!(roots, vec![artifact_id]);

        let cached = device_store::begin_device_profile(
            &mut connection,
            "device-request-cache",
            &[4; 32],
            &fingerprint,
            false,
            31,
        )
        .expect("cached profile");
        let device_store::BeginDeviceProfile::Profile {
            record: cached_record,
            events: cached_events,
        } = cached
        else {
            panic!("cache record expected");
        };
        assert_eq!(
            cached_record.state,
            device_store::DeviceProfileState::Succeeded
        );
        assert_eq!(cached_record.artifact_id, Some(artifact_id));
        assert!(cached_events.is_empty());
        let response = device_store::finish_device_request(
            &mut connection,
            "device-request-cache",
            &[4; 32],
            artifact_id,
            b"encoded-response",
            32,
        )
        .expect("finish response");
        assert_eq!(response, b"encoded-response");
        let replayed = device_store::begin_device_profile(
            &mut connection,
            "device-request-cache",
            &[4; 32],
            &fingerprint,
            false,
            33,
        )
        .expect("replay response");
        assert!(matches!(
            replayed,
            device_store::BeginDeviceProfile::Response { bytes, .. }
                if bytes == b"encoded-response"
        ));
    }

    #[test]
    fn task_admission_uses_measured_backend_availability() {
        let temp = TempDir::new().expect("tempdir");
        let (_path, mut connection) = database(&temp);
        let project = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAV", "Accelerator", 10);
        create_project(&mut connection, "create-accelerator", &[1; 32], &project)
            .expect("create project");
        let project_id = project.project_id.parse::<ProjectId>().expect("project id");
        let mut plan = JobPlan::demo(&project_id, b"backend".to_vec(), 20);
        plan.tasks.truncate(1);
        plan.tasks[0].is_final = true;
        plan.tasks[0].resources.accelerator_class = "videotoolbox".to_owned();
        job_store::submit_job(&mut connection, "submit-accelerator", &[2; 32], &plan)
            .expect("submit job");

        let unavailable = job_store::lease_next_task(
            &mut connection,
            &LeaseId::new().to_string(),
            "01ARZ3NDEKTSV4RRFFQ69G5FAV",
            30,
            15_030,
            ResourceCapacity::measured(4, 1024 * 1024 * 1024),
        )
        .expect("query without measured backend");
        assert!(unavailable.task.is_none());

        let available = ResourceCapacity::measured(4, 1024 * 1024 * 1024)
            .with_available_backends(&BTreeSet::from(["videotoolbox".to_owned()]));
        let leased = job_store::lease_next_task(
            &mut connection,
            &LeaseId::new().to_string(),
            "01ARZ3NDEKTSV4RRFFQ69G5FAV",
            31,
            15_031,
            available,
        )
        .expect("query with measured backend");
        assert_eq!(
            leased
                .task
                .expect("accelerated task")
                .resources
                .accelerator_class,
            "videotoolbox"
        );
    }

    #[test]
    fn sqlite_version_and_integrity_results_are_enforced() {
        assert!(enforce_sqlite_version(SQLITE_MIN_VERSION, "minimum").is_ok());
        assert!(matches!(
            enforce_sqlite_version(SQLITE_MIN_VERSION - 1, "too-old"),
            Err(DaemonError::SqliteTooOld { .. })
        ));
        assert!(enforce_integrity_check("ok").is_ok());
        assert!(matches!(
            enforce_integrity_check("database disk image is malformed"),
            Err(DaemonError::IntegrityCheckFailed { .. })
        ));
    }

    #[test]
    fn newer_schema_is_refused() {
        let temp = TempDir::new().expect("tempdir");
        let path = temp.path().join("newer.db");
        let connection = Connection::open(&path).expect("open raw database");
        connection
            .execute_batch(&format!(
                "PRAGMA application_id = 1129074765; PRAGMA user_version = {};",
                SCHEMA_VERSION + 1
            ))
            .expect("set version");
        drop(connection);
        assert!(open_database(&path, &temp.path().join("backups")).is_err());
    }

    #[test]
    fn v1_upgrade_creates_verified_private_backup_and_preserves_state() {
        let temp = TempDir::new().expect("tempdir");
        let path = temp.path().join("v1.db");
        let backups = temp.path().join("backups");
        let connection = Connection::open(&path).expect("open v1 database");
        connection
            .execute_batch(CREATE_V1_TABLES)
            .expect("v1 tables");
        connection
            .execute_batch(
                "INSERT INTO projects(project_id, name, created_unix_millis)
                 VALUES ('prj_01ARZ3NDEKTSV4RRFFQ69G5FAV', 'Before upgrade', 1);
                 PRAGMA application_id = 1129074765;
                 PRAGMA user_version = 1;",
            )
            .expect("v1 state");
        drop(connection);

        let upgraded = open_database(&path, &backups).expect("upgrade");
        let version: i64 = upgraded
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("version");
        let name: String = upgraded
            .query_row("SELECT name FROM projects", [], |row| row.get(0))
            .expect("project preserved");
        assert_eq!(version, SCHEMA_VERSION);
        assert_eq!(name, "Before upgrade");
        drop(upgraded);

        let backup_paths = fs::read_dir(&backups)
            .expect("backups")
            .map(|entry| entry.expect("backup entry").path())
            .collect::<Vec<_>>();
        assert_eq!(backup_paths.len(), 1);
        assert_eq!(
            backup_paths[0].extension().and_then(|value| value.to_str()),
            Some("db")
        );
        let backup = Connection::open_with_flags(
            &backup_paths[0],
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .expect("open backup read-only");
        let backup_version: i64 = backup
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("backup version");
        let backup_check: String = backup
            .query_row("PRAGMA quick_check(1)", [], |row| row.get(0))
            .expect("backup quick check");
        assert_eq!(backup_version, 1);
        assert_eq!(backup_check, "ok");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&backup_paths[0])
                    .expect("backup metadata")
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
    }

    #[test]
    fn v2_upgrade_creates_backup_and_installs_strict_job_schema() {
        let temp = TempDir::new().expect("tempdir");
        let path = temp.path().join("v2.db");
        let backups = temp.path().join("backups");
        let connection = Connection::open(&path).expect("open v2 database");
        connection
            .execute_batch(CREATE_V1_TABLES)
            .expect("v1 schema");
        connection
            .execute_batch(CREATE_V2_TABLES)
            .expect("v2 schema");
        connection
            .execute_batch(
                "INSERT INTO projects(project_id, name, created_unix_millis)
                 VALUES ('prj_01ARZ3NDEKTSV4RRFFQ69G5FAV', 'V2', 1);
                 PRAGMA application_id = 1129074765;
                 PRAGMA user_version = 2;",
            )
            .expect("v2 state");
        drop(connection);

        let upgraded = open_database(&path, &backups).expect("upgrade v2");
        let version: i64 = upgraded
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("version");
        let strict_jobs: i64 = upgraded
            .query_row(
                "SELECT strict FROM pragma_table_list WHERE name = 'jobs'",
                [],
                |row| row.get(0),
            )
            .expect("strict jobs");
        let without_rowid_dependencies: i64 = upgraded
            .query_row(
                "SELECT wr FROM pragma_table_list WHERE name = 'task_dependencies'",
                [],
                |row| row.get(0),
            )
            .expect("without rowid dependencies");
        assert_eq!(version, SCHEMA_VERSION);
        assert_eq!(strict_jobs, 1);
        assert_eq!(without_rowid_dependencies, 1);
        drop(upgraded);

        let backup_path = fs::read_dir(&backups)
            .expect("backups")
            .next()
            .expect("one backup")
            .expect("backup entry")
            .path();
        let backup = Connection::open_with_flags(
            backup_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .expect("open backup");
        let backup_version: i64 = backup
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("backup version");
        assert_eq!(backup_version, 2);
    }

    /// The claim the Inspector's gate rests on: a decision outlives the process
    /// that took it.
    ///
    /// Written against a database that is opened, written, closed, and opened
    /// again rather than against a live handle, because "survives a kill" is a
    /// statement about what reached the disk and nothing held in memory can
    /// answer it.
    #[test]
    fn a_clip_decision_survives_the_daemon_that_recorded_it() {
        let temp = TempDir::new().expect("tempdir");
        let path = temp.path().join("decisions.db");
        let backups = temp.path().join("backups");

        let connection = open_database(&path, &backups).expect("open");
        connection
            .execute(
                // OR IGNORE because opening a store seeds a project of its
                // own; this test cares about the decision rows, not about
                // being the first writer.
                "INSERT OR IGNORE INTO projects(project_id, name, created_unix_millis)\n\
                 VALUES ('prj_00000000000000000000000000', 'test', 0)",
                [],
            )
            .expect("a project to hang the decision from");
        decision_store::set(
            &connection,
            "prj_00000000000000000000000000",
            "src_1",
            "cand_a",
            Decision::Rejected,
            10,
        )
        .expect("record a rejection");
        decision_store::set(
            &connection,
            "prj_00000000000000000000000000",
            "src_1",
            "cand_b",
            Decision::Approved,
            20,
        )
        .expect("record an approval");
        // Changing your mind replaces the answer rather than appending to it.
        decision_store::set(
            &connection,
            "prj_00000000000000000000000000",
            "src_1",
            "cand_a",
            Decision::Kept,
            30,
        )
        .expect("change of mind");
        drop(connection);

        let reopened = open_database(&path, &backups).expect("reopen");
        let found = decision_store::list(&reopened, "prj_00000000000000000000000000", "src_1")
            .expect("read back");

        assert_eq!(found.len(), 2, "one row per candidate: {found:?}");
        let changed = found
            .iter()
            .find(|record| record.candidate_id == "cand_a")
            .expect("the candidate whose decision changed");
        assert_eq!(changed.decision, Decision::Kept);
        assert_eq!(changed.decided_unix_millis, 30);
        // Newest first, which is the order a board wants.
        assert_eq!(found[0].candidate_id, "cand_a");
    }

    #[test]
    fn v3_upgrade_creates_backup_and_installs_source_evidence_schema() {
        let temp = TempDir::new().expect("tempdir");
        let path = temp.path().join("v3.db");
        let backups = temp.path().join("backups");
        let connection = Connection::open(&path).expect("open v3 database");
        connection
            .execute_batch(CREATE_V1_TABLES)
            .expect("v1 schema");
        connection
            .execute_batch(CREATE_V2_TABLES)
            .expect("v2 schema");
        connection
            .execute_batch(job_store::CREATE_V3_TABLES)
            .expect("v3 schema");
        connection
            .execute_batch(
                "INSERT INTO projects(project_id, name, created_unix_millis)
                 VALUES ('prj_01ARZ3NDEKTSV4RRFFQ69G5FAV', 'V3', 1);
                 PRAGMA application_id = 1129074765;
                 PRAGMA user_version = 3;",
            )
            .expect("v3 state");
        drop(connection);

        let upgraded = open_database(&path, &backups).expect("upgrade v3");
        let version: i64 = upgraded
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("version");
        let strict_sources: i64 = upgraded
            .query_row(
                "SELECT strict FROM pragma_table_list WHERE name = 'sources'",
                [],
                |row| row.get(0),
            )
            .expect("strict sources");
        let jobs_has_source: i64 = upgraded
            .query_row(
                "SELECT count(*) FROM pragma_table_info('jobs') WHERE name = 'source_id'",
                [],
                |row| row.get(0),
            )
            .expect("jobs source column");
        assert_eq!(version, SCHEMA_VERSION);
        assert_eq!(strict_sources, 1);
        assert_eq!(jobs_has_source, 1);
        drop(upgraded);

        let backup_path = fs::read_dir(&backups)
            .expect("backups")
            .next()
            .expect("one backup")
            .expect("backup entry")
            .path();
        let backup = Connection::open_with_flags(
            backup_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .expect("open backup");
        let backup_version: i64 = backup
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("backup version");
        assert_eq!(backup_version, 3);
    }

    #[test]
    fn v4_upgrade_creates_backup_and_installs_device_profile_schema() {
        let temp = TempDir::new().expect("tempdir");
        let path = temp.path().join("v4.db");
        let backups = temp.path().join("backups");
        let connection = Connection::open(&path).expect("open v4 database");
        connection
            .execute_batch(CREATE_V1_TABLES)
            .expect("v1 schema");
        connection
            .execute_batch(CREATE_V2_TABLES)
            .expect("v2 schema");
        connection
            .execute_batch(job_store::CREATE_V3_TABLES)
            .expect("v3 schema");
        connection
            .execute_batch(source_store::CREATE_V4_TABLES)
            .expect("v4 schema");
        connection
            .execute_batch(
                "INSERT INTO projects(project_id, name, created_unix_millis)
                 VALUES ('prj_01ARZ3NDEKTSV4RRFFQ69G5FAV', 'V4', 1);
                 PRAGMA application_id = 1129074765;
                 PRAGMA user_version = 4;",
            )
            .expect("v4 state");
        drop(connection);

        let upgraded = open_database(&path, &backups).expect("upgrade v4");
        let version: i64 = upgraded
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("version");
        let strict_profiles: i64 = upgraded
            .query_row(
                "SELECT strict FROM pragma_table_list
                 WHERE name = 'device_profile_generations'",
                [],
                |row| row.get(0),
            )
            .expect("strict profiles");
        let system_projects: i64 = upgraded
            .query_row(
                "SELECT count(*) FROM projects WHERE is_system = 1",
                [],
                |row| row.get(0),
            )
            .expect("system project");
        assert_eq!(version, SCHEMA_VERSION);
        assert_eq!(strict_profiles, 1);
        assert_eq!(system_projects, 1);
        drop(upgraded);

        let backup_path = fs::read_dir(&backups)
            .expect("backups")
            .next()
            .expect("one backup")
            .expect("backup entry")
            .path();
        let backup = Connection::open_with_flags(
            backup_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .expect("open backup");
        let backup_version: i64 = backup
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("backup version");
        assert_eq!(backup_version, 4);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn durable_job_dag_leases_events_completion_and_cancellation_are_transactional() {
        let temp = TempDir::new().expect("tempdir");
        let (_path, mut connection) = database(&temp);
        let project = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAV", "Jobs", 10);
        create_project(&mut connection, "create-jobs", &[1; 32], &project).expect("create project");
        let project_id = project.project_id.parse::<ProjectId>().expect("project id");
        let plan = JobPlan::demo(&project_id, b"payload".to_vec(), 20);
        let submitted =
            job_store::submit_job(&mut connection, "submit-job", &[2; 32], &plan).expect("submit");
        let replayed = job_store::submit_job(
            &mut connection,
            "submit-job",
            &[2; 32],
            &JobPlan::demo(&project_id, b"different".to_vec(), 21),
        )
        .expect("replay");
        assert_eq!(submitted.bytes, replayed.bytes);
        assert!(matches!(
            job_store::submit_job(
                &mut connection,
                "submit-job",
                &[3; 32],
                &JobPlan::demo(&project_id, b"different".to_vec(), 21),
            ),
            Err(StoreError::Conflict)
        ));
        assert_eq!(submitted.events.len(), 4);

        let mut cursor = 0;
        for index in 0..4_u8 {
            let lease_id = LeaseId::new().to_string();
            let selection = job_store::lease_next_task(
                &mut connection,
                &lease_id,
                "01ARZ3NDEKTSV4RRFFQ69G5FAV",
                30 + u64::from(index),
                15_030 + u64::from(index),
                ResourceCapacity::w4_builtin(),
            )
            .expect("lease");
            assert_eq!(selection.events.len(), 2);
            let leased = selection.task.expect("runnable task");
            let artifact = ArtifactId::from_digest(Sha256Digest::from_bytes([index; 32]));
            let response = artifact.to_string().into_bytes();
            let completed = job_store::complete_task(
                &mut connection,
                &leased.lease_id,
                artifact,
                &[index; 32],
                &response,
                40 + u64::from(index),
            )
            .expect("complete");
            assert_eq!(completed.response, response);
            assert_eq!(completed.events.len(), 1);
            assert!(completed.events[0].event_id > cursor);
            cursor = completed.events[0].event_id;
            let replayed = job_store::complete_task(
                &mut connection,
                &leased.lease_id,
                artifact,
                &[index; 32],
                b"response bytes are ignored on a matching durable retry",
                41 + u64::from(index),
            )
            .expect("completion retry");
            assert_eq!(replayed.response, response);
            assert!(replayed.events.is_empty());
            assert!(matches!(
                job_store::complete_task(
                    &mut connection,
                    &leased.lease_id,
                    artifact,
                    &[index.saturating_add(1); 32],
                    b"conflicting response",
                    42 + u64::from(index),
                ),
                Err(StoreError::Conflict)
            ));
        }
        let completed = job_store::get_job(&connection, &plan.job_id).expect("completed job");
        assert_eq!(completed.state, JobState::Succeeded as i32);
        assert!(
            completed
                .tasks
                .iter()
                .all(|task| task.state == TaskState::Succeeded as i32)
        );
        assert_eq!(completed.output_artifact_ids.len(), 1);
        assert_eq!(list_artifact_roots(&connection).expect("roots").len(), 1);

        let cancellable = JobPlan::demo(&project_id, b"cancel".to_vec(), 100);
        job_store::submit_job(&mut connection, "submit-cancel", &[4; 32], &cancellable)
            .expect("submit cancellable");
        let cancelled = job_store::cancel_job(
            &mut connection,
            "cancel",
            &[5; 32],
            &cancellable.job_id,
            101,
        )
        .expect("cancel");
        assert_eq!(cancelled.events.len(), 4);
        assert_eq!(
            job_store::get_job(&connection, &cancellable.job_id)
                .expect("cancelled job")
                .state,
            JobState::Cancelled as i32
        );
    }

    #[test]
    fn failed_worker_completions_replay_only_to_the_original_worker() {
        let temp = TempDir::new().expect("tempdir");
        let (_path, mut connection) = database(&temp);
        let project = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAV", "Failure replay", 10);
        create_project(&mut connection, "create-failure", &[1; 32], &project).expect("project");
        let project_id = project.project_id.parse().expect("project id");
        let plan = JobPlan::demo(&project_id, b"retryable".to_vec(), 20);
        job_store::submit_job(&mut connection, "submit-failure", &[2; 32], &plan).expect("submit");
        let lease = job_store::lease_next_task(
            &mut connection,
            &LeaseId::new().to_string(),
            "01ARZ3NDEKTSV4RRFFQ69G5FAV",
            30,
            15_030,
            ResourceCapacity::w4_builtin(),
        )
        .expect("lease")
        .task
        .expect("runnable task");
        let response = b"durable failure acknowledgement";
        let completed = job_store::complete_failed_task(
            &mut connection,
            &lease.lease_id,
            FailureClass::Transient as i32,
            "retryable fixture",
            &[7; 32],
            response,
            40,
        )
        .expect("record failed completion");
        assert_eq!(completed.response, response);
        assert_eq!(completed.events.len(), 1);
        assert_eq!(completed.events[0].state, TaskState::Retryable as i32);
        assert_eq!(
            job_store::replay_task_completion(
                &connection,
                &lease.lease_id,
                "builtin-fixture",
                &[7; 32],
            )
            .expect("replay"),
            Some(response.to_vec())
        );
        assert!(matches!(
            job_store::replay_task_completion(
                &connection,
                &lease.lease_id,
                "wrk_01J00000000000000000000000",
                &[7; 32],
            ),
            Err(StoreError::Conflict)
        ));
        assert!(matches!(
            job_store::replay_task_completion(
                &connection,
                &lease.lease_id,
                "builtin-fixture",
                &[8; 32],
            ),
            Err(StoreError::Conflict)
        ));
    }

    #[test]
    fn daemon_recovery_retries_without_spending_worker_failure_budget() {
        let temp = TempDir::new().expect("tempdir");
        let (_path, mut connection) = database(&temp);
        let project = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAV", "Recovery", 10);
        create_project(&mut connection, "create-recovery", &[1; 32], &project).expect("project");
        let project_id = project.project_id.parse().expect("project id");
        let plan = JobPlan::demo(&project_id, b"recover".to_vec(), 20);
        job_store::submit_job(&mut connection, "submit-recovery", &[2; 32], &plan).expect("submit");
        let old_lease = LeaseId::new().to_string();
        let selection = job_store::lease_next_task(
            &mut connection,
            &old_lease,
            "01ARZ3NDEKTSV4RRFFQ69G5FAV",
            30,
            15_030,
            ResourceCapacity::w4_builtin(),
        )
        .expect("lease");
        let leased = selection.task.expect("task");
        let recovery = job_store::recover_jobs(&mut connection, "01ARZ3NDEKTSV4RRFFQ69G5FAW", 31)
            .expect("recovery");
        assert_eq!(recovery.len(), 1);
        assert_eq!(recovery[0].state, TaskState::Retryable as i32);
        assert_eq!(recovery[0].attempt, 0);
        let artifact = ArtifactId::from_digest(Sha256Digest::from_bytes([9; 32]));
        assert!(matches!(
            job_store::complete_task(
                &mut connection,
                &leased.lease_id,
                artifact,
                &[9; 32],
                b"stale",
                32,
            ),
            Err(StoreError::Conflict)
        ));
        let selection = job_store::lease_next_task(
            &mut connection,
            &LeaseId::new().to_string(),
            "01ARZ3NDEKTSV4RRFFQ69G5FAW",
            33,
            15_033,
            ResourceCapacity::w4_builtin(),
        )
        .expect("retry lease");
        let retried = selection.task.expect("retry task");
        assert_eq!(retried.task_id, leased.task_id);
        assert_eq!(retried.attempt, 1);
    }

    #[test]
    fn lease_heartbeats_extend_ttl_and_stale_heartbeats_are_rejected() {
        let temp = TempDir::new().expect("tempdir");
        let (_path, mut connection) = database(&temp);
        let project = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAV", "Heartbeat", 10);
        create_project(&mut connection, "create-heartbeat", &[1; 32], &project).expect("project");
        let project_id = project.project_id.parse().expect("project id");
        let plan = JobPlan::demo(&project_id, b"heartbeat".to_vec(), 20);
        job_store::submit_job(&mut connection, "submit-heartbeat", &[2; 32], &plan)
            .expect("submit");
        let lease_id = LeaseId::new().to_string();
        let selection = job_store::lease_next_task(
            &mut connection,
            &lease_id,
            "01ARZ3NDEKTSV4RRFFQ69G5FAV",
            30,
            15_030,
            ResourceCapacity::w4_builtin(),
        )
        .expect("lease");
        assert!(selection.task.is_some());
        job_store::heartbeat_task(&mut connection, &lease_id, 10_000, 25_000).expect("heartbeat");
        assert!(
            job_store::expire_task_leases(&mut connection, 15_031, "01ARZ3NDEKTSV4RRFFQ69G5FAV")
                .expect("not expired")
                .is_empty()
        );
        let expired =
            job_store::expire_task_leases(&mut connection, 25_000, "01ARZ3NDEKTSV4RRFFQ69G5FAV")
                .expect("expired");
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].state, TaskState::Retryable as i32);
        assert!(matches!(
            job_store::heartbeat_task(&mut connection, &lease_id, 25_001, 40_001),
            Err(StoreError::Conflict)
        ));
    }

    #[test]
    fn terminal_lease_expiry_fails_the_job_and_cancels_remaining_tasks() {
        let temp = TempDir::new().expect("tempdir");
        let (_path, mut connection) = database(&temp);
        let project = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAV", "Expiry", 10);
        create_project(&mut connection, "create-expiry", &[1; 32], &project).expect("project");
        let project_id = project.project_id.parse().expect("project id");
        let mut plan = JobPlan::demo(&project_id, b"expiry".to_vec(), 20);
        plan.tasks[0].max_attempts = 1;
        job_store::submit_job(&mut connection, "submit-expiry", &[2; 32], &plan).expect("submit");
        let selection = job_store::lease_next_task(
            &mut connection,
            &LeaseId::new().to_string(),
            "01ARZ3NDEKTSV4RRFFQ69G5FAV",
            30,
            15_030,
            ResourceCapacity::w4_builtin(),
        )
        .expect("lease");
        assert!(selection.task.is_some());
        let events =
            job_store::expire_task_leases(&mut connection, 15_030, "01ARZ3NDEKTSV4RRFFQ69G5FAV")
                .expect("expire");
        assert_eq!(events.len(), 4);
        let failed = job_store::get_job(&connection, &plan.job_id).expect("failed job");
        assert_eq!(failed.state, JobState::Failed as i32);
        assert_eq!(failed.tasks[0].state, TaskState::Failed as i32);
        assert!(
            failed.tasks[1..]
                .iter()
                .all(|task| task.state == TaskState::Cancelled as i32)
        );
    }

    #[test]
    fn equivalent_deterministic_failures_open_a_durable_circuit_breaker() {
        let temp = TempDir::new().expect("tempdir");
        let (_path, mut connection) = database(&temp);
        let project = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAV", "Breaker", 10);
        create_project(&mut connection, "create-breaker", &[1; 32], &project).expect("project");
        let project_id = project.project_id.parse().expect("project id");
        for index in 0..3_u8 {
            let plan = JobPlan::demo(&project_id, b"same-input".to_vec(), 20 + u64::from(index));
            job_store::submit_job(
                &mut connection,
                &format!("submit-breaker-{index}"),
                &[index.saturating_add(2); 32],
                &plan,
            )
            .expect("submit breaker fixture");
            let selection = job_store::lease_next_task(
                &mut connection,
                &LeaseId::new().to_string(),
                "01ARZ3NDEKTSV4RRFFQ69G5FAV",
                30 + u64::from(index),
                15_030 + u64::from(index),
                ResourceCapacity::w4_builtin(),
            )
            .expect("lease breaker fixture");
            let leased = selection.task.expect("runnable breaker fixture");
            job_store::fail_task(
                &mut connection,
                &leased.lease_id,
                FailureClass::Deterministic as i32,
                "fixture deterministic failure",
                40 + u64::from(index),
            )
            .expect("fail breaker fixture");
        }

        let blocked = JobPlan::demo(&project_id, b"same-input".to_vec(), 100);
        job_store::submit_job(&mut connection, "submit-blocked", &[9; 32], &blocked)
            .expect("submit blocked fixture");
        let selection = job_store::lease_next_task(
            &mut connection,
            &LeaseId::new().to_string(),
            "01ARZ3NDEKTSV4RRFFQ69G5FAV",
            101,
            15_101,
            ResourceCapacity::w4_builtin(),
        )
        .expect("evaluate breaker");
        assert!(selection.task.is_none());
        assert_eq!(selection.events.len(), 4);
        let failed = job_store::get_job(&connection, &blocked.job_id).expect("blocked job");
        assert_eq!(failed.state, JobState::Failed as i32);
        assert_eq!(failed.failure_class, FailureClass::Deterministic as i32);
        assert!(failed.failure_detail.contains("circuit breaker open"));
    }

    #[test]
    fn cyclic_job_plan_rolls_back_without_claiming_request_id() {
        let temp = TempDir::new().expect("tempdir");
        let (_path, mut connection) = database(&temp);
        let project = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAV", "Cycle", 10);
        create_project(&mut connection, "create-cycle", &[1; 32], &project).expect("project");
        let project_id = project.project_id.parse::<ProjectId>().expect("project id");
        let mut cyclic = JobPlan::demo(&project_id, b"cycle".to_vec(), 20);
        let final_id = cyclic.tasks[3].task_id.clone();
        cyclic.tasks[0].dependencies.push(final_id);
        assert!(matches!(
            job_store::submit_job(&mut connection, "cycle", &[2; 32], &cyclic),
            Err(StoreError::InvalidData(_))
        ));
        let valid = JobPlan::demo(&project_id, b"valid".to_vec(), 21);
        job_store::submit_job(&mut connection, "cycle", &[2; 32], &valid)
            .expect("request id remains available");
        assert_eq!(
            job_store::list_jobs(&connection, &project.project_id)
                .expect("jobs")
                .len(),
            1
        );
    }

    #[test]
    fn failed_v1_upgrade_keeps_v1_transactionally_intact() {
        let temp = TempDir::new().expect("tempdir");
        let path = temp.path().join("v1-failure.db");
        let backups = temp.path().join("backups");
        let connection = Connection::open(&path).expect("open v1 database");
        connection
            .execute_batch(CREATE_V1_TABLES)
            .expect("v1 tables");
        connection
            .execute_batch(
                "CREATE TABLE project_artifact_roots (marker TEXT NOT NULL);
                 INSERT INTO project_artifact_roots(marker) VALUES ('prior');
                 PRAGMA application_id = 1129074765;
                 PRAGMA user_version = 1;",
            )
            .expect("conflicting v1 state");
        drop(connection);

        assert!(open_database(&path, &backups).is_err());
        let prior = Connection::open(&path).expect("reopen v1");
        let version: i64 = prior
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("v1 version remains");
        let marker: String = prior
            .query_row("SELECT marker FROM project_artifact_roots", [], |row| {
                row.get(0)
            })
            .expect("prior table remains");
        assert_eq!(version, 1);
        assert_eq!(marker, "prior");
        assert_eq!(fs::read_dir(backups).expect("backup retained").count(), 1);
    }

    #[test]
    fn failed_v3_upgrade_keeps_orchestration_schema_intact() {
        let temp = TempDir::new().expect("tempdir");
        let path = temp.path().join("v3-failure.db");
        let backups = temp.path().join("backups");
        let connection = Connection::open(&path).expect("open v3 database");
        connection
            .execute_batch(CREATE_V1_TABLES)
            .expect("v1 schema");
        connection
            .execute_batch(CREATE_V2_TABLES)
            .expect("v2 schema");
        connection
            .execute_batch(job_store::CREATE_V3_TABLES)
            .expect("v3 schema");
        connection
            .execute_batch(
                "CREATE TABLE sources (marker TEXT NOT NULL);
                 INSERT INTO sources(marker) VALUES ('prior-v3');
                 PRAGMA application_id = 1129074765;
                 PRAGMA user_version = 3;",
            )
            .expect("conflicting v3 state");
        drop(connection);

        assert!(open_database(&path, &backups).is_err());
        let prior = Connection::open(&path).expect("reopen v3");
        let version: i64 = prior
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("v3 version remains");
        let marker: String = prior
            .query_row("SELECT marker FROM sources", [], |row| row.get(0))
            .expect("prior source table remains");
        let source_roots: i64 = prior
            .query_row(
                "SELECT count(*) FROM sqlite_master
                 WHERE type = 'table' AND name = 'source_artifact_roots'",
                [],
                |row| row.get(0),
            )
            .expect("source roots count");
        assert_eq!(version, 3);
        assert_eq!(marker, "prior-v3");
        assert_eq!(source_roots, 0);
        assert_eq!(fs::read_dir(backups).expect("backup retained").count(), 1);
    }

    #[test]
    fn failed_migration_rolls_back_to_the_prior_schema() {
        let temp = TempDir::new().expect("tempdir");
        let path = temp.path().join("migration-failure.db");
        let connection = Connection::open(&path).expect("open raw database");
        connection
            .execute_batch(
                "CREATE TABLE projects (marker TEXT NOT NULL);\n\
                 INSERT INTO projects(marker) VALUES ('prior');",
            )
            .expect("create prior schema");
        drop(connection);

        assert!(open_database(&path, &temp.path().join("backups")).is_err());

        let connection = Connection::open(&path).expect("reopen prior database");
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("schema version");
        let application_id: i64 = connection
            .query_row("PRAGMA application_id", [], |row| row.get(0))
            .expect("application id");
        let marker: String = connection
            .query_row("SELECT marker FROM projects", [], |row| row.get(0))
            .expect("prior row");
        let dedup_tables: i64 = connection
            .query_row(
                "SELECT count(*) FROM sqlite_master\n\
                 WHERE type = 'table' AND name = 'request_dedup'",
                [],
                |row| row.get(0),
            )
            .expect("dedup table count");
        assert_eq!(version, 0);
        assert_eq!(application_id, 0);
        assert_eq!(marker, "prior");
        assert_eq!(dedup_tables, 0);
    }

    #[test]
    fn project_crud_and_ordering_are_deterministic() {
        let temp = TempDir::new().expect("tempdir");
        let (_path, mut connection) = database(&temp);
        let first = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAV", "First", 10);
        let second = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAW", "First", 20);
        create_project(&mut connection, "create-1", &[1; 32], &first).expect("create first");
        create_project(&mut connection, "create-2", &[2; 32], &second).expect("create second");

        assert_eq!(
            get_project(&connection, &first.project_id).expect("get"),
            first
        );
        assert_eq!(
            list_projects(&connection).expect("list"),
            vec![second, first]
        );

        delete_project(
            &mut connection,
            "delete-1",
            &[3; 32],
            "prj_01ARZ3NDEKTSV4RRFFQ69G5FAV",
            30,
        )
        .expect("delete");
        assert!(matches!(
            get_project(&connection, "prj_01ARZ3NDEKTSV4RRFFQ69G5FAV"),
            Err(StoreError::NotFound)
        ));
    }

    #[test]
    fn source_records_are_idempotent_ordered_and_cascade_with_projects() {
        let temp = TempDir::new().expect("tempdir");
        let (_path, mut connection) = database(&temp);
        let project = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAV", "Sources", 10);
        create_project(&mut connection, "create-sources", &[1; 32], &project).expect("project");
        let source_id = SourceId::new().to_string();
        let inspection = InspectedSource {
            observation: FileObservation {
                absolute_path: "/private/media/source.mkv".to_owned(),
                byte_size: 123,
                sample_sha256: format!("sha256:{}", "11".repeat(32)),
                device_id: 1,
                inode: 2,
                modified_unix_nanos: 3,
            },
            source_fingerprint: format!("sha256:{}", "22".repeat(32)),
            source_map_json: b"{\"schema_version\":\"clipmill.source_map.v1\"}".to_vec(),
        };
        let first = source_store::register_source(
            &mut connection,
            "register-source",
            &[2; 32],
            &project.project_id,
            &source_id,
            &inspection,
            20,
        )
        .expect("register source");
        let replay = source_store::register_source(
            &mut connection,
            "register-source",
            &[2; 32],
            &project.project_id,
            &SourceId::new().to_string(),
            &inspection,
            21,
        )
        .expect("replay source registration");
        assert_eq!(first, replay);
        let fetched = source_store::get_source(&connection, &source_id).expect("get source");
        assert_eq!(fetched.observation, inspection.observation);
        assert_eq!(
            source_store::list_sources(&connection, &project.project_id)
                .expect("list sources")
                .len(),
            1
        );
        let artifact = ArtifactId::from_digest(Sha256Digest::from_bytes([0x33; 32]));
        connection
            .execute(
                "INSERT INTO source_artifact_roots(source_id, artifact_id) VALUES (?1, ?2)",
                params![source_id, artifact.to_string()],
            )
            .expect("source root");
        assert!(
            list_artifact_roots(&connection)
                .expect("GC roots")
                .contains(&artifact)
        );
        delete_project(
            &mut connection,
            "delete-source-project",
            &[3; 32],
            &project.project_id,
            30,
        )
        .expect("delete project");
        let sources: i64 = connection
            .query_row("SELECT count(*) FROM sources", [], |row| row.get(0))
            .expect("source count");
        let roots: i64 = connection
            .query_row("SELECT count(*) FROM source_artifact_roots", [], |row| {
                row.get(0)
            })
            .expect("source root count");
        assert_eq!(sources, 0);
        assert_eq!(roots, 0);
    }

    #[test]
    fn artifact_roots_are_idempotent_and_cascade_with_projects() {
        let temp = TempDir::new().expect("tempdir");
        let (_path, mut connection) = database(&temp);
        let project = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAV", "Rooted", 10);
        create_project(&mut connection, "create-rooted", &[1; 32], &project)
            .expect("create project");
        let artifact = ArtifactId::from_digest(Sha256Digest::from_bytes([0xab; 32]));
        attach_artifact_root(&mut connection, &project.project_id, &artifact.to_string())
            .expect("attach");
        attach_artifact_root(&mut connection, &project.project_id, &artifact.to_string())
            .expect("idempotent attach");
        assert_eq!(
            list_artifact_roots(&connection).expect("roots"),
            vec![artifact]
        );

        delete_project(
            &mut connection,
            "delete-rooted",
            &[2; 32],
            &project.project_id,
            20,
        )
        .expect("delete project");
        assert!(
            list_artifact_roots(&connection)
                .expect("roots after delete")
                .is_empty()
        );
    }

    #[test]
    fn successful_mutation_replays_exact_bytes_and_rejects_collision() {
        let temp = TempDir::new().expect("tempdir");
        let (_path, mut connection) = database(&temp);
        let original = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAV", "Original", 10);
        let different = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAW", "Different", 20);

        let first =
            create_project(&mut connection, "same", &[9; 32], &original).expect("first response");
        let replayed = create_project(&mut connection, "same", &[9; 32], &different)
            .expect("replayed response");
        assert_eq!(first, replayed);
        assert!(clipmill_contracts::proto::ipc::v1::Response::decode(first.as_slice()).is_ok());
        assert!(matches!(
            create_project(&mut connection, "same", &[8; 32], &different),
            Err(StoreError::Conflict)
        ));
        assert_eq!(list_projects(&connection).expect("list"), vec![original]);
    }

    #[test]
    fn failed_mutation_rolls_back_without_claiming_request_id() {
        let temp = TempDir::new().expect("tempdir");
        let (_path, mut connection) = database(&temp);
        let original = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAV", "Original", 10);
        create_project(&mut connection, "original", &[1; 32], &original).expect("create original");

        let duplicate = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAV", "Duplicate", 20);
        assert!(matches!(
            create_project(&mut connection, "retryable", &[2; 32], &duplicate),
            Err(StoreError::Database(_))
        ));

        let retry = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAW", "Retry", 30);
        create_project(&mut connection, "retryable", &[2; 32], &retry)
            .expect("request id remains available after rollback");
        assert_eq!(list_projects(&connection).expect("list").len(), 2);
    }

    #[test]
    fn database_file_is_private() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let temp = TempDir::new().expect("tempdir");
            let (path, _connection) = database(&temp);
            let mode = std::fs::metadata(Path::new(&path))
                .expect("metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
        }
    }
}
