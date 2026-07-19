use std::time::{SystemTime, UNIX_EPOCH};

use clipmill_contracts::proto::ipc::v1::{
    CreateProjectRequest, Error, ErrorCode, GetProjectResponse, HealthResponse,
    ListProjectsResponse, PingResponse, Request, Response, request, response,
};
use clipmill_core::ProjectId;
use prost::Message;
use sha2::{Digest, Sha256};

use crate::db::{DbHandle, ProjectRecord, StoreError};

const REQUEST_ID_MAX_CHARS: usize = 128;
const PROJECT_NAME_MAX_CHARS: usize = 200;

#[derive(Clone, Debug)]
pub(crate) struct Service {
    database: DbHandle,
    started_unix_millis: u64,
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

impl Service {
    pub(crate) fn new(database: DbHandle, started_unix_millis: u64) -> Self {
        Self {
            database,
            started_unix_millis,
        }
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
            request::Body::SubmitJob(_)
            | request::Body::SubscribeTaskEvents(_)
            | request::Body::GetDeviceProfile(_) => error_reply(
                request_id,
                ErrorCode::Unavailable,
                "operation is not available in W2",
            ),
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
        CreateProjectRequest, GetDeviceProfileRequest, Request, Response, request, response,
    };
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
}
