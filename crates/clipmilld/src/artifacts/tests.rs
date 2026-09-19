#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::{
    collections::BTreeMap,
    io::Write,
    time::{Duration, SystemTime},
};

use clipmill_artifacts::{
    ArtifactRecipe, NetworkPolicy, PrepareOutcome, Producer, RecipeSpec, Timebase,
};
use clipmill_core::{ArtifactId, ProjectId, Sha256Digest};
use serde_json::Map;
use tempfile::TempDir;
use tokio::sync::oneshot;

use super::{ArtifactActor, ArtifactHandle, Command, GcPause};
use crate::db::{DbActor, DbHandle, ProjectRecord};

async fn database(temp: &TempDir) -> (DbActor, DbHandle, ProjectId) {
    let database =
        DbActor::start(&temp.path().join("state.db"), &temp.path().join("backups")).unwrap();
    let db = database.handle();
    let project = ProjectId::new();
    db.create_project(
        "gc-project".to_owned(),
        [0; 32],
        ProjectRecord {
            project_id: project.to_string(),
            name: "GC fixture".to_owned(),
            created_unix_millis: 1,
        },
    )
    .await
    .unwrap();
    (database, db, project)
}

fn paused_actor(
    temp: &TempDir,
) -> (
    ArtifactActor,
    oneshot::Receiver<()>,
    std::sync::mpsc::Sender<()>,
) {
    let (reached, paused) = oneshot::channel();
    let (resume, resumed) = std::sync::mpsc::channel();
    let (actor, _) = ArtifactActor::start_inner(
        &temp.path().join("artifacts"),
        Some(GcPause {
            reached,
            resume: resumed,
        }),
    )
    .unwrap();
    (actor, paused, resume)
}

async fn publish(handle: &ArtifactHandle, id: u8) -> ArtifactId {
    let recipe = ArtifactRecipe::try_from_spec(RecipeSpec {
        kind: "evidence.probe.v1".to_owned(),
        source_fingerprint: Sha256Digest::from_bytes([id; 32]),
        timebase: Timebase {
            num: 1,
            den: 90_000,
        },
        producer: Producer {
            stage: "gc-test".to_owned(),
            implementation: "fixture@1".to_owned(),
            model_digest: None,
        },
        inputs: Vec::new(),
        policy: NetworkPolicy::LocalLock,
        config: Map::new(),
        semantic_version: "1.0.0".to_owned(),
    })
    .unwrap();
    let PrepareOutcome::Miss(staging) = handle.prepare(recipe).await.unwrap() else {
        panic!("new recipe");
    };
    let path = "payload.bin".parse().unwrap();
    staging
        .create_file(&path)
        .unwrap()
        .write_all(&vec![id; 1024 * 1024])
        .unwrap();
    let lease = handle
        .commit(staging.id().clone(), vec![path], BTreeMap::new())
        .await
        .unwrap();
    lease.artifact_id()
}

#[tokio::test]
async fn foreground_open_interrupts_gc_and_retry_uses_fresh_roots_and_reader_pins() {
    let temp = TempDir::new().unwrap();
    let (actor, paused, resume) = paused_actor(&temp);
    let handle = actor.handle();
    let (database, db, project) = database(&temp).await;
    let pinned = publish(&handle, 1).await;
    let attached = publish(&handle, 2).await;
    let orphan = publish(&handle, 3).await;
    let now = SystemTime::now() + Duration::from_hours(192);
    let grace = Duration::from_hours(168);
    let (reply, collected) = oneshot::channel();
    handle
        .sender
        .send(Command::Collect {
            roots: db.list_artifact_roots().await.unwrap(),
            now,
            grace,
            reply,
        })
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(2), paused)
        .await
        .unwrap()
        .unwrap();
    // Queue the read while verification is paused, so success cannot depend on
    // a race between a tiny fixture and the scheduler.
    let (reply, opened) = oneshot::channel();
    handle
        .sender
        .send(Command::Open {
            artifact_id: pinned,
            reply,
        })
        .await
        .unwrap();
    resume.send(()).unwrap();
    let lease = tokio::time::timeout(Duration::from_secs(2), opened)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let first = collected.await.unwrap().unwrap();
    assert!(first.deferred);
    assert_eq!(first.deleted, 0);
    assert_eq!(handle.usage().await.unwrap().objects, 3);

    // A cache attachment during the quiet interval is a new database root;
    // the old root list must not be reused by a deferred retry.
    db.attach_artifact_root(project, attached).await.unwrap();
    let retry = handle
        .collect(db.list_artifact_roots().await.unwrap(), now, grace)
        .await
        .unwrap();
    assert!(!retry.deferred);
    assert_eq!(
        retry.deleted, 1,
        "only the unrooted, unpinned orphan is removed"
    );
    assert!(handle.open(orphan).await.is_err());
    assert!(handle.open(attached).await.is_ok());
    drop(lease);
    let idle = handle
        .collect(db.list_artifact_roots().await.unwrap(), now, grace)
        .await
        .unwrap();
    assert_eq!(
        idle.deleted, 1,
        "released reader pins do not prevent eventual cleanup"
    );
    assert!(handle.open(pinned).await.is_err());
    assert!(handle.open(attached).await.is_ok());
    actor.shutdown().await.unwrap();
    database.shutdown().await.unwrap();
}

#[tokio::test]
async fn maintenance_retries_a_deferred_pass_with_new_roots_after_foreground_work() {
    let temp = TempDir::new().unwrap();
    let (actor, paused, resume) = paused_actor(&temp);
    let handle = actor.handle();
    let (database, db, project) = database(&temp).await;
    let pinned = publish(&handle, 4).await;
    let attached = publish(&handle, 5).await;
    let orphan = publish(&handle, 6).await;
    let paths = clipmill_artifacts::StorePaths::new(temp.path().join("artifacts"));
    let (stop, stopped) = oneshot::channel();
    let maintenance = tokio::spawn(crate::daemon::run_artifact_maintenance(
        handle.clone(),
        db.clone(),
        Duration::ZERO,
        stopped,
    ));
    tokio::time::timeout(Duration::from_secs(2), paused)
        .await
        .unwrap()
        .unwrap();
    // The running pass has already read the old (empty) root list.
    db.attach_artifact_root(project, attached).await.unwrap();
    let (reply, opened) = oneshot::channel();
    handle
        .sender
        .send(Command::Open {
            artifact_id: pinned,
            reply,
        })
        .await
        .unwrap();
    let (reply, usage) = oneshot::channel();
    handle.sender.send(Command::Usage { reply }).await.unwrap();
    resume.send(()).unwrap();
    let lease = tokio::time::timeout(Duration::from_secs(2), opened)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(usage.await.unwrap().objects, 3);
    // Observe the filesystem instead of polling the actor: this leaves a real
    // quiet interval for the production retry timer to run.
    tokio::time::timeout(Duration::from_secs(5), async {
        while paths.object_dir(orphan).exists() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("maintenance must retry before the six-hour regular interval");
    assert!(
        paths.object_dir(pinned).exists(),
        "retry refreshes reader pins"
    );
    assert!(
        paths.object_dir(attached).exists(),
        "retry refreshes database roots"
    );
    stop.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(2), maintenance)
        .await
        .unwrap()
        .unwrap();
    drop(lease);
    actor.shutdown().await.unwrap();
    database.shutdown().await.unwrap();
}

#[tokio::test]
async fn stopping_maintenance_abandons_queued_verification_without_deleting() {
    let temp = TempDir::new().unwrap();
    let (actor, paused, resume) = paused_actor(&temp);
    let handle = actor.handle();
    let (database, db, _) = database(&temp).await;
    publish(&handle, 7).await;
    let (stop, stopped) = oneshot::channel();
    let maintenance = tokio::spawn(crate::daemon::run_artifact_maintenance(
        handle.clone(),
        db,
        Duration::ZERO,
        stopped,
    ));
    tokio::time::timeout(Duration::from_secs(2), paused)
        .await
        .unwrap()
        .unwrap();
    stop.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(2), maintenance)
        .await
        .unwrap()
        .unwrap();
    resume.send(()).unwrap();
    assert_eq!(handle.usage().await.unwrap().objects, 1);
    actor.shutdown().await.unwrap();
    database.shutdown().await.unwrap();
}
