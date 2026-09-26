mod batch;
mod models;
mod storage;
mod youtube;
mod youtube_publish;

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
    DeliverExportPayloadV1, DemoDagPayloadV1, DeriveCaptionsPayloadV1, DetectFacesPayloadV1,
    DetectShotsPayloadV1, DirectClipRequest, DiscoverCandidatesPayloadV1, Error, ErrorCode,
    ExportArchiveRequest, ExportArchiveResponse, ExportClipPayloadV1, ExportClipRequest,
    ExportClipResponse, ExportFindingV1, ExportRequestV1, ExportSeverity, ExportValidationV1,
    FaceSightingV1, GetDeviceProfileRequest, GetDeviceProfileResponse, GetEditDocResponse,
    GetJobResponse, GetLocalLockResponse, GetPreviewPlanRequest, GetPreviewPlanResponse,
    GetProjectResponse, GetReadinessResponse, GetSourceResponse, HealthResponse,
    IndexTranscriptPayloadV1, IngestSourcePayloadV1, ListClipDecisionsRequest,
    ListClipDecisionsResponse, ListEditDocsResponse, ListFacesRequest, ListFacesResponse,
    ListJobsResponse, ListProjectsResponse, ListSourcesResponse, LocalLockStatusV1, MediaFileV1,
    PingResponse, PlanExportRequest, PlanExportResponse, PreviewCropV1, PreviewCueV1,
    PreviewGainV1, PreviewLineV1, PreviewProxyV1, PreviewSegmentV1, PreviewSourceV1, PreviewWordV1,
    ProbeSourcePayloadV1, RankCandidatesPayloadV1, ReadArtifactRequest, ReadArtifactResponse,
    RegisterSourceRequest, RenderClipPayloadV1, Request, ResolveMediaRequest, ResolveMediaResponse,
    Response, SetClipDecisionRequest, SetClipDecisionResponse, SnapshotEditDocResponse,
    SolveCropPathRequest, SolveCropPathResponse, StageReadinessV1, SubmitJobRequest,
    SubscribeTaskEventsRequest, SubscribeTaskEventsResponse, TranscribeSourcePayloadV1,
    WorkerPresenceV1, request, response,
};
use clipmill_contracts::schemas::vision_face_track::VisionFaceTrack;
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
const EXPORT_CLIP_KEY_VERSION: &str = "clipmill.export-clip.v1";
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
    /// How long an unreferenced artifact is kept. Settings reports this value
    /// but does not expose controls for changing collection policy.
    retention_grace: std::time::Duration,
    /// The Local Lock, which Health and Settings both read rather than assert.
    policy: std::sync::Arc<crate::policy::LocalLockPolicy>,
    /// Who is connected to the worker plane right now, for readiness.
    roster: crate::worker::WorkerRoster,
    /// The pinned decoder every media stage runs, for the same question.
    decoder: Option<std::path::PathBuf>,
    /// Where the pinned caption fonts are, to say which this installation has.
    fonts_dir: Option<std::path::PathBuf>,
    batch_admission: std::sync::Arc<tokio::sync::Mutex<()>>,
    youtube: Option<std::sync::Arc<youtube::YoutubeRuntime>>,
    publishing: std::sync::Arc<youtube_publish::PublishingRuntime>,
    /// Which models are installed, which one does each job, and getting them.
    /// Absent in the tests that build a service without a workspace.
    library: Option<std::sync::Arc<crate::library::ModelLibrary>>,
    /// The loop every artifact collection runs through, for the clean-ups
    /// Settings asks for. Absent where no daemon runs that loop.
    collector: Option<crate::collector::Collector>,
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
            retention_grace: std::time::Duration::ZERO,
            policy: std::sync::Arc::default(),
            roster: crate::worker::new_roster(),
            decoder: None,
            fonts_dir: None,
            batch_admission: std::sync::Arc::default(),
            youtube: None,
            publishing: std::sync::Arc::default(),
            library: None,
            collector: None,
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
        retention_grace: std::time::Duration,
        policy: std::sync::Arc<crate::policy::LocalLockPolicy>,
        roster: crate::worker::WorkerRoster,
        decoder: std::path::PathBuf,
    ) -> Self {
        let helper = std::env::var_os("CLIPMILL_YOUTUBE_IMPORTER").map_or_else(
            || {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../tools/import-youtube.sh")
            },
            std::path::PathBuf::from,
        );
        let youtube = Some(std::sync::Arc::new(youtube::YoutubeRuntime::new(
            crate::youtube_transport::YoutubeDownloader::new(helper, decoder.clone()),
            storage.data.join("imports"),
        )));
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
            retention_grace,
            policy,
            roster,
            decoder: Some(decoder),
            fonts_dir: None,
            batch_admission: std::sync::Arc::default(),
            youtube,
            publishing: std::sync::Arc::default(),
            library: None,
            collector: None,
        }
    }

    /// Tell the service where the pinned caption fonts are.
    pub(crate) fn with_fonts(mut self, fonts_dir: std::path::PathBuf) -> Self {
        self.fonts_dir = Some(fonts_dir);
        self
    }

    /// Every caption typeface, with whether its pinned file is installed.
    ///
    /// The editor offers only the installed ones: a face whose file is not
    /// here could be chosen and then refused by the render.
    fn caption_fonts(&self) -> Vec<clipmill_contracts::proto::ipc::v1::CaptionFontV1> {
        clipmill_captions::FONTS
            .iter()
            .map(|face| clipmill_contracts::proto::ipc::v1::CaptionFontV1 {
                family: face.family.to_owned(),
                label: face.label.to_owned(),
                file: face.file.to_owned(),
                installed: self
                    .fonts_dir
                    .as_ref()
                    .is_some_and(|dir| dir.join(face.file).is_file()),
            })
            .collect()
    }

    /// One derivative ingest produced for a source, with the source's
    /// fingerprint.
    ///
    /// Resolved through the ingest manifest rather than by searching the
    /// store, because the manifest is what roots the derivatives and its
    /// children are what garbage collection keeps reachable. Anything found
    /// another way might be an object nobody is holding on to.
    ///
    /// The manifest is found by the stage that published it, whatever job ran
    /// it: ingest lives inside the `analyze-source` DAG now, and a lookup by
    /// the old standalone job kind answered "never ingested" for every source.
    async fn ingested_derivative(&self, source_id: &str, kind: &str) -> Option<(String, String)> {
        let artifacts = self.artifacts.as_ref()?;
        let manifest_id = self
            .database
            .latest_source_task_artifact(
                source_id.to_owned(),
                crate::media::KIND_MANIFEST.to_owned(),
            )
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
                    // Derived from the stage registry and what has actually
                    // started, never asserted. See `policy.rs`.
                    local_lock: self.policy.status().engaged,
                }),
            ),
            request::Body::CreateProject(create) => {
                self.create_project(request_id, request_hash, &create).await
            }
            request::Body::RenameProject(rename) => {
                self.rename_project(request_id, request_hash, &rename.project_id, &rename.name)
                    .await
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
            request::Body::PreviewCaptions(preview) => {
                self.preview_captions(request_id, &preview).await
            }
            request::Body::ListEditHistory(list) => {
                self.list_edit_history(request_id, &list.doc_id).await
            }
            request::Body::ListFaces(list) => self.list_faces(request_id, &list).await,
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
            request::Body::PlanExport(plan) => self.plan_export(request_id, &plan).await,
            request::Body::ExportClip(export) => {
                self.export_clip(request_id, request_hash, &export).await
            }
            request::Body::ExportArchive(archive) => {
                self.export_archive(request_id, &archive).await
            }
            request::Body::GetLocalLock(_) => self.get_local_lock(request_id),
            request::Body::GetReadiness(_) => self.get_readiness(request_id),
            request::Body::SubmitExportBatch(batch) => {
                self.submit_export_batch(request_id, request_hash, batch)
                    .await
            }
            request::Body::ListExportBatches(_) => self.list_export_batches(request_id).await,
            request::Body::UpdateExportBatchItem(update) => {
                self.update_export_batch_item(request_id, request_hash, update)
                    .await
            }
            request::Body::ConfigureYoutubePublishing(asked) => {
                self.configure_youtube_publishing(request_id, asked).await
            }
            request::Body::ConnectYoutubeChannel(_) => {
                self.connect_youtube_channel(request_id).await
            }
            request::Body::GetYoutubePublishingStatus(_) => {
                self.youtube_publishing_status(request_id).await
            }
            request::Body::UpdateYoutubeConnection(asked) => {
                self.update_youtube_connection(request_id, asked).await
            }
            request::Body::StartYoutubeUpload(asked) => {
                self.start_youtube_upload(request_id, request_hash, asked)
                    .await
            }
            request::Body::GetYoutubeUpload(asked) => {
                self.get_youtube_upload(request_id, asked.upload_id).await
            }
            request::Body::ListYoutubeUploads(asked) => {
                self.list_youtube_uploads(request_id, asked.project_id)
                    .await
            }
            request::Body::UpdateYoutubeUpload(asked) => {
                self.update_youtube_upload(request_id, request_hash, asked)
                    .await
            }
            request::Body::PublishYoutubeUpload(asked) => {
                self.publish_youtube_upload(request_id, request_hash, asked.upload_id)
                    .await
            }
            request::Body::DraftYoutubeMetadata(asked) => {
                self.draft_youtube_metadata(request_id, asked).await
            }
            request::Body::StartYoutubeImport(asked) => {
                self.start_youtube_import(request_id, request_hash, asked)
                    .await
            }
            request::Body::GetYoutubeImport(asked) => {
                self.get_youtube_import(request_id, asked.import_id).await
            }
            request::Body::ListYoutubeImports(asked) => {
                self.list_youtube_imports(request_id, asked.project_id)
                    .await
            }
            request::Body::UpdateYoutubeImport(asked) => {
                self.update_youtube_import(request_id, request_hash, asked)
                    .await
            }
            request::Body::ListModels(_) => self.list_models(request_id),
            request::Body::DownloadModels(asked) => self.download_models(request_id, &asked),
            request::Body::CancelModelDownload(asked) => {
                self.cancel_model_download(request_id, &asked)
            }
            request::Body::RemoveModel(asked) => self.remove_model(request_id, &asked).await,
            request::Body::VerifyModel(asked) => self.verify_model(request_id, &asked),
            request::Body::SetModelChoice(asked) => self.set_model_choice(request_id, &asked),
            request::Body::InspectHubModel(asked) => {
                self.inspect_hub_model(request_id, &asked).await
            }
            request::Body::AddCustomModel(asked) => self.add_custom_model(request_id, &asked).await,
            request::Body::ForgetModel(asked) => self.forget_model(request_id, &asked).await,
            request::Body::CleanStorage(asked) => self.clean_storage(request_id, &asked).await,
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

    async fn rename_project(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        value: &str,
        name: &str,
    ) -> Reply {
        let project_id = match value.parse::<ProjectId>() {
            Ok(project_id) => project_id,
            Err(error) => {
                return error_reply(request_id, ErrorCode::InvalidArgument, error.to_string());
            }
        };
        let name = match validate_project_name(name) {
            Ok(name) => name,
            Err(message) => return error_reply(request_id, ErrorCode::InvalidArgument, message),
        };
        let now = match unix_millis() {
            Ok(now) => now,
            Err(message) => return error_reply(request_id, ErrorCode::Internal, message),
        };
        match self
            .database
            .rename_project(
                request_id.clone(),
                request_hash,
                project_id.to_string(),
                name,
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
            Ok(bytes) => {
                self.pause_youtube_project(&project_id.to_string()).await;
                self.cleanup_youtube_project(&project_id.to_string()).await;
                Reply {
                    bytes,
                    outcome: Outcome::Success,
                }
            }
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
                let bindings = self.planning_bindings();
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
                let bindings = self.planning_bindings();
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
    ///
    /// Directing a clip that already has a document hands that document back
    /// as it stands — the trims and corrections somebody made to it are why it
    /// is the answer — and the reply says so. That is asked of the store
    /// *before* any evidence is read: a saved edit is reopenable whether or
    /// not the analysis it was cut from can still be loaded, and a re-analysis
    /// that renumbered the candidates must not lock a person out of the edit
    /// they made. Only a clip with no document, or a variation asked for by
    /// name, needs the director — and it reads the run the request names, as
    /// one snapshot. An approval, when requested, lands in the same
    /// transaction as the document either way, so "approved but nothing to
    /// open" is not a state the store can be left in.
    #[allow(
        clippy::too_many_lines,
        reason = "explicit selection admission and retry recovery precede one document transaction"
    )]
    async fn direct_clip(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        direct: &DirectClipRequest,
    ) -> Reply {
        let mut normalized = direct.clone();
        if direct.manual_span {
            if direct.job_id.is_empty() || direct.start_ticks >= direct.end_ticks {
                return error_reply(
                    request_id,
                    ErrorCode::InvalidArgument,
                    "Choose an analysis run and a nonempty source interval",
                );
            }
            let identity = format!(
                "{}:{}:{}:{}",
                direct.source_id, direct.job_id, direct.start_ticks, direct.end_ticks
            );
            normalized.candidate_id = format!(
                "cand_{}",
                &hex::encode(Sha256::digest(identity.as_bytes()))[..16]
            );
            normalized.approve = false;
        }
        let direct = &normalized;
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
        if direct.candidate_id.is_empty() {
            return error_reply(request_id, ErrorCode::InvalidArgument, "no candidate named");
        }
        let now = match unix_millis() {
            Ok(now) => now,
            Err(message) => return error_reply(request_id, ErrorCode::Internal, message),
        };
        let identity = crate::db::ClipIdentity {
            project: project_id.to_string(),
            source: source_id.to_string(),
            candidate: direct.candidate_id.clone(),
            run: (!direct.job_id.is_empty()).then(|| direct.job_id.clone()),
        };

        if !direct.variation {
            match self
                .database
                .reopen_edit_doc(
                    request_id.clone(),
                    request_hash,
                    identity.clone(),
                    direct.approve,
                    now,
                )
                .await
            {
                Ok(Some(bytes)) => {
                    return Reply {
                        bytes,
                        outcome: Outcome::Success,
                    };
                }
                Ok(None) => {}
                Err(error) => return store_error_reply(request_id, &error),
            }
        }

        let document_json = match self
            .assemble_clip(artifacts, &source.source_map_json, identity.clone(), direct)
            .await
        {
            Ok(json) => json,
            Err((code, message)) => return error_reply(request_id, code, message),
        };
        match self
            .database
            .direct_edit_doc(
                request_id.clone(),
                request_hash,
                identity,
                document_json,
                crate::db::DirectOptions {
                    variation: direct.variation,
                    approve: direct.approve,
                },
                now,
            )
            .await
        {
            // The store encoded the whole reply — the document, where its
            // segment stands, and whether it was reopened — because for a
            // reopened document those answers come from the stored copy, not
            // from the one assembled above.
            Ok(bytes) => Reply {
                bytes,
                outcome: Outcome::Success,
            },
            Err(error) => store_error_reply(request_id, &error),
        }
    }

    /// Read the clip's run and build the document the director proposes.
    async fn assemble_clip(
        &self,
        artifacts: &ArtifactHandle,
        source_map_json: &[u8],
        identity: crate::db::ClipIdentity,
        direct: &DirectClipRequest,
    ) -> Result<String, (ErrorCode, String)> {
        let evidence = crate::inspector::load(
            &self.database,
            artifacts,
            &identity.source,
            source_map_json,
            identity.run.as_deref(),
        )
        .await
        .map_err(|error| (ErrorCode::Conflict, error.message()))?;
        let mut document =
            assemble(&evidence, direct).map_err(|message| (ErrorCode::InvalidArgument, message))?;
        document.captions.options.highlight_spoken_word = direct.highlight_spoken_word;
        if document.video.segments.is_empty() {
            return Err((
                ErrorCode::Internal,
                "the director produced a document with no segment".to_owned(),
            ));
        }
        serde_json::to_string(&document).map_err(|_| {
            (
                ErrorCode::Internal,
                "the directed document did not serialize".to_owned(),
            )
        })
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
        let decision = match ClipDecisionV1::try_from(decide.decision) {
            Ok(ClipDecisionV1::Rejected) => Some(Decision::Rejected),
            Ok(ClipDecisionV1::Kept) => Some(Decision::Kept),
            Ok(ClipDecisionV1::Approved) => Some(Decision::Approved),
            // No decision is what a person answers by taking theirs back: the
            // clip is undecided again. An edit an approval made is left alone —
            // undoing a verdict is not deleting somebody's work.
            Ok(ClipDecisionV1::Unspecified) => None,
            Err(_) => {
                return error_reply(
                    request_id,
                    ErrorCode::InvalidArgument,
                    "a decision must be rejected, kept, approved, or none",
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
        let stored = match decision {
            Some(decision) => {
                self.database
                    .set_clip_decision(
                        project_id.to_string(),
                        source_id.to_string(),
                        decide.candidate_id.clone(),
                        decision,
                        now,
                    )
                    .await
            }
            None => {
                self.database
                    .clear_clip_decision(
                        project_id.to_string(),
                        source_id.to_string(),
                        decide.candidate_id.clone(),
                    )
                    .await
            }
        };
        if let Err(error) = stored {
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

    #[allow(
        clippy::too_many_lines,
        reason = "source geometry and matching evidence must be resolved before solving"
    )]
    /// A face track this project published, verified and parsed, with the
    /// display frame of the source it was measured on.
    async fn face_track(
        &self,
        request_id: &str,
        project: &str,
        artifact: &str,
    ) -> Result<(VisionFaceTrack, (u32, u32)), Reply> {
        let refuse = |code, message: &str| Err(error_reply(request_id.to_owned(), code, message));
        let Ok(project_id) = project.parse::<ProjectId>() else {
            return refuse(ErrorCode::InvalidArgument, "the request names no project");
        };
        let Ok(artifact_id) = artifact.parse::<clipmill_core::ArtifactId>() else {
            return refuse(
                ErrorCode::InvalidArgument,
                "the request names no face track address",
            );
        };
        let Some(artifacts) = self.artifacts.as_ref() else {
            return refuse(
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
                return refuse(
                    ErrorCode::NotFound,
                    "this project published no such artifact",
                );
            }
            Err(error) => return Err(store_error_reply(request_id.to_owned(), &error)),
        }
        let Ok(lease) = artifacts.open(artifact_id).await else {
            return refuse(ErrorCode::NotFound, "the artifact is not in this store");
        };
        if lease.kind() != "vision.face_track.v1" {
            return Err(error_reply(
                request_id.to_owned(),
                ErrorCode::InvalidArgument,
                format!("{} is not a face track", lease.kind()),
            ));
        }
        let document: VisionFaceTrack =
            match crate::media::read_artifact_document(&lease, "faces.json") {
                Ok(value) => value,
                Err(error) => {
                    tracing::warn!(?error, "a published face track failed verification");
                    return refuse(
                        ErrorCode::Internal,
                        "the face track does not match its manifest",
                    );
                }
            };

        let registered = match self.database.list_sources(project_id.to_string()).await {
            Ok(sources) => sources,
            Err(error) => return Err(store_error_reply(request_id.to_owned(), &error)),
        };
        let frame = registered
            .iter()
            .find(|source| source.source_fingerprint == document.source_fingerprint.as_str())
            .and_then(|source| crate::inspector::frame_of(&source.source_map_json));
        let Some(frame) = frame else {
            return refuse(
                ErrorCode::InvalidArgument,
                "the face tracks have no registered source display dimensions",
            );
        };
        let (Ok(width), Ok(height)) = (u32::try_from(frame.width), u32::try_from(frame.height))
        else {
            return refuse(
                ErrorCode::InvalidArgument,
                "source display dimensions are too large",
            );
        };
        Ok((document, (width, height)))
    }

    async fn solve_crop_path(&self, request_id: String, solve: &SolveCropPathRequest) -> Reply {
        let (document, frame) = match self
            .face_track(
                &request_id,
                &solve.project_id,
                &solve.face_track_artifact_id,
            )
            .await
        {
            Ok(found) => found,
            Err(reply) => return reply,
        };
        match crop_solve(document, solve, frame) {
            Ok(solved) => response_reply(request_id, response::Body::SolveCropPath(solved)),
            Err(error) => error_reply(request_id, ErrorCode::InvalidArgument, error.to_string()),
        }
    }

    /// The faces seen over a span, so a person can point at the one to follow.
    async fn list_faces(&self, request_id: String, list: &ListFacesRequest) -> Reply {
        if list.end_ticks <= list.start_ticks
            || list.end_ticks - list.start_ticks > MAX_FACE_SPAN_TICKS
        {
            return error_reply(
                request_id,
                ErrorCode::InvalidArgument,
                "ask for the faces of one section, up to ten minutes of it",
            );
        }
        let (document, _) = match self
            .face_track(&request_id, &list.project_id, &list.face_track_artifact_id)
            .await
        {
            Ok(found) => found,
            Err(reply) => return reply,
        };
        let sightings = document
            .tracks
            .iter()
            .flat_map(|track| {
                track
                    .boxes
                    .iter()
                    .filter(|seen| {
                        seen.t_ticks >= list.start_ticks && seen.t_ticks < list.end_ticks
                    })
                    .map(|seen| FaceSightingV1 {
                        track_id: u32::try_from(track.track_id).unwrap_or(u32::MAX),
                        t_ticks: seen.t_ticks,
                        x: seen.x,
                        y: seen.y,
                        width: seen.w,
                        height: seen.h,
                    })
            })
            .collect();
        response_reply(
            request_id,
            response::Body::ListFaces(ListFacesResponse { sightings }),
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
        // Parse that inventory once: a filmstrip may contain thousands of tiles.
        let Ok(declared_sizes) = lease.declared_file_sizes() else {
            return error_reply(
                request_id,
                ErrorCode::Internal,
                "the media manifest cannot be read",
            );
        };
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
            let Some(bytes) = declared_sizes.get(&path).copied() else {
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
        let name = |value: &str| (!value.is_empty()).then(|| value.to_owned());
        match self
            .database
            .create_edit_doc(
                request_id.clone(),
                request_hash,
                project_id.to_string(),
                create.document_json.clone(),
                crate::db::DocumentOrigin {
                    source: name(&create.source_id),
                    candidate: name(&create.candidate_id),
                    run: name(&create.job_id),
                },
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
        // Two requests are resolved here into commands the log can replay
        // without the evidence they were derived from.
        let op = serde_json::from_str::<serde_json::Value>(&apply.command_json)
            .ok()
            .and_then(|value| {
                value
                    .get("op")
                    .and_then(|op| op.as_str())
                    .map(str::to_owned)
            });
        let resolved = match op.as_deref() {
            Some("extend_with_captions") => {
                Some(self.prepare_extension(&doc_id.to_string(), apply).await)
            }
            Some("refresh_captions") => {
                Some(self.prepare_refresh(&doc_id.to_string(), apply).await)
            }
            _ => None,
        };
        let command_json = match resolved {
            Some(Ok(command)) => command,
            Some(Err((code, message))) => return error_reply(request_id, code, message),
            None => apply.command_json.clone(),
        };
        match self
            .database
            .apply_edit_command(
                request_id.clone(),
                request_hash,
                doc_id.to_string(),
                apply.expected_revision,
                command_json,
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

    fn extension_request(json: &str) -> Result<(String, i64, i64), (ErrorCode, String)> {
        let request: serde_json::Value = serde_json::from_str(json).map_err(|_| {
            (
                ErrorCode::InvalidArgument,
                "Invalid extension request".to_owned(),
            )
        })?;
        let string = |key: &str| {
            request
                .get(key)
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        };
        let number = |key: &str| request.get(key).and_then(serde_json::Value::as_i64);
        let segment_id = string("segment_id").ok_or((
            ErrorCode::InvalidArgument,
            "Choose a section to extend".to_owned(),
        ))?;
        let in_ticks = number("in_ticks").ok_or((
            ErrorCode::InvalidArgument,
            "Choose a new in point".to_owned(),
        ))?;
        let out_ticks = number("out_ticks").ok_or((
            ErrorCode::InvalidArgument,
            "Choose a new out point".to_owned(),
        ))?;
        Ok((segment_id, in_ticks, out_ticks))
    }

    fn extension_span(
        document: &clipmill_edit_ir::EditDocument,
        segment_id: &str,
        in_ticks: i64,
        out_ticks: i64,
    ) -> Result<(usize, clipmill_director::Boundary), (ErrorCode, String)> {
        let position = document
            .video
            .segments
            .iter()
            .position(|part| part.segment_id == segment_id)
            .ok_or((ErrorCode::InvalidArgument, "No such section".to_owned()))?;
        let part = &document.video.segments[position];
        let span = if position == 0
            && in_ticks >= 0
            && in_ticks < part.in_ticks
            && out_ticks == part.out_ticks
        {
            clipmill_director::Boundary {
                start_ticks: in_ticks,
                end_ticks: part.in_ticks,
            }
        } else if position + 1 == document.video.segments.len()
            && in_ticks == part.in_ticks
            && out_ticks > part.out_ticks
        {
            clipmill_director::Boundary {
                start_ticks: part.out_ticks,
                end_ticks: out_ticks,
            }
        } else {
            return Err((
                ErrorCode::InvalidArgument,
                "Only the first or last clip edge can be pulled outward".to_owned(),
            ));
        };
        Ok((position, span))
    }

    /// Resolve a source edge gesture into one replayable command. The command
    /// log stores the derived cues, so replay never depends on a later analysis.
    async fn prepare_extension(
        &self,
        doc_id: &str,
        apply: &ApplyEditCommandRequest,
    ) -> Result<String, (ErrorCode, String)> {
        let (segment_id, in_ticks, out_ticks) = Self::extension_request(&apply.command_json)?;
        let record = self
            .database
            .get_edit_doc(doc_id.to_owned())
            .await
            .map_err(|error| (ErrorCode::NotFound, error.to_string()))?;
        if record.revision != apply.expected_revision {
            return Err((
                ErrorCode::Conflict,
                "The edit changed. Reload before extending.".to_owned(),
            ));
        }
        let document =
            clipmill_edit_ir::EditDocument::from_canonical_json(record.document_json.as_bytes())
                .map_err(|error| (ErrorCode::Internal, error.to_string()))?;
        let (position, span) = Self::extension_span(&document, &segment_id, in_ticks, out_ticks)?;
        let part = &document.video.segments[position];
        let source_id = record.source_id.ok_or((
            ErrorCode::InvalidArgument,
            "This edit has no linked source".to_owned(),
        ))?;
        let artifacts = self
            .artifacts
            .as_ref()
            .ok_or((ErrorCode::Unavailable, "No artifact store".to_owned()))?;
        let source = self
            .database
            .get_source(source_id.clone())
            .await
            .map_err(|error| (ErrorCode::NotFound, error.to_string()))?;
        if source.project_id != record.project_id
            || source.source_fingerprint != part.source_fingerprint
        {
            return Err((
                ErrorCode::Conflict,
                "The edit source no longer matches its recording".to_owned(),
            ));
        }
        let evidence = crate::inspector::load(
            &self.database,
            artifacts,
            &source_id,
            &source.source_map_json,
            record.job_id.as_deref(),
        )
        .await
        .map_err(|error| (ErrorCode::Conflict, error.message()))?;
        let captions = clipmill_director::captions_for_span(
            &evidence.transcript,
            evidence.index.as_ref(),
            evidence.shots.as_ref(),
            span,
            &document.captions.style_ref,
        )
        .map_err(|error| (ErrorCode::InvalidArgument, error.to_string()))?;
        let span_ticks = span.end_ticks - span.start_ticks;
        let command = clipmill_edit_ir::EditCommand::ExtendSegment {
            segment_id,
            in_ticks,
            out_ticks,
            reading_cues: fit_extension_cues(
                &document.captions.cues,
                captions.cues,
                span_ticks,
                span.start_ticks,
            ),
            burn_in_cues: fit_extension_cues(
                &document.captions.burn_in,
                captions.burn_in,
                span_ticks,
                span.start_ticks,
            ),
        };
        String::from_utf8(
            command
                .to_canonical_json()
                .map_err(|error| (ErrorCode::Internal, error.to_string()))?,
        )
        .map_err(|error| (ErrorCode::Internal, error.to_string()))
    }

    /// Resolve "refresh the captions" into one replayable command: both
    /// presentations derived afresh from the newest transcript over the
    /// recording, carried whole so the log replays without it, and undone as
    /// one step. The clip-wide look and options stay; corrections made to the
    /// old captions do not, which the Editor says before asking.
    async fn prepare_refresh(
        &self,
        doc_id: &str,
        apply: &ApplyEditCommandRequest,
    ) -> Result<String, (ErrorCode, String)> {
        let record = self
            .database
            .get_edit_doc(doc_id.to_owned())
            .await
            .map_err(|error| (ErrorCode::NotFound, error.to_string()))?;
        if record.revision != apply.expected_revision {
            return Err((
                ErrorCode::Conflict,
                "The edit changed. Reload before refreshing its captions.".to_owned(),
            ));
        }
        let document =
            clipmill_edit_ir::EditDocument::from_canonical_json(record.document_json.as_bytes())
                .map_err(|error| (ErrorCode::Internal, error.to_string()))?;
        let source_id = record.source_id.ok_or((
            ErrorCode::InvalidArgument,
            "This edit has no linked source".to_owned(),
        ))?;
        let artifacts = self
            .artifacts
            .as_ref()
            .ok_or((ErrorCode::Unavailable, "No artifact store".to_owned()))?;
        let source = self
            .database
            .get_source(source_id.clone())
            .await
            .map_err(|error| (ErrorCode::NotFound, error.to_string()))?;
        if source.project_id != record.project_id
            || document
                .video
                .segments
                .iter()
                .any(|part| part.source_fingerprint != source.source_fingerprint)
        {
            return Err((
                ErrorCode::Conflict,
                "The edit source no longer matches its recording".to_owned(),
            ));
        }
        let speech = crate::inspector::load_speech(&self.database, artifacts, &source_id)
            .await
            .map_err(|error| (ErrorCode::Conflict, error.message()))?;
        let (reading, burn_in) = clipmill_director::captions_for_program(
            &speech.transcript,
            speech.index.as_ref(),
            speech.shots.as_ref(),
            &document,
        )
        .map_err(|error| (ErrorCode::InvalidArgument, error.to_string()))?;
        let mut commands = vec![
            clipmill_edit_ir::EditCommand::ReplaceCues {
                cues: reading,
                presentation: clipmill_edit_ir::Presentation::Reading,
            },
            clipmill_edit_ir::EditCommand::ReplaceCues {
                cues: burn_in,
                presentation: clipmill_edit_ir::Presentation::BurnIn,
            },
        ];
        // How many words the clip shows at once is the person's choice, and a
        // refresh keeps it.
        if let Some(max_words) = document.captions.options.words_on_screen {
            commands.push(clipmill_edit_ir::EditCommand::RegroupOnScreen { max_words });
        }
        String::from_utf8(
            clipmill_edit_ir::EditCommand::Batch { commands }
                .to_canonical_json()
                .map_err(|error| (ErrorCode::Internal, error.to_string()))?,
        )
        .map_err(|error| (ErrorCode::Internal, error.to_string()))
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
        let profile = self.preview_profile(project_id.as_str(), &document).await;
        let plan = match clipmill_render::preview_plan(&document, &profile) {
            Ok(plan) => plan,
            Err(error) => {
                return error_reply(request_id, ErrorCode::InvalidArgument, error.to_string());
            }
        };
        let (sources, proxies) = self.preview_media(&project_id, &document).await;
        let mut reply = preview_response(record.revision, &plan);
        reply.sources = sources;
        reply.proxies = proxies;
        reply.fonts = self.caption_fonts();
        response_reply(request_id, response::Body::GetPreviewPlan(reply))
    }

    /// The sources a document draws from and the proxy each is previewed on.
    ///
    /// Resolved here, beside the plan, rather than left to the shell: the
    /// shell used to take whichever proxy the project published last, which
    /// is the wrong recording as soon as a project holds two. A source is
    /// matched by the fingerprint the segment names, the frame the crops are
    /// measured in comes from its probe, and the proxy is the newest one
    /// published over it. A source with no proxy is listed without one; the
    /// player then says there is nothing to play rather than playing
    /// something else.
    async fn preview_media(
        &self,
        project_id: &ProjectId,
        document: &clipmill_edit_ir::EditDocument,
    ) -> (Vec<PreviewSourceV1>, Vec<PreviewProxyV1>) {
        let mut sources = Vec::new();
        let mut proxies = Vec::new();
        let Ok(registered) = self.database.list_sources(project_id.to_string()).await else {
            return (sources, proxies);
        };
        let mut seen: Vec<&str> = Vec::new();
        for segment in &document.video.segments {
            let fingerprint = segment.source_fingerprint.as_str();
            if seen.contains(&fingerprint) {
                continue;
            }
            seen.push(fingerprint);
            let Some(source) = registered
                .iter()
                .find(|source| source.source_fingerprint == fingerprint)
            else {
                continue;
            };
            let frame = crate::inspector::frame_of(&source.source_map_json);
            sources.push(PreviewSourceV1 {
                source_fingerprint: fingerprint.to_owned(),
                source_id: source.source_id.clone(),
                display_width: frame.map_or(0, |frame| frame.width),
                display_height: frame.map_or(0, |frame| frame.height),
            });
            if let Some(proxy) = self.proxy_of(&source.source_id).await {
                proxies.push(PreviewProxyV1 {
                    source_fingerprint: fingerprint.to_owned(),
                    ..proxy
                });
            }
        }
        (sources, proxies)
    }

    /// The newest proxy published over a source, as the player needs it.
    async fn proxy_of(&self, source_id: &str) -> Option<PreviewProxyV1> {
        let artifacts = self.artifacts.as_ref()?;
        let address = self
            .database
            .latest_source_task_artifact(source_id.to_owned(), crate::media::KIND_PROXY.to_owned())
            .await
            .ok()
            .flatten()?;
        let artifact_id = address.parse::<clipmill_core::ArtifactId>().ok()?;
        let lease = artifacts.open(artifact_id).await.ok()?;
        let descriptor: clipmill_contracts::schemas::media_proxy::MediaProxy =
            crate::media::read_artifact_document(&lease, "proxy.json").ok()?;
        Some(PreviewProxyV1 {
            source_fingerprint: String::new(),
            artifact_id: address,
            file: descriptor.file.to_string(),
            coverage_start_ticks: i64::try_from(descriptor.coverage.start_ticks).unwrap_or(0),
            coverage_end_ticks: i64::try_from(descriptor.coverage.end_ticks).unwrap_or(i64::MAX),
            width: descriptor.video.width,
            height: descriptor.video.height,
            rate_num: u32::try_from(descriptor.video.frame_rate.num.get()).unwrap_or(0),
            rate_den: u32::try_from(descriptor.video.frame_rate.den.get()).unwrap_or(0),
        })
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

    /// The profile a document is previewed at: the default frame, at the
    /// frame rate of the recording the clip opens on, which is what an export
    /// renders at unless another rate is chosen.
    async fn preview_profile(
        &self,
        project_id: &str,
        document: &clipmill_edit_ir::EditDocument,
    ) -> clipmill_render::RenderProfile {
        let sources = self
            .database
            .list_sources(project_id.to_owned())
            .await
            .unwrap_or_default();
        let mut profile = clipmill_render::RenderProfile::default();
        if let Some(first) = document.video.segments.first()
            && let Some(rate) = sources
                .iter()
                .find(|source| source.source_fingerprint == first.source_fingerprint)
                .and_then(|source| crate::inspector::frame_rate_of(&source.source_map_json))
        {
            profile.frame_rate = rate;
        }
        profile
    }

    /// The captions a document would burn in under another look, unsaved.
    ///
    /// Trying a look is not an edit: the editor draws this while a person is
    /// still choosing and sends one command when they have chosen.
    async fn preview_captions(
        &self,
        request_id: String,
        request: &clipmill_contracts::proto::ipc::v1::PreviewCaptionsRequest,
    ) -> Reply {
        let doc_id = match request.doc_id.parse::<EditDocId>() {
            Ok(value) => value,
            Err(error) => {
                return error_reply(request_id, ErrorCode::InvalidArgument, error.to_string());
            }
        };
        let record = match self.database.get_edit_doc(doc_id.to_string()).await {
            Ok(record) => record,
            Err(error) => return store_error_reply(request_id, &error),
        };
        let Ok(mut document) =
            clipmill_edit_ir::EditDocument::from_canonical_json(record.document_json.as_bytes())
        else {
            return error_reply(
                request_id,
                ErrorCode::Internal,
                "the stored document did not parse",
            );
        };
        if !request.style_ref.is_empty() {
            if clipmill_captions::preset(&request.style_ref).is_none() {
                return error_reply(
                    request_id,
                    ErrorCode::InvalidArgument,
                    "unknown caption look",
                );
            }
            document.captions.style_ref.clone_from(&request.style_ref);
        }
        if !request.options_json.is_empty() {
            match serde_json::from_str::<clipmill_edit_ir::CaptionOptions>(&request.options_json) {
                Ok(options) => document.captions.options = options,
                Err(error) => {
                    return error_reply(request_id, ErrorCode::InvalidArgument, error.to_string());
                }
            }
        }
        let profile = self.preview_profile(&record.project_id, &document).await;
        match clipmill_render::caption_ass(&document, &profile) {
            Ok(ass) => response_reply(
                request_id,
                response::Body::PreviewCaptions(
                    clipmill_contracts::proto::ipc::v1::PreviewCaptionsResponse {
                        ass,
                        revision: record.revision,
                    },
                ),
            ),
            Err(error) => error_reply(request_id, ErrorCode::InvalidArgument, error.to_string()),
        }
    }

    /// Every command a document has had, oldest first, with its inverse.
    async fn list_edit_history(&self, request_id: String, value: &str) -> Reply {
        let doc_id = match value.parse::<EditDocId>() {
            Ok(value) => value,
            Err(error) => {
                return error_reply(request_id, ErrorCode::InvalidArgument, error.to_string());
            }
        };
        match self.database.get_edit_log(doc_id.to_string()).await {
            Ok((_initial, entries)) => response_reply(
                request_id,
                response::Body::ListEditHistory(
                    clipmill_contracts::proto::ipc::v1::ListEditHistoryResponse {
                        entries: entries
                            .into_iter()
                            .map(
                                |entry| clipmill_contracts::proto::ipc::v1::EditHistoryEntryV1 {
                                    revision: entry.revision,
                                    command_json: entry.command_json,
                                    inverse_json: entry.inverse_json,
                                    applied_unix_millis: entry.applied_unix_millis,
                                },
                            )
                            .collect(),
                    },
                ),
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

    /// Point a registered source at the file it moved to.
    ///
    /// The new file is fully inspected and accepted only when its fingerprint
    /// is the one the source was registered with, so everything already made
    /// from the recording — analysis, clips, edits — stays attached to it.
    async fn relink_source(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        project_id: ProjectId,
        register: &RegisterSourceRequest,
        inspector: &crate::sources::SourceInspector,
        sampled: crate::sources::SampledSource,
    ) -> Reply {
        if register.source_id.parse::<SourceId>().is_err() {
            return error_reply(
                request_id,
                ErrorCode::InvalidArgument,
                "source id is invalid",
            );
        }
        let inspection = match inspector.complete(sampled).await {
            Ok(value) => value,
            Err(error) => return source_probe_error_reply(request_id, &error),
        };
        let now = match unix_millis() {
            Ok(value) => value,
            Err(message) => return error_reply(request_id, ErrorCode::Internal, message),
        };
        match self
            .database
            .relink_source(
                request_id.clone(),
                request_hash,
                project_id.to_string(),
                register.source_id.clone(),
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
        if !register.source_id.is_empty() {
            return self
                .relink_source(
                    request_id,
                    request_hash,
                    project_id,
                    register,
                    inspector,
                    sampled,
                )
                .await;
        }
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
                    source_map_json: String::from_utf8(source.source_map_json.clone())
                        .unwrap_or_default(),
                    source: Some(source_reply(source)),
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
                    sources: sources.into_iter().map(source_reply).collect(),
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

/// The gate for a face somebody picked: it only has to be seen in the span.
/// Presence, score and margin keep the automatic camera honest; a person who
/// points at a face has already made the call they guard.
const CHOSEN_FACE: FocusGate = FocusGate {
    min_presence: 0.05,
    min_score: 0.0,
    min_margin: 0.0,
};

/// The longest span `ListFaces` answers for: a clip's section, not a recording.
const MAX_FACE_SPAN_TICKS: u64 = 10 * 60 * 90_000;

/// A fitted answer with the reason for it.
fn refused_crop(reason: &str) -> SolveCropPathResponse {
    SolveCropPathResponse {
        fit: true,
        fit_reason: reason.to_owned(),
        ..SolveCropPathResponse::default()
    }
}

/// The camera a solve request asks for: the gate's choice, the face a person
/// picked, or both portraits of a two-person layout.
fn crop_solve(
    document: VisionFaceTrack,
    solve: &SolveCropPathRequest,
    (source_width, source_height): (u32, u32),
) -> Result<SolveCropPathResponse, clipmill_reframe::SolveError> {
    let geometry = |output_width: u32| clipmill_reframe::FrameGeometry {
        source_width,
        source_height,
        output_width,
        output_height: solve.aspect_height,
    };
    let weights = crop_weights(solve.weights.as_ref());
    if solve.two_up {
        // Each portrait is half the frame tall, so each crop is twice as wide
        // for its height as the frame is.
        let halves = geometry(solve.aspect_width.saturating_mul(2));
        return two_up_solve(&document, solve, halves, weights);
    }
    let (document, gate) = if solve.follow_track {
        let mut one = document;
        one.tracks
            .retain(|track| track.track_id == u64::from(solve.track_id));
        if one.tracks.is_empty() {
            return Ok(refused_crop("that person is not in this recording"));
        }
        (one, CHOSEN_FACE)
    } else {
        (document, FocusGate::default())
    };
    clipmill_reframe::solve_in_frame(
        &document,
        solve.start_ticks,
        solve.end_ticks,
        geometry(solve.aspect_width),
        weights,
        gate,
    )
    .map(|solved| crop_response(&solved))
}

/// Both portraits of a two-person layout, solved the way the director solves
/// them: the pair the two-up gate finds, left-hand face on top, each followed
/// alone through the span.
fn two_up_solve(
    document: &VisionFaceTrack,
    solve: &SolveCropPathRequest,
    halves: clipmill_reframe::FrameGeometry,
    weights: Weights,
) -> Result<SolveCropPathResponse, clipmill_reframe::SolveError> {
    let Some(pair) = clipmill_reframe::resolve_pair(document, solve.start_ticks, solve.end_ticks)
    else {
        return Ok(refused_crop(
            "two people are not both clearly in this section",
        ));
    };
    let mut solved = Vec::with_capacity(2);
    for id in pair {
        let mut one = document.clone();
        one.tracks.retain(|track| track.track_id == id);
        let path = clipmill_reframe::solve_in_frame(
            &one,
            solve.start_ticks,
            solve.end_ticks,
            halves,
            weights,
            FocusGate::default(),
        )?;
        if path.fit {
            return Ok(refused_crop(
                "one of the two people is not clear enough in this section to follow",
            ));
        }
        solved.push(path);
    }
    let [upper, lower] = solved.as_slice() else {
        return Ok(refused_crop(
            "two people are not both clearly in this section",
        ));
    };
    let mut response = crop_response(upper);
    let second = crop_response(lower);
    response.secondary_keyframes = second.keyframes;
    response.secondary_track_id = second.track_id;
    response.containment = upper.containment.min(lower.containment);
    Ok(response)
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
        ..SolveCropPathResponse::default()
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
        Some(request::Body::PlanExport(_)) => "plan_export",
        Some(request::Body::ExportClip(_)) => "export_clip",
        Some(request::Body::ExportArchive(_)) => "export_archive",
        Some(request::Body::GetLocalLock(_)) => "get_local_lock",
        Some(request::Body::GetReadiness(_)) => "get_readiness",
        Some(request::Body::SubmitExportBatch(_)) => "submit_export_batch",
        Some(request::Body::ListExportBatches(_)) => "list_export_batches",
        Some(request::Body::ConfigureYoutubePublishing(_)) => "configure_youtube_publishing",
        Some(request::Body::ConnectYoutubeChannel(_)) => "connect_youtube_channel",
        Some(request::Body::GetYoutubePublishingStatus(_)) => "get_youtube_publishing_status",
        Some(request::Body::UpdateYoutubeConnection(_)) => "update_youtube_connection",
        Some(request::Body::StartYoutubeUpload(_)) => "start_youtube_upload",
        Some(request::Body::GetYoutubeUpload(_)) => "get_youtube_upload",
        Some(request::Body::ListYoutubeUploads(_)) => "list_youtube_uploads",
        Some(request::Body::UpdateYoutubeUpload(_)) => "update_youtube_upload",
        Some(request::Body::PublishYoutubeUpload(_)) => "publish_youtube_upload",
        Some(request::Body::DraftYoutubeMetadata(_)) => "draft_youtube_metadata",
        Some(request::Body::StartYoutubeImport(_)) => "start_youtube_import",
        Some(request::Body::GetYoutubeImport(_)) => "get_youtube_import",
        Some(request::Body::ListYoutubeImports(_)) => "list_youtube_imports",
        Some(request::Body::UpdateYoutubeImport(_)) => "update_youtube_import",
        Some(request::Body::UpdateExportBatchItem(_)) => "update_export_batch_item",
        Some(request::Body::Ping(_)) => "ping",
        Some(request::Body::Health(_)) => "health",
        Some(request::Body::CreateProject(_)) => "create_project",
        Some(request::Body::GetProject(_)) => "get_project",
        Some(request::Body::ListProjects(_)) => "list_projects",
        Some(request::Body::DeleteProject(_)) => "delete_project",
        Some(request::Body::RenameProject(_)) => "rename_project",
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
        Some(request::Body::PreviewCaptions(_)) => "preview_captions",
        Some(request::Body::ListEditHistory(_)) => "list_edit_history",
        Some(request::Body::ListFaces(_)) => "list_faces",
        Some(request::Body::SnapshotEditDoc(_)) => "snapshot_edit_doc",
        Some(request::Body::ListModels(_)) => "list_models",
        Some(request::Body::DownloadModels(_)) => "download_models",
        Some(request::Body::CancelModelDownload(_)) => "cancel_model_download",
        Some(request::Body::RemoveModel(_)) => "remove_model",
        Some(request::Body::VerifyModel(_)) => "verify_model",
        Some(request::Body::SetModelChoice(_)) => "set_model_choice",
        Some(request::Body::InspectHubModel(_)) => "inspect_hub_model",
        Some(request::Body::AddCustomModel(_)) => "add_custom_model",
        Some(request::Body::ForgetModel(_)) => "forget_model",
        Some(request::Body::CleanStorage(_)) => "clean_storage",
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
        StoreError::Conflict
        | StoreError::RelinkMismatch
        | StoreError::ImportQualityConflict
        | StoreError::PublishingConflict(_) => {
            error_reply(request_id, ErrorCode::Conflict, error.to_string())
        }
        StoreError::NotFound => error_reply(request_id, ErrorCode::NotFound, error.to_string()),
        StoreError::Database(_) | StoreError::InvalidData(_) | StoreError::Stopped => {
            // The caller is told only that the store failed; the reason is
            // for the log, where a store that refuses a document says why.
            tracing::warn!(%request_id, error = %error, "store request failed");
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
fn preview_colour(colour: clipmill_render::Colour) -> String {
    format!(
        "#{:02x}{:02x}{:02x}{:02x}",
        colour.red,
        colour.green,
        colour.blue,
        255 - colour.transparency
    )
}

#[allow(
    clippy::too_many_lines,
    reason = "field-for-field conversion of the shared render preview contract"
)]
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
        transition_ticks: plan.transition_ticks,
        transitions: plan
            .transitions
            .iter()
            .map(
                |transition| clipmill_contracts::proto::ipc::v1::PreviewTransitionV1 {
                    incoming_segment_id: transition.incoming_segment_id.clone(),
                    outgoing_frame: transition.outgoing_frame,
                    first_frame: transition.first_frame,
                    end_frame: transition.end_frame,
                },
            )
            .collect(),
        caption_style: Some(clipmill_contracts::proto::ipc::v1::PreviewCaptionStyleV1 {
            style_ref: plan.caption_style.style_ref.clone(),
            font_family: plan.caption_style.font_family.clone(),
            font_size: plan.caption_style.font_size,
            spoken: preview_colour(plan.caption_style.spoken),
            unspoken: preview_colour(plan.caption_style.unspoken),
            outline: preview_colour(plan.caption_style.outline),
            shadow: preview_colour(plan.caption_style.shadow),
            outline_width: plan.caption_style.outline_width,
            shadow_depth: plan.caption_style.shadow_depth,
            bold: plan.caption_style.bold,
            boxed: plan.caption_style.boxed,
            margin_horizontal: plan.caption_style.margin_horizontal,
            margin_vertical: plan.caption_style.margin_vertical,
            accent: preview_colour(plan.caption_style.accent),
            highlight: match plan.caption_style.highlight {
                clipmill_edit_ir::HighlightStyle::Fill => "fill",
                clipmill_edit_ir::HighlightStyle::Word => "word",
                clipmill_edit_ir::HighlightStyle::Box => "box",
                clipmill_edit_ir::HighlightStyle::Pop => "pop",
                clipmill_edit_ir::HighlightStyle::Underline => "underline",
            }
            .to_owned(),
        }),
        ass: plan.ass.clone(),
        // Filled by the caller, which knows where the fonts are installed.
        fonts: Vec::new(),
        secondary_crops: plan
            .secondary_crops
            .iter()
            .map(|crop| match crop {
                Some(rect) => PreviewCropV1 {
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height,
                    present: true,
                },
                None => PreviewCropV1::default(),
            })
            .collect(),
        cues: plan.cues.iter().map(preview_cue_response).collect(),
        reading_cues: plan.reading_cues.iter().map(preview_cue_response).collect(),
        reading_min_duration_ticks: clipmill_captions::Profile::ACCESSIBILITY_EN.min_duration_ticks,
        reading_min_gap_ticks: clipmill_captions::Profile::ACCESSIBILITY_EN.min_gap_ticks,
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
        segments: plan
            .segments
            .iter()
            .map(|segment| PreviewSegmentV1 {
                segment_id: segment.segment_id.clone(),
                source_fingerprint: segment.source_fingerprint.clone(),
                in_ticks: segment.in_ticks,
                out_ticks: segment.out_ticks,
                program_start_ticks: segment.program_start_ticks,
                first_frame: segment.first_frame,
                end_frame: segment.end_frame,
                has_two_up_paths: segment.has_two_up_paths,
                framing_warning: segment.framing_warning.clone(),
                layout: segment.layout.to_owned(),
                upper_height: segment.upper_height,
                has_inset: segment.inset.is_some(),
                inset_x: segment.inset.map_or(0, |(x, _, _)| x),
                inset_y: segment.inset.map_or(0, |(_, y, _)| y),
                inset_side: segment.inset.map_or(0, |(_, _, side)| side),
                background_colour: segment.background_colour.clone().unwrap_or_default(),
                zoom_percent: u32::from(segment.zoom_percent),
            })
            .collect(),
        // Resolved by the handler, which is the one with a database.
        sources: Vec::new(),
        proxies: Vec::new(),
        presentation: plan.presentation.as_str().to_owned(),
    }
}

fn preview_cue_response(cue: &clipmill_render::PreviewCue) -> PreviewCueV1 {
    PreviewCueV1 {
        cue_id: cue.cue_id.clone(),
        positioned: cue.position.is_some(),
        position_x: cue.position.map_or(0, |position| position.x),
        position_y: cue.position.map_or(0, |position| position.y),
        start_ticks: cue.start_ticks,
        end_ticks: cue.end_ticks,
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
                        word_id: word.word_id.clone().unwrap_or_default(),
                    })
                    .collect(),
            })
            .collect(),
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
    if evidence
        .ranking
        .declined
        .iter()
        .any(|row| row.candidate_id.as_str() == direct.candidate_id)
        && !direct.allow_declined
    {
        return Err(
            "This nomination was declined. Choose Edit despite review to create a manual edit."
                .to_owned(),
        );
    }
    let style_ref = if direct.style_ref.is_empty() {
        clipmill_captions::DEFAULT_STYLE_REF.to_owned()
    } else {
        direct.style_ref.clone()
    };
    let cut = if direct.manual_span {
        clipmill_director::Cut::Chosen
    } else {
        match ClipCutV1::try_from(direct.cut).unwrap_or(ClipCutV1::Unspecified) {
            ClipCutV1::Alternative => clipmill_director::Cut::Alternative,
            // A person's cut may land between any two words (R63). An edge that
            // arrives inside one is moved out to keep the word before anything
            // is built: this socket is not the only way a pair can get here, and
            // the director refuses a severed word rather than repairing it.
            ClipCutV1::Exact => clipmill_director::Cut::Exact(whole_words(evidence, direct)?),
            ClipCutV1::Chosen | ClipCutV1::Unspecified => clipmill_director::Cut::Chosen,
        }
    };
    let director_evidence = clipmill_director::Evidence {
        candidates: &evidence.candidates,
        ranking: &evidence.ranking,
        transcript: &evidence.transcript,
        index: evidence.index.as_ref(),
        shots: evidence.shots.as_ref(),
        faces: evidence.faces.as_ref(),
    };
    let request = clipmill_director::Request {
        candidate_id: direct.candidate_id.clone(),
        cut,
        style_ref,
        frame: evidence.frame,
        aspect: clipmill_director::Aspect::default(),
    };
    if direct.manual_span {
        let boundary = clipmill_director::Boundary {
            start_ticks: i64::try_from(direct.start_ticks)
                .map_err(|_| "start exceeds timeline".to_owned())?,
            end_ticks: i64::try_from(direct.end_ticks)
                .map_err(|_| "end exceeds timeline".to_owned())?,
        };
        clipmill_director::direct_span(director_evidence, &request, boundary)
    } else {
        clipmill_director::direct(director_evidence, &request)
    }
    .map_err(|error| error.to_string())
}

/// A hand-set boundary, with any edge that falls inside a word moved out to
/// keep the whole word.
///
/// The lattice is not consulted: it is the search's shortlist, and a reviewer
/// is free to disagree with it. The one thing a cut may not do is sever a word,
/// and moving the edge outward is what the person evidently meant — the handle
/// they dragged was on that word.
fn whole_words(
    evidence: &crate::inspector::Evidence,
    direct: &DirectClipRequest,
) -> Result<clipmill_director::Boundary, String> {
    let wanted = clipmill_director::Boundary {
        start_ticks: i64::try_from(direct.start_ticks)
            .map_err(|_| "the start is past the end of the timeline".to_owned())?,
        end_ticks: i64::try_from(direct.end_ticks)
            .map_err(|_| "the end is past the end of the timeline".to_owned())?,
    };
    if wanted.end_ticks <= wanted.start_ticks {
        return Err("a cut has to end after it starts".to_owned());
    }
    Ok(clipmill_director::keep_whole_words(
        &evidence.transcript,
        wanted,
    ))
}

/// Export planning, delivery, and settings operations.
///
/// Two of these four are the same operation asked twice. `plan_export` answers
/// "what would happen", `export_clip` makes it happen, and both start by
/// running the same strip over the same document — because a preview that
/// checked something different from what the export checks is a preview that
/// lies about the interesting case.
impl Service {
    /// The strip and the names, without touching anything.
    async fn plan_export(&self, request_id: String, request: &PlanExportRequest) -> Reply {
        let Some(asked) = request.request.as_ref() else {
            return error_reply(
                request_id,
                ErrorCode::InvalidArgument,
                "an export request is required",
            );
        };
        let (document, record) = match self.export_document(&request_id, &asked.doc_id).await {
            Ok(loaded) => loaded,
            Err(reply) => return reply,
        };
        let pattern = match crate::export::naming_pattern(&asked.naming_pattern) {
            Ok(pattern) => pattern,
            Err(error) => {
                return error_reply(request_id, ErrorCode::InvalidArgument, error.to_string());
            }
        };

        let available = available_at(&asked.destination_dir);
        let (height, chosen_rate) = match crate::render::output_request(asked.format.as_ref()) {
            Ok(value) => value,
            Err(message) => return error_reply(request_id, ErrorCode::InvalidArgument, message),
        };
        let sources = self
            .database
            .list_sources(record.project_id.clone())
            .await
            .unwrap_or_default();
        let output = output_profile(&document, &sources, height, chosen_rate);
        let estimated = scaled_estimate(document.program_duration_ticks(), &output);
        let report = clipmill_export::validate(
            &document,
            &clipmill_export::Context {
                source_attestation: &asked.source_attestation,
                gates_passed: &asked.gates_passed,
                estimated_bytes: estimated,
                available_bytes: available,
            },
        );
        // A destination that cannot be written to is a blocking finding rather
        // than an error, so it arrives beside the others in the same list a
        // user is already reading.
        let mut findings = validation_of(&report);
        for path in missing_recordings(&document, &sources) {
            findings.passes = false;
            findings.findings.push(missing_recording_finding(&path));
        }
        if let Some(finding) = upscale_finding(&document, &sources, &output) {
            findings.findings.push(finding);
        }
        if let Err(error) = crate::export::probe_destination(&asked.destination_dir) {
            findings.passes = false;
            findings.findings.push(ExportFindingV1 {
                code: "destination.unusable".to_owned(),
                severity: ExportSeverity::Blocking as i32,
                detail: error.to_string(),
                cue_id: String::new(),
            });
        }

        let stem = planned_stem(&pattern, asked, &document);
        response_reply(
            request_id,
            response::Body::PlanExport(PlanExportResponse {
                validation: Some(findings),
                file_names: delivered_names(&stem),
                stem,
                estimated_bytes: estimated,
                available_bytes: available.unwrap_or(0),
                available_known: available.is_some(),
                revision: record.revision,
            }),
        )
    }

    /// Why an export may not start, in the words the export strip uses, or
    /// `None` when it may. A recording that moved is named first: nothing
    /// else can be fixed from here until it is found.
    async fn export_refusal(
        &self,
        asked: &ExportRequestV1,
        document: &clipmill_edit_ir::EditDocument,
        project_id: &str,
    ) -> Option<String> {
        if let Ok(sources) = self.database.list_sources(project_id.to_owned()).await
            && let Some(path) = missing_recordings(document, &sources).first()
        {
            return Some(missing_recording_finding(path).detail);
        }
        let report = clipmill_export::validate(
            document,
            &clipmill_export::Context {
                source_attestation: &asked.source_attestation,
                gates_passed: &asked.gates_passed,
                estimated_bytes: clipmill_export::estimate_bytes(document.program_duration_ticks()),
                available_bytes: available_at(&asked.destination_dir),
            },
        );
        // The reasons travel in the message: an export that failed without
        // saying why is a dialog a user closes and gives up on.
        (!report.passes()).then(|| {
            report
                .blocking()
                .map(|finding| finding.detail.clone())
                .collect::<Vec<_>>()
                .join(" ")
        })
    }

    /// Perform an export: snapshot, render, deliver — as one job.
    async fn export_clip(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        request: &ExportClipRequest,
    ) -> Reply {
        // An accepted export belongs to its frozen snapshot even if the edit
        // or destination has changed since. Recover it before mutable checks.
        match self
            .database
            .replay_request(request_id.clone(), request_hash)
            .await
        {
            Ok(Some(bytes)) => return export_submission_reply(request_id, &bytes),
            Ok(None) => {}
            Err(error) => return store_error_reply(request_id, &error),
        }
        let Some(asked) = request.request.as_ref() else {
            return error_reply(
                request_id,
                ErrorCode::InvalidArgument,
                "an export request is required",
            );
        };
        let Some(artifacts) = &self.artifacts else {
            return error_reply(
                request_id,
                ErrorCode::Unavailable,
                "artifact publication is not available",
            );
        };
        let (document, record) = match self.export_document(&request_id, &asked.doc_id).await {
            Ok(loaded) => loaded,
            Err(reply) => return reply,
        };
        if let Err((code, message)) = admit_export(asked, record.revision) {
            return error_reply(request_id, code, message);
        }
        // Checked before the destination is created, so a refused export leaves
        // no empty folder behind explaining nothing.
        if let Some(reasons) = self
            .export_refusal(asked, &document, &record.project_id)
            .await
        {
            return error_reply(request_id, ErrorCode::PolicyDenied, reasons);
        }
        let destination = match crate::export::resolve_destination(&asked.destination_dir) {
            Ok(path) => path,
            Err(error) => {
                return error_reply(request_id, ErrorCode::InvalidArgument, error.to_string());
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
        let ir_artifact_id = match Self::publish_snapshot(artifacts, recipe, &payload).await {
            Ok(artifact_id) => artifact_id,
            Err((code, message)) => return error_reply(request_id, code, message),
        };
        if let Err(error) = self
            .database
            .attach_artifact_root(project_id.clone(), ir_artifact_id)
            .await
        {
            return store_error_reply(request_id, &error);
        }

        // The destination is resolved into the payload, so the delivery reads
        // the folder that was checked rather than re-resolving a string that
        // may point somewhere else by the time it runs.
        let mut resolved = asked.clone();
        resolved.destination_dir = destination.to_string_lossy().into_owned();
        // The revision rendered is written into the job's own record of the
        // request, whether or not the caller named one, so the job can say
        // afterwards which revision it delivered.
        resolved.expected_revision = Some(record.revision);
        self.submit_export(
            request_id,
            request_hash,
            &project_id,
            resolved,
            ir_artifact_id,
            record.revision,
        )
        .await
    }

    /// Plan and submit the two-task export job.
    async fn submit_export(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        project_id: &ProjectId,
        resolved: ExportRequestV1,
        ir_artifact_id: clipmill_core::ArtifactId,
        _revision: u64,
    ) -> Reply {
        let render_payload = RenderClipPayloadV1 {
            key_version: RENDER_CLIP_KEY_VERSION.to_owned(),
            doc_id: resolved.doc_id.clone(),
            ir_artifact_id: ir_artifact_id.to_string(),
            source_attestation: resolved.source_attestation.clone(),
            gates_passed: resolved.gates_passed.clone(),
            ai_assistance: resolved.ai_assistance.clone(),
            format: resolved.format,
        }
        .encode_to_vec();
        let deliver_payload = DeliverExportPayloadV1 {
            key_version: EXPORT_CLIP_KEY_VERSION.to_owned(),
            request: Some(resolved.clone()),
        }
        .encode_to_vec();
        let job_payload = ExportClipPayloadV1 {
            key_version: EXPORT_CLIP_KEY_VERSION.to_owned(),
            request: Some(resolved),
            ir_artifact_id: ir_artifact_id.to_string(),
        }
        .encode_to_vec();

        let Ok(now) = unix_millis() else {
            return error_reply(request_id, ErrorCode::Internal, "the clock is unreadable");
        };
        let plan = JobPlan::export_clip(
            project_id,
            job_payload,
            render_payload,
            deliver_payload,
            now,
        );
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
                export_submission_reply(request_id, &result.bytes)
            }
            Err(error) => store_error_reply(request_id, &error),
        }
    }

    /// Pack a project's work into a zip.
    ///
    /// Synchronous rather than a job: this reads state and writes one file of
    /// documents, which is milliseconds, and a progress bar for it would be a
    /// progress bar that is never seen.
    async fn export_archive(&self, request_id: String, request: &ExportArchiveRequest) -> Reply {
        let project_id = match request.project_id.parse::<ProjectId>() {
            Ok(value) => value,
            Err(error) => {
                return error_reply(request_id, ErrorCode::InvalidArgument, error.to_string());
            }
        };
        let project = match self.database.get_project(project_id.to_string()).await {
            Ok(project) => project,
            Err(error) => return store_error_reply(request_id, &error),
        };
        let destination = match crate::export::resolve_destination(&request.destination_dir) {
            Ok(path) => path,
            Err(error) => {
                return error_reply(request_id, ErrorCode::InvalidArgument, error.to_string());
            }
        };

        let sources = self
            .database
            .list_sources(project_id.to_string())
            .await
            .unwrap_or_default();
        let docs = self
            .database
            .list_edit_docs(project_id.to_string())
            .await
            .unwrap_or_default();
        let mut logs = Vec::with_capacity(docs.len());
        for doc in &docs {
            match self.database.get_edit_log(doc.doc_id.clone()).await {
                Ok((initial, entries)) => logs.push((doc.doc_id.clone(), initial, entries)),
                // A document whose log could not be read is still worth
                // archiving; the index will simply not claim to carry one.
                Err(error) => tracing::warn!(%error, doc_id = doc.doc_id, "edit log unreadable"),
            }
        }
        // Decisions are held per source, so the archive gathers them the same
        // way — and carries the source each one belongs to, because a candidate
        // id alone does not say which recording it came from.
        let mut decisions = Vec::new();
        for source in &sources {
            if let Ok(found) = self
                .database
                .list_clip_decisions(project_id.to_string(), source.source_id.clone())
                .await
            {
                decisions.extend(
                    found
                        .into_iter()
                        .map(|record| (source.source_id.clone(), record)),
                );
            }
        }
        let jobs = self
            .database
            .list_jobs(project_id.to_string())
            .await
            .unwrap_or_default();

        let built = crate::export::build_archive(&crate::export::ArchiveInputs {
            project: &project,
            sources: &sources,
            docs: &docs,
            logs: &logs,
            decisions: &decisions,
            jobs: &jobs,
            created_unix_millis: unix_millis().unwrap_or(0),
            writer_version: env!("CARGO_PKG_VERSION"),
        });
        let (bytes, entry_count) = match built {
            Ok(built) => built,
            Err(error) => return error_reply(request_id, ErrorCode::Internal, error.to_string()),
        };
        let name = format!(
            "{}.clipmill-archive.zip",
            crate::export::archive_stem(&project)
        );
        let path = destination.join(&name);
        let digest = clipmill_export::digest_of(&bytes);
        let written = bytes.len() as u64;
        if let Err(error) = crate::export::write_atomically(&path, &bytes) {
            return error_reply(request_id, ErrorCode::Internal, error.to_string());
        }
        response_reply(
            request_id,
            response::Body::ExportArchive(ExportArchiveResponse {
                path: path.to_string_lossy().into_owned(),
                sha256: digest,
                bytes: written,
                entry_count,
            }),
        )
    }

    fn get_local_lock(&self, request_id: String) -> Reply {
        let status = self.policy.status();
        response_reply(
            request_id,
            response::Body::GetLocalLock(GetLocalLockResponse {
                status: Some(LocalLockStatusV1 {
                    engaged: status.engaged,
                    stages: status.stages,
                    network_allowed_stages: status.network_allowed_stages,
                    egress_attempts: status.egress_attempts,
                }),
            }),
        )
    }

    /// Whether an analysis could run right now, stage by stage.
    ///
    /// Answered from what the planner would bind — the same bindings a job
    /// is planned with — against what is on disk and who is connected. A
    /// stage is ready when its model's pinned files are present at the sizes
    /// the registry pins and a worker of the family that runs its model is on
    /// the roster: another family declaring the stage is never handed it.
    /// The remedy names the command, because a status that says "not ready"
    /// and nothing else is a spinner with a label.
    fn get_readiness(&self, request_id: String) -> Reply {
        // The bindings an analysis submitted now would plan: the person's
        // choices and the installed fallbacks included, so readiness judges
        // the models that would actually run rather than the profile's.
        let bindings = self.planning_bindings();
        let roster = self
            .roster
            .lock()
            .map(|workers| workers.clone())
            .unwrap_or_default();
        let weights = self.storage.as_ref().map(|dirs| dirs.weights.clone());
        let capacity = self
            .scheduler
            .as_ref()
            .map(crate::jobs::SchedulerHandle::machine_capacity);
        let mut stages = Vec::new();
        for binding in bindings.iter() {
            // Publishing copy is not part of an analysis; its screen asks.
            if binding.stage == "youtube-metadata" {
                continue;
            }
            let (present, missing) = weights.as_deref().map_or_else(
                || (false, vec![binding.model.clone()]),
                |root| self.model_files_present(&binding.model, root),
            );
            let resident = self
                .models
                .get(&binding.model)
                .map(|model| model.memory.resident_bytes());
            let mut readiness = stage_readiness(
                &binding.stage,
                &binding.capability,
                &binding.implementation,
                &binding.model,
                &binding.backend,
                present,
                missing,
                roster
                    .values()
                    .any(|worker| worker.runs(&binding.stage, &binding.implementation)),
            );
            check_worker_ceiling(
                &mut readiness,
                &roster,
                resident,
                capacity.map(|capacity| capacity.ram_bytes),
            );
            if binding.capability == "editorial" {
                check_editorial_capacity(&mut readiness, capacity, resident);
            }
            stages.push(readiness);
        }
        // These stages need a connected worker but no local model files.
        for kind in crate::recipes::stages_with_network_policy(NetworkPolicy::NetworkAllowed)
            .chain(crate::recipes::modelless_worker_stages())
        {
            stages.push(stage_readiness(
                kind,
                "",
                "",
                "",
                "",
                true,
                Vec::new(),
                roster.values().any(|worker| worker.serves(kind)),
            ));
        }
        stages.sort_by(|left, right| left.stage.cmp(&right.stage));
        let decoder_path = self
            .decoder
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_default();
        let decoder_present = self.decoder.as_ref().is_some_and(|path| path.is_file());
        // Optional cloud workers never gate the default local analysis route.
        // A cloud analysis checks its selected stages before submission.
        let ready = decoder_present && local_analysis_stages_ready(&stages);
        response_reply(
            request_id,
            response::Body::GetReadiness(GetReadinessResponse {
                workers: roster
                    .into_iter()
                    .map(|(worker_id, presence)| WorkerPresenceV1 {
                        worker_id,
                        family: presence.family,
                        capabilities: presence.capabilities,
                        backend: presence.backend,
                        since_unix_millis: presence.since_unix_millis,
                    })
                    .collect(),
                stages,
                decoder_present,
                decoder_path,
                ready,
            }),
        )
    }

    /// Whether every pinned file of a model is where the worker will look,
    /// at the size the registry pins. Sizes rather than digests: a truncated
    /// download is the common failure, and the worker hashes before loading.
    fn model_files_present(
        &self,
        model: &str,
        weights_root: &std::path::Path,
    ) -> (bool, Vec<String>) {
        let Some(manifest) = self.models.get(model) else {
            return (false, vec![format!("{model} (not in the registry)")]);
        };
        let missing: Vec<String> = manifest
            .files
            .iter()
            .filter(|file| {
                let path = weights_root.join(model).join(&file.path);
                !std::fs::metadata(&path)
                    .is_ok_and(|meta| meta.is_file() && meta.len() == file.bytes)
            })
            .map(|file| format!("{model}/{}", file.path))
            .collect();
        (missing.is_empty(), missing)
    }

    /// The document an export names, parsed, with the row it came from.
    async fn export_document(
        &self,
        request_id: &str,
        doc_id: &str,
    ) -> Result<(clipmill_edit_ir::EditDocument, crate::db::EditDocRecord), Reply> {
        let parsed = match doc_id.parse::<EditDocId>() {
            Ok(value) => value,
            Err(error) => {
                return Err(error_reply(
                    request_id.to_owned(),
                    ErrorCode::InvalidArgument,
                    error.to_string(),
                ));
            }
        };
        let record = match self.database.get_edit_doc(parsed.to_string()).await {
            Ok(record) => record,
            Err(error) => return Err(store_error_reply(request_id.to_owned(), &error)),
        };
        match clipmill_edit_ir::EditDocument::from_canonical_json(record.document_json.as_bytes()) {
            Ok(document) => Ok((document, record)),
            Err(error) => {
                tracing::warn!(%error, "stored edit document failed to parse");
                Err(error_reply(
                    request_id.to_owned(),
                    ErrorCode::Internal,
                    "stored edit document is not valid",
                ))
            }
        }
    }
}

fn local_analysis_stages_ready(stages: &[StageReadinessV1]) -> bool {
    stages
        .iter()
        .filter(|stage| {
            crate::recipes::lookup(&stage.stage)
                .is_none_or(|recipe| recipe.network == NetworkPolicy::LocalLock)
        })
        .all(|stage| stage.ready)
}

/// A connected worker takes only tasks whose model fits the memory ceiling
/// it declared and signed, and is given nothing at all while that ceiling is
/// above the device's processing budget. Either way a task would wait for a
/// worker that can never take it — so the stage is not ready, and says why.
fn check_worker_ceiling(
    readiness: &mut StageReadinessV1,
    roster: &std::collections::BTreeMap<String, crate::worker::WorkerPresence>,
    required: Option<u64>,
    budget: Option<u64>,
) {
    if !readiness.ready {
        return;
    }
    let serving = roster
        .values()
        .filter(|worker| worker.runs(&readiness.stage, &readiness.implementation))
        .collect::<Vec<_>>();
    let admitted = serving
        .iter()
        .filter(|worker| budget.is_none_or(|budget| worker.max_memory_bytes <= budget))
        .collect::<Vec<_>>();
    if admitted.is_empty() {
        let declared = serving
            .iter()
            .map(|worker| worker.max_memory_bytes)
            .min()
            .unwrap_or(0);
        readiness.ready = false;
        readiness.remedy = format!(
            "The connected worker for this stage offered {} MiB, more than this device's processing budget of {} MiB, so it is never given work. Rescan the device in Models, then restart the workers.",
            declared / (1024 * 1024),
            budget.unwrap_or(0) / (1024 * 1024)
        );
        return;
    }
    let Some(required) = required else {
        return;
    };
    if admitted
        .iter()
        .any(|worker| worker.max_memory_bytes >= required)
    {
        return;
    }
    let largest = admitted
        .iter()
        .map(|worker| worker.max_memory_bytes)
        .max()
        .unwrap_or(0);
    readiness.ready = false;
    readiness.remedy = format!(
        "{} needs about {} MiB, and the connected worker accepts models up to {} MiB. Restart the workers so they size themselves for this device, or choose a smaller model in Models.",
        readiness.model,
        required.div_ceil(1024 * 1024),
        largest / (1024 * 1024)
    );
}

fn check_editorial_capacity(
    readiness: &mut StageReadinessV1,
    capacity: Option<crate::jobs::ResourceCapacity>,
    required: Option<u64>,
) {
    if !readiness.ready {
        return;
    }
    let (Some(capacity), Some(required)) = (capacity, required) else {
        return;
    };
    if capacity.accelerator_mask & crate::jobs::accelerator_bit("metal").unwrap_or(0) == 0 {
        readiness.ready = false;
        readiness.remedy = format!(
            "{} has not passed its local runtime check. Restart `just workers` to run the check, then refresh readiness.",
            readiness.model
        );
    } else if capacity.ram_bytes < required {
        readiness.ready = false;
        readiness.remedy = if cfg!(target_os = "macos") {
            format!(
                "{} needs {} MiB; this Mac's physical-memory processing budget is {} MiB. Choose a smaller editorial model in Models, or use a Mac with more memory. Current free memory does not block admission.",
                readiness.model,
                required.div_ceil(1024 * 1024),
                capacity.ram_bytes / (1024 * 1024)
            )
        } else {
            format!(
                "{} needs {} MiB of schedulable memory; {} MiB is available. Close memory-heavy applications, rescan in Models, then refresh readiness.",
                readiness.model,
                required.div_ceil(1024 * 1024),
                capacity.ram_bytes / (1024 * 1024)
            )
        };
    }
}

/// One stage's readiness row, with the sentence that says what to do.
#[allow(clippy::too_many_arguments)]
fn stage_readiness(
    stage: &str,
    capability: &str,
    implementation: &str,
    model: &str,
    backend: &str,
    model_present: bool,
    missing_files: Vec<String>,
    worker_present: bool,
) -> StageReadinessV1 {
    let remedy = match (model_present, worker_present) {
        (true, true) => String::new(),
        (false, _) => format!(
            "{model} is not installed ({} file(s) missing). Download it in Models, or choose \
             another model for this job there.",
            missing_files.len()
        ),
        (true, false)
            if crate::recipes::lookup(stage)
                .is_some_and(|recipe| recipe.network == NetworkPolicy::NetworkAllowed) =>
        {
            format!(
                "Optional cloud worker for {stage} is not running. To enable it, use `./tools/run-workers.sh --cloud-editorial`; each analysis still needs explicit cloud consent."
            )
        }
        (true, false) if stage.starts_with("editorial-") => format!(
            "No worker is connected that runs {stage}: install it with `uv sync --project workers/editorial`, \
             restart `just app` to enroll it, then run `just workers`."
        ),
        (true, false) => match crate::implementations::lookup(implementation) {
            Some(known) => format!(
                "{model} runs in the {}, which is not connected. Restart `just workers` to start \
                 it, or choose another model for this job in Models.",
                crate::implementations::worker_title(known.worker)
            ),
            None => format!(
                "No worker is connected that runs {stage}: start the workers with `just workers`."
            ),
        },
    };
    StageReadinessV1 {
        stage: stage.to_owned(),
        capability: capability.to_owned(),
        implementation: implementation.to_owned(),
        model: model.to_owned(),
        backend: backend.to_owned(),
        model_present,
        missing_files,
        worker_present,
        ready: model_present && worker_present,
        remedy,
    }
}

/// The refusals an export request earns before anything is read or written.
///
/// The revision the person reviewed is the one that may leave. An edit that
/// landed between the review and this request — another window, a late
/// command — would otherwise be delivered unseen; the caller re-plans against
/// what the document is now and asks again.
fn admit_export(asked: &ExportRequestV1, revision: u64) -> Result<(), (ErrorCode, String)> {
    if let Some(expected) = asked.expected_revision
        && expected != revision
    {
        return Err((
            ErrorCode::Conflict,
            format!(
                "the document moved since it was reviewed: revision {expected} was approved, \
                 it is now at revision {revision}"
            ),
        ));
    }
    if crate::export::naming_pattern(&asked.naming_pattern).is_err() {
        return Err((
            ErrorCode::InvalidArgument,
            "the naming pattern cannot be resolved".to_owned(),
        ));
    }
    if let Some(token) = asked
        .ai_assistance
        .iter()
        .find(|token| !crate::render::ai_assistance_is_known(token))
    {
        return Err((
            ErrorCode::InvalidArgument,
            format!("an export declared a disclosure token nobody recognises: {token}"),
        ));
    }
    crate::render::output_request(asked.format.as_ref())
        .map_err(|message| (ErrorCode::InvalidArgument, message))?;
    Ok(())
}

/// The file name an export would take, resolved before any render exists.
fn planned_stem(
    pattern: &clipmill_export::Pattern,
    asked: &ExportRequestV1,
    document: &clipmill_edit_ir::EditDocument,
) -> String {
    pattern.resolve(&clipmill_export::Fields {
        project: String::new(),
        clip: asked.title.clone(),
        index: asked.index.max(1),
        duration_seconds: u64::try_from(document.program_duration_ticks().max(0) / 90_000)
            .unwrap_or(0),
        date: asked.date.clone(),
        // Resolved before a render exists, so `{address}` has nothing to
        // shorten yet and the preview says so rather than inventing one.
        address: String::new(),
    })
}

/// The profile an export would render with: the chosen height, and the chosen
/// rate or else the rate of the recording the clip opens on.
fn output_profile(
    document: &clipmill_edit_ir::EditDocument,
    sources: &[crate::db::SourceRecord],
    height: i64,
    chosen_rate: Option<clipmill_render::FrameRateSpec>,
) -> clipmill_render::RenderProfile {
    let source_rate = document.video.segments.first().and_then(|first| {
        sources
            .iter()
            .find(|source| source.source_fingerprint == first.source_fingerprint)
            .and_then(|source| crate::inspector::frame_rate_of(&source.source_map_json))
    });
    let rate = chosen_rate
        .or(source_rate)
        .unwrap_or(clipmill_render::RenderProfile::default().frame_rate);
    clipmill_render::RenderProfile::for_output(height, rate).unwrap_or_default()
}

/// The size estimate, scaled from the 1080 × 1920 rate it is stated at by the
/// picture's area and by frame rates above thirty.
fn scaled_estimate(duration_ticks: i64, profile: &clipmill_render::RenderProfile) -> u64 {
    let base = clipmill_export::estimate_bytes(duration_ticks);
    let default = clipmill_render::RenderProfile::default();
    let area = u64::try_from(profile.width * profile.height).unwrap_or(1);
    let default_area = u64::try_from(default.width * default.height).unwrap_or(1);
    let fast = profile.frame_rate.num > 31 * profile.frame_rate.den;
    base.saturating_mul(area)
        .checked_div(default_area.max(1))
        .unwrap_or(base)
        .saturating_mul(if fast { 3 } else { 2 })
        / 2
}

/// Advice when the chosen size enlarges the recording more than twice over.
fn upscale_finding(
    document: &clipmill_edit_ir::EditDocument,
    sources: &[crate::db::SourceRecord],
    profile: &clipmill_render::RenderProfile,
) -> Option<ExportFindingV1> {
    let inputs = sources
        .iter()
        .filter_map(|source| {
            let frame = crate::inspector::frame_of(&source.source_map_json)?;
            Some(clipmill_render::SourceInput {
                fingerprint: source.source_fingerprint.clone(),
                path: String::new(),
                width: frame.width,
                height: frame.height,
                has_audio: true,
                duration_ticks: 0,
                keyframe_ticks: Vec::new(),
            })
        })
        .collect::<Vec<_>>();
    let factor = clipmill_render::largest_upscale(document, &inputs, profile)?;
    (factor > 2.0).then(|| ExportFindingV1 {
        code: "format.upscaled".to_owned(),
        severity: ExportSeverity::Advisory as i32,
        detail: format!(
            "At {}p this clip enlarges the recording up to {factor:.1} times, so it will not look sharper than a smaller export. It still exports.",
            profile.width
        ),
        cue_id: String::new(),
    })
}

/// A source as a reply states it, including whether its file is still there.
fn source_reply(record: crate::db::SourceRecord) -> clipmill_contracts::proto::ipc::v1::Source {
    let missing = !source_file_present(&record.observation);
    let mut source: clipmill_contracts::proto::ipc::v1::Source = record.into();
    source.missing = missing;
    source
}

/// Whether the registered file is still where it was, at the size it had.
///
/// A size check rather than a fingerprint: this runs on every listing, and a
/// different size is already proof of a different file. Relinking is where
/// the whole fingerprint is compared.
fn source_file_present(observation: &crate::sources::FileObservation) -> bool {
    std::fs::metadata(&observation.absolute_path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.len() == observation.byte_size)
}

/// The recordings a document cuts from that are not where they were
/// registered, by path, so an export can say which one to find.
fn missing_recordings(
    document: &clipmill_edit_ir::EditDocument,
    sources: &[crate::db::SourceRecord],
) -> Vec<String> {
    let mut missing = Vec::new();
    for source in sources {
        let used = document
            .video
            .segments
            .iter()
            .any(|segment| segment.source_fingerprint == source.source_fingerprint);
        if used
            && !source_file_present(&source.observation)
            && !missing.contains(&source.observation.absolute_path)
        {
            missing.push(source.observation.absolute_path.clone());
        }
    }
    missing
}

fn missing_recording_finding(path: &str) -> ExportFindingV1 {
    let name = std::path::Path::new(path).file_name().map_or_else(
        || path.to_owned(),
        |name| name.to_string_lossy().into_owned(),
    );
    ExportFindingV1 {
        code: "source.missing".to_owned(),
        severity: ExportSeverity::Blocking as i32,
        detail: format!(
            "The recording {name} is no longer where it was imported from. Locate it to export this clip."
        ),
        cue_id: String::new(),
    }
}

/// Free space where an export would land, or on the nearest folder above it
/// that exists.
///
/// A destination the user has not created yet still has an answer: it will be
/// created on the volume its parent is on, and that is the volume the export
/// has to fit into.
fn available_at(raw: &str) -> Option<u64> {
    let mut candidate = std::path::Path::new(raw);
    loop {
        if candidate.exists() {
            return fs2::available_space(candidate).ok();
        }
        candidate = candidate.parent()?;
    }
}

fn validation_of(report: &clipmill_export::Report) -> ExportValidationV1 {
    ExportValidationV1 {
        passes: report.passes(),
        findings: report
            .findings
            .iter()
            .map(|finding| ExportFindingV1 {
                code: finding.code.clone(),
                severity: match finding.severity {
                    clipmill_export::Severity::Blocking => ExportSeverity::Blocking as i32,
                    clipmill_export::Severity::Advisory => ExportSeverity::Advisory as i32,
                },
                detail: finding.detail.clone(),
                cue_id: finding.cue_id.clone().unwrap_or_default(),
            })
            .collect(),
    }
}

/// Every file an export writes, in delivery order.
///
/// Built from the same roles the delivery iterates, so a preview cannot list a
/// file the export does not write or miss one it does.
fn delivered_names(stem: &str) -> Vec<String> {
    crate::export::DELIVERED_ROLES
        .iter()
        .map(|role| format!("{stem}.{}", role.extension()))
        .collect()
}

/// Turn the durable job submission receipt into its public export receipt.
/// A retry must never invent a new job id beside the job SQLite already owns.
fn export_submission_reply(request_id: String, bytes: &[u8]) -> Reply {
    let Ok(response) = Response::decode(bytes) else {
        return error_reply(
            request_id,
            ErrorCode::Internal,
            "saved export response is invalid",
        );
    };
    match response.body {
        Some(response::Body::SubmitJob(submitted)) => {
            let Some(job) = submitted.job else {
                return error_reply(request_id, ErrorCode::Internal, "saved export has no job");
            };
            let Some(export) = job.export else {
                return error_reply(
                    request_id,
                    ErrorCode::Conflict,
                    "request id belongs to another operation",
                );
            };
            response_reply(
                request_id,
                response::Body::ExportClip(ExportClipResponse {
                    job_id: job.job_id,
                    revision: export.revision,
                    ir_artifact_id: export.ir_artifact_id,
                    destination_dir: export.destination_dir,
                }),
            )
        }
        Some(response::Body::ExportClip(export)) => {
            response_reply(request_id, response::Body::ExportClip(export))
        }
        _ => error_reply(
            request_id,
            ErrorCode::Conflict,
            "request id belongs to another operation",
        ),
    }
}

/// Make captions derived for a newly included span fit the document they join.
///
/// The caption engine numbers a derivation's cues from one, and the clip being
/// extended already has cues with those ids, so each new cue gets an id no cue
/// in that list has — derived from the span, so a replay of the stored command
/// is the same edit. A cue whose hold ran past the span is ended at the span's
/// edge, where the clip's existing captions begin or end.
fn fit_extension_cues(
    existing: &[clipmill_edit_ir::CaptionCue],
    incoming: Vec<clipmill_edit_ir::CaptionCue>,
    span_ticks: i64,
    span_start: i64,
) -> Vec<clipmill_edit_ir::CaptionCue> {
    let mut taken: std::collections::HashSet<String> =
        existing.iter().map(|cue| cue.cue_id.clone()).collect();
    incoming
        .into_iter()
        .filter(|cue| cue.start_ticks < span_ticks)
        .enumerate()
        .map(|(index, mut cue)| {
            let last_word = cue.words().map(|word| word.end_ticks).max().unwrap_or(0);
            cue.end_ticks = cue.end_ticks.min(span_ticks).max(last_word);
            let mut id = format!("cue_x{span_start}_{}", index + 1);
            let mut suffix = 2;
            while taken.contains(&id) {
                id = format!("cue_x{span_start}_{}_{suffix}", index + 1);
                suffix += 1;
            }
            taken.insert(id.clone());
            cue.cue_id = id;
            cue
        })
        .collect()
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

    /// Two people side by side for ten seconds at four frames a second: the
    /// left one at a quarter of the frame, the right at three quarters.
    fn two_people() -> clipmill_contracts::schemas::vision_face_track::VisionFaceTrack {
        let track = |id: u64, x: f64| {
            let boxes: Vec<_> = (0..40u64)
                .map(|frame| serde_json::json!({ "t_ticks": frame * 22_500, "x": x, "y": 0.3, "w": 0.1, "h": 0.2, "score": 0.95 }))
                .collect();
            serde_json::json!({ "track_id": id, "first_ticks": 0, "last_ticks": 39 * 22_500, "frames_present": 40, "mean_score": 0.95, "boxes": boxes })
        };
        serde_json::from_value(serde_json::json!({
            "schema_version": "clipmill.vision.face_track.v1",
            "source_fingerprint": format!("sha256:{}", "1".repeat(64)),
            "frames_artifact_id": format!("sha256:{}", "2".repeat(64)),
            "producer": { "stage": "detect-faces", "implementation": "fixture" },
            "coverage": { "start_ticks": 0, "end_ticks": 900_000, "analyzed": true },
            "detection": { "score_threshold": 0.6, "nms_iou": 0.3, "input_width": 320, "input_height": 320, "match_iou": 0.5, "recover_iou": 0.3, "max_gap_frames": 6, "min_track_frames": 4, "frame_rate": { "num": 4, "den": 1 } },
            "tracks": [track(0, 0.2), track(1, 0.7)],
        }))
        .expect("a face track")
    }

    fn ask(adjust: impl FnOnce(&mut super::SolveCropPathRequest)) -> super::SolveCropPathRequest {
        let mut solve = super::SolveCropPathRequest {
            start_ticks: 0,
            end_ticks: 900_000,
            aspect_width: 9,
            aspect_height: 16,
            ..super::SolveCropPathRequest::default()
        };
        adjust(&mut solve);
        solve
    }

    #[test]
    fn two_people_alike_leave_the_automatic_camera_fitted() {
        let solved = super::crop_solve(two_people(), &ask(|_| {}), (1_920, 1_080)).expect("solve");
        assert!(solved.fit, "neither face is more worth following");
        assert!(!solved.fit_reason.is_empty());
    }

    #[test]
    fn a_face_somebody_picked_is_followed() {
        for (track, side) in [(0u32, 0.25), (1, 0.75)] {
            let solved = super::crop_solve(
                two_people(),
                &ask(|solve| {
                    solve.follow_track = true;
                    solve.track_id = track;
                }),
                (1_920, 1_080),
            )
            .expect("solve");
            assert!(!solved.fit, "{}", solved.fit_reason);
            assert!(solved.has_track && solved.track_id == track);
            assert!((solved.keyframes[0].center_x - side).abs() < 0.05);
        }
        let absent = super::crop_solve(
            two_people(),
            &ask(|solve| {
                solve.follow_track = true;
                solve.track_id = 7;
            }),
            (1_920, 1_080),
        )
        .expect("solve");
        assert!(absent.fit && absent.fit_reason.contains("not in this recording"));
    }

    #[test]
    fn a_two_up_solve_returns_both_portraits_with_the_left_face_on_top() {
        let solved = super::crop_solve(
            two_people(),
            &ask(|solve| solve.two_up = true),
            (1_920, 1_080),
        )
        .expect("solve");
        assert!(!solved.fit, "{}", solved.fit_reason);
        assert!(!solved.keyframes.is_empty() && !solved.secondary_keyframes.is_empty());
        assert!(solved.keyframes[0].center_x < solved.secondary_keyframes[0].center_x);
        assert_eq!((solved.track_id, solved.secondary_track_id), (0, 1));

        let mut alone = two_people();
        alone.tracks.truncate(1);
        let refused = super::crop_solve(alone, &ask(|solve| solve.two_up = true), (1_920, 1_080))
            .expect("solve");
        assert!(refused.fit && refused.secondary_keyframes.is_empty());
        assert!(refused.fit_reason.contains("two people"));
    }

    #[test]
    fn an_export_may_ask_for_the_offered_sizes_and_rates_only() {
        use clipmill_contracts::proto::ipc::v1::OutputFormatV1;
        let ask = |num, den, height| {
            crate::render::output_request(Some(&OutputFormatV1 {
                frame_rate_num: num,
                frame_rate_den: den,
                height,
            }))
        };
        assert_eq!(crate::render::output_request(None), Ok((1_920, None)));
        assert_eq!(ask(0, 0, 0), Ok((1_920, None)));
        let (height, rate) = ask(60, 1, 3_840).expect("4K at sixty");
        assert_eq!(height, 3_840);
        assert_eq!(rate.map(|rate| (rate.num, rate.den)), Some((60, 1)));
        assert!(ask(0, 0, 1_000).is_err(), "an odd height");
        assert!(ask(1_000, 1, 0).is_err(), "an absurd rate");
        assert!(ask(30, 0, 0).is_err(), "a rate with no denominator");
    }

    #[test]
    fn a_moved_recording_is_named_by_the_export_checks() {
        use crate::{db::SourceRecord, sources::FileObservation};
        use clipmill_edit_ir::{EditDocument, Layout, LayoutState, VideoSegment};
        let temp = TempDir::new().expect("tempdir");
        let present = temp.path().join("present.mov");
        std::fs::write(&present, b"12345").expect("recording");
        let record = |path: &std::path::Path, fingerprint: &str, size: u64| SourceRecord {
            source_id: "src_1".to_owned(),
            project_id: "prj_1".to_owned(),
            observation: FileObservation {
                absolute_path: path.to_string_lossy().into_owned(),
                byte_size: size,
                sample_sha256: String::new(),
                device_id: 0,
                inode: 0,
                modified_unix_nanos: 0,
            },
            source_fingerprint: fingerprint.to_owned(),
            source_map_json: Vec::new(),
            source_map_artifact_id: String::new(),
            created_unix_millis: 0,
        };
        let mut document = EditDocument::default();
        document.video.segments = vec![VideoSegment {
            segment_id: "seg".to_owned(),
            source_fingerprint: "sha256:aa".to_owned(),
            in_ticks: 0,
            out_ticks: 90_000,
            layout: Layout {
                state: LayoutState::Fit,
                crop_path: Vec::new(),
                secondary_crop_path: Vec::new(),
                ..Layout::default()
            },
        }];
        let here = [record(&present, "sha256:aa", 5)];
        assert!(super::missing_recordings(&document, &here).is_empty());
        // Replaced by a different file of another size: not the recording.
        let changed = [record(&present, "sha256:aa", 6)];
        assert_eq!(super::missing_recordings(&document, &changed).len(), 1);
        let gone = [record(&temp.path().join("gone.mov"), "sha256:aa", 5)];
        let missing = super::missing_recordings(&document, &gone);
        assert_eq!(missing.len(), 1);
        let finding = super::missing_recording_finding(&missing[0]);
        assert_eq!(finding.code, "source.missing");
        assert!(finding.detail.contains("gone.mov"), "{}", finding.detail);
        // A recording the clip does not use is none of the export's business.
        let unrelated = [record(&temp.path().join("gone.mov"), "sha256:bb", 5)];
        assert!(super::missing_recordings(&document, &unrelated).is_empty());
    }

    #[test]
    fn extension_cues_never_reuse_an_id_and_stay_inside_their_span() {
        use clipmill_edit_ir::{
            CaptionAnimation, CaptionCue, CaptionLine, CaptionRegion, CaptionWord,
        };
        let cue = |id: &str, start: i64, end: i64| CaptionCue {
            cue_id: id.to_owned(),
            start_ticks: start,
            end_ticks: end,
            region: CaptionRegion::LowerSafe,
            anim: CaptionAnimation::Karaoke,
            lines: vec![CaptionLine {
                words: vec![CaptionWord {
                    word_id: Some(format!("w{start}")),
                    text: "word".to_owned(),
                    start_ticks: start,
                    end_ticks: start + 10_000,
                    emphasis: false,
                }],
            }],
            position: None,
        };
        let existing = [cue("cue_1", 0, 60_000), cue("cue_x500_2", 60_000, 90_000)];
        let fitted = super::fit_extension_cues(
            &existing,
            vec![cue("cue_1", 0, 40_000), cue("cue_2", 45_000, 120_000)],
            90_000,
            500,
        );
        let ids: Vec<_> = fitted.iter().map(|cue| cue.cue_id.as_str()).collect();
        assert_eq!(ids, ["cue_x500_1", "cue_x500_2_2"]);
        assert_eq!(
            fitted[1].end_ticks, 90_000,
            "a hold past the span ends at its edge"
        );
    }

    #[test]
    fn editorial_readiness_requires_verified_gpu_and_enough_memory() {
        let ready = || {
            super::stage_readiness(
                "editorial-propose",
                "editorial",
                "worker",
                "qwen",
                "mlx",
                true,
                vec![],
                true,
            )
        };
        let mut capacity = crate::jobs::ResourceCapacity::measured(4, 16 << 30, 10 << 30);
        let mut stage = ready();
        super::check_editorial_capacity(&mut stage, Some(capacity), Some(8 << 30));
        assert!(!stage.ready);
        assert!(stage.remedy.contains("runtime check"));
        capacity.accelerator_mask = crate::jobs::accelerator_bit("metal").unwrap();
        capacity.ram_bytes = 4 << 30;
        let mut stage = ready();
        super::check_editorial_capacity(&mut stage, Some(capacity), Some(8 << 30));
        assert!(!stage.ready);
        assert!(stage.remedy.contains("8192 MiB"));
        capacity.ram_bytes = 10 << 30;
        let mut stage = ready();
        super::check_editorial_capacity(&mut stage, Some(capacity), Some(8 << 30));
        assert!(stage.ready);
    }

    /// A worker is only useful for a model it will be handed: one within its
    /// declared ceiling, while that ceiling is within the device's budget.
    #[test]
    fn readiness_names_a_worker_that_can_never_take_the_chosen_model() {
        let ready = || {
            super::stage_readiness(
                "speech-asr",
                "asr",
                "clipmill-worker-asr@0.1.0/whisper-large-v3-turbo",
                "whisper-large-v3-turbo",
                "cpu",
                true,
                vec![],
                true,
            )
        };
        let roster = |ceiling: u64| {
            std::collections::BTreeMap::from([(
                "wrk".to_owned(),
                crate::worker::WorkerPresence {
                    family: "speech-asr".to_owned(),
                    capabilities: vec!["speech-asr".to_owned()],
                    backend: "cpu".to_owned(),
                    since_unix_millis: 1,
                    max_memory_bytes: ceiling,
                },
            )])
        };

        let mut small = ready();
        super::check_worker_ceiling(
            &mut small,
            &roster(768 << 20),
            Some(3 << 30),
            Some(12 << 30),
        );
        assert!(!small.ready);
        assert!(
            small.remedy.contains("accepts models up to 768 MiB"),
            "{}",
            small.remedy
        );

        let mut over = ready();
        super::check_worker_ceiling(&mut over, &roster(16 << 30), Some(3 << 30), Some(12 << 30));
        assert!(!over.ready, "a worker above the budget is never given work");
        assert!(over.remedy.contains("processing budget"), "{}", over.remedy);

        let mut sized = ready();
        super::check_worker_ceiling(&mut sized, &roster(8 << 30), Some(3 << 30), Some(12 << 30));
        assert!(sized.ready);
    }

    /// The case a person met: Word timing set to the MLX aligner while only
    /// the ONNX aligner's worker runs. That worker declares the stage but is
    /// never handed the task, so the stage is not ready — and the sentence
    /// names the worker to start rather than blaming its memory.
    #[test]
    fn readiness_names_the_worker_family_a_chosen_model_needs() {
        let aligner = crate::worker::WorkerPresence {
            family: "speech-align".to_owned(),
            capabilities: vec!["speech-align".to_owned()],
            backend: "onnx-cpu".to_owned(),
            since_unix_millis: 1,
            max_memory_bytes: 1152 << 20,
        };
        let mlx = "clipmill-worker-speech-mlx@0.1.0/align";
        assert!(aligner.serves("speech-align"));
        assert!(!aligner.runs("speech-align", mlx));
        let mut stage = super::stage_readiness(
            "speech-align",
            "forced-align",
            mlx,
            "qwen3-aligner-mlx",
            "mlx",
            true,
            vec![],
            aligner.runs("speech-align", mlx),
        );
        let roster = std::collections::BTreeMap::from([("wrk".to_owned(), aligner)]);
        super::check_worker_ceiling(&mut stage, &roster, Some(1955 << 20), Some(12 << 30));
        assert!(!stage.ready);
        assert!(
            stage.remedy.contains("runs in the MLX speech worker"),
            "{}",
            stage.remedy
        );
        assert!(!stage.remedy.contains("MiB"), "{}", stage.remedy);

        let portable = super::stage_readiness(
            "speech-align",
            "forced-align",
            "clipmill-worker-align@0.1.0",
            "wav2vec2-ctc-en",
            "onnx-cpu",
            true,
            vec![],
            roster
                .values()
                .any(|worker| worker.runs("speech-align", "clipmill-worker-align@0.1.0")),
        );
        assert!(portable.ready, "its own model is still served");
    }

    #[test]
    fn optional_cloud_workers_do_not_block_local_analysis_readiness() {
        let mut stages = vec![
            super::stage_readiness(
                "speech-asr",
                "speech",
                "asr",
                "model",
                "cpu",
                true,
                vec![],
                true,
            ),
            super::stage_readiness(
                "editorial-propose-cloud",
                "",
                "",
                "",
                "",
                true,
                vec![],
                false,
            ),
            super::stage_readiness(
                "editorial-review-cloud",
                "",
                "",
                "",
                "",
                true,
                vec![],
                false,
            ),
        ];
        assert!(super::local_analysis_stages_ready(&stages));
        assert!(stages[1].remedy.contains("--cloud-editorial"));
        assert!(stages[2].remedy.contains("explicit cloud consent"));
        stages[0].ready = false;
        assert!(!super::local_analysis_stages_ready(&stages));
    }

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
