#![cfg(unix)]
#![allow(clippy::expect_used, clippy::panic)]

mod support;

use std::{
    ffi::OsString,
    io::Read,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

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
    config_with_ffprobe(temp, None)
}

fn config_with_ffprobe(temp: &TempDir, ffprobe: Option<PathBuf>) -> Config {
    Config::from_sources_with_gc(
        Some(temp.path().to_path_buf()),
        None,
        None,
        Some(OsString::from("/ignored/env")),
        None,
        None,
        ffprobe,
        None,
        PathBuf::from("/ignored/default"),
    )
    .expect("config")
}

fn workspace_tool(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(".cache/bin")
        .join(name)
        .canonicalize()
        .unwrap_or_else(|_| panic!("{name} is missing; run ./tools/fetch-ffmpeg.sh"))
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

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "run through tools/drills/device-drill.sh with pinned FFmpeg"]
async fn pinned_ffmpeg_profile_executes_bounded_measurements() {
    let temp = workspace_tempdir();
    let config = config_with_ffprobe(&temp, Some(workspace_tool("ffprobe")));
    let (socket, _artifacts, shutdown, task) = running(config).await;
    let measured = profile(
        send(&socket, request("profile-pinned-ffmpeg", false))
            .await
            .expect("measured profile"),
    );
    verify_device_profile(&measured.profile_json, None).expect("signed profile");
    let value: serde_json::Value =
        serde_json::from_str(&measured.profile_json).expect("profile JSON");
    let phase0 = value.get("phase0").expect("Phase 0 extension");
    assert_eq!(
        phase0
            .get("runtime_identities")
            .and_then(serde_json::Value::as_array)
            .and_then(|identities| identities.first())
            .and_then(|identity| identity.get("available"))
            .and_then(serde_json::Value::as_bool),
        Some(true),
        "the pinned FFmpeg runtime must be measured"
    );
    assert_eq!(
        phase0
            .get("capability_results")
            .and_then(serde_json::Value::as_array)
            .and_then(|capabilities| capabilities.first())
            .and_then(|capability| capability.get("available"))
            .and_then(serde_json::Value::as_bool),
        Some(true),
        "the bounded CPU video round trip must execute"
    );
    assert!(
        phase0
            .get("shared_memory")
            .and_then(|shared| shared.get("bytes_per_second"))
            .and_then(serde_json::Value::as_f64)
            .is_some_and(|throughput| throughput > 0.0),
        "the bounded shared-memory measurement must execute"
    );
    stop(shutdown, task).await;
}

/// Where the accelerated drill starts: a daemon fingerprints the machine, and
/// the benchmark that runs next is bound to that fingerprint and no other.
///
/// Split into two ignored tests rather than one because the measurement in
/// between happens in Python, inside the environment that loads the models.
/// Nothing else can take it: a real-time factor measured by a process that did
/// not load the weights is a guess.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "run through tools/drills/asr-mlx-drill.sh on a machine with the accelerator"]
async fn mlx_drill_reports_the_hardware_fingerprint() {
    let data_dir = drill_directory();
    let config = drill_config(&data_dir);
    let (socket, _artifacts, shutdown, task) = running(config).await;
    let measured = profile(
        send(&socket, request("mlx-drill-fingerprint", false))
            .await
            .expect("measured profile"),
    );
    let verified = verify_device_profile(&measured.profile_json, None).expect("signed profile");
    std::fs::write(
        data_dir.join("fingerprint.txt"),
        format!("{}\n", verified.hardware_fingerprint),
    )
    .expect("fingerprint is written for the benchmark");
    stop(shutdown, task).await;
}

/// The assertion the drill exists for: with a benchmark in place, this device
/// runs the accelerated implementations, admits the accelerator they ran on,
/// and binds every contested capability by measurement rather than by falling
/// back.
///
/// Note what is *not* asserted. Requiring MLX to win would reinstate the
/// static per-platform default D19 removes, and would be wrong here: the first
/// real run of this gate measured whisper.cpp-base recognizing faster than a
/// 1.7B Qwen3 on the same machine, while the Qwen3 aligner beat the CTC one
/// five times over. Both are correct answers, and both are answers only a
/// measurement could give.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "run through tools/drills/asr-mlx-drill.sh on a machine with the accelerator"]
async fn mlx_drill_asserts_the_measured_binding() {
    let data_dir = drill_directory();
    let config = drill_config(&data_dir);
    let (socket, _artifacts, shutdown, task) = running(config).await;
    // Re-measured on purpose. The first profile was taken before the benchmark
    // existed, and serving it from cache would be exactly the mistake this
    // gate is here to catch.
    let measured = profile(
        send(&socket, request("mlx-drill-binding", true))
            .await
            .expect("re-measured profile"),
    );
    verify_device_profile(&measured.profile_json, None).expect("signed profile");
    let value: serde_json::Value =
        serde_json::from_str(&measured.profile_json).expect("profile JSON");
    let selection = value.get("selection").expect("a selection block");
    let bindings = selection
        .get("bindings")
        .and_then(serde_json::Value::as_array)
        .expect("bindings");
    let candidates = selection
        .get("candidates")
        .and_then(serde_json::Value::as_array)
        .expect("candidates");

    for implementation in [
        "clipmill-worker-speech-mlx@0.1.0/asr",
        "clipmill-worker-speech-mlx@0.1.0/align",
    ] {
        let candidate = candidates
            .iter()
            .find(|candidate| {
                candidate
                    .get("implementation")
                    .and_then(serde_json::Value::as_str)
                    == Some(implementation)
            })
            .unwrap_or_else(|| panic!("no candidate for {implementation}"));
        assert_eq!(
            candidate
                .get("runnable")
                .and_then(serde_json::Value::as_bool),
            Some(true),
            "{implementation} did not run here: {:?}",
            candidate.get("unavailable_reason")
        );
    }
    for capability in ["asr", "forced-align"] {
        let binding = bindings
            .iter()
            .find(|binding| {
                binding
                    .get("capability")
                    .and_then(serde_json::Value::as_str)
                    == Some(capability)
            })
            .unwrap_or_else(|| panic!("no binding for {capability}"));
        assert_eq!(
            binding
                .get("selected_by")
                .and_then(serde_json::Value::as_str),
            Some("measured"),
            "{capability} was bound without a measurement behind it"
        );
    }
    // Having run a model on the accelerator is what makes it admissible. A
    // profile that did not record it would leave the scheduler declining the
    // very worker that just produced these numbers.
    let verified = verify_device_profile(&measured.profile_json, None).expect("signed profile");
    assert!(
        verified.available_backends.contains("metal"),
        "a measured MLX run must make Metal admissible: {:?}",
        verified.available_backends
    );
    std::fs::write(data_dir.join("profile.json"), &measured.profile_json)
        .expect("profile is written for the attestation");
    stop(shutdown, task).await;
}

/// The drill's working directory, handed in rather than temporary: the
/// fingerprint, the benchmark, and the attestation all have to be the same
/// machine's, and a fresh temp directory per test would break that chain.
fn drill_directory() -> PathBuf {
    let raw = std::env::var("CLIPMILL_MLX_DRILL_DIR")
        .expect("CLIPMILL_MLX_DRILL_DIR; run tools/drills/asr-mlx-drill.sh");
    let path = PathBuf::from(raw);
    assert!(path.is_absolute(), "the drill directory must be absolute");
    std::fs::create_dir_all(&path).expect("drill directory");
    path
}

fn drill_config(data_dir: &Path) -> Config {
    let mut config = Config::from_sources_with_gc(
        Some(data_dir.to_path_buf()),
        None,
        None,
        None,
        None,
        None,
        Some(workspace_tool("ffprobe")),
        None,
        PathBuf::from("/ignored/default"),
    )
    .expect("config");
    config.models_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../models/registry");
    config.weights_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.cache/models")
        .canonicalize()
        .expect("pinned weights; run ./tools/fetch-models.sh");
    config
}
