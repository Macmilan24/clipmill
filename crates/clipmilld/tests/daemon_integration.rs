#![cfg(unix)]
#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod support;

use std::{
    collections::BTreeMap,
    ffi::OsString,
    io::{Read, Write},
    path::PathBuf,
    sync::Arc,
};

use clipmill_artifacts::{
    ArtifactPath, ArtifactRecipe, NetworkPolicy, PrepareOutcome, Producer, RecipeSpec, Timebase,
};
use clipmill_contracts::proto::ipc::v1::{
    CreateProjectRequest, DeleteProjectRequest, ErrorCode, GetProjectRequest, HealthRequest,
    ListProjectsRequest, PingRequest, Request, request, response,
};
use clipmill_core::{ProjectId, Sha256Digest};
use clipmilld::{Config, Daemon, DaemonError};
use serde_json::Map;
use tempfile::TempDir;
use tokio::{sync::oneshot, task::JoinHandle};

use support::{
    create, send, send_without_reading_response, signal_terminate, spawn_daemon, wait_for_exit,
    wait_until_ready, workspace_tempdir,
};

fn artifact_recipe() -> ArtifactRecipe {
    ArtifactRecipe::try_from_spec(RecipeSpec {
        kind: "evidence.probe.v1".to_owned(),
        source_fingerprint: Sha256Digest::from_bytes([0x31; 32]),
        timebase: Timebase {
            num: 1,
            den: 90_000,
        },
        producer: Producer {
            stage: "probe".to_owned(),
            implementation: "probe-adapter@1.0.0".to_owned(),
            model_digest: None,
        },
        inputs: Vec::new(),
        policy: NetworkPolicy::LocalLock,
        config: Map::new(),
        semantic_version: "1.0.0".to_owned(),
    })
    .expect("artifact recipe")
}

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
    std::path::PathBuf,
    oneshot::Sender<()>,
    JoinHandle<Result<(), DaemonError>>,
) {
    let daemon = Daemon::start(config).await.expect("daemon starts");
    let socket = daemon.socket_path().to_path_buf();
    let (shutdown, stopped) = oneshot::channel();
    let task = tokio::spawn(daemon.serve_until(async {
        let _result = stopped.await;
    }));
    (socket, shutdown, task)
}

async fn stop(shutdown: oneshot::Sender<()>, task: JoinHandle<Result<(), DaemonError>>) {
    let _result = shutdown.send(());
    task.await
        .expect("daemon task joins")
        .expect("daemon shuts down");
}

#[tokio::test]
async fn ping_health_crud_idempotency_and_restart() {
    let temp = workspace_tempdir();
    let config = config(&temp);
    let (socket, shutdown, task) = running(config.clone()).await;

    let ping_reply = send(
        &socket,
        Request {
            request_id: "ping-1".to_owned(),
            body: Some(request::Body::Ping(PingRequest {
                echo: "hello".to_owned(),
            })),
        },
    )
    .await
    .expect("ping");
    assert!(
        matches!(ping_reply.body, Some(response::Body::Ping(payload)) if payload.echo == "hello")
    );

    let health = send(
        &socket,
        Request {
            request_id: "health-1".to_owned(),
            body: Some(request::Body::Health(HealthRequest {})),
        },
    )
    .await
    .expect("health");
    assert!(matches!(health.body, Some(response::Body::Health(value)) if value.local_lock));

    let create_request = Request {
        request_id: "create-1".to_owned(),
        body: Some(request::Body::CreateProject(CreateProjectRequest {
            name: "  Persistent project  ".to_owned(),
        })),
    };
    let created = send(&socket, create_request.clone()).await.expect("create");
    let replayed = send(&socket, create_request).await.expect("retry");
    assert_eq!(created, replayed);
    let project = match created.body {
        Some(response::Body::CreateProject(value)) => value.project.expect("project"),
        _ => panic!("unexpected create response"),
    };
    assert_eq!(project.name, "Persistent project");

    let conflict = send(
        &socket,
        Request {
            request_id: "create-1".to_owned(),
            body: Some(request::Body::CreateProject(CreateProjectRequest {
                name: "Different".to_owned(),
            })),
        },
    )
    .await
    .expect("conflict response");
    assert!(matches!(
        conflict.body,
        Some(response::Body::Error(error)) if error.code == ErrorCode::Conflict as i32
    ));

    stop(shutdown, task).await;
    assert!(!socket.exists());

    let (socket, shutdown, task) = running(config).await;
    let fetched = send(
        &socket,
        Request {
            request_id: "get-1".to_owned(),
            body: Some(request::Body::GetProject(GetProjectRequest {
                project_id: project.project_id.clone(),
            })),
        },
    )
    .await
    .expect("get after restart");
    assert!(matches!(
        fetched.body,
        Some(response::Body::GetProject(value)) if value.project == Some(project.clone())
    ));

    let deleted = send(
        &socket,
        Request {
            request_id: "delete-1".to_owned(),
            body: Some(request::Body::DeleteProject(DeleteProjectRequest {
                project_id: project.project_id.clone(),
            })),
        },
    )
    .await
    .expect("delete");
    let deleted_retry = send(
        &socket,
        Request {
            request_id: "delete-1".to_owned(),
            body: Some(request::Body::DeleteProject(DeleteProjectRequest {
                project_id: project.project_id,
            })),
        },
    )
    .await
    .expect("delete retry");
    assert_eq!(deleted, deleted_retry);
    stop(shutdown, task).await;
}

#[tokio::test]
async fn mutation_retry_after_response_loss_is_applied_once() {
    let temp = workspace_tempdir();
    let (socket, shutdown, task) = running(config(&temp)).await;
    let request = Request {
        request_id: "create-after-response-loss".to_owned(),
        body: Some(request::Body::CreateProject(CreateProjectRequest {
            name: "Lost response".to_owned(),
        })),
    };

    send_without_reading_response(&socket, request.clone())
        .await
        .expect("send mutation and drop response");
    let recovered = send(&socket, request)
        .await
        .expect("retry after response loss");
    assert!(matches!(
        recovered.body,
        Some(response::Body::CreateProject(_))
    ));

    let listed = send(
        &socket,
        Request {
            request_id: "list-after-response-loss".to_owned(),
            body: Some(request::Body::ListProjects(ListProjectsRequest {})),
        },
    )
    .await
    .expect("list after response loss");
    assert!(
        matches!(listed.body, Some(response::Body::ListProjects(value)) if value.projects.len() == 1)
    );
    stop(shutdown, task).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifact_publication_is_rooted_reused_and_persistent() {
    let temp = workspace_tempdir();
    let config = config(&temp);
    let daemon = Daemon::start(config.clone()).await.expect("daemon starts");
    let coordinator = daemon.artifact_coordinator();
    let socket = daemon.socket_path().to_path_buf();
    let (shutdown, stopped) = oneshot::channel();
    let task = tokio::spawn(daemon.serve_until(async {
        let _stopped = stopped.await;
    }));

    let first = create(&socket, "artifact-project-1", "Artifact one")
        .await
        .expect("first project");
    let first_id = first.project_id.parse::<ProjectId>().expect("first id");
    let recipe = artifact_recipe();
    let staging = match coordinator.prepare(recipe.clone()).await.expect("prepare") {
        PrepareOutcome::Miss(staging) => staging,
        other => panic!("expected cache miss, got {other:?}"),
    };
    let path = "probe.json".parse::<ArtifactPath>().expect("artifact path");
    {
        let mut file = staging.create_file(&path).expect("staging file");
        file.write_all(b"{\"probe\":true}\n")
            .expect("write artifact");
        file.sync_all().expect("sync artifact");
    }
    let lease = coordinator
        .publish_project(
            first_id,
            staging.id().clone(),
            vec![path.clone()],
            BTreeMap::new(),
        )
        .await
        .expect("publish first root");
    let artifact_id = lease.artifact_id();
    drop(lease);

    let second = create(&socket, "artifact-project-2", "Artifact two")
        .await
        .expect("second project");
    let second_id = second.project_id.parse::<ProjectId>().expect("second id");
    match coordinator.prepare(recipe).await.expect("cache lookup") {
        PrepareOutcome::Hit(hit) => assert_eq!(hit.artifact_id(), artifact_id),
        other => panic!("expected cache hit, got {other:?}"),
    }
    drop(
        coordinator
            .attach_existing_project(second_id, artifact_id)
            .await
            .expect("attach shared artifact"),
    );
    stop(shutdown, task).await;

    let daemon = Daemon::start(config).await.expect("restart daemon");
    let coordinator = daemon.artifact_coordinator();
    let socket = daemon.socket_path().to_path_buf();
    let (shutdown, stopped) = oneshot::channel();
    let task = tokio::spawn(daemon.serve_until(async {
        let _stopped = stopped.await;
    }));
    let reopened = coordinator
        .open(artifact_id)
        .await
        .expect("open after restart");
    let mut file = reopened.open_verified(&path).expect("verified artifact");
    let mut payload = String::new();
    file.read_to_string(&mut payload).expect("read artifact");
    assert_eq!(payload, "{\"probe\":true}\n");
    drop(file);
    drop(reopened);
    stop(shutdown, task).await;
    assert!(!socket.exists());
}

#[tokio::test]
async fn executable_rejects_a_second_process_and_cleans_up_on_sigterm() {
    let temp = workspace_tempdir();
    let data_dir = temp.path().join("data");
    let socket = temp.path().join("clipmilld.sock");
    let mut daemon = spawn_daemon(&data_dir, &socket);
    wait_until_ready(&socket).await.expect("daemon ready");

    let mut second = spawn_daemon(&data_dir, &socket);
    let second_status = wait_for_exit(&mut second)
        .await
        .expect("second daemon exits");
    assert!(!second_status.success());

    signal_terminate(&daemon).expect("send SIGTERM");
    let status = wait_for_exit(&mut daemon).await.expect("daemon exits");
    assert!(status.success());
    assert!(!socket.exists());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_clients_are_serialized_and_second_daemon_is_rejected() {
    let temp = workspace_tempdir();
    let config = config(&temp);
    let paths = config.paths.clone();
    let daemon = Daemon::start(config.clone()).await.expect("daemon starts");
    let socket = daemon.socket_path().to_path_buf();

    let second = Daemon::start(config).await;
    assert!(matches!(second, Err(DaemonError::AlreadyRunning(_))));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&socket)
            .expect("socket metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
        for path in [&paths.database, &paths.lock] {
            let mode = std::fs::metadata(path)
                .expect("private file metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600, "private mode for {}", path.display());
        }
        for path in [&paths.data_dir, &paths.state_dir, &paths.run_dir] {
            let mode = std::fs::metadata(path)
                .expect("private directory metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o700, "private mode for {}", path.display());
        }
    }

    let (shutdown, stopped) = oneshot::channel();
    let daemon_task = tokio::spawn(daemon.serve_until(async {
        let _result = stopped.await;
    }));
    let socket = Arc::new(socket);
    let mut clients = Vec::new();
    for index in 0..32 {
        let socket = Arc::clone(&socket);
        clients.push(tokio::spawn(async move {
            send(
                &socket,
                Request {
                    request_id: format!("concurrent-{index}"),
                    body: Some(request::Body::CreateProject(CreateProjectRequest {
                        name: "Same name".to_owned(),
                    })),
                },
            )
            .await
        }));
    }
    for client in clients {
        let response = client
            .await
            .expect("client joins")
            .expect("client response");
        assert!(matches!(
            response.body,
            Some(response::Body::CreateProject(_))
        ));
    }

    let listed = send(
        &socket,
        Request {
            request_id: "list-all".to_owned(),
            body: Some(request::Body::ListProjects(ListProjectsRequest {})),
        },
    )
    .await
    .expect("list");
    assert!(
        matches!(listed.body, Some(response::Body::ListProjects(value)) if value.projects.len() == 32)
    );

    let _result = shutdown.send(());
    daemon_task
        .await
        .expect("daemon joins")
        .expect("daemon stops");
}

#[tokio::test]
async fn invalid_and_unavailable_requests_return_stable_errors() {
    let temp = workspace_tempdir();
    let (socket, shutdown, task) = running(config(&temp)).await;

    let invalid = send(
        &socket,
        Request {
            request_id: "invalid".to_owned(),
            body: Some(request::Body::GetProject(GetProjectRequest {
                project_id: "not-an-id".to_owned(),
            })),
        },
    )
    .await
    .expect("invalid response");
    assert!(
        matches!(invalid.body, Some(response::Body::Error(error)) if error.code == ErrorCode::InvalidArgument as i32)
    );

    let missing = send(
        &socket,
        Request {
            request_id: "missing".to_owned(),
            body: Some(request::Body::DeleteProject(DeleteProjectRequest {
                project_id: "prj_01ARZ3NDEKTSV4RRFFQ69G5FAV".to_owned(),
            })),
        },
    )
    .await
    .expect("missing response");
    assert!(
        matches!(missing.body, Some(response::Body::Error(error)) if error.code == ErrorCode::NotFound as i32)
    );

    let unavailable = send(
        &socket,
        Request {
            request_id: "unavailable".to_owned(),
            body: Some(request::Body::GetDeviceProfile(
                clipmill_contracts::proto::ipc::v1::GetDeviceProfileRequest { remeasure: false },
            )),
        },
    )
    .await
    .expect("unavailable response");
    assert!(
        matches!(unavailable.body, Some(response::Body::Error(error)) if error.code == ErrorCode::Unavailable as i32)
    );

    stop(shutdown, task).await;
}
