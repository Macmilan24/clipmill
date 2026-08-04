//! Sandboxed FFmpeg execution for the ingest fan-out (book ch. 12).
//!
//! One decode, all derivatives: the expensive source video decode happens
//! exactly once (the proxy); filmstrip and analysis frames decode the proxy,
//! peaks and loudness read the committed PCM renditions, and the reference
//! index demuxes without decoding at all. Every derivative is its own durable
//! task publishing its own content-addressed artifact, so a kill mid-proxy
//! retries the proxy alone and a warm re-ingest is a pure cache lookup.
//!
//! FFmpeg runs under the same containment discipline as the FFprobe sidecar
//! in `sources.rs`: private scratch, cleared environment, protocol whitelist,
//! bounded outputs, TERM-then-KILL deadlines. Progress is parsed from
//! `-progress` key/value output into real media-time work units — never a
//! synthetic percentage.

use std::{
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io::{BufReader, Read, Seek, SeekFrom, Write},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use clipmill_artifacts::{
    ArtifactLease, ArtifactPath, ArtifactRecipe, NetworkPolicy, PrepareOutcome, Producer,
    RecipeSpec, Timebase,
};
use clipmill_contracts::proto::worker::v1::ProgressUnits;
use clipmill_core::{ArtifactId, Sha256Digest};
use serde_json::{Map, Value, json};
use thiserror::Error;
use ulid::Ulid;

use crate::{
    artifacts::ArtifactHandle,
    db::DbHandle,
    jobs::{LeasedTask, TaskExecutionError},
    sources::{SourceInspector, SourceProbeError, to_edit_ticks},
};

/// The pinned FFmpeg substrate identity every media recipe keys on (R4).
/// Hoisted here so the probe stage, ingest stages, and later the render
/// compiler cannot drift apart silently.
pub(crate) const FFMPEG_BOM: &str = "ffmpeg-8.1.2-btb-n8.1.2";

pub(crate) const KIND_PROXY: &str = "ingest-proxy";
pub(crate) const KIND_AUDIO_16K: &str = "ingest-audio-16k";
pub(crate) const KIND_AUDIO_48K: &str = "ingest-audio-48k";
pub(crate) const KIND_LOUDNESS: &str = "ingest-loudness";
pub(crate) const KIND_REFERENCE_INDEX: &str = "ingest-reference-index";
pub(crate) const KIND_FILMSTRIP: &str = "ingest-filmstrip";
pub(crate) const KIND_AUDIO_PEAKS: &str = "ingest-audio-peaks";
pub(crate) const KIND_FRAMES: &str = "ingest-frames";
pub(crate) const KIND_MANIFEST: &str = "ingest-manifest";

pub(crate) const INGEST_TASK_KINDS: [&str; 9] = [
    KIND_PROXY,
    KIND_AUDIO_16K,
    KIND_AUDIO_48K,
    KIND_LOUDNESS,
    KIND_REFERENCE_INDEX,
    KIND_FILMSTRIP,
    KIND_AUDIO_PEAKS,
    KIND_FRAMES,
    KIND_MANIFEST,
];

const POLL_INTERVAL: Duration = Duration::from_millis(50);
const TERMINATE_GRACE: Duration = Duration::from_millis(500);
const BASE_DEADLINE_MILLIS: u64 = 30_000;
const MAX_DEADLINE_MILLIS: u64 = 4 * 60 * 60 * 1_000;
const MAX_STDERR_BYTES: u64 = 256 * 1024;
const MAX_PROBE_STDOUT_BYTES: u64 = 64 * 1024 * 1024;
const PROGRESS_TAIL_BYTES: u64 = 4 * 1024;
const TICKS_PER_SECOND: i128 = 90_000;
const TICKS_PER_MILLI: i128 = 90;

/// The floor a loudness reading is pinned to when it is below measurement.
///
/// EBU R128 gates quiet windows out rather than reporting them, and ffmpeg
/// says so with `-inf`. Rust parses that to `f64::NEG_INFINITY` quite happily,
/// and `serde_json` then writes it as `null` — because JSON has no infinity —
/// producing a document whose own schema requires a number there. Digital
/// silence already measures about -120.7 LUFS through the same filter, so this
/// is the value the surrounding data is already using for "nothing here".
const SILENCE_FLOOR_LUFS: f64 = -120.0;

/// One `lavfi.r128` reading, with the gated ones pinned to the floor.
///
/// Pinned rather than dropped: the envelope is a series, and a hole in it is
/// harder to read than a floor. See [`SILENCE_FLOOR_LUFS`] for why a raw parse
/// is not enough.
fn measured_lufs(value: &str) -> Option<f64> {
    let parsed = value.trim().parse::<f64>().ok()?;
    Some(if parsed.is_finite() {
        parsed
    } else {
        SILENCE_FLOOR_LUFS
    })
}

const FILMSTRIP_INTERVAL_TICKS: i64 = 90_000;
const FILMSTRIP_TILE_HEIGHT: u32 = 90;
const FRAMES_PER_SECOND: u32 = 6;
const FRAMES_HEIGHT: u32 = 360;
const FRAMES_INTERVAL_TICKS: i64 = 15_000;
const PEAKS_BUCKET_TICKS: i64 = 9_000;
const PEAKS_BUCKET_SAMPLES_16K: usize = 1_600;
const LOUDNESS_CADENCE_HZ: u32 = 10;

/// Latest structured progress for one running task, read by the lease
/// heartbeat so durable task events carry real media-time units.
#[derive(Clone, Debug, Default)]
pub(crate) struct ProgressSlot {
    inner: Arc<Mutex<Option<ProgressUnits>>>,
}

impl ProgressSlot {
    pub(crate) fn set(&self, unit: &str, done: u64, total: u64) {
        if let Ok(mut slot) = self.inner.lock() {
            *slot = Some(ProgressUnits {
                unit: unit.to_owned(),
                done: done.min(total.max(done)),
                total,
            });
        }
    }

    pub(crate) fn take(&self) -> Option<ProgressUnits> {
        self.inner.lock().ok().and_then(|mut slot| slot.take())
    }
}

#[derive(Debug, Error)]
pub(crate) enum MediaError {
    #[error("cannot run media sidecar: {0}")]
    Io(String),
    #[error("FFmpeg exceeded its duration-scaled deadline")]
    Timeout,
    #[error("FFmpeg output exceeded its bounded budget")]
    OutputLimit,
    #[error("FFmpeg rejected the media input")]
    Failed,
    #[error("media sidecar returned invalid output: {0}")]
    InvalidOutput(&'static str),
    #[error("media task stopped unexpectedly")]
    Stopped,
}

impl MediaError {
    pub(crate) fn into_task_error(self) -> TaskExecutionError {
        match self {
            Self::Failed | Self::InvalidOutput(_) | Self::OutputLimit => {
                TaskExecutionError::deterministic(self.to_string())
            }
            Self::Io(_) | Self::Timeout | Self::Stopped => {
                TaskExecutionError::transient(self.to_string())
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn io_error(error: std::io::Error) -> MediaError {
    MediaError::Io(error.to_string())
}

/// Sandboxed runner for the pinned FFmpeg/FFprobe sidecars.
#[derive(Clone, Debug)]
pub(crate) struct MediaRunner {
    ffmpeg: PathBuf,
    ffprobe: PathBuf,
    scratch: PathBuf,
}

/// One FFmpeg invocation: arguments after the fixed containment prefix,
/// relative output names resolved against `output_dir`.
pub(crate) struct FfmpegSpec {
    pub args: Vec<OsString>,
    pub output_dir: PathBuf,
    pub duration_hint_millis: u64,
    pub max_output_bytes: u64,
    /// Return the sidecar's diagnostics on success. Only the loudness
    /// measurement pass needs them, and only to read five numbers out of the
    /// filter's own JSON report; the text itself is never logged or published
    /// because FFmpeg diagnostics can embed local paths.
    pub capture_stderr: bool,
}

impl MediaRunner {
    pub(crate) fn new(ffprobe: PathBuf, scratch: PathBuf) -> Result<Self, MediaError> {
        fs::create_dir_all(&scratch).map_err(io_error)?;
        fs::set_permissions(&scratch, fs::Permissions::from_mode(0o700)).map_err(io_error)?;
        for entry in fs::read_dir(&scratch).map_err(io_error)? {
            let path = entry.map_err(io_error)?.path();
            let metadata = fs::symlink_metadata(&path).map_err(io_error)?;
            if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
                fs::remove_dir_all(&path).map_err(io_error)?;
            } else {
                fs::remove_file(&path).map_err(io_error)?;
            }
        }
        Ok(Self {
            ffmpeg: ffmpeg_sibling(&ffprobe),
            ffprobe,
            scratch,
        })
    }

    /// The pinned decoder, for the one stage that does its own decoding.
    ///
    /// Handed to a worker on its lease rather than resolved there (R4, and the
    /// same reasoning as pinned weights): the daemon fetched this binary
    /// against the bill of materials, and a stage that looked for its own
    /// would publish frames nobody can attribute.
    pub(crate) fn ffmpeg(&self) -> &Path {
        &self.ffmpeg
    }

    pub(crate) async fn run_ffmpeg(
        &self,
        spec: FfmpegSpec,
        progress: ProgressSlot,
    ) -> Result<String, MediaError> {
        let ffmpeg = self.ffmpeg.clone();
        let scratch = self.scratch.clone();
        tokio::task::spawn_blocking(move || {
            run_ffmpeg_blocking(&ffmpeg, &scratch, &spec, &progress)
        })
        .await
        .map_err(|_| MediaError::Stopped)?
    }

    /// Bounded FFprobe over a committed output or the registered source.
    pub(crate) async fn run_ffprobe_json(
        &self,
        target: PathBuf,
        entries: Vec<OsString>,
    ) -> Result<Value, MediaError> {
        let ffprobe = self.ffprobe.clone();
        let scratch = self.scratch.clone();
        tokio::task::spawn_blocking(move || {
            run_ffprobe_blocking(&ffprobe, &scratch, &target, &entries)
        })
        .await
        .map_err(|_| MediaError::Stopped)?
    }
}

fn ffmpeg_sibling(ffprobe: &Path) -> PathBuf {
    let mut ffmpeg = ffprobe.to_path_buf();
    let replacement = ffprobe
        .file_name()
        .and_then(|name| name.to_str())
        .map_or_else(
            || OsString::from("ffmpeg"),
            |name| OsString::from(name.replacen("ffprobe", "ffmpeg", 1)),
        );
    ffmpeg.set_file_name(replacement);
    ffmpeg
}

#[allow(clippy::too_many_lines)]
fn run_ffmpeg_blocking(
    ffmpeg: &Path,
    scratch: &Path,
    spec: &FfmpegSpec,
    progress: &ProgressSlot,
) -> Result<String, MediaError> {
    let work = scratch.join(format!("media_{}", Ulid::new()));
    fs::create_dir(&work).map_err(io_error)?;
    fs::set_permissions(&work, fs::Permissions::from_mode(0o700)).map_err(io_error)?;
    let _cleanup = ScratchGuard(work.clone());
    let stderr_path = work.join("stderr.txt");
    let progress_path = work.join("progress.txt");
    let stderr = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&stderr_path)
        .map_err(io_error)?;

    let mut command = Command::new(ffmpeg);
    command
        .arg("-v")
        .arg("error")
        .arg("-nostdin")
        .arg("-y")
        .arg("-protocol_whitelist")
        .arg("file,pipe");
    command.args(&spec.args);
    command
        .arg("-progress")
        .arg(&progress_path)
        .arg("-nostats")
        .current_dir(&spec.output_dir)
        .env_clear()
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr));
    let mut child = command.spawn().map_err(io_error)?;

    let deadline_millis = BASE_DEADLINE_MILLIS
        .saturating_add(spec.duration_hint_millis.saturating_mul(4))
        .min(MAX_DEADLINE_MILLIS);
    let deadline = Instant::now() + Duration::from_millis(deadline_millis);
    let mut ticks_since_progress = 0_u32;
    let mut ticks_since_size = 0_u32;
    let status = loop {
        if let Some(status) = child.try_wait().map_err(io_error)? {
            break status;
        }
        if Instant::now() >= deadline {
            terminate_child(&mut child);
            return Err(MediaError::Timeout);
        }
        ticks_since_progress += 1;
        if ticks_since_progress >= 10 {
            ticks_since_progress = 0;
            if spec.duration_hint_millis > 0
                && let Some(done) = read_progress_millis(&progress_path)
            {
                progress.set("media_millis", done, spec.duration_hint_millis);
            }
        }
        ticks_since_size += 1;
        if ticks_since_size >= 20 {
            ticks_since_size = 0;
            if directory_bytes(&spec.output_dir) > spec.max_output_bytes {
                terminate_child(&mut child);
                return Err(MediaError::OutputLimit);
            }
        }
        thread::sleep(POLL_INTERVAL);
    };
    if directory_bytes(&spec.output_dir) > spec.max_output_bytes {
        return Err(MediaError::OutputLimit);
    }
    if !status.success() {
        let stderr_len = fs::metadata(&stderr_path).map_or(0, |meta| meta.len());
        if stderr_len > MAX_STDERR_BYTES {
            return Err(MediaError::OutputLimit);
        }
        // FFmpeg diagnostics may embed the source path; classify without
        // retaining any of it (same posture as the FFprobe sidecar).
        return Err(MediaError::Failed);
    }
    if spec.duration_hint_millis > 0 {
        progress.set(
            "media_millis",
            spec.duration_hint_millis,
            spec.duration_hint_millis,
        );
    }
    if !spec.capture_stderr {
        return Ok(String::new());
    }
    if fs::metadata(&stderr_path).map_or(0, |meta| meta.len()) > MAX_STDERR_BYTES {
        return Err(MediaError::OutputLimit);
    }
    fs::read_to_string(&stderr_path).map_err(io_error)
}

fn run_ffprobe_blocking(
    ffprobe: &Path,
    scratch: &Path,
    target: &Path,
    entries: &[OsString],
) -> Result<Value, MediaError> {
    let work = scratch.join(format!("probe_{}", Ulid::new()));
    fs::create_dir(&work).map_err(io_error)?;
    fs::set_permissions(&work, fs::Permissions::from_mode(0o700)).map_err(io_error)?;
    let _cleanup = ScratchGuard(work.clone());
    let stdout_path = work.join("stdout.json");
    let stderr_path = work.join("stderr.txt");
    let stdout = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&stdout_path)
        .map_err(io_error)?;
    let stderr = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&stderr_path)
        .map_err(io_error)?;
    let mut command = Command::new(ffprobe);
    command
        .arg("-v")
        .arg("error")
        .arg("-protocol_whitelist")
        .arg("file,pipe")
        .arg("-print_format")
        .arg("json");
    command.args(entries);
    command
        .arg(target)
        .current_dir(&work)
        .env_clear()
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    let mut child = command.spawn().map_err(io_error)?;
    let deadline = Instant::now() + Duration::from_millis(BASE_DEADLINE_MILLIS.saturating_mul(4));
    let status = loop {
        if let Some(status) = child.try_wait().map_err(io_error)? {
            break status;
        }
        if Instant::now() >= deadline {
            terminate_child(&mut child);
            return Err(MediaError::Timeout);
        }
        thread::sleep(POLL_INTERVAL);
    };
    let stdout_size = fs::metadata(&stdout_path).map_err(io_error)?.len();
    if stdout_size > MAX_PROBE_STDOUT_BYTES {
        return Err(MediaError::OutputLimit);
    }
    if !status.success() {
        return Err(MediaError::Failed);
    }
    let bytes = fs::read(&stdout_path).map_err(io_error)?;
    serde_json::from_slice(&bytes).map_err(|_| MediaError::InvalidOutput("probe JSON is invalid"))
}

fn terminate_child(child: &mut std::process::Child) {
    let _status = Command::new("/bin/kill")
        .arg("-TERM")
        .arg(child.id().to_string())
        .env_clear()
        .status();
    let deadline = Instant::now() + TERMINATE_GRACE;
    while Instant::now() < deadline {
        if child.try_wait().ok().flatten().is_some() {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    let _killed = child.kill();
    let _waited = child.wait();
}

/// Parse the trailing `out_time_us=` from an FFmpeg `-progress` file.
fn read_progress_millis(path: &Path) -> Option<u64> {
    let mut file = File::open(path).ok()?;
    let len = file.metadata().ok()?.len();
    let start = len.saturating_sub(PROGRESS_TAIL_BYTES);
    file.seek(SeekFrom::Start(start)).ok()?;
    let mut tail = String::new();
    file.take(PROGRESS_TAIL_BYTES)
        .read_to_string(&mut tail)
        .ok()?;
    tail.lines()
        .rev()
        .find_map(|line| line.strip_prefix("out_time_us="))
        .and_then(|value| value.trim().parse::<i64>().ok())
        .map(|micros| u64::try_from(micros.max(0) / 1_000).unwrap_or(0))
}

fn directory_bytes(root: &Path) -> u64 {
    fn walk(path: &Path, total: &mut u64) {
        let Ok(entries) = fs::read_dir(path) else {
            return;
        };
        for entry in entries.flatten() {
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            if metadata.is_dir() {
                walk(&entry.path(), total);
            } else {
                *total = total.saturating_add(metadata.len());
            }
        }
    }
    let mut total = 0;
    walk(root, &mut total);
    total
}

#[derive(Debug)]
struct ScratchGuard(PathBuf);

impl Drop for ScratchGuard {
    fn drop(&mut self) {
        let _removed = fs::remove_dir_all(&self.0);
    }
}

// ---- Ingest task execution --------------------------------------------------

/// Everything a source-decoding derivative needs: the re-verified source
/// observation plus the normalized facts from its registered source map.
struct SourceContext {
    absolute_path: String,
    fingerprint: Sha256Digest,
    duration_ticks: i64,
    stream_timebases: Vec<(u64, i128, i128)>,
    stream_kinds: Vec<(u64, String)>,
}

pub(crate) fn is_ingest_kind(kind: &str) -> bool {
    INGEST_TASK_KINDS.contains(&kind)
}

pub(crate) async fn execute_ingest_task(
    database: &DbHandle,
    artifacts: &ArtifactHandle,
    media: &MediaRunner,
    sources: &SourceInspector,
    task: &LeasedTask,
    progress: &ProgressSlot,
) -> Result<ArtifactId, TaskExecutionError> {
    match task.kind.as_str() {
        KIND_PROXY => execute_proxy(database, artifacts, media, sources, task, progress).await,
        KIND_AUDIO_16K => {
            execute_audio(
                database, artifacts, media, sources, task, progress, 16_000, 1,
            )
            .await
        }
        KIND_AUDIO_48K => {
            execute_audio(
                database, artifacts, media, sources, task, progress, 48_000, 2,
            )
            .await
        }
        KIND_LOUDNESS => execute_loudness(artifacts, media, task, progress).await,
        KIND_REFERENCE_INDEX => {
            execute_reference_index(database, artifacts, media, sources, task).await
        }
        KIND_FILMSTRIP => {
            execute_tiles(
                artifacts,
                media,
                task,
                progress,
                TilePlan {
                    prefix: "tile",
                    height: FILMSTRIP_TILE_HEIGHT,
                    rate_num: 1,
                    rate_den: 1,
                    interval_ticks: FILMSTRIP_INTERVAL_TICKS,
                    jpeg_quality: 5,
                },
            )
            .await
        }
        KIND_FRAMES => {
            execute_tiles(
                artifacts,
                media,
                task,
                progress,
                TilePlan {
                    prefix: "frame",
                    height: FRAMES_HEIGHT,
                    rate_num: FRAMES_PER_SECOND,
                    rate_den: 1,
                    interval_ticks: FRAMES_INTERVAL_TICKS,
                    jpeg_quality: 4,
                },
            )
            .await
        }
        KIND_AUDIO_PEAKS => execute_audio_peaks(artifacts, task).await,
        KIND_MANIFEST => execute_ingest_manifest(artifacts, task).await,
        _ => Err(TaskExecutionError::deterministic(
            "unknown ingest task kind",
        )),
    }
}

/// Fetch the durable source record, re-verify the on-disk observation, and
/// extract the normalized facts every source-decoding derivative needs.
async fn source_context(
    database: &DbHandle,
    sources: &SourceInspector,
    task: &LeasedTask,
) -> Result<SourceContext, TaskExecutionError> {
    let source_id = task
        .source_id
        .as_ref()
        .ok_or_else(|| TaskExecutionError::deterministic("ingest task omitted source id"))?;
    let source = database
        .get_source(source_id.clone())
        .await
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
    sources
        .verify(&source.observation)
        .await
        .map_err(|error| match error {
            SourceProbeError::SourceChanged => TaskExecutionError::deterministic("SOURCE_CHANGED"),
            SourceProbeError::InvalidPath(_) | SourceProbeError::ProbeFailed(_) => {
                TaskExecutionError::deterministic(error.to_string())
            }
            _ => TaskExecutionError::transient(error.to_string()),
        })?;
    let fingerprint = source
        .source_fingerprint
        .strip_prefix("sha256:")
        .ok_or_else(|| TaskExecutionError::deterministic("source fingerprint is invalid"))?
        .parse::<Sha256Digest>()
        .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    let map: Value = serde_json::from_slice(&source.source_map_json)
        .map_err(|_| TaskExecutionError::deterministic("source map is not valid JSON"))?;
    let duration_ticks = map["container"]["duration_ticks"].as_i64().unwrap_or(0);
    let mut stream_timebases = Vec::new();
    let mut stream_kinds = Vec::new();
    for stream in map["streams"].as_array().into_iter().flatten() {
        let Some(index) = stream["index"].as_u64() else {
            continue;
        };
        if let (Some(num), Some(den)) = (
            stream["timebase"]["num"].as_i64(),
            stream["timebase"]["den"].as_i64(),
        ) {
            stream_timebases.push((index, i128::from(num), i128::from(den)));
        }
        if let Some(kind) = stream["kind"].as_str() {
            stream_kinds.push((index, kind.to_owned()));
        }
    }
    Ok(SourceContext {
        absolute_path: source.observation.absolute_path,
        fingerprint,
        duration_ticks,
        stream_timebases,
        stream_kinds,
    })
}

pub(crate) fn ticks_to_millis(ticks: i64) -> u64 {
    u64::try_from(i128::from(ticks.max(0)) / TICKS_PER_MILLI).unwrap_or(0)
}

fn media_recipe(
    task: &LeasedTask,
    fingerprint: Sha256Digest,
    semantic_version: &str,
    config: Map<String, Value>,
) -> Result<ArtifactRecipe, TaskExecutionError> {
    ArtifactRecipe::try_from_spec(RecipeSpec {
        kind: task.output_kind.clone(),
        source_fingerprint: fingerprint,
        timebase: Timebase {
            num: 1,
            den: 90_000,
        },
        producer: Producer {
            stage: task.kind.clone(),
            implementation: task.implementation.clone(),
            model_digest: None,
        },
        inputs: task.input_artifact_ids.clone(),
        policy: NetworkPolicy::LocalLock,
        config,
        semantic_version: semantic_version.to_owned(),
    })
    .map_err(|error| TaskExecutionError::deterministic(error.to_string()))
}

pub(crate) enum Prepared {
    Hit(ArtifactId),
    Staged(clipmill_artifacts::StagingArea),
}

pub(crate) async fn prepare_or_hit(
    artifacts: &ArtifactHandle,
    recipe: ArtifactRecipe,
) -> Result<Prepared, TaskExecutionError> {
    match artifacts
        .prepare(recipe)
        .await
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?
    {
        PrepareOutcome::Hit(lease) => Ok(Prepared::Hit(lease.artifact_id())),
        PrepareOutcome::InFlight { .. } => Err(TaskExecutionError::transient(
            "artifact key is already in flight".to_owned(),
        )),
        PrepareOutcome::Miss(staging) => Ok(Prepared::Staged(staging)),
    }
}

/// Quarantine a failed staging area so retries can re-prepare the same key
/// instead of colliding with an abandoned in-flight entry.
pub(crate) async fn abandon_staging(
    artifacts: &ArtifactHandle,
    staging_id: clipmill_core::StagingId,
) {
    if let Err(error) = artifacts.abandon(staging_id).await {
        tracing::warn!(%error, "failed ingest staging could not be abandoned");
    }
}

pub(crate) fn write_canonical_json(
    staging: &clipmill_artifacts::StagingArea,
    path: &ArtifactPath,
    value: &Value,
) -> Result<(), TaskExecutionError> {
    let bytes = serde_json_canonicalizer::to_vec(value)
        .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    let mut file = staging
        .create_file(path)
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
    Ok(())
}

pub(crate) fn artifact_path(value: &str) -> Result<ArtifactPath, TaskExecutionError> {
    value
        .parse::<ArtifactPath>()
        .map_err(|error| TaskExecutionError::deterministic(error.to_string()))
}

pub(crate) async fn commit_staging(
    artifacts: &ArtifactHandle,
    staging_id: clipmill_core::StagingId,
    paths: Vec<ArtifactPath>,
) -> Result<ArtifactId, TaskExecutionError> {
    artifacts
        .commit(staging_id, paths, std::collections::BTreeMap::new())
        .await
        .map(|lease| lease.artifact_id())
        .map_err(|error| TaskExecutionError::transient(error.to_string()))
}

/// Open one verified input artifact and resolve the on-disk path of one of
/// its payload files for sidecar consumption.
pub(crate) async fn verified_input_file(
    artifacts: &ArtifactHandle,
    artifact_id: ArtifactId,
    file: &str,
) -> Result<(ArtifactLease, PathBuf), TaskExecutionError> {
    let lease = artifacts
        .open(artifact_id)
        .await
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
    let path = artifact_path(file)?;
    let disk = lease
        .verified_path(&path)
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
    Ok((lease, disk))
}

/// Read the JSON descriptor a media artifact carries beside its payload.
pub(crate) fn read_descriptor(
    lease: &ArtifactLease,
    file: &str,
) -> Result<Value, TaskExecutionError> {
    let path = artifact_path(file)?;
    let mut reader = lease
        .open_verified(&path)
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
    serde_json::from_slice(&bytes)
        .map_err(|_| TaskExecutionError::deterministic("input descriptor is not valid JSON"))
}

/// Read one verified file out of an input artifact into its contract type.
///
/// The read is verified, so what can still go wrong is the document being
/// something other than what the artifact's kind claimed — a deterministic
/// failure, because a retry reads the same bytes.
pub(crate) fn read_artifact_document<T: serde::de::DeserializeOwned>(
    lease: &ArtifactLease,
    file: &str,
) -> Result<T, TaskExecutionError> {
    let path = artifact_path(file)?;
    let mut reader = lease
        .open_verified(&path)
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
    serde_json::from_slice(&bytes).map_err(|error| {
        TaskExecutionError::deterministic(format!("{file} is not the document it claims: {error}"))
    })
}

#[allow(clippy::too_many_lines)]
async fn execute_proxy(
    database: &DbHandle,
    artifacts: &ArtifactHandle,
    media: &MediaRunner,
    sources: &SourceInspector,
    task: &LeasedTask,
    progress: &ProgressSlot,
) -> Result<ArtifactId, TaskExecutionError> {
    let context = source_context(database, sources, task).await?;
    let mut config = Map::new();
    config.insert("ffmpeg_bom".to_owned(), json!(FFMPEG_BOM));
    config.insert(
        "algorithm".to_owned(),
        json!("clipmill.media.proxy.encode.v1"),
    );
    config.insert(
        "target".to_owned(),
        json!({
            "max_width": 1280,
            "max_height": 720,
            "frame_rate": "30000/1001",
            "gop_frames": 30,
            "video_codec": "libx264",
            "crf": 23,
            "preset": "veryfast",
            "pix_fmt": "yuv420p",
            "audio_codec": "aac",
            "audio_bitrate": 128_000,
            "audio_rate": 48_000,
            "audio_channels": 2,
        }),
    );
    let recipe = media_recipe(task, context.fingerprint, "clipmill.media.proxy.v1", config)?;
    let staging = match prepare_or_hit(artifacts, recipe).await? {
        Prepared::Hit(artifact_id) => return Ok(artifact_id),
        Prepared::Staged(staging) => staging,
    };
    let staging_id = staging.id().clone();
    let result = async {
        let spec = FfmpegSpec {
            args: [
                "-i",
                &context.absolute_path,
                "-map",
                "0:v:0",
                "-map",
                "0:a:0?",
                "-vf",
                "scale=w='min(1280,iw)':h='min(720,ih)':force_original_aspect_ratio=decrease:force_divisible_by=2",
                "-fps_mode",
                "cfr",
                "-r",
                "30000/1001",
                "-c:v",
                "libx264",
                "-preset",
                "veryfast",
                "-crf",
                "23",
                "-pix_fmt",
                "yuv420p",
                "-g",
                "30",
                "-keyint_min",
                "30",
                "-sc_threshold",
                "0",
                "-c:a",
                "aac",
                "-b:a",
                "128k",
                "-ar",
                "48000",
                "-ac",
                "2",
                "-movflags",
                "+faststart",
                "-f",
                "mp4",
                "proxy.mp4",
            ]
            .into_iter()
            .map(OsString::from)
            .collect(),
            output_dir: staging.path().to_path_buf(),
            duration_hint_millis: ticks_to_millis(context.duration_ticks),
            max_output_bytes: 8 * 1024 * 1024 * 1024,
            capture_stderr: false,
        };
        let _diagnostics = media
            .run_ffmpeg(spec, progress.clone())
            .await
            .map_err(MediaError::into_task_error)?;
        let probed = media
            .run_ffprobe_json(
                staging.path().join("proxy.mp4"),
                ["-show_format", "-show_streams"]
                    .into_iter()
                    .map(OsString::from)
                    .collect(),
            )
            .await
            .map_err(MediaError::into_task_error)?;
        let descriptor = proxy_descriptor(&probed, context.fingerprint)?;
        write_canonical_json(&staging, &artifact_path("proxy.json")?, &descriptor)?;
        commit_staging(
            artifacts,
            staging_id.clone(),
            vec![artifact_path("proxy.mp4")?, artifact_path("proxy.json")?],
        )
        .await
    }
    .await;
    if result.is_err() {
        abandon_staging(artifacts, staging_id).await;
    }
    result
}

fn proxy_descriptor(
    probed: &Value,
    fingerprint: Sha256Digest,
) -> Result<Value, TaskExecutionError> {
    let streams = probed["streams"]
        .as_array()
        .ok_or_else(|| TaskExecutionError::deterministic("proxy probe has no streams"))?;
    let video = streams
        .iter()
        .find(|stream| stream["codec_type"] == "video")
        .ok_or_else(|| TaskExecutionError::deterministic("proxy has no video stream"))?;
    let audio = streams
        .iter()
        .find(|stream| stream["codec_type"] == "audio");
    let duration_ticks = probed["format"]["duration"]
        .as_str()
        .map(crate::sources::decimal_seconds_to_ticks)
        .transpose()
        .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?
        .unwrap_or(0)
        .max(0);
    let duration_ticks = i64::try_from(duration_ticks).unwrap_or(i64::MAX);
    let mut descriptor = json!({
        "schema_version": "clipmill.media.proxy.v1",
        "source_fingerprint": format!("sha256:{fingerprint}"),
        "file": "proxy.mp4",
        "container": probed["format"]["format_name"].as_str().unwrap_or("unknown"),
        "duration_ticks": duration_ticks,
        "coverage": {"start_ticks": 0, "end_ticks": duration_ticks},
        "video": {
            "codec": video["codec_name"].as_str().unwrap_or("unknown"),
            "width": video["width"].as_u64().unwrap_or(0),
            "height": video["height"].as_u64().unwrap_or(0),
            "frame_rate": {"num": 30_000, "den": 1_001},
            "gop_frames": 30,
            "pix_fmt": video["pix_fmt"].as_str().unwrap_or("unknown"),
        },
    });
    if let Some(audio) = audio
        && let Some(object) = descriptor.as_object_mut()
    {
        object.insert(
            "audio".to_owned(),
            json!({
                "codec": audio["codec_name"].as_str().unwrap_or("unknown"),
                "sample_rate": audio["sample_rate"]
                    .as_str()
                    .and_then(|value| value.parse::<u64>().ok())
                    .unwrap_or(0),
                "channels": audio["channels"].as_u64().unwrap_or(0),
            }),
        );
    }
    Ok(descriptor)
}

#[allow(clippy::too_many_arguments)]
async fn execute_audio(
    database: &DbHandle,
    artifacts: &ArtifactHandle,
    media: &MediaRunner,
    sources: &SourceInspector,
    task: &LeasedTask,
    progress: &ProgressSlot,
    sample_rate: u32,
    channels: u16,
) -> Result<ArtifactId, TaskExecutionError> {
    let context = source_context(database, sources, task).await?;
    let mut config = Map::new();
    config.insert("ffmpeg_bom".to_owned(), json!(FFMPEG_BOM));
    config.insert(
        "algorithm".to_owned(),
        json!("clipmill.media.audio.render.v1"),
    );
    config.insert("sample_rate".to_owned(), json!(sample_rate));
    config.insert("channels".to_owned(), json!(channels));
    let recipe = media_recipe(task, context.fingerprint, "clipmill.media.audio.v1", config)?;
    let staging = match prepare_or_hit(artifacts, recipe).await? {
        Prepared::Hit(artifact_id) => return Ok(artifact_id),
        Prepared::Staged(staging) => staging,
    };
    let staging_id = staging.id().clone();
    let result = async {
        let spec = FfmpegSpec {
            args: [
                "-i",
                &context.absolute_path,
                "-map",
                "0:a:0",
                "-vn",
                "-ac",
                &channels.to_string(),
                "-ar",
                &sample_rate.to_string(),
                "-c:a",
                "pcm_s16le",
                "-f",
                "wav",
                "audio.wav",
            ]
            .into_iter()
            .map(OsString::from)
            .collect(),
            output_dir: staging.path().to_path_buf(),
            duration_hint_millis: ticks_to_millis(context.duration_ticks),
            max_output_bytes: 4 * 1024 * 1024 * 1024,
            capture_stderr: false,
        };
        let _diagnostics = media
            .run_ffmpeg(spec, progress.clone())
            .await
            .map_err(MediaError::into_task_error)?;
        let wav = read_wav_info(&staging.path().join("audio.wav"))?;
        if wav.sample_rate != u64::from(sample_rate) || wav.channels != u64::from(channels) {
            return Err(TaskExecutionError::deterministic(
                "rendered audio format does not match the recipe",
            ));
        }
        let sample_count = wav.data_len / (2 * wav.channels.max(1));
        let duration_ticks =
            i128::from(sample_count) * TICKS_PER_SECOND / i128::from(wav.sample_rate.max(1));
        let duration_ticks = i64::try_from(duration_ticks).unwrap_or(i64::MAX);
        let descriptor = json!({
            "schema_version": "clipmill.media.audio.v1",
            "source_fingerprint": format!("sha256:{}", context.fingerprint),
            "file": "audio.wav",
            "codec": "pcm_s16le",
            "sample_rate": wav.sample_rate,
            "channels": wav.channels,
            "sample_count": sample_count,
            "duration_ticks": duration_ticks,
            "coverage": {"start_ticks": 0, "end_ticks": duration_ticks},
        });
        write_canonical_json(&staging, &artifact_path("audio.json")?, &descriptor)?;
        commit_staging(
            artifacts,
            staging_id.clone(),
            vec![artifact_path("audio.wav")?, artifact_path("audio.json")?],
        )
        .await
    }
    .await;
    if result.is_err() {
        abandon_staging(artifacts, staging_id).await;
    }
    result
}

struct WavInfo {
    sample_rate: u64,
    channels: u64,
    data_offset: u64,
    data_len: u64,
}

/// Minimal RIFF/WAVE walk for the PCM files this daemon itself rendered.
fn read_wav_info(path: &Path) -> Result<WavInfo, TaskExecutionError> {
    let mut file =
        File::open(path).map_err(|error| TaskExecutionError::transient(error.to_string()))?;
    let mut header = [0_u8; 12];
    file.read_exact(&mut header)
        .map_err(|_| TaskExecutionError::deterministic("rendered WAV is truncated"))?;
    if &header[0..4] != b"RIFF" || &header[8..12] != b"WAVE" {
        return Err(TaskExecutionError::deterministic(
            "rendered audio is not a RIFF/WAVE file",
        ));
    }
    let mut sample_rate = 0_u64;
    let mut channels = 0_u64;
    let mut offset = 12_u64;
    loop {
        let mut chunk = [0_u8; 8];
        if file.read_exact(&mut chunk).is_err() {
            return Err(TaskExecutionError::deterministic(
                "rendered WAV has no data chunk",
            ));
        }
        let chunk_len = u64::from(u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]));
        offset += 8;
        match &chunk[0..4] {
            b"fmt " => {
                let mut fmt = vec![0_u8; usize::try_from(chunk_len.min(64)).unwrap_or(16)];
                file.read_exact(&mut fmt).map_err(|_| {
                    TaskExecutionError::deterministic("rendered WAV fmt is truncated")
                })?;
                if fmt.len() >= 8 {
                    channels = u64::from(u16::from_le_bytes([fmt[2], fmt[3]]));
                    sample_rate = u64::from(u32::from_le_bytes([fmt[4], fmt[5], fmt[6], fmt[7]]));
                }
                let remaining = chunk_len.saturating_sub(fmt.len() as u64);
                file.seek(SeekFrom::Current(i64::try_from(remaining).unwrap_or(0)))
                    .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
                offset += chunk_len;
            }
            b"data" => {
                // FFmpeg streams WAV to a non-seekable-unknown length as
                // 0xFFFFFFFF; fall back to the file size when it does.
                let data_len = if chunk_len == u64::from(u32::MAX) {
                    file.metadata()
                        .map_or(0, |meta| meta.len().saturating_sub(offset))
                } else {
                    chunk_len
                };
                return Ok(WavInfo {
                    sample_rate,
                    channels,
                    data_offset: offset,
                    data_len,
                });
            }
            _ => {
                let padded = chunk_len + (chunk_len & 1);
                file.seek(SeekFrom::Current(i64::try_from(padded).unwrap_or(0)))
                    .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
                offset += padded;
            }
        }
    }
}

async fn execute_loudness(
    artifacts: &ArtifactHandle,
    media: &MediaRunner,
    task: &LeasedTask,
    progress: &ProgressSlot,
) -> Result<ArtifactId, TaskExecutionError> {
    let input_id = *task.input_artifact_ids.first().ok_or_else(|| {
        TaskExecutionError::deterministic("loudness task requires the 48 kHz rendition input")
    })?;
    let (lease, wav_path) = verified_input_file(artifacts, input_id, "audio.wav").await?;
    let descriptor = read_descriptor(&lease, "audio.json")?;
    let fingerprint = descriptor_fingerprint(&descriptor)?;
    let duration_ticks = descriptor["duration_ticks"].as_i64().unwrap_or(0);
    let mut config = Map::new();
    config.insert("ffmpeg_bom".to_owned(), json!(FFMPEG_BOM));
    config.insert(
        "algorithm".to_owned(),
        json!("clipmill.media.loudness.ebur128.v1"),
    );
    config.insert("cadence_hz".to_owned(), json!(LOUDNESS_CADENCE_HZ));
    let recipe = media_recipe(
        task,
        fingerprint,
        "clipmill.media.loudness_envelope.v1",
        config,
    )?;
    let staging = match prepare_or_hit(artifacts, recipe).await? {
        Prepared::Hit(artifact_id) => return Ok(artifact_id),
        Prepared::Staged(staging) => staging,
    };
    let staging_id = staging.id().clone();
    let result = async {
        let metadata_name = "r128.txt";
        let spec = FfmpegSpec {
            args: [
                OsString::from("-i"),
                wav_path.clone().into_os_string(),
                OsString::from("-af"),
                OsString::from(format!(
                    "ebur128=metadata=1:peak=sample,ametadata=mode=print:file={metadata_name}"
                )),
                OsString::from("-f"),
                OsString::from("null"),
                OsString::from("/dev/null"),
            ]
            .into_iter()
            .collect(),
            output_dir: staging.path().to_path_buf(),
            duration_hint_millis: ticks_to_millis(duration_ticks),
            max_output_bytes: 512 * 1024 * 1024,
            capture_stderr: false,
        };
        let _diagnostics = media
            .run_ffmpeg(spec, progress.clone())
            .await
            .map_err(MediaError::into_task_error)?;
        let metadata_path = staging.path().join(metadata_name);
        let envelope = parse_loudness_metadata(&metadata_path, fingerprint, duration_ticks)?;
        fs::remove_file(&metadata_path)
            .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
        write_canonical_json(&staging, &artifact_path("loudness.json")?, &envelope)?;
        commit_staging(
            artifacts,
            staging_id.clone(),
            vec![artifact_path("loudness.json")?],
        )
        .await
    }
    .await;
    if result.is_err() {
        abandon_staging(artifacts, staging_id).await;
    }
    result
}

fn descriptor_fingerprint(descriptor: &Value) -> Result<Sha256Digest, TaskExecutionError> {
    descriptor["source_fingerprint"]
        .as_str()
        .and_then(|value| value.strip_prefix("sha256:"))
        .and_then(|value| value.parse::<Sha256Digest>().ok())
        .ok_or_else(|| TaskExecutionError::deterministic("input descriptor fingerprint is invalid"))
}

/// Parse `ametadata=mode=print` output from the ebur128 filter into the
/// envelope document. Lines look like `frame:12 pts:57600 pts_time:1.2`
/// followed by `lavfi.r128.M=-23.5` key/value rows.
fn parse_loudness_metadata(
    path: &Path,
    fingerprint: Sha256Digest,
    duration_ticks: i64,
) -> Result<Value, TaskExecutionError> {
    let file =
        File::open(path).map_err(|error| TaskExecutionError::transient(error.to_string()))?;
    let mut reader = BufReader::new(file);
    let mut text = String::new();
    reader
        .read_to_string(&mut text)
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
    let mut points = Vec::new();
    let mut current_ticks: Option<i64> = None;
    let mut momentary: Option<f64> = None;
    let mut short_term: Option<f64> = None;
    let mut integrated: Option<f64> = None;
    let mut range: Option<f64> = None;
    let mut sample_peak: Option<f64> = None;
    let flush = |ticks: Option<i64>,
                 momentary: &mut Option<f64>,
                 short_term: &mut Option<f64>,
                 points: &mut Vec<Value>| {
        if let (Some(t_ticks), Some(m)) = (ticks, momentary.take()) {
            let mut point = json!({"t_ticks": t_ticks.max(0), "momentary_lufs": m});
            if let (Some(s), Some(object)) = (short_term.take(), point.as_object_mut()) {
                object.insert("short_term_lufs".to_owned(), json!(s));
            }
            points.push(point);
        }
    };
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("frame:") {
            flush(current_ticks, &mut momentary, &mut short_term, &mut points);
            current_ticks = rest
                .split_whitespace()
                .find_map(|field| field.strip_prefix("pts_time:"))
                .and_then(|value| crate::sources::decimal_seconds_to_ticks(value).ok())
                .and_then(|ticks| i64::try_from(ticks).ok());
        } else if let Some(value) = line.strip_prefix("lavfi.r128.M=") {
            momentary = measured_lufs(value);
        } else if let Some(value) = line.strip_prefix("lavfi.r128.S=") {
            short_term = measured_lufs(value);
        } else if let Some(value) = line.strip_prefix("lavfi.r128.I=") {
            integrated = measured_lufs(value);
        } else if let Some(value) = line.strip_prefix("lavfi.r128.LRA=") {
            range = measured_lufs(value);
        } else if let Some((key, value)) = line.split_once('=')
            && key.starts_with("lavfi.r128.sample_peaks_ch")
            && let Ok(peak) = value.trim().parse::<f64>()
        {
            let dbfs = if peak > 0.0 {
                20.0 * peak.log10()
            } else {
                -120.0
            };
            sample_peak = Some(sample_peak.map_or(dbfs, |existing: f64| existing.max(dbfs)));
        }
    }
    flush(current_ticks, &mut momentary, &mut short_term, &mut points);
    let integrated = integrated.ok_or_else(|| {
        TaskExecutionError::deterministic("ebur128 metadata carried no integrated loudness")
    })?;
    let mut summary = json!({"integrated_lufs": integrated});
    if let Some(object) = summary.as_object_mut() {
        if let Some(range) = range {
            object.insert("loudness_range_lu".to_owned(), json!(range));
        }
        if let Some(peak) = sample_peak {
            object.insert("sample_peak_dbfs".to_owned(), json!(peak));
        }
    }
    Ok(json!({
        "schema_version": "clipmill.media.loudness_envelope.v1",
        "source_fingerprint": format!("sha256:{fingerprint}"),
        "cadence_hz": LOUDNESS_CADENCE_HZ,
        "points": points,
        "summary": summary,
        "coverage": {"start_ticks": 0, "end_ticks": duration_ticks.max(0)},
    }))
}

async fn execute_reference_index(
    database: &DbHandle,
    artifacts: &ArtifactHandle,
    media: &MediaRunner,
    sources: &SourceInspector,
    task: &LeasedTask,
) -> Result<ArtifactId, TaskExecutionError> {
    let context = source_context(database, sources, task).await?;
    let mut config = Map::new();
    config.insert("ffmpeg_bom".to_owned(), json!(FFMPEG_BOM));
    config.insert(
        "algorithm".to_owned(),
        json!("clipmill.media.reference_index.demux.v1"),
    );
    config.insert("keyframe_streams".to_owned(), json!("video"));
    let recipe = media_recipe(
        task,
        context.fingerprint,
        "clipmill.media.reference_index.v1",
        config,
    )?;
    let staging = match prepare_or_hit(artifacts, recipe).await? {
        Prepared::Hit(artifact_id) => return Ok(artifact_id),
        Prepared::Staged(staging) => staging,
    };
    let staging_id = staging.id().clone();
    let result = async {
        let probed = media
            .run_ffprobe_json(
                PathBuf::from(&context.absolute_path),
                [
                    "-select_streams",
                    "v",
                    "-show_entries",
                    "packet=stream_index,pts,flags,pos",
                ]
                .into_iter()
                .map(OsString::from)
                .collect(),
            )
            .await
            .map_err(MediaError::into_task_error)?;
        let index = reference_index_document(&probed, &context)?;
        write_canonical_json(&staging, &artifact_path("reference-index.json")?, &index)?;
        commit_staging(
            artifacts,
            staging_id.clone(),
            vec![artifact_path("reference-index.json")?],
        )
        .await
    }
    .await;
    if result.is_err() {
        abandon_staging(artifacts, staging_id).await;
    }
    result
}

fn reference_index_document(
    probed: &Value,
    context: &SourceContext,
) -> Result<Value, TaskExecutionError> {
    let mut keyframes = Vec::new();
    let mut packet_counts = std::collections::BTreeMap::<u64, u64>::new();
    for packet in probed["packets"].as_array().into_iter().flatten() {
        let Some(stream_index) = packet["stream_index"].as_u64() else {
            continue;
        };
        *packet_counts.entry(stream_index).or_default() += 1;
        let flags = packet["flags"].as_str().unwrap_or("");
        if !flags.contains('K') {
            continue;
        }
        let Some(pts) = packet["pts"]
            .as_i64()
            .map(i128::from)
            .or_else(|| packet["pts"].as_str().and_then(|value| value.parse().ok()))
        else {
            continue;
        };
        let Some((_, tb_num, tb_den)) = context
            .stream_timebases
            .iter()
            .find(|(index, _, _)| *index == stream_index)
        else {
            continue;
        };
        let pts_ticks = to_edit_ticks(pts, *tb_num, *tb_den)
            .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
        let mut keyframe = json!({
            "stream_index": stream_index,
            "pts_ticks": i64::try_from(pts_ticks).unwrap_or(i64::MAX),
        });
        let byte_pos = packet["pos"]
            .as_u64()
            .or_else(|| packet["pos"].as_str().and_then(|value| value.parse().ok()));
        if let (Some(pos), Some(object)) = (byte_pos, keyframe.as_object_mut()) {
            object.insert("byte_pos".to_owned(), json!(pos));
        }
        keyframes.push(keyframe);
    }
    let streams = context
        .stream_kinds
        .iter()
        .map(|(index, kind)| {
            json!({
                "index": index,
                "kind": kind,
                "packet_count": packet_counts.get(index).copied().unwrap_or(0),
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "schema_version": "clipmill.media.reference_index.v1",
        "source_fingerprint": format!("sha256:{}", context.fingerprint),
        "streams": streams,
        "video_keyframes": keyframes,
        "coverage": {"start_ticks": 0, "end_ticks": context.duration_ticks.max(0)},
    }))
}

struct TilePlan {
    prefix: &'static str,
    height: u32,
    rate_num: u32,
    rate_den: u32,
    interval_ticks: i64,
    jpeg_quality: u32,
}

/// Shared executor for the filmstrip and analysis-frame extractions: decode
/// the proxy input at a fixed cadence into numbered JPEGs plus an index that
/// names each tile's tick.
async fn execute_tiles(
    artifacts: &ArtifactHandle,
    media: &MediaRunner,
    task: &LeasedTask,
    progress: &ProgressSlot,
    plan: TilePlan,
) -> Result<ArtifactId, TaskExecutionError> {
    let input_id = *task
        .input_artifact_ids
        .first()
        .ok_or_else(|| TaskExecutionError::deterministic("tile task requires the proxy input"))?;
    let (lease, proxy_path) = verified_input_file(artifacts, input_id, "proxy.mp4").await?;
    let descriptor = read_descriptor(&lease, "proxy.json")?;
    let fingerprint = descriptor_fingerprint(&descriptor)?;
    let duration_ticks = descriptor["duration_ticks"].as_i64().unwrap_or(0);
    let is_frames = task.kind == KIND_FRAMES;
    let mut config = Map::new();
    config.insert("ffmpeg_bom".to_owned(), json!(FFMPEG_BOM));
    if is_frames {
        config.insert("algorithm".to_owned(), json!("clipmill.media.frames.v1"));
        config.insert(
            "frame_rate".to_owned(),
            json!(format!("{}/{}", plan.rate_num, plan.rate_den)),
        );
        config.insert("frame_height".to_owned(), json!(plan.height));
        config.insert(
            "interval".to_owned(),
            json!({"start_ticks": 0, "end_ticks": duration_ticks.max(0)}),
        );
    } else {
        config.insert("algorithm".to_owned(), json!("clipmill.media.filmstrip.v1"));
        config.insert("interval_ticks".to_owned(), json!(plan.interval_ticks));
        config.insert("tile_height".to_owned(), json!(plan.height));
    }
    config.insert("jpeg_quality".to_owned(), json!(plan.jpeg_quality));
    let semantic = if is_frames {
        "clipmill.media.frames.v1"
    } else {
        "clipmill.media.filmstrip.v1"
    };
    let recipe = media_recipe(task, fingerprint, semantic, config)?;
    let staging = match prepare_or_hit(artifacts, recipe).await? {
        Prepared::Hit(artifact_id) => return Ok(artifact_id),
        Prepared::Staged(staging) => staging,
    };
    let staging_id = staging.id().clone();
    let result = async {
        let pattern = format!("{}_%05d.jpg", plan.prefix);
        let spec = FfmpegSpec {
            args: [
                OsString::from("-i"),
                proxy_path.clone().into_os_string(),
                OsString::from("-vf"),
                OsString::from(format!(
                    "fps={}/{},scale=-2:{}",
                    plan.rate_num, plan.rate_den, plan.height
                )),
                OsString::from("-q:v"),
                OsString::from(plan.jpeg_quality.to_string()),
                OsString::from("-f"),
                OsString::from("image2"),
                OsString::from(pattern),
            ]
            .into_iter()
            .collect(),
            output_dir: staging.path().to_path_buf(),
            duration_hint_millis: ticks_to_millis(duration_ticks),
            max_output_bytes: 4 * 1024 * 1024 * 1024,
            capture_stderr: false,
        };
        let _diagnostics = media
            .run_ffmpeg(spec, progress.clone())
            .await
            .map_err(MediaError::into_task_error)?;
        let (tile_names, index) = index_extracted_tiles(
            staging.path(),
            &plan,
            fingerprint,
            duration_ticks,
            is_frames,
        )?;
        let index_name = "index.json";
        write_canonical_json(&staging, &artifact_path(index_name)?, &index)?;
        let mut declared = vec![artifact_path(index_name)?];
        for name in &tile_names {
            declared.push(artifact_path(name)?);
        }
        commit_staging(artifacts, staging_id.clone(), declared).await
    }
    .await;
    if result.is_err() {
        abandon_staging(artifacts, staging_id).await;
    }
    result
}

/// Enumerate the JPEGs one tile extraction produced and build its index
/// document; each tile's tick follows its ordinal on the fixed cadence.
fn index_extracted_tiles(
    staging_path: &Path,
    plan: &TilePlan,
    fingerprint: Sha256Digest,
    duration_ticks: i64,
    is_frames: bool,
) -> Result<(Vec<String>, Value), TaskExecutionError> {
    let mut tile_names = Vec::new();
    for entry in fs::read_dir(staging_path)
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?
    {
        let entry = entry.map_err(|error| TaskExecutionError::transient(error.to_string()))?;
        if let Some(name) = entry.file_name().to_str()
            && name.starts_with(plan.prefix)
            && Path::new(name)
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("jpg"))
        {
            tile_names.push(name.to_owned());
        }
    }
    tile_names.sort();
    if tile_names.is_empty() {
        return Err(TaskExecutionError::deterministic(
            "tile extraction produced no frames",
        ));
    }
    let tiles = tile_names
        .iter()
        .enumerate()
        .map(|(ordinal, name)| {
            let t_ticks = i64::try_from(ordinal)
                .unwrap_or(i64::MAX)
                .saturating_mul(plan.interval_ticks);
            json!({"file": name, "t_ticks": t_ticks.max(0)})
        })
        .collect::<Vec<_>>();
    let index = if is_frames {
        json!({
            "schema_version": "clipmill.media.frames.v1",
            "source_fingerprint": format!("sha256:{fingerprint}"),
            "frame_rate": {"num": plan.rate_num, "den": plan.rate_den},
            "frame_height": plan.height,
            "frames": tiles,
            "coverage": {"start_ticks": 0, "end_ticks": duration_ticks.max(0)},
        })
    } else {
        json!({
            "schema_version": "clipmill.media.filmstrip.v1",
            "source_fingerprint": format!("sha256:{fingerprint}"),
            "tile_height": plan.height,
            "interval_ticks": plan.interval_ticks,
            "tiles": tiles,
            "coverage": {"start_ticks": 0, "end_ticks": duration_ticks.max(0)},
        })
    };
    Ok((tile_names, index))
}

async fn execute_audio_peaks(
    artifacts: &ArtifactHandle,
    task: &LeasedTask,
) -> Result<ArtifactId, TaskExecutionError> {
    let input_id = *task.input_artifact_ids.first().ok_or_else(|| {
        TaskExecutionError::deterministic("peaks task requires the 16 kHz rendition input")
    })?;
    let (lease, wav_path) = verified_input_file(artifacts, input_id, "audio.wav").await?;
    let descriptor = read_descriptor(&lease, "audio.json")?;
    let fingerprint = descriptor_fingerprint(&descriptor)?;
    let duration_ticks = descriptor["duration_ticks"].as_i64().unwrap_or(0);
    let mut config = Map::new();
    config.insert(
        "algorithm".to_owned(),
        json!("clipmill.media.audio_peaks.v1"),
    );
    config.insert("bucket_ticks".to_owned(), json!(PEAKS_BUCKET_TICKS));
    let recipe = media_recipe(task, fingerprint, "clipmill.media.audio_peaks.v1", config)?;
    let staging = match prepare_or_hit(artifacts, recipe).await? {
        Prepared::Hit(artifact_id) => return Ok(artifact_id),
        Prepared::Staged(staging) => staging,
    };
    let staging_id = staging.id().clone();
    let result = async {
        let peaks = tokio::task::spawn_blocking(move || compute_peaks(&wav_path))
            .await
            .map_err(|_| TaskExecutionError::transient("peaks task stopped".to_owned()))??;
        let document = json!({
            "schema_version": "clipmill.media.audio_peaks.v1",
            "source_fingerprint": format!("sha256:{fingerprint}"),
            "bucket_ticks": PEAKS_BUCKET_TICKS,
            "peaks": peaks,
            "coverage": {"start_ticks": 0, "end_ticks": duration_ticks.max(0)},
        });
        write_canonical_json(&staging, &artifact_path("peaks.json")?, &document)?;
        commit_staging(
            artifacts,
            staging_id.clone(),
            vec![artifact_path("peaks.json")?],
        )
        .await
    }
    .await;
    if result.is_err() {
        abandon_staging(artifacts, staging_id).await;
    }
    result
}

fn compute_peaks(wav_path: &Path) -> Result<Vec<Value>, TaskExecutionError> {
    let info = read_wav_info(wav_path)?;
    if info.channels != 1 || info.sample_rate != 16_000 {
        return Err(TaskExecutionError::deterministic(
            "peaks input is not the 16 kHz mono rendition",
        ));
    }
    let mut file =
        File::open(wav_path).map_err(|error| TaskExecutionError::transient(error.to_string()))?;
    file.seek(SeekFrom::Start(info.data_offset))
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
    let mut reader = BufReader::new(file.take(info.data_len));
    let mut peaks = Vec::new();
    let mut buffer = vec![0_u8; PEAKS_BUCKET_SAMPLES_16K * 2];
    loop {
        let mut filled = 0;
        while filled < buffer.len() {
            let read = reader
                .read(&mut buffer[filled..])
                .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
            if read == 0 {
                break;
            }
            filled += read;
        }
        if filled < 2 {
            break;
        }
        let mut minimum = i16::MAX;
        let mut maximum = i16::MIN;
        for sample in buffer[..filled - (filled % 2)].chunks_exact(2) {
            let value = i16::from_le_bytes([sample[0], sample[1]]);
            minimum = minimum.min(value);
            maximum = maximum.max(value);
        }
        peaks.push(json!({"min": minimum, "max": maximum}));
        if filled < buffer.len() {
            break;
        }
    }
    Ok(peaks)
}

async fn execute_ingest_manifest(
    artifacts: &ArtifactHandle,
    task: &LeasedTask,
) -> Result<ArtifactId, TaskExecutionError> {
    if task.input_artifact_ids.is_empty() {
        return Err(TaskExecutionError::deterministic(
            "ingest manifest requires at least one derivative input",
        ));
    }
    let mut children = Vec::new();
    let mut fingerprint: Option<String> = None;
    for artifact_id in &task.input_artifact_ids {
        let lease = artifacts
            .open(*artifact_id)
            .await
            .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
        children.push(json!({
            "kind": lease.kind(),
            "artifact_id": artifact_id.to_string(),
        }));
        drop(lease);
    }
    // The manifest's own fingerprint comes from the shared source; every
    // child recipe already pinned it, so read it from the first descriptor
    // available (all children agree or their recipes would differ).
    for (artifact_id, candidate) in task.input_artifact_ids.iter().zip(children.iter()) {
        let kind = candidate["kind"].as_str().unwrap_or("");
        let descriptor_file = match kind {
            "media.proxy.v1" => Some("proxy.json"),
            "media.audio_16k.v1" | "media.audio_48k.v1" => Some("audio.json"),
            "media.loudness_envelope.v1" => Some("loudness.json"),
            "media.reference_index.v1" => Some("reference-index.json"),
            "media.filmstrip.v1" | "media.frames.v1" => Some("index.json"),
            "media.audio_peaks.v1" => Some("peaks.json"),
            _ => None,
        };
        if let Some(file) = descriptor_file {
            let lease = artifacts
                .open(*artifact_id)
                .await
                .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
            if let Ok(descriptor) = read_descriptor(&lease, file)
                && let Some(value) = descriptor["source_fingerprint"].as_str()
            {
                fingerprint = Some(value.to_owned());
                break;
            }
        }
    }
    let fingerprint = fingerprint.ok_or_else(|| {
        TaskExecutionError::deterministic("no ingest derivative carries a source fingerprint")
    })?;
    let digest = fingerprint
        .strip_prefix("sha256:")
        .and_then(|value| value.parse::<Sha256Digest>().ok())
        .ok_or_else(|| TaskExecutionError::deterministic("derivative fingerprint is invalid"))?;
    let mut config = Map::new();
    config.insert(
        "algorithm".to_owned(),
        json!("clipmill.media.ingest_manifest.v1"),
    );
    let recipe = media_recipe(task, digest, "clipmill.media.ingest_manifest.v1", config)?;
    let staging = match prepare_or_hit(artifacts, recipe).await? {
        Prepared::Hit(artifact_id) => return Ok(artifact_id),
        Prepared::Staged(staging) => staging,
    };
    let staging_id = staging.id().clone();
    let document = json!({
        "schema_version": "clipmill.media.ingest_manifest.v1",
        "source_fingerprint": fingerprint,
        "children": children,
    });
    let result = async {
        write_canonical_json(&staging, &artifact_path("ingest-manifest.json")?, &document)?;
        commit_staging(
            artifacts,
            staging_id.clone(),
            vec![artifact_path("ingest-manifest.json")?],
        )
        .await
    }
    .await;
    if result.is_err() {
        abandon_staging(artifacts, staging_id).await;
    }
    result
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use std::fs;

    use clipmill_core::Sha256Digest;

    use super::{
        ProgressSlot, directory_bytes, ffmpeg_sibling, parse_loudness_metadata,
        read_progress_millis, read_wav_info, ticks_to_millis,
    };

    /// A gated window must not become `null` in the document.
    ///
    /// EBU R128 does not report loudness for a window below the gate; ffmpeg
    /// says `-inf`, Rust parses that to negative infinity, and `serde_json`
    /// writes non-finite floats as `null`. The contract requires a number
    /// there, so the whole analysis of any recording containing a quiet
    /// passage failed at the first consumer that parsed strictly — which is
    /// every real recording, and none of the silent test fixtures.
    #[test]
    fn a_gated_loudness_window_lands_on_the_floor_rather_than_null() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("loudness.txt");
        fs::write(
            &path,
            "frame:0 pts:0 pts_time:0\n\
             lavfi.r128.M=-inf\n\
             lavfi.r128.S=-18.4\n\
             frame:1 pts:9000 pts_time:0.1\n\
             lavfi.r128.M=-23.5\n\
             lavfi.r128.S=-inf\n\
             lavfi.r128.I=-18.5\n\
             lavfi.r128.LRA=1.7\n",
        )
        .expect("write metadata");

        let document = parse_loudness_metadata(&path, Sha256Digest::from_bytes([7; 32]), 90_000)
            .expect("the envelope parses");
        let points = document["points"].as_array().expect("points");
        assert_eq!(points.len(), 2);
        for point in points {
            for field in ["momentary_lufs", "short_term_lufs"] {
                let value = &point[field];
                assert!(
                    value.is_null() || value.as_f64().is_some(),
                    "{field} is neither absent nor a number: {value}"
                );
                if let Some(number) = value.as_f64() {
                    assert!(number.is_finite(), "{field} serialized a non-finite value");
                }
            }
        }
        assert_eq!(points[0]["momentary_lufs"].as_f64(), Some(-120.0));
        assert_eq!(points[1]["short_term_lufs"].as_f64(), Some(-120.0));
        // Serializing is where the failure actually appeared, so the assertion
        // is made against the bytes a consumer receives rather than the tree.
        let encoded = serde_json::to_string(&document).expect("serialize");
        assert!(
            !encoded.contains("null"),
            "the document still carries a null: {encoded}"
        );
    }

    #[test]
    fn ffmpeg_path_derives_from_the_ffprobe_sibling() {
        assert_eq!(
            ffmpeg_sibling(std::path::Path::new("/opt/bin/ffprobe")),
            std::path::PathBuf::from("/opt/bin/ffmpeg")
        );
        assert_eq!(
            ffmpeg_sibling(std::path::Path::new("/opt/bin/ffprobe-8.1")),
            std::path::PathBuf::from("/opt/bin/ffmpeg-8.1")
        );
    }

    #[test]
    fn progress_parsing_reads_the_last_out_time() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let path = temp.path().join("progress.txt");
        fs::write(
            &path,
            "frame=1\nout_time_us=500000\nprogress=continue\nframe=2\nout_time_us=1500000\nprogress=continue\n",
        )
        .expect("write progress");
        assert_eq!(read_progress_millis(&path), Some(1_500));
        fs::write(&path, "garbage\n").expect("overwrite");
        assert_eq!(read_progress_millis(&path), None);
    }

    #[test]
    fn progress_slot_takes_latest_once() {
        let slot = ProgressSlot::default();
        assert!(slot.take().is_none());
        slot.set("media_millis", 10, 100);
        slot.set("media_millis", 20, 100);
        let taken = slot.take().expect("latest progress");
        assert_eq!((taken.done, taken.total), (20, 100));
        assert!(slot.take().is_none());
    }

    #[test]
    fn tick_conversion_matches_ninety_per_milli() {
        assert_eq!(ticks_to_millis(90_000), 1_000);
        assert_eq!(ticks_to_millis(45), 0);
        assert_eq!(ticks_to_millis(-90_000), 0);
    }

    #[test]
    fn wav_parsing_reads_the_daemon_rendered_layout() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let path = temp.path().join("audio.wav");
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&36_u32.to_le_bytes());
        bytes.extend_from_slice(b"WAVE");
        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&16_u32.to_le_bytes());
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&16_000_u32.to_le_bytes());
        bytes.extend_from_slice(&32_000_u32.to_le_bytes());
        bytes.extend_from_slice(&2_u16.to_le_bytes());
        bytes.extend_from_slice(&16_u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&8_u32.to_le_bytes());
        bytes.extend_from_slice(&[0, 0, 255, 127, 0, 128, 1, 0]);
        fs::write(&path, &bytes).expect("write wav");
        let info = read_wav_info(&path).expect("parse wav");
        assert_eq!(info.sample_rate, 16_000);
        assert_eq!(info.channels, 1);
        assert_eq!(info.data_len, 8);
        let peaks = super::compute_peaks(&path).expect("peaks");
        assert_eq!(peaks.len(), 1);
        assert_eq!(peaks[0]["min"], -32_768);
        assert_eq!(peaks[0]["max"], 32_767);
    }

    #[test]
    fn directory_budget_sums_nested_payloads() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        fs::write(temp.path().join("a.bin"), vec![0_u8; 100]).expect("write");
        fs::create_dir(temp.path().join("nested")).expect("mkdir");
        fs::write(temp.path().join("nested/b.bin"), vec![0_u8; 50]).expect("write");
        assert_eq!(directory_bytes(temp.path()), 150);
    }
}
