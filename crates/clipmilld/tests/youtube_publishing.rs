//! Real IPC and SQLite recovery, with a seeded remote-side-effect ledger.
//! No Google account or request is used; transport protocol tests are separate.
#![cfg(unix)]
#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod support;

use clipmill_contracts::proto::ipc::v1::{
    ConfigureYoutubePublishingRequest, DeleteProjectRequest, ErrorCode, GetYoutubeUploadRequest,
    ListYoutubeUploadsRequest, PublishYoutubeUploadRequest, Request, StartYoutubeUploadRequest,
    UpdateYoutubeUploadRequest, YoutubeConnectionV1, YoutubeUploadV1, YoutubeVideoMetadataV1,
    request, response,
};
use clipmilld::{Config, Daemon, DaemonError};
use prost::Message;
use rusqlite::{Connection, params};
use std::path::PathBuf;
use support::{create, send, wait_until_ready, workspace_tempdir};
use tempfile::TempDir;
use tokio::{sync::oneshot, task::JoinHandle};

struct Running {
    socket: PathBuf,
    stop: oneshot::Sender<()>,
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
        .unwrap();
        let daemon = Daemon::start(config).await.unwrap();
        let socket = daemon.socket_path().to_path_buf();
        let (stop, stopped) = oneshot::channel();
        let task = tokio::spawn(daemon.serve_until(async {
            let _ = stopped.await;
        }));
        wait_until_ready(&socket).await.unwrap();
        Self { socket, stop, task }
    }
    async fn call(&self, id: &str, body: request::Body) -> response::Body {
        send(
            &self.socket,
            Request {
                request_id: id.into(),
                body: Some(body),
            },
        )
        .await
        .unwrap()
        .body
        .unwrap()
    }
    async fn stop(self) {
        self.stop.send(()).unwrap();
        self.task.await.unwrap().unwrap();
    }
}
fn upload(body: response::Body) -> YoutubeUploadV1 {
    match body {
        response::Body::YoutubeUpload(value) => value.record.unwrap(),
        other => panic!("expected upload, got {other:?}"),
    }
}

#[tokio::test]
async fn interrupted_remote_intent_is_paused_replayable_and_survives_project_deletion() {
    let temp = workspace_tempdir();
    let live = Running::start(&temp).await;
    let project = create(&live.socket, "create", "Publishing recovery")
        .await
        .unwrap();
    live.stop().await;
    let db = Connection::open(temp.path().join("state/clipmill.db")).unwrap();
    let connection = YoutubeConnectionV1 {
        connection_id: "fixture-connection".into(),
        channel_id: "UC_fixture".into(),
        state: "connected".into(),
        ..Default::default()
    };
    db.execute(
        "INSERT INTO youtube_connections VALUES(?1,?2)",
        params![connection.connection_id, connection.encode_to_vec()],
    )
    .unwrap();
    let original = YoutubeUploadV1 {
        upload_id: "fixture-upload".into(),
        project_id: project.project_id.clone(),
        doc_id: "fixture-doc".into(),
        revision: 7,
        channel_id: connection.channel_id,
        connection_id: connection.connection_id,
        ir_artifact_id: format!("sha256:{}", "11".repeat(32)),
        render_artifact_id: format!("sha256:{}", "22".repeat(32)),
        state: "uploading".into(),
        total_bytes: 1000,
        acknowledged_bytes: 750,
        metadata: Some(YoutubeVideoMetadataV1 {
            title: "Reviewed title".into(),
            ..Default::default()
        }),
        ..Default::default()
    };
    db.execute("INSERT INTO youtube_uploads(upload_id,project_id,channel_id,doc_id,revision,ir_artifact_id,record,sha256,session_started,final_possible) VALUES(?1,?2,?3,?4,?5,?6,?7,'fixture',1,1)", params![original.upload_id,original.project_id,original.channel_id,original.doc_id,7,original.ir_artifact_id,original.encode_to_vec()]).unwrap();
    drop(db);
    let live = Running::start(&temp).await;
    let current = upload(
        live.call(
            "get",
            request::Body::GetYoutubeUpload(GetYoutubeUploadRequest {
                upload_id: original.upload_id.clone(),
            }),
        )
        .await,
    );
    assert_eq!(current.state, "paused");
    assert_eq!(current.acknowledged_bytes, 750);
    assert!(current.video_id.is_empty());
    let pause = request::Body::UpdateYoutubeUpload(UpdateYoutubeUploadRequest {
        upload_id: original.upload_id.clone(),
        action: "pause".into(),
    });
    let first = live.call("pause", pause.clone()).await;
    assert_eq!(first, live.call("pause", pause).await);
    assert!(
        matches!(live.call("publish",request::Body::PublishYoutubeUpload(PublishYoutubeUploadRequest { upload_id: original.upload_id.clone() })).await, response::Body::Error(value) if value.code==ErrorCode::Conflict as i32)
    );
    live.call(
        "delete",
        request::Body::DeleteProject(DeleteProjectRequest {
            project_id: project.project_id,
        }),
    )
    .await;
    live.stop().await;
    let live = Running::start(&temp).await;
    match live
        .call(
            "all",
            request::Body::ListYoutubeUploads(ListYoutubeUploadsRequest {
                project_id: String::new(),
            }),
        )
        .await
    {
        response::Body::ListYoutubeUploads(value) => {
            assert_eq!(value.uploads.len(), 1);
            assert_eq!(value.uploads[0].upload_id, original.upload_id);
            assert_eq!(value.uploads[0].state, "paused");
        }
        other => panic!("expected history, got {other:?}"),
    }
    live.stop().await;
}

#[tokio::test]
async fn upload_requires_explicit_rights_and_client_configuration_is_bounded() {
    let temp = workspace_tempdir();
    let live = Running::start(&temp).await;
    let answer = live
        .call(
            "unapproved",
            request::Body::StartYoutubeUpload(StartYoutubeUploadRequest {
                export_job_id: "anything".into(),
                rights_confirmed: false,
                ..Default::default()
            }),
        )
        .await;
    assert!(
        matches!(answer,response::Body::Error(value) if value.code==ErrorCode::PolicyDenied as i32)
    );
    let path = temp.path().join("too-large.json");
    std::fs::write(&path, vec![b' '; 65 * 1024]).unwrap();
    let answer = live
        .call(
            "oversize",
            request::Body::ConfigureYoutubePublishing(ConfigureYoutubePublishingRequest {
                client_config_path: path.to_string_lossy().into_owned(),
            }),
        )
        .await;
    assert!(
        matches!(answer,response::Body::Error(value) if value.code==ErrorCode::InvalidArgument as i32)
    );
    live.stop().await;
}
