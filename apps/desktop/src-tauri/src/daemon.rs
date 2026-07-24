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
    GetDeviceProfileRequest, GetDeviceProfileResponse, HealthRequest, HealthResponse, Request,
    Response, request, response,
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
