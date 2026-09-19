//! Independent protocol tests: synthetic credentials and loopback HTTP only.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::{collections::BTreeMap, fmt::Write as _};

use serde_json::{Value, json};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    task::JoinHandle,
    time::{Duration, timeout},
};
use url::Url;

use crate::{DesktopClient, Error, Metadata, Token, UploadReply, YouTube};

const VIDEO: &str = "abcdefghijk";
const CHANNEL: &str = "UC_fixture_channel";
const SESSION: &str =
    "https://www.googleapis.com/upload/youtube/v3/videos?upload_id=fixture-session";
const ACCESS: &str = "fixture-access-secret";

fn token() -> Token {
    serde_json::from_value(json!({
        "access_token": ACCESS, "refresh_token": "fixture-refresh-secret",
        "expires_in": 3600, "scope": crate::oauth::SCOPE, "token_type": "Bearer",
    }))
    .unwrap()
}

fn config() -> DesktopClient {
    DesktopClient {
        client_id: "fixture.apps.googleusercontent.com".into(),
        client_secret: Some("fixture-client-secret".into()),
    }
}

fn metadata() -> Metadata {
    Metadata {
        title: "Reviewed clip".into(),
        description: "The approved description. #interview".into(),
        tags: vec!["interview".into()],
        made_for_kids: false,
        contains_synthetic_media: false,
    }
}

fn video(privacy: &str) -> Value {
    json!({
        "id": VIDEO, "etag": "\"resource-version-one\"",
        "snippet": {"channelId": CHANNEL},
        "status": {
            "privacyStatus": privacy, "uploadStatus": "processed",
            "license": "creativeCommon", "embeddable": false,
            "publicStatsViewable": false, "selfDeclaredMadeForKids": true,
            "containsSyntheticMedia": true,
        },
        "processingDetails": {"processingStatus": "succeeded"},
    })
}

fn response(status: u16, body: &Value) -> reqwest::Response {
    http::Response::builder()
        .status(status)
        .body(body.to_string())
        .unwrap()
        .into()
}

#[derive(Debug)]
struct Captured {
    method: String,
    target: String,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

enum Reply {
    Http(u16, Vec<(String, String)>, Vec<u8>),
    DropConnection,
}

impl Reply {
    fn json(status: u16, body: &Value) -> Self {
        Self::Http(status, Vec::new(), body.to_string().into_bytes())
    }

    fn header(status: u16, key: &str, value: &str) -> Self {
        Self::Http(status, vec![(key.into(), value.into())], Vec::new())
    }
}

struct Server {
    base: String,
    task: JoinHandle<Vec<Captured>>,
}

impl Server {
    async fn start(replies: Vec<Reply>) -> Self {
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            let mut requests = Vec::new();
            for reply in replies {
                let (mut stream, _) = timeout(Duration::from_secs(5), listener.accept())
                    .await
                    .expect("transport sent expected request")
                    .unwrap();
                requests.push(
                    timeout(Duration::from_secs(5), capture(&mut stream))
                        .await
                        .expect("bounded request read"),
                );
                if let Reply::Http(status, headers, body) = reply {
                    let mut head = format!(
                        "HTTP/1.1 {status} Fixture\r\nContent-Length: {}\r\nConnection: close\r\n",
                        body.len()
                    );
                    for (name, value) in headers {
                        write!(head, "{name}: {value}\r\n").unwrap();
                    }
                    head.push_str("\r\n");
                    stream.write_all(head.as_bytes()).await.unwrap();
                    stream.write_all(&body).await.unwrap();
                }
            }
            requests
        });
        Self { base, task }
    }

    fn api(&self) -> YouTube {
        YouTube::with_test_endpoint(&token(), &self.base).unwrap()
    }

    async fn finish(self) -> Vec<Captured> {
        timeout(Duration::from_secs(8), self.task)
            .await
            .expect("fixture finished")
            .expect("fixture task")
    }
}

async fn capture(stream: &mut TcpStream) -> Captured {
    let mut received = Vec::new();
    let mut buffer = [0_u8; 4096];
    let end = loop {
        let count = stream.read(&mut buffer).await.unwrap();
        assert!(count > 0, "request header ended early");
        received.extend_from_slice(&buffer[..count]);
        if let Some(position) = received.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
            break position + 4;
        }
        assert!(received.len() < 32 * 1024, "request header bounded");
    };
    let head = std::str::from_utf8(&received[..end]).unwrap();
    let mut lines = head.lines();
    let mut first = lines.next().unwrap().split_whitespace();
    let method = first.next().unwrap().into();
    let target = first.next().unwrap().into();
    let headers: BTreeMap<String, String> = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(key, value)| (key.to_ascii_lowercase(), value.trim().into()))
        .collect();
    let length = headers
        .get("content-length")
        .map_or(0, |length| length.parse::<usize>().unwrap());
    assert!(length <= 1024 * 1024, "fixture body bounded");
    while received.len() - end < length {
        let count = stream.read(&mut buffer).await.unwrap();
        assert!(count > 0, "request body ended early");
        received.extend_from_slice(&buffer[..count]);
    }
    Captured {
        method,
        target,
        headers,
        body: received[end..end + length].to_vec(),
    }
}

#[test]
fn token_validation_refuses_partial_grants_empty_credentials_and_other_token_types() {
    assert!(token().validate(true).is_ok());
    for field in ["access_token", "refresh_token", "scope", "token_type"] {
        let mut value = serde_json::to_value(token()).unwrap();
        value[field] = json!("");
        let invalid: Token = serde_json::from_value(value).unwrap();
        assert!(invalid.validate(true).is_err(), "empty {field}");
    }
    let mut partial = token();
    partial.scope = "https://www.googleapis.com/auth/youtube.upload".into();
    assert!(partial.validate(true).is_err());
    partial = token();
    partial.token_type = "MAC".into();
    assert!(partial.validate(true).is_err());
}

#[test]
fn secret_bearing_debug_and_errors_do_not_disclose_credentials() {
    let text = format!("{:?} {:?}", token(), config());
    for secret in [ACCESS, "fixture-refresh-secret", "fixture-client-secret"] {
        assert!(!text.contains(secret));
    }
    assert!(!format!("{}", Error::Authorization).contains(ACCESS));
}

#[tokio::test]
async fn provider_errors_and_oversized_json_never_echo_the_response_body() {
    let error = crate::checked(response(403, &json!({"error": ACCESS})))
        .await
        .unwrap_err();
    assert!(!error.to_string().contains(ACCESS));
    let oversized: reqwest::Response = http::Response::builder()
        .status(200)
        .body(vec![b'x'; 1024 * 1024 + 1])
        .unwrap()
        .into();
    assert!(matches!(
        crate::json::<Value>(oversized).await,
        Err(Error::Protocol)
    ));
}

#[tokio::test]
async fn revoked_refresh_token_requires_reconnection_without_echoing_google_detail() {
    for code in [
        "invalid_grant",
        "invalid_client",
        "unauthorized_client",
        "access_denied",
    ] {
        let result = crate::oauth::token_response(response(
            400,
            &json!({
                "error": code, "error_description": format!("secret {ACCESS} at {SESSION}"),
            }),
        ))
        .await;
        assert!(matches!(result, Err(Error::Authorization)), "{code}");
        let error = result.unwrap_err().to_string();
        assert!(!error.contains(ACCESS));
        assert!(!error.contains(SESSION));
    }
}

#[tokio::test]
async fn incomplete_upload_needs_a_valid_range_and_never_means_completed() {
    let absent: reqwest::Response = http::Response::builder()
        .status(308)
        .body(Vec::<u8>::new())
        .unwrap()
        .into();
    assert_eq!(
        crate::transport::upload_reply(absent, 10).await.unwrap(),
        UploadReply::Incomplete { acknowledged: 0 }
    );
    let full: reqwest::Response = http::Response::builder()
        .status(308)
        .header("Range", "bytes=0-9")
        .body(Vec::<u8>::new())
        .unwrap()
        .into();
    assert_eq!(
        crate::transport::upload_reply(full, 10).await.unwrap(),
        UploadReply::Incomplete { acknowledged: 10 }
    );
    let invalid: reqwest::Response = http::Response::builder()
        .status(308)
        .header("Range", http::HeaderValue::from_bytes(b"\xff").unwrap())
        .body(Vec::<u8>::new())
        .unwrap()
        .into();
    assert!(crate::transport::upload_reply(invalid, 10).await.is_err());
}

#[tokio::test]
async fn completed_receipt_preserves_identity_and_actual_visibility() {
    for privacy in ["private", "unlisted", "public"] {
        assert_eq!(
            crate::transport::upload_reply(response(201, &video(privacy)), 10)
                .await
                .unwrap(),
            UploadReply::Complete {
                video_id: VIDEO.into(),
                channel_id: CHANNEL.into(),
                privacy: privacy.into(),
            }
        );
    }
    assert!(
        crate::transport::upload_reply(response(201, &video("unknown-visibility")), 10)
            .await
            .is_err()
    );
    for status in [404, 410] {
        assert!(matches!(
            crate::transport::upload_reply(response(status, &json!({})), 10).await,
            Err(Error::ExpiredSession)
        ));
    }
}

#[tokio::test]
async fn lost_final_reply_recovers_the_same_session_without_a_second_insert() {
    let server = Server::start(vec![
        Reply::header(200, "Location", SESSION),
        Reply::Http(308, Vec::new(), Vec::new()),
        Reply::header(308, "Range", "bytes=0-262143"),
        Reply::DropConnection,
        Reply::json(201, &video("private")),
    ])
    .await;
    let api = server.api();
    let session = api.begin(&metadata(), 262_147).await.unwrap();
    assert_eq!(session, SESSION);
    assert_eq!(
        api.position(&session, 262_147).await.unwrap(),
        UploadReply::Incomplete { acknowledged: 0 }
    );
    assert_eq!(
        api.chunk(&session, 0, 262_147, vec![42; 262_144])
            .await
            .unwrap(),
        UploadReply::Incomplete {
            acknowledged: 262_144
        }
    );
    assert!(
        api.chunk(&session, 262_144, 262_147, vec![1, 2, 3])
            .await
            .is_err()
    );
    assert_eq!(
        api.position(&session, 262_147).await.unwrap(),
        UploadReply::Complete {
            video_id: VIDEO.into(),
            channel_id: CHANNEL.into(),
            privacy: "private".into(),
        }
    );
    let requests = server.finish().await;
    assert_eq!(
        requests
            .iter()
            .filter(|request| request.method == "POST")
            .count(),
        1
    );
    let creation: Value = serde_json::from_slice(&requests[0].body).unwrap();
    assert_eq!(creation["status"]["privacyStatus"], "private");
    assert_eq!(creation["snippet"]["title"], metadata().title);
    assert!(requests[0].target.contains("notifySubscribers=false"));
    assert_eq!(requests[0].headers["x-upload-content-length"], "262147");
    for request in &requests {
        assert_eq!(request.headers["authorization"], format!("Bearer {ACCESS}"));
    }
    assert_eq!(requests[1].headers["content-range"], "bytes */262147");
    assert_eq!(
        requests[2].headers["content-range"],
        "bytes 0-262143/262147"
    );
    assert_eq!(requests[2].body, vec![42; 262_144]);
    assert_eq!(
        requests[3].headers["content-range"],
        "bytes 262144-262146/262147"
    );
    assert_eq!(requests[3].body, vec![1, 2, 3]);
    assert_eq!(requests[4].target, requests[1].target);
    assert_eq!(requests[4].headers["content-range"], "bytes */262147");
    assert!(requests[4].body.is_empty());
}

#[tokio::test]
async fn hostile_session_and_redirect_do_not_receive_tokens_or_media() {
    let server = Server::start(vec![Reply::header(
        200,
        "Location",
        "https://attacker.invalid/upload/youtube/v3/videos?upload_id=stolen",
    )])
    .await;
    assert!(server.api().begin(&metadata(), 10).await.is_err());
    assert_eq!(server.finish().await.len(), 1);

    let trap = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let redirected = format!("http://{}/capture", trap.local_addr().unwrap());
    let server = Server::start(vec![Reply::header(302, "Location", &redirected)]).await;
    assert!(server.api().begin(&metadata(), 10).await.is_err());
    assert_eq!(server.finish().await.len(), 1);
    assert!(
        timeout(Duration::from_millis(100), trap.accept())
            .await
            .is_err(),
        "redirect destination must receive no connection"
    );
}

#[tokio::test]
async fn channel_identity_comes_from_one_authenticated_channel() {
    for items in [
        json!([]),
        json!([{"id": CHANNEL, "snippet": {"title": "Fixture channel"}}]),
        json!([{}, {}]),
    ] {
        let expected = items.as_array().unwrap().len() == 1;
        let server = Server::start(vec![Reply::json(200, &json!({"items": items}))]).await;
        let result = server.api().channel().await;
        assert_eq!(result.is_ok(), expected);
        if let Ok(channel) = result {
            assert_eq!(channel.id, CHANNEL);
        }
        let requests = server.finish().await;
        assert_eq!(requests[0].method, "GET");
        assert!(requests[0].target.contains("mine=true"));
        assert_eq!(
            requests[0].headers["authorization"],
            format!("Bearer {ACCESS}")
        );
    }
}

#[tokio::test]
async fn publish_uses_resource_etag_and_preserves_status_on_concurrent_change() {
    let server = Server::start(vec![
        Reply::json(
            200,
            &json!({"etag": "list-etag", "items": [video("private")]}),
        ),
        Reply::json(
            412,
            &json!({"error": {"message": "Concurrent Studio change"}}),
        ),
    ])
    .await;
    assert!(server.api().publish(VIDEO, CHANNEL).await.is_err());
    let requests = server.finish().await;
    assert_eq!(requests.len(), 2, "no silent retry with stale status");
    assert_eq!(requests[1].method, "PUT");
    assert_eq!(requests[1].headers["if-match"], "\"resource-version-one\"");
    let updated: Value = serde_json::from_slice(&requests[1].body).unwrap();
    assert_eq!(updated["id"], VIDEO);
    assert_eq!(updated["status"]["privacyStatus"], "public");
    for key in [
        "license",
        "embeddable",
        "publicStatsViewable",
        "selfDeclaredMadeForKids",
        "containsSyntheticMedia",
    ] {
        assert_eq!(
            updated["status"][key],
            video("private")["status"][key],
            "{key}"
        );
    }
    assert!(updated["status"].get("uploadStatus").is_none());
    assert!(updated.get("snippet").is_none());
}

#[tokio::test]
async fn already_public_is_idempotent_and_wrong_channel_never_gets_an_update() {
    for (privacy, expected_channel, succeeds) in [
        ("public", CHANNEL, true),
        ("private", "UC_someone_else", false),
    ] {
        let server =
            Server::start(vec![Reply::json(200, &json!({"items": [video(privacy)]}))]).await;
        assert_eq!(
            server.api().publish(VIDEO, expected_channel).await.is_ok(),
            succeeds
        );
        assert_eq!(server.finish().await.len(), 1);
    }
}

#[tokio::test]
async fn lost_publish_response_is_reconciled_without_a_second_update() {
    let server = Server::start(vec![
        Reply::json(200, &json!({"items": [video("private")]})),
        Reply::DropConnection,
        Reply::json(200, &json!({"items": [video("public")]})),
    ])
    .await;
    let api = server.api();
    assert!(api.publish(VIDEO, CHANNEL).await.is_err());
    let reconciled = api.publish(VIDEO, CHANNEL).await.unwrap();
    assert_eq!(reconciled.privacy, "public");
    let requests = server.finish().await;
    assert_eq!(
        requests
            .iter()
            .filter(|request| request.method == "PUT")
            .count(),
        1
    );
}

#[tokio::test]
async fn successful_publish_accepts_the_status_only_update_response() {
    let updated = json!({"id": VIDEO, "status": video("public")["status"]});
    let server = Server::start(vec![
        Reply::json(200, &json!({"items": [video("private")]})),
        Reply::json(200, &updated),
    ])
    .await;
    let result = server.api().publish(VIDEO, CHANNEL).await.unwrap();
    assert_eq!(result.id, VIDEO);
    assert_eq!(result.channel_id, CHANNEL);
    assert_eq!(result.privacy, "public");
    assert_eq!(result.status, "processed");
    let requests = server.finish().await;
    assert_eq!(requests[1].method, "PUT");
    assert!(requests[1].target.contains("part=status"));
}

#[tokio::test]
async fn unprocessed_or_refused_video_never_receives_a_publish_update() {
    for status in ["uploaded", "failed", "rejected", "deleted"] {
        let mut current = video("private");
        current["status"]["uploadStatus"] = json!(status);
        let server = Server::start(vec![Reply::json(200, &json!({"items": [current]}))]).await;
        assert!(
            server.api().publish(VIDEO, CHANNEL).await.is_err(),
            "{status}"
        );
        assert_eq!(server.finish().await.len(), 1, "{status}");
    }
}

#[tokio::test]
async fn oauth_ignores_wrong_state_but_denial_closes_the_callback_promptly() {
    let config = config();
    let (url, pending) = crate::authorize(&config).await.unwrap();
    let parameters: BTreeMap<_, _> = Url::parse(&url)
        .unwrap()
        .query_pairs()
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();
    assert_eq!(parameters["code_challenge_method"], "S256");
    assert_eq!(parameters["code_challenge"].len(), 43);
    assert_eq!(parameters["state"].len(), 43);
    let redirect = Url::parse(&parameters["redirect_uri"]).unwrap();
    assert_eq!(redirect.host_str(), Some("127.0.0.1"));
    let address = format!("127.0.0.1:{}", redirect.port().unwrap());
    let finishing =
        tokio::spawn(async move { crate::complete_authorization(&config, pending).await });
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    let wrong = client
        .get(format!("{redirect}?state=wrong&code=discard-me"))
        .send()
        .await
        .unwrap();
    assert!(!wrong.status().is_success());
    assert!(
        !finishing.is_finished(),
        "junk callback must not consume authorization"
    );
    let denied = client
        .get(format!(
            "{redirect}?state={}&error=access_denied",
            parameters["state"]
        ))
        .send()
        .await
        .unwrap();
    assert!(!denied.text().await.unwrap().contains(ACCESS));
    let result = timeout(Duration::from_secs(2), finishing)
        .await
        .expect("explicit denial must not wait five minutes")
        .unwrap();
    assert!(result.is_err());
    assert!(
        TcpStream::connect(address).await.is_err(),
        "callback listener is closed"
    );
}

#[tokio::test]
async fn dropping_pending_authorization_closes_its_bound_listener() {
    let (url, pending) = crate::authorize(&config()).await.unwrap();
    let redirect = Url::parse(&url)
        .unwrap()
        .query_pairs()
        .find(|(key, _)| key == "redirect_uri")
        .map(|(_, value)| Url::parse(&value).unwrap())
        .unwrap();
    let address = format!("127.0.0.1:{}", redirect.port().unwrap());
    assert!(TcpStream::connect(&address).await.is_ok());
    drop(pending);
    assert!(TcpStream::connect(&address).await.is_err());
}
