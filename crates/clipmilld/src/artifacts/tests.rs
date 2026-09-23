#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::{
    collections::BTreeMap,
    io::Write,
    path::PathBuf,
    time::{Duration, Instant, SystemTime},
};

use clipmill_artifacts::{
    ArtifactRecipe, NetworkPolicy, PrepareOutcome, Producer, RecipeSpec, Timebase,
};
use clipmill_core::{ArtifactId, ProjectId, Sha256Digest};
use serde_json::Map;
use tempfile::TempDir;
use tokio::sync::oneshot;

use super::{ArtifactActor, ArtifactHandle, COMMAND_CAPACITY, Command, GcPause};
use crate::{
    collector::{Collector, Freed},
    db::{DbActor, DbHandle, ProjectRecord},
};

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
    let (_collector, requests) = Collector::new();
    let (stop, stopped) = oneshot::channel();
    let maintenance = tokio::spawn(crate::collector::run(
        handle.clone(),
        db.clone(),
        Duration::ZERO,
        requests,
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
    let (_collector, requests) = Collector::new();
    let (stop, stopped) = oneshot::channel();
    let maintenance = tokio::spawn(crate::collector::run(
        handle.clone(),
        db,
        Duration::ZERO,
        requests,
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

/// A retention period no fixture outlives, so only a clean-up's own grace
/// frees anything.
const RETENTION: Duration = Duration::from_hours(168);

#[tokio::test]
async fn a_clean_up_is_answered_with_what_its_pass_freed() {
    let temp = TempDir::new().unwrap();
    let (actor, _) = ArtifactActor::start(&temp.path().join("artifacts")).unwrap();
    let handle = actor.handle();
    let (database, db, project) = database(&temp).await;
    let kept = publish(&handle, 8).await;
    let orphan = publish(&handle, 9).await;
    db.attach_artifact_root(project, kept).await.unwrap();
    let (collector, requests) = Collector::new();
    let (stop, stopped) = oneshot::channel();
    let collecting = tokio::spawn(crate::collector::run(
        handle.clone(),
        db.clone(),
        RETENTION,
        requests,
        stopped,
    ));

    let freed = tokio::time::timeout(Duration::from_secs(5), collector.clean_up(Duration::ZERO))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        freed,
        Freed {
            objects: 1,
            bytes: 1024 * 1024
        }
    );
    assert!(handle.open(orphan).await.is_err());
    assert!(handle.open(kept).await.is_ok(), "a rooted object stays");

    stop.send(()).unwrap();
    collecting.await.unwrap();
    actor.shutdown().await.unwrap();
    database.shutdown().await.unwrap();
}

/// The bug a person saw as "success" that freed nothing: foreground work
/// deferred the clean-up's pass, and the deferred pass was the answer.
#[tokio::test]
async fn a_clean_up_deferred_by_foreground_work_finishes_before_it_answers() {
    let temp = TempDir::new().unwrap();
    let (actor, paused, resume) = paused_actor(&temp);
    let handle = actor.handle();
    let (database, db, _) = database(&temp).await;
    let read = publish(&handle, 10).await;
    let orphan = publish(&handle, 11).await;
    let (collector, requests) = Collector::new();
    let asked = tokio::spawn({
        let collector = collector.clone();
        async move { collector.clean_up(Duration::ZERO).await }
    });
    // Asked before the loop starts, so its first attempt is the clean-up's.
    while requests.is_empty() {
        tokio::task::yield_now().await;
    }
    let (stop, stopped) = oneshot::channel();
    let collecting = tokio::spawn(crate::collector::run(
        handle.clone(),
        db.clone(),
        RETENTION,
        requests,
        stopped,
    ));
    tokio::time::timeout(Duration::from_secs(2), paused)
        .await
        .unwrap()
        .unwrap();
    let (reply, opened) = oneshot::channel();
    handle
        .sender
        .send(Command::Open {
            artifact_id: read,
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

    let freed = tokio::time::timeout(Duration::from_secs(5), asked)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(freed.objects, 1, "the retry finished the clean-up");
    assert!(handle.open(orphan).await.is_err());
    assert!(
        handle.open(read).await.is_ok(),
        "the retry saw the new reader"
    );

    drop(lease);
    stop.send(()).unwrap();
    collecting.await.unwrap();
    actor.shutdown().await.unwrap();
    database.shutdown().await.unwrap();
}

#[tokio::test]
async fn a_clean_up_asked_for_during_a_scheduled_pass_takes_it_over() {
    let temp = TempDir::new().unwrap();
    let (actor, paused, resume) = paused_actor(&temp);
    let handle = actor.handle();
    let (database, db, _) = database(&temp).await;
    let orphan = publish(&handle, 12).await;
    let (collector, requests) = Collector::new();
    let (stop, stopped) = oneshot::channel();
    let collecting = tokio::spawn(crate::collector::run(
        handle.clone(),
        db.clone(),
        RETENTION,
        requests,
        stopped,
    ));
    // The startup pass, with the retention period, is mid-scan.
    tokio::time::timeout(Duration::from_secs(2), paused)
        .await
        .unwrap()
        .unwrap();
    let asked = tokio::spawn({
        let collector = collector.clone();
        async move { collector.clean_up(Duration::ZERO).await }
    });
    // The clean-up's own attempt is queued behind the paused one, which must
    // now yield to it rather than finish a pass that would free nothing.
    tokio::time::timeout(Duration::from_secs(2), async {
        while handle.sender.capacity() == COMMAND_CAPACITY {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("the clean-up starts while the scheduled pass is still running");
    resume.send(()).unwrap();

    let freed = tokio::time::timeout(Duration::from_secs(5), asked)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(freed.objects, 1);
    assert!(handle.open(orphan).await.is_err());

    stop.send(()).unwrap();
    collecting.await.unwrap();
    actor.shutdown().await.unwrap();
    database.shutdown().await.unwrap();
}

#[tokio::test]
async fn a_clean_up_asked_for_after_the_collector_stopped_says_so() {
    let temp = TempDir::new().unwrap();
    let (actor, _) = ArtifactActor::start(&temp.path().join("artifacts")).unwrap();
    let (database, db, _) = database(&temp).await;
    let (collector, requests) = Collector::new();
    let (stop, stopped) = oneshot::channel();
    let collecting = tokio::spawn(crate::collector::run(
        actor.handle(),
        db,
        RETENTION,
        requests,
        stopped,
    ));
    stop.send(()).unwrap();
    collecting.await.unwrap();
    assert_eq!(
        collector.clean_up(Duration::ZERO).await,
        Err(crate::collector::CleanUpError::Stopped)
    );
    actor.shutdown().await.unwrap();
    database.shutdown().await.unwrap();
}

/// A clean-up on a real library while something keeps opening objects, as a
/// screen loading thumbnails does, so the pass has to yield and resume many
/// times. By hand, against a disposable copy of a data directory: it deletes
/// what the copy's projects no longer use.
///
/// ```sh
/// COPY=$(mktemp -d)
/// DATA="$HOME/Library/Application Support/dev.clipmill.ClipMill"
/// cp -cRp "$DATA/artifacts" "$COPY/artifacts" && mkdir "$COPY/state"
/// sqlite3 "file:$DATA/state/clipmill.db?mode=ro" ".backup '$COPY/state/clipmill.db'"
/// CLIPMILL_CLEAN_UP_DRILL=$COPY cargo test -p clipmilld --lib \
///     a_clean_up_finishes_on_a_real_library -- --ignored --nocapture
/// ```
#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs a disposable copy of a real data directory"]
async fn a_clean_up_finishes_on_a_real_library_while_objects_are_opened() {
    let data = PathBuf::from(
        std::env::var_os("CLIPMILL_CLEAN_UP_DRILL").expect("CLIPMILL_CLEAN_UP_DRILL names a copy"),
    );
    let grace = Duration::from_mins(15);
    let (actor, _) = ArtifactActor::start(&data.join("artifacts")).unwrap();
    let handle = actor.handle();
    let database =
        DbActor::start(&data.join("state/clipmill.db"), &data.join("state/backups")).unwrap();
    let db = database.handle();
    let roots = db.list_artifact_roots().await.unwrap();
    let stored = handle.usage().await.unwrap();
    let estimate = handle
        .reclaimable(roots.clone(), SystemTime::now(), grace)
        .await
        .unwrap();

    let reader = tokio::spawn({
        let handle = handle.clone();
        async move {
            let mut opened = 0_u64;
            for root in roots.iter().cycle() {
                drop(handle.open(*root).await);
                opened += 1;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            opened
        }
    });
    let (collector, requests) = Collector::new();
    let (stop, stopped) = oneshot::channel();
    let collecting = tokio::spawn(crate::collector::run(
        handle.clone(),
        db.clone(),
        RETENTION,
        requests,
        stopped,
    ));
    let started = Instant::now();
    let freed = collector.clean_up(grace).await.unwrap();
    let took = started.elapsed();
    reader.abort();
    eprintln!(
        "freed {} objects, {} bytes, in {took:.1?}; estimated {} objects, {} bytes, of {} \
         objects, {} bytes stored",
        freed.objects, freed.bytes, estimate.objects, estimate.bytes, stored.objects, stored.bytes
    );
    assert_eq!(
        (freed.objects, freed.bytes),
        (estimate.objects, estimate.bytes)
    );

    stop.send(()).unwrap();
    collecting.await.unwrap();
    actor.shutdown().await.unwrap();
    database.shutdown().await.unwrap();
}
