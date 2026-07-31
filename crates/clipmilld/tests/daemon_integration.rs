#![cfg(unix)]
#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod support;

use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    io::{Read, Write},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use clipmill_artifacts::{
    ArtifactPath, ArtifactRecipe, NetworkPolicy, PrepareOutcome, Producer, RecipeSpec, Timebase,
};
use clipmill_contracts::proto::ipc::v1::{
    CancelJobRequest, CreateProjectRequest, DeleteProjectRequest, ErrorCode, GetProjectRequest,
    HealthRequest, JobState, ListProjectsRequest, PingRequest, Request, SubscribeTaskEventsRequest,
    TaskState, request, response,
};
use clipmill_core::{ProjectId, Sha256Digest};
use clipmilld::{Config, Daemon, DaemonError, verify_device_profile};
use serde_json::Map;
use tempfile::TempDir;
use tokio::{
    net::UnixStream,
    sync::oneshot,
    task::JoinHandle,
    time::{sleep, timeout},
};

use support::{
    create, get_job, list_jobs, read_response, send, send_on_stream, send_without_reading_response,
    signal_terminate, spawn_daemon, spawn_daemon_with_step_delay, submit_demo, wait_for_exit,
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
    .with_builtin_fixture_executor_for_tests()
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

    let profile = send(
        &socket,
        Request {
            request_id: "unavailable".to_owned(),
            body: Some(request::Body::GetDeviceProfile(
                clipmill_contracts::proto::ipc::v1::GetDeviceProfileRequest { remeasure: false },
            )),
        },
    )
    .await
    .expect("device profile response");
    let Some(response::Body::GetDeviceProfile(profile)) = profile.body else {
        panic!("expected a measured device profile");
    };
    let verified = verify_device_profile(&profile.profile_json, None).expect("verified profile");
    assert_eq!(verified.measurement_generation, 1);
    assert!(profile.artifact_id.starts_with("sha256:"));

    stop(shutdown, task).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn durable_demo_job_completes_replays_events_and_survives_restart() {
    let temp = workspace_tempdir();
    let config = config(&temp);
    let (socket, shutdown, task) = running(config.clone()).await;
    let project = create(&socket, "job-project", "Durable job")
        .await
        .expect("project");
    let submitted = submit_demo(
        &socket,
        "submit-demo",
        &project.project_id,
        b"deterministic input",
    )
    .await
    .expect("submit demo");
    let replayed = submit_demo(
        &socket,
        "submit-demo",
        &project.project_id,
        b"deterministic input",
    )
    .await
    .expect("replay submit");
    assert_eq!(submitted, replayed);

    let mut completed = None;
    for attempt in 0..250 {
        let job = get_job(&socket, &format!("poll-{attempt}"), &submitted.job_id)
            .await
            .expect("poll job");
        if job.state == JobState::Succeeded as i32 {
            completed = Some(job);
            break;
        }
        sleep(Duration::from_millis(20)).await;
    }
    let completed = completed.expect("job completes");
    assert_eq!(completed.tasks.len(), 4);
    assert!(
        completed
            .tasks
            .iter()
            .all(|task| task.state == TaskState::Succeeded as i32)
    );
    // Every task says what it publishes, and says it as a contract kind. A shell
    // looking for one particular observation finds it here rather than by
    // knowing what the daemon calls the work that produces it.
    assert!(
        completed
            .tasks
            .iter()
            .all(|task| task.output_kind.contains(".v1")),
        "a task reported no output kind: {:?}",
        completed
            .tasks
            .iter()
            .map(|task| (&task.kind, &task.output_kind))
            .collect::<Vec<_>>()
    );
    assert_eq!(completed.output_artifact_ids.len(), 1);
    assert_eq!(
        list_jobs(&socket, "list-jobs", &project.project_id)
            .await
            .expect("list jobs"),
        vec![completed.clone()]
    );

    let mut stream = UnixStream::connect(&socket)
        .await
        .expect("event connection");
    send_on_stream(
        &mut stream,
        Request {
            request_id: "events".to_owned(),
            body: Some(request::Body::SubscribeTaskEvents(
                SubscribeTaskEventsRequest {
                    project_id: project.project_id,
                    job_id: completed.job_id.clone(),
                    after_event_id: 0,
                },
            )),
        },
    )
    .await
    .expect("subscribe");
    let ready = read_response(&mut stream)
        .await
        .expect("subscription ready");
    assert!(matches!(
        ready.body,
        Some(response::Body::SubscribeTaskEvents(value)) if value.current_event_id > 0
    ));
    let mut cursor = 0;
    let mut saw_final = false;
    for _event in 0..32 {
        let response = read_response(&mut stream).await.expect("event replay");
        let Some(response::Body::TaskEvent(event)) = response.body else {
            panic!("unexpected subscription frame");
        };
        assert!(event.event_id > cursor);
        cursor = event.event_id;
        if event.task_id == completed.tasks[3].task_id && event.state == TaskState::Succeeded as i32
        {
            saw_final = true;
            break;
        }
    }
    assert!(saw_final);
    drop(stream);
    stop(shutdown, task).await;

    let (socket, shutdown, task) = running(config).await;
    let persisted = get_job(&socket, "job-after-restart", &completed.job_id)
        .await
        .expect("job after restart");
    assert_eq!(persisted, completed);
    stop(shutdown, task).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn live_event_subscription_reconnects_without_gaps_or_duplicates() {
    let temp = workspace_tempdir();
    let (socket, shutdown, task) = running(config(&temp)).await;
    let project = create(&socket, "event-project", "Event replay")
        .await
        .expect("project");
    let mut stream = UnixStream::connect(&socket)
        .await
        .expect("event connection");
    send_on_stream(
        &mut stream,
        Request {
            request_id: "live-events".to_owned(),
            body: Some(request::Body::SubscribeTaskEvents(
                SubscribeTaskEventsRequest {
                    project_id: project.project_id.clone(),
                    job_id: String::new(),
                    after_event_id: 0,
                },
            )),
        },
    )
    .await
    .expect("subscribe");
    let ready = read_response(&mut stream)
        .await
        .expect("subscription ready");
    assert!(matches!(
        ready.body,
        Some(response::Body::SubscribeTaskEvents(_))
    ));

    let first = submit_demo(&socket, "live-job-1", &project.project_id, b"first")
        .await
        .expect("submit first job");
    let final_task_id = first.tasks.last().expect("final task").task_id.clone();
    let mut cursor = 0;
    let mut seen = BTreeSet::new();
    loop {
        let response = timeout(Duration::from_secs(5), read_response(&mut stream))
            .await
            .expect("event timeout")
            .expect("live event");
        let Some(response::Body::TaskEvent(event)) = response.body else {
            panic!("unexpected subscription frame");
        };
        assert!(event.event_id > cursor);
        assert!(seen.insert(event.event_id), "duplicate event cursor");
        cursor = event.event_id;
        if event.job_id == first.job_id
            && event.task_id == final_task_id
            && event.state == TaskState::Succeeded as i32
        {
            break;
        }
    }
    drop(stream);

    let mut resumed = UnixStream::connect(&socket)
        .await
        .expect("resume connection");
    send_on_stream(
        &mut resumed,
        Request {
            request_id: "resumed-events".to_owned(),
            body: Some(request::Body::SubscribeTaskEvents(
                SubscribeTaskEventsRequest {
                    project_id: project.project_id.clone(),
                    job_id: String::new(),
                    after_event_id: cursor,
                },
            )),
        },
    )
    .await
    .expect("resume subscription");
    let ready = read_response(&mut resumed).await.expect("resume ready");
    assert!(matches!(
        ready.body,
        Some(response::Body::SubscribeTaskEvents(_))
    ));
    let second = submit_demo(&socket, "live-job-2", &project.project_id, b"second")
        .await
        .expect("submit second job");
    let response = timeout(Duration::from_secs(5), read_response(&mut resumed))
        .await
        .expect("resumed event timeout")
        .expect("resumed event");
    let Some(response::Body::TaskEvent(event)) = response.body else {
        panic!("unexpected resumed subscription frame");
    };
    assert!(event.event_id > cursor);
    assert_eq!(event.job_id, second.job_id);
    drop(resumed);
    stop(shutdown, task).await;
}

#[tokio::test]
async fn cancelling_a_running_lease_is_durable_and_rejects_late_success() {
    let temp = workspace_tempdir();
    let data_dir = temp.path().join("cancel-data");
    let socket = temp.path().join("cancel.sock");
    let mut daemon = spawn_daemon_with_step_delay(&data_dir, &socket, Some(2_000));
    wait_until_ready(&socket).await.expect("daemon ready");
    let project = create(&socket, "cancel-project", "Cancellation")
        .await
        .expect("project");
    let submitted = submit_demo(&socket, "cancel-submit", &project.project_id, b"cancel")
        .await
        .expect("submit");
    let mut observed_running = false;
    for attempt in 0..100 {
        let current = get_job(
            &socket,
            &format!("cancel-poll-{attempt}"),
            &submitted.job_id,
        )
        .await
        .expect("poll");
        if current
            .tasks
            .iter()
            .any(|task| task.state == TaskState::Running as i32)
        {
            observed_running = true;
            break;
        }
        sleep(Duration::from_millis(10)).await;
    }
    assert!(observed_running, "fixture task never entered running state");
    let cancel = Request {
        request_id: "cancel-running".to_owned(),
        body: Some(request::Body::CancelJob(CancelJobRequest {
            job_id: submitted.job_id.clone(),
        })),
    };
    let first = send(&socket, cancel.clone()).await.expect("cancel");
    let replay = send(&socket, cancel).await.expect("cancel replay");
    assert_eq!(first, replay);
    assert!(matches!(
        first.body,
        Some(response::Body::CancelJob(value))
            if value.job.as_ref().is_some_and(|job| job.state == JobState::Cancelled as i32)
    ));
    sleep(Duration::from_millis(2_200)).await;
    let persisted = get_job(&socket, "cancel-persisted", &submitted.job_id)
        .await
        .expect("cancelled job");
    assert_eq!(persisted.state, JobState::Cancelled as i32);
    assert!(persisted.output_artifact_ids.is_empty());
    assert!(
        persisted
            .tasks
            .iter()
            .all(|task| task.state == TaskState::Cancelled as i32)
    );
    signal_terminate(&daemon).expect("terminate daemon");
    assert!(
        wait_for_exit(&mut daemon)
            .await
            .expect("daemon exits")
            .success()
    );
}
