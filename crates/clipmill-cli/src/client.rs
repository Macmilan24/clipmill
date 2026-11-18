//! A connection to the running engine, speaking the engine API v2.
//!
//! Each call opens a connection, sends one framed request and reads one framed
//! answer, as the Studio's host does: a stale socket cannot block the next call.

use std::path::{Path, PathBuf};

use clipmill_contracts::proto::ipc::v1::{EngineCallRequest, Request, Response, request, response};
use prost::Message;
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const MAX_FRAME_BYTES: usize = 8 * 1024 * 1024;

/// Why a call did not get an answer from the engine at all.
#[derive(Debug)]
pub enum LinkError {
    /// Nothing is listening: ClipMill is not running.
    NotRunning(PathBuf),
    Io(std::io::Error),
    Protocol(String),
}

impl std::fmt::Display for LinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotRunning(path) => write!(
                f,
                "ClipMill is not running (no engine at {}). Open the ClipMill app, or start clipmilld.",
                path.display()
            ),
            Self::Io(error) => write!(f, "the connection to ClipMill failed: {error}"),
            Self::Protocol(detail) => write!(f, "ClipMill answered something unexpected: {detail}"),
        }
    }
}

impl std::error::Error for LinkError {}

/// Where the engine listens and who is calling.
#[derive(Clone, Debug)]
pub struct Engine {
    socket: PathBuf,
    token: String,
}

fn encode_varint(mut value: u64, out: &mut Vec<u8>) {
    loop {
        #[allow(clippy::cast_possible_truncation, reason = "masked to seven bits")]
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

impl Engine {
    pub fn new(socket: PathBuf, token: String) -> Self {
        Self { socket, token }
    }

    /// The engine of this account's ClipMill, unless a socket is named.
    pub fn locate(socket: Option<PathBuf>, token: String) -> Result<Self, String> {
        let socket = match socket {
            Some(socket) => socket,
            None => {
                clipmilld::Config::resolve(None, None)
                    .map_err(|error| error.to_string())?
                    .paths
                    .socket
            }
        };
        Ok(Self::new(socket, token))
    }

    pub fn socket(&self) -> &Path {
        &self.socket
    }

    /// One engine API call: the result on success, the engine's refusal as
    /// `Err(Value)` with its code and message.
    pub async fn call(&self, op: &str, args: Value) -> Result<Result<Value, Value>, LinkError> {
        self.call_keyed(op, args, None).await
    }

    /// An explicit retry key survives reconnects. Transport request IDs are
    /// independent, since clients may reuse JSON-RPC IDs on a new connection.
    pub async fn call_keyed(
        &self,
        op: &str,
        args: Value,
        key: Option<&str>,
    ) -> Result<Result<Value, Value>, LinkError> {
        let mut call = json!({ "op": op, "args": args });
        if let Some(key) = key {
            call["request_id"] = json!(key);
        }
        let request = Request {
            request_id: format!("cli_{}", ulid::Ulid::new().to_string().to_lowercase()),
            body: Some(request::Body::EngineCall(EngineCallRequest {
                token: self.token.clone(),
                call_json: call.to_string(),
            })),
        };
        let response = self.send(&request).await?;
        let Some(response::Body::EngineCall(answer)) = response.body else {
            return Err(LinkError::Protocol(match response.body {
                Some(response::Body::Error(error)) => error.message,
                _ => "no engine answer; is this ClipMill 0.5 or later?".to_owned(),
            }));
        };
        let value: Value = serde_json::from_str(&answer.result_json)
            .map_err(|error| LinkError::Protocol(error.to_string()))?;
        if value["ok"] == true {
            Ok(Ok(value["result"].clone()))
        } else {
            Ok(Err(value["error"].clone()))
        }
    }

    async fn send(&self, request: &Request) -> Result<Response, LinkError> {
        let mut stream = clipmilld::endpoint::connect(&self.socket)
            .await
            .map_err(|error| match error.kind() {
                std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused => {
                    LinkError::NotRunning(self.socket.clone())
                }
                _ => LinkError::Io(error),
            })?;
        let payload = request.encode_to_vec();
        let mut frame = Vec::with_capacity(payload.len() + 10);
        encode_varint(payload.len() as u64, &mut frame);
        frame.extend_from_slice(&payload);
        stream.write_all(&frame).await.map_err(LinkError::Io)?;
        stream.flush().await.map_err(LinkError::Io)?;

        let mut length = 0_u64;
        let mut shift = 0_u32;
        loop {
            let mut byte = [0_u8; 1];
            if stream.read(&mut byte).await.map_err(LinkError::Io)? == 0 {
                return Err(LinkError::Protocol(
                    "the engine closed the connection".to_owned(),
                ));
            }
            length |= u64::from(byte[0] & 0x7f) << shift;
            if byte[0] & 0x80 == 0 {
                break;
            }
            shift += 7;
            if shift > 63 {
                return Err(LinkError::Protocol("a malformed frame length".to_owned()));
            }
        }
        let length = usize::try_from(length)
            .ok()
            .filter(|length| *length <= MAX_FRAME_BYTES)
            .ok_or_else(|| LinkError::Protocol("an oversized answer".to_owned()))?;
        let mut payload = vec![0_u8; length];
        stream
            .read_exact(&mut payload)
            .await
            .map_err(LinkError::Io)?;
        Response::decode(payload.as_slice()).map_err(|error| LinkError::Protocol(error.to_string()))
    }
}
