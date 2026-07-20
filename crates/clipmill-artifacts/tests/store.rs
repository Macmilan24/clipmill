#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    time::{Duration, SystemTime},
};

use clipmill_artifacts::{
    ArtifactError, ArtifactPath, ArtifactRecipe, ArtifactStore, CacheLookup, CacheMissReason,
    NetworkPolicy, PrepareOutcome, Producer, RecipeSpec, Timebase,
};
use clipmill_core::{ArtifactId, Sha256Digest};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

fn recipe_with(
    source: u8,
    inputs: Vec<ArtifactId>,
    model: Option<u8>,
    config: Map<String, Value>,
) -> ArtifactRecipe {
    ArtifactRecipe::try_from_spec(RecipeSpec {
        kind: "evidence.probe.v1".to_owned(),
        source_fingerprint: Sha256Digest::from_bytes([source; 32]),
        timebase: Timebase {
            num: 1,
            den: 90_000,
        },
        producer: Producer {
            stage: "probe".to_owned(),
            implementation: "probe-adapter@1.0.0".to_owned(),
            model_digest: model.map(|byte| Sha256Digest::from_bytes([byte; 32])),
        },
        inputs,
        policy: NetworkPolicy::LocalLock,
        config,
        semantic_version: "1.0.0".to_owned(),
    })
    .expect("valid recipe")
}

fn recipe(source: u8) -> ArtifactRecipe {
    recipe_with(source, Vec::new(), None, Map::new())
}

fn prepare_miss(
    store: &mut ArtifactStore,
    recipe: ArtifactRecipe,
) -> clipmill_artifacts::StagingArea {
    match store.prepare(recipe).expect("prepare") {
        PrepareOutcome::Miss(staging) => staging,
        other => panic!("expected miss, got {other:?}"),
    }
}

fn commit_payload(
    store: &mut ArtifactStore,
    recipe: ArtifactRecipe,
    path: &str,
    payload: &[u8],
) -> clipmill_artifacts::ArtifactLease {
    let staging = prepare_miss(store, recipe);
    let artifact_path = path.parse::<ArtifactPath>().expect("artifact path");
    {
        let mut file = staging
            .create_file(&artifact_path)
            .expect("create staging file");
        file.write_all(payload).expect("write payload");
        file.sync_all().expect("sync payload");
    }
    store
        .commit(staging.id(), vec![artifact_path], BTreeMap::new())
        .expect("commit")
}

#[test]
fn deterministic_roundtrip_survives_reopen_and_response_loss() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path().join("artifacts");
    let recipe = recipe(1);
    let expected = recipe.artifact_id().expect("id");
    let (mut store, recovery) = ArtifactStore::initialize(&root).expect("store");
    assert_eq!(recovery.committed_loaded, 0);

    let lease = commit_payload(&mut store, recipe.clone(), "probe.json", b"deterministic");
    assert_eq!(lease.artifact_id(), expected);
    let mut verified = lease
        .open_verified(&"probe.json".parse().expect("path"))
        .expect("verified read");
    let mut bytes = Vec::new();
    verified.read_to_end(&mut bytes).expect("read");
    assert_eq!(bytes, b"deterministic");
    drop(verified);
    drop(lease);

    match store.prepare(recipe.clone()).expect("lost-response retry") {
        PrepareOutcome::Hit(hit) => assert_eq!(hit.artifact_id(), expected),
        PrepareOutcome::Miss(miss) => panic!("expected hit, got miss {miss:?}"),
        PrepareOutcome::InFlight { artifact_id } => {
            panic!("expected hit, got in-flight {artifact_id}");
        }
    }
    drop(store);

    let (reopened, recovery) = ArtifactStore::initialize(&root).expect("reopen");
    assert_eq!(recovery.committed_loaded, 1);
    match reopened.lookup(&recipe).expect("lookup") {
        CacheLookup::Hit(hit) => assert_eq!(hit.artifact_id(), expected),
        miss @ CacheLookup::Miss(_) => panic!("expected hit, got {miss:?}"),
    }
}

#[test]
fn one_writer_per_key_and_model_changes_create_sibling_lineages() {
    let temp = TempDir::new().expect("tempdir");
    let (mut store, _) = ArtifactStore::initialize(temp.path()).expect("store");
    let original = recipe_with(1, Vec::new(), Some(7), Map::new());
    let staging = prepare_miss(&mut store, original.clone());
    match store.prepare(original.clone()).expect("second prepare") {
        PrepareOutcome::InFlight { artifact_id } => {
            assert_eq!(artifact_id, original.artifact_id().expect("id"));
        }
        other => panic!("expected in-flight, got {other:?}"),
    }
    drop(staging);

    let sibling = recipe_with(1, Vec::new(), Some(8), Map::new());
    assert_ne!(
        original.artifact_id().expect("original"),
        sibling.artifact_id().expect("sibling")
    );
    assert!(matches!(
        store.lookup(&sibling).expect("lookup"),
        CacheLookup::Miss(CacheMissReason::NotPresent { .. })
    ));
}

#[test]
fn abandoning_worker_staging_revokes_the_token_and_quarantines_bytes() {
    let temp = TempDir::new().expect("tempdir");
    let (mut store, _) = ArtifactStore::initialize(temp.path()).expect("store");
    let recipe = recipe(41);
    let staging = prepare_miss(&mut store, recipe.clone());
    let staging_id = staging.id().clone();
    let staging_path = staging.path().to_path_buf();
    let path = "partial.bin".parse::<ArtifactPath>().expect("path");
    let mut file = staging.create_file(&path).expect("partial output");
    file.write_all(b"unacknowledged").expect("write partial");
    file.sync_all().expect("sync partial");
    drop(file);

    assert!(store.abandon(&staging_id).expect("abandon token"));
    assert!(!staging_path.exists());
    assert!(!store.abandon(&staging_id).expect("repeat abandon"));
    assert!(
        fs::read_dir(temp.path().join("quarantine"))
            .expect("quarantine")
            .any(|entry| entry
                .expect("quarantine entry")
                .file_name()
                .to_string_lossy()
                .starts_with("abandoned-"))
    );
    assert!(matches!(
        store.prepare(recipe).expect("prepare after abandon"),
        PrepareOutcome::Miss(_)
    ));
}

#[test]
fn concurrent_stores_reject_nondeterministic_output_for_one_key() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path().join("artifacts");
    let (mut first, _) = ArtifactStore::initialize(&root).expect("first");
    let (mut second, _) = ArtifactStore::initialize(&root).expect("second");
    let recipe = recipe(2);
    let first_stage = prepare_miss(&mut first, recipe.clone());
    let second_stage = prepare_miss(&mut second, recipe);
    let path = "result.bin".parse::<ArtifactPath>().expect("path");
    first_stage
        .create_file(&path)
        .expect("first file")
        .write_all(b"first")
        .expect("first write");
    second_stage
        .create_file(&path)
        .expect("second file")
        .write_all(b"second")
        .expect("second write");
    drop(
        first
            .commit(first_stage.id(), vec![path.clone()], BTreeMap::new())
            .expect("first commit"),
    );
    assert!(matches!(
        second.commit(second_stage.id(), vec![path], BTreeMap::new()),
        Err(ArtifactError::NonDeterministicOutput(_))
    ));
    assert_eq!(
        fs::read_dir(second.paths().quarantine.clone())
            .expect("quarantine")
            .count(),
        1
    );
}

#[test]
fn differing_quality_for_one_key_is_nondeterministic() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path().join("artifacts");
    let (mut first, _) = ArtifactStore::initialize(&root).expect("first");
    let (mut second, _) = ArtifactStore::initialize(&root).expect("second");
    let recipe = recipe(20);
    let first_stage = prepare_miss(&mut first, recipe.clone());
    let second_stage = prepare_miss(&mut second, recipe);
    let path = "result.bin".parse::<ArtifactPath>().expect("path");
    for staging in [&first_stage, &second_stage] {
        staging
            .create_file(&path)
            .expect("staged file")
            .write_all(b"identical")
            .expect("staged write");
    }
    drop(
        first
            .commit(first_stage.id(), vec![path.clone()], BTreeMap::new())
            .expect("first commit"),
    );
    let quality = BTreeMap::from([("confidence".to_owned(), 0.9)]);
    assert!(matches!(
        second.commit(second_stage.id(), vec![path], quality),
        Err(ArtifactError::NonDeterministicOutput(_))
    ));
}

#[test]
fn exact_declared_files_are_enforced_and_failures_are_quarantined() {
    let temp = TempDir::new().expect("tempdir");
    let (mut store, _) = ArtifactStore::initialize(temp.path()).expect("store");
    let stage = prepare_miss(&mut store, recipe(3));
    let first = "first.bin".parse::<ArtifactPath>().expect("first");
    let second = "nested/second.bin".parse::<ArtifactPath>().expect("second");
    stage
        .create_file(&first)
        .expect("first file")
        .write_all(b"one")
        .expect("write first");
    stage
        .create_file(&second)
        .expect("second file")
        .write_all(b"two")
        .expect("write second");
    assert!(matches!(
        store.commit(stage.id(), vec![first], BTreeMap::new()),
        Err(ArtifactError::DeclaredFileSetMismatch)
    ));
    assert_eq!(
        fs::read_dir(&store.paths().staging)
            .expect("staging")
            .count(),
        0
    );
    assert_eq!(
        fs::read_dir(&store.paths().quarantine)
            .expect("quarantine")
            .count(),
        1
    );

    let missing_stage = prepare_miss(&mut store, recipe(17));
    let actual = "actual.bin".parse::<ArtifactPath>().expect("actual");
    missing_stage
        .create_file(&actual)
        .expect("actual file")
        .write_all(b"actual")
        .expect("actual write");
    assert!(matches!(
        store.commit(
            missing_stage.id(),
            vec![actual, "missing.bin".parse().expect("missing path")],
            BTreeMap::new()
        ),
        Err(ArtifactError::DeclaredFileSetMismatch)
    ));

    let duplicate_stage = prepare_miss(&mut store, recipe(18));
    let duplicate = "duplicate.bin"
        .parse::<ArtifactPath>()
        .expect("duplicate path");
    duplicate_stage
        .create_file(&duplicate)
        .expect("duplicate file")
        .write_all(b"duplicate")
        .expect("duplicate write");
    assert!(matches!(
        store.commit(
            duplicate_stage.id(),
            vec![duplicate.clone(), duplicate],
            BTreeMap::new()
        ),
        Err(ArtifactError::DuplicateFilePath)
    ));
    assert_eq!(
        fs::read_dir(&store.paths().quarantine)
            .expect("all failed stages quarantined")
            .count(),
        3
    );
}

#[test]
fn payload_tampering_is_detected_on_verified_read() {
    let temp = TempDir::new().expect("tempdir");
    let (mut store, _) = ArtifactStore::initialize(temp.path()).expect("store");
    let recipe = recipe(4);
    let id = recipe.artifact_id().expect("id");
    let lease = commit_payload(&mut store, recipe, "payload.bin", b"original");
    drop(lease);
    let payload = store.paths().object_dir(id).join("payload.bin");
    make_writable(&payload);
    fs::write(&payload, b"tampered").expect("tamper same-size payload");
    let lease = store.open(id).expect("manifest remains readable");
    assert!(matches!(
        lease.open_verified(&"payload.bin".parse().expect("path")),
        Err(ArtifactError::PayloadHashMismatch)
    ));
}

#[test]
fn restart_quarantines_staging_and_structurally_corrupt_objects() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path().join("artifacts");
    let (mut store, _) = ArtifactStore::initialize(&root).expect("store");
    let stale = prepare_miss(&mut store, recipe(5));
    stale
        .create_file(&"partial.bin".parse().expect("path"))
        .expect("partial")
        .write_all(b"partial")
        .expect("partial write");

    let committed_recipe = recipe(6);
    let committed_id = committed_recipe.artifact_id().expect("id");
    drop(commit_payload(
        &mut store,
        committed_recipe,
        "complete.bin",
        b"complete",
    ));
    let committed_path = store.paths().object_dir(committed_id).join("complete.bin");
    make_writable(&committed_path);
    fs::write(&committed_path, b"short").expect("truncate");

    let malformed_id = recipe(19).artifact_id().expect("malformed id");
    let malformed = store.paths().object_dir(malformed_id);
    fs::create_dir_all(&malformed).expect("malformed object directory");
    fs::write(malformed.join("manifest.json"), b"{").expect("malformed manifest");
    drop(store);

    let (reopened, recovery) = ArtifactStore::initialize(&root).expect("reopen");
    assert_eq!(recovery.staging_quarantined, 1);
    assert_eq!(recovery.objects_quarantined, 2);
    assert_eq!(reopened.committed_count(), 0);
}

#[test]
fn garbage_collection_marks_transitive_inputs_and_reader_pins() {
    let temp = TempDir::new().expect("tempdir");
    let (mut store, _) = ArtifactStore::initialize(temp.path()).expect("store");
    let child_recipe = recipe(7);
    let child_id = child_recipe.artifact_id().expect("child id");
    drop(commit_payload(
        &mut store,
        child_recipe,
        "child.bin",
        b"child",
    ));
    let parent_recipe = recipe_with(8, vec![child_id], None, Map::new());
    let parent_id = parent_recipe.artifact_id().expect("parent id");
    drop(commit_payload(
        &mut store,
        parent_recipe,
        "parent.bin",
        b"parent",
    ));
    let orphan_recipe = recipe(9);
    let orphan_id = orphan_recipe.artifact_id().expect("orphan id");
    let orphan_pin = commit_payload(&mut store, orphan_recipe, "orphan.bin", b"orphan");

    let future = SystemTime::now() + Duration::from_hours(192);
    let grace = Duration::from_hours(168);
    let pinned = store
        .collect_garbage([parent_id], future, grace)
        .expect("pinned gc");
    assert_eq!(pinned.deleted, 0);
    assert_eq!(pinned.reachable, 3);
    drop(orphan_pin);

    let collected = store
        .collect_garbage([parent_id], future, grace)
        .expect("collect orphan");
    assert_eq!(collected.deleted, 1);
    assert!(matches!(
        store.open(orphan_id),
        Err(ArtifactError::NotFound(id)) if id == orphan_id
    ));
    assert!(store.open(parent_id).is_ok());
    assert!(store.open(child_id).is_ok());
}

#[test]
fn garbage_collection_fails_closed_for_a_missing_reachable_root() {
    let temp = TempDir::new().expect("tempdir");
    let (mut store, _) = ArtifactStore::initialize(temp.path()).expect("store");
    let orphan_recipe = recipe(10);
    let orphan_id = orphan_recipe.artifact_id().expect("orphan id");
    drop(commit_payload(
        &mut store,
        orphan_recipe,
        "orphan.bin",
        b"orphan",
    ));
    let missing = ArtifactId::from_digest(Sha256Digest::from_bytes([0xff; 32]));
    assert!(matches!(
        store.collect_garbage(
            [missing],
            SystemTime::now() + Duration::from_hours(192),
            Duration::from_hours(168)
        ),
        Err(ArtifactError::ReachableMissing(id)) if id == missing
    ));
    assert!(store.open(orphan_id).is_ok());
}

#[test]
fn garbage_collection_fails_closed_for_a_corrupt_reachable_payload() {
    let temp = TempDir::new().expect("tempdir");
    let (mut store, _) = ArtifactStore::initialize(temp.path()).expect("store");
    let reachable_recipe = recipe(14);
    let reachable_id = reachable_recipe.artifact_id().expect("reachable id");
    drop(commit_payload(
        &mut store,
        reachable_recipe,
        "reachable.bin",
        b"rooted",
    ));
    let orphan_recipe = recipe(15);
    let orphan_id = orphan_recipe.artifact_id().expect("orphan id");
    drop(commit_payload(
        &mut store,
        orphan_recipe,
        "orphan.bin",
        b"orphan",
    ));

    let reachable_payload = store.paths().object_dir(reachable_id).join("reachable.bin");
    make_writable(&reachable_payload);
    fs::write(&reachable_payload, b"broken").expect("same-size corruption");

    assert!(matches!(
        store.collect_garbage(
            [reachable_id],
            SystemTime::now() + Duration::from_hours(192),
            Duration::from_hours(168)
        ),
        Err(ArtifactError::ReachableCorrupt { artifact_id, .. })
            if artifact_id == reachable_id
    ));
    assert!(store.open(orphan_id).is_ok(), "GC must delete nothing");
}

#[test]
fn garbage_collection_honors_exact_grace_and_recovers_interrupted_deletion() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path().join("artifacts");
    let (mut store, _) = ArtifactStore::initialize(&root).expect("store");
    let orphan_recipe = recipe(16);
    let orphan_id = orphan_recipe.artifact_id().expect("orphan id");
    drop(commit_payload(
        &mut store,
        orphan_recipe,
        "orphan.bin",
        b"orphan",
    ));
    let manifest = store.paths().object_dir(orphan_id).join("manifest.json");
    let published_at = fs::metadata(manifest)
        .expect("manifest metadata")
        .modified()
        .expect("manifest timestamp");
    let grace = Duration::from_hours(168);
    let before_grace = grace
        .checked_sub(Duration::from_nanos(1))
        .expect("positive grace");
    let just_before = published_at + before_grace;

    let preserved = store
        .collect_garbage([], just_before, grace)
        .expect("preserve before grace");
    assert_eq!(preserved.preserved_by_grace, 1);
    assert!(store.open(orphan_id).is_ok());
    let collected = store
        .collect_garbage([], published_at + grace, grace)
        .expect("collect at grace boundary");
    assert_eq!(collected.deleted, 1);

    let interrupted = store.paths().quarantine.join("gc-interrupted");
    fs::create_dir(&interrupted).expect("interrupted quarantine wrapper");
    fs::write(interrupted.join("item"), b"partially deleted").expect("quarantine item");
    let quarantined_at = fs::metadata(&interrupted)
        .expect("quarantine metadata")
        .modified()
        .expect("quarantine timestamp");
    drop(store);

    let (mut recovered, _) = ArtifactStore::initialize(root).expect("restart store");
    assert!(interrupted.exists(), "restart retains recent quarantine");
    let retained = recovered
        .collect_garbage([], quarantined_at + before_grace, grace)
        .expect("retain interrupted deletion before grace");
    assert_eq!(retained.quarantine_deleted, 0);
    let cleaned = recovered
        .collect_garbage([], quarantined_at + grace, grace)
        .expect("clean interrupted deletion at grace");
    assert_eq!(cleaned.quarantine_deleted, 1);
    assert!(!interrupted.exists());
}

#[test]
fn legacy_manifest_is_readable_by_id_but_not_a_computed_hit() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path().join("artifacts");
    let recipe = recipe(11);
    let id = recipe.artifact_id().expect("id");
    let paths = clipmill_artifacts::StorePaths::new(root.clone());
    let object = paths.object_dir(id);
    fs::create_dir_all(&object).expect("object dir");
    let payload = b"legacy";
    let payload_digest = Sha256::digest(payload);
    fs::write(object.join("legacy.bin"), payload).expect("payload");
    let manifest = json!({
        "artifact_id": id.to_string(),
        "files": [{
            "bytes": payload.len(),
            "path": "legacy.bin",
            "sha256": format!("sha256:{}", hex_string(payload_digest.as_slice()))
        }],
        "inputs": [],
        "kind": "evidence.probe.v1",
        "policy": "local-lock",
        "producer": {"implementation": "legacy@1", "stage": "probe"},
        "schema_version": "clipmill.artifact.manifest.v1",
        "source_fingerprint": format!("sha256:{}", "0b".repeat(32)),
        "timebase": {"den": 90000, "num": 1}
    });
    let mut text = serde_json::to_string_pretty(&manifest).expect("manifest json");
    text.push('\n');
    fs::write(object.join("manifest.json"), text).expect("manifest");

    let (store, recovery) = ArtifactStore::initialize(root).expect("store");
    assert_eq!(recovery.legacy_loaded, 1);
    assert!(store.open(id).expect("legacy open").is_legacy());
    assert!(matches!(
        store.lookup(&recipe).expect("legacy lookup"),
        CacheLookup::Miss(CacheMissReason::LegacyUnverifiable { artifact_id })
            if artifact_id == id
    ));
}

#[cfg(unix)]
#[test]
fn symlinks_are_rejected_and_committed_permissions_are_private() {
    use std::os::unix::fs::{PermissionsExt, symlink};

    let temp = TempDir::new().expect("tempdir");
    let (mut store, _) = ArtifactStore::initialize(temp.path()).expect("store");
    let stage = prepare_miss(&mut store, recipe(12));
    let outside = temp.path().join("outside");
    fs::write(&outside, b"outside").expect("outside");
    symlink(&outside, stage.path().join("link.bin")).expect("symlink");
    assert!(matches!(
        store.commit(
            stage.id(),
            vec!["link.bin".parse().expect("path")],
            BTreeMap::new()
        ),
        Err(ArtifactError::SymlinkRejected)
    ));

    let recipe = recipe(13);
    let id = recipe.artifact_id().expect("id");
    drop(commit_payload(
        &mut store,
        recipe,
        "private.bin",
        b"private",
    ));
    let object = store.paths().object_dir(id);
    assert_eq!(
        fs::metadata(&object)
            .expect("object metadata")
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(object.join("private.bin"))
            .expect("payload metadata")
            .permissions()
            .mode()
            & 0o777,
        0o400
    );
    assert_eq!(
        fs::metadata(object.join("manifest.json"))
            .expect("manifest metadata")
            .permissions()
            .mode()
            & 0o777,
        0o400
    );
}

fn make_writable(path: &std::path::Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("chmod writable");
    }
}

fn hex_string(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}
