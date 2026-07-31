//! The shell's link to `clipmilld`.
//!
//! The WebView never speaks to the daemon. It has no socket, no filesystem and
//! no network capability; it can only call the commands this host process
//! exposes. Everything below — framing, the request/response envelope, process
//! supervision — runs in Rust so that the renderer stays a pure view layer.

use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use clipmill_contracts::proto::ipc::v1::{
    CreateProjectRequest, DemoDagPayloadV1, GetDeviceProfileRequest, GetDeviceProfileResponse,
    GetJobRequest, HealthRequest, HealthResponse, Job, ListJobsRequest, ListProjectsRequest,
    ListSourcesRequest, Project, ReadArtifactRequest, ReadArtifactResponse, Request,
    ResolveMediaRequest, ResolveMediaResponse, Response, Source, SubmitJobRequest,
    SubscribeTaskEventsRequest, TaskEvent, request, response,
};
use prost::Message;
use serde::Serialize;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
    process::{Child, Command},
    sync::{Mutex, RwLock},
    time::{sleep, timeout},
};

/// Frames larger than this are refused rather than allocated: the socket is
/// trusted, but a corrupted length prefix should not become an OOM.
const MAX_FRAME_BYTES: usize = 8 * 1024 * 1024;
const CALL_TIMEOUT: Duration = Duration::from_secs(10);
/// How long a freshly spawned daemon gets to open its socket.
const STARTUP_TIMEOUT: Duration = Duration::from_secs(20);
const STARTUP_POLL_INTERVAL: Duration = Duration::from_millis(150);

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(0);

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

    async fn call(&self, body: request::Body) -> Result<response::Body, DaemonLinkError> {
        let request_id = format!("shell-{}", REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed));
        let envelope = Request {
            request_id: request_id.clone(),
            body: Some(body),
        };

        let exchange = async {
            let mut stream = UnixStream::connect(&self.socket)
                .await
                .map_err(DaemonLinkError::Unavailable)?;
            write_frame(&mut stream, &envelope.encode_to_vec()).await?;
            let payload = read_frame(&mut stream).await?;
            Response::decode(payload.as_slice()).map_err(DaemonLinkError::from)
        };

        let response = timeout(CALL_TIMEOUT, exchange)
            .await
            .map_err(|_| DaemonLinkError::TimedOut)??;

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
        let request_id = format!(
            "shell-events-{}",
            REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed)
        );
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
