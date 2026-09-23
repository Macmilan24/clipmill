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
    mark: MarkProgress,
}

/// The verification a collection had done when it yielded to foreground work.
///
/// A mark reads every reachable manifest and hashes every reachable payload.
/// On a large store that takes long enough that foreground work nearly always
/// arrives first, and a retry that started again from the first byte would
/// never finish; nor would one that restarted an object of thousands of files
/// each time. The retry still marks from fresh roots and reader pins, but
/// resumes where this pass stopped: past the objects it verified, and inside
/// the object it was verifying, at the payload and byte it had reached. A pass
/// that completes or fails forgets all of it, so every pass verifies every
/// reachable byte once.
#[derive(Debug, Default)]
struct MarkProgress {
    /// Objects this pass verified, with the inputs their manifests name.
    verified: BTreeMap<ArtifactId, Vec<ArtifactId>>,
    /// Objects this pass began verifying and had not finished.
    partial: BTreeMap<ArtifactId, PartialObject>,
}

/// An object part-way through verification: its manifest read, whether the
/// files on disk were found to be exactly those it declares, the payloads
/// `records[..next]` hashed as declared, and `hashing` the progress through
/// `records[next]`.
#[derive(Debug)]
struct PartialObject {
    dir: PathBuf,
    records: Vec<FileRecord>,
    inputs: Vec<ArtifactId>,
    listed: bool,
    next: usize,
    hashing: PartialHash,
}

impl PartialObject {
    /// Read a reachable object's manifest, the one step of its verification
    /// that cannot yield part-way, and so is done once per pass.
    fn read(
        dir: &Path,
        artifact_id: ArtifactId,
        interrupted: &mut impl FnMut() -> bool,
    ) -> Result<Self, ArtifactError> {
        let manifest = read_object_manifest(dir, artifact_id, interrupted)?;
        manifest.recipe()?;
        Ok(Self {
            dir: dir.to_path_buf(),
            records: manifest.file_records()?,
            inputs: manifest.input_ids()?,
            listed: false,
            next: 0,
            hashing: PartialHash::default(),
        })
    }

    /// Check the file set, then hash the payloads not yet verified, from
    /// where the last call stopped.
    fn verify(&mut self, interrupted: &mut impl FnMut() -> bool) -> Result<(), ArtifactError> {
        if !self.listed {
            // A listing cannot resume part-way, so it runs without yielding:
            // one directory's metadata, bounded by the object's file count.
            // Yielding inside it would restart it, and an object of thousands
            // of files would never get past it while the engine is busy.
            check_declared_files(&self.dir, &self.records, &mut || false)?;
            self.listed = true;
        }
        while let Some(record) = self.records.get(self.next) {
            checkpoint(interrupted)?;
            let payload = self.dir.join(record.path.as_path());
            let metadata = fs::symlink_metadata(&payload)
                .map_err(|source| ArtifactError::io(&payload, source))?;
            if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
                return Err(ArtifactError::NonRegularFile);
            }
            let mut file =
                File::open(&payload).map_err(|source| ArtifactError::io(&payload, source))?;
            if metadata.len() != record.bytes {
                return Err(ArtifactError::PayloadSizeMismatch);
            }
            let from = std::mem::take(&mut self.hashing);
            match hash_open_file_resumable(&mut file, &payload, from, interrupted)? {
                Hashed::Done(digest) if digest == record.digest => self.next += 1,
                Hashed::Done(_) => return Err(ArtifactError::PayloadHashMismatch),
                Hashed::Yielded(progress) => {
                    self.hashing = progress;
                    return Err(ArtifactError::CollectionInterrupted);
                }
            }
        }
        Ok(())
    }
}

/// A payload hashed up to `offset`.
#[derive(Clone, Debug, Default)]
struct PartialHash {
    offset: u64,
    hasher: Sha256,
}

/// Whether a resumable hash reached the end of its file.
enum Hashed {
    Done(Sha256Digest),
    Yielded(PartialHash),
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
                mark: MarkProgress::default(),
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

    /// What the published objects occupy.
    ///
    /// Summed from the manifests the catalog already holds, not by walking the
    /// object tree. The manifests declare every file's size, they are in memory
    /// already, and a `du` over a store with tens of thousands of objects is not
    /// something a screen should be made to wait on.
    ///
    /// It reports what was published. Staging and quarantine are excluded on
    /// purpose: those are work in progress and wreckage, and neither is an
    /// honest answer to "what do my artifacts occupy".
    #[must_use]
    pub fn usage(&self) -> StoreUsage {
        StoreUsage {
            objects: self.catalog.len() as u64,
            bytes: self
                .catalog
                .values()
                .map(|entry| entry.manifest.declared_total_bytes())
                .sum(),
        }
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

    /// What collection would remove with this grace, without removing it.
    ///
    /// Read from the manifests already in memory, so it is an estimate made
    /// in a moment: the collection itself re-reads and verifies every
    /// reachable manifest from disk before it deletes anything, and keeps
    /// whatever is pinned when it runs.
    pub fn reclaimable(
        &self,
        roots: impl IntoIterator<Item = ArtifactId>,
        now: SystemTime,
        grace: Duration,
    ) -> Result<StoreUsage, ArtifactError> {
        let mut pending = roots.into_iter().collect::<Vec<_>>();
        pending.extend(pinned_ids(&self.pins)?);
        let mut reachable = BTreeSet::new();
        while let Some(artifact_id) = pending.pop() {
            if !reachable.insert(artifact_id) {
                continue;
            }
            // A manifest that cannot name its inputs keeps nothing else alive
            // in this estimate; collection refuses to proceed past it anyway.
            if let Some(entry) = self.catalog.get(&artifact_id)
                && let Ok(inputs) = entry.manifest.input_ids()
            {
                pending.extend(inputs);
            }
        }
        Ok(self
            .catalog
            .iter()
            .filter(|(artifact_id, entry)| {
                !reachable.contains(*artifact_id) && older_than(entry.published_at, now, grace)
            })
            .fold(StoreUsage::default(), |usage, (_, entry)| StoreUsage {
                objects: usage.objects + 1,
                bytes: usage
                    .bytes
                    .saturating_add(entry.manifest.declared_total_bytes()),
            }))
    }

    pub fn collect_garbage(
        &mut self,
        roots: impl IntoIterator<Item = ArtifactId>,
        now: SystemTime,
        grace: Duration,
    ) -> Result<GcReport, ArtifactError> {
        self.collect_garbage_interruptible(roots, now, grace, || false)
    }

    /// Yield to foreground work without treating an incomplete integrity scan
    /// as permission to delete. Callers must retry with fresh roots and pins;
    /// the retry resumes the verification this pass had already done.
    pub fn collect_garbage_interruptible(
        &mut self,
        roots: impl IntoIterator<Item = ArtifactId>,
        now: SystemTime,
        grace: Duration,
        mut interrupted: impl FnMut() -> bool,
    ) -> Result<GcReport, ArtifactError> {
        let mut report = GcReport::default();
        match self.collect_garbage_inner(roots, now, grace, &mut interrupted, &mut report) {
            Ok(()) => {
                self.mark = MarkProgress::default();
                Ok(report)
            }
            Err(ArtifactError::CollectionInterrupted) => {
                report.deferred = true;
                Ok(report)
            }
            Err(error) => {
                self.mark = MarkProgress::default();
                Err(error)
            }
        }
    }

    fn collect_garbage_inner(
        &mut self,
        roots: impl IntoIterator<Item = ArtifactId>,
        now: SystemTime,
        grace: Duration,
        interrupted: &mut impl FnMut() -> bool,
        report: &mut GcReport,
    ) -> Result<(), ArtifactError> {
        checkpoint(interrupted)?;
        let mut pending = roots.into_iter().collect::<Vec<_>>();
        pending.extend(pinned_ids(&self.pins)?);
        let mut reachable = BTreeSet::new();

        while let Some(artifact_id) = pending.pop() {
            checkpoint(interrupted)?;
            if !reachable.insert(artifact_id) {
                continue;
            }
            let entry = self
                .catalog
                .get(&artifact_id)
                .ok_or(ArtifactError::ReachableMissing(artifact_id))?;
            if let Some(inputs) = self.mark.verified.get(&artifact_id) {
                pending.extend(inputs.iter().copied());
                continue;
            }
            let corrupt = |error: ArtifactError| {
                if matches!(error, ArtifactError::CollectionInterrupted) {
                    error
                } else {
                    ArtifactError::ReachableCorrupt {
                        artifact_id,
                        detail: error.to_string(),
                    }
                }
            };
            let mut object = match self.mark.partial.remove(&artifact_id) {
                Some(object) => object,
                None => {
                    PartialObject::read(&entry.dir, artifact_id, interrupted).map_err(corrupt)?
                }
            };
            if let Err(error) = object.verify(interrupted) {
                if matches!(error, ArtifactError::CollectionInterrupted) {
                    self.mark.partial.insert(artifact_id, object);
                }
                return Err(corrupt(error));
            }
            pending.extend(object.inputs.iter().copied());
            self.mark.verified.insert(artifact_id, object.inputs);
        }

        let candidates = self
            .catalog
            .iter()
            .filter_map(|(artifact_id, entry)| {
                if reachable.contains(artifact_id) || !older_than(entry.published_at, now, grace) {
                    None
                } else {
                    Some((
                        *artifact_id,
                        entry.dir.clone(),
                        entry.manifest.declared_total_bytes(),
                    ))
                }
            })
            .collect::<Vec<_>>();

        *report = GcReport {
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
        // No mutation, including old quarantine cleanup, precedes a complete
        // verified mark and this final foreground-work check.
        checkpoint(interrupted)?;
        for (artifact_id, path, bytes) in candidates {
            checkpoint(interrupted)?;
            let quarantine = quarantine_entry(&self.paths, &path, "gc")?;
            self.catalog.remove(&artifact_id);
            self.mark.verified.remove(&artifact_id);
            self.mark.partial.remove(&artifact_id);
            // Once quarantined, this object is safely removed from the store.
            // An interrupted unlink leaves only recoverable quarantine bytes.
            report.deleted += 1;
            report.deleted_bytes = report.deleted_bytes.saturating_add(bytes);
            remove_tree_interruptible(&quarantine, interrupted)?;
            sync_directory(&self.paths.quarantine)?;
        }
        cleanup_quarantine(&self.paths, now, grace, interrupted, report)?;
        Ok(())
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

    /// The size the manifest declares for one payload file.
    ///
    /// The manifest rather than the filesystem, because the manifest is what the
    /// digest covers: a file whose length disagrees with its record is a corrupt
    /// artifact, and reporting the on-disk number would hide that instead of
    /// letting the next verified read refuse it.
    pub fn declared_bytes(&self, path: &ArtifactPath) -> Option<u64> {
        self.manifest
            .file_records()
            .ok()?
            .into_iter()
            .find(|file| file.path == *path)
            .map(|file| file.bytes)
    }

    /// Parse the validated manifest's payload sizes once for a whole inventory.
    ///
    /// Every record still passes path and digest validation. Callers resolving
    /// many files can then check membership without reparsing every record for
    /// each file. These are declared sizes; payload reads must remain verified.
    pub fn declared_file_sizes(&self) -> Result<BTreeMap<ArtifactPath, u64>, ArtifactError> {
        Ok(self
            .manifest
            .file_records()?
            .into_iter()
            .map(|file| (file.path, file.bytes))
            .collect())
    }

    /// Verify one payload file and return its on-disk path for sidecar
    /// processes that must read by filename. Committed payloads are immutable
    /// (mode 0400) so the verified bytes are the bytes the sidecar reads; the
    /// lease pin keeps garbage collection away while the path is in use.
    pub fn verified_path(&self, path: &ArtifactPath) -> Result<PathBuf, ArtifactError> {
        let file = self.open_verified(path)?;
        drop(file);
        Ok(self.dir.join(path.as_path()))
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

/// What the published objects occupy, as the manifests declare it.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StoreUsage {
    pub objects: u64,
    pub bytes: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GcReport {
    pub reachable: usize,
    pub preserved_by_grace: usize,
    pub deleted: usize,
    /// What the deleted objects' manifests declared, so a person told
    /// "cleaned up" can also be told how much.
    pub deleted_bytes: u64,
    pub quarantine_deleted: usize,
    /// Foreground work interrupted this pass. Counts include only completed
    /// mutations; a fresh mark from fresh roots and pins is required before
    /// collecting anything else, though it resumes the verification this pass
    /// had already done.
    pub deferred: bool,
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
    load_catalog_entry_interruptible(path, artifact_id, &mut || false)
}

fn load_catalog_entry_interruptible(
    path: &Path,
    artifact_id: ArtifactId,
    interrupted: &mut impl FnMut() -> bool,
) -> Result<CatalogEntry, ArtifactError> {
    let manifest = read_object_manifest(path, artifact_id, interrupted)?;
    check_declared_files(path, &manifest.file_records()?, interrupted)?;
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

fn read_object_manifest(
    path: &Path,
    artifact_id: ArtifactId,
    interrupted: &mut impl FnMut() -> bool,
) -> Result<StoredManifest, ArtifactError> {
    checkpoint(interrupted)?;
    let metadata = fs::symlink_metadata(path).map_err(|source| ArtifactError::io(path, source))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(ArtifactError::NonRegularFile);
    }
    let bytes = read_manifest_bytes(path)?;
    Ok(StoredManifest::from_bytes(&bytes, artifact_id)?)
}

/// The files in an object's directory are exactly those its manifest
/// declares, each at its declared size.
fn check_declared_files(
    path: &Path,
    records: &[FileRecord],
    interrupted: &mut impl FnMut() -> bool,
) -> Result<(), ArtifactError> {
    let declared = records
        .iter()
        .map(|file| (file.path.clone(), file.bytes))
        .collect::<BTreeMap<_, _>>();
    let actual = scan_payload_paths_interruptible(path, true, interrupted)?;
    if actual != declared.keys().cloned().collect() {
        return Err(ArtifactError::DeclaredFileSetMismatch);
    }
    for (artifact_path, expected_bytes) in &declared {
        checkpoint(interrupted)?;
        let payload = path.join(artifact_path.as_path());
        let payload_metadata =
            fs::symlink_metadata(&payload).map_err(|source| ArtifactError::io(&payload, source))?;
        if payload_metadata.len() != *expected_bytes {
            return Err(ArtifactError::PayloadSizeMismatch);
        }
    }
    Ok(())
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
    scan_payload_paths_interruptible(root, skip_manifest, &mut || false)
}

fn scan_payload_paths_interruptible(
    root: &Path,
    skip_manifest: bool,
    interrupted: &mut impl FnMut() -> bool,
) -> Result<BTreeSet<ArtifactPath>, ArtifactError> {
    let mut paths = BTreeSet::new();
    scan_payload_directory(root, root, skip_manifest, &mut paths, interrupted)?;
    Ok(paths)
}

fn scan_payload_directory(
    root: &Path,
    current: &Path,
    skip_manifest: bool,
    paths: &mut BTreeSet<ArtifactPath>,
    interrupted: &mut impl FnMut() -> bool,
) -> Result<(), ArtifactError> {
    for entry in directory_entries_interruptible(current, interrupted)? {
        checkpoint(interrupted)?;
        let metadata =
            fs::symlink_metadata(&entry).map_err(|source| ArtifactError::io(&entry, source))?;
        if metadata.file_type().is_symlink() {
            return Err(ArtifactError::SymlinkRejected);
        }
        if metadata.is_dir() {
            scan_payload_directory(root, &entry, skip_manifest, paths, interrupted)?;
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
    match hash_open_file_resumable(file, path, PartialHash::default(), &mut || false)? {
        Hashed::Done(digest) => Ok(digest),
        Hashed::Yielded(_) => Err(ArtifactError::CollectionInterrupted),
    }
}

/// Hash a file from where `from` stopped, yielding between reads when
/// `interrupted` says so, with the state a later call resumes from.
fn hash_open_file_resumable(
    file: &mut File,
    path: &Path,
    from: PartialHash,
    interrupted: &mut impl FnMut() -> bool,
) -> Result<Hashed, ArtifactError> {
    let PartialHash {
        mut offset,
        mut hasher,
    } = from;
    file.seek(SeekFrom::Start(offset))
        .map_err(|source| ArtifactError::io(path, source))?;
    let mut reader = BufReader::new(file);
    let mut buffer = vec![0_u8; 64 * 1024].into_boxed_slice();
    loop {
        if interrupted() {
            return Ok(Hashed::Yielded(PartialHash { offset, hasher }));
        }
        let read = reader
            .read(&mut buffer)
            .map_err(|source| ArtifactError::io(path, source))?;
        if read == 0 {
            return Ok(Hashed::Done(Sha256Digest::from_bytes(
                hasher.finalize().into(),
            )));
        }
        hasher.update(&buffer[..read]);
        offset = offset.saturating_add(u64::try_from(read).unwrap_or(u64::MAX));
    }
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
    directory_entries_interruptible(path, &mut || false)
}

fn directory_entries_interruptible(
    path: &Path,
    interrupted: &mut impl FnMut() -> bool,
) -> Result<Vec<PathBuf>, ArtifactError> {
    checkpoint(interrupted)?;
    let entries = fs::read_dir(path).map_err(|source| ArtifactError::io(path, source))?;
    entries
        .map(|entry| {
            checkpoint(interrupted)?;
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
    interrupted: &mut impl FnMut() -> bool,
    report: &mut GcReport,
) -> Result<(), ArtifactError> {
    for entry in directory_entries_interruptible(&paths.quarantine, interrupted)? {
        checkpoint(interrupted)?;
        let metadata =
            fs::symlink_metadata(&entry).map_err(|source| ArtifactError::io(&entry, source))?;
        if !older_than(metadata.modified().ok(), now, grace) {
            continue;
        }
        remove_tree_interruptible(&entry, interrupted)?;
        report.quarantine_deleted += 1;
    }
    if report.quarantine_deleted > 0 {
        sync_directory(&paths.quarantine)?;
    }
    Ok(())
}

fn checkpoint(interrupted: &mut impl FnMut() -> bool) -> Result<(), ArtifactError> {
    if interrupted() {
        Err(ArtifactError::CollectionInterrupted)
    } else {
        Ok(())
    }
}

fn remove_tree_interruptible(
    path: &Path,
    interrupted: &mut impl FnMut() -> bool,
) -> Result<(), ArtifactError> {
    checkpoint(interrupted)?;
    let metadata = fs::symlink_metadata(path).map_err(|source| ArtifactError::io(path, source))?;
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        // Keep the standard library's descriptor-relative, no-follow deletion
        // on Unix. Yield between objects, not via a racy hand-written walk.
        fs::remove_dir_all(path).map_err(|source| ArtifactError::io(path, source))
    } else {
        fs::remove_file(path).map_err(|source| ArtifactError::io(path, source))
    }
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
    #[error("garbage collection yielded to foreground work")]
    CollectionInterrupted,
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
