use std::{
    io::{Read, Seek, SeekFrom},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use clipmill_artifacts::{
    ArtifactPath, ArtifactRecipe, NetworkPolicy, PrepareOutcome, Producer, RecipeSpec, Timebase,
};
use clipmill_contracts::proto::ipc::v1::{
    AnalyzeSourcePayloadV1, ApplyEditCommandRequest, ClipCutV1, ClipDecisionRecordV1,
    ClipDecisionV1, CreateEditDocRequest, CreateProjectRequest, CropKeyframeV1, CropWeightsV1,
    DemoDagPayloadV1, DeriveCaptionsPayloadV1, DetectFacesPayloadV1, DetectShotsPayloadV1,
    DirectClipRequest, DirectClipResponse, DiscoverCandidatesPayloadV1, Error, ErrorCode,
    GetDeviceProfileRequest, GetDeviceProfileResponse, GetEditDocResponse, GetJobResponse,
    GetPreviewPlanRequest, GetPreviewPlanResponse, GetProjectResponse, GetSourceResponse,
    GetStorageStatsResponse, HealthResponse, IndexTranscriptPayloadV1, IngestSourcePayloadV1,
    ListClipDecisionsRequest, ListClipDecisionsResponse, ListEditDocsResponse, ListJobsResponse,
    ListProjectsResponse, ListSourcesResponse, MediaFileV1, PingResponse, PreviewCropV1,
    PreviewCueV1, PreviewGainV1, PreviewLineV1, PreviewWordV1, ProbeSourcePayloadV1,
    RankCandidatesPayloadV1, ReadArtifactRequest, ReadArtifactResponse, RegisterSourceRequest,
    RenderClipPayloadV1, Request, ResolveMediaRequest, ResolveMediaResponse, Response,
    SetClipDecisionRequest, SetClipDecisionResponse, SnapshotEditDocResponse, SolveCropPathRequest,
    SolveCropPathResponse, StorageCategoryV1, SubmitJobRequest, SubscribeTaskEventsRequest,
    SubscribeTaskEventsResponse, TranscribeSourcePayloadV1, request, response,
};
use clipmill_core::{EditDocId, JobId, ProjectId, Sha256Digest, SourceId, TaskEventCursor};
use clipmill_reframe::{FocusGate, Weights};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::artifacts::ArtifactHandle;
use crate::db::{BeginDeviceProfile, Decision, DeviceProfileState};
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
const DETECT_FACES_KEY_VERSION: &str = "clipmill.detect-faces.v1";
const INDEX_TRANSCRIPT_KEY_VERSION: &str = "clipmill.index-transcript.v1";
const DISCOVER_CANDIDATES_KEY_VERSION: &str = "clipmill.discover-candidates.v1";
const RANK_CANDIDATES_KEY_VERSION: &str = "clipmill.rank-candidates.v1";
const DERIVE_CAPTIONS_KEY_VERSION: &str = "clipmill.derive-captions.v1";
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

    #[allow(
        clippy::too_many_lines,
        reason = "one arm per request kind; splitting it would hide the surface"
    )]
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
            request::Body::SolveCropPath(solve) => self.solve_crop_path(request_id, &solve).await,
            request::Body::DirectClip(direct) => {
                self.direct_clip(request_id, request_hash, &direct).await
            }
            request::Body::SetClipDecision(decide) => {
                self.set_clip_decision(request_id, &decide).await
            }
            request::Body::ListClipDecisions(list) => {
                self.list_clip_decisions(request_id, &list).await
            }
            request::Body::GetPreviewPlan(plan) => self.get_preview_plan(request_id, &plan).await,
            request::Body::ListEditDocs(list) => {
                self.list_edit_docs(request_id, &list.project_id).await
            }
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
            "detect-faces" => {
                let Ok(payload) = DetectFacesPayloadV1::decode(submit.payload.as_slice()) else {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "faces job payload is not a valid DetectFacesPayloadV1",
                    );
                };
                if payload.key_version != DETECT_FACES_KEY_VERSION {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "faces job payload key_version is unsupported",
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
                // Faces are detected on the frames ingest already sampled, not
                // on a decode of this stage's own. A source with no frames is
                // either not ingested or has no video, and saying so beats
                // planning a task that will fail to find its input.
                let Some((frames_artifact_id, fingerprint)) = self
                    .ingested_derivative(&source_id.to_string(), "media.frames.v1")
                    .await
                else {
                    return error_reply(
                        request_id,
                        ErrorCode::Conflict,
                        "this source has no sampled frames to detect faces in",
                    );
                };
                JobPlan::detect_faces(
                    &project_id,
                    source_id.to_string(),
                    crate::jobs::FacesFrames {
                        artifact_id: &frames_artifact_id,
                        source_fingerprint: &fingerprint,
                    },
                    &payload,
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
            "derive-captions" => {
                let Ok(payload) = DeriveCaptionsPayloadV1::decode(submit.payload.as_slice()) else {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "captions job payload is not a valid DeriveCaptionsPayloadV1",
                    );
                };
                if payload.key_version != DERIVE_CAPTIONS_KEY_VERSION {
                    return error_reply(
                        request_id,
                        ErrorCode::InvalidArgument,
                        "captions job payload key_version is unsupported",
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
                // Words are the one thing captions cannot be derived without.
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
                        "this source has no transcript to derive captions from",
                    );
                };
                // The other two make the segmentation better informed rather
                // than possible, and their absence is recorded in the document.
                let index = self
                    .database
                    .latest_source_job_artifact(
                        source_id.to_string(),
                        "index-transcript".to_owned(),
                    )
                    .await
                    .ok()
                    .flatten();
                let shots = self
                    .database
                    .latest_source_job_artifact(source_id.to_string(), "detect-shots".to_owned())
                    .await
                    .ok()
                    .flatten();
                JobPlan::derive_captions(
                    &project_id,
                    source_id.to_string(),
                    crate::jobs::CaptionsJobInputs {
                        transcript: &transcript,
                        index: index.as_deref(),
                        shots: shots.as_deref(),
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

    /// Solve a crop path over one span of one face-track artifact.
    ///
    /// Not a job, and nothing here is written. The answer is wanted while
    /// somebody is looking at a clip, it is arithmetic over evidence that
    /// already exists, and what comes back is a proposal the caller may keep or
    /// discard — which is what makes re-solving after a nudge free and what
    /// stops a re-run mutating an edit somebody accepted.
    ///
    /// The same refusal ladder every artifact read runs: the address parses,
    /// this project produced it, the kind is the one asked for, and the store
    /// re-verifies the object before a byte is parsed.
    /// Turn an approved candidate into an edit document.
    ///
    /// Assembling and creating in one call rather than two: a caller that
    /// assembled, then created, would have a window where a clip is half
    /// approved, and nothing downstream could tell that state from a crash.
    async fn direct_clip(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        direct: &DirectClipRequest,
    ) -> Reply {
        let Ok(project_id) = direct.project_id.parse::<ProjectId>() else {
            return error_reply(request_id, ErrorCode::InvalidArgument, "no project named");
        };
        let Ok(source_id) = direct.source_id.parse::<SourceId>() else {
            return error_reply(request_id, ErrorCode::InvalidArgument, "no source named");
        };
        let Some(artifacts) = self.artifacts.as_ref() else {
            return error_reply(
                request_id,
                ErrorCode::Unavailable,
                "this daemon serves no artifact store",
            );
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

        let evidence = match crate::inspector::load(
            &self.database,
            artifacts,
            &source_id.to_string(),
            &source.source_map_json,
        )
        .await
        {
            Ok(evidence) => evidence,
            Err(error) => {
                return error_reply(request_id, ErrorCode::Conflict, error.message());
            }
        };

        let document = match assemble(&evidence, direct) {
            Ok(document) => document,
            Err(message) => return error_reply(request_id, ErrorCode::InvalidArgument, message),
        };

        let Some(segment) = document.video.segments.first() else {
            return error_reply(
                request_id,
                ErrorCode::Internal,
                "the director produced a document with no segment",
            );
        };
        let (start_ticks, end_ticks) = (segment.in_ticks, segment.out_ticks);
        let decisions = document
            .rationale
            .as_ref()
            .map(|rationale| rationale.decisions.clone())
            .unwrap_or_default();
        let Ok(document_json) = serde_json::to_string(&document) else {
            return error_reply(
                request_id,
                ErrorCode::Internal,
                "the directed document did not serialize",
            );
        };

        let now = match unix_millis() {
            Ok(now) => now,
            Err(message) => return error_reply(request_id, ErrorCode::Internal, message),
        };
        let encoded = match self
            .database
            .create_edit_doc(
                request_id.clone(),
                request_hash,
                project_id.to_string(),
                document_json,
                now,
            )
            .await
        {
            Ok(encoded) => encoded,
            Err(error) => return store_error_reply(request_id, &error),
        };
        let Ok(doc) = clipmill_contracts::proto::ipc::v1::EditDoc::decode(encoded.as_slice())
        else {
            return error_reply(
                request_id,
                ErrorCode::Internal,
                "the stored document did not decode",
            );
        };
        response_reply(
            request_id,
            response::Body::DirectClip(DirectClipResponse {
                doc: Some(doc),
                start_ticks: u64::try_from(start_ticks).unwrap_or(0),
                end_ticks: u64::try_from(end_ticks).unwrap_or(0),
                decisions,
            }),
        )
    }

    async fn set_clip_decision(
        &self,
        request_id: String,
        decide: &SetClipDecisionRequest,
    ) -> Reply {
        let Ok(project_id) = decide.project_id.parse::<ProjectId>() else {
            return error_reply(request_id, ErrorCode::InvalidArgument, "no project named");
        };
        let Ok(source_id) = decide.source_id.parse::<SourceId>() else {
            return error_reply(request_id, ErrorCode::InvalidArgument, "no source named");
        };
        if decide.candidate_id.is_empty() {
            return error_reply(request_id, ErrorCode::InvalidArgument, "no candidate named");
        }
        let decision = match ClipDecisionV1::try_from(decide.decision)
            .unwrap_or(ClipDecisionV1::Unspecified)
        {
            ClipDecisionV1::Rejected => Decision::Rejected,
            ClipDecisionV1::Kept => Decision::Kept,
            ClipDecisionV1::Approved => Decision::Approved,
            // Refused rather than defaulted: "I did not say" is not one of the
            // three things a person can decide about a clip.
            ClipDecisionV1::Unspecified => {
                return error_reply(
                    request_id,
                    ErrorCode::InvalidArgument,
                    "a decision must be one of rejected, kept, or approved",
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
        let now = match unix_millis() {
            Ok(now) => now,
            Err(message) => return error_reply(request_id, ErrorCode::Internal, message),
        };
        if let Err(error) = self
            .database
            .set_clip_decision(
                project_id.to_string(),
                source_id.to_string(),
                decide.candidate_id.clone(),
                decision,
                now,
            )
            .await
        {
            return store_error_reply(request_id, &error);
        }
        response_reply(
            request_id,
            response::Body::SetClipDecision(SetClipDecisionResponse {
                decision: decide.decision,
                decided_unix_millis: now,
            }),
        )
    }

    async fn list_clip_decisions(
        &self,
        request_id: String,
        list: &ListClipDecisionsRequest,
    ) -> Reply {
        let Ok(project_id) = list.project_id.parse::<ProjectId>() else {
            return error_reply(request_id, ErrorCode::InvalidArgument, "no project named");
        };
        let Ok(source_id) = list.source_id.parse::<SourceId>() else {
            return error_reply(request_id, ErrorCode::InvalidArgument, "no source named");
        };
        match self
            .database
            .list_clip_decisions(project_id.to_string(), source_id.to_string())
            .await
        {
            Ok(records) => response_reply(
                request_id,
                response::Body::ListClipDecisions(ListClipDecisionsResponse {
                    decisions: records
                        .into_iter()
                        .map(|record| ClipDecisionRecordV1 {
                            candidate_id: record.candidate_id,
                            decision: i32::from(match record.decision {
                                Decision::Rejected => 1_u8,
                                Decision::Kept => 2,
                                Decision::Approved => 3,
                            }),
                            decided_unix_millis: record.decided_unix_millis,
                        })
                        .collect(),
                }),
            ),
            Err(error) => store_error_reply(request_id, &error),
        }
    }

    async fn solve_crop_path(&self, request_id: String, solve: &SolveCropPathRequest) -> Reply {
        let Ok(project_id) = solve.project_id.parse::<ProjectId>() else {
            return error_reply(
                request_id,
                ErrorCode::InvalidArgument,
                "solve names no project",
            );
        };
        let Ok(artifact_id) = solve
            .face_track_artifact_id
            .parse::<clipmill_core::ArtifactId>()
        else {
            return error_reply(
                request_id,
                ErrorCode::InvalidArgument,
                "solve names no face track address",
            );
        };
        let Some(artifacts) = self.artifacts.as_ref() else {
            return error_reply(
                request_id,
                ErrorCode::Unavailable,
                "this daemon serves no artifact store",
            );
        };
        // Not found rather than denied: a project that learned "that exists,
        // but not for you" would learn something about another project.
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
        if lease.kind() != "vision.face_track.v1" {
            return error_reply(
                request_id,
                ErrorCode::InvalidArgument,
                format!("{} is not a face track", lease.kind()),
            );
        }
        let document: clipmill_contracts::schemas::vision_face_track::VisionFaceTrack =
            match crate::media::read_artifact_document(&lease, "faces.json") {
                Ok(value) => value,
                Err(error) => {
                    tracing::warn!(?error, "a published face track failed verification");
                    return error_reply(
                        request_id,
                        ErrorCode::Internal,
                        "the face track does not match its manifest",
                    );
                }
            };

        match clipmill_reframe::solve(
            &document,
            solve.start_ticks,
            solve.end_ticks,
            solve.aspect_width,
            solve.aspect_height,
            crop_weights(solve.weights.as_ref()),
            FocusGate::default(),
        ) {
            Ok(solved) => response_reply(
                request_id,
                response::Body::SolveCropPath(crop_response(&solved)),
            ),
            Err(error) => error_reply(request_id, ErrorCode::InvalidArgument, error.to_string()),
        }
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

    /// What the editor's player must draw.
    ///
    /// Interpreted here rather than in the renderer process, by the same code
    /// the renderer uses. The document is read from the store at whatever
    /// revision it is at, and that revision travels back with the plan so a
    /// caller holding a stale document can tell it is looking at a stale
    /// picture rather than discovering it on a frame.
    async fn get_preview_plan(&self, request_id: String, request: &GetPreviewPlanRequest) -> Reply {
        let Ok(project_id) = request.project_id.parse::<ProjectId>() else {
            return error_reply(request_id, ErrorCode::InvalidArgument, "no project named");
        };
        let Ok(doc_id) = request.doc_id.parse::<EditDocId>() else {
            return error_reply(request_id, ErrorCode::InvalidArgument, "no document named");
        };
        let record = match self.database.get_edit_doc(doc_id.to_string()).await {
            Ok(record) => record,
            Err(error) => return store_error_reply(request_id, &error),
        };
        // Not found rather than denied: a project that learned "that exists,
        // but not for you" would learn something about another project.
        if record.project_id != project_id.as_str() {
            return error_reply(
                request_id,
                ErrorCode::NotFound,
                "this project has no such document",
            );
        }
        let Ok(document) =
            serde_json::from_str::<clipmill_edit_ir::EditDocument>(&record.document_json)
        else {
            return error_reply(
                request_id,
                ErrorCode::Internal,
                "the stored document did not parse",
            );
        };
        match clipmill_render::preview_plan(&document, &clipmill_render::RenderProfile::default()) {
            Ok(plan) => response_reply(
                request_id,
                response::Body::GetPreviewPlan(preview_response(record.revision, &plan)),
            ),
            Err(error) => error_reply(request_id, ErrorCode::InvalidArgument, error.to_string()),
        }
    }

    /// Every document a project holds, oldest first.
    ///
    /// The editor opens the newest. Listing exists so opening it in a later
    /// session finds the work rather than an empty screen.
    async fn list_edit_docs(&self, request_id: String, project_id: &str) -> Reply {
        let Ok(project_id) = project_id.parse::<ProjectId>() else {
            return error_reply(request_id, ErrorCode::InvalidArgument, "no project named");
        };
        match self.database.list_edit_docs(project_id.to_string()).await {
            Ok(records) => response_reply(
                request_id,
                response::Body::ListEditDocs(ListEditDocsResponse {
                    docs: records.into_iter().map(Into::into).collect(),
                }),
            ),
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
/// A weight a caller left at zero means "use the default", which is what the
/// contract says and what a caller with no opinion should be able to send. A
/// negative one falls back the same way, because a negative damping term makes
/// the objective unbounded rather than merely unusual.
fn positive_or(asked: f64, default: f64) -> f64 {
    if asked.is_finite() && asked > 0.0 {
        asked
    } else {
        default
    }
}

fn crop_weights(asked: Option<&CropWeightsV1>) -> Weights {
    let default = Weights::default();
    let Some(asked) = asked else { return default };
    Weights {
        subject: positive_or(asked.subject, default.subject),
        velocity: positive_or(asked.velocity, default.velocity),
        acceleration: positive_or(asked.acceleration, default.acceleration),
        zoom: positive_or(asked.zoom, default.zoom),
        max_speed_per_second: positive_or(asked.max_speed_per_second, default.max_speed_per_second),
    }
}

fn crop_response(solved: &clipmill_reframe::CropPath) -> SolveCropPathResponse {
    SolveCropPathResponse {
        keyframes: solved
            .keyframes
            .iter()
            .map(|frame| CropKeyframeV1 {
                t_ticks: frame.t_ticks,
                center_x: frame.center_x,
                center_y: frame.center_y,
                scale: frame.scale,
            })
            .collect(),
        fit: solved.fit,
        // Always present with `fit`, because "why is this not tracking" is the
        // first thing anybody asks.
        fit_reason: solved
            .fit_reason
            .map(clipmill_reframe::FitReason::as_str)
            .unwrap_or_default()
            .to_owned(),
        track_id: u32::try_from(solved.track_id.unwrap_or(0)).unwrap_or(0),
        has_track: solved.track_id.is_some(),
        containment: solved.containment,
    }
}

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
        Some(request::Body::SolveCropPath(_)) => "solve_crop_path",
        Some(request::Body::DirectClip(_)) => "direct_clip",
        Some(request::Body::SetClipDecision(_)) => "set_clip_decision",
        Some(request::Body::ListClipDecisions(_)) => "list_clip_decisions",
        Some(request::Body::GetPreviewPlan(_)) => "get_preview_plan",
        Some(request::Body::ListEditDocs(_)) => "list_edit_docs",
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

/// The plan as the wire carries it.
///
/// A flat list of rectangles, one per frame, rather than the keyframes they
/// were interpolated from. Sending keyframes would make the player
/// interpolate, and interpolating is exactly where the two sides would have to
/// agree about rounding — which is the agreement that cannot be assumed.
fn preview_response(revision: u64, plan: &clipmill_render::PreviewPlan) -> GetPreviewPlanResponse {
    GetPreviewPlanResponse {
        revision,
        rate_num: u32::try_from(plan.rate.num).unwrap_or(30_000),
        rate_den: u32::try_from(plan.rate.den).unwrap_or(1_001),
        frame_count: plan.frame_count,
        crops: plan
            .crops
            .iter()
            .map(|crop| match crop {
                Some(rect) => PreviewCropV1 {
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height,
                    present: true,
                },
                None => PreviewCropV1 {
                    present: false,
                    ..PreviewCropV1::default()
                },
            })
            .collect(),
        cues: plan
            .cues
            .iter()
            .map(|cue| PreviewCueV1 {
                cue_id: cue.cue_id.clone(),
                first_frame: cue.first_frame,
                end_frame: cue.end_frame,
                region: match cue.region {
                    clipmill_edit_ir::CaptionRegion::LowerSafe => "lower_safe",
                    clipmill_edit_ir::CaptionRegion::UpperSafe => "upper_safe",
                    clipmill_edit_ir::CaptionRegion::Center => "center",
                }
                .to_owned(),
                karaoke: cue.karaoke,
                lead_in_centis: cue.lead_in_centis,
                lines: cue
                    .lines
                    .iter()
                    .map(|line| PreviewLineV1 {
                        words: line
                            .words
                            .iter()
                            .map(|word| PreviewWordV1 {
                                text: word.text.clone(),
                                hold_centis: word.hold_centis,
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect(),
        gain: plan
            .gain
            .iter()
            .map(|point| PreviewGainV1 {
                frame: point.frame,
                gain_db: point.gain_db,
            })
            .collect(),
        width: plan.width,
        height: plan.height,
    }
}

/// Build the document for a directed clip.
///
/// Split out of the handler so what remains there is the refusal ladder: a
/// reader can see every way the request is turned down before anything is
/// assembled, which is the part that decides whether a bad request reaches the
/// store.
fn assemble(
    evidence: &crate::inspector::Evidence,
    direct: &DirectClipRequest,
) -> Result<clipmill_edit_ir::EditDocument, String> {
    let style_ref = if direct.style_ref.is_empty() {
        clipmill_captions::DEFAULT_STYLE_REF.to_owned()
    } else {
        direct.style_ref.clone()
    };
    let cut = match ClipCutV1::try_from(direct.cut).unwrap_or(ClipCutV1::Unspecified) {
        ClipCutV1::Alternative => clipmill_director::Cut::Alternative,
        // An exact pair is snapped before anything is built from it: a boundary
        // arriving over this socket has been through a process the director does
        // not control, and the lattice is what legal means.
        ClipCutV1::Exact => clipmill_director::Cut::Exact(snapped(evidence, direct)?),
        ClipCutV1::Chosen | ClipCutV1::Unspecified => clipmill_director::Cut::Chosen,
    };
    clipmill_director::direct(
        clipmill_director::Evidence {
            candidates: &evidence.candidates,
            ranking: &evidence.ranking,
            transcript: &evidence.transcript,
            index: evidence.index.as_ref(),
            shots: evidence.shots.as_ref(),
            faces: evidence.faces.as_ref(),
        },
        &clipmill_director::Request {
            candidate_id: direct.candidate_id.clone(),
            cut,
            style_ref,
            frame: evidence.frame,
            aspect: clipmill_director::Aspect::default(),
        },
    )
    .map_err(|error| error.to_string())
}

/// A hand-set boundary, put on the candidate's lattice.
fn snapped(
    evidence: &crate::inspector::Evidence,
    direct: &DirectClipRequest,
) -> Result<clipmill_director::Boundary, String> {
    let candidate = evidence
        .candidates
        .candidates
        .iter()
        .find(|item| item.id.as_str() == direct.candidate_id)
        .ok_or_else(|| format!("no candidate is called {}", direct.candidate_id))?;
    let mut starts: Vec<i64> = candidate
        .boundary_lattice
        .starts
        .iter()
        .map(|at| i64::try_from(*at).unwrap_or(i64::MAX))
        .collect();
    let mut ends: Vec<i64> = candidate
        .boundary_lattice
        .ends
        .iter()
        .map(|at| i64::try_from(*at).unwrap_or(i64::MAX))
        .collect();
    starts.sort_unstable();
    ends.sort_unstable();
    clipmill_director::snap(
        clipmill_director::Lattice {
            starts: &starts,
            ends: &ends,
        },
        clipmill_director::Boundary {
            start_ticks: i64::try_from(direct.start_ticks).unwrap_or(0),
            end_ticks: i64::try_from(direct.end_ticks).unwrap_or(0),
        },
        clipmill_director::Duration {
            min_ticks: i64::try_from(evidence.candidates.duration_target.min_ticks.get())
                .unwrap_or(0),
            max_ticks: i64::try_from(evidence.candidates.duration_target.max_ticks.get())
                .unwrap_or(i64::MAX),
        },
        // The start is the edge a person is answering "where should this begin"
        // with, so it is the one held.
        clipmill_director::Edge::Start,
    )
    .map_err(|error| error.to_string())
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
