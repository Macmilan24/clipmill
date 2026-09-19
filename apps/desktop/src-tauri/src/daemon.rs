//! The shell's link to `clipmilld`.
//!
//! The WebView never speaks to the daemon. It has no socket, no filesystem and
//! no network capability; it can only call the commands this host process
//! exposes. Everything below — framing, the request/response envelope, process
//! supervision — runs in Rust so that the renderer stays a pure view layer.

use std::{
    path::{Path, PathBuf},
    sync::{
        LazyLock,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use clipmill_contracts::proto::ipc::v1::{
    AnalyzeSourcePayloadV1, ApplyEditCommandRequest, ApplyEditCommandResponse,
    ClipDecisionRecordV1, CreateProjectRequest, DemoDagPayloadV1, DirectClipRequest,
    DirectClipResponse, EditDoc, ExportArchiveRequest, ExportArchiveResponse, ExportBatchV1,
    ExportClipRequest, ExportClipResponse, ExportRequestV1, GetDeviceProfileRequest,
    GetDeviceProfileResponse, GetJobRequest, GetLocalLockRequest, GetLocalLockResponse,
    GetPreviewPlanRequest, GetPreviewPlanResponse, GetReadinessRequest, GetReadinessResponse,
    GetStorageStatsRequest, GetStorageStatsResponse, HealthRequest, HealthResponse, Job,
    ListClipDecisionsRequest, ListEditDocsRequest, ListExportBatchesRequest, ListJobsRequest,
    ListProjectsRequest, ListSourcesRequest, PlanExportRequest, PlanExportResponse, Project,
    ReadArtifactRequest, ReadArtifactResponse, RegisterSourceRequest, RegisterSourceResponse,
    Request, ResolveMediaRequest, ResolveMediaResponse, Response, SetClipDecisionRequest,
    SetClipDecisionResponse, SolveCropPathRequest, SolveCropPathResponse, Source,
    SubmitExportBatchRequest, SubmitJobRequest, SubscribeTaskEventsRequest, TaskEvent,
    UpdateExportBatchItemRequest, request, response,
};
use prost::Message;
use serde::Serialize;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
    process::{Child, Command},
    sync::{Mutex, RwLock, Semaphore},
    time::{sleep, timeout},
};

/// Frames larger than this are refused rather than allocated: the socket is
/// trusted, but a corrupted length prefix should not become an OOM.
const MAX_FRAME_BYTES: usize = 8 * 1024 * 1024;
const CALL_TIMEOUT: Duration = Duration::from_secs(10);
/// How long a freshly spawned daemon gets to open its socket.
const STARTUP_TIMEOUT: Duration = Duration::from_secs(20);
const STARTUP_POLL_INTERVAL: Duration = Duration::from_millis(150);
/// How many calls this process keeps open to the daemon at once.
///
/// Every call is its own connection, and the daemon accepts sixty-four in
/// all — for this shell, its event subscription, the workers' side, and
/// anything else on the socket. A screen that opens with a card per project
/// asks for several documents and a thumbnail per card in one burst, and a
/// burst past the limit was dropped at the door: cards blank, documents
/// unread, nothing said. The burst now queues here instead, well under the
/// daemon's ceiling, so nothing this process sends is refused for being
/// sent together.
const CALLS_IN_FLIGHT: usize = 16;
/// The pause before a call is sent once more after the daemon closed the
/// connection without answering.
const RETRY_AFTER: Duration = Duration::from_millis(100);

static REQUEST_IDS: LazyLock<RequestIds> = LazyLock::new(RequestIds::new);
static CALLS: Semaphore = Semaphore::const_new(CALLS_IN_FLIGHT);

/// The daemon persists mutation receipts across shell restarts. A counter alone
/// reuses an old receipt's key when the next shell process begins at zero.
struct RequestIds {
    session: ulid::Ulid,
    counter: AtomicU64,
}

impl RequestIds {
    fn new() -> Self {
        Self {
            session: ulid::Ulid::new(),
            counter: AtomicU64::new(0),
        }
    }

    fn next(&self, prefix: &str) -> String {
        format!(
            "{prefix}-{}-{}",
            self.session,
            self.counter.fetch_add(1, Ordering::Relaxed)
        )
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DaemonLinkError {
    #[error("daemon socket is unavailable: {0}")]
    Unavailable(#[source] std::io::Error),
    #[error("daemon connection failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("daemon frame was malformed: {0}")]
    Decode(#[from] prost::DecodeError),
    #[error("daemon closed the connection before answering")]
    Closed,
    #[error("daemon frame exceeds {MAX_FRAME_BYTES} bytes")]
    Oversized,
    #[error("daemon did not answer within {}s", CALL_TIMEOUT.as_secs())]
    TimedOut,
    #[error("daemon answered a different request")]
    Mismatched,
    #[error("daemon returned an empty response body")]
    Empty,
    #[error("daemon returned an unexpected response body")]
    Unexpected,
    #[error("daemon error: {0}")]
    Remote(String),
    #[error("cannot locate the clipmilld executable")]
    MissingBinary,
}

/// Whether the daemon hung up on a call rather than answering it: the
/// connection was accepted and then closed, which reads as an end of stream
/// or a broken pipe depending on which side got there first.
fn dropped_by_daemon(error: &DaemonLinkError) -> bool {
    match error {
        DaemonLinkError::Closed => true,
        DaemonLinkError::Io(io) => matches!(
            io.kind(),
            std::io::ErrorKind::BrokenPipe | std::io::ErrorKind::ConnectionReset
        ),
        _ => false,
    }
}

fn encode_varint(mut value: u64, out: &mut Vec<u8>) {
    loop {
        // Masked to seven bits, so the narrowing cast is exact.
        #[allow(clippy::cast_possible_truncation)]
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if value == 0 {
            return;
        }
    }
}

async fn write_frame(stream: &mut UnixStream, payload: &[u8]) -> Result<(), DaemonLinkError> {
    let mut frame = Vec::with_capacity(payload.len() + 10);
    encode_varint(payload.len() as u64, &mut frame);
    frame.extend_from_slice(payload);
    stream.write_all(&frame).await?;
    stream.flush().await?;
    Ok(())
}

async fn read_frame(stream: &mut UnixStream) -> Result<Vec<u8>, DaemonLinkError> {
    let mut length = 0_u64;
    let mut shift = 0_u32;
    for index in 0..10_u32 {
        let mut byte = [0_u8; 1];
        if stream.read(&mut byte).await? == 0 {
            return Err(DaemonLinkError::Closed);
        }
        let value = byte[0];
        length |= u64::from(value & 0x7f) << shift;
        if value & 0x80 == 0 {
            let length = usize::try_from(length).map_err(|_| DaemonLinkError::Oversized)?;
            if length > MAX_FRAME_BYTES {
                return Err(DaemonLinkError::Oversized);
            }
            let mut payload = vec![0_u8; length];
            stream.read_exact(&mut payload).await?;
            return Ok(payload);
        }
        shift += 7;
        if index == 9 {
            break;
        }
    }
    Err(DaemonLinkError::Oversized)
}

/// A stateless request/response client. Each call opens its own connection:
/// the control plane is low-frequency, and a fresh socket means a half-dead
/// connection can never wedge the UI.
#[derive(Debug, Clone)]
pub struct DaemonClient {
    socket: PathBuf,
}

impl DaemonClient {
    pub fn new(socket: PathBuf) -> Self {
        Self { socket }
    }

    pub fn socket(&self) -> &Path {
        &self.socket
    }

    /// Apply one command to a document.
    ///
    /// The reply carries the inverse. The daemon keeps no undo stack — it
    /// hands the inverse back and logs it durably beside the command, so an
    /// undo is just another command and survives a restart the same way.
    pub async fn apply_edit_command(
        &self,
        doc_id: &str,
        expected_revision: u64,
        command_json: String,
    ) -> Result<ApplyEditCommandResponse, DaemonLinkError> {
        let request = ApplyEditCommandRequest {
            doc_id: doc_id.to_owned(),
            expected_revision,
            command_json,
        };
        match self.call(request::Body::ApplyEditCommand(request)).await? {
            response::Body::ApplyEditCommand(reply) => Ok(reply),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    /// Every edit document a project holds, oldest first.
    pub async fn list_edit_docs(&self, project_id: &str) -> Result<Vec<EditDoc>, DaemonLinkError> {
        let request = ListEditDocsRequest {
            project_id: project_id.to_owned(),
        };
        match self.call(request::Body::ListEditDocs(request)).await? {
            response::Body::ListEditDocs(reply) => Ok(reply.docs),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    /// What the player must draw for a document.
    pub async fn preview_plan(
        &self,
        project_id: &str,
        doc_id: &str,
    ) -> Result<GetPreviewPlanResponse, DaemonLinkError> {
        let request = GetPreviewPlanRequest {
            project_id: project_id.to_owned(),
            doc_id: doc_id.to_owned(),
        };
        match self.call(request::Body::GetPreviewPlan(request)).await? {
            response::Body::GetPreviewPlan(reply) => Ok(reply),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    /// What an export would do. Writes nothing, so a screen may ask again
    /// every time the naming pattern changes under a keystroke.
    pub async fn plan_export(
        &self,
        request: ExportRequestV1,
    ) -> Result<PlanExportResponse, DaemonLinkError> {
        let plan = PlanExportRequest {
            request: Some(request),
        };
        match self.call(request::Body::PlanExport(plan)).await? {
            response::Body::PlanExport(reply) => Ok(reply),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    /// Perform an export. Returns the job to watch, not the finished files.
    pub async fn export_clip(
        &self,
        request: ExportRequestV1,
    ) -> Result<ExportClipResponse, DaemonLinkError> {
        let export = ExportClipRequest {
            request: Some(request),
        };
        match self.call(request::Body::ExportClip(export)).await? {
            response::Body::ExportClip(reply) => Ok(reply),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    pub async fn submit_export_batch(
        &self,
        requests: Vec<ExportRequestV1>,
    ) -> Result<ExportBatchV1, DaemonLinkError> {
        match self
            .call(request::Body::SubmitExportBatch(SubmitExportBatchRequest {
                requests,
            }))
            .await?
        {
            response::Body::ExportBatch(reply) => reply.batch.ok_or(DaemonLinkError::Empty),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    pub async fn list_export_batches(&self) -> Result<Vec<ExportBatchV1>, DaemonLinkError> {
        match self
            .call(request::Body::ListExportBatches(
                ListExportBatchesRequest {},
            ))
            .await?
        {
            response::Body::ListExportBatches(reply) => Ok(reply.batches),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    pub async fn update_export_batch_item(
        &self,
        batch_id: String,
        index: u32,
        action: String,
    ) -> Result<ExportBatchV1, DaemonLinkError> {
        match self
            .call(request::Body::UpdateExportBatchItem(
                UpdateExportBatchItemRequest {
                    batch_id,
                    index,
                    action,
                },
            ))
            .await?
        {
            response::Body::ExportBatch(reply) => reply.batch.ok_or(DaemonLinkError::Empty),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    /// Pack a project's work into a zip that outlives this application.
    pub async fn export_archive(
        &self,
        project_id: &str,
        destination_dir: &str,
    ) -> Result<ExportArchiveResponse, DaemonLinkError> {
        let request = ExportArchiveRequest {
            project_id: project_id.to_owned(),
            destination_dir: destination_dir.to_owned(),
        };
        match self.call(request::Body::ExportArchive(request)).await? {
            response::Body::ExportArchive(reply) => Ok(reply),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    /// Whether this installation is offline, with the evidence behind it.
    /// Whether an analysis could run right now, and what each stage lacks.
    pub async fn readiness(&self) -> Result<GetReadinessResponse, DaemonLinkError> {
        match self
            .call(request::Body::GetReadiness(GetReadinessRequest {}))
            .await?
        {
            response::Body::GetReadiness(reply) => Ok(reply),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    pub async fn local_lock(&self) -> Result<GetLocalLockResponse, DaemonLinkError> {
        match self
            .call(request::Body::GetLocalLock(GetLocalLockRequest {}))
            .await?
        {
            response::Body::GetLocalLock(reply) => Ok(reply),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    /// A crop path proposal for a span. Writes nothing: it is arithmetic over
    /// evidence that already exists, which is why the Inspector may ask for one
    /// every time somebody moves a boundary.
    pub async fn solve_crop_path(
        &self,
        request: SolveCropPathRequest,
    ) -> Result<SolveCropPathResponse, DaemonLinkError> {
        match self.call(request::Body::SolveCropPath(request)).await? {
            response::Body::SolveCropPath(reply) => Ok(reply),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    /// Turn an approved candidate into an edit document.
    pub async fn direct_clip(
        &self,
        request: DirectClipRequest,
    ) -> Result<DirectClipResponse, DaemonLinkError> {
        match self.call(request::Body::DirectClip(request)).await? {
            response::Body::DirectClip(reply) => Ok(reply),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    /// Record what somebody decided about a clip.
    pub async fn set_clip_decision(
        &self,
        request: SetClipDecisionRequest,
    ) -> Result<SetClipDecisionResponse, DaemonLinkError> {
        match self.call(request::Body::SetClipDecision(request)).await? {
            response::Body::SetClipDecision(reply) => Ok(reply),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    pub async fn list_clip_decisions(
        &self,
        project_id: &str,
        source_id: &str,
    ) -> Result<Vec<ClipDecisionRecordV1>, DaemonLinkError> {
        let request = ListClipDecisionsRequest {
            project_id: project_id.to_owned(),
            source_id: source_id.to_owned(),
        };
        match self.call(request::Body::ListClipDecisions(request)).await? {
            response::Body::ListClipDecisions(reply) => Ok(reply.decisions),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    async fn call(&self, body: request::Body) -> Result<response::Body, DaemonLinkError> {
        let request_id = REQUEST_IDS.next("shell");
        let envelope = Request {
            request_id: request_id.clone(),
            body: Some(body),
        };
        let frame = envelope.encode_to_vec();

        // Queued behind the calls already in flight rather than sent into a
        // daemon that would drop it. The permit is held for the whole
        // exchange, reply included.
        let _permit = CALLS.acquire().await.map_err(|_| DaemonLinkError::Closed)?;
        let exchange = || async {
            let mut stream = UnixStream::connect(&self.socket)
                .await
                .map_err(DaemonLinkError::Unavailable)?;
            write_frame(&mut stream, &frame).await?;
            let payload = read_frame(&mut stream).await?;
            Response::decode(payload.as_slice()).map_err(DaemonLinkError::from)
        };
        let mut response = timeout(CALL_TIMEOUT, exchange())
            .await
            .map_err(|_| DaemonLinkError::TimedOut)?;
        if response.as_ref().is_err_and(dropped_by_daemon) {
            // A connection closed before it answered is the daemon at its
            // limit, or restarting under this call. The same envelope goes
            // once more, under the same request id: every mutation the
            // daemon accepts is stored under that id, so a retry of a lost
            // reply is the same reply and never a second edit.
            sleep(RETRY_AFTER).await;
            response = timeout(CALL_TIMEOUT, exchange())
                .await
                .map_err(|_| DaemonLinkError::TimedOut)?;
        }
        let response = response?;

        if response.request_id != request_id {
            return Err(DaemonLinkError::Mismatched);
        }
        match response.body {
            Some(response::Body::Error(error)) => Err(DaemonLinkError::Remote(error.message)),
            Some(body) => Ok(body),
            None => Err(DaemonLinkError::Empty),
        }
    }

    pub async fn health(&self) -> Result<HealthResponse, DaemonLinkError> {
        match self.call(request::Body::Health(HealthRequest {})).await? {
            response::Body::Health(health) => Ok(health),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    pub async fn device_profile(
        &self,
        remeasure: bool,
    ) -> Result<GetDeviceProfileResponse, DaemonLinkError> {
        let body = request::Body::GetDeviceProfile(GetDeviceProfileRequest { remeasure });
        match self.call(body).await? {
            response::Body::GetDeviceProfile(profile) => Ok(profile),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    /// One window of a published document. The daemon decides which file the
    /// artifact's kind carries and whether this project may see it.
    pub async fn read_artifact(
        &self,
        project_id: &str,
        artifact_id: &str,
        offset: u64,
        length: u64,
    ) -> Result<ReadArtifactResponse, DaemonLinkError> {
        let body = request::Body::ReadArtifact(ReadArtifactRequest {
            project_id: project_id.to_owned(),
            artifact_id: artifact_id.to_owned(),
            offset,
            length,
        });
        match self.call(body).await? {
            response::Body::ReadArtifact(read) => Ok(read),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    pub async fn list_projects(&self) -> Result<Vec<Project>, DaemonLinkError> {
        match self
            .call(request::Body::ListProjects(ListProjectsRequest {}))
            .await?
        {
            response::Body::ListProjects(listed) => Ok(listed.projects),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    pub async fn list_sources(&self, project_id: &str) -> Result<Vec<Source>, DaemonLinkError> {
        let body = request::Body::ListSources(ListSourcesRequest {
            project_id: project_id.to_owned(),
        });
        match self.call(body).await? {
            response::Body::ListSources(listed) => Ok(listed.sources),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    pub async fn list_jobs(&self, project_id: &str) -> Result<Vec<Job>, DaemonLinkError> {
        let body = request::Body::ListJobs(ListJobsRequest {
            project_id: project_id.to_owned(),
        });
        match self.call(body).await? {
            response::Body::ListJobs(listed) => Ok(listed.jobs),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    /// Register a local file as a source, which probes it.
    ///
    /// The daemon reads the container here, so the reply is what a screen shows
    /// before anyone commits to a run: this is the only way to learn a file's
    /// duration and streams, because probing is the daemon's job and it records
    /// what it found as an artifact.
    pub async fn register_source(
        &self,
        project_id: &str,
        absolute_path: &str,
    ) -> Result<RegisterSourceResponse, DaemonLinkError> {
        let body = request::Body::RegisterSource(RegisterSourceRequest {
            project_id: project_id.to_owned(),
            absolute_path: absolute_path.to_owned(),
        });
        match self.call(body).await? {
            response::Body::RegisterSource(registered) => Ok(registered),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    /// Submit the analysis DAG for a registered source.
    pub async fn submit_analyze(
        &self,
        project_id: &str,
        payload: AnalyzeSourcePayloadV1,
    ) -> Result<Job, DaemonLinkError> {
        let body = request::Body::SubmitJob(SubmitJobRequest {
            project_id: project_id.to_owned(),
            kind: "analyze-source".to_owned(),
            payload: payload.encode_to_vec(),
        });
        match self.call(body).await? {
            response::Body::SubmitJob(submitted) => submitted.job.ok_or(DaemonLinkError::Empty),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    pub async fn storage_stats(&self) -> Result<GetStorageStatsResponse, DaemonLinkError> {
        match self
            .call(request::Body::GetStorageStats(GetStorageStatsRequest {}))
            .await?
        {
            response::Body::GetStorageStats(stats) => Ok(stats),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    pub async fn get_job(&self, job_id: &str) -> Result<Job, DaemonLinkError> {
        let body = request::Body::GetJob(GetJobRequest {
            job_id: job_id.to_owned(),
        });
        match self.call(body).await? {
            response::Body::GetJob(fetched) => fetched.job.ok_or(DaemonLinkError::Empty),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    /// A whole document, gathered from however many chunks it takes.
    ///
    /// The daemon caps what one reply carries, so anything larger arrives in
    /// pieces and this reassembles them. It stops when it has the total the
    /// daemon stated — not when a short read happens to look final — so a
    /// truncated document is an error rather than a document with the end
    /// missing, which a parser would report as malformed JSON somewhere
    /// unhelpful.
    pub async fn read_document(
        &self,
        project_id: &str,
        artifact_id: &str,
    ) -> Result<(String, String), DaemonLinkError> {
        let mut bytes = Vec::new();
        let mut kind = String::new();
        loop {
            let offset = bytes.len() as u64;
            let chunk = self
                .read_artifact(project_id, artifact_id, offset, 0)
                .await?;
            if kind.is_empty() {
                kind.clone_from(&chunk.kind);
            } else if kind != chunk.kind {
                // The artifact changed underneath a multi-chunk read, which a
                // content-addressed store should make impossible.
                return Err(DaemonLinkError::Unexpected);
            }
            let total = chunk.total_bytes;
            if chunk.chunk.is_empty() && (bytes.len() as u64) < total {
                return Err(DaemonLinkError::Closed);
            }
            bytes.extend_from_slice(&chunk.chunk);
            if bytes.len() as u64 >= total {
                break;
            }
        }
        let text = String::from_utf8(bytes).map_err(|_| DaemonLinkError::Unexpected)?;
        Ok((kind, text))
    }

    /// Create a project and return its id.
    pub async fn create_project(&self, name: &str) -> Result<String, DaemonLinkError> {
        let body = request::Body::CreateProject(CreateProjectRequest {
            name: name.to_owned(),
        });
        match self.call(body).await? {
            response::Body::CreateProject(created) => created
                .project
                .map(|project| project.project_id)
                .ok_or(DaemonLinkError::Empty),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    /// Submit the reference DAG. Four tasks with real transitions and no media,
    /// which is what makes it the right thing to prove the event stream with.
    pub async fn submit_demo(
        &self,
        project_id: &str,
        seed: &[u8],
    ) -> Result<String, DaemonLinkError> {
        let payload = DemoDagPayloadV1 {
            key_version: "clipmill.demo-dag.v1".to_owned(),
            seed: seed.to_vec(),
        };
        let body = request::Body::SubmitJob(SubmitJobRequest {
            project_id: project_id.to_owned(),
            kind: "demo-dag".to_owned(),
            payload: payload.encode_to_vec(),
        });
        match self.call(body).await? {
            response::Body::SubmitJob(submitted) => submitted
                .job
                .map(|job| job.job_id)
                .ok_or(DaemonLinkError::Empty),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }

    /// Follow task events until the connection ends, calling `on_event` for each.
    ///
    /// Unlike every other call here this one holds its connection open: the
    /// daemon answers once with the cursor it is starting from, then pushes
    /// frames as tasks move. Returning means the link dropped, which is the
    /// caller's cue to resubscribe rather than an error to surface.
    ///
    /// `after_event_id` is what makes a reconnect honest. The daemon replays
    /// durable events strictly after that cursor, so a shell that was away comes
    /// back with the transitions it missed instead of a stage frozen wherever it
    /// was when the socket died.
    pub async fn stream_task_events<F>(
        &self,
        after_event_id: u64,
        mut on_event: F,
    ) -> Result<(), DaemonLinkError>
    where
        F: FnMut(TaskEvent),
    {
        let request_id = REQUEST_IDS.next("shell-events");
        let envelope = Request {
            request_id: request_id.clone(),
            body: Some(request::Body::SubscribeTaskEvents(
                SubscribeTaskEventsRequest {
                    // Every project and every job: one subscription serves the
                    // whole window, and the renderer routes by job id.
                    project_id: String::new(),
                    job_id: String::new(),
                    after_event_id,
                },
            )),
        };
        let mut stream = UnixStream::connect(&self.socket)
            .await
            .map_err(DaemonLinkError::Unavailable)?;
        write_frame(&mut stream, &envelope.encode_to_vec()).await?;

        // Only the handshake is given a deadline. After it, silence is the
        // normal state of a pipeline nobody is running.
        let opening = timeout(CALL_TIMEOUT, read_frame(&mut stream))
            .await
            .map_err(|_| DaemonLinkError::TimedOut)??;
        let opening = Response::decode(opening.as_slice())?;
        if opening.request_id != request_id {
            return Err(DaemonLinkError::Mismatched);
        }
        match opening.body {
            Some(response::Body::SubscribeTaskEvents(_)) => {}
            Some(response::Body::Error(error)) => {
                return Err(DaemonLinkError::Remote(error.message));
            }
            _ => return Err(DaemonLinkError::Unexpected),
        }

        loop {
            let frame = match read_frame(&mut stream).await {
                Ok(frame) => frame,
                // A closed socket is how this call ends, not a failure to report.
                Err(DaemonLinkError::Closed) => return Ok(()),
                Err(error) => return Err(error),
            };
            let response = Response::decode(frame.as_slice())?;
            if response.request_id != request_id {
                return Err(DaemonLinkError::Mismatched);
            }
            match response.body {
                Some(response::Body::TaskEvent(event)) => on_event(event),
                Some(response::Body::Error(error)) => {
                    return Err(DaemonLinkError::Remote(error.message));
                }
                // A body this shell does not know is skipped rather than fatal:
                // a newer daemon may push something a newer renderer wants.
                Some(_) | None => {}
            }
        }
    }

    /// Permission to stream a media artifact, and the inventory of what it
    /// holds. The bytes never come back through here.
    pub async fn resolve_media(
        &self,
        project_id: &str,
        artifact_id: &str,
    ) -> Result<ResolveMediaResponse, DaemonLinkError> {
        let body = request::Body::ResolveMedia(ResolveMediaRequest {
            project_id: project_id.to_owned(),
            artifact_id: artifact_id.to_owned(),
        });
        match self.call(body).await? {
            response::Body::ResolveMedia(resolved) => Ok(resolved),
            _ => Err(DaemonLinkError::Unexpected),
        }
    }
}

/// What the sidebar and the Models screen render. `local_lock` is the daemon's
/// own answer, never a constant in the UI: the badge has to be able to be wrong.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum ConnectionState {
    Connecting,
    Connected {
        #[serde(rename = "daemonVersion")]
        daemon_version: String,
        #[serde(rename = "localLock")]
        local_lock: bool,
        #[serde(rename = "startedUnixMillis")]
        started_unix_millis: u64,
    },
    Disconnected {
        reason: String,
    },
}

/// Locates `clipmilld` without giving the renderer any say in it.
fn resolve_daemon_binary() -> Option<PathBuf> {
    if let Some(explicit) = std::env::var_os("CLIPMILL_DAEMON_BIN") {
        let path = PathBuf::from(explicit);
        if path.is_file() {
            return Some(path);
        }
    }
    // Bundled layout: the daemon sits beside the shell executable.
    if let Ok(current) = std::env::current_exe()
        && let Some(directory) = current.parent()
    {
        let sibling = directory.join("clipmilld");
        if sibling.is_file() {
            return Some(sibling);
        }
    }
    None
}

/// Owns the daemon lifecycle: probes it, starts it when it is missing, and
/// keeps reporting the truth in between.
#[derive(Debug)]
pub struct DaemonSupervisor {
    client: DaemonClient,
    state: RwLock<ConnectionState>,
    child: Mutex<Option<Child>>,
}

impl DaemonSupervisor {
    pub fn new(client: DaemonClient) -> Self {
        Self {
            client,
            state: RwLock::new(ConnectionState::Connecting),
            child: Mutex::new(None),
        }
    }

    pub fn client(&self) -> &DaemonClient {
        &self.client
    }

    pub async fn state(&self) -> ConnectionState {
        self.state.read().await.clone()
    }

    async fn publish(&self, next: ConnectionState) -> Option<ConnectionState> {
        let mut current = self.state.write().await;
        if *current == next {
            return None;
        }
        *current = next.clone();
        Some(next)
    }

    /// Spawn a daemon only if one is not already answering. An externally
    /// started daemon keeps ownership of its own lifetime; we never kill it.
    async fn spawn(&self) -> Result<(), DaemonLinkError> {
        let mut slot = self.child.lock().await;
        if let Some(child) = slot.as_mut()
            && matches!(child.try_wait(), Ok(None))
        {
            return Ok(());
        }
        let binary = resolve_daemon_binary().ok_or(DaemonLinkError::MissingBinary)?;
        tracing::info!(binary = %binary.display(), "starting clipmilld");
        let child = Command::new(binary).kill_on_drop(true).spawn()?;
        *slot = Some(child);
        Ok(())
    }

    /// One supervision step: probe, and if the socket is dead try to revive it.
    /// Returns the new state when it changed, so callers only emit on edges.
    pub async fn reconcile(&self) -> Option<ConnectionState> {
        if let Ok(health) = self.client.health().await {
            return self
                .publish(ConnectionState::Connected {
                    daemon_version: health.daemon_version,
                    local_lock: health.local_lock,
                    started_unix_millis: health.started_unix_millis,
                })
                .await;
        }

        let changed = self
            .publish(ConnectionState::Disconnected {
                reason: "daemon is not answering".to_owned(),
            })
            .await;

        if let Err(error) = self.spawn().await {
            tracing::warn!(%error, "cannot start clipmilld");
            return changed;
        }

        let deadline = tokio::time::Instant::now() + STARTUP_TIMEOUT;
        while tokio::time::Instant::now() < deadline {
            sleep(STARTUP_POLL_INTERVAL).await;
            if let Ok(health) = self.client.health().await {
                return self
                    .publish(ConnectionState::Connected {
                        daemon_version: health.daemon_version,
                        local_lock: health.local_lock,
                        started_unix_millis: health.started_unix_millis,
                    })
                    .await
                    .or(changed);
            }
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    #[test]
    fn request_ids_do_not_repeat_when_a_shell_session_restarts_its_counter() {
        let before_restart = RequestIds::new();
        let after_restart = RequestIds::new();
        let old = before_restart.next("shell");
        let new = after_restart.next("shell");
        // Both processes begin at zero, but durable mutation receipt keys must
        // still differ. Events and concurrent calls share the same sequence.
        assert!(old.ends_with("-0"));
        assert!(new.ends_with("-0"));
        assert_ne!(old, new);
        assert_ne!(old, before_restart.next("shell"));
        assert_ne!(new, after_restart.next("shell-events"));
        assert!(old.len() <= 128 && new.len() <= 128);
        assert!(old.is_ascii() && new.is_ascii());
    }

    #[tokio::test]
    async fn a_dropped_mutation_reply_retries_the_identical_request_envelope() {
        use tokio::net::UnixListener;

        // macOS temporary directories can exceed the Unix socket path limit.
        let socket = PathBuf::from(format!("/tmp/cm-retry-{}.sock", ulid::Ulid::new()));
        let listener = UnixListener::bind(&socket).expect("bind test socket");
        let server = tokio::spawn(async move {
            let (mut first, _) = listener.accept().await.expect("first connection");
            let original = read_frame(&mut first).await.expect("first request");
            // The mutation could already be committed. Lose only its reply.
            drop(first);
            let (mut retry, _) = listener.accept().await.expect("retry connection");
            let repeated = read_frame(&mut retry).await.expect("retried request");
            assert_eq!(
                original, repeated,
                "a retry must reuse the complete envelope"
            );
            let request = Request::decode(repeated.as_slice()).expect("request envelope");
            assert!(matches!(
                request.body,
                Some(request::Body::CreateProject(_))
            ));
            let response = Response {
                request_id: request.request_id,
                body: Some(response::Body::CreateProject(
                    clipmill_contracts::proto::ipc::v1::CreateProjectResponse {
                        project: Some(Project {
                            project_id: "prj_retry_receipt".to_owned(),
                            ..Project::default()
                        }),
                    },
                )),
            };
            write_frame(&mut retry, &response.encode_to_vec())
                .await
                .expect("send receipt");
        });
        let received = DaemonClient::new(socket.clone())
            .create_project("a retried mutation")
            .await;
        timeout(Duration::from_secs(2), server)
            .await
            .expect("retry reached test server")
            .expect("test server");
        std::fs::remove_file(socket).expect("remove test socket");
        assert_eq!(received.expect("replayed response"), "prj_retry_receipt");
    }

    #[test]
    fn varint_round_trips_through_the_frame_reader() {
        for value in [0_u64, 1, 127, 128, 300, 16_383, 16_384, 1_000_000] {
            let mut encoded = Vec::new();
            encode_varint(value, &mut encoded);

            // Decode with the same shift loop read_frame uses.
            let mut decoded = 0_u64;
            let mut shift = 0_u32;
            for byte in &encoded {
                decoded |= u64::from(byte & 0x7f) << shift;
                shift += 7;
            }
            assert_eq!(decoded, value, "varint {value} did not round-trip");
        }
    }

    #[test]
    fn single_byte_varints_stay_single_byte() {
        let mut encoded = Vec::new();
        encode_varint(127, &mut encoded);
        assert_eq!(encoded, vec![127]);
    }

    #[test]
    fn connection_state_serialises_as_a_tagged_union() {
        let json = serde_json::to_string(&ConnectionState::Connected {
            daemon_version: "0.0.1".to_owned(),
            local_lock: true,
            started_unix_millis: 42,
        })
        .unwrap();
        assert!(json.contains(r#""status":"connected""#), "got {json}");
        assert!(json.contains(r#""localLock":true"#), "got {json}");
    }
}
