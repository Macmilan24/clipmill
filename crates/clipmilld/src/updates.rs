//! Whether a newer ClipMill release is out.
//!
//! One narrow question, asked only when the app asks it: which of the
//! project's published releases on GitHub is the newest. Only its version is
//! taken from the answer, and the app links to that version's page itself.
//! Nothing is downloaded or installed and no credential is sent. The daemon
//! counts each look-up as a network operation in the Local Lock.

use std::{fmt, time::Duration};

use serde::Deserialize;

/// Where releases are published: the workspace's `repository`.
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
/// A release document is its assets and notes: kilobytes, rarely more.
const ANSWER_LIMIT: usize = 1024 * 1024;
/// Under the shell's own wait for a daemon reply.
const DEADLINE: Duration = Duration::from_secs(20);

/// A release version as the tags write it: three numbers.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

impl Version {
    /// `0.3.0` or `v0.3.0`. A pre-release or build suffix is not a release.
    pub(crate) fn parse(text: &str) -> Option<Self> {
        let mut parts = text.strip_prefix('v').unwrap_or(text).split('.');
        let mut number = || {
            let part = parts.next()?;
            let digits = !part.is_empty() && part.len() <= 9;
            if digits && part.bytes().all(|byte| byte.is_ascii_digit()) {
                part.parse().ok()
            } else {
                None
            }
        };
        let version = Self {
            major: number()?,
            minor: number()?,
            patch: number()?,
        };
        parts.next().is_none().then_some(version)
    }

    /// This build's version.
    pub(crate) fn current() -> Option<Self> {
        Self::parse(env!("CARGO_PKG_VERSION"))
    }
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum CheckError {
    #[error("GitHub could not be reached. ClipMill will ask again tomorrow.")]
    Network,
    #[error("GitHub refused the request (HTTP {0}).")]
    Status(u16),
    #[error("GitHub's answer did not name a released version.")]
    Answer,
}

pub(crate) struct ReleaseCheck {
    client: reqwest::Client,
    url: String,
}

impl ReleaseCheck {
    /// The project's newest release on GitHub, over HTTPS only.
    pub(crate) fn new() -> Result<Self, CheckError> {
        let project = REPOSITORY
            .strip_prefix("https://github.com/")
            .ok_or(CheckError::Answer)?;
        Self::build(
            &format!("https://api.github.com/repos/{project}/releases/latest"),
            true,
        )
    }

    /// A loopback fixture for tests, which has no certificate.
    #[cfg(test)]
    fn with_endpoint(url: &str) -> Result<Self, CheckError> {
        Self::build(url, false)
    }

    fn build(url: &str, https_only: bool) -> Result<Self, CheckError> {
        // Select the supported ring provider explicitly; keep the platform's
        // certificate verifier. Another installed provider wins.
        let _already_installed = rustls::crypto::ring::default_provider().install_default();
        let client = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::limited(3))
            .https_only(https_only)
            .connect_timeout(Duration::from_secs(10))
            .timeout(DEADLINE)
            // GitHub refuses a request without one. The version stays out:
            // which build is asking is nobody's business.
            .user_agent("ClipMill")
            .build()
            .map_err(|_| CheckError::Network)?;
        Ok(Self {
            client,
            url: url.to_owned(),
        })
    }

    /// The newest published release: not a draft and not a pre-release.
    pub(crate) async fn latest(&self) -> Result<Version, CheckError> {
        let mut response = self
            .client
            .get(&self.url)
            .header(reqwest::header::ACCEPT, "application/vnd.github+json")
            .send()
            .await
            .map_err(|_| CheckError::Network)?;
        if !response.status().is_success() {
            return Err(CheckError::Status(response.status().as_u16()));
        }
        let mut body = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|_| CheckError::Network)? {
            if body.len() + chunk.len() > ANSWER_LIMIT {
                return Err(CheckError::Answer);
            }
            body.extend_from_slice(&chunk);
        }
        released_version(&body)
    }
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
}

fn released_version(body: &[u8]) -> Result<Version, CheckError> {
    let release: Release = serde_json::from_slice(body).map_err(|_| CheckError::Answer)?;
    if release.draft || release.prerelease {
        return Err(CheckError::Answer);
    }
    Version::parse(&release.tag_name).ok_or(CheckError::Answer)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::io::{Read as _, Write as _};

    use super::{CheckError, ReleaseCheck, Version, released_version};

    #[test]
    fn only_a_plain_three_part_tag_is_a_release() {
        let version = |text| Version::parse(text).map(|found| found.to_string());
        assert_eq!(version("v0.3.0").as_deref(), Some("0.3.0"));
        assert_eq!(version("12.0.4").as_deref(), Some("12.0.4"));
        for refused in [
            "",
            "v",
            "0.3",
            "0.3.0.1",
            "v0.4.0-rc.1",
            "0.x.0",
            "0..1",
            "+1.0.0",
        ] {
            assert_eq!(version(refused), None, "{refused:?}");
        }
        let (older, newer) = (Version::parse("0.9.9"), Version::parse("0.10.0"));
        assert!(newer > older);
        assert!(Version::current().is_some());
    }

    #[test]
    fn a_draft_or_pre_release_is_not_taken() {
        let body = |tag: &str, prerelease: bool| {
            format!(
                r#"{{"tag_name":"{tag}","draft":false,"prerelease":{prerelease},"body":"notes"}}"#
            )
        };
        assert_eq!(
            released_version(body("v0.4.0", false).as_bytes()).ok(),
            Version::parse("0.4.0")
        );
        assert!(released_version(body("v0.4.0", true).as_bytes()).is_err());
        assert!(released_version(br#"{"tag_name":"nightly"}"#).is_err());
        assert!(released_version(b"<html>").is_err());
    }

    #[tokio::test]
    async fn the_check_reads_one_release_document() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            for status in ["200 OK", "403 Forbidden"] {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0_u8; 4096];
                let read = stream.read(&mut request).unwrap();
                let request = String::from_utf8_lossy(&request[..read]).to_lowercase();
                assert!(request.contains("user-agent: clipmill\r\n"), "{request}");
                assert!(request.contains("accept: application/vnd.github+json"));
                let body = r#"{"tag_name":"v0.4.0","draft":false,"prerelease":false}"#;
                write!(
                    stream,
                    "HTTP/1.1 {status}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                )
                .unwrap();
            }
        });
        let check = ReleaseCheck::with_endpoint(&format!("http://{address}/latest")).unwrap();
        assert_eq!(check.latest().await.ok(), Version::parse("0.4.0"));
        assert!(matches!(check.latest().await, Err(CheckError::Status(403))));
        server.join().unwrap();
    }
}
