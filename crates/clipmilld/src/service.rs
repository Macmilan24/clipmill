use std::{
    io::{Read, Seek, SeekFrom},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use clipmill_artifacts::{
    ArtifactPath, ArtifactRecipe, NetworkPolicy, PrepareOutcome, Producer, RecipeSpec, Timebase,
};
use clipmill_contracts::proto::ipc::v1::{
    AnalyzeSourcePayloadV1, ApplyEditCommandRequest, CreateEditDocRequest, CreateProjectRequest,
    DemoDagPayloadV1, DetectShotsPayloadV1, DiscoverCandidatesPayloadV1, Error, ErrorCode,
    GetDeviceProfileRequest, GetDeviceProfileResponse, GetEditDocResponse, GetJobResponse,
    GetProjectResponse, GetSourceResponse, GetStorageStatsResponse, HealthResponse,
    IndexTranscriptPayloadV1, IngestSourcePayloadV1, ListJobsResponse, ListProjectsResponse,
    ListSourcesResponse, MediaFileV1, PingResponse, ProbeSourcePayloadV1, RankCandidatesPayloadV1,
    ReadArtifactRequest, ReadArtifactResponse, RegisterSourceRequest, RenderClipPayloadV1, Request,
    ResolveMediaRequest, ResolveMediaResponse, Response, SnapshotEditDocResponse,
    StorageCategoryV1, SubmitJobRequest, SubscribeTaskEventsRequest, SubscribeTaskEventsResponse,
    TranscribeSourcePayloadV1, request, response,
};
use clipmill_core::{EditDocId, JobId, ProjectId, Sha256Digest, SourceId, TaskEventCursor};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::artifacts::ArtifactHandle;
use crate::db::{BeginDeviceProfile, DeviceProfileState};
use crate::db::{DbHandle, ProjectRecord, StoreError};
use crate::device::{DeviceProfiler, verify_profile};
use crate::jobs::{EventFilter, TaskEventRecord};
use crate::jobs::{
    EventHub, INGEST_SOURCE_KEY_VERSION, JobPlan, PROBE_SOURCE_KEY_VERSION, SchedulerHandle,
};
use crate::sources::{SourceInspector, SourceProbeError};
use tokio::sync::broadcast;
use tokio::time::{Instant, sleep};

const REQUEST_ID_MAX_CHARS: usize = 128;
const PROJECT_NAME_MAX_CHARS: usize = 200;
/// Edit documents and commands travel inline; the frame cap is the real
/// limit, this keeps a hostile payload from reaching the parser at all.
const MAX_EDIT_DOCUMENT_BYTES: usize = 8 * 1024 * 1024;
const DEMO_DAG_KEY_VERSION: &str = "clipmill.demo-dag.v1";
const RENDER_CLIP_KEY_VERSION: &str = "clipmill.render-clip.v1";
const TRANSCRIBE_SOURCE_KEY_VERSION: &str = "clipmill.transcribe-source.v1";
const DETECT_SHOTS_KEY_VERSION: &str = "clipmill.detect-shots.v1";
const INDEX_TRANSCRIPT_KEY_VERSION: &str = "clipmill.index-transcript.v1";
const DISCOVER_CANDIDATES_KEY_VERSION: &str = "clipmill.discover-candidates.v1";
const RANK_CANDIDATES_KEY_VERSION: &str = "clipmill.rank-candidates.v1";
const ANALYZE_SOURCE_KEY_VERSION: &str = "clipmill.analyze-source.v1";

#[derive(Clone, Debug)]
pub(crate) struct Service {
    database: DbHandle,
    started_unix_millis: u64,
    events: EventHub,
    scheduler: Option<SchedulerHandle>,
    sources: Option<SourceInspector>,
    artifacts: Option<ArtifactHandle>,
    device_profiler: Option<DeviceProfiler>,
    /// Read when planning a stage that runs a model, so the plan's resource
    /// declaration comes from what the registry pinned rather than a guess.
    models: std::sync::Arc<crate::models::ModelRegistry>,
    /// The three directories a storage report covers. Absent in the tests that
    /// build a service without a workspace, where there is nothing to measure.
    storage: Option<crate::storage::StorageDirs>,
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
            artifacts: None,
            device_profiler: None,
            models: std::sync::Arc::default(),
            storage: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn with_scheduler(
        database: DbHandle,
        started_unix_millis: u64,
        events: EventHub,
        scheduler: SchedulerHandle,
        sources: SourceInspector,
        artifacts: ArtifactHandle,
        device_profiler: DeviceProfiler,
        models: std::sync::Arc<crate::models::ModelRegistry>,
        storage: crate::storage::StorageDirs,
    ) -> Self {
        Self {
            database,
            started_unix_millis,
            events,
            scheduler: Some(scheduler),
            sources: Some(sources),
            artifacts: Some(artifacts),
            device_profiler: Some(device_profiler),
            models,
            storage: Some(storage),
        }
    }

    /// One derivative ingest produced for a source, with the source's
    /// fingerprint.
    ///
    /// Resolved through the ingest manifest rather than by searching the
    /// store, because the manifest is the job's single rooted artifact and its
    /// children are what garbage collection keeps reachable. Anything found
    /// another way might be an object nobody is holding on to.
    async fn ingested_derivative(&self, source_id: &str, kind: &str) -> Option<(String, String)> {
        let artifacts = self.artifacts.as_ref()?;
        let manifest_id = self
            .database
            .latest_source_job_artifact(source_id.to_owned(), "ingest-source".to_owned())
            .await
            .ok()
            .flatten()?
            .parse::<clipmill_core::ArtifactId>()
            .ok()?;
        let (lease, _) =
            crate::media::verified_input_file(artifacts, manifest_id, "ingest-manifest.json")
                .await
                .ok()?;
        let manifest = crate::media::read_descriptor(&lease, "ingest-manifest.json").ok()?;
        let child = manifest["children"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|child| child["kind"] == kind)
            .and_then(|child| child["artifact_id"].as_str())
            .map(ToOwned::to_owned)?;
        let fingerprint = manifest["source_fingerprint"].as_str()?.to_owned();
        Some((child, fingerprint))
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
            request::Body::GetDeviceProfile(get) => {
                self.get_device_profile(request_id, request_hash, &get)
                    .await
            }
            request::Body::CreateEditDoc(create) => {
                self.create_edit_doc(request_id, request_hash, &create)
                    .await
            }
            request::Body::ApplyEditCommand(apply) => {
                self.apply_edit_command(request_id, request_hash, &apply)
                    .await
            }
            request::Body::GetEditDoc(get) => self.get_edit_doc(request_id, &get.doc_id).await,
            request::Body::SnapshotEditDoc(snapshot) => {
                self.snapshot_edit_doc(request_id, &snapshot.doc_id).await
            }
            request::Body::ReadArtifact(read) => self.read_artifact(request_id, &read).await,
            request::Body::ResolveMedia(resolve) => self.resolve_media(request_id, &resolve).await,
            request::Body::GetStorageStats(_) => self.get_storage_stats(request_id).await,
            request::Body::SubscribeTaskEvents(_) => error_reply(
                request_id,
                ErrorCode::Unavailable,
                "operation is not available",
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
            "ingest-source" => {
                let Ok(payload) = IngestSourcePayloadV1::decode(submit.payload.as_slice()) else {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "ingest job payload is not a valid IngestSourcePayloadV1",
                    );
                };
                if payload.key_version != INGEST_SOURCE_KEY_VERSION {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "ingest job payload key_version is unsupported",
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
                let (has_video, has_audio) = source_stream_kinds(&source.source_map_json);
                match JobPlan::ingest_source(
                    &project_id,
                    source_id.to_string(),
                    submit.payload.clone(),
                    has_video,
                    has_audio,
                    now,
                ) {
                    Ok(plan) => plan,
                    Err(message) => {
                        return error_reply(request_id, ErrorCode::InvalidArgument, message);
                    }
                }
            }
            "transcribe-source" => {
                let Ok(payload) = TranscribeSourcePayloadV1::decode(submit.payload.as_slice())
                else {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "transcribe job payload is not a valid TranscribeSourcePayloadV1",
                    );
                };
                if payload.key_version != TRANSCRIBE_SOURCE_KEY_VERSION {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "transcribe job payload key_version is unsupported",
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
                // The speech chain reads what ingest already decoded. Asking
                // it to transcribe a source nobody ingested is a request with
                // no audio behind it, and saying so is more useful than
                // planning four tasks that will each fail to find their input.
                let Some((audio_artifact_id, fingerprint)) = self
                    .ingested_derivative(&source_id.to_string(), "media.audio_16k.v1")
                    .await
                else {
                    return error_reply(
                        request_id,
                        ErrorCode::Conflict,
                        "this source has no ingested 16 kHz audio to transcribe",
                    );
                };
                // Which implementation runs each stage is decided here, once,
                // from the last verified device profile — and written into the
                // plan. A job's stages therefore agree with each other even if
                // the device is re-measured while they run.
                let bindings = self
                    .scheduler
                    .as_ref()
                    .map(crate::jobs::SchedulerHandle::bindings)
                    .unwrap_or_default();
                JobPlan::transcribe_source(
                    &project_id,
                    source_id.to_string(),
                    crate::jobs::SpeechAudio {
                        artifact_id: &audio_artifact_id,
                        source_fingerprint: &fingerprint,
                    },
                    &payload,
                    &self.models,
                    &bindings,
                    now,
                )
            }
            "detect-shots" => {
                let Ok(payload) = DetectShotsPayloadV1::decode(submit.payload.as_slice()) else {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "shots job payload is not a valid DetectShotsPayloadV1",
                    );
                };
                if payload.key_version != DETECT_SHOTS_KEY_VERSION {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "shots job payload key_version is unsupported",
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
                // Shot detection reads the proxy ingest already derived. A
                // source with no proxy is either not ingested or has no video,
                // and saying so is more useful than planning a task that will
                // fail to find its input.
                let Some((proxy_artifact_id, fingerprint)) = self
                    .ingested_derivative(&source_id.to_string(), "media.proxy.v1")
                    .await
                else {
                    return error_reply(
                        request_id,
                        ErrorCode::Conflict,
                        "this source has no ingested proxy to detect shots in",
                    );
                };
                JobPlan::detect_shots(
                    &project_id,
                    source_id.to_string(),
                    crate::jobs::ShotsProxy {
                        artifact_id: &proxy_artifact_id,
                        source_fingerprint: &fingerprint,
                    },
                    &payload,
                    crate::media::FFMPEG_BOM,
                    now,
                )
            }
            "index-transcript" => {
                let Ok(payload) = IndexTranscriptPayloadV1::decode(submit.payload.as_slice())
                else {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "index job payload is not a valid IndexTranscriptPayloadV1",
                    );
                };
                if payload.key_version != INDEX_TRANSCRIPT_KEY_VERSION {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "index job payload key_version is unsupported",
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
                // The index reads what the speech chain published. A source
                // with no transcript is one nobody has transcribed, and saying
                // so beats planning a task with nothing to read.
                let Ok(Some(transcript)) = self
                    .database
                    .latest_source_job_artifact(
                        source_id.to_string(),
                        "transcribe-source".to_owned(),
                    )
                    .await
                else {
                    return error_reply(
                        request_id,
                        ErrorCode::Conflict,
                        "this source has no published transcript to index",
                    );
                };
                // Shot cuts are optional: a source with no video has none, and
                // an index built without them is a different document rather
                // than the same one with a shorter edge list.
                let shots = self
                    .database
                    .latest_source_job_artifact(source_id.to_string(), "detect-shots".to_owned())
                    .await
                    .ok()
                    .flatten();
                JobPlan::index_transcript(
                    &project_id,
                    source_id.to_string(),
                    crate::jobs::EvidenceInputs {
                        transcript: &transcript,
                        shots: shots.as_deref(),
                    },
                    &payload,
                    now,
                )
            }
            "discover-candidates" => {
                let Ok(payload) = DiscoverCandidatesPayloadV1::decode(submit.payload.as_slice())
                else {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "discovery job payload is not a valid DiscoverCandidatesPayloadV1",
                    );
                };
                if payload.key_version != DISCOVER_CANDIDATES_KEY_VERSION {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "discovery job payload key_version is unsupported",
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
                // Discovery searches structure, so it needs the index — and
                // the transcript behind it, for the word boundaries that make
                // a lattice point legal.
                let Ok(Some(index)) = self
                    .database
                    .latest_source_job_artifact(
                        source_id.to_string(),
                        "index-transcript".to_owned(),
                    )
                    .await
                else {
                    return error_reply(
                        request_id,
                        ErrorCode::Conflict,
                        "this source has no evidence index to search",
                    );
                };
                let Ok(Some(transcript)) = self
                    .database
                    .latest_source_job_artifact(
                        source_id.to_string(),
                        "transcribe-source".to_owned(),
                    )
                    .await
                else {
                    return error_reply(
                        request_id,
                        ErrorCode::Conflict,
                        "this source has no published transcript",
                    );
                };
                // Prosody is optional evidence: without it the quote proposer
                // weighs three proxies instead of four rather than assuming a
                // delivery nobody measured.
                let loudness = self
                    .ingested_derivative(&source_id.to_string(), "media.loudness_envelope.v1")
                    .await
                    .map(|(artifact_id, _)| artifact_id);
                JobPlan::discover_candidates(
                    &project_id,
                    source_id.to_string(),
                    crate::jobs::DiscoveryInputs {
                        index: &index,
                        transcript: &transcript,
                        loudness: loudness.as_deref(),
                    },
                    &payload,
                    now,
                )
            }
            "rank-candidates" => {
                let Ok(payload) = RankCandidatesPayloadV1::decode(submit.payload.as_slice()) else {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "ranking job payload is not a valid RankCandidatesPayloadV1",
                    );
                };
                if payload.key_version != RANK_CANDIDATES_KEY_VERSION {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "ranking job payload key_version is unsupported",
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
                let mut found = Vec::new();
                for kind in [
                    "discover-candidates",
                    "index-transcript",
                    "transcribe-source",
                ] {
                    let Ok(Some(artifact)) = self
                        .database
                        .latest_source_job_artifact(source_id.to_string(), kind.to_owned())
                        .await
                    else {
                        return error_reply(
                            request_id,
                            ErrorCode::Conflict,
                            format!("this source has no published {kind} to rank from"),
                        );
                    };
                    found.push(artifact);
                }
                JobPlan::rank_candidates(
                    &project_id,
                    source_id.to_string(),
                    crate::jobs::RankingJobInputs {
                        candidates: &found[0],
                        index: &found[1],
                        transcript: &found[2],
                    },
                    &payload,
                    now,
                )
            }
            "analyze-source" => {
                let Ok(payload) = AnalyzeSourcePayloadV1::decode(submit.payload.as_slice()) else {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "analyze job payload is not a valid AnalyzeSourcePayloadV1",
                    );
                };
                if payload.key_version != ANALYZE_SOURCE_KEY_VERSION {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "analyze job payload key_version is unsupported",
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
                // The plan's shape depends on which streams the file has, and
                // that is a measurement rather than a guess. A source map with
                // no streams in it is one nobody has probed: refusing here beats
                // planning a fan-out that finds nothing to decode.
                let (has_video, has_audio) = source_stream_kinds(&source.source_map_json);
                if !has_video && !has_audio {
                    return error_reply(
                        request_id,
                        ErrorCode::Conflict,
                        "this source has not been probed, so nothing knows which streams it has",
                    );
                }
                let bindings = self
                    .scheduler
                    .as_ref()
                    .map(crate::jobs::SchedulerHandle::bindings)
                    .unwrap_or_default();
                match JobPlan::analyze_source(
                    &project_id,
                    crate::jobs::AnalyzeSource {
                        source_id: &source_id.to_string(),
                        source_fingerprint: &source.source_fingerprint,
                        has_video,
                        has_audio,
                    },
                    &payload,
                    &self.models,
                    &bindings,
                    crate::media::FFMPEG_BOM,
                    now,
                ) {
                    Ok(plan) => plan,
                    Err(message) => {
                        return error_reply(request_id, ErrorCode::InvalidArgument, message);
                    }
                }
            }
            "render-clip" => {
                let Ok(payload) = RenderClipPayloadV1::decode(submit.payload.as_slice()) else {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "render job payload is not a valid RenderClipPayloadV1",
                    );
                };
                if payload.key_version != RENDER_CLIP_KEY_VERSION {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "render job payload key_version is unsupported",
                    );
                }
                // The snapshot must belong to a document in this project.
                // Rendering someone else's document through a project id the
                // caller happens to hold would be a cross-project read.
                let doc = match self.database.get_edit_doc(payload.doc_id.clone()).await {
                    Ok(doc) => doc,
                    Err(error) => return store_error_reply(request_id, &error),
                };
                if doc.project_id != project_id.as_str() {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "edit document does not belong to the requested project",
                    );
                }
                if payload
                    .ir_artifact_id
                    .parse::<clipmill_core::ArtifactId>()
                    .is_err()
                {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "render job names no edit snapshot to render",
                    );
                }
                if payload.source_attestation.trim().is_empty() {
                    // The manifest states a rights position; there is no
                    // honest default for one the user never made.
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "a render requires a rights attestation",
                    );
                }
                if let Some(unknown) = payload
                    .ai_assistance
                    .iter()
                    .find(|token| !crate::render::ai_assistance_is_known(token))
                {
                    tracing::debug!(%unknown, "render declined an unrecognised disclosure token");
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "AI-use disclosure carries a token this version does not define",
                    );
                }
                JobPlan::render_clip(&project_id, submit.payload.clone(), now)
            }
            _ => {
                return error_reply(
                    request_id,
                    ErrorCode::Unavailable,
                    "job kind is not available",
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

    /// Serve one published document to a shell.
    ///
    /// Four refusals before a byte is read, in this order because each one makes
    /// the next cheaper to trust:
    ///
    ///   the address parses, so no path or fragment reaches the store;
    ///   the project produced it, so a renderer cannot read another project's
    ///   observations with an address it guessed or kept;
    ///   the kind is on the list, so weights and media are unreachable here
    ///   whatever the caller asks;
    ///   the artifact verifies against its own manifest, which `open_verified`
    ///   does by re-hashing — a corrupt object is refused rather than rendered.
    ///
    /// Only then is the requested window copied out. The window is clamped
    /// rather than rejected: a caller reading to the end of a document should not
    /// have to know its length first, and the response states the total so the
    /// next call knows where to stop.
    #[allow(
        clippy::too_many_lines,
        reason = "four refusals then a bounded read; each one names what it rejects"
    )]
    async fn read_artifact(&self, request_id: String, read: &ReadArtifactRequest) -> Reply {
        let Ok(project_id) = read.project_id.parse::<ProjectId>() else {
            return error_reply(
                request_id,
                ErrorCode::InvalidArgument,
                "read names no project",
            );
        };
        let Ok(artifact_id) = read.artifact_id.parse::<clipmill_core::ArtifactId>() else {
            return error_reply(
                request_id,
                ErrorCode::InvalidArgument,
                "read names no artifact address",
            );
        };
        let Some(artifacts) = self.artifacts.as_ref() else {
            return error_reply(
                request_id,
                ErrorCode::Unavailable,
                "this daemon serves no artifact store",
            );
        };
        // Not found rather than denied, and deliberately: a project that learned
        // "that exists, but not for you" would learn something about another
        // project from an address it was not given.
        match self
            .database
            .artifact_is_project_output(project_id.to_string(), artifact_id.to_string())
            .await
        {
            Ok(true) => {}
            Ok(false) => {
                return error_reply(
                    request_id,
                    ErrorCode::NotFound,
                    "this project published no such artifact",
                );
            }
            Err(error) => return store_error_reply(request_id, &error),
        }
        let Ok(lease) = artifacts.open(artifact_id).await else {
            return error_reply(
                request_id,
                ErrorCode::NotFound,
                "the artifact is not in this store",
            );
        };
        let kind = lease.kind().to_owned();
        let Some(file_name) = crate::shell::document_for(&kind) else {
            return error_reply(
                request_id,
                ErrorCode::PolicyDenied,
                format!("{kind} is not a kind a shell may read"),
            );
        };
        let Ok(path) = file_name.parse::<ArtifactPath>() else {
            return error_reply(
                request_id,
                ErrorCode::Internal,
                "the allowlist names an invalid artifact path",
            );
        };
        // Re-verified here, every read. The store is the daemon's own, and that
        // is exactly why: a corrupt object it hands out under a content address
        // is the one failure the address cannot reveal.
        let mut file = match lease.open_verified(&path) {
            Ok(file) => file,
            Err(error) => {
                tracing::warn!(%kind, %error, "a published document failed verification");
                return error_reply(
                    request_id,
                    ErrorCode::Internal,
                    "the document does not match its manifest",
                );
            }
        };
        let Ok(total_bytes) = file.metadata().map(|data| data.len()) else {
            return error_reply(
                request_id,
                ErrorCode::Internal,
                "the document has no readable size",
            );
        };
        if read.offset > total_bytes {
            return error_reply(
                request_id,
                ErrorCode::InvalidArgument,
                "read starts past the end of the document",
            );
        }
        let wanted = if read.length == 0 {
            crate::shell::MAX_CHUNK_BYTES
        } else {
            read.length.min(crate::shell::MAX_CHUNK_BYTES)
        };
        let remaining = total_bytes - read.offset;
        let take = wanted.min(remaining);
        if file.seek(SeekFrom::Start(read.offset)).is_err() {
            return error_reply(
                request_id,
                ErrorCode::Internal,
                "the document cannot be positioned",
            );
        }
        let mut chunk = vec![0_u8; usize::try_from(take).unwrap_or(0)];
        if file.read_exact(&mut chunk).is_err() {
            return error_reply(
                request_id,
                ErrorCode::Internal,
                "the document ended before the requested window",
            );
        }
        response_reply(
            request_id,
            response::Body::ReadArtifact(ReadArtifactResponse {
                artifact_id: artifact_id.to_string(),
                kind,
                path: file_name.to_owned(),
                offset: read.offset,
                total_bytes,
                chunk,
            }),
        )
    }

    /// What this installation is using on disk, by category.
    ///
    /// The store answers for artifacts from manifests it already holds. The
    /// other two are directory walks, so the whole measurement goes to a
    /// blocking thread — small trees today, but a screen asking how much disk it
    /// is using must never be the thing that stalls the event loop.
    async fn get_storage_stats(&self, request_id: String) -> Reply {
        let (Some(artifacts), Some(dirs)) = (self.artifacts.as_ref(), self.storage.clone()) else {
            return error_reply(
                request_id,
                ErrorCode::Unavailable,
                "this daemon measures no storage",
            );
        };
        let Ok(usage) = artifacts.usage().await else {
            return error_reply(
                request_id,
                ErrorCode::Unavailable,
                "the artifact store is not answering",
            );
        };
        let Ok(report) = tokio::task::spawn_blocking(move || dirs.measure(usage)).await else {
            return error_reply(
                request_id,
                ErrorCode::Internal,
                "the storage measurement did not finish",
            );
        };
        let category = |key: &str, measured: crate::storage::Category| StorageCategoryV1 {
            key: key.to_owned(),
            bytes: measured.bytes,
            items: measured.items,
        };
        response_reply(
            request_id,
            response::Body::GetStorageStats(GetStorageStatsResponse {
                categories: vec![
                    category(crate::storage::ARTIFACTS, report.artifacts),
                    category(crate::storage::MODELS, report.models),
                    category(crate::storage::STATE, report.state),
                ],
                available_bytes: report.available_bytes.unwrap_or(0),
                available_known: report.available_bytes.is_some(),
            }),
        )
    }

    /// Authorize a media artifact and say what it holds.
    ///
    /// The same two policy checks `ReadArtifact` makes — the project produced it,
    /// the kind is on a list — and then a third the document door does not need:
    /// the file names come from the artifact's own descriptor, so the protocol
    /// can refuse a name the descriptor never mentioned without opening anything.
    ///
    /// No path in the response. The caller derives the object directory from the
    /// content address the same way the store does, so nothing here can be turned
    /// into a pointer outside it.
    #[allow(
        clippy::too_many_lines,
        reason = "the refusal ladder, then one check per file the descriptor named"
    )]
    async fn resolve_media(&self, request_id: String, resolve: &ResolveMediaRequest) -> Reply {
        let Ok(project_id) = resolve.project_id.parse::<ProjectId>() else {
            return error_reply(
                request_id,
                ErrorCode::InvalidArgument,
                "resolve names no project",
            );
        };
        let Ok(artifact_id) = resolve.artifact_id.parse::<clipmill_core::ArtifactId>() else {
            return error_reply(
                request_id,
                ErrorCode::InvalidArgument,
                "resolve names no artifact address",
            );
        };
        let Some(artifacts) = self.artifacts.as_ref() else {
            return error_reply(
                request_id,
                ErrorCode::Unavailable,
                "this daemon serves no artifact store",
            );
        };
        match self
            .database
            .artifact_is_project_output(project_id.to_string(), artifact_id.to_string())
            .await
        {
            Ok(true) => {}
            Ok(false) => {
                return error_reply(
                    request_id,
                    ErrorCode::NotFound,
                    "this project published no such artifact",
                );
            }
            Err(error) => return store_error_reply(request_id, &error),
        }
        let Ok(lease) = artifacts.open(artifact_id).await else {
            return error_reply(
                request_id,
                ErrorCode::NotFound,
                "the artifact is not in this store",
            );
        };
        let kind = lease.kind().to_owned();
        let Some((descriptor_file, layout)) = crate::shell::media_descriptor_for(&kind) else {
            return error_reply(
                request_id,
                ErrorCode::PolicyDenied,
                format!("{kind} is not a kind a shell may stream"),
            );
        };
        let Ok(descriptor) =
            crate::media::read_artifact_document::<serde_json::Value>(&lease, descriptor_file)
        else {
            return error_reply(
                request_id,
                ErrorCode::Internal,
                "the media descriptor does not match its manifest",
            );
        };
        let named = crate::shell::media_files(&descriptor, layout);
        if named.is_empty() {
            return error_reply(
                request_id,
                ErrorCode::Internal,
                "the media descriptor names no files",
            );
        }
        // Every named file is checked against the manifest that published it, so
        // the response cannot promise bytes that are not there — and its declared
        // size is the manifest's, not the descriptor's, because the manifest is
        // what the digest covers.
        let mut files = Vec::with_capacity(named.len());
        for name in named {
            let Some(media_type) = crate::shell::media_type_for(&name) else {
                return error_reply(
                    request_id,
                    ErrorCode::PolicyDenied,
                    format!("{name} is not a file type a shell may stream"),
                );
            };
            let Ok(path) = name.parse::<ArtifactPath>() else {
                return error_reply(
                    request_id,
                    ErrorCode::Internal,
                    "the descriptor names an invalid artifact path",
                );
            };
            let Some(bytes) = lease.declared_bytes(&path) else {
                return error_reply(
                    request_id,
                    ErrorCode::Internal,
                    format!("{name} is named by the descriptor and not by the manifest"),
                );
            };
            files.push(MediaFileV1 {
                path: name,
                bytes,
                media_type: media_type.to_owned(),
            });
        }
        response_reply(
            request_id,
            response::Body::ResolveMedia(ResolveMediaResponse {
                artifact_id: artifact_id.to_string(),
                kind,
                files,
            }),
        )
    }

    #[allow(clippy::too_many_lines)]
    async fn get_device_profile(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        request: &GetDeviceProfileRequest,
    ) -> Reply {
        let (Some(profiler), Some(artifacts)) = (&self.device_profiler, &self.artifacts) else {
            return error_reply(
                request_id,
                ErrorCode::Unavailable,
                "device profiling is not available",
            );
        };
        let fingerprint = match profiler.hardware_fingerprint().await {
            Ok(fingerprint) => fingerprint,
            Err(error) => {
                tracing::warn!(operation = "device-profile", %error, "device fingerprint failed");
                return error_reply(
                    request_id,
                    ErrorCode::Internal,
                    "device fingerprint could not be measured",
                );
            }
        };
        let now = match unix_millis() {
            Ok(now) => now,
            Err(message) => return error_reply(request_id, ErrorCode::Internal, message),
        };
        let (started, cached_response) = match self
            .database
            .begin_device_profile(
                request_id.clone(),
                request_hash,
                fingerprint.clone(),
                request.remeasure,
                now,
            )
            .await
        {
            Ok(BeginDeviceProfile::Response { bytes, record }) => (record, Some(bytes)),
            Ok(BeginDeviceProfile::Profile { record, events }) => {
                self.events.publish_all(events);
                if let Some(scheduler) = &self.scheduler {
                    scheduler.notify();
                }
                (record, None)
            }
            Err(error) => return store_error_reply(request_id, &error),
        };
        let deadline = Instant::now() + Duration::from_secs(30);
        let record = loop {
            match started.state {
                DeviceProfileState::Succeeded => break started.clone(),
                DeviceProfileState::Failed => {
                    return error_reply(
                        request_id,
                        ErrorCode::Internal,
                        "device-profile job failed",
                    );
                }
                DeviceProfileState::Pending => {}
            }
            if Instant::now() >= deadline {
                return error_reply(
                    request_id,
                    ErrorCode::Unavailable,
                    "device-profile job is still running; retry the same request_id",
                );
            }
            sleep(Duration::from_millis(50)).await;
            match self
                .database
                .device_profile_for_job(started.job_id.clone())
                .await
            {
                Ok(current) if current.state == DeviceProfileState::Succeeded => break current,
                Ok(current) if current.state == DeviceProfileState::Failed => {
                    return error_reply(
                        request_id,
                        ErrorCode::Internal,
                        "device-profile job failed",
                    );
                }
                Ok(_) => {}
                Err(error) => return store_error_reply(request_id, &error),
            }
        };
        let Some(artifact_id) = record.artifact_id else {
            return error_reply(
                request_id,
                ErrorCode::Internal,
                "device profile completed without an artifact",
            );
        };
        let Some(profile_json) = record.profile_json else {
            return error_reply(
                request_id,
                ErrorCode::Internal,
                "device profile completed without measured JSON",
            );
        };
        let verified = match verify_profile(&profile_json, Some(&fingerprint)) {
            Ok(verified) => verified,
            Err(error) => {
                tracing::warn!(%artifact_id, %error, "cached device profile verification failed");
                return error_reply(
                    request_id,
                    ErrorCode::Internal,
                    "cached device profile verification failed",
                );
            }
        };
        let lease = match artifacts.open(artifact_id).await {
            Ok(lease) => lease,
            Err(error) => {
                tracing::warn!(%artifact_id, %error, "device profile artifact could not be opened");
                return error_reply(
                    request_id,
                    ErrorCode::Internal,
                    "device profile artifact could not be verified",
                );
            }
        };
        let Ok(profile_path) = "profile.json".parse::<ArtifactPath>() else {
            return error_reply(
                request_id,
                ErrorCode::Internal,
                "device profile artifact path is invalid",
            );
        };
        let mut stored_profile = String::new();
        let profile_read = lease
            .open_verified(&profile_path)
            .map_err(|error| error.to_string())
            .and_then(|mut file| {
                file.read_to_string(&mut stored_profile)
                    .map(|_| ())
                    .map_err(|error| error.to_string())
            });
        if profile_read.is_err() || stored_profile != profile_json {
            return error_reply(
                request_id,
                ErrorCode::Internal,
                "device profile artifact payload does not match durable state",
            );
        }
        if let Some(scheduler) = &self.scheduler {
            scheduler.apply_device_profile(&verified);
        }
        if let Some(bytes) = cached_response {
            let cached_matches = Response::decode(bytes.as_slice())
                .ok()
                .filter(|response| response.request_id == request_id)
                .and_then(|response| response.body)
                .is_some_and(|body| {
                    matches!(
                        body,
                        response::Body::GetDeviceProfile(profile)
                            if profile.artifact_id == artifact_id.to_string()
                                && profile.profile_json == profile_json
                    )
                });
            if !cached_matches {
                return error_reply(
                    request_id,
                    ErrorCode::Internal,
                    "durable device profile response is inconsistent",
                );
            }
            return Reply {
                bytes,
                outcome: Outcome::Success,
            };
        }
        let response = Response {
            request_id: request_id.clone(),
            body: Some(response::Body::GetDeviceProfile(GetDeviceProfileResponse {
                artifact_id: artifact_id.to_string(),
                profile_json,
            })),
        }
        .encode_to_vec();
        let completed = unix_millis().unwrap_or(now);
        match self
            .database
            .finish_device_profile_request(
                request_id.clone(),
                request_hash,
                artifact_id,
                response,
                completed,
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

    async fn create_edit_doc(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        create: &CreateEditDocRequest,
    ) -> Reply {
        let project_id = match create.project_id.parse::<ProjectId>() {
            Ok(value) => value,
            Err(error) => {
                return error_reply(request_id, ErrorCode::InvalidArgument, error.to_string());
            }
        };
        if create.document_json.len() > MAX_EDIT_DOCUMENT_BYTES {
            return error_reply(
                request_id,
                ErrorCode::InvalidArgument,
                "initial edit document exceeds 8 MiB",
            );
        }
        let now = match unix_millis() {
            Ok(now) => now,
            Err(message) => return error_reply(request_id, ErrorCode::Internal, message),
        };
        match self
            .database
            .create_edit_doc(
                request_id.clone(),
                request_hash,
                project_id.to_string(),
                create.document_json.clone(),
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

    async fn apply_edit_command(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        apply: &ApplyEditCommandRequest,
    ) -> Reply {
        let doc_id = match apply.doc_id.parse::<EditDocId>() {
            Ok(value) => value,
            Err(error) => {
                return error_reply(request_id, ErrorCode::InvalidArgument, error.to_string());
            }
        };
        if apply.command_json.len() > MAX_EDIT_DOCUMENT_BYTES {
            return error_reply(
                request_id,
                ErrorCode::InvalidArgument,
                "edit command exceeds 8 MiB",
            );
        }
        let now = match unix_millis() {
            Ok(now) => now,
            Err(message) => return error_reply(request_id, ErrorCode::Internal, message),
        };
        match self
            .database
            .apply_edit_command(
                request_id.clone(),
                request_hash,
                doc_id.to_string(),
                apply.expected_revision,
                apply.command_json.clone(),
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

    async fn get_edit_doc(&self, request_id: String, value: &str) -> Reply {
        let doc_id = match value.parse::<EditDocId>() {
            Ok(value) => value,
            Err(error) => {
                return error_reply(request_id, ErrorCode::InvalidArgument, error.to_string());
            }
        };
        match self.database.get_edit_doc(doc_id.to_string()).await {
            Ok(record) => response_reply(
                request_id,
                response::Body::GetEditDoc(GetEditDocResponse {
                    doc: Some(record.into()),
                }),
            ),
            Err(error) => store_error_reply(request_id, &error),
        }
    }

    /// Freeze the current document into the immutable `edit.ir.v1` artifact a
    /// render consumes.
    ///
    /// The snapshot carries the render projection, not the whole document:
    /// `rationale` explains an edit and is never rendered, so keeping it out
    /// of this artifact makes "explanation cannot perturb pixels" a property
    /// of the content address rather than a promise. Re-explaining an edit
    /// therefore cannot invalidate a render cache.
    async fn snapshot_edit_doc(&self, request_id: String, value: &str) -> Reply {
        let doc_id = match value.parse::<EditDocId>() {
            Ok(value) => value,
            Err(error) => {
                return error_reply(request_id, ErrorCode::InvalidArgument, error.to_string());
            }
        };
        let Some(artifacts) = &self.artifacts else {
            return error_reply(
                request_id,
                ErrorCode::Unavailable,
                "artifact publication is not available",
            );
        };
        let record = match self.database.get_edit_doc(doc_id.to_string()).await {
            Ok(record) => record,
            Err(error) => return store_error_reply(request_id, &error),
        };
        let document = match clipmill_edit_ir::EditDocument::from_canonical_json(
            record.document_json.as_bytes(),
        ) {
            Ok(document) => document,
            Err(error) => {
                tracing::warn!(%error, "stored edit document failed to parse");
                return error_reply(
                    request_id,
                    ErrorCode::Internal,
                    "stored edit document is not valid",
                );
            }
        };
        let Ok(project_id) = record.project_id.parse::<ProjectId>() else {
            return error_reply(
                request_id,
                ErrorCode::Internal,
                "stored edit document has an invalid project",
            );
        };
        let Ok((recipe, payload)) = Self::snapshot_recipe(&document) else {
            return error_reply(
                request_id,
                ErrorCode::Internal,
                "edit document could not be projected for render",
            );
        };
        let artifact_id = match Self::publish_snapshot(artifacts, recipe, &payload).await {
            Ok(artifact_id) => artifact_id,
            Err((code, message)) => return error_reply(request_id, code, message),
        };
        if let Err(error) = self
            .database
            .attach_artifact_root(project_id, artifact_id)
            .await
        {
            return store_error_reply(request_id, &error);
        }
        response_reply(
            request_id,
            response::Body::SnapshotEditDoc(SnapshotEditDocResponse {
                artifact_id: artifact_id.to_string(),
                revision: record.revision,
            }),
        )
    }

    /// The render projection and the recipe that content-addresses it.
    /// Identical documents therefore resolve to one artifact, and a changed
    /// rationale resolves to the same one.
    fn snapshot_recipe(
        document: &clipmill_edit_ir::EditDocument,
    ) -> Result<(ArtifactRecipe, Vec<u8>), ()> {
        let projection = document.render_projection().map_err(|_| ())?;
        let payload = serde_json_canonicalizer::to_vec(&projection).map_err(|_| ())?;
        let payload_digest = Sha256Digest::from_bytes(Sha256::digest(&payload).into());
        // Lineage points at the media when there is any; a document with no
        // segments is its own origin.
        let source_fingerprint = document
            .video
            .segments
            .first()
            .and_then(|segment| segment.source_fingerprint.strip_prefix("sha256:"))
            .and_then(|digest| digest.parse::<Sha256Digest>().ok())
            .unwrap_or(payload_digest);
        let mut config = serde_json::Map::new();
        config.insert(
            "document_sha256".to_owned(),
            serde_json::Value::String(format!("sha256:{payload_digest}")),
        );
        config.insert(
            "projection".to_owned(),
            serde_json::Value::String("render".to_owned()),
        );
        let recipe = ArtifactRecipe::try_from_spec(RecipeSpec {
            kind: "edit.ir.v1".to_owned(),
            source_fingerprint,
            timebase: Timebase {
                num: 1,
                den: 90_000,
            },
            producer: Producer {
                stage: "snapshot-edit-doc".to_owned(),
                implementation: "clipmill-edit-ir@1.0.0".to_owned(),
                model_digest: None,
            },
            inputs: Vec::new(),
            policy: NetworkPolicy::LocalLock,
            config,
            semantic_version: "clipmill.edit_ir.v1".to_owned(),
        })
        .map_err(|_| ())?;
        Ok((recipe, payload))
    }

    /// Publish the snapshot bytes, or explain why not. A failed staging area
    /// is abandoned so a retry can re-prepare the same key.
    async fn publish_snapshot(
        artifacts: &ArtifactHandle,
        recipe: ArtifactRecipe,
        payload: &[u8],
    ) -> Result<clipmill_core::ArtifactId, (ErrorCode, &'static str)> {
        match artifacts.prepare(recipe).await {
            Ok(PrepareOutcome::Hit(lease)) => Ok(lease.artifact_id()),
            Ok(PrepareOutcome::InFlight { .. }) => Err((
                ErrorCode::Unavailable,
                "an identical snapshot is already being published; retry",
            )),
            Ok(PrepareOutcome::Miss(staging)) => {
                let staging_id = staging.id().clone();
                let staged = Self::write_snapshot(&staging, payload);
                let path = match staged {
                    Ok(path) => path,
                    Err(message) => {
                        let _abandoned = artifacts.abandon(staging_id).await;
                        return Err((ErrorCode::Internal, message));
                    }
                };
                artifacts
                    .commit(staging_id, vec![path], std::collections::BTreeMap::new())
                    .await
                    .map(|lease| lease.artifact_id())
                    .map_err(|error| {
                        tracing::warn!(%error, "edit snapshot could not be committed");
                        (ErrorCode::Internal, "edit snapshot could not be published")
                    })
            }
            Err(error) => {
                tracing::warn!(%error, "edit snapshot could not be prepared");
                Err((ErrorCode::Internal, "edit snapshot could not be prepared"))
            }
        }
    }

    fn write_snapshot(
        staging: &clipmill_artifacts::StagingArea,
        payload: &[u8],
    ) -> Result<ArtifactPath, &'static str> {
        use std::io::Write;
        let path = "edit-ir.json"
            .parse::<ArtifactPath>()
            .map_err(|_| "edit snapshot path is invalid")?;
        let mut file = staging
            .create_file(&path)
            .map_err(|_| "edit snapshot could not be staged")?;
        file.write_all(payload)
            .and_then(|()| file.sync_all())
            .map_err(|_| "edit snapshot could not be written")?;
        Ok(path)
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

/// Read which stream kinds the registered source map observed, so the ingest
/// plan only schedules derivatives the source can actually produce.
fn source_stream_kinds(source_map_json: &[u8]) -> (bool, bool) {
    let Ok(map) = serde_json::from_slice::<serde_json::Value>(source_map_json) else {
        return (false, false);
    };
    let mut has_video = false;
    let mut has_audio = false;
    for stream in map["streams"].as_array().into_iter().flatten() {
        match stream["kind"].as_str() {
            Some("video") => has_video = true,
            Some("audio") => has_audio = true,
            _ => {}
        }
    }
    (has_video, has_audio)
}

pub(crate) fn request_kind(request: &Request) -> &'static str {
    match request.body.as_ref() {
        Some(request::Body::ReadArtifact(_)) => "read_artifact",
        Some(request::Body::ResolveMedia(_)) => "resolve_media",
        Some(request::Body::GetStorageStats(_)) => "get_storage_stats",
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
        Some(request::Body::CreateEditDoc(_)) => "create_edit_doc",
        Some(request::Body::ApplyEditCommand(_)) => "apply_edit_command",
        Some(request::Body::GetEditDoc(_)) => "get_edit_doc",
        Some(request::Body::SnapshotEditDoc(_)) => "snapshot_edit_doc",
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
