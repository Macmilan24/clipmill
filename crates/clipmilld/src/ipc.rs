use std::{io, time::Duration};

use clipmill_contracts::proto::ipc::v1::Request;
use prost::Message;
use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::UnixStream,
    time::{Instant, timeout},
};

use crate::service::{Service, request_kind};

pub(crate) const MAX_FRAME_BYTES: usize = 4 * 1024 * 1024;
const READ_TIMEOUT: Duration = Duration::from_secs(30);
const WRITE_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Error)]
pub(crate) enum FrameError {
    #[error("frame length prefix is malformed")]
    MalformedLength,
    #[error("frame is {length} bytes; maximum is {maximum}")]
    Oversized { length: u64, maximum: usize },
    #[error("peer closed in the middle of a frame")]
    Truncated,
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("read timed out")]
    ReadTimeout,
    #[error("write timed out")]
    WriteTimeout,
    #[error("request protobuf is malformed")]
    MalformedRequest,
}

pub(crate) async fn handle_connection(
    mut stream: UnixStream,
    service: Service,
) -> Result<(), FrameError> {
    loop {
        let frame = timeout(READ_TIMEOUT, read_frame(&mut stream))
            .await
            .map_err(|_| FrameError::ReadTimeout)??;
        let Some(frame) = frame else {
            return Ok(());
        };
        let request =
            Request::decode(frame.as_slice()).map_err(|_| FrameError::MalformedRequest)?;
        let kind = request_kind(&request);
        let request_id = request.request_id.clone();
        let started = Instant::now();
        let reply = service.handle(request).await;
        let outcome = reply.outcome;
        let write_result = timeout(WRITE_TIMEOUT, write_frame(&mut stream, &reply.bytes))
            .await
            .map_err(|_| FrameError::WriteTimeout);
        tracing::info!(
            request_kind = kind,
            request_id,
            outcome = outcome.as_str(),
            latency_ms = started.elapsed().as_millis(),
            "handled IPC request"
        );
        write_result??;
    }
}

pub(crate) async fn read_frame<R>(reader: &mut R) -> Result<Option<Vec<u8>>, FrameError>
where
    R: AsyncRead + Unpin,
{
    let mut length = 0_u64;
    let mut shift = 0_u32;
    for index in 0..10 {
        let mut byte = [0_u8; 1];
        let read = reader.read(&mut byte).await?;
        if read == 0 {
            return if index == 0 {
                Ok(None)
            } else {
                Err(FrameError::Truncated)
            };
        }
        let value = byte[0];
        if index == 9 && value > 1 {
            return Err(FrameError::MalformedLength);
        }
        length |= u64::from(value & 0x7f) << shift;
        if value & 0x80 == 0 {
            if length > MAX_FRAME_BYTES as u64 {
                return Err(FrameError::Oversized {
                    length,
                    maximum: MAX_FRAME_BYTES,
                });
            }
            let length = usize::try_from(length).map_err(|_| FrameError::Oversized {
                length,
                maximum: MAX_FRAME_BYTES,
            })?;
            let mut payload = vec![0_u8; length];
            if let Err(error) = reader.read_exact(&mut payload).await {
                return if error.kind() == io::ErrorKind::UnexpectedEof {
                    Err(FrameError::Truncated)
                } else {
                    Err(FrameError::Io(error))
                };
            }
            return Ok(Some(payload));
        }
        shift += 7;
    }
    Err(FrameError::MalformedLength)
}

pub(crate) async fn write_frame<W>(writer: &mut W, payload: &[u8]) -> Result<(), FrameError>
where
    W: AsyncWrite + Unpin,
{
    if payload.len() > MAX_FRAME_BYTES {
        return Err(FrameError::Oversized {
            length: payload.len() as u64,
            maximum: MAX_FRAME_BYTES,
        });
    }
    let mut length = payload.len() as u64;
    let mut prefix = [0_u8; 10];
    let mut prefix_len = 0;
    loop {
        let mut byte = (length & 0x7f) as u8;
        length >>= 7;
        if length != 0 {
            byte |= 0x80;
        }
        prefix[prefix_len] = byte;
        prefix_len += 1;
        if length == 0 {
            break;
        }
    }
    writer.write_all(&prefix[..prefix_len]).await?;
    writer.write_all(payload).await?;
    writer.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use std::io::Cursor;

    use tokio::io::AsyncWriteExt;

    use super::{FrameError, MAX_FRAME_BYTES, read_frame, write_frame};

    #[tokio::test]
    async fn roundtrips_concatenated_and_zero_length_frames() {
        let mut encoded = Vec::new();
        write_frame(&mut encoded, b"first").await.expect("write");
        write_frame(&mut encoded, b"").await.expect("write empty");
        write_frame(&mut encoded, b"third").await.expect("write");
        let mut reader = Cursor::new(encoded);
        assert_eq!(
            read_frame(&mut reader).await.expect("read"),
            Some(b"first".to_vec())
        );
        assert_eq!(
            read_frame(&mut reader).await.expect("read"),
            Some(Vec::new())
        );
        assert_eq!(
            read_frame(&mut reader).await.expect("read"),
            Some(b"third".to_vec())
        );
        assert_eq!(read_frame(&mut reader).await.expect("eof"), None);
    }

    #[tokio::test]
    async fn rejects_malformed_and_oversized_prefixes() {
        let mut malformed = Cursor::new(vec![0x80; 10]);
        assert!(matches!(
            read_frame(&mut malformed).await,
            Err(FrameError::MalformedLength)
        ));

        let oversized = (MAX_FRAME_BYTES as u64) + 1;
        let mut bytes = Vec::new();
        let mut value = oversized;
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            bytes.push(byte);
            if value == 0 {
                break;
            }
        }
        let mut reader = Cursor::new(bytes);
        assert!(matches!(
            read_frame(&mut reader).await,
            Err(FrameError::Oversized { .. })
        ));
    }

    #[tokio::test]
    async fn rejects_truncated_prefix_and_body() {
        let mut prefix = Cursor::new(vec![0x80]);
        assert!(matches!(
            read_frame(&mut prefix).await,
            Err(FrameError::Truncated)
        ));

        let mut body = Cursor::new(vec![5, 1, 2]);
        assert!(matches!(
            read_frame(&mut body).await,
            Err(FrameError::Truncated)
        ));
    }

    #[tokio::test]
    async fn reassembles_fragmented_prefixes_and_payloads() {
        let payload = vec![0x5a; 130];
        let expected = payload.clone();
        let (mut writer, mut reader) = tokio::io::duplex(256);
        let writing = tokio::spawn(async move {
            writer.write_all(&[0x82]).await.expect("prefix part one");
            tokio::task::yield_now().await;
            writer.write_all(&[0x01]).await.expect("prefix part two");
            writer
                .write_all(&payload[..17])
                .await
                .expect("payload part one");
            tokio::task::yield_now().await;
            writer
                .write_all(&payload[17..])
                .await
                .expect("payload part two");
        });

        assert_eq!(
            read_frame(&mut reader).await.expect("fragmented frame"),
            Some(expected)
        );
        writing.await.expect("writer joins");
    }
}
