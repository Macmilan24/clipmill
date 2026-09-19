//! Export intent survives mutable edits and interruption at either durable boundary.
//! No encoder or model is needed: these tests exercise admission and recovery over
//! a real daemon socket. Rendering/delivery are verified by separate media drills.
#![cfg(unix)]
#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod support;

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use clipmill_contracts::proto::ipc::v1::{
    ApplyEditCommandRequest, ErrorCode, ExportBatchItemV1, ExportBatchResponse, ExportBatchV1,
    ExportClipRequest, ExportClipResponse, ExportRequestV1, ListExportBatchesRequest, Request,
    Response, SubmitExportBatchRequest, UpdateExportBatchItemRequest, request, response,
};
use clipmilld::{Config, Daemon, DaemonError};
use prost::Message;
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};
use support::{create, create_edit_doc, list_jobs, send, wait_until_ready, workspace_tempdir};
use tempfile::TempDir;
use tokio::{sync::oneshot, task::JoinHandle, time::sleep};

struct Running {
    socket: PathBuf,
    shutdown: oneshot::Sender<()>,
    task: JoinHandle<Result<(), DaemonError>>,
}
impl Running {
    async fn start(temp: &TempDir) -> Self {
        let config = Config::from_sources_with_gc(
            Some(temp.path().to_path_buf()),
            None,
            None,
            None,
            None,
            None,
            Some(temp.path().join("no-ffprobe")),
            None,
            temp.path().to_path_buf(),
        )
        .expect("isolated export test config");
        let daemon = Daemon::start(config).await.expect("daemon starts");
        let socket = daemon.socket_path().to_path_buf();
        let (shutdown, stopped) = oneshot::channel();
        let task = tokio::spawn(daemon.serve_until(async {
            let _ = stopped.await;
        }));
        wait_until_ready(&socket).await.expect("daemon ready");
        Self {
            socket,
            shutdown,
            task,
        }
    }
    async fn stop(self) {
        self.shutdown.send(()).expect("stop signal");
        self.task
            .await
            .expect("daemon joins")
            .expect("daemon stops");
    }
}

fn document() -> String {
    serde_json::json!({
        "version": "ir/1", "timebase": {"num":1,"den":90000},
        "video": {"segments":[{
            "segment_id":"seg_a", "source_fingerprint":format!("sha256:{}", "cd".repeat(32)),
            "in_ticks":0,"out_ticks":900_000,"layout":{"state":"fit"}
        }]},
        "captions":{"style_ref":"clean","cues":[]},
        "audio":{"target_lufs":-14.0,"true_peak_dbtp":-1.0},
        "rationale":{"candidate_id":"manual_recovery","decisions":[]}
    })
    .to_string()
}

async fn setup(socket: &Path) -> (String, String, String) {
    let project = create(socket, "project", "Export recovery")
        .await
        .expect("project");
    let a = create_edit_doc(socket, "doc-a", &project.project_id, &document())
        .await
        .expect("doc a");
    let b = create_edit_doc(socket, "doc-b", &project.project_id, &document())
        .await
        .expect("doc b");
    (project.project_id, a, b)
}

fn asked(temp: &TempDir, doc_id: &str, index: u32) -> ExportRequestV1 {
    ExportRequestV1 {
        doc_id: doc_id.to_owned(),
        destination_dir: temp.path().join("exports").to_string_lossy().into_owned(),
        naming_pattern: "{index}-{clip}".to_owned(),
        source_attestation: "own_content".to_owned(),
        index,
        date: "2026-09-18".to_owned(),
        title: format!("Clip {index}"),
        expected_revision: Some(0),
        ..Default::default()
    }
}

async fn call(socket: &Path, id: &str, body: request::Body) -> response::Body {
    send(
        socket,
        Request {
            request_id: id.to_owned(),
            body: Some(body),
        },
    )
    .await
    .expect("socket response")
    .body
    .expect("response body")
}

fn exported(body: response::Body) -> ExportClipResponse {
    match body {
        response::Body::ExportClip(reply) => reply,
        other => panic!("expected export acceptance, got {other:?}"),
    }
}
fn batch(body: response::Body) -> ExportBatchV1 {
    match body {
        response::Body::ExportBatch(reply) => reply.batch.expect("batch"),
        other => panic!("expected durable batch, got {other:?}"),
    }
}
fn export_body(request: &ExportRequestV1) -> request::Body {
    request::Body::ExportClip(ExportClipRequest {
        request: Some(request.clone()),
    })
}
fn batch_body(requests: Vec<ExportRequestV1>) -> request::Body {
    request::Body::SubmitExportBatch(SubmitExportBatchRequest { requests })
}
fn update_body(id: &str, index: u32, action: &str) -> request::Body {
    request::Body::UpdateExportBatchItem(UpdateExportBatchItemRequest {
        batch_id: id.to_owned(),
        index,
        action: action.to_owned(),
    })
}
async fn list_batches(socket: &Path) -> Vec<ExportBatchV1> {
    match call(
        socket,
        "list-batches",
        request::Body::ListExportBatches(ListExportBatchesRequest {}),
    )
    .await
    {
        response::Body::ListExportBatches(reply) => reply.batches,
        other => panic!("expected batch list, got {other:?}"),
    }
}
async fn admitted(socket: &Path, id: &str) -> ExportBatchV1 {
    for _ in 0..200 {
        let current = list_batches(socket)
            .await
            .into_iter()
            .find(|batch| batch.batch_id == id)
            .expect("saved batch remains listed");
        if current.items.iter().all(|item| item.state != "pending") {
            return current;
        }
        sleep(Duration::from_millis(10)).await;
    }
    panic!("batch admission did not settle");
}
async fn change_revision(socket: &Path, doc_id: &str) {
    let body = call(
        socket,
        "change-revision",
        request::Body::ApplyEditCommand(ApplyEditCommandRequest {
            doc_id: doc_id.to_owned(),
            expected_revision: 0,
            command_json: serde_json::json!({"op":"set_gain","t_ticks":0,"gain_db":-3.0})
                .to_string(),
        }),
    )
    .await;
    match body {
        response::Body::ApplyEditCommand(reply) => {
            assert_eq!(reply.doc.expect("changed doc").revision, 1);
        }
        other => panic!("expected edit revision one, got {other:?}"),
    }
}

#[tokio::test]
async fn accepted_export_replays_before_revision_revalidation_after_restart() {
    let temp = workspace_tempdir();
    let live = Running::start(&temp).await;
    let (project, doc, _) = setup(&live.socket).await;
    let request = asked(&temp, &doc, 1);
    let original = exported(call(&live.socket, "stable-export", export_body(&request)).await);
    assert_eq!(original.revision, 0);
    change_revision(&live.socket, &doc).await;
    live.stop().await;

    let live = Running::start(&temp).await;
    let replay = exported(call(&live.socket, "stable-export", export_body(&request)).await);
    assert_eq!(
        replay, original,
        "lost response recovers the frozen export, including its job"
    );
    let fresh = call(&live.socket, "different-intent", export_body(&request)).await;
    assert!(
        matches!(fresh, response::Body::Error(error) if error.code == ErrorCode::Conflict as i32)
    );
    assert_eq!(
        list_jobs(&live.socket, "jobs", &project)
            .await
            .expect("jobs")
            .len(),
        1
    );
    live.stop().await;
}

#[tokio::test]
async fn batch_and_item_retries_are_idempotent_and_keep_the_approved_snapshot() {
    let temp = workspace_tempdir();
    let live = Running::start(&temp).await;
    let (project, doc, other_doc) = setup(&live.socket).await;
    let first = asked(&temp, &doc, 1);
    let mut stale = asked(&temp, &other_doc, 2);
    stale.expected_revision = Some(9);
    let submit = batch_body(vec![first.clone(), stale.clone()]);
    let saved = batch(call(&live.socket, "stable-batch", submit.clone()).await);
    let settled = admitted(&live.socket, &saved.batch_id).await;
    assert_eq!(settled.items[0].state, "queued");
    assert_eq!(settled.items[1].state, "failed");
    assert!(settled.items[1].error.contains("revision"));
    let original = settled.items[0]
        .queued
        .clone()
        .expect("accepted first export");
    assert_eq!(
        batch(call(&live.socket, "stable-batch", submit.clone()).await).batch_id,
        saved.batch_id
    );
    assert_eq!(list_batches(&live.socket).await.len(), 1);
    let cancel = update_body(&saved.batch_id, 1, "cancel");
    let cancelled = batch(call(&live.socket, "cancel-once", cancel.clone()).await);
    assert_eq!(cancelled.items[0].state, "cancelled");
    change_revision(&live.socket, &doc).await;
    live.stop().await;

    let live = Running::start(&temp).await;
    assert_eq!(
        batch(call(&live.socket, "stable-batch", submit).await).batch_id,
        saved.batch_id
    );
    assert_eq!(
        batch(call(&live.socket, "cancel-once", cancel.clone()).await),
        cancelled
    );
    let retry = update_body(&saved.batch_id, 1, "retry");
    let retry_receipt = batch(call(&live.socket, "retry-once", retry.clone()).await);
    assert_eq!(retry_receipt.items[0].attempt, 1);
    let retried = admitted(&live.socket, &saved.batch_id).await;
    let new_export = retried.items[0]
        .queued
        .as_ref()
        .expect("new export attempt");
    assert_ne!(new_export.job_id, original.job_id);
    assert_eq!(
        new_export.revision, 0,
        "retry does not approve the document's new revision"
    );
    assert_eq!(new_export.ir_artifact_id, original.ir_artifact_id);
    assert_eq!(
        batch(call(&live.socket, "retry-once", retry).await),
        retry_receipt
    );
    // An old cancellation's lost response cannot cancel the new attempt.
    assert_eq!(
        batch(call(&live.socket, "cancel-once", cancel).await),
        cancelled
    );
    let unchanged = admitted(&live.socket, &saved.batch_id).await;
    assert_eq!(unchanged.items[0].state, "queued");
    assert_eq!(unchanged.items[0].attempt, 1);
    assert_eq!(unchanged.items[0].queued, retried.items[0].queued);
    assert_eq!(unchanged.items[1].state, "failed");
    assert_eq!(unchanged.items[1].attempt, 0);
    assert_eq!(
        list_jobs(&live.socket, "jobs", &project)
            .await
            .expect("jobs")
            .len(),
        2
    );
    live.stop().await;
}

/// Reconstruct the exact committed batch intent before asynchronous admission.
/// This is deliberate stopped-store boundary injection, not a timed crash claim.
fn seed_pending_boundary(temp: &TempDir, project: &str, request: &ExportRequestV1) -> String {
    let id = "batch_pending_boundary";
    let envelope = Request {
        request_id: "pending-boundary".to_owned(),
        body: Some(batch_body(vec![request.clone()])),
    };
    let hash: [u8; 32] = Sha256::digest(envelope.encode_to_vec()).into();
    let receipt = Response {
        request_id: envelope.request_id.clone(),
        body: Some(response::Body::ExportBatch(ExportBatchResponse {
            batch: Some(ExportBatchV1 {
                batch_id: id.to_owned(),
                created_unix_millis: 1,
                items: vec![ExportBatchItemV1 {
                    project_id: project.to_owned(),
                    index: request.index,
                    request: Some(request.clone()),
                    state: "pending".to_owned(),
                    attempt: 0,
                    queued: None,
                    error: String::new(),
                }],
            }),
        })),
    };
    let mut connection =
        Connection::open(temp.path().join("state/clipmill.db")).expect("stopped store");
    let tx = connection.transaction().expect("boundary transaction");
    tx.execute(
        "INSERT INTO export_batches VALUES (?1,?2,?3,1)",
        params![id, envelope.request_id, hash.as_slice()],
    )
    .expect("intent");
    tx.execute("INSERT INTO export_batch_items(batch_id,item_index,doc_id,request,state) VALUES (?1,?2,?3,?4,'pending')", params![id,request.index,request.doc_id,request.encode_to_vec()]).expect("pending item");
    tx.execute(
        "INSERT INTO request_dedup VALUES (?1,?2,?3,1)",
        params![
            envelope.request_id,
            hash.as_slice(),
            receipt.encode_to_vec()
        ],
    )
    .expect("intent receipt");
    tx.commit().expect("commit boundary");
    id.to_owned()
}

#[tokio::test]
async fn restart_recovers_before_admission_and_after_job_acceptance_before_linking() {
    let temp = workspace_tempdir();
    let live = Running::start(&temp).await;
    let (project, doc, pending_doc) = setup(&live.socket).await;
    let request = asked(&temp, &doc, 1);
    let saved = batch(call(&live.socket, "accepted-boundary", batch_body(vec![request])).await);
    let settled = admitted(&live.socket, &saved.batch_id).await;
    let original = settled.items[0].queued.clone().expect("accepted export");
    change_revision(&live.socket, &doc).await;
    live.stop().await;

    // Remove only the second durable write. The accepted job and stable request
    // receipt remain exactly as written by the daemon, and the edit is now newer.
    {
        let db = Connection::open(temp.path().join("state/clipmill.db")).expect("stopped store");
        assert_eq!(db.execute("UPDATE export_batch_items SET state='pending',queued=NULL WHERE batch_id=?1 AND item_index=1", [&saved.batch_id]).expect("rewind link boundary"), 1);
    }
    let pending_request = asked(&temp, &pending_doc, 2);
    let pending_id = seed_pending_boundary(&temp, &project, &pending_request);

    let live = Running::start(&temp).await;
    let recovered = admitted(&live.socket, &saved.batch_id).await;
    assert_eq!(recovered.items[0].state, "queued");
    assert_eq!(recovered.items[0].attempt, 0);
    assert_eq!(
        recovered.items[0].queued.as_ref(),
        Some(&original),
        "recovery links the original job even after a later edit"
    );
    let pending = admitted(&live.socket, &pending_id).await;
    assert_eq!(pending.items[0].state, "queued");
    assert_eq!(pending.items[0].attempt, 0);
    assert_eq!(pending.items[0].request.as_ref(), Some(&pending_request));
    assert_eq!(pending.items[0].project_id, project);
    assert_eq!(list_batches(&live.socket).await.len(), 2);
    assert_eq!(
        list_jobs(&live.socket, "jobs", &project)
            .await
            .expect("jobs")
            .len(),
        2,
        "one job per approved intent"
    );
    live.stop().await;
}
