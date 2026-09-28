//! Narrow Hugging Face transport for pinned model weights.
//!
//! Two operations, both started by a person and both bounded. Reading what a
//! repository holds at one commit, so a model can be pinned by digest before
//! any weight is fetched; and fetching one pinned file, verifying its size and
//! SHA-256 before the caller may give the bytes their final name.
//!
//! Which model, where it goes and whether anybody asked for it are the
//! daemon's decisions. This crate is handed a repository, a commit, a path and
//! the digest the bytes must hash to, and nothing it returns is trusted by the
//! caller until checked. No credential is ever sent: a repository that needs
//! signing in is reported as such rather than signed in to.

use std::{
    io::Read as _,
    path::{Path, PathBuf},
    time::Duration,
};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio::{
    fs::OpenOptions,
    io::{AsyncWriteExt, BufWriter},
    sync::watch,
    time::timeout,
};
use url::Url;

const HUGGING_FACE: &str = "https://huggingface.co/";
/// A repository listing is a few kilobytes; a large one with hundreds of
/// files is still far under this.
const METADATA_LIMIT: u64 = 8 * 1024 * 1024;
/// Files fetched whole into memory at pin time — configs, tokenizers,
/// vocabularies — which the repository stores without a SHA-256 of its own.
pub const SMALL_FILE_LIMIT: u64 = 64 * 1024 * 1024;
/// How long a connection may send nothing before it is abandoned. A large
/// weight can take an hour; a silent minute is a stall, not a slow link.
const IDLE: Duration = Duration::from_mins(1);
const MAX_REDIRECTS: usize = 5;
const WRITE_BUFFER: usize = 1024 * 1024;
const HASH_BUFFER: usize = 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0:?} is not a Hugging Face repository name (owner/name)")]
    Repository(String),
    #[error("{0:?} is not a commit, branch or tag name")]
    Revision(String),
    #[error("{0:?} is not a safe file path inside a repository")]
    Path(String),
    #[error("{0:?} is not a SHA-256 digest")]
    Digest(String),
    #[error("Hugging Face could not be reached. Check the connection and try again.")]
    Network,
    #[error("The connection stalled: nothing arrived for a minute.")]
    Stalled,
    #[error("Hugging Face has no such repository or file at this revision.")]
    NotFound,
    #[error("This repository requires signing in to Hugging Face, which ClipMill does not do.")]
    Gated,
    #[error("Hugging Face refused the request (HTTP {0}).")]
    Status(u16),
    #[error("Hugging Face answered with something that is not what was asked for.")]
    Protocol,
    #[error("{path} arrived with {received} bytes; {expected} are pinned.")]
    Size {
        path: String,
        expected: u64,
        received: u64,
    },
    #[error("{0} does not match its pinned SHA-256, so it was discarded.")]
    Mismatch(String),
    #[error("{0} is larger than a file fetched into memory may be.")]
    TooLarge(String),
    #[error("The download could not be written: {0}")]
    Io(String),
    #[error("The download was cancelled.")]
    Cancelled,
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

/// What a repository holds at one commit.
#[derive(Clone, Debug)]
pub struct Repository {
    pub id: String,
    /// The commit the answer describes. Always a full hash, whatever name the
    /// caller asked with, and the only revision a manifest may pin.
    pub commit: String,
    /// The licence identifier the repository declares, lowercased, as the
    /// hub spells it (`apache-2.0`, `mit`, …). `None` when it declares none,
    /// or more than one.
    pub license: Option<String>,
    pub gated: bool,
    pub private: bool,
    pub library: Option<String>,
    pub pipeline: Option<String>,
    pub tags: Vec<String>,
    pub files: Vec<RepositoryFile>,
}

/// One file in a repository listing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryFile {
    pub path: String,
    pub bytes: u64,
    /// The SHA-256 the repository's large-file storage records. Present for
    /// weights; absent for small files kept in git, whose digest is computed
    /// from the bytes when they are pinned.
    pub sha256: Option<String>,
}

/// One pinned file to fetch. Every field was pinned before this was built.
#[derive(Clone, Copy, Debug)]
pub struct PinnedFile<'a> {
    pub repo: &'a str,
    pub revision: &'a str,
    pub path: &'a str,
    pub sha256: &'a str,
    pub bytes: u64,
}

/// The transport. Cheap to clone; one connection pool per process.
#[derive(Clone, Debug)]
pub struct Hub {
    client: reqwest::Client,
    base: Url,
}

impl Hub {
    /// The public Hugging Face hub, over HTTPS only, following HTTPS
    /// redirects to its content-delivery hosts and nowhere else.
    pub fn new() -> Result<Self, Error> {
        Self::build(HUGGING_FACE, true)
    }

    /// A loopback fixture for tests. Plain HTTP is allowed because a test
    /// server has no certificate; nothing outside a test should call this.
    #[doc(hidden)]
    pub fn with_test_endpoint(base: &str) -> Result<Self, Error> {
        Self::build(base, false)
    }

    fn build(base: &str, https_only: bool) -> Result<Self, Error> {
        // Select the supported ring provider explicitly; keep the normal
        // platform certificate verifier. Another installed provider wins.
        let _already_installed = rustls::crypto::ring::default_provider().install_default();
        let redirects = reqwest::redirect::Policy::custom(move |attempt| {
            if attempt.previous().len() >= MAX_REDIRECTS {
                attempt.error("too many redirects")
            } else if https_only && attempt.url().scheme() != "https" {
                attempt.error("a redirect left HTTPS")
            } else {
                attempt.follow()
            }
        });
        let client = reqwest::Client::builder()
            .no_proxy()
            .redirect(redirects)
            .https_only(https_only)
            .connect_timeout(Duration::from_secs(15))
            .user_agent(concat!("ClipMill/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|_| Error::Network)?;
        let base = Url::parse(base).map_err(|_| Error::Protocol)?;
        Ok(Self { client, base })
    }

    /// What `repo` holds at `revision`, with every file's size and, where the
    /// repository records one, its SHA-256.
    pub async fn repository(&self, repo: &str, revision: &str) -> Result<Repository, Error> {
        check_repository(repo)?;
        check_revision(revision)?;
        let mut url = self
            .base
            .join(&format!("api/models/{repo}/revision/{revision}"))
            .map_err(|_| Error::Protocol)?;
        url.set_query(Some("blobs=true"));
        let response = checked(
            self.client
                .get(url)
                .send()
                .await
                .map_err(|_| Error::Network)?,
        )?;
        let body = read_limited(response, METADATA_LIMIT, "the repository listing").await?;
        let raw: RawRepository = serde_json::from_slice(&body).map_err(|_| Error::Protocol)?;
        let commit = raw
            .sha
            .filter(|sha| is_commit(sha))
            .ok_or(Error::Protocol)?;
        // Asked for a commit, answered about another: not an answer to use.
        if is_commit(revision) && commit != revision {
            return Err(Error::Protocol);
        }
        let license = raw
            .card
            .and_then(|card| match card.license {
                serde_json::Value::String(value) => Some(value),
                _ => None,
            })
            .or_else(|| {
                let declared = raw
                    .tags
                    .iter()
                    .filter_map(|tag| tag.strip_prefix("license:"))
                    .collect::<Vec<_>>();
                match declared.as_slice() {
                    [only] => Some((*only).to_owned()),
                    _ => None,
                }
            })
            .map(|license| license.to_ascii_lowercase());
        let mut files = Vec::with_capacity(raw.siblings.len());
        for sibling in raw.siblings {
            let Some(bytes) = sibling.size.or(sibling.lfs.as_ref().map(|lfs| lfs.size)) else {
                return Err(Error::Protocol);
            };
            let sha256 = match sibling.lfs {
                Some(lfs) if is_sha256(&lfs.sha256) && lfs.size == bytes => Some(lfs.sha256),
                Some(_) => return Err(Error::Protocol),
                None => None,
            };
            files.push(RepositoryFile {
                path: sibling.rfilename,
                bytes,
                sha256,
            });
        }
        files.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(Repository {
            id: if raw.id.is_empty() {
                repo.to_owned()
            } else {
                raw.id
            },
            commit,
            license,
            gated: !matches!(
                raw.gated,
                serde_json::Value::Bool(false) | serde_json::Value::Null
            ),
            private: raw.private,
            library: raw.library_name,
            pipeline: raw.pipeline_tag,
            tags: raw.tags,
            files,
        })
    }

    /// A small file, whole, for pinning: the caller hashes what arrives and
    /// the digest becomes the pin. `bytes` is the size the listing stated,
    /// and anything else is refused.
    pub async fn fetch_small(
        &self,
        repo: &str,
        commit: &str,
        path: &str,
        bytes: u64,
    ) -> Result<Vec<u8>, Error> {
        if bytes > SMALL_FILE_LIMIT {
            return Err(Error::TooLarge(path.to_owned()));
        }
        let url = self.file_url(repo, commit, path)?;
        let response = checked(
            self.client
                .get(url)
                .send()
                .await
                .map_err(|_| Error::Network)?,
        )?;
        let body = read_limited(response, bytes, path).await?;
        let received = body.len() as u64;
        if received != bytes {
            return Err(Error::Size {
                path: path.to_owned(),
                expected: bytes,
                received,
            });
        }
        Ok(body)
    }

    /// Fetch one pinned file into `partial`, resuming what an earlier attempt
    /// left there, and return only once the whole file hashes to its pin.
    ///
    /// The caller renames `partial` into place; until then the bytes have no
    /// name anybody loads from. A file that arrives the wrong size or with the
    /// wrong digest is discarded rather than kept for a resume, since resuming
    /// it would only extend a wrong file. A cancelled or interrupted download
    /// keeps what arrived: those bytes were hashed on the way in and are
    /// hashed again from disk before the next attempt trusts them.
    pub async fn fetch(
        &self,
        file: PinnedFile<'_>,
        partial: &Path,
        progress: &mut (dyn FnMut(u64) + Send),
        cancel: &mut watch::Receiver<bool>,
    ) -> Result<(), Error> {
        check_repository(file.repo)?;
        if !is_commit(file.revision) {
            return Err(Error::Revision(file.revision.to_owned()));
        }
        check_path(file.path)?;
        if !is_sha256(file.sha256) {
            return Err(Error::Digest(file.sha256.to_owned()));
        }
        let url = self.file_url(file.repo, file.revision, file.path)?;
        // At most two passes: one that may resume, and one from nothing if a
        // resumed file turns out not to match.
        for pass in 0..2 {
            if *cancel.borrow() {
                return Err(Error::Cancelled);
            }
            let (mut hasher, mut have) = if pass == 0 {
                hash_prefix(partial, file.bytes).await?
            } else {
                discard(partial).await?;
                (Sha256::new(), 0)
            };
            // Said before anything is fetched, so a resumed download shows
            // what it already has rather than appearing to start at zero.
            progress(have);
            if have < file.bytes {
                have = match self
                    .transfer(&url, file, partial, &mut hasher, have, progress, cancel)
                    .await
                {
                    // The kept bytes were refused as a prefix: start over.
                    Err(Error::Status(416)) if pass == 0 => continue,
                    other => other?,
                };
            }
            if have != file.bytes {
                // A short body with no error: the connection ended early.
                // What arrived is kept, so the next attempt resumes.
                return Err(Error::Network);
            }
            if hex(&hasher.finalize()) == file.sha256 {
                return Ok(());
            }
        }
        discard(partial).await?;
        Err(Error::Mismatch(file.path.to_owned()))
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "one transfer's state, threaded through rather than stored"
    )]
    async fn transfer(
        &self,
        url: &Url,
        file: PinnedFile<'_>,
        partial: &Path,
        hasher: &mut Sha256,
        mut have: u64,
        progress: &mut (dyn FnMut(u64) + Send),
        cancel: &mut watch::Receiver<bool>,
    ) -> Result<u64, Error> {
        let mut request = self.client.get(url.clone());
        if have > 0 {
            request = request.header(reqwest::header::RANGE, format!("bytes={have}-"));
        }
        let response = tokio::select! {
            biased;
            () = cancelled(cancel) => return Err(Error::Cancelled),
            sent = request.send() => sent.map_err(|_| Error::Network)?,
        };
        let append = match response.status().as_u16() {
            206 if have > 0 => true,
            200 => {
                // The host ignored the range: it is sending the whole file,
                // so what was kept is dropped and hashing starts over.
                *hasher = Sha256::new();
                have = 0;
                false
            }
            416 => {
                // The kept bytes do not describe a prefix of this file.
                discard(partial).await?;
                return Err(Error::Status(416));
            }
            _ => {
                checked(response)?;
                return Err(Error::Protocol);
            }
        };
        let response = checked(response)?;
        if let Some(length) = response.content_length()
            && have.saturating_add(length) != file.bytes
        {
            if !append {
                discard(partial).await?;
            }
            return Err(Error::Size {
                path: file.path.to_owned(),
                expected: file.bytes,
                received: have.saturating_add(length),
            });
        }
        let handle = OpenOptions::new()
            .create(true)
            .write(true)
            .append(append)
            .truncate(!append)
            .open(partial)
            .await?;
        let mut writer = BufWriter::with_capacity(WRITE_BUFFER, handle);
        let mut response = response;
        loop {
            let next = tokio::select! {
                biased;
                () = cancelled(cancel) => {
                    writer.flush().await?;
                    return Err(Error::Cancelled);
                }
                next = timeout(IDLE, response.chunk()) => next,
            };
            let chunk = match next {
                Err(_) => {
                    writer.flush().await?;
                    return Err(Error::Stalled);
                }
                Ok(Err(_)) => {
                    writer.flush().await?;
                    return Err(Error::Network);
                }
                Ok(Ok(None)) => break,
                Ok(Ok(Some(chunk))) => chunk,
            };
            have = have.saturating_add(chunk.len() as u64);
            if have > file.bytes {
                drop(writer);
                discard(partial).await?;
                return Err(Error::Size {
                    path: file.path.to_owned(),
                    expected: file.bytes,
                    received: have,
                });
            }
            hasher.update(&chunk);
            writer.write_all(&chunk).await?;
            progress(have);
        }
        writer.flush().await?;
        writer.get_ref().sync_all().await?;
        Ok(have)
    }

    fn file_url(&self, repo: &str, revision: &str, path: &str) -> Result<Url, Error> {
        check_repository(repo)?;
        check_revision(revision)?;
        check_path(path)?;
        // Every part was checked against a character set that needs no
        // escaping, so joining cannot turn a path into a query or a host.
        self.base
            .join(&format!("{repo}/resolve/{revision}/{path}"))
            .map_err(|_| Error::Protocol)
    }
}

/// Resolves when a cancellation is requested, or the requester went away.
async fn cancelled(cancel: &mut watch::Receiver<bool>) {
    loop {
        if *cancel.borrow_and_update() {
            return;
        }
        if cancel.changed().await.is_err() {
            return;
        }
    }
}

fn checked(response: reqwest::Response) -> Result<reqwest::Response, Error> {
    match response.status().as_u16() {
        200..=299 => Ok(response),
        401 | 403 => Err(Error::Gated),
        404 => Err(Error::NotFound),
        code => Err(Error::Status(code)),
    }
}

async fn read_limited(
    mut response: reqwest::Response,
    limit: u64,
    what: &str,
) -> Result<Vec<u8>, Error> {
    if response
        .content_length()
        .is_some_and(|length| length > limit)
    {
        return Err(Error::TooLarge(what.to_owned()));
    }
    let mut body = Vec::new();
    loop {
        let chunk = match timeout(IDLE, response.chunk()).await {
            Err(_) => return Err(Error::Stalled),
            Ok(Err(_)) => return Err(Error::Network),
            Ok(Ok(None)) => return Ok(body),
            Ok(Ok(Some(chunk))) => chunk,
        };
        if (body.len() + chunk.len()) as u64 > limit {
            return Err(Error::TooLarge(what.to_owned()));
        }
        body.extend_from_slice(&chunk);
    }
}

/// Hash what an earlier attempt left, so the final digest covers every byte
/// rather than only the ones this attempt received. A file longer than the pin
/// cannot be a prefix of it and is dropped.
async fn hash_prefix(partial: &Path, expected: u64) -> Result<(Sha256, u64), Error> {
    let path: PathBuf = partial.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let mut hasher = Sha256::new();
        let mut handle = match std::fs::File::open(&path) {
            Ok(handle) => handle,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok((hasher, 0)),
            Err(error) => return Err(Error::from(error)),
        };
        let length = handle.metadata()?.len();
        if length > expected {
            drop(handle);
            std::fs::remove_file(&path)?;
            return Ok((hasher, 0));
        }
        let mut buffer = vec![0_u8; HASH_BUFFER];
        let mut total = 0_u64;
        loop {
            let read = handle.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
            total += read as u64;
        }
        Ok((hasher, total))
    })
    .await
    .map_err(|error| Error::Io(error.to_string()))?
}

async fn discard(partial: &Path) -> Result<(), Error> {
    match tokio::fs::remove_file(partial).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

/// `owner/name`, each part a hub identifier.
pub fn check_repository(repo: &str) -> Result<(), Error> {
    let valid = repo.split_once('/').is_some_and(|(owner, name)| {
        identifier(owner, 96) && identifier(name, 96) && !name.contains('/')
    });
    if valid {
        Ok(())
    } else {
        Err(Error::Repository(repo.to_owned()))
    }
}

/// A full commit hash, or a branch or tag name with no path separators.
pub fn check_revision(revision: &str) -> Result<(), Error> {
    if is_commit(revision) || identifier(revision, 128) {
        Ok(())
    } else {
        Err(Error::Revision(revision.to_owned()))
    }
}

/// A relative path whose every part is a plain file name.
pub fn check_path(path: &str) -> Result<(), Error> {
    let valid = !path.is_empty()
        && path.len() <= 512
        && path.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part.bytes().all(|byte| {
                    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b'+')
                })
        });
    if valid {
        Ok(())
    } else {
        Err(Error::Path(path.to_owned()))
    }
}

fn identifier(value: &str, limit: usize) -> bool {
    !value.is_empty()
        && value.len() <= limit
        && value
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        && !value.contains("..")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
}

pub fn is_commit(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

pub fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

/// Lowercase hex, as every pin in the registry is written.
pub fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut out, byte| {
            let _ = write!(out, "{byte:02x}");
            out
        })
}

/// The SHA-256 of some bytes, as a pin.
pub fn sha256_hex(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

#[derive(Deserialize)]
struct RawRepository {
    #[serde(default)]
    id: String,
    sha: Option<String>,
    #[serde(default)]
    gated: serde_json::Value,
    #[serde(default)]
    private: bool,
    #[serde(default, rename = "cardData")]
    card: Option<RawCard>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    library_name: Option<String>,
    #[serde(default)]
    pipeline_tag: Option<String>,
    #[serde(default)]
    siblings: Vec<RawSibling>,
}

#[derive(Deserialize)]
struct RawCard {
    #[serde(default)]
    license: serde_json::Value,
}

#[derive(Deserialize)]
struct RawSibling {
    rfilename: String,
    #[serde(default)]
    size: Option<u64>,
    #[serde(default)]
    lfs: Option<RawLfs>,
}

#[derive(Deserialize)]
struct RawLfs {
    sha256: String,
    size: u64,
}

#[cfg(test)]
mod tests;
