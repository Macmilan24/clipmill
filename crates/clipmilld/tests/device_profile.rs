#![cfg(unix)]
#![allow(clippy::expect_used, clippy::panic)]

mod support;

use std::{ffi::OsString, io::Read, os::unix::fs::PermissionsExt, path::PathBuf};

use clipmill_artifacts::ArtifactPath;
use clipmill_contracts::proto::ipc::v1::{
    GetDeviceProfileRequest, GetDeviceProfileResponse, Request, Response, request, response,
};
use clipmill_core::ArtifactId;
use clipmilld::{ArtifactCoordinator, Config, Daemon, DaemonError, verify_device_profile};
use prost::Message;
use tempfile::TempDir;
use tokio::{
    sync::oneshot,
    task::JoinHandle,
    time::{Duration, sleep},
};

use support::{send, send_without_reading_response, workspace_tempdir};

fn config(temp: &TempDir) -> Config {
    Config::from_sources(
        Some(temp.path().to_path_buf()),
        None,
        Some(OsString::from("/ignored/env")),
        None,
        PathBuf::from("/ignored/default"),
    )
    .expect("config")
}

async fn running(
    config: Config,
) -> (
    PathBuf,
    ArtifactCoordinator,
    oneshot::Sender<()>,
    JoinHandle<Result<(), DaemonError>>,
) {
    let daemon = Daemon::start(config).await.expect("daemon starts");
    let socket = daemon.socket_path().to_path_buf();
    let artifacts = daemon.artifact_coordinator();
    let (shutdown, stopped) = oneshot::channel();
    let task = tokio::spawn(daemon.serve_until(async {
        let _result = stopped.await;
    }));
    (socket, artifacts, shutdown, task)
}

async fn stop(shutdown: oneshot::Sender<()>, task: JoinHandle<Result<(), DaemonError>>) {
    let _sent = shutdown.send(());
    task.await
        .expect("daemon task joins")
        .expect("daemon stops");
}

fn request(request_id: &str, remeasure: bool) -> Request {
    Request {
        request_id: request_id.to_owned(),
        body: Some(request::Body::GetDeviceProfile(GetDeviceProfileRequest {
            remeasure,
        })),
    }
}

fn profile(response: Response) -> GetDeviceProfileResponse {
    match response.body {
        Some(response::Body::GetDeviceProfile(profile)) => profile,
        Some(response::Body::Error(error)) => panic!("device profile failed: {}", error.message),
        _ => panic!("unexpected device profile response"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn profile_requests_join_publish_verify_remeasure_and_survive_restart() {
    let temp = workspace_tempdir();
    let config = config(&temp);
    let (socket, artifacts, shutdown, task) = running(config.clone()).await;

    let (first, second) = tokio::join!(
        send(&socket, request("profile-concurrent-a", false)),
        send(&socket, request("profile-concurrent-b", false)),
    );
    let first = profile(first.expect("first profile"));
    let second = profile(second.expect("second profile"));
    assert_eq!(first.artifact_id, second.artifact_id);
    assert_eq!(first.profile_json, second.profile_json);
    let verified = verify_device_profile(&first.profile_json, None).expect("signature");
    assert_eq!(verified.measurement_generation, 1);

    let artifact_id = first
        .artifact_id
        .parse::<ArtifactId>()
        .expect("artifact id");
    let lease = artifacts
        .open(artifact_id)
        .await
        .expect("open profile artifact");
    let path = "profile.json"
        .parse::<ArtifactPath>()
        .expect("profile path");
    let mut stored = String::new();
    lease
        .open_verified(&path)
        .expect("verified payload")
        .read_to_string(&mut stored)
        .expect("read profile");
    assert_eq!(stored, first.profile_json);

    let loss_request = request("profile-response-loss", false);
    send_without_reading_response(&socket, loss_request.clone())
        .await
        .expect("send without response");
    sleep(Duration::from_millis(100)).await;
    let loss_retry = send(&socket, loss_request.clone())
        .await
        .expect("retry response");
    let replay = send(&socket, loss_request)
        .await
        .expect("second retry response");
    assert_eq!(loss_retry.encode_to_vec(), replay.encode_to_vec());
    assert_eq!(profile(loss_retry).artifact_id, first.artifact_id);

    let remeasured = profile(
        send(&socket, request("profile-remeasure", true))
            .await
            .expect("remeasure"),
    );
    let remeasured_verified =
        verify_device_profile(&remeasured.profile_json, None).expect("remeasured signature");
    assert_eq!(remeasured_verified.measurement_generation, 2);
    assert_eq!(
        remeasured_verified.hardware_fingerprint,
        verified.hardware_fingerprint
    );
    assert_ne!(remeasured.artifact_id, first.artifact_id);

    let key = temp.path().join("state/device-attestation.key");
    let key_mode = std::fs::metadata(&key)
        .expect("key metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(key_mode, 0o600);
    stop(shutdown, task).await;

    let (socket, _artifacts, shutdown, task) = running(config).await;
    let cached = profile(
        send(&socket, request("profile-after-restart", false))
            .await
            .expect("cached after restart"),
    );
    assert_eq!(cached.artifact_id, remeasured.artifact_id);
    assert_eq!(cached.profile_json, remeasured.profile_json);
    assert_eq!(
        verify_device_profile(&cached.profile_json, None)
            .expect("restart signature")
            .measurement_generation,
        2
    );
    stop(shutdown, task).await;
}
