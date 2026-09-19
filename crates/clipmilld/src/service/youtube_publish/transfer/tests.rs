#![allow(clippy::unwrap_used)]
use super::*;
use crate::db::{DbActor, ProjectRecord, PublishingCommand};
use clipmill_contracts::proto::ipc::v1::{
    YoutubeConnectionV1, YoutubeUploadV1, YoutubeVideoMetadataV1,
};
use std::sync::{Arc, Mutex};

#[allow(
    clippy::struct_excessive_bools,
    reason = "independent provider failure fixture switches"
)]
struct Fake {
    existing: bool,
    offset: u64,
    expired: bool,
    complete_query: bool,
    channel: &'static str,
    began: Mutex<usize>,
    sent: Mutex<Vec<(u64, Vec<u8>)>>,
    pause: Option<(Service, String)>,
    chunk_ack: Option<u64>,
    remote_private: bool,
    published: Mutex<usize>,
}
impl Fake {
    fn new() -> Self {
        Self {
            existing: true,
            offset: 0,
            expired: false,
            complete_query: false,
            channel: "UC_expected",
            began: Mutex::new(0),
            sent: Mutex::new(Vec::new()),
            pause: None,
            chunk_ack: None,
            remote_private: false,
            published: Mutex::new(0),
        }
    }
}
impl UploadApi for Fake {
    type Session = ();
    async fn channel(&self) -> Result<Channel, Error> {
        Ok(Channel {
            id: self.channel.into(),
            title: "Channel".into(),
        })
    }
    fn existing(&self, _id: &str) -> Result<Option<()>, Error> {
        Ok(self.existing.then_some(()))
    }
    async fn begin(&self, _id: &str, _metadata: &Metadata, _total: u64) -> Result<(), Error> {
        *self.began.lock().unwrap() += 1;
        Ok(())
    }
    async fn position(&self, _session: &(), _total: u64) -> Result<UploadReply, Error> {
        if self.expired {
            Err(Error::ExpiredSession)
        } else if self.complete_query {
            Ok(UploadReply::Complete {
                video_id: "abcdefghijk".into(),
                channel_id: self.channel.into(),
                privacy: "private".into(),
            })
        } else {
            Ok(UploadReply::Incomplete {
                acknowledged: self.offset,
            })
        }
    }
    async fn chunk(
        &self,
        _session: &(),
        start: u64,
        _total: u64,
        bytes: Vec<u8>,
    ) -> Result<UploadReply, Error> {
        self.sent.lock().unwrap().push((start, bytes));
        if let Some((service, id)) = &self.pause {
            service
                .database
                .publishing(PublishingCommand::Action {
                    request_id: "pause-inflight".into(),
                    request_hash: [3; 32],
                    id: id.clone(),
                    action: "pause".into(),
                    now: 3,
                })
                .await
                .unwrap();
        }
        if let Some(acknowledged) = self.chunk_ack {
            return Ok(UploadReply::Incomplete { acknowledged });
        }
        Ok(UploadReply::Complete {
            video_id: "abcdefghijk".into(),
            channel_id: self.channel.into(),
            privacy: "private".into(),
        })
    }
    async fn video(&self, id: &str, channel: &str) -> Result<VideoStatus, Error> {
        Ok(VideoStatus {
            id: id.into(),
            channel_id: channel.into(),
            privacy: if self.remote_private {
                "private"
            } else {
                "public"
            }
            .into(),
            status: "processed".into(),
        })
    }
    async fn publish(&self, id: &str, channel: &str) -> Result<VideoStatus, Error> {
        *self.published.lock().unwrap() += 1;
        Ok(VideoStatus {
            id: id.into(),
            channel_id: channel.into(),
            privacy: "public".into(),
            status: "processed".into(),
        })
    }
}

async fn setup() -> (
    tempfile::TempDir,
    DbActor,
    Service,
    Publication,
    tokio::fs::File,
) {
    let root = tempfile::tempdir().unwrap();
    let actor =
        DbActor::start(&root.path().join("store.db"), &root.path().join("backups")).unwrap();
    let service = Service::new(actor.handle(), 1);
    let project = "prj_01ARZ3NDEKTSV4RRFFQ69G5FAV";
    service
        .database
        .create_project(
            "project".into(),
            [1; 32],
            ProjectRecord {
                project_id: project.into(),
                name: "Project".into(),
                created_unix_millis: 1,
            },
        )
        .await
        .unwrap();
    service
        .database
        .publishing(PublishingCommand::SaveConnection {
            record: YoutubeConnectionV1 {
                connection_id: "conn".into(),
                channel_id: "UC_expected".into(),
                state: "connected".into(),
                ..Default::default()
            },
            expected_state: None,
        })
        .await
        .unwrap();
    let record = Publication {
        view: YoutubeUploadV1 {
            upload_id: "upload".into(),
            project_id: project.into(),
            doc_id: "doc".into(),
            channel_id: "UC_expected".into(),
            connection_id: "conn".into(),
            ir_artifact_id: format!("sha256:{}", "11".repeat(32)),
            render_artifact_id: format!("sha256:{}", "22".repeat(32)),
            total_bytes: 8,
            state: "queued".into(),
            metadata: Some(YoutubeVideoMetadataV1 {
                title: "Approved".into(),
                ..Default::default()
            }),
            ..Default::default()
        },
        sha256: String::new(),
        generation: 0,
        session_started: false,
        final_possible: false,
        publish_intent: false,
        paused: false,
        intent_epoch: 0,
        reset_session: false,
    };
    service
        .database
        .publishing(PublishingCommand::Create {
            request_id: "upload".into(),
            request_hash: [2; 32],
            record,
        })
        .await
        .unwrap();
    let record = service
        .database
        .publishing(PublishingCommand::Claim {
            id: "upload".into(),
            now: 2,
        })
        .await
        .unwrap()
        .uploads
        .remove(0);
    let path = root.path().join("clip.mp4");
    std::fs::write(&path, b"abcdefgh").unwrap();
    let file = tokio::fs::File::open(path).await.unwrap();
    (root, actor, service, record, file)
}

#[tokio::test]
async fn resume_uses_server_position_and_preserves_late_success_after_pause() {
    let (_root, actor, service, mut record, file) = setup().await;
    let mut fake = Fake::new();
    fake.offset = 3;
    fake.pause = Some((service.clone(), record.view.upload_id.clone()));
    service
        .transfer_verified_bytes(&fake, &mut record, file)
        .await
        .unwrap();
    assert_eq!(*fake.began.lock().unwrap(), 0);
    assert_eq!(*fake.sent.lock().unwrap(), vec![(3, b"defgh".to_vec())]);
    let saved = service.publication("upload").await.unwrap();
    assert_eq!(saved.view.state, "private");
    assert_eq!(saved.view.video_id, "abcdefghijk");
    assert!(saved.final_possible);
    assert!(
        service
            .database
            .list_artifact_roots()
            .await
            .unwrap()
            .is_empty(),
        "remote receipt releases upload-only roots"
    );
    actor.shutdown().await.unwrap();
}
#[tokio::test]
async fn completed_session_query_does_not_send_or_begin_again() {
    let (_root, actor, service, mut record, file) = setup().await;
    let mut fake = Fake::new();
    fake.complete_query = true;
    service
        .transfer_verified_bytes(&fake, &mut record, file)
        .await
        .unwrap();
    assert_eq!(record.view.state, "private");
    assert!(fake.sent.lock().unwrap().is_empty());
    assert_eq!(*fake.began.lock().unwrap(), 0);
    actor.shutdown().await.unwrap();
}
#[tokio::test]
async fn expired_ambiguous_final_session_never_creates_a_replacement_video() {
    let (_root, actor, service, mut record, file) = setup().await;
    record.final_possible = true;
    record.session_started = true;
    service.checkpoint_publication(&mut record).await.unwrap();
    let mut fake = Fake::new();
    fake.expired = true;
    let error = service
        .transfer_verified_bytes(&fake, &mut record, file)
        .await
        .unwrap_err();
    service.fail_publication(&mut record, &error, "conn").await;
    assert_eq!(
        service.publication("upload").await.unwrap().view.state,
        "completion_uncertain"
    );
    assert_eq!(*fake.began.lock().unwrap(), 0);
    assert!(fake.sent.lock().unwrap().is_empty());
    actor.shutdown().await.unwrap();
}
#[tokio::test]
async fn complete_byte_acknowledgement_without_receipt_does_not_claim_success() {
    let (_root, actor, service, mut record, file) = setup().await;
    let mut fake = Fake::new();
    fake.offset = 8;
    let error = service
        .transfer_verified_bytes(&fake, &mut record, file)
        .await
        .unwrap_err();
    service.fail_publication(&mut record, &error, "conn").await;
    assert_eq!(record.view.state, "completion_uncertain");
    assert!(record.view.video_id.is_empty());
    assert!(fake.sent.lock().unwrap().is_empty());
    actor.shutdown().await.unwrap();
}
#[tokio::test]
async fn wrong_remote_channel_is_refused_before_artifact_or_media_access() {
    let (_root, actor, service, mut record, _file) = setup().await;
    let mut fake = Fake::new();
    fake.channel = "UC_other";
    assert!(
        service
            .transfer_publication(&fake, &mut record)
            .await
            .is_err()
    );
    assert!(fake.sent.lock().unwrap().is_empty());
    assert_eq!(*fake.began.lock().unwrap(), 0);
    actor.shutdown().await.unwrap();
}

#[tokio::test]
async fn unchanged_or_regressing_chunk_acknowledgement_stops_after_one_request() {
    for acknowledged in [0, 3] {
        let (_root, actor, service, mut record, file) = setup().await;
        let mut fake = Fake::new();
        fake.offset = 3;
        fake.chunk_ack = Some(acknowledged);
        let error = service
            .transfer_verified_bytes(&fake, &mut record, file)
            .await
            .unwrap_err();
        assert!(matches!(error, Error::Protocol));
        assert_eq!(fake.sent.lock().unwrap().len(), 1);
        assert!(record.view.video_id.is_empty());
        actor.shutdown().await.unwrap();
    }
}

#[tokio::test]
async fn publish_requested_while_previous_worker_retires_runs_once() {
    let (_root, actor, service, mut record, file) = setup().await;
    let mut fake = Fake::new();
    fake.remote_private = true;
    let fake = Arc::new(fake);
    let saved_receipt = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Notify::new());
    let worker_service = service.clone();
    let worker_fake = fake.clone();
    let worker_saved = saved_receipt.clone();
    let worker_release = release.clone();
    let (done_tx, done_rx) = tokio::sync::oneshot::channel();
    let mut tasks = service.publishing.running.lock().await;
    let task = tokio::spawn(async move {
        worker_service
            .transfer_verified_bytes(worker_fake.as_ref(), &mut record, file)
            .await
            .unwrap();
        worker_saved.notify_one();
        worker_release.notified().await;
        assert!(worker_service.continue_publication("upload").await);
        let mut claimed = worker_service
            .database
            .publishing(PublishingCommand::Claim {
                id: "upload".into(),
                now: 4,
            })
            .await
            .unwrap()
            .uploads
            .remove(0);
        worker_service
            .transfer_publication(worker_fake.as_ref(), &mut claimed)
            .await
            .unwrap();
        assert!(!worker_service.continue_publication("upload").await);
        let _ = done_tx.send(());
    });
    tasks.insert("upload".into(), ("conn".into(), task));
    drop(tasks);
    saved_receipt.notified().await;
    let pending = service
        .database
        .publishing(PublishingCommand::Action {
            request_id: "publish".into(),
            request_hash: [4; 32],
            id: "upload".into(),
            action: "publish".into(),
            now: 3,
        })
        .await
        .unwrap()
        .uploads
        .remove(0);
    service.dispatch_publication(&pending).await;
    release.notify_one();
    tokio::time::timeout(std::time::Duration::from_secs(2), done_rx)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(*fake.published.lock().unwrap(), 1);
    assert_eq!(
        service.publication("upload").await.unwrap().view.state,
        "public"
    );
    assert!(service.publishing.running.lock().await.is_empty());
    actor.shutdown().await.unwrap();
}

#[tokio::test]
async fn pause_serialized_before_publish_checkpoint_prevents_visibility_change() {
    let (_root, actor, service, mut record, _file) = setup().await;
    record.view.state = "private".into();
    record.view.video_id = "abcdefghijk".into();
    service.checkpoint_publication(&mut record).await.unwrap();
    record = service
        .database
        .publishing(PublishingCommand::Action {
            request_id: "publish".into(),
            request_hash: [4; 32],
            id: "upload".into(),
            action: "publish".into(),
            now: 3,
        })
        .await
        .unwrap()
        .uploads
        .remove(0);
    // Simulate the guard read already succeeding, followed by the ordered
    // Pause transaction before the final pre-request checkpoint.
    assert!(!service.publication_paused(&mut record).await.unwrap());
    service
        .database
        .publishing(PublishingCommand::Action {
            request_id: "pause".into(),
            request_hash: [5; 32],
            id: "upload".into(),
            action: "pause".into(),
            now: 4,
        })
        .await
        .unwrap();
    let fake = Fake::new();
    assert!(
        service
            .publish_visibility(&fake, &mut record, "conn")
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(*fake.published.lock().unwrap(), 0);
    assert_eq!(
        service.publication("upload").await.unwrap().view.state,
        "paused"
    );
    actor.shutdown().await.unwrap();
}

#[tokio::test]
async fn reauthorization_recovers_same_channel_without_disconnecting_old_connection_first() {
    let (_root, actor, service, mut record, _file) = setup().await;
    service
        .fail_publication(&mut record, &Error::Authorization, "conn")
        .await;
    assert_eq!(service.connection("conn").await.unwrap().state, "failed");
    for (id, channel) in [("replacement", "UC_expected"), ("other", "UC_other")] {
        service
            .database
            .publishing(PublishingCommand::SaveConnection {
                record: YoutubeConnectionV1 {
                    connection_id: id.into(),
                    channel_id: channel.into(),
                    state: "connected".into(),
                    ..Default::default()
                },
                expected_state: None,
            })
            .await
            .unwrap();
    }
    let resumed = service
        .database
        .publishing(PublishingCommand::Action {
            request_id: "resume".into(),
            request_hash: [6; 32],
            id: "upload".into(),
            action: "resume".into(),
            now: 5,
        })
        .await
        .unwrap()
        .uploads
        .remove(0);
    assert_eq!(resumed.view.connection_id, "replacement");
    // A stale failure is attributed to the client that sent it, never the new one.
    service
        .fail_publication(&mut record, &Error::Authorization, "conn")
        .await;
    assert_eq!(
        service.connection("replacement").await.unwrap().state,
        "connected"
    );
    assert_eq!(
        service
            .publication("upload")
            .await
            .unwrap()
            .view
            .connection_id,
        "replacement"
    );
    actor.shutdown().await.unwrap();
}

#[tokio::test]
async fn recovered_receipt_preserves_visibility_changed_in_youtube_studio() {
    for privacy in ["public", "unlisted"] {
        let (_root, actor, service, mut record, _file) = setup().await;
        service
            .apply_upload_reply(
                &mut record,
                UploadReply::Complete {
                    video_id: "abcdefghijk".into(),
                    channel_id: "UC_expected".into(),
                    privacy: privacy.into(),
                },
            )
            .await
            .unwrap();
        let saved = service.publication("upload").await.unwrap();
        assert_eq!(saved.view.visibility, privacy);
        assert_eq!(
            saved.view.state,
            if privacy == "public" {
                "public"
            } else {
                "private"
            }
        );
        assert!(
            !saved.publish_intent,
            "observing an existing public video grants no new publication intent"
        );
        actor.shutdown().await.unwrap();
    }
}
