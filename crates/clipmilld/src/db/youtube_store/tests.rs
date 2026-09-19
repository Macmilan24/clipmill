#![allow(clippy::unwrap_used)]
use super::*;
use crate::{db::open_database, sources::FileObservation};
use tempfile::TempDir;

const PROJECT: &str = "prj_01ARZ3NDEKTSV4RRFFQ69G5FAV";
const SOURCE: &str = "src_01ARZ3NDEKTSV4RRFFQ69G5FAV";
fn setup() -> (TempDir, Connection) {
    let temp = TempDir::new().unwrap();
    let connection =
        open_database(&temp.path().join("store.db"), &temp.path().join("backups")).unwrap();
    connection
        .execute(
            "INSERT INTO projects(project_id,name,created_unix_millis) VALUES(?1,?2,1)",
            params![PROJECT, "YouTube · NYFGCESmikA"],
        )
        .unwrap();
    (temp, connection)
}
fn record() -> YoutubeImportV1 {
    YoutubeImportV1 {
        import_id: "yt_test".into(),
        project_id: PROJECT.into(),
        video_id: "NYFGCESmikA".into(),
        canonical_url: "https://www.youtube.com/watch?v=NYFGCESmikA".into(),
        state: "queued".into(),
        created_unix_millis: 1,
        updated_unix_millis: 1,
        ..Default::default()
    }
}
fn create(
    connection: &mut Connection,
    request_id: &str,
    hash: u8,
) -> Result<Vec<YoutubeImportV1>, StoreError> {
    execute(
        connection,
        YoutubeCommand::Create {
            request_id: request_id.into(),
            request_hash: [hash; 32],
            record: record(),
        },
    )
}
fn action(
    connection: &mut Connection,
    name: &str,
    request_id: &str,
) -> Result<Vec<YoutubeImportV1>, StoreError> {
    execute(
        connection,
        YoutubeCommand::Update {
            request_id: request_id.into(),
            request_hash: [3; 32],
            id: "yt_test".into(),
            action: name.into(),
            now: 2,
        },
    )
}
fn claim(connection: &mut Connection, attempt: u32) -> YoutubeImportV1 {
    execute(
        connection,
        YoutubeCommand::Claim {
            id: "yt_test".into(),
            attempt,
            now: 2,
        },
    )
    .unwrap()
    .remove(0)
}
fn inspection() -> InspectedSource {
    InspectedSource {
        source_fingerprint: format!("sha256:{}", "a".repeat(64)),
        source_map_json: b"{}".to_vec(),
        observation: FileObservation {
            absolute_path: "/managed/source.mkv".into(),
            byte_size: 100,
            sample_sha256: format!("sha256:{}", "b".repeat(64)),
            device_id: 1,
            inode: 1,
            modified_unix_nanos: 1,
        },
    }
}
fn complete(connection: &mut Connection, attempt: u32) -> Result<Vec<YoutubeImportV1>, StoreError> {
    execute(
        connection,
        YoutubeCommand::Complete {
            id: "yt_test".into(),
            attempt,
            source_id: SOURCE.into(),
            inspection: Box::new(inspection()),
            now: 3,
        },
    )
}

#[test]
fn repeated_submit_is_one_import_and_request_reuse_is_checked() {
    let (_temp, mut db) = setup();
    let first = create(&mut db, "first", 1).unwrap();
    assert_eq!(create(&mut db, "first", 1).unwrap(), first);
    assert_eq!(create(&mut db, "different-request", 2).unwrap(), first);
    assert!(matches!(
        create(&mut db, "first", 2),
        Err(StoreError::Conflict)
    ));
    assert_eq!(list(&db, "").unwrap().len(), 1);
}

#[test]
fn cancel_retry_and_restart_fence_all_old_callbacks() {
    let (_temp, mut db) = setup();
    create(&mut db, "start", 1).unwrap();
    let old = claim(&mut db, 0);
    action(&mut db, "cancel", "cancel").unwrap();
    assert!(
        execute(
            &mut db,
            YoutubeCommand::Progress {
                record: old.clone()
            }
        )
        .is_err()
    );
    assert!(complete(&mut db, 0).is_err());
    let retried = action(&mut db, "retry", "retry").unwrap();
    assert_eq!(retried[0].attempt, 1);
    assert_eq!(action(&mut db, "retry", "retry").unwrap()[0].attempt, 1);
    claim(&mut db, 1);
    assert!(execute(&mut db, YoutubeCommand::Progress { record: old }).is_err());
    execute(&mut db, YoutubeCommand::Recover { now: 4 }).unwrap();
    assert_eq!(read(&db, "yt_test").unwrap().state, "interrupted");
    assert!(complete(&mut db, 1).is_err());
    assert_eq!(
        db.query_row("SELECT count(*) FROM sources", [], |row| row
            .get::<_, u32>(0))
            .unwrap(),
        0
    );
}

#[test]
fn registration_and_import_link_commit_together_and_survive_restart() {
    let (temp, mut db) = setup();
    create(&mut db, "start", 1).unwrap();
    let mut value = claim(&mut db, 0);
    value.state = "registering".into();
    value.title = "The real video title".into();
    execute(&mut db, YoutubeCommand::Progress { record: value }).unwrap();
    complete(&mut db, 0).unwrap();
    assert!(complete(&mut db, 0).is_err());
    drop(db);
    let mut db =
        open_database(&temp.path().join("store.db"), &temp.path().join("backups")).unwrap();
    execute(&mut db, YoutubeCommand::Recover { now: 4 }).unwrap();
    let saved = read(&db, "yt_test").unwrap();
    assert_eq!(
        (saved.state.as_str(), saved.source_id.as_str()),
        ("completed", SOURCE)
    );
    assert_eq!(
        db.query_row("SELECT count(*) FROM sources", [], |row| row
            .get::<_, u32>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        db.query_row(
            "SELECT name FROM projects WHERE project_id=?1",
            [PROJECT],
            |row| row.get::<_, String>(0)
        )
        .unwrap(),
        "The real video title"
    );
}

#[test]
fn invalid_source_rolls_back_without_completing_or_overwriting_a_named_project() {
    let (_temp, mut db) = setup();
    db.execute(
        "UPDATE projects SET name='Chosen by user' WHERE project_id=?1",
        [PROJECT],
    )
    .unwrap();
    create(&mut db, "start", 1).unwrap();
    let mut value = claim(&mut db, 0);
    value.state = "registering".into();
    value.title = "A different video title".into();
    execute(&mut db, YoutubeCommand::Progress { record: value }).unwrap();
    let mut invalid = inspection();
    invalid.observation.sample_sha256 = "bad".into();
    assert!(
        execute(
            &mut db,
            YoutubeCommand::Complete {
                id: "yt_test".into(),
                attempt: 0,
                source_id: SOURCE.into(),
                inspection: Box::new(invalid),
                now: 3
            }
        )
        .is_err()
    );
    assert_eq!(read(&db, "yt_test").unwrap().state, "registering");
    assert_eq!(
        db.query_row("SELECT count(*) FROM sources", [], |row| row
            .get::<_, u32>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        db.query_row(
            "SELECT name FROM projects WHERE project_id=?1",
            [PROJECT],
            |row| row.get::<_, String>(0)
        )
        .unwrap(),
        "Chosen by user"
    );
}

#[test]
fn version_twelve_migrates_without_losing_projects() {
    let (temp, db) = setup();
    db.execute_batch("DROP TABLE youtube_imports; PRAGMA user_version=12;")
        .unwrap();
    drop(db);
    let db = open_database(&temp.path().join("store.db"), &temp.path().join("backups")).unwrap();
    assert_eq!(
        db.query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))
            .unwrap(),
        13
    );
    assert_eq!(
        db.query_row(
            "SELECT count(*) FROM projects WHERE project_id=?1",
            [PROJECT],
            |row| row.get::<_, u32>(0)
        )
        .unwrap(),
        1
    );
    assert!(list(&db, "").unwrap().is_empty());
}

#[test]
fn quality_is_saved_through_retry_and_a_different_quality_is_not_deduplicated() {
    let (_temp, mut db) = setup();
    let mut value = record();
    value.max_height = 360;
    execute(
        &mut db,
        YoutubeCommand::Create {
            request_id: "low".into(),
            request_hash: [1; 32],
            record: value.clone(),
        },
    )
    .unwrap();
    assert_eq!(claim(&mut db, 0).max_height, 360);
    action(&mut db, "cancel", "cancel").unwrap();
    let retried = action(&mut db, "retry", "retry").unwrap();
    assert_eq!(retried[0].max_height, 360);
    assert_eq!(read(&db, "yt_test").unwrap().max_height, 360);
    value.max_height = 720;
    assert!(matches!(
        execute(
            &mut db,
            YoutubeCommand::Create {
                request_id: "higher".into(),
                request_hash: [2; 32],
                record: value,
            }
        ),
        Err(StoreError::ImportQualityConflict)
    ));
    assert_eq!(list(&db, PROJECT).unwrap().len(), 1);
}

#[test]
fn legacy_record_without_quality_reads_as_1080_and_matches_default_requests() {
    let (_temp, mut db) = setup();
    create(&mut db, "legacy", 1).unwrap();
    // This fixture's absent protobuf field models imports saved before quality.
    assert_eq!(record().max_height, 0);
    db.execute(
        "UPDATE youtube_imports SET record=?1 WHERE import_id='yt_test'",
        [record().encode_to_vec()],
    )
    .unwrap();
    assert_eq!(read(&db, "yt_test").unwrap().max_height, 1080);
    assert_eq!(create(&mut db, "repeat", 2).unwrap()[0].max_height, 1080);
    let mut value = record();
    value.max_height = 1080;
    assert_eq!(
        execute(
            &mut db,
            YoutubeCommand::Create {
                request_id: "explicit".into(),
                request_hash: [4; 32],
                record: value,
            }
        )
        .unwrap()[0]
            .max_height,
        1080
    );
}
