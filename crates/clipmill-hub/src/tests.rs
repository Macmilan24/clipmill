//! Loopback protocol tests: no request leaves the machine.
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{
    collections::HashMap,
    fmt::Write as _,
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

use serde_json::json;
use tempfile::TempDir;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::watch,
    time::sleep,
};

use crate::{Error, Hub, PinnedFile, sha256_hex};

const REPO: &str = "ggerganov/whisper.cpp";
const COMMIT: &str = "5359861c739e955e79d9a303bcbc70fb988958b1";

#[derive(Clone)]
enum Route {
    /// Serve these bytes, honouring `Range` unless told not to.
    Bytes {
        body: Vec<u8>,
        ranges: bool,
    },
    /// Send this many bytes of the body, then hold the connection open.
    Stall {
        body: Vec<u8>,
        after: usize,
    },
    Redirect(String),
    Status(u16),
}

#[derive(Clone, Debug)]
struct Seen {
    target: String,
    range: Option<String>,
    authorization: bool,
}

struct Fixture {
    base: String,
    seen: Arc<Mutex<Vec<Seen>>>,
}

impl Fixture {
    async fn start(routes: Vec<(&str, Route)>) -> Self {
        let routes: Arc<HashMap<String, Route>> = Arc::new(
            routes
                .into_iter()
                .map(|(path, route)| (path.to_owned(), route))
                .collect(),
        );
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let base = format!("http://{}/", listener.local_addr().unwrap());
        let seen = Arc::new(Mutex::new(Vec::new()));
        let recorded = Arc::clone(&seen);
        tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    return;
                };
                let routes = Arc::clone(&routes);
                let recorded = Arc::clone(&recorded);
                tokio::spawn(async move { serve(stream, &routes, &recorded).await });
            }
        });
        Self { base, seen }
    }

    fn hub(&self) -> Hub {
        Hub::with_test_endpoint(&self.base).unwrap()
    }

    fn seen(&self) -> Vec<Seen> {
        self.seen.lock().unwrap().clone()
    }
}

async fn serve(mut stream: TcpStream, routes: &HashMap<String, Route>, seen: &Mutex<Vec<Seen>>) {
    let mut received = Vec::new();
    let mut buffer = [0_u8; 4096];
    let end = loop {
        let Ok(count) = stream.read(&mut buffer).await else {
            return;
        };
        if count == 0 {
            return;
        }
        received.extend_from_slice(&buffer[..count]);
        if let Some(position) = received.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
            break position;
        }
    };
    let head = String::from_utf8_lossy(&received[..end]).into_owned();
    let mut lines = head.lines();
    let target = lines
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or_default()
        .to_owned();
    let mut range = None;
    let mut authorization = false;
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            match name.trim().to_ascii_lowercase().as_str() {
                "range" => range = Some(value.trim().to_owned()),
                "authorization" => authorization = true,
                _ => {}
            }
        }
    }
    seen.lock().unwrap().push(Seen {
        target: target.clone(),
        range: range.clone(),
        authorization,
    });
    let path = target.split('?').next().unwrap_or_default();
    let reply = |status: u16, headers: &[(&str, String)], body: &[u8]| {
        let mut head = format!(
            "HTTP/1.1 {status} Fixture\r\nContent-Length: {}\r\nConnection: close\r\n",
            body.len()
        );
        for (name, value) in headers {
            write!(head, "{name}: {value}\r\n").unwrap();
        }
        head.push_str("\r\n");
        let mut bytes = head.into_bytes();
        bytes.extend_from_slice(body);
        bytes
    };
    match routes.get(path).cloned() {
        None => {
            let _ = stream.write_all(&reply(404, &[], b"")).await;
        }
        Some(Route::Status(code)) => {
            let _ = stream.write_all(&reply(code, &[], b"")).await;
        }
        Some(Route::Redirect(location)) => {
            let _ = stream
                .write_all(&reply(302, &[("Location", location)], b""))
                .await;
        }
        Some(Route::Bytes { body, ranges }) => {
            let start = range
                .as_deref()
                .filter(|_| ranges)
                .and_then(|value| value.strip_prefix("bytes="))
                .and_then(|value| value.strip_suffix('-'))
                .and_then(|value| value.parse::<usize>().ok());
            let bytes = match start {
                Some(start) if start >= body.len() => reply(416, &[], b""),
                Some(start) => reply(
                    206,
                    &[(
                        "Content-Range",
                        format!("bytes {start}-{}/{}", body.len() - 1, body.len()),
                    )],
                    &body[start..],
                ),
                None => reply(200, &[], &body),
            };
            let _ = stream.write_all(&bytes).await;
        }
        Some(Route::Stall { body, after }) => {
            let head = format!(
                "HTTP/1.1 200 Fixture\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(head.as_bytes()).await;
            let _ = stream.write_all(&body[..after]).await;
            let _ = stream.flush().await;
            sleep(Duration::from_secs(30)).await;
        }
    }
    let _ = stream.shutdown().await;
}

fn weights(length: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from(index * 31 % 251).unwrap())
        .collect()
}

fn pinned<'a>(path: &'a str, body: &[u8], digest: &'a str) -> PinnedFile<'a> {
    PinnedFile {
        repo: REPO,
        revision: COMMIT,
        path,
        sha256: digest,
        bytes: body.len() as u64,
    }
}

fn resolve(path: &str) -> String {
    format!("/{REPO}/resolve/{COMMIT}/{path}")
}

async fn fetch(hub: &Hub, file: PinnedFile<'_>, partial: &Path) -> (Result<(), Error>, Vec<u64>) {
    let (_keep, mut cancel) = watch::channel(false);
    let mut reports = Vec::new();
    let result = hub
        .fetch(file, partial, &mut |done| reports.push(done), &mut cancel)
        .await;
    (result, reports)
}

#[tokio::test]
async fn a_pinned_file_arrives_verified_with_progress() {
    let body = weights(300_000);
    let digest = sha256_hex(&body);
    let fixture = Fixture::start(vec![(
        &resolve("ggml-base.bin"),
        Route::Bytes {
            body: body.clone(),
            ranges: true,
        },
    )])
    .await;
    let directory = TempDir::new().unwrap();
    let partial = directory.path().join("ggml-base.bin.part");

    let (result, reports) = fetch(
        &fixture.hub(),
        pinned("ggml-base.bin", &body, &digest),
        &partial,
    )
    .await;

    result.expect("the download verifies");
    assert_eq!(std::fs::read(&partial).unwrap(), body);
    assert_eq!(reports.last(), Some(&(body.len() as u64)));
    assert!(reports.windows(2).all(|pair| pair[0] <= pair[1]));
    let seen = fixture.seen();
    assert_eq!(seen.len(), 1);
    assert!(seen[0].range.is_none());
    assert!(!seen[0].authorization, "no credential is ever sent");
}

#[tokio::test]
async fn an_interrupted_download_resumes_from_what_is_kept() {
    let body = weights(200_000);
    let digest = sha256_hex(&body);
    let fixture = Fixture::start(vec![(
        &resolve("model.safetensors"),
        Route::Bytes {
            body: body.clone(),
            ranges: true,
        },
    )])
    .await;
    let directory = TempDir::new().unwrap();
    let partial = directory.path().join("model.part");
    std::fs::write(&partial, &body[..75_000]).unwrap();

    let (result, reports) = fetch(
        &fixture.hub(),
        pinned("model.safetensors", &body, &digest),
        &partial,
    )
    .await;

    result.expect("a resumed file still hashes to its pin");
    assert_eq!(std::fs::read(&partial).unwrap(), body);
    assert_eq!(
        reports.first(),
        Some(&75_000),
        "progress starts from what was kept"
    );
    assert_eq!(fixture.seen()[0].range.as_deref(), Some("bytes=75000-"));
}

#[tokio::test]
async fn a_host_that_ignores_the_range_restarts_the_file() {
    let body = weights(120_000);
    let digest = sha256_hex(&body);
    let fixture = Fixture::start(vec![(
        &resolve("model.safetensors"),
        Route::Bytes {
            body: body.clone(),
            ranges: false,
        },
    )])
    .await;
    let directory = TempDir::new().unwrap();
    let partial = directory.path().join("model.part");
    std::fs::write(&partial, &body[..50_000]).unwrap();

    let (result, _) = fetch(
        &fixture.hub(),
        pinned("model.safetensors", &body, &digest),
        &partial,
    )
    .await;

    result.expect("a whole-file answer replaces the kept prefix");
    assert_eq!(std::fs::read(&partial).unwrap(), body);
}

#[tokio::test]
async fn a_corrupt_kept_prefix_is_discarded_and_fetched_again() {
    let body = weights(90_000);
    let digest = sha256_hex(&body);
    let fixture = Fixture::start(vec![(
        &resolve("model.safetensors"),
        Route::Bytes {
            body: body.clone(),
            ranges: true,
        },
    )])
    .await;
    let directory = TempDir::new().unwrap();
    let partial = directory.path().join("model.part");
    let mut corrupt = body[..40_000].to_vec();
    corrupt[123] ^= 0xff;
    std::fs::write(&partial, &corrupt).unwrap();

    let (result, _) = fetch(
        &fixture.hub(),
        pinned("model.safetensors", &body, &digest),
        &partial,
    )
    .await;

    result.expect("the second pass starts from nothing");
    assert_eq!(std::fs::read(&partial).unwrap(), body);
    let seen = fixture.seen();
    assert_eq!(seen.len(), 2);
    assert_eq!(seen[0].range.as_deref(), Some("bytes=40000-"));
    assert!(seen[1].range.is_none());
}

#[tokio::test]
async fn bytes_that_do_not_match_the_pin_never_survive() {
    let body = weights(10_000);
    let wrong = sha256_hex(b"something else entirely");
    let fixture = Fixture::start(vec![(
        &resolve("model.safetensors"),
        Route::Bytes {
            body: body.clone(),
            ranges: true,
        },
    )])
    .await;
    let directory = TempDir::new().unwrap();
    let partial = directory.path().join("model.part");

    let (result, _) = fetch(
        &fixture.hub(),
        pinned("model.safetensors", &body, &wrong),
        &partial,
    )
    .await;

    assert!(matches!(result, Err(Error::Mismatch(_))), "{result:?}");
    assert!(
        !partial.exists(),
        "a mismatched file is not kept for a resume"
    );
}

#[tokio::test]
async fn a_body_longer_than_the_pin_is_refused_before_it_is_kept() {
    let body = weights(10_000);
    let fixture = Fixture::start(vec![(
        &resolve("model.safetensors"),
        Route::Bytes {
            body: body.clone(),
            ranges: true,
        },
    )])
    .await;
    let directory = TempDir::new().unwrap();
    let partial = directory.path().join("model.part");
    let digest = sha256_hex(&body[..5_000]);

    let (result, _) = fetch(
        &fixture.hub(),
        pinned("model.safetensors", &body[..5_000], &digest),
        &partial,
    )
    .await;

    assert!(matches!(result, Err(Error::Size { .. })), "{result:?}");
    assert!(!partial.exists());
}

#[tokio::test]
async fn redirects_to_the_content_host_are_followed() {
    let body = weights(20_000);
    let digest = sha256_hex(&body);
    let fixture = Fixture::start(vec![
        (
            &resolve("ggml-tiny.bin"),
            Route::Redirect("/cdn/blob".to_owned()),
        ),
        (
            "/cdn/blob",
            Route::Bytes {
                body: body.clone(),
                ranges: true,
            },
        ),
    ])
    .await;
    let directory = TempDir::new().unwrap();
    let partial = directory.path().join("tiny.part");

    let (result, _) = fetch(
        &fixture.hub(),
        pinned("ggml-tiny.bin", &body, &digest),
        &partial,
    )
    .await;

    result.expect("the redirected body verifies");
    assert_eq!(fixture.seen().len(), 2);
}

#[tokio::test]
async fn cancelling_keeps_what_arrived_for_the_next_attempt() {
    let body = weights(64_000);
    let digest = sha256_hex(&body);
    let fixture = Fixture::start(vec![(
        &resolve("model.safetensors"),
        Route::Stall {
            body: body.clone(),
            after: 16_000,
        },
    )])
    .await;
    let directory = TempDir::new().unwrap();
    let partial = directory.path().join("model.part");
    let (stop, mut cancel) = watch::channel(false);
    let hub = fixture.hub();
    let file = pinned("model.safetensors", &body, &digest);

    let download = async {
        let mut seen = 0;
        hub.fetch(file, &partial, &mut |done| seen = done, &mut cancel)
            .await
    };
    let cancel_later = async {
        sleep(Duration::from_millis(300)).await;
        stop.send(true).unwrap();
    };
    let (result, ()) = tokio::join!(download, cancel_later);

    assert!(matches!(result, Err(Error::Cancelled)), "{result:?}");
    assert_eq!(std::fs::metadata(&partial).unwrap().len(), 16_000);
}

#[tokio::test]
async fn a_repository_that_needs_signing_in_is_named_as_such() {
    let fixture = Fixture::start(vec![
        (&resolve("gated.bin"), Route::Status(401)),
        (&resolve("missing.bin"), Route::Status(404)),
    ])
    .await;
    let directory = TempDir::new().unwrap();
    let digest = sha256_hex(b"x");
    let body = [0_u8; 1];

    let (gated, _) = fetch(
        &fixture.hub(),
        pinned("gated.bin", &body, &digest),
        &directory.path().join("a"),
    )
    .await;
    assert!(matches!(gated, Err(Error::Gated)), "{gated:?}");
    let (missing, _) = fetch(
        &fixture.hub(),
        pinned("missing.bin", &body, &digest),
        &directory.path().join("b"),
    )
    .await;
    assert!(matches!(missing, Err(Error::NotFound)), "{missing:?}");
}

#[tokio::test]
async fn a_repository_listing_names_its_commit_licence_and_digests() {
    let listing = json!({
        "id": REPO,
        "sha": COMMIT,
        "gated": false,
        "private": false,
        "pipeline_tag": "automatic-speech-recognition",
        "cardData": {"license": "MIT"},
        "tags": ["automatic-speech-recognition", "license:mit"],
        "siblings": [
            {"rfilename": "README.md", "blobId": "bb07", "size": 3196},
            {"rfilename": "ggml-base.bin", "blobId": "1799", "size": 147_951_465_u64,
             "lfs": {"sha256": "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
                     "size": 147_951_465_u64, "pointerSize": 134}}
        ]
    });
    let fixture = Fixture::start(vec![(
        &format!("/api/models/{REPO}/revision/main"),
        Route::Bytes {
            body: listing.to_string().into_bytes(),
            ranges: false,
        },
    )])
    .await;

    let repository = fixture
        .hub()
        .repository(REPO, "main")
        .await
        .expect("listing");

    assert_eq!(repository.commit, COMMIT, "a branch resolves to its commit");
    assert_eq!(repository.license.as_deref(), Some("mit"));
    assert!(!repository.gated);
    assert_eq!(
        repository.pipeline.as_deref(),
        Some("automatic-speech-recognition")
    );
    assert_eq!(repository.files.len(), 2);
    let weights = &repository.files[1];
    assert_eq!(weights.path, "ggml-base.bin");
    assert_eq!(weights.bytes, 147_951_465);
    assert_eq!(
        weights.sha256.as_deref(),
        Some("60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe")
    );
    assert!(
        repository.files[0].sha256.is_none(),
        "git files carry no SHA-256"
    );
    assert!(fixture.seen()[0].target.ends_with("?blobs=true"));
}

#[tokio::test]
async fn an_answer_about_another_commit_is_not_used() {
    let other = "f".repeat(40);
    let fixture = Fixture::start(vec![(
        &format!("/api/models/{REPO}/revision/{COMMIT}"),
        Route::Bytes {
            body: json!({"sha": other, "siblings": []})
                .to_string()
                .into_bytes(),
            ranges: false,
        },
    )])
    .await;

    let refused = fixture.hub().repository(REPO, COMMIT).await;

    assert!(matches!(refused, Err(Error::Protocol)), "{refused:?}");
}

#[tokio::test]
async fn a_small_file_is_fetched_whole_at_the_stated_size() {
    let body = b"{\"model_type\": \"whisper\"}".to_vec();
    let fixture = Fixture::start(vec![(
        &resolve("config.json"),
        Route::Bytes {
            body: body.clone(),
            ranges: false,
        },
    )])
    .await;
    let hub = fixture.hub();

    let fetched = hub
        .fetch_small(REPO, COMMIT, "config.json", body.len() as u64)
        .await
        .expect("fetched");
    assert_eq!(fetched, body);

    let wrong_size = hub
        .fetch_small(REPO, COMMIT, "config.json", body.len() as u64 + 1)
        .await;
    assert!(
        matches!(wrong_size, Err(Error::Size { .. })),
        "{wrong_size:?}"
    );
}

#[tokio::test]
async fn names_that_could_escape_are_refused_before_any_request() {
    let fixture = Fixture::start(Vec::new()).await;
    let hub = fixture.hub();
    let directory = TempDir::new().unwrap();
    let digest = sha256_hex(b"x");
    for (repo, path) in [
        ("../etc", "passwd"),
        ("owner/name/extra", "a"),
        ("owner", "a"),
        (REPO, "../../escape"),
        (REPO, "a b"),
        (REPO, "a?query=1"),
        (REPO, "/absolute"),
    ] {
        let file = PinnedFile {
            repo,
            revision: COMMIT,
            path,
            sha256: &digest,
            bytes: 1,
        };
        let (result, _) = fetch(&hub, file, &directory.path().join("x")).await;
        assert!(result.is_err(), "{repo} {path} was accepted");
    }
    let branch = PinnedFile {
        repo: REPO,
        revision: "main",
        path: "a",
        sha256: &digest,
        bytes: 1,
    };
    let (result, _) = fetch(&hub, branch, &directory.path().join("y")).await;
    assert!(
        matches!(result, Err(Error::Revision(_))),
        "a weight is only ever fetched at a commit"
    );
    assert!(hub.repository(REPO, "feature/branch").await.is_err());
    assert!(fixture.seen().is_empty(), "nothing was sent");
}
