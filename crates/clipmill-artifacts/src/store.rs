use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{self, BufReader, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use clipmill_core::{ArtifactId, Sha256Digest, StagingId};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::{
    ArtifactPath, ArtifactPathError, ArtifactRecipe, RecipeError,
    manifest::{FileRecord, MANIFEST_NAME, ManifestError, StoredManifest},
};

const MAX_MANIFEST_BYTES: u64 = 4 * 1024 * 1024;
const DIRECTORY_MODE: u32 = 0o700;
const STAGING_FILE_MODE: u32 = 0o600;
const COMMITTED_FILE_MODE: u32 = 0o400;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorePaths {
    pub root: PathBuf,
    pub objects: PathBuf,
    pub staging: PathBuf,
    pub quarantine: PathBuf,
    sha256_objects: PathBuf,
}

impl StorePaths {
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        let objects = root.join("objects");
        Self {
            sha256_objects: objects.join("sha256"),
            staging: root.join("staging"),
            quarantine: root.join("quarantine"),
            root,
            objects,
        }
    }

    #[must_use]
    pub fn object_dir(&self, artifact_id: ArtifactId) -> PathBuf {
        let hex = artifact_id.hex();
        self.sha256_objects.join(&hex[..2]).join(hex)
    }
}

#[derive(Debug)]
pub struct ArtifactStore {
    paths: StorePaths,
    catalog: BTreeMap<ArtifactId, CatalogEntry>,
    active: BTreeMap<ArtifactId, StagingState>,
    pins: Arc<Mutex<BTreeMap<ArtifactId, usize>>>,
}

#[derive(Clone, Debug)]
struct CatalogEntry {
    dir: PathBuf,
    manifest: StoredManifest,
    published_at: Option<SystemTime>,
    legacy: bool,
}

#[derive(Clone, Debug)]
struct StagingState {
    id: StagingId,
    artifact_id: ArtifactId,
    path: PathBuf,
    recipe: ArtifactRecipe,
}

impl ArtifactStore {
    pub fn initialize(root: impl Into<PathBuf>) -> Result<(Self, RecoveryReport), ArtifactError> {
        let paths = StorePaths::new(root.into());
        for directory in [
            &paths.root,
            &paths.objects,
            &paths.sha256_objects,
            &paths.staging,
            &paths.quarantine,
        ] {
            create_private_directory(directory)?;
        }

        let mut recovery = RecoveryReport::default();
        quarantine_stale_staging(&paths, &mut recovery)?;
        let catalog = rebuild_catalog(&paths, &mut recovery)?;
        Ok((
            Self {
                paths,
                catalog,
                active: BTreeMap::new(),
                pins: Arc::new(Mutex::new(BTreeMap::new())),
            },
            recovery,
        ))
    }

    #[must_use]
    pub fn paths(&self) -> &StorePaths {
        &self.paths
    }

    #[must_use]
    pub fn committed_count(&self) -> usize {
        self.catalog.len()
    }

    pub fn lookup(&self, recipe: &ArtifactRecipe) -> Result<CacheLookup, ArtifactError> {
        let artifact_id = recipe.artifact_id()?;
        if self.active.contains_key(&artifact_id) {
            return Ok(CacheLookup::Miss(CacheMissReason::InFlight { artifact_id }));
        }
        let Some(entry) = self.catalog.get(&artifact_id) else {
            return Ok(CacheLookup::Miss(CacheMissReason::NotPresent {
                artifact_id,
            }));
        };
        if entry.legacy {
            return Ok(CacheLookup::Miss(CacheMissReason::LegacyUnverifiable {
                artifact_id,
            }));
        }
        let stored_recipe = entry
            .manifest
            .recipe()?
            .ok_or(ArtifactError::LegacyManifest(artifact_id))?;
        if stored_recipe != *recipe {
            return Err(ArtifactError::ArtifactIdCollision(artifact_id));
        }
        Ok(CacheLookup::Hit(self.open(artifact_id)?))
    }

    pub fn prepare(&mut self, recipe: ArtifactRecipe) -> Result<PrepareOutcome, ArtifactError> {
        let artifact_id = recipe.artifact_id()?;
        if self.active.contains_key(&artifact_id) {
            return Ok(PrepareOutcome::InFlight { artifact_id });
        }
        if let Some(entry) = self.catalog.get(&artifact_id) {
            if entry.legacy {
                return Err(ArtifactError::LegacyManifest(artifact_id));
            }
            let stored_recipe = entry
                .manifest
                .recipe()?
                .ok_or(ArtifactError::LegacyManifest(artifact_id))?;
            if stored_recipe != recipe {
                return Err(ArtifactError::ArtifactIdCollision(artifact_id));
            }
            return Ok(PrepareOutcome::Hit(self.open(artifact_id)?));
        }

        let final_path = self.paths.object_dir(artifact_id);
        if fs::symlink_metadata(&final_path).is_ok() {
            return Err(ArtifactError::UncataloguedObject(artifact_id));
        }
        let id = StagingId::new();
        let path = self.paths.staging.join(id.as_str());
        create_private_directory(&path)?;
        self.active.insert(
            artifact_id,
            StagingState {
                id: id.clone(),
                artifact_id,
                path: path.clone(),
                recipe,
            },
        );
        Ok(PrepareOutcome::Miss(StagingArea {
            id,
            artifact_id,
            path,
        }))
    }

    pub fn commit(
        &mut self,
        staging_id: &StagingId,
        declared_paths: Vec<ArtifactPath>,
        quality: BTreeMap<String, f64>,
    ) -> Result<ArtifactLease, ArtifactError> {
        let artifact_id = self
            .active
            .iter()
            .find_map(|(artifact_id, state)| (state.id == *staging_id).then_some(*artifact_id))
            .ok_or_else(|| ArtifactError::UnknownStaging(staging_id.to_string()))?;
        let state = self
            .active
            .remove(&artifact_id)
            .ok_or_else(|| ArtifactError::UnknownStaging(staging_id.to_string()))?;
        let result = self.commit_inner(&state, declared_paths, quality);
        match result {
            Ok(lease) => Ok(lease),
            Err(error) => {
                if fs::symlink_metadata(&state.path).is_ok()
                    && let Err(quarantine_error) =
                        quarantine_entry(&self.paths, &state.path, "commit-failed")
                {
                    return Err(ArtifactError::QuarantineAfterFailure {
                        original: error.to_string(),
                        quarantine: quarantine_error.to_string(),
                    });
                }
                Err(error)
            }
        }
    }

    /// Revoke an uncommitted staging token and quarantine its directory.
    ///
    /// Worker disconnect, cancellation, and lease expiry use this operation so
    /// abandoned bytes can never remain an active candidate or be published by
    /// a later connection.
    pub fn abandon(&mut self, staging_id: &StagingId) -> Result<bool, ArtifactError> {
        let artifact_id = self
            .active
            .iter()
            .find_map(|(artifact_id, state)| (state.id == *staging_id).then_some(*artifact_id));
        let Some(artifact_id) = artifact_id else {
            return Ok(false);
        };
        let state = self
            .active
            .remove(&artifact_id)
            .ok_or(ArtifactError::InvalidStoreLayout)?;
        if fs::symlink_metadata(&state.path).is_ok() {
            quarantine_entry(&self.paths, &state.path, "abandoned")?;
        }
        Ok(true)
    }

    fn commit_inner(
        &mut self,
        state: &StagingState,
        declared_paths: Vec<ArtifactPath>,
        quality: BTreeMap<String, f64>,
    ) -> Result<ArtifactLease, ArtifactError> {
        if declared_paths.is_empty() {
            return Err(ArtifactError::NoDeclaredFiles);
        }
        let declared_count = declared_paths.len();
        let declared = declared_paths.into_iter().collect::<BTreeSet<_>>();
        if declared.len() != declared_count {
            return Err(ArtifactError::DuplicateFilePath);
        }
        if declared.is_empty() {
            return Err(ArtifactError::NoDeclaredFiles);
        }
        let actual = scan_payload_paths(&state.path, false)?;
        if actual != declared {
            return Err(ArtifactError::DeclaredFileSetMismatch);
        }

        let mut files = Vec::with_capacity(declared.len());
        for path in &declared {
            let disk_path = state.path.join(path.as_path());
            set_private_permissions(&disk_path, COMMITTED_FILE_MODE)?;
            let (digest, bytes) = hash_and_sync_file(&disk_path)?;
            files.push(FileRecord {
                path: path.clone(),
                digest,
                bytes,
            });
        }

        let manifest =
            StoredManifest::from_parts(state.artifact_id, &state.recipe, &files, quality);
        manifest.validate(state.artifact_id)?;
        let manifest_bytes = manifest.to_pretty_bytes()?;
        let temporary_manifest = state.path.join(".manifest.tmp");
        write_private_file(&temporary_manifest, &manifest_bytes, STAGING_FILE_MODE)?;
        let manifest_path = state.path.join(MANIFEST_NAME);
        fs::rename(&temporary_manifest, &manifest_path)
            .map_err(|source| ArtifactError::io(&manifest_path, source))?;
        set_private_permissions(&manifest_path, COMMITTED_FILE_MODE)?;
        sync_directory(&state.path)?;

        let final_path = self.paths.object_dir(state.artifact_id);
        let final_parent = final_path
            .parent()
            .ok_or(ArtifactError::InvalidStoreLayout)?;
        create_private_directory(final_parent)?;
        // Persist a newly created digest shard (`sha256/ab`) before relying on
        // it as the parent of an acknowledged object rename.
        sync_directory(&self.paths.sha256_objects)?;

        if fs::symlink_metadata(&final_path).is_ok() {
            let existing_bytes = read_manifest_bytes(&final_path)?;
            if existing_bytes != manifest_bytes {
                return Err(ArtifactError::NonDeterministicOutput(state.artifact_id));
            }
            fs::remove_dir_all(&state.path)
                .map_err(|source| ArtifactError::io(&state.path, source))?;
            sync_directory(&self.paths.staging)?;
            let entry = load_catalog_entry(&final_path, state.artifact_id)?;
            self.catalog.insert(state.artifact_id, entry);
            return self.open(state.artifact_id);
        }

        fs::rename(&state.path, &final_path)
            .map_err(|source| ArtifactError::io(&final_path, source))?;
        sync_directory(&self.paths.staging)?;
        sync_directory(final_parent)?;
        let entry = load_catalog_entry(&final_path, state.artifact_id)?;
        self.catalog.insert(state.artifact_id, entry);
        self.open(state.artifact_id)
    }

    pub fn open(&self, artifact_id: ArtifactId) -> Result<ArtifactLease, ArtifactError> {
        let entry = self
            .catalog
            .get(&artifact_id)
            .ok_or(ArtifactError::NotFound(artifact_id))?;
        let current = load_catalog_entry(&entry.dir, artifact_id)?;
        pin(&self.pins, artifact_id)?;
        Ok(ArtifactLease {
            artifact_id,
            dir: current.dir,
            manifest: Box::new(current.manifest),
            pins: Arc::clone(&self.pins),
        })
    }

    pub fn collect_garbage(
        &mut self,
        roots: impl IntoIterator<Item = ArtifactId>,
        now: SystemTime,
        grace: Duration,
    ) -> Result<GcReport, ArtifactError> {
        let mut pending = roots.into_iter().collect::<Vec<_>>();
        pending.extend(pinned_ids(&self.pins)?);
        let mut reachable = BTreeSet::new();

        while let Some(artifact_id) = pending.pop() {
            if !reachable.insert(artifact_id) {
                continue;
            }
            let entry = self
                .catalog
                .get(&artifact_id)
                .ok_or(ArtifactError::ReachableMissing(artifact_id))?;
            let inputs = load_reachable_inputs(&entry.dir, artifact_id).map_err(|error| {
                ArtifactError::ReachableCorrupt {
                    artifact_id,
                    detail: error.to_string(),
                }
            })?;
            pending.extend(inputs);
        }

        let candidates = self
            .catalog
            .iter()
            .filter_map(|(artifact_id, entry)| {
                if reachable.contains(artifact_id) || !older_than(entry.published_at, now, grace) {
                    None
                } else {
                    Some((*artifact_id, entry.dir.clone()))
                }
            })
            .collect::<Vec<_>>();

        let mut report = GcReport {
            reachable: reachable.len(),
            preserved_by_grace: self
                .catalog
                .iter()
                .filter(|(id, entry)| {
                    !reachable.contains(id) && !older_than(entry.published_at, now, grace)
                })
                .count(),
            ..GcReport::default()
        };
        for (artifact_id, path) in candidates {
            let quarantine = quarantine_entry(&self.paths, &path, "gc")?;
            self.catalog.remove(&artifact_id);
            fs::remove_dir_all(&quarantine)
                .map_err(|source| ArtifactError::io(&quarantine, source))?;
            sync_directory(&self.paths.quarantine)?;
            report.deleted += 1;
        }
        report.quarantine_deleted = cleanup_quarantine(&self.paths, now, grace)?;
        Ok(report)
    }
}

#[derive(Debug)]
pub enum CacheLookup {
    Hit(ArtifactLease),
    Miss(CacheMissReason),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CacheMissReason {
    NotPresent { artifact_id: ArtifactId },
    InFlight { artifact_id: ArtifactId },
    LegacyUnverifiable { artifact_id: ArtifactId },
}

#[derive(Debug)]
pub enum PrepareOutcome {
    Hit(ArtifactLease),
    Miss(StagingArea),
    InFlight { artifact_id: ArtifactId },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagingArea {
    id: StagingId,
    artifact_id: ArtifactId,
    path: PathBuf,
}

impl StagingArea {
    #[must_use]
    pub fn id(&self) -> &StagingId {
        &self.id
    }

    #[must_use]
    pub const fn artifact_id(&self) -> ArtifactId {
        self.artifact_id
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn create_file(&self, path: &ArtifactPath) -> Result<File, ArtifactError> {
        let disk_path = self.path.join(path.as_path());
        let parent = disk_path
            .parent()
            .ok_or(ArtifactError::InvalidStoreLayout)?;
        create_private_directory(parent)?;
        let mut options = OpenOptions::new();
        options.create(true).truncate(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(STAGING_FILE_MODE);
        }
        let file = options
            .open(&disk_path)
            .map_err(|source| ArtifactError::io(&disk_path, source))?;
        set_private_permissions(&disk_path, STAGING_FILE_MODE)?;
        Ok(file)
    }
}

#[derive(Debug)]
pub struct ArtifactLease {
    artifact_id: ArtifactId,
    dir: PathBuf,
    manifest: Box<StoredManifest>,
    pins: Arc<Mutex<BTreeMap<ArtifactId, usize>>>,
}

impl ArtifactLease {
    #[must_use]
    pub const fn artifact_id(&self) -> ArtifactId {
        self.artifact_id
    }

    #[must_use]
    pub fn kind(&self) -> &str {
        self.manifest.kind()
    }

    #[must_use]
    pub fn stage(&self) -> &str {
        self.manifest.stage()
    }

    #[must_use]
    pub fn is_legacy(&self) -> bool {
        self.manifest.recipe().is_ok_and(|recipe| recipe.is_none())
    }

    pub fn file_paths(&self) -> Result<Vec<ArtifactPath>, ArtifactError> {
        Ok(self
            .manifest
            .file_records()?
            .into_iter()
            .map(|file| file.path)
            .collect())
    }

    pub fn open_verified(&self, path: &ArtifactPath) -> Result<File, ArtifactError> {
        let record = self
            .manifest
            .file_records()?
            .into_iter()
            .find(|file| file.path == *path)
            .ok_or_else(|| ArtifactError::FileNotDeclared(path.to_string()))?;
        let disk_path = self.dir.join(path.as_path());
        let metadata = fs::symlink_metadata(&disk_path)
            .map_err(|source| ArtifactError::io(&disk_path, source))?;
        if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            return Err(ArtifactError::NonRegularFile);
        }
        let mut file =
            File::open(&disk_path).map_err(|source| ArtifactError::io(&disk_path, source))?;
        if metadata.len() != record.bytes {
            return Err(ArtifactError::PayloadSizeMismatch);
        }
        let digest = hash_open_file(&mut file, &disk_path)?;
        if digest != record.digest {
            return Err(ArtifactError::PayloadHashMismatch);
        }
        file.seek(SeekFrom::Start(0))
            .map_err(|source| ArtifactError::io(&disk_path, source))?;
        Ok(file)
    }
}

impl Drop for ArtifactLease {
    fn drop(&mut self) {
        if let Ok(mut pins) = self.pins.lock()
            && let Some(count) = pins.get_mut(&self.artifact_id)
        {
            *count = count.saturating_sub(1);
            if *count == 0 {
                pins.remove(&self.artifact_id);
            }
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecoveryReport {
    pub staging_quarantined: usize,
    pub objects_quarantined: usize,
    pub committed_loaded: usize,
    pub legacy_loaded: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GcReport {
    pub reachable: usize,
    pub preserved_by_grace: usize,
    pub deleted: usize,
    pub quarantine_deleted: usize,
}

fn rebuild_catalog(
    paths: &StorePaths,
    report: &mut RecoveryReport,
) -> Result<BTreeMap<ArtifactId, CatalogEntry>, ArtifactError> {
    let mut catalog = BTreeMap::new();
    for prefix in directory_entries(&paths.sha256_objects)? {
        let metadata =
            fs::symlink_metadata(&prefix).map_err(|source| ArtifactError::io(&prefix, source))?;
        let prefix_name = utf8_file_name(&prefix)?;
        if metadata.file_type().is_symlink()
            || !metadata.is_dir()
            || prefix_name.len() != 2
            || !is_lower_hex(prefix_name)
        {
            quarantine_entry(paths, &prefix, "invalid-prefix")?;
            report.objects_quarantined += 1;
            continue;
        }
        for object in directory_entries(&prefix)? {
            let object_name = utf8_file_name(&object)?;
            let expected = format!("sha256:{object_name}").parse::<ArtifactId>();
            let valid_name = object_name.len() == 64
                && is_lower_hex(object_name)
                && object_name.starts_with(prefix_name);
            if !valid_name {
                quarantine_entry(paths, &object, "invalid-object-name")?;
                report.objects_quarantined += 1;
                continue;
            }
            let Ok(artifact_id) = expected else {
                quarantine_entry(paths, &object, "invalid-object-name")?;
                report.objects_quarantined += 1;
                continue;
            };
            if let Ok(entry) = load_catalog_entry(&object, artifact_id) {
                if entry.legacy {
                    report.legacy_loaded += 1;
                }
                report.committed_loaded += 1;
                catalog.insert(artifact_id, entry);
            } else {
                quarantine_entry(paths, &object, "invalid-object")?;
                report.objects_quarantined += 1;
            }
        }
    }
    Ok(catalog)
}

fn quarantine_stale_staging(
    paths: &StorePaths,
    report: &mut RecoveryReport,
) -> Result<(), ArtifactError> {
    for entry in directory_entries(&paths.staging)? {
        quarantine_entry(paths, &entry, "stale-staging")?;
        report.staging_quarantined += 1;
    }
    Ok(())
}

fn load_catalog_entry(path: &Path, artifact_id: ArtifactId) -> Result<CatalogEntry, ArtifactError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| ArtifactError::io(path, source))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(ArtifactError::NonRegularFile);
    }
    let bytes = read_manifest_bytes(path)?;
    let manifest = StoredManifest::from_bytes(&bytes, artifact_id)?;
    let declared = manifest
        .file_records()?
        .into_iter()
        .map(|file| (file.path, file.bytes))
        .collect::<BTreeMap<_, _>>();
    let actual = scan_payload_paths(path, true)?;
    if actual != declared.keys().cloned().collect() {
        return Err(ArtifactError::DeclaredFileSetMismatch);
    }
    for (artifact_path, expected_bytes) in &declared {
        let payload = path.join(artifact_path.as_path());
        let payload_metadata =
            fs::symlink_metadata(&payload).map_err(|source| ArtifactError::io(&payload, source))?;
        if payload_metadata.len() != *expected_bytes {
            return Err(ArtifactError::PayloadSizeMismatch);
        }
    }
    let manifest_metadata = fs::metadata(path.join(MANIFEST_NAME))
        .map_err(|source| ArtifactError::io(path.join(MANIFEST_NAME), source))?;
    let published_at = manifest_metadata.modified().ok();
    let legacy = manifest.recipe()?.is_none();
    Ok(CatalogEntry {
        dir: path.to_path_buf(),
        manifest,
        published_at,
        legacy,
    })
}

fn load_reachable_inputs(
    path: &Path,
    artifact_id: ArtifactId,
) -> Result<Vec<ArtifactId>, ArtifactError> {
    let entry = load_catalog_entry(path, artifact_id)?;
    for record in entry.manifest.file_records()? {
        let payload = entry.dir.join(record.path.as_path());
        let metadata =
            fs::symlink_metadata(&payload).map_err(|source| ArtifactError::io(&payload, source))?;
        if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
            return Err(ArtifactError::NonRegularFile);
        }
        let mut file =
            File::open(&payload).map_err(|source| ArtifactError::io(&payload, source))?;
        if metadata.len() != record.bytes {
            return Err(ArtifactError::PayloadSizeMismatch);
        }
        if hash_open_file(&mut file, &payload)? != record.digest {
            return Err(ArtifactError::PayloadHashMismatch);
        }
    }
    entry.manifest.input_ids().map_err(Into::into)
}

fn read_manifest_bytes(object_dir: &Path) -> Result<Vec<u8>, ArtifactError> {
    let path = object_dir.join(MANIFEST_NAME);
    let metadata =
        fs::symlink_metadata(&path).map_err(|source| ArtifactError::io(&path, source))?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Err(ArtifactError::NonRegularFile);
    }
    if metadata.len() > MAX_MANIFEST_BYTES {
        return Err(ArtifactError::ManifestTooLarge);
    }
    fs::read(&path).map_err(|source| ArtifactError::io(&path, source))
}

fn scan_payload_paths(
    root: &Path,
    skip_manifest: bool,
) -> Result<BTreeSet<ArtifactPath>, ArtifactError> {
    let mut paths = BTreeSet::new();
    scan_payload_directory(root, root, skip_manifest, &mut paths)?;
    Ok(paths)
}

fn scan_payload_directory(
    root: &Path,
    current: &Path,
    skip_manifest: bool,
    paths: &mut BTreeSet<ArtifactPath>,
) -> Result<(), ArtifactError> {
    for entry in directory_entries(current)? {
        let metadata =
            fs::symlink_metadata(&entry).map_err(|source| ArtifactError::io(&entry, source))?;
        if metadata.file_type().is_symlink() {
            return Err(ArtifactError::SymlinkRejected);
        }
        if metadata.is_dir() {
            scan_payload_directory(root, &entry, skip_manifest, paths)?;
            continue;
        }
        if !metadata.is_file() {
            return Err(ArtifactError::NonRegularFile);
        }
        let relative = portable_relative_path(root, &entry)?;
        if skip_manifest && relative == MANIFEST_NAME {
            continue;
        }
        let artifact_path = relative.parse::<ArtifactPath>()?;
        if !paths.insert(artifact_path) {
            return Err(ArtifactError::DuplicateFilePath);
        }
    }
    Ok(())
}

fn portable_relative_path(root: &Path, path: &Path) -> Result<String, ArtifactError> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| ArtifactError::InvalidStoreLayout)?;
    let mut components = Vec::new();
    for component in relative.components() {
        let value = component
            .as_os_str()
            .to_str()
            .ok_or(ArtifactError::NonUtf8Path)?;
        components.push(value);
    }
    Ok(components.join("/"))
}

fn hash_and_sync_file(path: &Path) -> Result<(Sha256Digest, u64), ArtifactError> {
    let mut file = File::open(path).map_err(|source| ArtifactError::io(path, source))?;
    file.sync_all()
        .map_err(|source| ArtifactError::io(path, source))?;
    let metadata = file
        .metadata()
        .map_err(|source| ArtifactError::io(path, source))?;
    let digest = hash_open_file(&mut file, path)?;
    Ok((digest, metadata.len()))
}

fn hash_open_file(file: &mut File, path: &Path) -> Result<Sha256Digest, ArtifactError> {
    file.seek(SeekFrom::Start(0))
        .map_err(|source| ArtifactError::io(path, source))?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 64 * 1024].into_boxed_slice();
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|source| ArtifactError::io(path, source))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(Sha256Digest::from_bytes(hasher.finalize().into()))
}

fn write_private_file(path: &Path, bytes: &[u8], mode: u32) -> Result<(), ArtifactError> {
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(mode);
    }
    let mut file = options
        .open(path)
        .map_err(|source| ArtifactError::io(path, source))?;
    file.write_all(bytes)
        .map_err(|source| ArtifactError::io(path, source))?;
    file.sync_all()
        .map_err(|source| ArtifactError::io(path, source))?;
    set_private_permissions(path, mode)
}

fn create_private_directory(path: &Path) -> Result<(), ArtifactError> {
    fs::create_dir_all(path).map_err(|source| ArtifactError::io(path, source))?;
    set_private_permissions(path, DIRECTORY_MODE)
}

fn set_private_permissions(path: &Path, mode: u32) -> Result<(), ArtifactError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(mode))
            .map_err(|source| ArtifactError::io(path, source))?;
    }
    #[cfg(not(unix))]
    let _ = (path, mode);
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), ArtifactError> {
    let directory = File::open(path).map_err(|source| ArtifactError::io(path, source))?;
    directory
        .sync_all()
        .map_err(|source| ArtifactError::io(path, source))
}

fn directory_entries(path: &Path) -> Result<Vec<PathBuf>, ArtifactError> {
    let entries = fs::read_dir(path).map_err(|source| ArtifactError::io(path, source))?;
    entries
        .map(|entry| {
            entry
                .map(|value| value.path())
                .map_err(|source| ArtifactError::io(path, source))
        })
        .collect()
}

fn utf8_file_name(path: &Path) -> Result<&str, ArtifactError> {
    path.file_name()
        .and_then(|value| value.to_str())
        .ok_or(ArtifactError::NonUtf8Path)
}

fn is_lower_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn quarantine_entry(
    paths: &StorePaths,
    source: &Path,
    reason: &str,
) -> Result<PathBuf, ArtifactError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ArtifactError::SystemClock)?
        .as_millis();
    let wrapper = paths
        .quarantine
        .join(format!("{reason}-{timestamp}-{}", StagingId::new()));
    create_private_directory(&wrapper)?;
    let target = wrapper.join("item");
    if let Err(source_error) = fs::rename(source, &target) {
        let _cleanup = fs::remove_dir(&wrapper);
        return Err(ArtifactError::io(source, source_error));
    }
    if let Some(parent) = source.parent() {
        sync_directory(parent)?;
    }
    sync_directory(&wrapper)?;
    sync_directory(&paths.quarantine)?;
    Ok(wrapper)
}

fn cleanup_quarantine(
    paths: &StorePaths,
    now: SystemTime,
    grace: Duration,
) -> Result<usize, ArtifactError> {
    let mut deleted = 0;
    for entry in directory_entries(&paths.quarantine)? {
        let metadata =
            fs::symlink_metadata(&entry).map_err(|source| ArtifactError::io(&entry, source))?;
        if !older_than(metadata.modified().ok(), now, grace) {
            continue;
        }
        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            fs::remove_dir_all(&entry).map_err(|source| ArtifactError::io(&entry, source))?;
        } else {
            fs::remove_file(&entry).map_err(|source| ArtifactError::io(&entry, source))?;
        }
        deleted += 1;
    }
    if deleted > 0 {
        sync_directory(&paths.quarantine)?;
    }
    Ok(deleted)
}

fn older_than(published: Option<SystemTime>, now: SystemTime, grace: Duration) -> bool {
    published
        .and_then(|time| now.duration_since(time).ok())
        .is_some_and(|age| age >= grace)
}

fn pin(
    pins: &Arc<Mutex<BTreeMap<ArtifactId, usize>>>,
    artifact_id: ArtifactId,
) -> Result<(), ArtifactError> {
    let mut pins = pins
        .lock()
        .map_err(|_| ArtifactError::PinRegistryPoisoned)?;
    let count = pins.entry(artifact_id).or_insert(0);
    *count = count.saturating_add(1);
    Ok(())
}

fn pinned_ids(
    pins: &Arc<Mutex<BTreeMap<ArtifactId, usize>>>,
) -> Result<Vec<ArtifactId>, ArtifactError> {
    let pins = pins
        .lock()
        .map_err(|_| ArtifactError::PinRegistryPoisoned)?;
    Ok(pins.keys().copied().collect())
}

#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error("artifact recipe is invalid: {0}")]
    Recipe(#[from] RecipeError),
    #[error("artifact path is invalid: {0}")]
    Path(#[from] ArtifactPathError),
    #[error("artifact manifest is invalid: {0}")]
    Manifest(String),
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("artifact {0} was not found")]
    NotFound(ArtifactId),
    #[error("artifact {0} uses a legacy manifest without a verifiable recipe")]
    LegacyManifest(ArtifactId),
    #[error("artifact id collision for {0}")]
    ArtifactIdCollision(ArtifactId),
    #[error("uncatalogued object already occupies artifact id {0}")]
    UncataloguedObject(ArtifactId),
    #[error("unknown staging id {0}")]
    UnknownStaging(String),
    #[error("an artifact must declare at least one payload file")]
    NoDeclaredFiles,
    #[error("staging files do not exactly match the declared artifact paths")]
    DeclaredFileSetMismatch,
    #[error("the same artifact path appears more than once")]
    DuplicateFilePath,
    #[error("artifact staging and object paths cannot contain symlinks")]
    SymlinkRejected,
    #[error("artifact entries must be regular files or directories")]
    NonRegularFile,
    #[error("artifact paths must be valid UTF-8")]
    NonUtf8Path,
    #[error("artifact manifest exceeds 4 MiB")]
    ManifestTooLarge,
    #[error("artifact payload size does not match its manifest")]
    PayloadSizeMismatch,
    #[error("artifact payload hash does not match its manifest")]
    PayloadHashMismatch,
    #[error("artifact file {0} is not declared by its manifest")]
    FileNotDeclared(String),
    #[error("artifact {0} produced different deterministic output for the same recipe")]
    NonDeterministicOutput(ArtifactId),
    #[error("reachable artifact {0} is missing; garbage collection aborted")]
    ReachableMissing(ArtifactId),
    #[error("reachable artifact {artifact_id} is corrupt; garbage collection aborted: {detail}")]
    ReachableCorrupt {
        artifact_id: ArtifactId,
        detail: String,
    },
    #[error("artifact reader pin registry is poisoned")]
    PinRegistryPoisoned,
    #[error("system clock is before the Unix epoch")]
    SystemClock,
    #[error("artifact store layout is invalid")]
    InvalidStoreLayout,
    #[error(
        "commit failed ({original}) and its staging area could not be quarantined ({quarantine})"
    )]
    QuarantineAfterFailure {
        original: String,
        quarantine: String,
    },
}

impl ArtifactError {
    fn io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}

impl From<ManifestError> for ArtifactError {
    fn from(value: ManifestError) -> Self {
        Self::Manifest(value.to_string())
    }
}
