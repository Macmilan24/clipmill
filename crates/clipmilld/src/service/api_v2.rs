//! The engine API v2 (docs/engine-api.md): the one surface agents, the
//! `clipmill` command and the local gateway share (R72, R81).
//!
//! A call is JSON, `{"op": "...", "args": {...}}`, carried in the v1 envelope.
//! Every operation is a translation onto the v1 handlers the Studio already
//! uses, so nothing here decides anything the Studio would decide differently;
//! what this layer adds is who is calling, what they may do, and a record of
//! what they did. Answers are JSON too, with times in seconds for people and
//! agents, and an error that says what to do about it.

use clipmill_contracts::proto::ipc::v1::{
    AgentClientCreated, AgentClientV1, AgentClientsResponse, AnalyzeSourcePayloadV1,
    CancelJobRequest, ClipDecisionV1, ClipDurationV1, CreateAgentClientRequest,
    CreateProjectRequest, DirectClipRequest, EngineAuditEntryV1, EngineAuditResponse,
    EngineCallRequest, EngineCallResponse, ErrorCode, ExportClipRequest, ExportRequestV1,
    GetJobRequest, HealthRequest, Job, JobState, ListClipDecisionsRequest, ListJobsRequest,
    ListModelsRequest, ListProjectsRequest, ListSourcesRequest, ReadArtifactRequest,
    RegisterSourceRequest, Request, Response, SubmitJobRequest, TaskState, request, response,
};
use prost::Message;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

use super::{Reply, Service, error_reply, response_reply, unix_millis};
use crate::db::{AgentClient, AgentCommand, AgentReply, AuditEntry};

const TICKS: f64 = 90_000.0;
/// What a process of this account gets without a token: everything but
/// publishing and deleting, which need a client the person created for it.
const LOCAL_SCOPES: [&str; 3] = ["read", "edit", "render"];
pub(super) const SCOPES: [&str; 5] = ["read", "edit", "render", "publish", "delete"];
const AUDIT_DEFAULT: u32 = 200;
const TOKEN_PREFIX: &str = "cmt_";
const READ_CHUNK: u64 = 4 * 1024 * 1024;

/// Every operation and the scope it needs.
pub(crate) const OPERATIONS: [(&str, &str); 36] = [
    ("whoami", "authenticated"),
    ("mcp_export_consent", "render"),
    ("read_edit_resource", "read"),
    ("audit_log", "read"),
    ("status", "read"),
    ("list_projects", "read"),
    ("create_project", "edit"),
    ("add_source", "edit"),
    ("list_sources", "read"),
    ("get_source", "read"),
    ("get_timeline", "read"),
    ("job_events", "read"),
    ("get_edit", "read"),
    ("apply_edits", "edit"),
    ("undo_edit", "edit"),
    ("edit_log", "read"),
    ("render_preview", "render"),
    ("start_analysis", "edit"),
    ("get_job", "read"),
    ("get_owned_job", "read"),
    ("cancel_owned_job", "edit"),
    ("list_jobs", "read"),
    ("cancel_job", "edit"),
    ("get_transcript", "read"),
    ("list_clips", "read"),
    ("describe_clip", "read"),
    ("find_moments", "read"),
    ("propose_clips", "edit"),
    ("reprompt", "edit"),
    ("create_clip_from_plan", "edit"),
    ("memory_overview", "read"),
    ("judge_questions", "read"),
    ("set_decision", "edit"),
    ("create_clip", "edit"),
    ("export_clip", "render"),
    ("list_models", "read"),
];

#[derive(Deserialize)]
struct Call {
    op: String,
    #[serde(default)]
    args: Value,
    /// Given, a retried call is answered once: the same id and the same call
    /// return the first answer instead of doing the work twice.
    #[serde(default)]
    request_id: Option<String>,
}

/// A refusal or failure, as the API states it.
pub(super) struct ApiError {
    code: &'static str,
    message: String,
    retryable: bool,
}

impl ApiError {
    pub(super) fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            retryable: code == "busy" || code == "unavailable",
        }
    }

    pub(super) fn invalid(message: impl Into<String>) -> Self {
        Self::new("invalid_argument", message)
    }

    pub(super) fn not_found(message: impl Into<String>) -> Self {
        Self::new("not_found", message)
    }
}

pub(super) type ApiResult = Result<Value, ApiError>;

/// Who is calling, and what they may do.
struct Caller {
    client_id: String,
    scopes: Vec<String>,
}

fn request_key(client: &str, operation: &str, key: &str) -> String {
    let mut digest = Sha256::new();
    for value in [client, operation, key] {
        digest.update(value.len().to_le_bytes());
        digest.update(value.as_bytes());
    }
    format!("v2_{}", hex::encode(digest.finalize()))
}

fn hash_token(token: &str) -> [u8; 32] {
    Sha256::digest(token.as_bytes()).into()
}

pub(super) fn seconds(ticks: u64) -> f64 {
    // Rounded to the millisecond: an agent reading "12.3456789" learns nothing
    // the tick count did not already say better.
    #[allow(clippy::cast_precision_loss)]
    let value = ticks as f64 / TICKS;
    (value * 1000.0).round() / 1000.0
}

fn ticks(seconds: f64) -> Result<u64, ApiError> {
    if !seconds.is_finite() || !(0.0..=86_400.0 * 7.0).contains(&seconds) {
        return Err(ApiError::invalid("a time must be a number of seconds"));
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    Ok((seconds * TICKS).round() as u64)
}

pub(super) fn arg_str<'a>(args: &'a Value, name: &str) -> Result<&'a str, ApiError> {
    args.get(name)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| ApiError::invalid(format!("`{name}` is required")))
}

pub(super) fn opt_str<'a>(args: &'a Value, name: &str) -> Option<&'a str> {
    args.get(name)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
}

pub(super) fn requested_duration(args: &Value) -> Result<Option<ClipDurationV1>, ApiError> {
    if args.get("min_seconds").is_none() && args.get("max_seconds").is_none() {
        return Ok(None);
    }
    let min = args["min_seconds"].as_f64();
    let max = args["max_seconds"].as_f64();
    let (Some(min), Some(max)) = (min, max) else {
        return Err(ApiError::invalid(
            "give both min_seconds and max_seconds as numbers",
        ));
    };
    let range = ClipDurationV1 {
        min_ticks: ticks(min)?,
        max_ticks: ticks(max)?,
    };
    if range.min_ticks >= range.max_ticks {
        return Err(ApiError::invalid(
            "min_seconds must be shorter than max_seconds",
        ));
    }
    Ok(Some(range))
}

fn job_state(state: i32) -> &'static str {
    match JobState::try_from(state).unwrap_or(JobState::Unspecified) {
        JobState::Planned => "queued",
        JobState::Running => "running",
        JobState::Succeeded => "succeeded",
        JobState::Failed => "failed",
        JobState::Cancelled => "cancelled",
        JobState::CancelRequested => "cancelling",
        JobState::Unspecified => "unknown",
    }
}

pub(super) fn job_json(job: &Job) -> Value {
    let done = job
        .tasks
        .iter()
        .filter(|task| task.state == TaskState::Succeeded as i32)
        .count();
    let stages: Vec<Value> = job
        .tasks
        .iter()
        .map(|task| {
            json!({
                "task_id": task.task_id,
                "kind": task.kind,
                "stage": task.output_kind,
                "artifact_id": (!task.output_artifact_id.is_empty()).then_some(task.output_artifact_id.as_str()),
                "state": TaskState::try_from(task.state)
                    .map(|state| state.as_str_name().trim_start_matches("TASK_STATE_").to_lowercase())
                    .unwrap_or_default(),
            })
        })
        .collect();
    let mut value = json!({
        "job_id": job.job_id,
        "project_id": job.project_id,
        "kind": job.kind,
        "created_unix_millis": job.created_unix_millis,
        "updated_unix_millis": job.updated_unix_millis,
        "state": job_state(job.state),
        "progress": { "done": done, "total": job.tasks.len() },
        "stages": stages,
        "artifacts":job.output_artifact_ids,
    });
    if !job.source_id.is_empty() {
        value["source_id"] = json!(job.source_id);
    }
    if !job.failure_detail.is_empty() {
        value["failure"] = json!(job.failure_detail);
    }
    value
}

fn client_v1(client: AgentClient) -> AgentClientV1 {
    AgentClientV1 {
        client_id: client.client_id,
        name: client.name,
        scopes: client.scopes,
        created_unix_millis: client.created_unix_millis,
        revoked_unix_millis: client.revoked_unix_millis,
        last_used_unix_millis: client.last_used_unix_millis,
    }
}

/// The candidate id the daemon gives a planned clip, so a decision on a plan
/// lands where the Studio's would.
pub(super) fn plan_candidate_id(source_id: &str, job_id: &str, plan_json: &str) -> String {
    let identity = format!("plan:{source_id}:{job_id}:{plan_json}");
    format!(
        "cand_{}",
        &hex::encode(Sha256::digest(identity.as_bytes()))[..16]
    )
}

/// What an agent is told about answering a run's questions.
const JUDGE_ANSWER_HELP: &str = "Read each question as written and answer it from the text alone, \
with yes or no, or with the probability that the answer is yes. Send the answers with reprompt \
as answers: [{question_id, answer}]. Some questions have answers the recording itself shows; \
ClipMill measures your answers to those before it uses any of them, and a kind of question is \
put to you only when you have answered every one of its kind. Answers are kept with the run and \
carried into the next one, so each round needs only the questions still open.";

/// The most answers one run may carry.
const MAX_JUDGE_ANSWERS: usize = 2_000;

/// An agent's answers, merged over the ones the earlier run carried: a new
/// answer to the same question replaces the old one.
pub(super) fn judge_answers(
    args: &Value,
    earlier: &[clipmill_contracts::proto::ipc::v1::JudgeAnswerV1],
) -> Result<Vec<clipmill_contracts::proto::ipc::v1::JudgeAnswerV1>, ApiError> {
    let mut merged: std::collections::BTreeMap<String, u32> = earlier
        .iter()
        .map(|answer| (answer.question_id.clone(), answer.yes_per_mille))
        .collect();
    for item in args["answers"].as_array().into_iter().flatten() {
        let id = item["question_id"].as_str().unwrap_or_default();
        let well_formed = id.len() == 26
            && id.starts_with("q_")
            && id[2..]
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
        if !well_formed {
            return Err(ApiError::invalid(
                "each answer needs the question_id a run listed",
            ));
        }
        let probability = match &item["answer"] {
            Value::String(word) if word.eq_ignore_ascii_case("yes") => 1.0,
            Value::String(word) if word.eq_ignore_ascii_case("no") => 0.0,
            Value::Number(number) => number.as_f64().unwrap_or(f64::NAN),
            _ => f64::NAN,
        };
        if !(0.0..=1.0).contains(&probability) {
            return Err(ApiError::invalid(
                "an answer is yes, no, or a probability of yes from 0 to 1",
            ));
        }
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a probability from 0 to 1, rounded to thousandths"
        )]
        merged.insert(id.to_owned(), (probability * 1000.0).round() as u32);
    }
    if merged.len() > MAX_JUDGE_ANSWERS {
        return Err(ApiError::invalid(format!(
            "a run carries at most {MAX_JUDGE_ANSWERS} answers"
        )));
    }
    Ok(merged
        .into_iter()
        .map(
            |(question_id, yes_per_mille)| clipmill_contracts::proto::ipc::v1::JudgeAnswerV1 {
                question_id,
                yes_per_mille,
            },
        )
        .collect())
}

/// How a decision names its clip in the project's taste: the clip's form and
/// title, within the stored subject's 200 characters.
fn clip_subject(clip: &Value) -> String {
    format!(
        "{} “{}”",
        clip["form"].as_str().unwrap_or("moment"),
        clip["title"].as_str().unwrap_or_default()
    )
    .chars()
    .take(200)
    .collect()
}

pub(super) fn canonical_json(value: &Value) -> String {
    let mut sorted = value.clone();
    sorted.sort_all_objects();
    sorted.to_string()
}

/// The analysis an answer is read from: the run named, or the newest
/// finished analysis of the source.
pub(super) struct Run {
    pub(super) job: Job,
    pub(super) project_id: String,
    pub(super) source_id: String,
    inherited: BTreeMap<String, String>,
}

impl Run {
    pub(super) fn artifact(&self, kind: &str) -> Option<&str> {
        self.job
            .tasks
            .iter()
            .find(|task| task.output_kind == kind && !task.output_artifact_id.is_empty())
            .map(|task| task.output_artifact_id.as_str())
            .or_else(|| self.inherited.get(kind).map(String::as_str))
    }
}

impl Service {
    /// One engine API call, answered with JSON whatever happened.
    pub(super) async fn engine_call(&self, request_id: String, call: EngineCallRequest) -> Reply {
        let answer = self.engine_answer(&call).await;
        response_reply(
            request_id,
            response::Body::EngineCall(EngineCallResponse {
                result_json: answer.to_string(),
            }),
        )
    }

    async fn engine_answer(&self, call: &EngineCallRequest) -> Value {
        let parsed: Call = match serde_json::from_str(&call.call_json) {
            Ok(parsed) => parsed,
            Err(error) => {
                return refusal(&ApiError::invalid(format!(
                    "the call is not {{\"op\", \"args\"}} JSON: {error}"
                )));
            }
        };
        let caller = match self.caller(&call.token).await {
            Ok(caller) => caller,
            Err(error) => return refusal(&error),
        };
        let target = target_of(&parsed.args);
        let outcome = match OPERATIONS.iter().find(|(op, _)| *op == parsed.op) {
            None => Err(ApiError::invalid(format!(
                "there is no operation `{}`; the operations are {}",
                parsed.op,
                OPERATIONS
                    .iter()
                    .map(|(op, _)| *op)
                    .collect::<Vec<_>>()
                    .join(", ")
            ))),
            Some(_)
                if parsed.request_id.as_ref().is_some_and(|key| {
                    key.is_empty() || key.len() > 255 || key.chars().any(char::is_control)
                }) =>
            {
                Err(ApiError::invalid(
                    "request_id must contain 1 to 255 bytes and no control characters",
                ))
            }
            Some((_, scope))
                if *scope != "authenticated"
                    && !caller.scopes.iter().any(|held| held == scope)
                    && !(parsed.op == "cancel_owned_job"
                        && caller.scopes.iter().any(|s| s == "render")) =>
            {
                Err(ApiError::new(
                    "out_of_scope",
                    format!(
                        "`{}` needs the {scope} scope, which this client was not given. Give it in ClipMill's Settings, Connected agents.",
                        parsed.op
                    ),
                ))
            }
            Some(_) => {
                let mut service = self.clone();
                service.engine_actor = Some(caller.client_id.clone());
                service
                    .dispatch(
                        &caller,
                        &parsed.op,
                        &parsed.args,
                        parsed
                            .request_id
                            .as_deref()
                            .map(|key| request_key(&caller.client_id, &parsed.op, key)),
                    )
                    .await
            }
        };
        let (status, detail) = match &outcome {
            Ok(_) => ("ok", String::new()),
            Err(error) if matches!(error.code, "out_of_scope" | "invalid_argument") => {
                ("refused", error.message.clone())
            }
            Err(error) => ("failed", error.message.clone()),
        };
        let entry = AuditEntry {
            at_unix_millis: unix_millis().unwrap_or(0),
            client_id: caller.client_id,
            client_name: String::new(),
            operation: parsed.op.chars().take(64).collect(),
            target,
            outcome: status,
            detail: detail.chars().take(400).collect(),
        };
        if let Err(error) = self.database.agents(AgentCommand::Record(entry)).await {
            tracing::warn!(%error, "an engine API call could not be written to the audit log");
        }
        match outcome {
            Ok(result) => json!({ "ok": true, "result": result }),
            Err(error) => refusal(&error),
        }
    }

    async fn caller(&self, token: &str) -> Result<Caller, ApiError> {
        if token.is_empty() {
            return Ok(Caller {
                client_id: "local".to_owned(),
                scopes: LOCAL_SCOPES
                    .iter()
                    .map(|scope| (*scope).to_owned())
                    .collect(),
            });
        }
        let reply = self
            .database
            .agents(AgentCommand::Resolve {
                token_hash: hash_token(token),
                at_unix_millis: unix_millis().unwrap_or(0),
            })
            .await
            .map_err(|error| ApiError::new("unavailable", error.to_string()))?;
        match reply {
            AgentReply::Client(Some(client)) => Ok(Caller {
                client_id: client.client_id,
                scopes: client.scopes,
            }),
            _ => Err(ApiError::new(
                "unauthenticated",
                "this token is not one ClipMill knows, or it was revoked. Create a new one in Settings, Connected agents.",
            )),
        }
    }

    /// Send one v1 request through the Studio's own handler.
    pub(super) async fn v1(
        &self,
        body: request::Body,
        request_id: Option<String>,
    ) -> Result<response::Body, ApiError> {
        let request = Request {
            request_id: request_id
                .unwrap_or_else(|| format!("v2_{}", ulid::Ulid::new().to_string().to_lowercase())),
            body: Some(body),
        };
        let reply = Box::pin(self.handle(request)).await;
        let response = Response::decode(reply.bytes.as_slice())
            .map_err(|error| ApiError::new("internal", error.to_string()))?;
        match response.body {
            Some(response::Body::Error(error)) => Err(ApiError::new(
                match ErrorCode::try_from(error.code).unwrap_or(ErrorCode::Unspecified) {
                    ErrorCode::InvalidArgument => "invalid_argument",
                    ErrorCode::NotFound => "not_found",
                    ErrorCode::Conflict => "conflict",
                    ErrorCode::Unavailable => "unavailable",
                    ErrorCode::PolicyDenied => "policy_denied",
                    ErrorCode::Unspecified | ErrorCode::Internal => "internal",
                },
                error.message,
            )),
            Some(body) => Ok(body),
            None => Err(ApiError::new("internal", "the engine answered nothing")),
        }
    }

    #[allow(
        clippy::too_many_lines,
        reason = "one arm per operation, each a translation onto a v1 request"
    )]
    async fn dispatch(
        &self,
        caller: &Caller,
        op: &str,
        args: &Value,
        id: Option<String>,
    ) -> ApiResult {
        match op {
            "whoami" => Ok(json!({"client_id": caller.client_id, "scopes": caller.scopes})),
            "mcp_export_consent" => super::mcp_consent::export_consent(&caller.client_id, args),
            "status" => {
                let health = self
                    .v1(request::Body::Health(HealthRequest {}), None)
                    .await?;
                let response::Body::Health(health) = health else {
                    return Err(ApiError::new("internal", "health answered something else"));
                };
                Ok(json!({
                    "engine": "clipmill",
                    "version": health.daemon_version,
                    "local_lock": health.local_lock,
                    "operations": OPERATIONS.iter().map(|(op, scope)| json!({"op": op, "scope": scope})).collect::<Vec<_>>(),
                }))
            }
            "list_projects" => {
                let body = self
                    .v1(request::Body::ListProjects(ListProjectsRequest {}), None)
                    .await?;
                let response::Body::ListProjects(list) = body else {
                    return Err(ApiError::new("internal", "unexpected answer"));
                };
                Ok(json!({ "projects": list.projects.iter().map(|p| json!({
                    "project_id": p.project_id, "name": p.name, "created_unix_millis": p.created_unix_millis,
                })).collect::<Vec<_>>() }))
            }
            "create_project" => {
                let name = arg_str(args, "name")?;
                let body = self
                    .v1(
                        request::Body::CreateProject(CreateProjectRequest {
                            name: name.to_owned(),
                        }),
                        id,
                    )
                    .await?;
                let response::Body::CreateProject(created) = body else {
                    return Err(ApiError::new("internal", "unexpected answer"));
                };
                let project = created.project.unwrap_or_default();
                Ok(json!({ "project_id": project.project_id, "name": project.name }))
            }
            "add_source" => {
                let project_id = arg_str(args, "project_id")?;
                let path = arg_str(args, "path")?;
                if !std::path::Path::new(path).is_absolute() {
                    return Err(ApiError::invalid(
                        "`path` must be an absolute path to a video or audio file on this computer",
                    ));
                }
                let body = self
                    .v1(
                        request::Body::RegisterSource(RegisterSourceRequest {
                            project_id: project_id.to_owned(),
                            absolute_path: path.to_owned(),
                            ..RegisterSourceRequest::default()
                        }),
                        id,
                    )
                    .await?;
                let response::Body::RegisterSource(registered) = body else {
                    return Err(ApiError::new("internal", "unexpected answer"));
                };
                let source = registered.source.unwrap_or_default();
                Ok(
                    json!({ "source_id": source.source_id, "project_id": source.project_id, "path": source.absolute_path, "bytes": source.byte_size }),
                )
            }
            "list_sources" => {
                let project_id = arg_str(args, "project_id")?;
                let body = self
                    .v1(
                        request::Body::ListSources(ListSourcesRequest {
                            project_id: project_id.to_owned(),
                        }),
                        None,
                    )
                    .await?;
                let response::Body::ListSources(list) = body else {
                    return Err(ApiError::new("internal", "unexpected answer"));
                };
                Ok(json!({ "sources": list.sources.iter().map(|s| json!({
                    "source_id": s.source_id, "path": s.absolute_path, "bytes": s.byte_size, "missing": s.missing,
                })).collect::<Vec<_>>() }))
            }
            "audit_log" => {
                let limit = args
                    .get("limit")
                    .map_or(Some(100), Value::as_u64)
                    .filter(|n| (1..=2000).contains(n))
                    .ok_or_else(|| ApiError::invalid("limit must be from 1 to 2000"))?;
                let client_id = (caller.client_id != "local").then(|| caller.client_id.clone());
                let log = self
                    .database
                    .agents(AgentCommand::Log {
                        limit: u32::try_from(limit).unwrap_or(2000),
                        client_id,
                    })
                    .await
                    .map_err(|e| ApiError::new("unavailable", e.to_string()))?;
                let AgentReply::Log(log) = log else {
                    return Err(ApiError::new("internal", "unexpected audit result"));
                };
                let entries:Vec<_>=log.into_iter().map(|e|json!({"at_unix_millis":e.at_unix_millis,"client":e.client_name,"op":e.operation,"target":e.target,"outcome":e.outcome,"detail":e.detail})).collect();
                Ok(json!({"entries":entries}))
            }
            "get_source" => self.get_api_source(args).await,
            "get_timeline" => self.timeline(args).await,
            "job_events" => self.job_events(args).await,
            "get_edit" | "read_edit_resource" => self.get_api_edit(args).await,
            "apply_edits" | "undo_edit" => self.change_edit(args, id, op == "undo_edit").await,
            "edit_log" => self.api_edit_log(args).await,
            "render_preview" => self.preview_render(args, id).await,
            "start_analysis" => self.start_analysis(args, id).await,
            "get_owned_job" | "cancel_owned_job" => {
                let job_id = arg_str(args, "job_id")?;
                let owned = self
                    .database
                    .agents(AgentCommand::OwnedJob {
                        job_id: job_id.to_owned(),
                        client_id: caller.client_id.clone(),
                    })
                    .await
                    .map_err(|e| ApiError::new("unavailable", e.to_string()))?;
                let AgentReply::OwnedJob(Some(kind)) = owned else {
                    return Err(ApiError::not_found("unknown task"));
                };
                if op == "cancel_owned_job" {
                    let scope = if matches!(kind.as_str(), "export-clip" | "render-clip") {
                        "render"
                    } else {
                        "edit"
                    };
                    if !caller.scopes.iter().any(|held| held == scope) {
                        return Err(ApiError::new(
                            "out_of_scope",
                            format!("cancelling this task needs {scope}"),
                        ));
                    }
                    self.v1(
                        request::Body::CancelJob(CancelJobRequest {
                            job_id: job_id.to_owned(),
                        }),
                        id,
                    )
                    .await?;
                }
                Ok(job_json(&self.job(job_id).await?))
            }
            "get_job" => {
                let job = self.job(arg_str(args, "job_id")?).await?;
                Ok(job_json(&job))
            }
            "list_jobs" => {
                let body = self
                    .v1(
                        request::Body::ListJobs(ListJobsRequest {
                            project_id: arg_str(args, "project_id")?.to_owned(),
                        }),
                        None,
                    )
                    .await?;
                let response::Body::ListJobs(list) = body else {
                    return Err(ApiError::new("internal", "unexpected answer"));
                };
                Ok(json!({ "jobs": list.jobs.iter().map(|job| json!({
                    "job_id": job.job_id, "kind": job.kind, "state": job_state(job.state),
                    "source_id": job.source_id, "updated_unix_millis": job.updated_unix_millis,
                })).collect::<Vec<_>>() }))
            }
            "cancel_job" => {
                let body = self
                    .v1(
                        request::Body::CancelJob(CancelJobRequest {
                            job_id: arg_str(args, "job_id")?.to_owned(),
                        }),
                        id,
                    )
                    .await?;
                let response::Body::CancelJob(cancelled) = body else {
                    return Err(ApiError::new("internal", "unexpected answer"));
                };
                Ok(cancelled.job.as_ref().map_or(Value::Null, job_json))
            }
            "get_transcript" => self.transcript(args).await,
            "list_clips" => self.list_clips(args).await,
            "describe_clip" => self.describe_clip(args).await,
            "memory_overview" => {
                let run = self.run(args).await?;
                let memory = self
                    .optional_document(&run, "mind.memory.v1")
                    .await?
                    .ok_or_else(|| ApiError::not_found("this run has no recording memory"))?;
                Ok(
                    json!({"job_id":run.job.job_id,"chapters":memory["chapters"],"events":memory["events"],
                    "links":memory["links"], "entities":memory["entities"]}),
                )
            }
            "judge_questions" => self.judge_questions(args).await,
            "propose_clips" | "reprompt" => self.reprompt(args, id).await,
            "create_clip_from_plan" => self.create_from_plan(args, id).await,
            "find_moments" => self.find_moments(args).await,
            "set_decision" => self.set_decision(args, id).await,
            "create_clip" => self.create_clip(args, id).await,
            "export_clip" => self.export(args, id).await,
            "list_models" => {
                let body = self
                    .v1(request::Body::ListModels(ListModelsRequest {}), None)
                    .await?;
                let response::Body::ModelLibrary(library) = body else {
                    return Err(ApiError::new("internal", "unexpected answer"));
                };
                Ok(json!({ "models": library.models.iter().map(|m| json!({
                    "name": m.name, "title": m.title, "capability": m.capability,
                    "installed": m.install_state == "installed", "in_use": m.in_use,
                    "supported": m.supported, "download_bytes": m.download_bytes,
                })).collect::<Vec<_>>() }))
            }
            _ => Err(ApiError::invalid(format!("there is no operation `{op}`"))),
        }
    }

    pub(super) async fn job(&self, job_id: &str) -> Result<Job, ApiError> {
        let body = self
            .v1(
                request::Body::GetJob(GetJobRequest {
                    job_id: job_id.to_owned(),
                }),
                None,
            )
            .await?;
        let response::Body::GetJob(found) = body else {
            return Err(ApiError::new("internal", "unexpected answer"));
        };
        found
            .job
            .ok_or_else(|| ApiError::not_found(format!("there is no job {job_id}")))
    }

    async fn start_analysis(&self, args: &Value, id: Option<String>) -> ApiResult {
        let project_id = arg_str(args, "project_id")?;
        let source_id = arg_str(args, "source_id")?;
        let duration = requested_duration(args)?;
        // The editorial model and the mind run unless asked not to: they are
        // what make clips worth an agent's time, and they stay on this computer.
        let local_editorial = match args.get("editorial") {
            None => true,
            Some(value) if value == "local" => true,
            Some(value) if value == "none" => false,
            Some(_) => return Err(ApiError::invalid("editorial must be local or none")),
        };
        let payload = AnalyzeSourcePayloadV1 {
            key_version: "clipmill.analyze-source.v1".to_owned(),
            source_id: source_id.to_owned(),
            language: opt_str(args, "language").unwrap_or_default().to_owned(),
            duration,
            editorial_prompt: opt_str(args, "prompt")
                .unwrap_or_default()
                .trim()
                .to_owned(),
            count: super::briefs::requested_count(args)?.unwrap_or_default(),
            local_editorial,
            ..AnalyzeSourcePayloadV1::default()
        };
        let body = self
            .v1(
                request::Body::SubmitJob(SubmitJobRequest {
                    project_id: project_id.to_owned(),
                    kind: "analyze-source".to_owned(),
                    payload: payload.encode_to_vec(),
                }),
                id,
            )
            .await?;
        let response::Body::SubmitJob(submitted) = body else {
            return Err(ApiError::new("internal", "unexpected answer"));
        };
        Ok(submitted.job.as_ref().map_or(Value::Null, job_json))
    }

    /// The run named in `args`, or the newest finished analysis of the source.
    pub(super) async fn run(&self, args: &Value) -> Result<Run, ApiError> {
        let project_id = arg_str(args, "project_id")?.to_owned();
        let source_id = arg_str(args, "source_id")?.to_owned();
        let job = if let Some(job_id) = opt_str(args, "job_id") {
            self.job(job_id).await?
        } else {
            let body = self
                .v1(
                    request::Body::ListJobs(ListJobsRequest {
                        project_id: project_id.clone(),
                    }),
                    None,
                )
                .await?;
            let response::Body::ListJobs(list) = body else {
                return Err(ApiError::new("internal", "unexpected answer"));
            };
            let newest = list
                .jobs
                .into_iter()
                .filter(|job| {
                    matches!(job.kind.as_str(), "analyze-source" | "reprompt")
                        && job.source_id == source_id
                        && job.state == JobState::Succeeded as i32
                })
                .max_by_key(|job| job.updated_unix_millis)
                .ok_or_else(|| {
                    ApiError::not_found(
                        "this source has no finished analysis; start one with start_analysis",
                    )
                })?;
            self.job(&newest.job_id).await?
        };
        if job.project_id != project_id
            || job.source_id != source_id
            || !matches!(job.kind.as_str(), "analyze-source" | "reprompt")
        {
            return Err(ApiError::not_found(
                "this run does not belong to the requested recording",
            ));
        }
        if job.state != JobState::Succeeded as i32 {
            return Err(ApiError::new("conflict", "this analysis has not finished"));
        }
        let mut run = Run {
            job,
            project_id,
            source_id,
            inherited: BTreeMap::new(),
        };
        if run.job.kind == "reprompt" {
            run.inherited = self.run_manifest(&run).await?.0;
        }
        Ok(run)
    }

    pub(super) async fn document(&self, project_id: &str, artifact_id: &str) -> ApiResult {
        let mut collected = Vec::new();
        loop {
            let body = self
                .v1(
                    request::Body::ReadArtifact(ReadArtifactRequest {
                        project_id: project_id.to_owned(),
                        artifact_id: artifact_id.to_owned(),
                        offset: collected.len() as u64,
                        length: READ_CHUNK,
                    }),
                    None,
                )
                .await?;
            let response::Body::ReadArtifact(read) = body else {
                return Err(ApiError::new("internal", "unexpected answer"));
            };
            collected.extend_from_slice(&read.chunk);
            if read.chunk.is_empty() || collected.len() as u64 >= read.total_bytes {
                break;
            }
        }
        serde_json::from_slice(&collected)
            .map_err(|error| ApiError::new("internal", format!("a document did not read: {error}")))
    }

    pub(super) async fn optional_document(
        &self,
        run: &Run,
        kind: &str,
    ) -> Result<Option<Value>, ApiError> {
        match run.artifact(kind) {
            None => Ok(None),
            Some(id) => self.document(&run.project_id, id).await.map(Some),
        }
    }

    async fn transcript(&self, args: &Value) -> ApiResult {
        let run = self.run(args).await?;
        let index = self
            .optional_document(&run, "index.transcript.v1")
            .await?
            .ok_or_else(|| ApiError::not_found("this analysis has no transcript"))?;
        let script = self.optional_document(&run, "mind.script.v1").await?;
        let voices: std::collections::BTreeMap<u64, String> = script
            .as_ref()
            .and_then(|s| s["units"].as_array())
            .into_iter()
            .flatten()
            .filter_map(|unit| {
                Some((
                    unit["index"].as_u64()?,
                    unit["speaker"].as_str()?.to_owned(),
                ))
            })
            .collect();
        let from = args
            .get("start_seconds")
            .and_then(Value::as_f64)
            .map(ticks)
            .transpose()?;
        let to = args
            .get("end_seconds")
            .and_then(Value::as_f64)
            .map(ticks)
            .transpose()?;
        let sentences: Vec<Value> = index["sentences"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|s| {
                let (start, end) = (
                    s["start_ticks"].as_u64().unwrap_or(0),
                    s["end_ticks"].as_u64().unwrap_or(0),
                );
                from.is_none_or(|from| end > from) && to.is_none_or(|to| start < to)
            })
            .map(|s| {
                let index = s["index"].as_u64().unwrap_or(0);
                let mut sentence = json!({
                    "unit": format!("s{index}"),
                    "start_ticks": s["start_ticks"],
                    "end_ticks": s["end_ticks"],
                    "start": seconds(s["start_ticks"].as_u64().unwrap_or(0)),
                    "end": seconds(s["end_ticks"].as_u64().unwrap_or(0)),
                    "text": s["text"],
                });
                if let Some(voice) = voices.get(&index) {
                    sentence["speaker"] = json!(voice);
                }
                sentence
            })
            .collect();
        Ok(json!({
            "job_id": run.job.job_id,
            "source_fingerprint": index["source_fingerprint"],
            "language": index["language"],
            "sentences": sentences
        }))
    }

    async fn decisions(
        &self,
        run: &Run,
    ) -> Result<BTreeMap<String, (&'static str, String)>, ApiError> {
        let body = self
            .v1(
                request::Body::ListClipDecisions(ListClipDecisionsRequest {
                    project_id: run.project_id.clone(),
                    source_id: run.source_id.clone(),
                }),
                None,
            )
            .await?;
        let response::Body::ListClipDecisions(list) = body else {
            return Err(ApiError::new("internal", "unexpected answer"));
        };
        Ok(list
            .decisions
            .into_iter()
            .map(|record| {
                let decision = match ClipDecisionV1::try_from(record.decision) {
                    Ok(ClipDecisionV1::Rejected) => "rejected",
                    Ok(ClipDecisionV1::Kept) => "kept",
                    Ok(ClipDecisionV1::Approved) => "approved",
                    _ => "undecided",
                };
                (record.candidate_id, (decision, record.reason))
            })
            .collect())
    }

    /// What a decision made in the Studio is about, worded as an engine
    /// review words it, so the project's taste can learn from a clip nobody
    /// edited and gave no reason for. `None` when the recording's newest
    /// analysis no longer lists the clip; the decision is kept either way.
    pub(super) async fn decision_subject(
        &self,
        project_id: &str,
        source_id: &str,
        candidate_id: &str,
    ) -> Option<String> {
        let run = self
            .run(&json!({"project_id": project_id, "source_id": source_id}))
            .await
            .ok()?;
        let clips = self.clips(&run).await.ok()?;
        clips
            .iter()
            .find(|clip| clip["candidate_id"] == candidate_id || clip["clip_id"] == candidate_id)
            .map(clip_subject)
    }

    /// The narrow questions an analysis put to its judge, or would have,
    /// that no connected agent has answered yet. An agent answers them with
    /// `reprompt`; known cases are mixed in and never marked.
    async fn judge_questions(&self, args: &Value) -> ApiResult {
        let run = self.run(args).await?;
        let mut seen = std::collections::BTreeSet::new();
        let mut questions = Vec::new();
        for kind in ["mind.judgments.v1", "mind.quality_report.v1"] {
            let Some(document) = self.optional_document(&run, kind).await? else {
                continue;
            };
            for question in document["questions"].as_array().into_iter().flatten() {
                if let Some(id) = question["question_id"].as_str()
                    && seen.insert(id.to_owned())
                {
                    questions.push(json!({
                        "question_id": id,
                        "kind": question["kind"],
                        "text": question["text"],
                    }));
                }
            }
        }
        let answered = run
            .job
            .analysis
            .as_ref()
            .map_or(0, |settings| settings.judge_answers.len());
        Ok(json!({
            "job_id": run.job.job_id,
            "questions": questions,
            "answered": answered,
            "how_to_answer": JUDGE_ANSWER_HELP,
        }))
    }

    /// The mind's plans and the ranked moments of one analysis, best first.
    async fn clips(&self, run: &Run) -> Result<Vec<Value>, ApiError> {
        let decisions = self.decisions(run).await?;
        let mut clips = Vec::new();
        let (plans, final_ranking) = self.plan_rows(run).await?;
        for candidate in plans {
            let plan = &candidate["plan"];
            let plan_json = canonical_json(plan);
            let decided = plan_candidate_id(&run.source_id, &run.job.job_id, &plan_json);
            let parts: Vec<Value> = plan["parts"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|part| {
                        json!({
                            "role": part["role"],
                            "start": seconds(part["start_ticks"].as_u64().unwrap_or(0)),
                            "end": seconds(part["end_ticks"].as_u64().unwrap_or(0)),
                            "removed": part["removed"].as_array().map_or(0, Vec::len),
                            "removed_ranges":part["removed"].as_array().into_iter().flatten().map(|span| json!({
                                "start":seconds(span["start_ticks"].as_u64().unwrap_or_default()),
                                "end":seconds(span["end_ticks"].as_u64().unwrap_or_default()),
                                "reason":span["reason"]})).collect::<Vec<_>>(),
                        })
                    })
                    .collect();
            clips.push(json!({
                    "clip_id": plan["plan_id"],
                    "kind": "plan", "candidate_id":decided,
                    "keep_score":candidate["keep_score"],"reasons":candidate["reasons"],"chapter":candidate["chapter"],
                    "quality":candidate["quality"],"joins":plan["joins"],"evidence":plan["evidence"],
                    "form": plan["form"],
                    "title": plan.get("title").or_else(|| plan.get("hook_text")).cloned().unwrap_or(Value::Null),
                    "duration": seconds(plan["duration_ticks"].as_u64().unwrap_or(0)),
                    "parts": parts,
                    "found_by": candidate["scout"],
                    "decision": decisions.get(&decided).map_or("undecided", |d| d.0),
                    "reason": decisions.get(&decided).map_or("", |d| d.1.as_str()),
                }));
        }
        if final_ranking {
            return Ok(clips);
        }
        if let Some(ranking) = self.optional_document(run, "ranking.set.v1").await? {
            let selected: std::collections::BTreeSet<&str> = ranking["selected"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|row| row["candidate_id"].as_str())
                .collect();
            for row in ranking["cohort"].as_array().into_iter().flatten() {
                let id = row["candidate_id"].as_str().unwrap_or_default();
                let chosen = &row["boundary"]["chosen"];
                clips.push(json!({
                    "clip_id": id,
                    "kind": "moment",
                    "title": row["title"],
                    "start": seconds(chosen["start_ticks"].as_u64().unwrap_or(0)),
                    "end": seconds(chosen["end_ticks"].as_u64().unwrap_or(0)),
                    "score": row["display_score"],
                    "recommended": selected.contains(id),
                    "decision": decisions.get(id).map_or("undecided", |d| d.0),
                    "reason": decisions.get(id).map_or("", |d| d.1.as_str()),
                }));
            }
        }
        Ok(clips)
    }

    async fn plan_rows(&self, run: &Run) -> Result<(Vec<Value>, bool), ApiError> {
        let (kind, rows, ranked) = if run.artifact("ranking.set.v2").is_some() {
            ("ranking.set.v2", "clips", true)
        } else {
            ("mind.candidates.v1", "candidates", false)
        };
        let Some(mut doc) = self.optional_document(run, kind).await? else {
            return Ok((Vec::new(), false));
        };
        let values = doc[rows]
            .as_array_mut()
            .ok_or_else(|| ApiError::new("internal", "invalid editorial results"))?;
        Ok((std::mem::take(values), ranked))
    }

    async fn list_clips(&self, args: &Value) -> ApiResult {
        let run = self.run(args).await?;
        let clips = self.clips(&run).await?;
        Ok(json!({ "job_id": run.job.job_id, "clips": clips }))
    }

    async fn describe_clip(&self, args: &Value) -> ApiResult {
        let run = self.run(args).await?;
        let clip_id = arg_str(args, "clip_id")?;
        let clips = self.clips(&run).await?;
        let mut clip = clips
            .into_iter()
            .find(|clip| clip["clip_id"] == clip_id)
            .ok_or_else(|| ApiError::not_found(format!("this analysis has no clip {clip_id}")))?;
        let index = self.optional_document(&run, "index.transcript.v1").await?;
        let sentences = index.as_ref().and_then(|i| i["sentences"].as_array());
        let text_between = |start: f64, end: f64, removed: &Value| -> String {
            sentences
                .into_iter()
                .flatten()
                .filter(|s| {
                    let (a, b) = (
                        seconds(s["start_ticks"].as_u64().unwrap_or(0)),
                        seconds(s["end_ticks"].as_u64().unwrap_or(0)),
                    );
                    b > start
                        && a < end
                        && !removed.as_array().into_iter().flatten().any(|cut| {
                            a >= cut["start"].as_f64().unwrap_or(f64::INFINITY)
                                && b <= cut["end"].as_f64().unwrap_or_default()
                        })
                })
                .filter_map(|s| s["text"].as_str())
                .collect::<Vec<_>>()
                .join(" ")
        };
        if let Some(parts) = clip["parts"].as_array_mut() {
            for part in parts {
                let text = text_between(
                    part["start"].as_f64().unwrap_or(0.0),
                    part["end"].as_f64().unwrap_or(0.0),
                    &part["removed_ranges"],
                );
                part["text"] = json!(text);
            }
        } else {
            let text = text_between(
                clip["start"].as_f64().unwrap_or(0.0),
                clip["end"].as_f64().unwrap_or(0.0),
                &Value::Null,
            );
            clip["text"] = json!(text);
        }
        Ok(clip)
    }

    /// The plan document behind a plan id, from the run's candidates.
    async fn plan(&self, run: &Run, clip_id: &str) -> Result<Option<String>, ApiError> {
        if !clip_id.starts_with("plan_") {
            return Ok(None);
        }
        let (plans, _) = self.plan_rows(run).await?;
        plans
            .iter()
            .map(|candidate| &candidate["plan"])
            .find(|plan| plan["plan_id"] == clip_id)
            .map(|plan| Some(canonical_json(plan)))
            .ok_or_else(|| ApiError::not_found(format!("this analysis has no plan {clip_id}")))
    }

    async fn set_decision(&self, args: &Value, id: Option<String>) -> ApiResult {
        let id = id.unwrap_or_else(|| format!("v2_{}", ulid::Ulid::new()));
        let hash = Sha256::digest(canonical_json(args).as_bytes()).into();
        let result = if let Some(saved) = self
            .database
            .replay_request(id.clone(), hash)
            .await
            .map_err(|e| super::briefs::store_error(&e))?
        {
            saved
        } else {
            let run = self.run(args).await?;
            let clip_id = arg_str(args, "clip_id")?;
            let decision = match arg_str(args, "decision")? {
                "undecided" => None,
                value => Some(crate::db::Decision::parse(value).ok_or_else(|| {
                    ApiError::invalid("decision must be approved, kept, rejected or undecided")
                })?),
            };
            let reason = opt_str(args, "reason").unwrap_or_default().trim();
            if reason.chars().count() > 240 {
                return Err(ApiError::invalid(
                    "A decision reason is limited to 240 characters",
                ));
            }
            let clip = self
                .clips(&run)
                .await?
                .into_iter()
                .find(|c| c["clip_id"] == clip_id)
                .ok_or_else(|| ApiError::not_found("unknown clip in this analysis"))?;
            let subject = clip_subject(&clip);
            let review = crate::db::Review {
                request_id: id,
                hash,
                project_id: run.project_id,
                source_id: run.source_id,
                candidate_id: clip["candidate_id"].as_str().unwrap_or(clip_id).to_owned(),
                clip_id: clip_id.to_owned(),
                decision,
                reason: reason.to_owned(),
                subject,
                now: unix_millis().map_err(|e| ApiError::new("internal", e))?,
            };
            self.database
                .save_review(review)
                .await
                .map_err(|e| super::briefs::store_error(&e))?
        };
        serde_json::from_slice(&result)
            .map_err(|_| ApiError::new("internal", "invalid saved review result"))
    }

    pub(super) async fn create_clip(&self, args: &Value, id: Option<String>) -> ApiResult {
        let run = self.run(args).await?;
        let clip_id = arg_str(args, "clip_id")?;
        let plan_json = self.plan(&run, clip_id).await?.unwrap_or_default();
        self.direct_api_plan(&run, args, clip_id, plan_json, id)
            .await
    }

    pub(super) async fn direct_api_plan(
        &self,
        run: &Run,
        args: &Value,
        clip_id: &str,
        plan_json: String,
        id: Option<String>,
    ) -> ApiResult {
        // A clip made here starts with the look the person chose in the
        // Studio for this project, and the installation's brand and frame.
        let defaults = self
            .database
            .clip_defaults(run.project_id.clone())
            .await
            .map_err(|error| super::briefs::store_error(&error))?;
        let shape = match opt_str(args, "shape") {
            Some(shape) => shape.to_owned(),
            None => defaults.shape,
        };
        let body = self
            .v1(
                request::Body::DirectClip(DirectClipRequest {
                    project_id: run.project_id.clone(),
                    source_id: run.source_id.clone(),
                    candidate_id: clip_id.to_owned(),
                    job_id: run.job.job_id.clone(),
                    shape,
                    style_ref: defaults.style_ref,
                    highlight_spoken_word: defaults.highlight_spoken_word,
                    caption_options_json: defaults.caption_options_json,
                    brand_json: defaults.brand_json,
                    plan_json,
                    ..DirectClipRequest::default()
                }),
                id,
            )
            .await?;
        let response::Body::DirectClip(directed) = body else {
            return Err(ApiError::new("internal", "unexpected answer"));
        };
        let doc = directed.doc.unwrap_or_default();
        Ok(json!({
            "doc_id": doc.doc_id,
            "revision": doc.revision,
            "reopened": directed.reopened,
            "start": seconds(directed.start_ticks),
            "end": seconds(directed.end_ticks),
        }))
    }

    async fn export(&self, args: &Value, id: Option<String>) -> ApiResult {
        let doc_id = arg_str(args, "doc_id")?;
        let destination = arg_str(args, "destination_dir")?;
        // A rendered file states its rights position, and there is no honest
        // default for one the person never stated: an agent must ask them.
        let attestation = arg_str(args, "rights").map_err(|_| {
            ApiError::invalid(
                "`rights` is required: ask the person whether they own or may use this recording, and pass what they said",
            )
        })?;
        let body = self
            .v1(
                request::Body::ExportClip(ExportClipRequest {
                    request: Some(ExportRequestV1 {
                        doc_id: doc_id.to_owned(),
                        destination_dir: destination.to_owned(),
                        source_attestation: attestation.chars().take(400).collect(),
                        title: opt_str(args, "title").unwrap_or_default().to_owned(),
                        ai_assistance: vec!["asr_captions".to_owned(), "reframe".to_owned()],
                        ..ExportRequestV1::default()
                    }),
                }),
                id,
            )
            .await?;
        let response::Body::ExportClip(started) = body else {
            return Err(ApiError::new("internal", "unexpected answer"));
        };
        Ok(json!({ "job_id": started.job_id, "destination_dir": started.destination_dir }))
    }

    /// A client for an agent the person is connecting, with a token shown once.
    pub(super) async fn create_agent_client(
        &self,
        request_id: String,
        create: CreateAgentClientRequest,
    ) -> Reply {
        let name = create.name.trim();
        if name.is_empty() || name.chars().count() > 80 || name.chars().any(char::is_control) {
            return error_reply(
                request_id,
                ErrorCode::InvalidArgument,
                "Name the agent in up to 80 characters.",
            );
        }
        let mut scopes: Vec<String> = Vec::new();
        for scope in &create.scopes {
            if !SCOPES.contains(&scope.as_str()) {
                return error_reply(
                    request_id,
                    ErrorCode::InvalidArgument,
                    format!(
                        "{scope} is not a scope; the scopes are {}",
                        SCOPES.join(", ")
                    ),
                );
            }
            if !scopes.contains(scope) {
                scopes.push(scope.clone());
            }
        }
        if scopes.is_empty() {
            scopes.push("read".to_owned());
        }
        let mut secret = [0_u8; 32];
        if getrandom::fill(&mut secret).is_err() {
            return error_reply(
                request_id,
                ErrorCode::Internal,
                "This computer could not make a random token.",
            );
        }
        let token = format!("{TOKEN_PREFIX}{}", hex::encode(secret));
        let client_id = format!("agt_{}", ulid::Ulid::new().to_string().to_lowercase());
        let created = self
            .database
            .agents(AgentCommand::Create {
                client_id: client_id.clone(),
                name: name.to_owned(),
                token_hash: hash_token(&token),
                scopes,
                created_unix_millis: unix_millis().unwrap_or(0),
            })
            .await;
        match created {
            Ok(AgentReply::Clients(clients)) => {
                let client = clients
                    .into_iter()
                    .find(|client| client.client_id == client_id)
                    .map(client_v1);
                response_reply(
                    request_id,
                    response::Body::AgentClientCreated(AgentClientCreated { client, token }),
                )
            }
            Ok(_) => error_reply(request_id, ErrorCode::Internal, "unexpected store answer"),
            Err(error) => error_reply(request_id, ErrorCode::Internal, error.to_string()),
        }
    }

    pub(super) async fn agent_clients(&self, request_id: String, command: AgentCommand) -> Reply {
        match self.database.agents(command).await {
            Ok(AgentReply::Clients(clients)) => response_reply(
                request_id,
                response::Body::AgentClients(AgentClientsResponse {
                    clients: clients.into_iter().map(client_v1).collect(),
                }),
            ),
            Ok(_) => error_reply(request_id, ErrorCode::Internal, "unexpected store answer"),
            Err(crate::db::StoreError::NotFound) => {
                error_reply(request_id, ErrorCode::NotFound, "No such connected agent.")
            }
            Err(error) => error_reply(request_id, ErrorCode::Internal, error.to_string()),
        }
    }

    pub(super) async fn engine_audit(&self, request_id: String, limit: u32) -> Reply {
        let limit = if limit == 0 {
            AUDIT_DEFAULT
        } else {
            limit.min(2_000)
        };
        match self
            .database
            .agents(AgentCommand::Log {
                limit,
                client_id: None,
            })
            .await
        {
            Ok(AgentReply::Log(entries)) => response_reply(
                request_id,
                response::Body::EngineAudit(EngineAuditResponse {
                    entries: entries
                        .into_iter()
                        .map(|entry| EngineAuditEntryV1 {
                            at_unix_millis: entry.at_unix_millis,
                            client_id: entry.client_id,
                            client_name: entry.client_name,
                            operation: entry.operation,
                            target: entry.target,
                            outcome: entry.outcome.to_owned(),
                            detail: entry.detail,
                        })
                        .collect(),
                }),
            ),
            Ok(_) => error_reply(request_id, ErrorCode::Internal, "unexpected store answer"),
            Err(error) => error_reply(request_id, ErrorCode::Internal, error.to_string()),
        }
    }
}

fn refusal(error: &ApiError) -> Value {
    json!({
        "ok": false,
        "error": { "code": error.code, "message": error.message, "retryable": error.retryable },
    })
}

/// What a call was about, for the audit log: the ids it named.
fn target_of(args: &Value) -> String {
    ["project_id", "source_id", "job_id", "clip_id", "doc_id"]
        .iter()
        .filter_map(|key| args.get(*key).and_then(Value::as_str))
        .map(|value| value.chars().take(64).collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn times_cross_as_seconds_and_back() {
        assert!((seconds(135_000) - 1.5).abs() < f64::EPSILON);
        assert_eq!(ticks(1.5).ok(), Some(135_000));
        assert!(ticks(-1.0).is_err());
    }

    #[test]
    fn a_supplied_duration_has_two_ordered_numeric_endpoints() {
        assert!(matches!(requested_duration(&json!({})), Ok(None)));
        for args in [
            json!({"min_seconds":"12","max_seconds":"15"}),
            json!({"min_seconds":5}),
            json!({"min_seconds":-1,"max_seconds":30}),
            json!({"min_seconds":18,"max_seconds":12}),
        ] {
            assert!(requested_duration(&args).is_err());
        }
        assert!(matches!(
            requested_duration(&json!({"min_seconds":12.25,"max_seconds":30})),
            Ok(Some(span)) if (span.min_ticks, span.max_ticks) == (1_102_500, 2_700_000)
        ));
    }

    #[test]
    fn a_plan_decides_under_the_id_the_studio_uses() {
        let id = plan_candidate_id("src_1", "job_1", "{\"plan_id\":\"plan_x\"}");
        assert!(id.starts_with("cand_") && id.len() == 21);
        assert_eq!(
            id,
            plan_candidate_id("src_1", "job_1", "{\"plan_id\":\"plan_x\"}")
        );
    }

    #[test]
    fn an_agents_answers_add_to_the_earlier_runs_and_replace_its_old_ones() {
        use clipmill_contracts::proto::ipc::v1::JudgeAnswerV1;
        let earlier = [
            JudgeAnswerV1 {
                question_id: format!("q_{}", "a".repeat(24)),
                yes_per_mille: 1000,
            },
            JudgeAnswerV1 {
                question_id: format!("q_{}", "b".repeat(24)),
                yes_per_mille: 0,
            },
        ];
        let args = json!({"answers": [
            {"question_id": format!("q_{}", "b".repeat(24)), "answer": "yes"},
            {"question_id": format!("q_{}", "c".repeat(24)), "answer": 0.25},
            {"question_id": format!("q_{}", "d".repeat(24)), "answer": "No"},
        ]});
        let merged = judge_answers(&args, &earlier).unwrap_or_default();
        let pairs: Vec<(Option<char>, u32)> = merged
            .iter()
            .map(|a| (a.question_id.chars().nth(2), a.yes_per_mille))
            .collect();
        assert_eq!(
            pairs,
            [
                (Some('a'), 1000),
                (Some('b'), 1000),
                (Some('c'), 250),
                (Some('d'), 0)
            ]
        );
        // Without new answers the earlier ones carry over unchanged.
        assert!(judge_answers(&json!({}), &earlier).is_ok_and(|carried| carried == earlier));
        for bad in [
            json!({"answers": [{"question_id": "q_short", "answer": "yes"}]}),
            json!({"answers": [{"question_id": format!("q_{}", "A".repeat(24)), "answer": "yes"}]}),
            json!({"answers": [{"question_id": format!("q_{}", "e".repeat(24)), "answer": "maybe"}]}),
            json!({"answers": [{"question_id": format!("q_{}", "e".repeat(24)), "answer": 1.5}]}),
        ] {
            assert!(judge_answers(&bad, &[]).is_err(), "{bad}");
        }
    }

    #[test]
    fn every_operation_needs_a_known_scope() {
        for (_, scope) in OPERATIONS {
            assert!(scope == "authenticated" || SCOPES.contains(&scope));
        }
    }
}
