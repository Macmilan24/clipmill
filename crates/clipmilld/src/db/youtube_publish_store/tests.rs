#![allow(clippy::unwrap_used)]
use super::*;
use crate::db::{list_artifact_roots, open_database};
use clipmill_contracts::proto::ipc::v1::YoutubeVideoMetadataV1;

const PROJECT: &str = "prj_01ARZ3NDEKTSV4RRFFQ69G5FAV";

fn metadata_plan(payload: &[u8], time: u64) -> crate::jobs::JobPlan {
    let models = crate::models::ModelRegistry::load(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../models/registry"),
    )
    .unwrap();
    crate::jobs::JobPlan::youtube_metadata(
        &PROJECT.parse().unwrap(),
        payload.to_vec(),
        format!("sha256:{}", "44".repeat(32)).parse().unwrap(),
        &models,
        time,
    )
    .unwrap()
}
fn metadata_command(
    payload: &[u8],
    plan: Option<crate::jobs::JobPlan>,
    retry: &str,
) -> PublishingCommand {
    PublishingCommand::MetadataJob {
        project_id: PROJECT.into(),
        payload: payload.into(),
        job_id: String::new(),
        plan: plan.map(Box::new),
        retry_job_id: retry.into(),
    }
}

#[test]
fn metadata_find_or_submit_reuses_completed_work_and_survives_store_restart() {
    use clipmill_contracts::proto::ipc::v1::JobState;
    let (temp, mut db) = setup();
    let payload = b"immutable-edit-model-prompt";
    let first = execute(
        &mut db,
        metadata_command(payload, Some(metadata_plan(payload, 10)), ""),
    )
    .unwrap()
    .metadata_job
    .unwrap();
    let repeat = execute(
        &mut db,
        metadata_command(payload, Some(metadata_plan(payload, 11)), ""),
    )
    .unwrap();
    assert_eq!(first.job_id, repeat.metadata_job.unwrap().job_id);
    assert!(repeat.events.is_empty());
    db.execute(
        "UPDATE jobs SET state=?1 WHERE job_id=?2",
        params![JobState::Succeeded as i32, first.job_id],
    )
    .unwrap();
    drop(db);
    let mut db =
        open_database(&temp.path().join("store.db"), &temp.path().join("backups")).unwrap();
    let restored = execute(
        &mut db,
        metadata_command(payload, Some(metadata_plan(payload, 12)), ""),
    )
    .unwrap()
    .metadata_job
    .unwrap();
    assert_eq!(restored.job_id, first.job_id);
    assert_eq!(restored.state, JobState::Succeeded as i32);
    assert!(
        execute(
            &mut db,
            metadata_command(payload, Some(metadata_plan(payload, 13)), &first.job_id)
        )
        .is_err(),
        "successful work cannot be retried into another model call"
    );
    let changed = b"new-immutable-edit-model-prompt";
    let next = execute(
        &mut db,
        metadata_command(changed, Some(metadata_plan(changed, 14)), ""),
    )
    .unwrap()
    .metadata_job
    .unwrap();
    assert_ne!(next.job_id, first.job_id);
}

#[test]
fn metadata_retry_is_predecessor_fenced_and_does_not_revive_cancellation() {
    use clipmill_contracts::proto::ipc::v1::JobState;
    let (_temp, mut db) = setup();
    let payload = b"immutable-edit-model-prompt";
    let first = execute(
        &mut db,
        metadata_command(payload, Some(metadata_plan(payload, 10)), ""),
    )
    .unwrap()
    .metadata_job
    .unwrap();
    db.execute(
        "UPDATE jobs SET state=?1 WHERE job_id=?2",
        params![JobState::Cancelled as i32, first.job_id],
    )
    .unwrap();
    let still_cancelled = execute(
        &mut db,
        metadata_command(payload, Some(metadata_plan(payload, 11)), ""),
    )
    .unwrap()
    .metadata_job
    .unwrap();
    assert_eq!(still_cancelled.job_id, first.job_id);
    let retried = execute(
        &mut db,
        metadata_command(payload, Some(metadata_plan(payload, 12)), &first.job_id),
    )
    .unwrap()
    .metadata_job
    .unwrap();
    assert_ne!(retried.job_id, first.job_id);
    db.execute(
        "UPDATE jobs SET state=?1 WHERE job_id=?2",
        params![JobState::Failed as i32, retried.job_id],
    )
    .unwrap();
    let replayed = execute(
        &mut db,
        metadata_command(payload, Some(metadata_plan(payload, 13)), &first.job_id),
    )
    .unwrap()
    .metadata_job
    .unwrap();
    assert_eq!(
        replayed.job_id, retried.job_id,
        "a replay names the same failed predecessor, not a new retry"
    );
    assert_eq!(replayed.state, JobState::Failed as i32);
}

#[test]
fn metadata_status_refuses_foreign_job_or_changed_snapshot() {
    let (_temp, mut db) = setup();
    let payload = b"immutable-edit-model-prompt";
    let saved = execute(
        &mut db,
        metadata_command(payload, Some(metadata_plan(payload, 10)), ""),
    )
    .unwrap()
    .metadata_job
    .unwrap();
    assert!(matches!(
        execute(
            &mut db,
            PublishingCommand::MetadataJob {
                project_id: PROJECT.into(),
                payload: b"different-edit".to_vec(),
                job_id: saved.job_id,
                plan: None,
                retry_job_id: String::new(),
            }
        ),
        Err(StoreError::NotFound)
    ));
    let unknown: String = clipmill_core::JobId::new().to_string();
    assert!(matches!(
        execute(
            &mut db,
            PublishingCommand::MetadataJob {
                project_id: PROJECT.into(),
                payload: payload.to_vec(),
                job_id: unknown,
                plan: None,
                retry_job_id: String::new(),
            }
        ),
        Err(StoreError::NotFound)
    ));
}

#[tokio::test]
async fn concurrent_metadata_requests_share_one_job_and_cancel_the_same_work() {
    use clipmill_contracts::proto::ipc::v1::JobState;
    let (temp, db) = setup();
    drop(db);
    let actor =
        crate::db::DbActor::start(&temp.path().join("store.db"), &temp.path().join("backups"))
            .unwrap();
    let database = actor.handle();
    let payload = b"concurrent-immutable-edit-model-prompt";
    let first_request = metadata_command(payload, Some(metadata_plan(payload, 10)), "");
    let second_request = metadata_command(payload, Some(metadata_plan(payload, 11)), "");
    let (first, second) = tokio::join!(
        database.publishing(first_request),
        database.publishing(second_request)
    );
    let first = first.unwrap().metadata_job.unwrap();
    assert_eq!(first.job_id, second.unwrap().metadata_job.unwrap().job_id);
    assert_eq!(database.list_jobs(PROJECT.into()).await.unwrap().len(), 1);
    database
        .cancel_job("cancel-metadata".into(), [9; 32], first.job_id.clone(), 12)
        .await
        .unwrap();
    let found = database
        .publishing(metadata_command(payload, None, ""))
        .await
        .unwrap()
        .metadata_job
        .unwrap();
    assert_eq!(found.state, JobState::Cancelled as i32);
    assert_eq!(found.job_id, first.job_id);
    drop(database);
    actor.shutdown().await.unwrap();
}
fn setup() -> (tempfile::TempDir, Connection) {
    let temp = tempfile::tempdir().unwrap();
    let mut db =
        open_database(&temp.path().join("store.db"), &temp.path().join("backups")).unwrap();
    db.execute(
        "INSERT INTO projects(project_id,name,created_unix_millis) VALUES(?1,'Video',1)",
        [PROJECT],
    )
    .unwrap();
    execute(
        &mut db,
        PublishingCommand::SaveConnection {
            record: YoutubeConnectionV1 {
                connection_id: "conn_test".into(),
                channel_id: "UC_channel".into(),
                state: "connected".into(),
                ..Default::default()
            },
            expected_state: None,
        },
    )
    .unwrap();
    (temp, db)
}
fn record() -> Publication {
    Publication {
        view: YoutubeUploadV1 {
            upload_id: "upl_test".into(),
            project_id: PROJECT.into(),
            channel_id: "UC_channel".into(),
            connection_id: "conn_test".into(),
            doc_id: "edt_test".into(),
            revision: 3,
            ir_artifact_id: format!("sha256:{}", "11".repeat(32)),
            render_artifact_id: format!("sha256:{}", "22".repeat(32)),
            total_bytes: 1000,
            metadata: Some(YoutubeVideoMetadataV1 {
                title: "Approved title".into(),
                ..Default::default()
            }),
            state: "queued".into(),
            created_unix_millis: 1,
            updated_unix_millis: 1,
            ..Default::default()
        },
        sha256: "33".repeat(32),
        generation: 0,
        session_started: false,
        final_possible: false,
        publish_intent: false,
        paused: false,
        intent_epoch: 0,
        reset_session: false,
    }
}
fn create(db: &mut Connection, id: &str) -> PublishingReply {
    execute(
        db,
        PublishingCommand::Create {
            request_id: id.into(),
            request_hash: [1; 32],
            record: record(),
        },
    )
    .unwrap()
}
fn action(db: &mut Connection, id: &str, action: &str) -> Publication {
    execute(
        db,
        PublishingCommand::Action {
            request_id: id.into(),
            request_hash: [2; 32],
            id: "upl_test".into(),
            action: action.into(),
            now: 3,
        },
    )
    .unwrap()
    .uploads
    .remove(0)
}
fn claim(db: &mut Connection) -> Publication {
    execute(
        db,
        PublishingCommand::Claim {
            id: "upl_test".into(),
            now: 2,
        },
    )
    .unwrap()
    .uploads
    .remove(0)
}
#[test]
fn creation_replays_exact_receipt_and_deduplicates_across_new_request_ids() {
    let (_temp, mut db) = setup();
    let first = create(&mut db, "first");
    assert_eq!(first.receipt, create(&mut db, "first").receipt);
    assert_eq!(
        create(&mut db, "another").uploads[0].view.upload_id,
        "upl_test"
    );
    assert_eq!(uploads(&db, "").unwrap().len(), 1);
    let mut changed = record();
    changed.view.metadata.as_mut().unwrap().title = "Changed approval".into();
    assert!(matches!(
        execute(
            &mut db,
            PublishingCommand::Create {
                request_id: "changed".into(),
                request_hash: [3; 32],
                record: changed
            }
        ),
        Err(StoreError::PublishingConflict(_))
    ));
}
#[test]
fn pause_preserves_late_remote_receipt_and_requires_receipt_for_completion() {
    let (_temp, mut db) = setup();
    create(&mut db, "create");
    let mut active = claim(&mut db);
    action(&mut db, "pause", "pause");
    active.view.acknowledged_bytes = 1000;
    active.view.state = "uploading".into();
    let result = execute(
        &mut db,
        PublishingCommand::Checkpoint {
            record: active.clone(),
        },
    )
    .unwrap();
    assert_eq!(
        result.uploads[0].view.state, "paused",
        "all bytes acknowledged is not a receipt"
    );
    active.view.state = "private".into();
    assert!(
        execute(
            &mut db,
            PublishingCommand::Checkpoint {
                record: active.clone()
            }
        )
        .is_err()
    );
    active.view.video_id = "abcdefghijk".into();
    active.view.visibility = "private".into();
    let result = execute(&mut db, PublishingCommand::Checkpoint { record: active }).unwrap();
    assert_eq!(result.uploads[0].view.state, "private");
    assert_eq!(result.uploads[0].view.video_id, "abcdefghijk");
}
#[test]
fn restart_keeps_session_final_send_intent_roots_and_history_after_project_deletion() {
    let (temp, mut db) = setup();
    create(&mut db, "create");
    let mut active = claim(&mut db);
    active.session_started = true;
    active.final_possible = true;
    active.view.state = "uploading".into();
    execute(&mut db, PublishingCommand::Checkpoint { record: active }).unwrap();
    db.execute("DELETE FROM projects WHERE project_id=?1", [PROJECT])
        .unwrap();
    assert_eq!(list_artifact_roots(&db).unwrap().len(), 2);
    drop(db);
    let mut db =
        open_database(&temp.path().join("store.db"), &temp.path().join("backups")).unwrap();
    execute(&mut db, PublishingCommand::Recover { now: 4 }).unwrap();
    let saved = read(&db, "upl_test").unwrap();
    assert_eq!(saved.view.state, "paused");
    assert!(saved.final_possible && saved.session_started);
    let resumed = action(&mut db, "resume", "resume");
    assert_eq!(resumed.view.state, "reconciling");
    assert_eq!(list_artifact_roots(&db).unwrap().len(), 2);
}
#[test]
fn publication_targets_only_an_existing_remote_receipt() {
    let (_temp, mut db) = setup();
    create(&mut db, "create");
    assert!(
        execute(
            &mut db,
            PublishingCommand::Action {
                request_id: "publish".into(),
                request_hash: [1; 32],
                id: "upl_test".into(),
                action: "publish".into(),
                now: 1
            }
        )
        .is_err()
    );
    let mut active = claim(&mut db);
    active.view.video_id = "abcdefghijk".into();
    active.view.state = "private".into();
    active.view.visibility = "private".into();
    execute(&mut db, PublishingCommand::Checkpoint { record: active }).unwrap();
    let publishing = action(&mut db, "publish-ok", "publish");
    assert!(publishing.publish_intent);
    assert_eq!(publishing.view.state, "publishing");
    assert_eq!(
        action(&mut db, "publish-ok", "publish").view.video_id,
        "abcdefghijk"
    );
}
#[test]
fn schema_thirteen_migration_preserves_import_rows_and_projects() {
    let (temp, db) = setup();
    db.execute_batch("DROP TABLE youtube_upload_roots; DROP TABLE youtube_uploads; DROP TABLE youtube_connections; PRAGMA user_version=13;").unwrap();
    drop(db);
    let db = open_database(&temp.path().join("store.db"), &temp.path().join("backups")).unwrap();
    assert_eq!(
        db.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        14
    );
    assert_eq!(
        db.query_row(
            "SELECT count(*) FROM projects WHERE project_id=?1",
            [PROJECT],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    assert!(uploads(&db, "").unwrap().is_empty());
}

#[test]
fn reconnect_and_new_resume_intent_survive_a_stale_worker_error() {
    let (_temp, mut db) = setup();
    create(&mut db, "create");
    let mut old = claim(&mut db);
    action(&mut db, "pause", "pause");
    for (connection_id, state) in [("conn_test", "disconnected"), ("replacement", "connected")] {
        execute(
            &mut db,
            PublishingCommand::SaveConnection {
                record: YoutubeConnectionV1 {
                    connection_id: connection_id.into(),
                    channel_id: "UC_channel".into(),
                    state: state.into(),
                    ..Default::default()
                },
                expected_state: None,
            },
        )
        .unwrap();
    }
    let resumed = action(&mut db, "resume", "resume");
    assert_eq!(resumed.view.connection_id, "replacement");
    old.view.state = "failed".into();
    old.view.error = "old request failed".into();
    let saved = execute(&mut db, PublishingCommand::Checkpoint { record: old })
        .unwrap()
        .uploads
        .remove(0);
    assert_eq!(saved.view.connection_id, "replacement");
    assert_eq!(saved.view.state, "queued");
    assert!(!saved.paused);
    assert_eq!(saved.intent_epoch, resumed.intent_epoch);
}
#[test]
fn explicit_publish_intent_survives_a_late_private_receipt_checkpoint() {
    let (_temp, mut db) = setup();
    create(&mut db, "create");
    let mut old = claim(&mut db);
    old.view.state = "private".into();
    old.view.video_id = "abcdefghijk".into();
    execute(
        &mut db,
        PublishingCommand::Checkpoint {
            record: old.clone(),
        },
    )
    .unwrap();
    action(&mut db, "publish", "publish");
    let saved = execute(&mut db, PublishingCommand::Checkpoint { record: old })
        .unwrap()
        .uploads
        .remove(0);
    assert_eq!(saved.view.state, "publishing");
    assert!(saved.publish_intent);
    assert!(list_artifact_roots(&db).unwrap().is_empty());
}

#[test]
fn expired_incomplete_session_requires_explicit_retry_and_never_resets_final_ambiguity() {
    let (temp, mut db) = setup();
    create(&mut db, "create");
    let mut active = claim(&mut db);
    active.session_started = true;
    active.view.acknowledged_bytes = 200;
    active.view.state = "failed".into();
    active.view.error_code = "session_expired".into();
    execute(&mut db, PublishingCommand::Checkpoint { record: active }).unwrap();
    assert!(!read(&db, "upl_test").unwrap().reset_session);
    let resumed = action(&mut db, "resume", "resume");
    assert!(resumed.reset_session);
    drop(db);
    let mut db =
        open_database(&temp.path().join("store.db"), &temp.path().join("backups")).unwrap();
    execute(&mut db, PublishingCommand::Recover { now: 5 }).unwrap();
    assert!(
        read(&db, "upl_test").unwrap().reset_session,
        "crash after Keychain deletion retains restart intent"
    );
    action(&mut db, "resume-after-crash", "resume");
    let active = claim(&mut db);
    let reset = execute(
        &mut db,
        PublishingCommand::ResetExpiredSession {
            id: "upl_test".into(),
            generation: active.generation,
        },
    )
    .unwrap()
    .uploads
    .remove(0);
    assert!(!reset.session_started && !reset.reset_session);
    assert_eq!(reset.view.acknowledged_bytes, 0);
    let mut final_send = reset;
    final_send.final_possible = true;
    final_send.session_started = true;
    final_send.view.state = "failed".into();
    final_send.view.error_code = "session_expired".into();
    execute(
        &mut db,
        PublishingCommand::Checkpoint { record: final_send },
    )
    .unwrap();
    let retry = action(&mut db, "unsafe-retry", "resume");
    assert!(!retry.reset_session);
    assert!(
        execute(
            &mut db,
            PublishingCommand::ResetExpiredSession {
                id: "upl_test".into(),
                generation: retry.generation
            }
        )
        .is_err()
    );
    assert!(read(&db, "upl_test").unwrap().final_possible);
}
