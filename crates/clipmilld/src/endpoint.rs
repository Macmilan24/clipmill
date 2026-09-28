//! Where the daemon's planes listen, and how the two ends of a connection
//! know each other.
//!
//! On Unix a plane is a socket in the daemon's private run directory: only
//! its user can open that directory, so only its user can connect. Windows
//! has no socket of that kind Python's standard library can use, so there a
//! plane listens on a loopback port, and its path in the same private
//! directory is a small file naming the port and a secret made for this start
//! of the daemon. A port is open to every process on the machine, and once
//! the daemon stops another process may listen on it, so before either end
//! says anything else each proves it holds that secret:
//!
//! ```text
//! client → server   HELLO, client nonce
//! server → client   server nonce, HMAC(secret, server role | client nonce | server nonce)
//! client → server   HMAC(secret, client role | server nonce | client nonce)
//! ```
//!
//! The server proves itself first, so a client that reached something else
//! stops before it has shown anything. The secret never crosses the
//! connection, both nonces are fresh, and the two roles differ, so no proof
//! can be replayed or reflected. Only then does the plane's own protocol
//! begin. The handshake is compiled and tested on every platform; only
//! Windows uses it.

use std::{
    fmt,
    io::{self, Read, Write},
    path::Path,
};

use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// The schema of an endpoint file.
pub const ENDPOINT_SCHEMA: &str = "clipmill.endpoint.v1";
/// How long either end waits for the other's half of the handshake.
pub const HANDSHAKE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// A client's first bytes: a server refuses a stranger on these alone.
const HELLO: &[u8; 8] = b"CMILL/1\n";
const SECRET_BYTES: usize = 32;
const NONCE_BYTES: usize = 32;
const PROOF_BYTES: usize = 32;
const GREETING_BYTES: usize = HELLO.len() + NONCE_BYTES;
const REPLY_BYTES: usize = NONCE_BYTES + PROOF_BYTES;
const SERVER_ROLE: &[u8] = b"clipmill endpoint server";
const CLIENT_ROLE: &[u8] = b"clipmill endpoint client";

#[derive(Debug, Error)]
pub enum HandshakeError {
    #[error("the other end did not greet as a ClipMill client")]
    NotAClient,
    #[error("the other end could not prove it holds this endpoint's secret")]
    WrongProof,
    #[error("no random bytes for the handshake: {0}")]
    Random(String),
    #[error("the endpoint file is not usable: {0}")]
    Endpoint(String),
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// The secret one start of the daemon made for one plane. Never printed.
#[derive(Clone, Eq, PartialEq)]
pub struct Secret([u8; SECRET_BYTES]);

impl fmt::Debug for Secret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Secret(..)")
    }
}

impl Secret {
    pub fn generate() -> Result<Self, HandshakeError> {
        let mut bytes = [0_u8; SECRET_BYTES];
        getrandom::fill(&mut bytes).map_err(|error| HandshakeError::Random(error.to_string()))?;
        Ok(Self(bytes))
    }
}

/// What an endpoint file holds: the loopback port a plane listens on, and
/// its secret.
#[derive(Debug, Deserialize, Serialize)]
struct EndpointFile {
    schema_version: String,
    port: u16,
    secret: String,
}

/// Write the file a client reads to reach a plane.
pub fn write_endpoint(path: &Path, port: u16, secret: &Secret) -> Result<(), HandshakeError> {
    let file = EndpointFile {
        schema_version: ENDPOINT_SCHEMA.to_owned(),
        port,
        secret: hex::encode(secret.0),
    };
    let bytes =
        serde_json::to_vec(&file).map_err(|error| HandshakeError::Endpoint(error.to_string()))?;
    crate::library::write_private(path, &bytes)?;
    Ok(())
}

/// Read the port and secret an endpoint file names.
pub fn read_endpoint(path: &Path) -> Result<(u16, Secret), HandshakeError> {
    let bytes = std::fs::read(path)?;
    let file: EndpointFile = serde_json::from_slice(&bytes)
        .map_err(|error| HandshakeError::Endpoint(error.to_string()))?;
    if file.schema_version != ENDPOINT_SCHEMA {
        return Err(HandshakeError::Endpoint(format!(
            "schema {} is not {ENDPOINT_SCHEMA}",
            file.schema_version
        )));
    }
    if file.port == 0 {
        return Err(HandshakeError::Endpoint("port 0".to_owned()));
    }
    let secret = hex::decode(&file.secret)
        .ok()
        .and_then(|bytes| <[u8; SECRET_BYTES]>::try_from(bytes).ok())
        .ok_or_else(|| HandshakeError::Endpoint("the secret is not 32 hex bytes".to_owned()))?;
    Ok((file.port, Secret(secret)))
}

// ---- the platform's connections ---------------------------------------------

/// A connection on one of the daemon's planes.
#[cfg(unix)]
pub type Stream = tokio::net::UnixStream;
/// A connection on one of the daemon's planes.
#[cfg(windows)]
pub type Stream = tokio::net::TcpStream;

/// The writing half of a [`Stream`] split in two.
#[cfg(unix)]
pub type WriteHalf = tokio::net::unix::OwnedWriteHalf;
/// The writing half of a [`Stream`] split in two.
#[cfg(windows)]
pub type WriteHalf = tokio::net::tcp::OwnedWriteHalf;

/// A blocking connection, for handlers that run on a blocking thread.
#[cfg(unix)]
pub type BlockingStream = std::os::unix::net::UnixStream;
/// A blocking connection, for handlers that run on a blocking thread.
#[cfg(windows)]
pub type BlockingStream = std::net::TcpStream;

/// Where a plane listens: a socket at its path on Unix, and on Windows a
/// loopback port the file at its path names.
#[derive(Debug)]
pub struct Listener {
    #[cfg(unix)]
    inner: tokio::net::UnixListener,
    #[cfg(windows)]
    inner: tokio::net::TcpListener,
    #[cfg(windows)]
    secret: Secret,
}

/// A connection a [`Listener`] took, not yet established: on Windows the
/// client has still to prove itself.
#[derive(Debug)]
pub struct Incoming {
    stream: Stream,
    #[cfg(windows)]
    secret: Secret,
}

impl Listener {
    /// Listen on the plane whose address is `path`. The caller has made sure
    /// nothing else answers there and that `path`'s directory is private.
    #[cfg_attr(unix, allow(clippy::unused_async))]
    pub async fn bind(path: &Path) -> io::Result<Self> {
        #[cfg(unix)]
        {
            Ok(Self {
                inner: tokio::net::UnixListener::bind(path)?,
            })
        }
        #[cfg(windows)]
        {
            let inner = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await?;
            let port = inner.local_addr()?.port();
            let secret = Secret::generate().map_err(into_io)?;
            write_endpoint(path, port, &secret).map_err(into_io)?;
            Ok(Self { inner, secret })
        }
    }

    pub async fn accept(&self) -> io::Result<Incoming> {
        let (stream, _address) = self.inner.accept().await?;
        Ok(Incoming {
            stream,
            #[cfg(windows)]
            secret: self.secret.clone(),
        })
    }
}

impl Incoming {
    /// The connection, once the client has proved itself where it must.
    /// Bounded by [`HANDSHAKE_TIMEOUT`], so a stalled client holds only its
    /// own task.
    #[cfg_attr(unix, allow(clippy::unused_async))]
    pub async fn establish(self) -> io::Result<Stream> {
        #[cfg(unix)]
        {
            Ok(self.stream)
        }
        #[cfg(windows)]
        {
            let mut stream = self.stream;
            tokio::time::timeout(HANDSHAKE_TIMEOUT, accept(&mut stream, &self.secret))
                .await
                .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "handshake timed out"))?
                .map_err(into_io)?;
            Ok(stream)
        }
    }
}

/// Connect to the plane whose address is `path`: on Windows, read the port
/// and the secret from the file there, and complete the handshake.
pub async fn connect(path: &Path) -> io::Result<Stream> {
    #[cfg(unix)]
    {
        tokio::net::UnixStream::connect(path).await
    }
    #[cfg(windows)]
    {
        let (port, secret) = read_endpoint(path).map_err(into_io)?;
        let mut stream =
            tokio::net::TcpStream::connect((std::net::Ipv4Addr::LOCALHOST, port)).await?;
        tokio::time::timeout(HANDSHAKE_TIMEOUT, connect_stream(&mut stream, &secret))
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "handshake timed out"))?
            .map_err(into_io)?;
        Ok(stream)
    }
}

/// Two connected ends, for tests of what runs over a plane.
#[cfg(test)]
#[cfg_attr(unix, allow(clippy::unused_async))]
pub(crate) async fn pair() -> io::Result<(Stream, Stream)> {
    #[cfg(unix)]
    {
        tokio::net::UnixStream::pair()
    }
    #[cfg(windows)]
    {
        let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await?;
        let address = listener.local_addr()?;
        let (connected, accepted) =
            tokio::join!(tokio::net::TcpStream::connect(address), listener.accept());
        Ok((connected?, accepted?.0))
    }
}

/// A handshake failure as the I/O error callers already handle: a peer that
/// could not prove itself is refused permission; a missing or unreadable
/// endpoint file means nothing answers there.
#[cfg(windows)]
fn into_io(error: HandshakeError) -> io::Error {
    match error {
        HandshakeError::Io(error) => error,
        HandshakeError::NotAClient | HandshakeError::WrongProof => {
            io::Error::new(io::ErrorKind::PermissionDenied, error)
        }
        HandshakeError::Endpoint(_) => io::Error::new(io::ErrorKind::NotFound, error),
        HandshakeError::Random(_) => io::Error::other(error),
    }
}

// ---- the messages, without any I/O ------------------------------------------

fn nonce() -> Result<[u8; NONCE_BYTES], HandshakeError> {
    let mut bytes = [0_u8; NONCE_BYTES];
    getrandom::fill(&mut bytes).map_err(|error| HandshakeError::Random(error.to_string()))?;
    Ok(bytes)
}

fn mac(secret: &Secret, role: &[u8], first: &[u8], second: &[u8]) -> Hmac<Sha256> {
    // An HMAC key may be any length, so this cannot fail.
    let mut mac = <Hmac<Sha256> as KeyInit>::new_from_slice(&secret.0)
        .unwrap_or_else(|_| unreachable!("HMAC accepts a key of any length"));
    mac.update(role);
    mac.update(first);
    mac.update(second);
    mac
}

fn prove(secret: &Secret, role: &[u8], first: &[u8], second: &[u8]) -> [u8; PROOF_BYTES] {
    mac(secret, role, first, second)
        .finalize()
        .into_bytes()
        .into()
}

fn verify(
    secret: &Secret,
    role: &[u8],
    first: &[u8],
    second: &[u8],
    proof: &[u8],
) -> Result<(), HandshakeError> {
    // `verify_slice` compares in constant time.
    mac(secret, role, first, second)
        .verify_slice(proof)
        .map_err(|_| HandshakeError::WrongProof)
}

struct Greeting {
    nonce: [u8; NONCE_BYTES],
    bytes: [u8; GREETING_BYTES],
}

fn client_greeting() -> Result<Greeting, HandshakeError> {
    let nonce = nonce()?;
    let mut bytes = [0_u8; GREETING_BYTES];
    bytes[..HELLO.len()].copy_from_slice(HELLO);
    bytes[HELLO.len()..].copy_from_slice(&nonce);
    Ok(Greeting { nonce, bytes })
}

/// The server's answer to a greeting, and what it must remember to check the
/// client's proof.
struct Reply {
    client_nonce: [u8; NONCE_BYTES],
    server_nonce: [u8; NONCE_BYTES],
    bytes: [u8; REPLY_BYTES],
}

fn server_reply(secret: &Secret, greeting: &[u8; GREETING_BYTES]) -> Result<Reply, HandshakeError> {
    let (hello, client_nonce) = greeting.split_at(HELLO.len());
    if hello != HELLO {
        return Err(HandshakeError::NotAClient);
    }
    let mut remembered = [0_u8; NONCE_BYTES];
    remembered.copy_from_slice(client_nonce);
    let server_nonce = nonce()?;
    let proof = prove(secret, SERVER_ROLE, client_nonce, &server_nonce);
    let mut bytes = [0_u8; REPLY_BYTES];
    bytes[..NONCE_BYTES].copy_from_slice(&server_nonce);
    bytes[NONCE_BYTES..].copy_from_slice(&proof);
    Ok(Reply {
        client_nonce: remembered,
        server_nonce,
        bytes,
    })
}

fn client_answer(
    secret: &Secret,
    greeting: &Greeting,
    reply: &[u8; REPLY_BYTES],
) -> Result<[u8; PROOF_BYTES], HandshakeError> {
    let (server_nonce, proof) = reply.split_at(NONCE_BYTES);
    verify(secret, SERVER_ROLE, &greeting.nonce, server_nonce, proof)?;
    Ok(prove(secret, CLIENT_ROLE, server_nonce, &greeting.nonce))
}

fn server_check(
    secret: &Secret,
    reply: &Reply,
    answer: &[u8; PROOF_BYTES],
) -> Result<(), HandshakeError> {
    verify(
        secret,
        CLIENT_ROLE,
        &reply.server_nonce,
        &reply.client_nonce,
        answer,
    )
}

// ---- over a connection ------------------------------------------------------

/// The server's half, on an async stream. The caller bounds it with
/// [`HANDSHAKE_TIMEOUT`].
pub async fn accept<S>(stream: &mut S, secret: &Secret) -> Result<(), HandshakeError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut greeting = [0_u8; GREETING_BYTES];
    stream.read_exact(&mut greeting).await?;
    let reply = server_reply(secret, &greeting)?;
    stream.write_all(&reply.bytes).await?;
    stream.flush().await?;
    let mut answer = [0_u8; PROOF_BYTES];
    stream.read_exact(&mut answer).await?;
    server_check(secret, &reply, &answer)
}

/// The client's half, on an async stream. The caller bounds it with
/// [`HANDSHAKE_TIMEOUT`].
pub async fn connect_stream<S>(stream: &mut S, secret: &Secret) -> Result<(), HandshakeError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let greeting = client_greeting()?;
    stream.write_all(&greeting.bytes).await?;
    stream.flush().await?;
    let mut reply = [0_u8; REPLY_BYTES];
    stream.read_exact(&mut reply).await?;
    let answer = client_answer(secret, &greeting, &reply)?;
    stream.write_all(&answer).await?;
    stream.flush().await?;
    Ok(())
}

/// The client's half, on a blocking stream whose timeouts the caller set.
pub fn connect_blocking<S>(stream: &mut S, secret: &Secret) -> Result<(), HandshakeError>
where
    S: Read + Write,
{
    let greeting = client_greeting()?;
    stream.write_all(&greeting.bytes)?;
    stream.flush()?;
    let mut reply = [0_u8; REPLY_BYTES];
    stream.read_exact(&mut reply)?;
    let answer = client_answer(secret, &greeting, &reply)?;
    stream.write_all(&answer)?;
    stream.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn secret(byte: u8) -> Secret {
        Secret([byte; SECRET_BYTES])
    }

    #[tokio::test]
    async fn two_ends_with_one_secret_complete_the_handshake() {
        let (mut client, mut server) = tokio::io::duplex(256);
        let shared = secret(7);
        let (accepted, connected) = tokio::join!(
            accept(&mut server, &shared),
            connect_stream(&mut client, &shared)
        );
        accepted.expect("the server accepts");
        connected.expect("the client connects");
        // The plane's own protocol follows on the same stream.
        client.write_all(b"frame").await.unwrap();
        let mut frame = [0_u8; 5];
        server.read_exact(&mut frame).await.unwrap();
        assert_eq!(&frame, b"frame");
    }

    #[tokio::test]
    async fn a_client_without_the_secret_is_refused_after_the_server_proves_itself() {
        let (client, mut server) = tokio::io::duplex(256);
        // The client end closes as soon as its half returns.
        let connecting = async move {
            let mut client = client;
            connect_stream(&mut client, &secret(2)).await
        };
        let server_secret = secret(1);
        let (accepted, connected) = tokio::join!(accept(&mut server, &server_secret), connecting);
        // The client checks the server's proof first, so it stops there and
        // never sends a proof of its own; the server sees the stream end.
        assert!(matches!(connected, Err(HandshakeError::WrongProof)));
        assert!(accepted.is_err());
    }

    #[tokio::test]
    async fn a_server_without_the_secret_learns_nothing_to_replay() {
        // A stranger listening on a port the daemon used to hold answers with
        // a proof it cannot make; the client refuses before proving anything.
        let (client, mut stranger) = tokio::io::duplex(256);
        let impostor = async {
            let mut greeting = [0_u8; GREETING_BYTES];
            stranger.read_exact(&mut greeting).await.unwrap();
            stranger.write_all(&[0_u8; REPLY_BYTES]).await.unwrap();
            let mut rest = Vec::new();
            stranger.read_to_end(&mut rest).await.unwrap();
            rest
        };
        let connecting = async move {
            let mut client = client;
            connect_stream(&mut client, &secret(3)).await
        };
        let (rest, connected) = tokio::join!(impostor, connecting);
        assert!(matches!(connected, Err(HandshakeError::WrongProof)));
        assert!(
            rest.is_empty(),
            "the client sent nothing after the bad proof"
        );
    }

    #[tokio::test]
    async fn a_stranger_that_does_not_greet_is_refused_at_once() {
        let (mut client, mut server) = tokio::io::duplex(256);
        client.write_all(&[b'x'; GREETING_BYTES]).await.unwrap();
        let accepted = accept(&mut server, &secret(4)).await;
        assert!(matches!(accepted, Err(HandshakeError::NotAClient)));
    }

    #[test]
    fn proofs_depend_on_role_and_both_nonces() {
        let shared = secret(5);
        let base = prove(&shared, SERVER_ROLE, &[1; 32], &[2; 32]);
        assert_ne!(base, prove(&shared, CLIENT_ROLE, &[1; 32], &[2; 32]));
        assert_ne!(base, prove(&shared, SERVER_ROLE, &[2; 32], &[1; 32]));
        assert_ne!(base, prove(&secret(6), SERVER_ROLE, &[1; 32], &[2; 32]));
        assert!(verify(&shared, SERVER_ROLE, &[1; 32], &[2; 32], &base).is_ok());
    }

    #[test]
    fn hmac_matches_the_rfc_4231_vector() {
        // RFC 4231, test case 2: key "Jefe", data "what do ya want for nothing?".
        let mut mac = <Hmac<Sha256> as KeyInit>::new_from_slice(b"Jefe").unwrap();
        mac.update(b"what do ya want for nothing?");
        assert_eq!(
            hex::encode(mac.finalize().into_bytes()),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
    }

    #[test]
    fn an_endpoint_file_round_trips_and_refuses_what_it_does_not_know() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("control.endpoint");
        let made = Secret::generate().unwrap();
        write_endpoint(&path, 50_123, &made).unwrap();
        let (port, read) = read_endpoint(&path).unwrap();
        assert_eq!((port, &read), (50_123, &made));
        assert_eq!(format!("{read:?}"), "Secret(..)");

        std::fs::write(
            &path,
            br#"{"schema_version":"other","port":1,"secret":"00"}"#,
        )
        .unwrap();
        assert!(read_endpoint(&path).is_err());
        let short = format!(r#"{{"schema_version":"{ENDPOINT_SCHEMA}","port":1,"secret":"abcd"}}"#);
        std::fs::write(&path, short).unwrap();
        assert!(read_endpoint(&path).is_err());
    }

    #[test]
    fn the_blocking_client_speaks_the_same_handshake() {
        let shared = secret(9);
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server_secret = shared.clone();
        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(async move {
                stream.set_nonblocking(true).unwrap();
                let mut stream = tokio::net::TcpStream::from_std(stream).unwrap();
                accept(&mut stream, &server_secret).await
            })
        });
        let mut client = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
        connect_blocking(&mut client, &shared).expect("the blocking client connects");
        server.join().unwrap().expect("the server accepts");
    }
}
