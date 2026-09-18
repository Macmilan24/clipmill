//! W12 edit-document gate, driven over the real control socket.
//!
//! An acknowledged command is a durable promise: the drill kills the daemon
//! without warning immediately after the acknowledgement and requires the
//! restarted daemon to serve the same revision, the same document bytes, and
//! a log that still replays onto them.
#![cfg(unix)]
#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod support;

use std::{
    ffi::OsString,
    io::Read,
    path::{Path, PathBuf},
};

use clipmill_artifacts::ArtifactPath;
use clipmill_contracts::proto::ipc::v1::{
    ApplyEditCommandRequest, CreateEditDocRequest, ErrorCode, ExportClipRequest, ExportRequestV1,
    GetEditDocRequest, PlanExportRequest, Request, SnapshotEditDocRequest, request, response,
};
use clipmill_core::ArtifactId;
use clipmill_edit_ir::{EditCommand, EditDocument};
use clipmilld::{Config, Daemon, DaemonError};
use tempfile::TempDir;
use tokio::{sync::oneshot, task::JoinHandle};

use support::{
    create, get_job, list_jobs, send, wait_for_exit, wait_until_ready, workspace_tempdir,
};

fn config(temp: &TempDir) -> Config {
    Config::from_sources_with_gc(
        Some(temp.path().to_path_buf()),
        None,
        None,
        Some(OsString::from("/ignored/env")),
        None,
        None,
        Some(PathBuf::from("/ignored/ffprobe")),
        None,
        PathBuf::from("/ignored/default"),
    )
    .expect("edit document test config")
}

fn sample_document() -> String {
    let fingerprint = format!("sha256:{}", "cd".repeat(32));
    serde_json::json!({
        "version": "ir/1",
        "timebase": {"num": 1, "den": 90000},
        "video": {"segments": [{
            "segment_id": "seg_a",
            "source_fingerprint": fingerprint,
            "in_ticks": 0,
            "out_ticks": 900_000,
            "layout": {"state": "fit"},
        }]},
        "captions": {"style_ref": "clean", "cues": [{
            "cue_id": "cue_a",
            "start_ticks": 0,
            "end_ticks": 90_000,
            "region": "lower_safe",
            "anim": "karaoke",
            "lines": [{"words": [
                {"text": "one", "start_ticks": 0, "end_ticks": 45_000},
                {"text": "two", "start_ticks": 45_000, "end_ticks": 90_000},
            ]}],
        }]},
        "audio": {"target_lufs": -14.0, "true_peak_dbtp": -1.0},
        "rationale": {"candidate_id": "cand_1", "decisions": ["hook at zero"]},
    })
    .to_string()
}

async fn create_doc(socket: &Path, request_id: &str, project_id: &str, document: &str) -> String {
    let response = send(
        socket,
        Request {
            request_id: request_id.to_owned(),
            body: Some(request::Body::CreateEditDoc(CreateEditDocRequest {
                project_id: project_id.to_owned(),
                document_json: document.to_owned(),
                ..CreateEditDocRequest::default()
            })),
        },
    )
    .await
    .expect("create edit doc");
    match response.body {
        Some(response::Body::CreateEditDoc(created)) => created.doc.expect("doc").doc_id,
        Some(response::Body::Error(error)) => panic!("create rejected: {}", error.message),
        _ => panic!("unexpected create response"),
    }
}

struct Applied {
    revision: u64,
    document_json: String,
    inverse_command_json: String,
}

async fn apply(
    socket: &Path,
    request_id: &str,
    doc_id: &str,
    expected_revision: u64,
    command: &EditCommand,
) -> Result<Applied, String> {
    let command_json =
        String::from_utf8(command.to_canonical_json().expect("serialize")).expect("utf-8");
    let response = send(
        socket,
        Request {
            request_id: request_id.to_owned(),
            body: Some(request::Body::ApplyEditCommand(ApplyEditCommandRequest {
                doc_id: doc_id.to_owned(),
                expected_revision,
                command_json,
            })),
        },
    )
    .await?;
    match response.body {
        Some(response::Body::ApplyEditCommand(applied)) => {
            let doc = applied
                .doc
                .ok_or_else(|| "response omitted doc".to_owned())?;
            Ok(Applied {
                revision: doc.revision,
                document_json: doc.document_json,
                inverse_command_json: applied.inverse_command_json,
            })
        }
        Some(response::Body::Error(error)) => Err(error.message),
        _ => Err("unexpected apply response".to_owned()),
    }
}

async fn get_doc(socket: &Path, request_id: &str, doc_id: &str) -> (u64, String) {
    let response = send(
        socket,
        Request {
            request_id: request_id.to_owned(),
            body: Some(request::Body::GetEditDoc(GetEditDocRequest {
                doc_id: doc_id.to_owned(),
            })),
        },
    )
    .await
    .expect("get edit doc");
    match response.body {
        Some(response::Body::GetEditDoc(got)) => {
            let doc = got.doc.expect("doc");
            (doc.revision, doc.document_json)
        }
        Some(response::Body::Error(error)) => panic!("get rejected: {}", error.message),
        _ => panic!("unexpected get response"),
    }
}

async fn snapshot(socket: &Path, request_id: &str, doc_id: &str) -> (String, u64) {
    let response = send(
        socket,
        Request {
            request_id: request_id.to_owned(),
            body: Some(request::Body::SnapshotEditDoc(SnapshotEditDocRequest {
                doc_id: doc_id.to_owned(),
            })),
        },
    )
    .await
    .expect("snapshot edit doc");
    match response.body {
        Some(response::Body::SnapshotEditDoc(taken)) => (taken.artifact_id, taken.revision),
        Some(response::Body::Error(error)) => panic!("snapshot rejected: {}", error.message),
        _ => panic!("unexpected snapshot response"),
    }
}

async fn running(
    config: Config,
) -> (
    PathBuf,
    clipmilld::ArtifactCoordinator,
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

#[tokio::test]
#[allow(
    clippy::too_many_lines,
    reason = "one document's life over the socket, in order"
)]
async fn commands_apply_invert_and_snapshot_over_the_control_socket() {
    let temp = workspace_tempdir();
    let (socket, artifacts, shutdown, task) = running(config(&temp)).await;
    wait_until_ready(&socket).await.expect("daemon ready");
    let project = create(&socket, "edit-project", "Edits")
        .await
        .expect("project");
    let doc_id = create_doc(
        &socket,
        "edit-create",
        &project.project_id,
        &sample_document(),
    )
    .await;

    let before = get_doc(&socket, "edit-get-0", &doc_id).await;
    assert_eq!(before.0, 0);

    let applied = apply(
        &socket,
        "edit-apply-1",
        &doc_id,
        0,
        &EditCommand::Trim {
            segment_id: "seg_a".to_owned(),
            in_ticks: 90_000,
            out_ticks: 900_000,
        },
    )
    .await
    .expect("trim applies");
    assert_eq!(applied.revision, 1);
    assert_ne!(applied.document_json, before.1);

    // Editing a revision the client has not seen is refused rather than
    // silently rebased.
    let stale = apply(
        &socket,
        "edit-apply-stale",
        &doc_id,
        0,
        &EditCommand::SetGain {
            t_ticks: 0,
            gain_db: -3.0,
        },
    )
    .await;
    assert!(stale.is_err(), "a stale revision must conflict");

    // The returned inverse walks the document back to where it started.
    let inverse = EditCommand::from_canonical_json(applied.inverse_command_json.as_bytes())
        .expect("inverse parses");
    let undone = apply(&socket, "edit-undo", &doc_id, 1, &inverse)
        .await
        .expect("undo applies");
    assert_eq!(undone.revision, 2);
    assert_eq!(
        undone.document_json, before.1,
        "applying the returned inverse restores the original bytes"
    );

    // Snapshots are content-addressed and carry the render projection, so a
    // document that differs only in its rationale snapshots to one artifact.
    let (artifact_id, revision) = snapshot(&socket, "edit-snapshot", &doc_id).await;
    assert_eq!(revision, 2);
    let restyled = apply(
        &socket,
        "edit-rationale",
        &doc_id,
        2,
        &EditCommand::SetCueLines {
            cue_id: "cue_a".to_owned(),
            line_word_counts: vec![1, 1],
            presentation: clipmill_edit_ir::Presentation::Reading,
        },
    )
    .await
    .expect("reflow applies");
    assert_eq!(restyled.revision, 3);
    let (changed_artifact, _) = snapshot(&socket, "edit-snapshot-2", &doc_id).await;
    assert_ne!(
        changed_artifact, artifact_id,
        "a change the renderer can see must produce a different snapshot"
    );

    let lease = artifacts
        .open(artifact_id.parse::<ArtifactId>().expect("artifact id"))
        .await
        .expect("open snapshot");
    assert_eq!(lease.kind(), "edit.ir.v1");
    let path = "edit-ir.json".parse::<ArtifactPath>().expect("path");
    let mut file = lease.open_verified(&path).expect("verified payload");
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).expect("read payload");
    let snapshot_document: serde_json::Value =
        serde_json::from_slice(&bytes).expect("snapshot parses");
    assert_eq!(snapshot_document["version"], "ir/1");
    assert!(
        snapshot_document.get("rationale").is_none(),
        "the renderer's copy must not carry the rationale that explains the edit"
    );
    assert!(
        EditDocument::from_canonical_json(&bytes).is_ok(),
        "the snapshot is itself a valid edit document"
    );

    // An export is of the revision that was reviewed. Planning says which
    // revision it looked at; an export naming an older one — the document
    // moved in between — is refused as a conflict, and one naming the current
    // revision gets past that check to whatever the strip has to say.
    let export = |request_id: &str, expected: Option<u64>| {
        let request = ExportRequestV1 {
            doc_id: doc_id.clone(),
            destination_dir: temp.path().join("exports").to_string_lossy().into_owned(),
            naming_pattern: String::new(),
            source_attestation: "own_content".to_owned(),
            gates_passed: Vec::new(),
            ai_assistance: Vec::new(),
            index: 1,
            date: "2026-09-17".to_owned(),
            title: String::new(),
            expected_revision: expected,
        };
        let request_id = request_id.to_owned();
        let socket = socket.clone();
        async move {
            let planned = send(
                &socket,
                Request {
                    request_id: format!("{request_id}-plan"),
                    body: Some(request::Body::PlanExport(PlanExportRequest {
                        request: Some(request.clone()),
                    })),
                },
            )
            .await
            .expect("plan");
            let revision = match planned.body {
                Some(response::Body::PlanExport(plan)) => plan.revision,
                other => panic!("unexpected plan reply: {other:?}"),
            };
            let exported = send(
                &socket,
                Request {
                    request_id,
                    body: Some(request::Body::ExportClip(ExportClipRequest {
                        request: Some(request),
                    })),
                },
            )
            .await
            .expect("export reply");
            (revision, exported.body)
        }
    };
    let (planned_revision, stale) = export("edit-export-stale", Some(2)).await;
    assert_eq!(
        planned_revision, 3,
        "the plan says which revision it looked at"
    );
    match stale {
        Some(response::Body::Error(error)) => {
            assert_eq!(error.code, ErrorCode::Conflict as i32);
            assert!(
                error.message.contains("revision 2 was approved")
                    && error.message.contains("now at revision 3"),
                "{}",
                error.message
            );
        }
        other => panic!("an export of a stale revision must conflict, got {other:?}"),
    }
    let (_, current) = export("edit-export-current", Some(3)).await;
    match &current {
        Some(response::Body::Error(error)) => {
            assert_ne!(
                error.code,
                ErrorCode::Conflict as i32,
                "the reviewed revision is not a conflict: {}",
                error.message
            );
        }
        Some(response::Body::ExportClip(queued)) => {
            // The job carries what it is delivering, so a screen reopened
            // on the document — or the application relaunched — can find
            // the export again from the daemon rather than from state it
            // no longer has.
            let job = get_job(&socket, "edit-export-job", &queued.job_id)
                .await
                .expect("the export job");
            let summary = job.export.expect("an export job says what it delivers");
            assert_eq!(summary.doc_id, doc_id);
            assert_eq!(summary.revision, 3);
            assert_eq!(summary.ir_artifact_id, queued.ir_artifact_id);
            assert_eq!(summary.destination_dir, queued.destination_dir);
            let listed = list_jobs(&socket, "edit-export-list", &project.project_id)
                .await
                .expect("the project's jobs");
            let found = listed
                .iter()
                .find(|job| job.job_id == queued.job_id)
                .expect("the export is listed");
            assert_eq!(
                found.export.as_ref().map(|e| e.doc_id.as_str()),
                Some(doc_id.as_str())
            );
            assert!(
                listed
                    .iter()
                    .filter(|job| job.kind != "export-clip")
                    .all(|job| job.export.is_none()),
                "only an export job says so"
            );
        }
        other => panic!("unexpected export reply: {other:?}"),
    }

    stop(shutdown, task).await;
}

async fn stop(shutdown: oneshot::Sender<()>, task: JoinHandle<Result<(), DaemonError>>) {
    let _sent = shutdown.send(());
    task.await
        .expect("daemon task joins")
        .expect("daemon shuts down");
}

#[tokio::test]
async fn acknowledged_commands_survive_a_killed_daemon() {
    use support::spawn_daemon;

    let temp = workspace_tempdir();
    let data_dir = temp.path().join("data");
    std::fs::create_dir_all(&data_dir).expect("data dir");
    let socket = temp.path().join("clipmill.sock");

    let mut child = spawn_daemon(&data_dir, &socket);
    wait_until_ready(&socket).await.expect("daemon ready");
    let project = create(&socket, "kill-project", "Edits")
        .await
        .expect("project");
    let doc_id = create_doc(
        &socket,
        "kill-create",
        &project.project_id,
        &sample_document(),
    )
    .await;

    let commands = [
        EditCommand::SetLayout {
            segment_id: "seg_a".to_owned(),
            state: clipmill_edit_ir::LayoutState::SpeakerFill,
        },
        EditCommand::EditCaptionText {
            cue_id: "cue_a".to_owned(),
            word_index: 1,
            text: "TWO".to_owned(),
            presentation: clipmill_edit_ir::Presentation::Reading,
        },
        EditCommand::SetGain {
            t_ticks: 0,
            gain_db: -2.5,
        },
    ];
    let mut acknowledged = String::new();
    for (index, command) in commands.iter().enumerate() {
        let applied = apply(
            &socket,
            &format!("kill-apply-{index}"),
            &doc_id,
            index as u64,
            command,
        )
        .await
        .expect("command applies");
        acknowledged = applied.document_json;
    }

    // No graceful shutdown: the acknowledgement is the promise under test.
    child.kill().expect("SIGKILL daemon");
    let _status = wait_for_exit(&mut child).await;

    let mut child = spawn_daemon(&data_dir, &socket);
    wait_until_ready(&socket).await.expect("daemon restarts");
    let (revision, document_json) = get_doc(&socket, "kill-get", &doc_id).await;
    assert_eq!(revision, 3, "every acknowledged command survived");
    assert_eq!(
        document_json, acknowledged,
        "the restarted daemon serves the exact bytes it acknowledged"
    );
    child.kill().expect("stop restarted daemon");
    let _status = wait_for_exit(&mut child).await;

    // The log the restarted daemon holds still replays onto those bytes.
    let daemon = Daemon::start(config_for(&data_dir, &socket))
        .await
        .expect("daemon starts in-process");
    let log = daemon.edit_log(doc_id).await.expect("edit log");
    let mut replayed = EditDocument::from_canonical_json(log.initial_document.as_bytes())
        .expect("initial document");
    assert_eq!(log.commands.len(), 3);
    for (_, command_json, _) in &log.commands {
        EditCommand::from_canonical_json(command_json.as_bytes())
            .expect("logged command parses")
            .apply(&mut replayed)
            .expect("logged command applies");
    }
    assert_eq!(
        String::from_utf8(replayed.to_canonical_json().expect("canonical")).expect("utf-8"),
        document_json,
        "replaying the recovered log reproduces the recovered document"
    );
}

fn config_for(data_dir: &Path, socket: &Path) -> Config {
    Config::from_sources_with_gc(
        Some(data_dir.to_path_buf()),
        Some(socket.to_path_buf()),
        None,
        Some(OsString::from("/ignored/env")),
        None,
        None,
        Some(PathBuf::from("/ignored/ffprobe")),
        None,
        PathBuf::from("/ignored/default"),
    )
    .expect("recovery config")
}
