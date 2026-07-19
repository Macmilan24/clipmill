use std::time::{SystemTime, UNIX_EPOCH};

use clipmill_contracts::proto::ipc::v1::{
    CreateProjectRequest, DemoDagPayloadV1, Error, ErrorCode, GetJobResponse, GetProjectResponse,
    GetSourceResponse, HealthResponse, ListJobsResponse, ListProjectsResponse, ListSourcesResponse,
    PingResponse, ProbeSourcePayloadV1, RegisterSourceRequest, Request, Response, SubmitJobRequest,
    SubscribeTaskEventsRequest, SubscribeTaskEventsResponse, request, response,
};
use clipmill_core::{JobId, ProjectId, SourceId, TaskEventCursor};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::db::{DbHandle, ProjectRecord, StoreError};
use crate::jobs::{EventFilter, TaskEventRecord};
use crate::jobs::{EventHub, JobPlan, SchedulerHandle};
use crate::sources::{SourceInspector, SourceProbeError};
use tokio::sync::broadcast;

const REQUEST_ID_MAX_CHARS: usize = 128;
const PROJECT_NAME_MAX_CHARS: usize = 200;
const DEMO_DAG_KEY_VERSION: &str = "clipmill.demo-dag.v1";
const PROBE_SOURCE_KEY_VERSION: &str = "clipmill.probe-source.v1";

#[derive(Clone, Debug)]
pub(crate) struct Service {
    database: DbHandle,
    started_unix_millis: u64,
    events: EventHub,
    scheduler: Option<SchedulerHandle>,
    sources: Option<SourceInspector>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Outcome {
    Success,
    InvalidArgument,
    NotFound,
    Conflict,
    Unavailable,
    Internal,
}

impl Outcome {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::InvalidArgument => "invalid_argument",
            Self::NotFound => "not_found",
            Self::Conflict => "conflict",
            Self::Unavailable => "unavailable",
            Self::Internal => "internal",
        }
    }
}

#[derive(Debug)]
pub(crate) struct Reply {
    pub bytes: Vec<u8>,
    pub outcome: Outcome,
}

#[derive(Debug)]
pub(crate) struct Subscription {
    pub request_id: String,
    pub ack: Vec<u8>,
    pub history: Vec<TaskEventRecord>,
    pub live: broadcast::Receiver<TaskEventRecord>,
    pub filter: EventFilter,
    pub after_event_id: u64,
}

impl Service {
    #[cfg(test)]
    pub(crate) fn new(database: DbHandle, started_unix_millis: u64) -> Self {
        Self {
            database,
            started_unix_millis,
            events: EventHub::new(),
            scheduler: None,
            sources: None,
        }
    }

    pub(crate) fn with_scheduler(
        database: DbHandle,
        started_unix_millis: u64,
        events: EventHub,
        scheduler: SchedulerHandle,
        sources: SourceInspector,
    ) -> Self {
        Self {
            database,
            started_unix_millis,
            events,
            scheduler: Some(scheduler),
            sources: Some(sources),
        }
    }

    #[must_use]
    pub(crate) fn event_hub(&self) -> EventHub {
        self.events.clone()
    }

    pub(crate) async fn subscribe(
        &self,
        request_id: String,
        request: &SubscribeTaskEventsRequest,
    ) -> Result<Subscription, Reply> {
        if let Err(message) = validate_request_id(&request_id) {
            return Err(error_reply(request_id, ErrorCode::InvalidArgument, message));
        }
        let project_id = if request.project_id.is_empty() {
            None
        } else {
            match request.project_id.parse::<ProjectId>() {
                Ok(value) => Some(value.to_string()),
                Err(error) => {
                    return Err(error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        error.to_string(),
                    ));
                }
            }
        };
        let job_id = if request.job_id.is_empty() {
            None
        } else {
            match request.job_id.parse::<JobId>() {
                Ok(value) => Some(value.to_string()),
                Err(error) => {
                    return Err(error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        error.to_string(),
                    ));
                }
            }
        };
        if let Some(job_id) = &job_id {
            let job = self
                .database
                .get_job(job_id.clone())
                .await
                .map_err(|error| store_error_reply(request_id.clone(), &error))?;
            if let Some(project_id) = &project_id
                && project_id != &job.project_id
            {
                return Err(error_reply(
                    request_id,
                    ErrorCode::InvalidArgument,
                    "job does not belong to the requested project",
                ));
            }
        } else if let Some(project_id) = &project_id {
            self.database
                .get_project(project_id.clone())
                .await
                .map_err(|error| store_error_reply(request_id.clone(), &error))?;
        }
        let filter = EventFilter { project_id, job_id };
        let after_event_id = match TaskEventCursor::try_from(request.after_event_id) {
            Ok(cursor) => cursor.get(),
            Err(error) => {
                return Err(error_reply(
                    request_id,
                    ErrorCode::InvalidArgument,
                    error.to_string(),
                ));
            }
        };
        let live = self.events.subscribe();
        let current_event_id = self
            .database
            .current_event_id()
            .await
            .map_err(|error| store_error_reply(request_id.clone(), &error))?;
        let history = self
            .database
            .list_events(after_event_id, filter.clone())
            .await
            .map_err(|error| store_error_reply(request_id.clone(), &error))?;
        let ack = Response {
            request_id: request_id.clone(),
            body: Some(response::Body::SubscribeTaskEvents(
                SubscribeTaskEventsResponse { current_event_id },
            )),
        }
        .encode_to_vec();
        Ok(Subscription {
            request_id,
            ack,
            history,
            live,
            filter,
            after_event_id,
        })
    }

    pub(crate) async fn handle(&self, request: Request) -> Reply {
        let request_id = request.request_id.clone();
        if let Err(message) = validate_request_id(&request_id) {
            return error_reply(request_id, ErrorCode::InvalidArgument, message);
        }
        let request_hash: [u8; 32] = Sha256::digest(request.encode_to_vec()).into();
        let Some(body) = request.body else {
            return error_reply(
                request_id,
                ErrorCode::InvalidArgument,
                "request body is required",
            );
        };

        match body {
            request::Body::Ping(ping) => response_reply(
                request_id,
                response::Body::Ping(PingResponse {
                    echo: ping.echo,
                    daemon_version: env!("CARGO_PKG_VERSION").to_owned(),
                }),
            ),
            request::Body::Health(_) => response_reply(
                request_id,
                response::Body::Health(HealthResponse {
                    daemon_version: env!("CARGO_PKG_VERSION").to_owned(),
                    started_unix_millis: self.started_unix_millis,
                    local_lock: true,
                }),
            ),
            request::Body::CreateProject(create) => {
                self.create_project(request_id, request_hash, &create).await
            }
            request::Body::GetProject(get) => self.get_project(request_id, &get.project_id).await,
            request::Body::ListProjects(_) => match self.database.list_projects().await {
                Ok(projects) => response_reply(
                    request_id,
                    response::Body::ListProjects(ListProjectsResponse {
                        projects: projects.into_iter().map(Into::into).collect(),
                    }),
                ),
                Err(error) => store_error_reply(request_id, &error),
            },
            request::Body::DeleteProject(delete) => {
                self.delete_project(request_id, request_hash, &delete.project_id)
                    .await
            }
            request::Body::SubmitJob(submit) => {
                self.submit_job(request_id, request_hash, &submit).await
            }
            request::Body::GetJob(get) => self.get_job(request_id, &get.job_id).await,
            request::Body::ListJobs(list) => self.list_jobs(request_id, &list.project_id).await,
            request::Body::CancelJob(cancel) => {
                self.cancel_job(request_id, request_hash, &cancel.job_id)
                    .await
            }
            request::Body::RegisterSource(register) => {
                self.register_source(request_id, request_hash, &register)
                    .await
            }
            request::Body::GetSource(get) => self.get_source(request_id, &get.source_id).await,
            request::Body::ListSources(list) => {
                self.list_sources(request_id, &list.project_id).await
            }
            request::Body::SubscribeTaskEvents(_) | request::Body::GetDeviceProfile(_) => {
                error_reply(
                    request_id,
                    ErrorCode::Unavailable,
                    "operation is not available",
                )
            }
        }
    }

    async fn create_project(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        create: &CreateProjectRequest,
    ) -> Reply {
        let name = match validate_project_name(&create.name) {
            Ok(name) => name,
            Err(message) => {
                return error_reply(request_id, ErrorCode::InvalidArgument, message);
            }
        };
        let now = match unix_millis() {
            Ok(now) => now,
            Err(message) => return error_reply(request_id, ErrorCode::Internal, message),
        };
        let project = ProjectRecord {
            project_id: ProjectId::new().to_string(),
            name,
            created_unix_millis: now,
        };
        match self
            .database
            .create_project(request_id.clone(), request_hash, project)
            .await
        {
            Ok(bytes) => Reply {
                bytes,
                outcome: Outcome::Success,
            },
            Err(error) => store_error_reply(request_id, &error),
        }
    }

    async fn get_project(&self, request_id: String, value: &str) -> Reply {
        let project_id = match value.parse::<ProjectId>() {
            Ok(project_id) => project_id,
            Err(error) => {
                return error_reply(request_id, ErrorCode::InvalidArgument, error.to_string());
            }
        };
        match self.database.get_project(project_id.to_string()).await {
            Ok(project) => response_reply(
                request_id,
                response::Body::GetProject(GetProjectResponse {
                    project: Some(project.into()),
                }),
            ),
            Err(error) => store_error_reply(request_id, &error),
        }
    }

    async fn delete_project(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        value: &str,
    ) -> Reply {
        let project_id = match value.parse::<ProjectId>() {
            Ok(project_id) => project_id,
            Err(error) => {
                return error_reply(request_id, ErrorCode::InvalidArgument, error.to_string());
            }
        };
        let now = match unix_millis() {
            Ok(now) => now,
            Err(message) => return error_reply(request_id, ErrorCode::Internal, message),
        };
        match self
            .database
            .delete_project(
                request_id.clone(),
                request_hash,
                project_id.to_string(),
                now,
            )
            .await
        {
            Ok(bytes) => Reply {
                bytes,
                outcome: Outcome::Success,
            },
            Err(error) => store_error_reply(request_id, &error),
        }
    }

    #[allow(clippy::too_many_lines)]
    async fn submit_job(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        submit: &SubmitJobRequest,
    ) -> Reply {
        let project_id = match submit.project_id.parse::<ProjectId>() {
            Ok(project_id) => project_id,
            Err(error) => {
                return error_reply(request_id, ErrorCode::InvalidArgument, error.to_string());
            }
        };
        if submit.payload.len() > 72 * 1024 {
            return error_reply(
                request_id,
                ErrorCode::InvalidArgument,
                "encoded job payload exceeds 72 KiB",
            );
        }
        let now = match unix_millis() {
            Ok(now) => now,
            Err(message) => return error_reply(request_id, ErrorCode::Internal, message),
        };
        let plan = match submit.kind.as_str() {
            "demo-dag" => {
                let Ok(payload) = DemoDagPayloadV1::decode(submit.payload.as_slice()) else {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "demo job payload is not a valid DemoDagPayloadV1",
                    );
                };
                if payload.key_version != DEMO_DAG_KEY_VERSION {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "demo job payload key_version is unsupported",
                    );
                }
                if payload.seed.len() > 64 * 1024 {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "demo job seed exceeds 64 KiB",
                    );
                }
                JobPlan::demo(&project_id, payload.seed, now)
            }
            "probe-source" => {
                let Ok(payload) = ProbeSourcePayloadV1::decode(submit.payload.as_slice()) else {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "probe job payload is not a valid ProbeSourcePayloadV1",
                    );
                };
                if payload.key_version != PROBE_SOURCE_KEY_VERSION {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "probe job payload key_version is unsupported",
                    );
                }
                let source_id = match payload.source_id.parse::<SourceId>() {
                    Ok(value) => value,
                    Err(error) => {
                        return error_reply(
                            request_id,
                            ErrorCode::InvalidArgument,
                            error.to_string(),
                        );
                    }
                };
                let source = match self.database.get_source(source_id.to_string()).await {
                    Ok(source) => source,
                    Err(error) => return store_error_reply(request_id, &error),
                };
                if source.project_id != project_id.as_str() {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "source does not belong to the requested project",
                    );
                }
                JobPlan::probe_source(
                    &project_id,
                    source_id.to_string(),
                    submit.payload.clone(),
                    now,
                )
            }
            _ => {
                return error_reply(
                    request_id,
                    ErrorCode::Unavailable,
                    "job kind is not available in W5",
                );
            }
        };
        match self
            .database
            .submit_job(request_id.clone(), request_hash, plan)
            .await
        {
            Ok(result) => {
                self.events.publish_all(result.events);
                if let Some(scheduler) = &self.scheduler {
                    scheduler.notify();
                }
                Reply {
                    bytes: result.bytes,
                    outcome: Outcome::Success,
                }
            }
            Err(error) => store_error_reply(request_id, &error),
        }
    }

    async fn register_source(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        register: &RegisterSourceRequest,
    ) -> Reply {
        let project_id = match register.project_id.parse::<ProjectId>() {
            Ok(value) => value,
            Err(error) => {
                return error_reply(request_id, ErrorCode::InvalidArgument, error.to_string());
            }
        };
        if let Err(error) = self.database.get_project(project_id.to_string()).await {
            return store_error_reply(request_id, &error);
        }
        let Some(inspector) = &self.sources else {
            return error_reply(
                request_id,
                ErrorCode::Unavailable,
                "source inspection is not available",
            );
        };
        let sampled = match inspector.sample(register.absolute_path.clone()).await {
            Ok(value) => value,
            Err(error) => return source_probe_error_reply(request_id, &error),
        };
        let existing = self
            .database
            .find_source_observation(project_id.to_string(), sampled.observation().clone())
            .await;
        match existing {
            Ok(Some(source)) => {
                let now = unix_millis().unwrap_or(source.created_unix_millis);
                match self
                    .database
                    .remember_source_hit(request_id.clone(), request_hash, source, now)
                    .await
                {
                    Ok(bytes) => Reply {
                        bytes,
                        outcome: Outcome::Success,
                    },
                    Err(error) => store_error_reply(request_id, &error),
                }
            }
            Ok(None) => {
                let inspection = match inspector.complete(sampled).await {
                    Ok(value) => value,
                    Err(error) => return source_probe_error_reply(request_id, &error),
                };
                let now = match unix_millis() {
                    Ok(value) => value,
                    Err(message) => {
                        return error_reply(request_id, ErrorCode::Internal, message);
                    }
                };
                match self
                    .database
                    .register_source(
                        request_id.clone(),
                        request_hash,
                        project_id.to_string(),
                        SourceId::new().to_string(),
                        inspection,
                        now,
                    )
                    .await
                {
                    Ok(bytes) => Reply {
                        bytes,
                        outcome: Outcome::Success,
                    },
                    Err(error) => store_error_reply(request_id, &error),
                }
            }
            Err(error) => store_error_reply(request_id, &error),
        }
    }

    async fn get_source(&self, request_id: String, value: &str) -> Reply {
        let source_id = match value.parse::<SourceId>() {
            Ok(value) => value,
            Err(error) => {
                return error_reply(request_id, ErrorCode::InvalidArgument, error.to_string());
            }
        };
        match self.database.get_source(source_id.to_string()).await {
            Ok(source) => response_reply(
                request_id,
                response::Body::GetSource(GetSourceResponse {
                    source: Some(source.into()),
                }),
            ),
            Err(error) => store_error_reply(request_id, &error),
        }
    }

    async fn list_sources(&self, request_id: String, value: &str) -> Reply {
        let project_id = match value.parse::<ProjectId>() {
            Ok(value) => value,
            Err(error) => {
                return error_reply(request_id, ErrorCode::InvalidArgument, error.to_string());
            }
        };
        match self.database.list_sources(project_id.to_string()).await {
            Ok(sources) => response_reply(
                request_id,
                response::Body::ListSources(ListSourcesResponse {
                    sources: sources.into_iter().map(Into::into).collect(),
                }),
            ),
            Err(error) => store_error_reply(request_id, &error),
        }
    }

    async fn get_job(&self, request_id: String, value: &str) -> Reply {
        let job_id = match value.parse::<JobId>() {
            Ok(job_id) => job_id,
            Err(error) => {
                return error_reply(request_id, ErrorCode::InvalidArgument, error.to_string());
            }
        };
        match self.database.get_job(job_id.to_string()).await {
            Ok(job) => response_reply(
                request_id,
                response::Body::GetJob(GetJobResponse {
                    job: Some(job.into()),
                }),
            ),
            Err(error) => store_error_reply(request_id, &error),
        }
    }

    async fn list_jobs(&self, request_id: String, value: &str) -> Reply {
        let project_id = match value.parse::<ProjectId>() {
            Ok(project_id) => project_id,
            Err(error) => {
                return error_reply(request_id, ErrorCode::InvalidArgument, error.to_string());
            }
        };
        match self.database.list_jobs(project_id.to_string()).await {
            Ok(jobs) => response_reply(
                request_id,
                response::Body::ListJobs(ListJobsResponse {
                    jobs: jobs.into_iter().map(Into::into).collect(),
                }),
            ),
            Err(error) => store_error_reply(request_id, &error),
        }
    }

    async fn cancel_job(&self, request_id: String, request_hash: [u8; 32], value: &str) -> Reply {
        let job_id = match value.parse::<JobId>() {
            Ok(job_id) => job_id,
            Err(error) => {
                return error_reply(request_id, ErrorCode::InvalidArgument, error.to_string());
            }
        };
        let now = match unix_millis() {
            Ok(now) => now,
            Err(message) => return error_reply(request_id, ErrorCode::Internal, message),
        };
        match self
            .database
            .cancel_job(request_id.clone(), request_hash, job_id.to_string(), now)
            .await
        {
            Ok(result) => {
                self.events.publish_all(result.events);
                if let Some(scheduler) = &self.scheduler {
                    scheduler.notify();
                }
                Reply {
                    bytes: result.bytes,
                    outcome: Outcome::Success,
                }
            }
            Err(error) => store_error_reply(request_id, &error),
        }
    }
}

pub(crate) fn request_kind(request: &Request) -> &'static str {
    match request.body.as_ref() {
        Some(request::Body::Ping(_)) => "ping",
        Some(request::Body::Health(_)) => "health",
        Some(request::Body::CreateProject(_)) => "create_project",
        Some(request::Body::GetProject(_)) => "get_project",
        Some(request::Body::ListProjects(_)) => "list_projects",
        Some(request::Body::DeleteProject(_)) => "delete_project",
        Some(request::Body::SubmitJob(_)) => "submit_job",
        Some(request::Body::SubscribeTaskEvents(_)) => "subscribe_task_events",
        Some(request::Body::GetDeviceProfile(_)) => "get_device_profile",
        Some(request::Body::GetJob(_)) => "get_job",
        Some(request::Body::ListJobs(_)) => "list_jobs",
        Some(request::Body::CancelJob(_)) => "cancel_job",
        Some(request::Body::RegisterSource(_)) => "register_source",
        Some(request::Body::GetSource(_)) => "get_source",
        Some(request::Body::ListSources(_)) => "list_sources",
        None => "missing_body",
    }
}

fn response_reply(request_id: String, body: response::Body) -> Reply {
    Reply {
        bytes: Response {
            request_id,
            body: Some(body),
        }
        .encode_to_vec(),
        outcome: Outcome::Success,
    }
}

fn error_reply(request_id: String, code: ErrorCode, message: impl Into<String>) -> Reply {
    let outcome = match code {
        ErrorCode::InvalidArgument => Outcome::InvalidArgument,
        ErrorCode::NotFound => Outcome::NotFound,
        ErrorCode::Conflict => Outcome::Conflict,
        ErrorCode::Unavailable => Outcome::Unavailable,
        ErrorCode::Unspecified | ErrorCode::PolicyDenied | ErrorCode::Internal => Outcome::Internal,
    };
    Reply {
        bytes: Response {
            request_id,
            body: Some(response::Body::Error(Error {
                code: code as i32,
                message: message.into(),
            })),
        }
        .encode_to_vec(),
        outcome,
    }
}

fn store_error_reply(request_id: String, error: &StoreError) -> Reply {
    match error {
        StoreError::Conflict => error_reply(request_id, ErrorCode::Conflict, error.to_string()),
        StoreError::NotFound => error_reply(request_id, ErrorCode::NotFound, error.to_string()),
        StoreError::Database(_) | StoreError::InvalidData(_) | StoreError::Stopped => {
            error_reply(request_id, ErrorCode::Internal, "internal database error")
        }
    }
}

fn source_probe_error_reply(request_id: String, error: &SourceProbeError) -> Reply {
    match error {
        SourceProbeError::InvalidPath(_) | SourceProbeError::ProbeFailed(_) => {
            error_reply(request_id, ErrorCode::InvalidArgument, error.to_string())
        }
        SourceProbeError::SourceChanged => {
            error_reply(request_id, ErrorCode::Conflict, error.to_string())
        }
        SourceProbeError::Timeout | SourceProbeError::OutputLimit => {
            error_reply(request_id, ErrorCode::Unavailable, error.to_string())
        }
        SourceProbeError::Io(_) | SourceProbeError::InvalidProbe(_) | SourceProbeError::Stopped => {
            error_reply(request_id, ErrorCode::Internal, "source inspection failed")
        }
    }
}

fn validate_request_id(request_id: &str) -> Result<(), &'static str> {
    let count = request_id.chars().count();
    if count == 0 {
        return Err("request_id is required");
    }
    if count > REQUEST_ID_MAX_CHARS {
        return Err("request_id exceeds 128 characters");
    }
    if request_id.chars().any(char::is_control) {
        return Err("request_id contains control characters");
    }
    Ok(())
}

fn validate_project_name(value: &str) -> Result<String, &'static str> {
    let trimmed = value.trim();
    let count = trimmed.chars().count();
    if count == 0 {
        return Err("project name is required");
    }
    if count > PROJECT_NAME_MAX_CHARS {
        return Err("project name exceeds 200 characters");
    }
    if trimmed.chars().any(char::is_control) {
        return Err("project name contains control characters");
    }
    Ok(trimmed.to_owned())
}

fn unix_millis() -> Result<u64, &'static str> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "system clock is before the Unix epoch")?;
    u64::try_from(duration.as_millis()).map_err(|_| "system clock exceeds timestamp range")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use clipmill_contracts::proto::ipc::v1::{
        CreateProjectRequest, DemoDagPayloadV1, GetDeviceProfileRequest, Request, Response,
        SubmitJobRequest, SubscribeTaskEventsRequest, request, response,
    };
    use clipmill_core::ProjectId;
    use prost::Message;
    use tempfile::TempDir;

    use super::{Service, validate_project_name, validate_request_id};
    use crate::db::DbActor;

    #[test]
    fn validates_and_trims_project_names() {
        assert_eq!(
            validate_project_name("  My project  "),
            Ok("My project".to_owned())
        );
        assert!(validate_project_name(" \n ").is_err());
        assert!(validate_project_name("bad\u{0000}name").is_err());
        assert!(validate_project_name(&"x".repeat(201)).is_err());
        assert_eq!(
            validate_project_name("  你好 🎬  "),
            Ok("你好 🎬".to_owned())
        );
        assert!(validate_project_name(&"🎬".repeat(200)).is_ok());
        assert!(validate_project_name(&"🎬".repeat(201)).is_err());
    }

    #[test]
    fn validates_request_ids() {
        assert!(validate_request_id("req_1").is_ok());
        assert!(validate_request_id("").is_err());
        assert!(validate_request_id("bad\nrequest").is_err());
        assert!(validate_request_id(&"x".repeat(129)).is_err());
    }

    #[tokio::test]
    async fn create_retry_returns_same_project_and_conflict_is_reported() {
        let temp = TempDir::new().expect("tempdir");
        let database = temp.path().join("clipmill.db");
        let actor =
            DbActor::start(&database, &temp.path().join("backups")).expect("database actor");
        let service = Service::new(actor.handle(), 1);
        let request = Request {
            request_id: "same-request".to_owned(),
            body: Some(request::Body::CreateProject(CreateProjectRequest {
                name: "Project".to_owned(),
            })),
        };
        let first = service.handle(request.clone()).await;
        let replay = service.handle(request).await;
        assert_eq!(first.bytes, replay.bytes);

        let conflict = service
            .handle(Request {
                request_id: "same-request".to_owned(),
                body: Some(request::Body::CreateProject(CreateProjectRequest {
                    name: "Different".to_owned(),
                })),
            })
            .await;
        let decoded = Response::decode(conflict.bytes.as_slice()).expect("decode response");
        assert!(matches!(decoded.body, Some(response::Body::Error(error)) if error.code == 3));
        actor.shutdown().await.expect("shutdown");
    }

    #[tokio::test]
    async fn future_operations_are_unavailable() {
        let temp = TempDir::new().expect("tempdir");
        let database = temp.path().join("clipmill.db");
        let actor =
            DbActor::start(&database, &temp.path().join("backups")).expect("database actor");
        let service = Service::new(actor.handle(), 1);
        let reply = service
            .handle(Request {
                request_id: "future".to_owned(),
                body: Some(request::Body::GetDeviceProfile(GetDeviceProfileRequest {
                    remeasure: false,
                })),
            })
            .await;
        let decoded = Response::decode(reply.bytes.as_slice()).expect("decode response");
        assert!(matches!(decoded.body, Some(response::Body::Error(error)) if error.code == 4));
        actor.shutdown().await.expect("shutdown");
    }

    #[tokio::test]
    async fn demo_payload_and_event_cursor_are_validated_at_the_boundary() {
        let temp = TempDir::new().expect("tempdir");
        let actor = DbActor::start(
            &temp.path().join("clipmill.db"),
            &temp.path().join("backups"),
        )
        .expect("database actor");
        let service = Service::new(actor.handle(), 1);
        let malformed = service
            .handle(Request {
                request_id: "malformed-demo".to_owned(),
                body: Some(request::Body::SubmitJob(SubmitJobRequest {
                    project_id: ProjectId::new().to_string(),
                    kind: "demo-dag".to_owned(),
                    payload: vec![0xff],
                })),
            })
            .await;
        let decoded = Response::decode(malformed.bytes.as_slice()).expect("decode response");
        assert!(matches!(decoded.body, Some(response::Body::Error(error)) if error.code == 1));

        let wrong_version = service
            .handle(Request {
                request_id: "wrong-demo-version".to_owned(),
                body: Some(request::Body::SubmitJob(SubmitJobRequest {
                    project_id: ProjectId::new().to_string(),
                    kind: "demo-dag".to_owned(),
                    payload: DemoDagPayloadV1 {
                        key_version: "clipmill.demo-dag.v2".to_owned(),
                        seed: Vec::new(),
                    }
                    .encode_to_vec(),
                })),
            })
            .await;
        let decoded = Response::decode(wrong_version.bytes.as_slice()).expect("decode response");
        assert!(matches!(decoded.body, Some(response::Body::Error(error)) if error.code == 1));

        let cursor = service
            .subscribe(
                "bad-cursor".to_owned(),
                &SubscribeTaskEventsRequest {
                    project_id: String::new(),
                    job_id: String::new(),
                    after_event_id: i64::MAX as u64 + 1,
                },
            )
            .await
            .expect_err("cursor is rejected");
        let decoded = Response::decode(cursor.bytes.as_slice()).expect("decode response");
        assert!(matches!(decoded.body, Some(response::Body::Error(error)) if error.code == 1));
        actor.shutdown().await.expect("shutdown");
    }
}
