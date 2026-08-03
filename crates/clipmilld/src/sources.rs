use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader},
    io::{Read, Seek, SeekFrom},
    os::unix::fs::{FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Arc,
    thread,
    time::{Duration, Instant, UNIX_EPOCH},
};

use clipmill_core::Sha256Digest;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::sync::Semaphore;
use ulid::Ulid;

const SMALL_FILE_LIMIT: u64 = 16 * 1024 * 1024;
const EDGE_SAMPLE_BYTES: u64 = 1024 * 1024;
const WINDOW_BYTES: u64 = 256 * 1024;
const INTERIOR_WINDOWS: u64 = 16;
const PROBE_TIMEOUT: Duration = Duration::from_secs(15);
/// How long the packet pass may take.
///
/// Longer than the metadata probe on purpose, and for a reason that is not
/// slack: reading every packet means demuxing the whole file, so the work is
/// proportional to its size where reading a header is not. Five minutes covers
/// a multi-gigabyte recording on a slow disk and still refuses a file that
/// never ends.
const PACKET_TIMEOUT: Duration = Duration::from_mins(5);
const TERMINATE_GRACE: Duration = Duration::from_millis(500);
const MAX_STDOUT_BYTES: u64 = 16 * 1024 * 1024;
/// Packets a single probe may read before the file is treated as hostile.
///
/// Not a length limit. Ten million packets is roughly six hours of video plus
/// its audio, and a file claiming more than that in a container we are about to
/// hand a decoder is not a recording somebody made.
const MAX_PACKETS: u64 = 10_000_000;
/// Discontinuities a single stream may have before the same conclusion.
///
/// A normal recording has one run per stream. A concatenation or a recovered
/// file has a handful. Sixty-five thousand is a file whose timeline is noise,
/// and the source map built from it would be noise too.
const MAX_RUNS_PER_STREAM: usize = 65_536;
const MAX_STDERR_BYTES: u64 = 256 * 1024;
const SOURCE_FINGERPRINT_DOMAIN: &[u8] = b"clipmill.source.fingerprint.v1\0";
const SOURCE_SAMPLE_DOMAIN: &[u8] = b"clipmill.source.sample.v1\0";

#[derive(Clone, Debug)]
pub(crate) struct SourceInspector {
    ffprobe: PathBuf,
    scratch: PathBuf,
    permits: Arc<Semaphore>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FileObservation {
    pub absolute_path: String,
    pub byte_size: u64,
    pub sample_sha256: String,
    pub device_id: u64,
    pub inode: u64,
    pub modified_unix_nanos: u64,
}

#[derive(Debug)]
pub(crate) struct SampledSource {
    observation: FileObservation,
    identity: FileIdentity,
    path: PathBuf,
    samples: Vec<SampleWindow>,
}

impl SampledSource {
    #[must_use]
    pub(crate) fn observation(&self) -> &FileObservation {
        &self.observation
    }
}

#[derive(Debug)]
pub(crate) struct InspectedSource {
    pub observation: FileObservation,
    pub source_fingerprint: String,
    pub source_map_json: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FileIdentity {
    device_id: u64,
    inode: u64,
    byte_size: u64,
    modified_unix_nanos: u64,
}

#[derive(Debug)]
struct SampleWindow {
    offset: u64,
    bytes: Vec<u8>,
}

#[derive(Debug, Error)]
pub(crate) enum SourceProbeError {
    #[error("invalid local source: {0}")]
    InvalidPath(&'static str),
    #[error("cannot inspect local source: {0}")]
    Io(String),
    #[error("FFprobe exceeded its 15-second deadline")]
    Timeout,
    #[error("FFprobe output exceeded its bounded limit")]
    OutputLimit,
    #[error("FFprobe rejected the source: {0}")]
    ProbeFailed(String),
    #[error("FFprobe returned invalid source metadata: {0}")]
    InvalidProbe(&'static str),
    #[error("SOURCE_CHANGED: the registered file changed during or after observation")]
    SourceChanged,
    #[error("source inspection task stopped unexpectedly")]
    Stopped,
}

impl SourceInspector {
    pub(crate) fn new(ffprobe: PathBuf, scratch: PathBuf) -> Result<Self, SourceProbeError> {
        fs::create_dir_all(&scratch).map_err(io_error)?;
        fs::set_permissions(&scratch, fs::Permissions::from_mode(0o700)).map_err(io_error)?;
        recover_probe_scratch(&scratch)?;
        Ok(Self {
            ffprobe,
            scratch,
            permits: Arc::new(Semaphore::new(4)),
        })
    }

    pub(crate) async fn sample(&self, value: String) -> Result<SampledSource, SourceProbeError> {
        let permit = Arc::clone(&self.permits)
            .acquire_owned()
            .await
            .map_err(|_| SourceProbeError::Stopped)?;
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            sample_source(&value)
        })
        .await
        .map_err(|_| SourceProbeError::Stopped)?
    }

    pub(crate) async fn complete(
        &self,
        sampled: SampledSource,
    ) -> Result<InspectedSource, SourceProbeError> {
        let ffprobe = self.ffprobe.clone();
        let scratch = self.scratch.clone();
        let permit = Arc::clone(&self.permits)
            .acquire_owned()
            .await
            .map_err(|_| SourceProbeError::Stopped)?;
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            complete_inspection(&ffprobe, &scratch, sampled)
        })
        .await
        .map_err(|_| SourceProbeError::Stopped)?
    }

    pub(crate) async fn verify(&self, expected: &FileObservation) -> Result<(), SourceProbeError> {
        let sampled = self.sample(expected.absolute_path.clone()).await?;
        if sampled.observation.byte_size != expected.byte_size
            || sampled.observation.sample_sha256 != expected.sample_sha256
        {
            return Err(SourceProbeError::SourceChanged);
        }
        Ok(())
    }
}

fn recover_probe_scratch(scratch: &Path) -> Result<(), SourceProbeError> {
    for entry in fs::read_dir(scratch).map_err(io_error)? {
        let entry = entry.map_err(io_error)?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(io_error)?;
        if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
            fs::remove_dir_all(&path).map_err(io_error)?;
        } else {
            fs::remove_file(&path).map_err(io_error)?;
        }
    }
    Ok(())
}

fn sample_source(value: &str) -> Result<SampledSource, SourceProbeError> {
    if value.is_empty() || value.contains('\0') {
        return Err(SourceProbeError::InvalidPath(
            "path is empty or contains NUL",
        ));
    }
    if value.contains("://") || value.starts_with("file:") {
        return Err(SourceProbeError::InvalidPath("URLs are not accepted"));
    }
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err(SourceProbeError::InvalidPath("path must be absolute"));
    }
    let link_metadata = fs::symlink_metadata(&path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            SourceProbeError::InvalidPath("path does not exist")
        } else {
            io_error(error)
        }
    })?;
    if link_metadata.file_type().is_symlink() {
        return Err(SourceProbeError::InvalidPath("symlinks are not accepted"));
    }
    if link_metadata.file_type().is_dir()
        || link_metadata.file_type().is_block_device()
        || link_metadata.file_type().is_char_device()
        || link_metadata.file_type().is_fifo()
        || link_metadata.file_type().is_socket()
        || !link_metadata.file_type().is_file()
    {
        return Err(SourceProbeError::InvalidPath(
            "path must name a regular file",
        ));
    }
    let canonical = fs::canonicalize(&path).map_err(io_error)?;
    let absolute_path = canonical
        .to_str()
        .ok_or(SourceProbeError::InvalidPath("path must be valid UTF-8"))?
        .to_owned();
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(nonblocking_flag() | nofollow_flag())
        .open(&canonical)
        .map_err(io_error)?;
    let before = identity(&file.metadata().map_err(io_error)?)?;
    let samples = read_sample_windows(&mut file, before.byte_size)?;
    let after = identity(&file.metadata().map_err(io_error)?)?;
    let path_after = identity(&fs::metadata(&canonical).map_err(io_error)?)?;
    if before != after || before != path_after {
        return Err(SourceProbeError::SourceChanged);
    }
    let sample_sha256 = digest_samples(SOURCE_SAMPLE_DOMAIN, before.byte_size, &samples, None);
    Ok(SampledSource {
        observation: FileObservation {
            absolute_path,
            byte_size: before.byte_size,
            sample_sha256: format!("sha256:{sample_sha256}"),
            device_id: before.device_id,
            inode: before.inode,
            modified_unix_nanos: before.modified_unix_nanos,
        },
        identity: before,
        path: canonical,
        samples,
    })
}

fn complete_inspection(
    ffprobe: &Path,
    scratch: &Path,
    sampled: SampledSource,
) -> Result<InspectedSource, SourceProbeError> {
    let (raw_probe, packet_runs) = run_ffprobe(ffprobe, scratch, &sampled.path)?;
    let path_after = identity(&fs::metadata(&sampled.path).map_err(io_error)?)?;
    if path_after != sampled.identity {
        return Err(SourceProbeError::SourceChanged);
    }
    let metadata = normalize_probe(&raw_probe, &packet_runs)?;
    let canonical_metadata = serde_json_canonicalizer::to_vec(&metadata)
        .map_err(|_| SourceProbeError::InvalidProbe("metadata is not canonical JSON"))?;
    let fingerprint = digest_samples(
        SOURCE_FINGERPRINT_DOMAIN,
        sampled.identity.byte_size,
        &sampled.samples,
        Some(&canonical_metadata),
    );
    let source_fingerprint = format!("sha256:{fingerprint}");
    let mut source_map = metadata;
    let object = source_map
        .as_object_mut()
        .ok_or(SourceProbeError::InvalidProbe(
            "normalized metadata is not an object",
        ))?;
    object.insert(
        "schema_version".to_owned(),
        Value::String("clipmill.source_map.v1".to_owned()),
    );
    object.insert(
        "source_fingerprint".to_owned(),
        Value::String(source_fingerprint.clone()),
    );
    let source_map_json = serde_json_canonicalizer::to_vec(&source_map)
        .map_err(|_| SourceProbeError::InvalidProbe("source map is not canonical JSON"))?;
    Ok(InspectedSource {
        observation: sampled.observation,
        source_fingerprint,
        source_map_json,
    })
}

fn read_sample_windows(
    file: &mut File,
    byte_size: u64,
) -> Result<Vec<SampleWindow>, SourceProbeError> {
    let spans = sample_spans(byte_size);
    let mut windows = Vec::with_capacity(spans.len());
    for (offset, length) in spans {
        file.seek(SeekFrom::Start(offset)).map_err(io_error)?;
        let length = usize::try_from(length)
            .map_err(|_| SourceProbeError::InvalidPath("source sample is too large"))?;
        let mut bytes = vec![0; length];
        file.read_exact(&mut bytes).map_err(io_error)?;
        windows.push(SampleWindow { offset, bytes });
    }
    Ok(windows)
}

fn sample_spans(byte_size: u64) -> Vec<(u64, u64)> {
    if byte_size <= SMALL_FILE_LIMIT {
        return vec![(0, byte_size)];
    }
    let last = byte_size - EDGE_SAMPLE_BYTES;
    let interior_start = EDGE_SAMPLE_BYTES;
    let interior_end = last.saturating_sub(WINDOW_BYTES);
    let mut spans = Vec::with_capacity(usize::try_from(INTERIOR_WINDOWS).unwrap_or(16) + 2);
    spans.push((0, EDGE_SAMPLE_BYTES));
    for index in 1..=INTERIOR_WINDOWS {
        let offset =
            interior_start + (interior_end - interior_start) * index / (INTERIOR_WINDOWS + 1);
        spans.push((offset, WINDOW_BYTES));
    }
    spans.push((last, EDGE_SAMPLE_BYTES));
    spans
}

fn digest_samples(
    domain: &[u8],
    byte_size: u64,
    samples: &[SampleWindow],
    metadata: Option<&[u8]>,
) -> Sha256Digest {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(byte_size.to_be_bytes());
    for sample in samples {
        hasher.update(sample.offset.to_be_bytes());
        hasher.update(
            u64::try_from(sample.bytes.len())
                .unwrap_or(u64::MAX)
                .to_be_bytes(),
        );
        hasher.update(&sample.bytes);
    }
    if let Some(metadata) = metadata {
        hasher.update(
            u64::try_from(metadata.len())
                .unwrap_or(u64::MAX)
                .to_be_bytes(),
        );
        hasher.update(metadata);
    }
    Sha256Digest::from_bytes(hasher.finalize().into())
}

fn identity(metadata: &fs::Metadata) -> Result<FileIdentity, SourceProbeError> {
    if !metadata.file_type().is_file() {
        return Err(SourceProbeError::InvalidPath(
            "source stopped being a regular file",
        ));
    }
    let modified = metadata.modified().map_err(io_error)?;
    let modified_unix_nanos = modified
        .duration_since(UNIX_EPOCH)
        .map_err(|_| SourceProbeError::InvalidPath("source modification time predates Unix epoch"))?
        .as_nanos();
    Ok(FileIdentity {
        device_id: metadata.dev(),
        inode: metadata.ino(),
        byte_size: metadata.len(),
        modified_unix_nanos: u64::try_from(modified_unix_nanos).unwrap_or(u64::MAX),
    })
}

fn run_ffprobe(
    ffprobe: &Path,
    scratch: &Path,
    source: &Path,
) -> Result<(Value, BTreeMap<u64, StreamRuns>), SourceProbeError> {
    run_ffprobe_with_timeout(ffprobe, scratch, source, PROBE_TIMEOUT)
}

fn run_ffprobe_with_timeout(
    ffprobe: &Path,
    scratch: &Path,
    source: &Path,
    probe_timeout: Duration,
) -> Result<(Value, BTreeMap<u64, StreamRuns>), SourceProbeError> {
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
        .arg("json")
        .arg("-show_format")
        .arg("-show_streams")
        .arg("-show_chapters")
        .arg(source)
        .current_dir(&work)
        .env_clear()
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    let mut child = command.spawn().map_err(io_error)?;
    let deadline = Instant::now() + probe_timeout;
    let status = loop {
        if let Some(status) = child.try_wait().map_err(io_error)? {
            break status;
        }
        if Instant::now() >= deadline {
            terminate_child(&mut child);
            return Err(SourceProbeError::Timeout);
        }
        thread::sleep(Duration::from_millis(10));
    };
    let stdout_size = fs::metadata(&stdout_path).map_err(io_error)?.len();
    let stderr_size = fs::metadata(&stderr_path).map_err(io_error)?.len();
    if stdout_size > MAX_STDOUT_BYTES || stderr_size > MAX_STDERR_BYTES {
        return Err(SourceProbeError::OutputLimit);
    }
    let stderr = fs::read_to_string(&stderr_path).map_err(io_error)?;
    if !status.success() {
        return Err(SourceProbeError::ProbeFailed(sanitize_detail(&stderr)));
    }
    let bytes = fs::read(&stdout_path).map_err(io_error)?;
    let metadata: Value = serde_json::from_slice(&bytes)
        .map_err(|_| SourceProbeError::InvalidProbe("output is not valid JSON"))?;

    // A second invocation, streamed. Kept separate from the metadata read so a
    // long recording can never cost the container's headers: the two answers
    // have very different sizes and only one of them grows with duration.
    let packets = stream_packets(ffprobe, &work, source, Instant::now() + PACKET_TIMEOUT)?;
    Ok((metadata, packets))
}

/// What a stream's packets amount to, kept instead of the packets themselves.
///
/// This is the whole point of reading packets as a stream. The mapping only
/// ever asks two things of them — where the timeline jumps backwards, and
/// whether more than one packet duration occurs — and both are folds. Holding
/// the answers costs one entry per discontinuity, which for an ordinary
/// recording is one; holding the packets costs an entry per frame, which for an
/// hour is over a million and for the JSON that carried them was ninety
/// megabytes.
#[derive(Debug, Default)]
pub(crate) struct StreamRuns {
    /// Contiguous `(source_start, source_end)` spans, in order.
    runs: Vec<(i128, i128)>,
    /// Distinct positive packet durations, counted no further than two —
    /// variable frame rate is "more than one duration occurs", and the third
    /// one answers nothing the second did not.
    distinct_durations: BTreeSet<i128>,
}

impl StreamRuns {
    /// Fold one packet in, exactly as the old walk over the collected vector
    /// did: extend the open run while time moves forward, and start a new one
    /// when it jumps back.
    fn observe(&mut self, pts: i128, duration: i128) -> Result<(), SourceProbeError> {
        if duration > 0 && self.distinct_durations.len() < 2 {
            self.distinct_durations.insert(duration);
        }
        let end = pts.saturating_add(duration.max(1));
        match self.runs.last_mut() {
            Some((_, source_end)) if pts >= *source_end => {
                *source_end = (*source_end).max(end);
            }
            Some(_) => {
                if self.runs.len() >= MAX_RUNS_PER_STREAM {
                    return Err(SourceProbeError::OutputLimit);
                }
                self.runs.push((pts, end));
            }
            None => self.runs.push((pts, end)),
        }
        Ok(())
    }

    fn is_variable(&self) -> bool {
        self.distinct_durations.len() > 1
    }

    fn first_pts(&self) -> Option<i128> {
        self.runs.first().map(|(start, _)| *start)
    }
}

/// Read every packet's stream, timestamp and duration, keeping none of them.
///
/// Two things make an hour-long recording work where it did not before. The
/// fields are named, so a packet costs about fifteen bytes of CSV instead of
/// three hundred and seventy of JSON; and the output is consumed as it arrives
/// and folded into [`StreamRuns`], so nothing grows with the length of the
/// recording. The old path wrote the whole of ffprobe's JSON to a file and
/// refused anything over sixteen megabytes, which is about ten minutes.
///
/// The bounds that remain are bounds on *nonsense* rather than on length: a
/// packet count no recording reaches, and a discontinuity count no timeline
/// has. Neither is reachable by making a video longer.
fn stream_packets(
    ffprobe: &Path,
    work: &Path,
    source: &Path,
    deadline: Instant,
) -> Result<BTreeMap<u64, StreamRuns>, SourceProbeError> {
    let stderr_path = work.join("packets-stderr.txt");
    let stderr = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&stderr_path)
        .map_err(io_error)?;
    let mut child = Command::new(ffprobe)
        .arg("-v")
        .arg("error")
        .arg("-protocol_whitelist")
        .arg("file,pipe")
        // Three fields, no keys, no JSON. Everything the fold reads and
        // nothing it does not.
        .arg("-show_entries")
        .arg("packet=stream_index,pts,duration")
        .arg("-of")
        .arg("csv=p=0")
        .arg(source)
        .current_dir(work)
        .env_clear()
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::from(stderr))
        .spawn()
        .map_err(io_error)?;

    let mut runs = BTreeMap::<u64, StreamRuns>::new();
    let outcome = (|| -> Result<(), SourceProbeError> {
        let stdout = child
            .stdout
            .take()
            .ok_or(SourceProbeError::InvalidProbe("ffprobe produced no output"))?;
        let mut seen = 0_u64;
        for line in BufReader::new(stdout).lines() {
            let line = line.map_err(io_error)?;
            if Instant::now() >= deadline {
                return Err(SourceProbeError::Timeout);
            }
            seen += 1;
            if seen > MAX_PACKETS {
                return Err(SourceProbeError::OutputLimit);
            }
            let mut fields = line.trim().split(',');
            let (Some(index), Some(pts), Some(duration)) =
                (fields.next(), fields.next(), fields.next())
            else {
                continue;
            };
            // A packet with no timestamp cannot place anything on a timeline,
            // and the old path skipped it for the same reason. `N/A` is how
            // ffprobe spells that here.
            let (Ok(index), Ok(pts)) = (index.parse::<u64>(), pts.parse::<i128>()) else {
                continue;
            };
            let duration = duration.parse::<i128>().unwrap_or(0).max(0);
            runs.entry(index).or_default().observe(pts, duration)?;
        }
        Ok(())
    })();

    let status = match outcome {
        Ok(()) => wait_bounded(&mut child, deadline)?,
        Err(error) => {
            terminate_child(&mut child);
            return Err(error);
        }
    };
    if fs::metadata(&stderr_path).map_err(io_error)?.len() > MAX_STDERR_BYTES {
        return Err(SourceProbeError::OutputLimit);
    }
    if !status.success() {
        let stderr = fs::read_to_string(&stderr_path).map_err(io_error)?;
        return Err(SourceProbeError::ProbeFailed(sanitize_detail(&stderr)));
    }
    Ok(runs)
}

/// Wait for a child, giving up at the deadline the probe was granted.
fn wait_bounded(
    child: &mut std::process::Child,
    deadline: Instant,
) -> Result<std::process::ExitStatus, SourceProbeError> {
    loop {
        if let Some(status) = child.try_wait().map_err(io_error)? {
            return Ok(status);
        }
        if Instant::now() >= deadline {
            terminate_child(child);
            return Err(SourceProbeError::Timeout);
        }
        thread::sleep(Duration::from_millis(10));
    }
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

#[allow(clippy::too_many_lines)]
fn normalize_probe(
    probe: &Value,
    packets: &BTreeMap<u64, StreamRuns>,
) -> Result<Value, SourceProbeError> {
    let raw_streams = probe
        .get("streams")
        .and_then(Value::as_array)
        .ok_or(SourceProbeError::InvalidProbe("streams are missing"))?;
    if raw_streams.is_empty() {
        return Err(SourceProbeError::InvalidProbe("source has no streams"));
    }
    let mut streams = Vec::with_capacity(raw_streams.len());
    let mut timebases = BTreeMap::<u64, (i128, i128)>::new();
    let mut rotations = BTreeSet::new();
    for raw in raw_streams {
        let index = json_u64(raw.get("index"))
            .ok_or(SourceProbeError::InvalidProbe("stream index is missing"))?;
        let kind =
            raw.get("codec_type")
                .and_then(Value::as_str)
                .map_or("data", |value| match value {
                    "video" | "audio" | "subtitle" | "data" => value,
                    _ => "data",
                });
        let codec = raw
            .get("codec_name")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let (tb_num, tb_den) = parse_ratio(
            raw.get("time_base")
                .and_then(Value::as_str)
                .ok_or(SourceProbeError::InvalidProbe("stream timebase is missing"))?,
        )?;
        timebases.insert(index, (tb_num, tb_den));
        let mut stream = Map::new();
        stream.insert("index".to_owned(), json!(index));
        stream.insert("kind".to_owned(), json!(kind));
        stream.insert("codec".to_owned(), json!(codec));
        stream.insert("timebase".to_owned(), json!({"num": tb_num, "den": tb_den}));
        if let Some(start) = json_i128(raw.get("start_pts")) {
            stream.insert(
                "start_offset_ticks".to_owned(),
                json!(to_edit_ticks(start, tb_num, tb_den)?),
            );
        }
        if let Some(duration) = json_i128(raw.get("duration_ts")) {
            stream.insert(
                "duration_ticks".to_owned(),
                json!(to_edit_ticks(duration.max(0), tb_num, tb_den)?),
            );
        }
        let rotation = rotation(raw);
        rotations.insert(rotation);
        if kind == "video" {
            let width = json_u64(raw.get("width")).unwrap_or(1).max(1);
            let height = json_u64(raw.get("height")).unwrap_or(1).max(1);
            let (display_width, display_height) = if matches!(rotation, 90 | 270) {
                (height, width)
            } else {
                (width, height)
            };
            let mut video = Map::new();
            video.insert("coded_width".to_owned(), json!(width));
            video.insert("coded_height".to_owned(), json!(height));
            video.insert("display_width".to_owned(), json!(display_width));
            video.insert("display_height".to_owned(), json!(display_height));
            if let Some(rate) = raw.get("avg_frame_rate").and_then(Value::as_str)
                && let Ok((num, den)) = parse_ratio(rate)
                && num > 0
            {
                video.insert("frame_rate".to_owned(), json!({"num": num, "den": den}));
            }
            let rates_differ = raw.get("avg_frame_rate") != raw.get("r_frame_rate");
            let varies = packets.get(&index).is_some_and(StreamRuns::is_variable);
            video.insert("vfr".to_owned(), Value::Bool(rates_differ || varies));
            if let Some(color) = color_metadata(raw) {
                video.insert("color".to_owned(), color);
            }
            if let Some(hdr) = hdr_metadata(raw) {
                video.insert("hdr".to_owned(), hdr);
            }
            stream.insert("video".to_owned(), Value::Object(video));
        } else if kind == "audio" {
            let sample_rate = raw
                .get("sample_rate")
                .and_then(json_string_u64)
                .unwrap_or(1)
                .max(1);
            let channels = json_u64(raw.get("channels")).unwrap_or(1).max(1);
            let mut audio = Map::new();
            audio.insert("sample_rate".to_owned(), json!(sample_rate));
            audio.insert("channels".to_owned(), json!(channels));
            if let Some(layout) = raw.get("channel_layout").and_then(Value::as_str) {
                audio.insert("layout".to_owned(), json!(layout));
            }
            stream.insert("audio".to_owned(), Value::Object(audio));
        }
        streams.push(Value::Object(stream));
    }

    let mapping = build_mapping(raw_streams, packets, &timebases)?;
    let format = probe.get("format").and_then(Value::as_object);
    let format_name = format
        .and_then(|value| value.get("format_name"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let duration_ticks = format
        .and_then(|value| value.get("duration"))
        .and_then(Value::as_str)
        .map(decimal_seconds_to_ticks)
        .transpose()?
        .unwrap_or_else(|| mapping_duration(&mapping));
    let mut container = Map::new();
    container.insert("format".to_owned(), json!(format_name));
    container.insert("duration_ticks".to_owned(), json!(duration_ticks.max(0)));
    container.insert(
        "rotation_deg".to_owned(),
        json!(
            rotations
                .iter()
                .copied()
                .find(|value| *value != 0)
                .unwrap_or(0)
        ),
    );

    Ok(json!({
        "chapters": normalize_chapters(probe.get("chapters"))?,
        "container": container,
        "mapping": mapping,
        "streams": streams,
    }))
}

fn build_mapping(
    raw_streams: &[Value],
    packets: &BTreeMap<u64, StreamRuns>,
    timebases: &BTreeMap<u64, (i128, i128)>,
) -> Result<Value, SourceProbeError> {
    let mut initial_edits = Vec::new();
    for (index, stream) in packets {
        if let (Some(first), Some((num, den))) = (stream.first_pts(), timebases.get(index)) {
            initial_edits.push(to_edit_ticks(first, *num, *den)?);
        }
    }
    let global_origin = initial_edits.into_iter().min().unwrap_or(0);
    let mut segments = Vec::new();
    for raw in raw_streams {
        let index = json_u64(raw.get("index"))
            .ok_or(SourceProbeError::InvalidProbe("stream index is missing"))?;
        let (num, den) = timebases
            .get(&index)
            .copied()
            .ok_or(SourceProbeError::InvalidProbe("stream timebase is missing"))?;
        // A stream the container declared but never carried a packet for
        // still needs a segment, from what the header said about it.
        let mut runs = packets
            .get(&index)
            .map(|stream| stream.runs.clone())
            .unwrap_or_default();
        if runs.is_empty() {
            let start = json_i128(raw.get("start_pts")).unwrap_or(0);
            let duration = json_i128(raw.get("duration_ts")).unwrap_or(1).max(1);
            runs.push((start, start.saturating_add(duration)));
        }
        let mut edit_start = to_edit_ticks(runs[0].0, num, den)? - global_origin;
        for (segment_number, (source_start, source_end)) in runs.into_iter().enumerate() {
            let edit_end = edit_start + to_edit_ticks(source_end - source_start, num, den)?;
            segments.push(mapping_segment(
                index,
                segment_number as u64,
                source_start,
                source_end,
                edit_start.max(0),
                edit_end.max(edit_start),
            ));
            edit_start = edit_end;
        }
    }
    segments.sort_by_key(|value| {
        (
            value["edit_start_ticks"].as_i64().unwrap_or(0),
            value["stream_index"].as_u64().unwrap_or(0),
        )
    });
    Ok(json!({
        "edit_timebase": {"num": 1, "den": 90_000},
        "segments": segments,
    }))
}

fn mapping_segment(
    stream_index: u64,
    segment: u64,
    source_start: i128,
    source_end: i128,
    edit_start: i128,
    edit_end: i128,
) -> Value {
    json!({
        "segment_id": format!("stream-{stream_index}-segment-{segment}"),
        "stream_index": stream_index,
        "source_start": clamp_i128(source_start),
        "source_end": clamp_i128(source_end),
        "edit_start_ticks": clamp_i128(edit_start),
        "edit_end_ticks": clamp_i128(edit_end),
    })
}

fn normalize_chapters(value: Option<&Value>) -> Result<Vec<Value>, SourceProbeError> {
    let mut chapters = Vec::new();
    for (ordinal, chapter) in value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let start = chapter
            .get("start_time")
            .and_then(Value::as_str)
            .map(decimal_seconds_to_ticks)
            .transpose()?
            .unwrap_or(0)
            .max(0);
        let end = chapter
            .get("end_time")
            .and_then(Value::as_str)
            .map(decimal_seconds_to_ticks)
            .transpose()?
            .unwrap_or(start)
            .max(start);
        let title = chapter
            .get("tags")
            .and_then(|tags| tags.get("title"))
            .and_then(Value::as_str)
            .unwrap_or("");
        chapters.push(json!({
            "id": json_u64(chapter.get("id")).unwrap_or(ordinal as u64),
            "start_ticks": clamp_i128(start),
            "end_ticks": clamp_i128(end),
            "title": title,
        }));
    }
    Ok(chapters)
}

fn rotation(stream: &Value) -> i64 {
    let raw = stream
        .get("side_data_list")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find_map(|entry| entry.get("rotation").and_then(Value::as_i64))
        .or_else(|| {
            stream
                .get("tags")
                .and_then(|tags| tags.get("rotate").or_else(|| tags.get("ROTATE")))
                .and_then(Value::as_str)
                .and_then(|value| value.parse().ok())
        })
        .unwrap_or(0);
    raw.rem_euclid(360) / 90 * 90
}

fn color_metadata(stream: &Value) -> Option<Value> {
    let fields = [
        ("space", "color_space"),
        ("transfer", "color_transfer"),
        ("primaries", "color_primaries"),
        ("range", "color_range"),
    ];
    let mut color = Map::new();
    for (output, input) in fields {
        if let Some(value) = stream.get(input).and_then(Value::as_str)
            && value != "unknown"
        {
            color.insert(output.to_owned(), json!(value));
        }
    }
    (!color.is_empty()).then_some(Value::Object(color))
}

fn hdr_metadata(stream: &Value) -> Option<Value> {
    let entries = stream.get("side_data_list")?.as_array()?;
    let mut hdr = Map::new();
    for entry in entries {
        match entry.get("side_data_type").and_then(Value::as_str) {
            Some("Mastering display metadata") => {
                hdr.insert(
                    "mastering_display".to_owned(),
                    Value::String(serde_json::to_string(entry).unwrap_or_default()),
                );
            }
            Some("Content light level metadata") => {
                hdr.insert(
                    "content_light_level".to_owned(),
                    Value::String(serde_json::to_string(entry).unwrap_or_default()),
                );
            }
            _ => {}
        }
    }
    (!hdr.is_empty()).then_some(Value::Object(hdr))
}

fn parse_ratio(value: &str) -> Result<(i128, i128), SourceProbeError> {
    let (num, den) = value.split_once('/').ok_or(SourceProbeError::InvalidProbe(
        "rational value is malformed",
    ))?;
    let num = num
        .parse::<i128>()
        .map_err(|_| SourceProbeError::InvalidProbe("rational numerator is malformed"))?;
    let den = den
        .parse::<i128>()
        .map_err(|_| SourceProbeError::InvalidProbe("rational denominator is malformed"))?;
    if den <= 0 {
        return Err(SourceProbeError::InvalidProbe(
            "rational denominator is not positive",
        ));
    }
    Ok((num, den))
}

pub(crate) fn decimal_seconds_to_ticks(value: &str) -> Result<i128, SourceProbeError> {
    let negative = value.starts_with('-');
    let unsigned = value.trim_start_matches(['-', '+']);
    let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    let scale = 10_i128
        .checked_pow(u32::try_from(fraction.len()).unwrap_or(u32::MAX))
        .ok_or(SourceProbeError::InvalidProbe(
            "decimal timestamp is too precise",
        ))?;
    let whole = whole
        .parse::<i128>()
        .map_err(|_| SourceProbeError::InvalidProbe("decimal timestamp is malformed"))?;
    let fraction = if fraction.is_empty() {
        0
    } else {
        fraction
            .parse::<i128>()
            .map_err(|_| SourceProbeError::InvalidProbe("decimal timestamp is malformed"))?
    };
    let numerator = whole
        .checked_mul(scale)
        .and_then(|number| number.checked_add(fraction))
        .ok_or(SourceProbeError::InvalidProbe(
            "decimal timestamp overflows",
        ))?;
    round_ratio(
        if negative { -numerator } else { numerator } * 90_000,
        scale,
    )
}

pub(crate) fn to_edit_ticks(value: i128, num: i128, den: i128) -> Result<i128, SourceProbeError> {
    let numerator = value
        .checked_mul(num)
        .and_then(|value| value.checked_mul(90_000))
        .ok_or(SourceProbeError::InvalidProbe(
            "timestamp conversion overflows",
        ))?;
    round_ratio(numerator, den)
}

fn round_ratio(numerator: i128, denominator: i128) -> Result<i128, SourceProbeError> {
    if denominator <= 0 {
        return Err(SourceProbeError::InvalidProbe(
            "timestamp denominator is invalid",
        ));
    }
    let negative = numerator.is_negative();
    let absolute = numerator.unsigned_abs();
    let denominator = u128::try_from(denominator)
        .map_err(|_| SourceProbeError::InvalidProbe("timestamp denominator is invalid"))?;
    let quotient = absolute / denominator;
    let remainder = absolute % denominator;
    let rounded = match remainder.saturating_mul(2).cmp(&denominator) {
        std::cmp::Ordering::Greater => quotient.saturating_add(1),
        std::cmp::Ordering::Equal if quotient % 2 == 1 => quotient.saturating_add(1),
        _ => quotient,
    };
    let rounded = i128::try_from(rounded)
        .map_err(|_| SourceProbeError::InvalidProbe("timestamp conversion overflows"))?;
    Ok(if negative { -rounded } else { rounded })
}

fn mapping_duration(mapping: &Value) -> i128 {
    mapping["segments"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|segment| segment["edit_end_ticks"].as_i64())
        .max()
        .map_or(0, i128::from)
}

fn json_u64(value: Option<&Value>) -> Option<u64> {
    value.and_then(|value| {
        value
            .as_u64()
            .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
    })
}

fn json_i128(value: Option<&Value>) -> Option<i128> {
    value.and_then(|value| {
        value
            .as_i64()
            .map(i128::from)
            .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
    })
}

fn json_string_u64(value: &Value) -> Option<u64> {
    value.as_str()?.parse().ok()
}

fn clamp_i128(value: i128) -> i64 {
    i64::try_from(value).unwrap_or_else(|_| {
        if value.is_negative() {
            i64::MIN
        } else {
            i64::MAX
        }
    })
}

fn sanitize_detail(stderr: &str) -> String {
    let text = stderr.trim().chars().take(512).collect::<String>();
    if text.is_empty() {
        "unrecognized or hostile media".to_owned()
    } else {
        // FFprobe may include the source path in diagnostics. Do not retain it.
        "unrecognized or hostile media (FFprobe returned an error)".to_owned()
    }
}

#[allow(clippy::needless_pass_by_value)]
fn io_error(error: std::io::Error) -> SourceProbeError {
    SourceProbeError::Io(error.to_string())
}

const fn nonblocking_flag() -> i32 {
    #[cfg(target_os = "linux")]
    {
        0x800
    }
    #[cfg(target_os = "macos")]
    {
        0x0004
    }
}

const fn nofollow_flag() -> i32 {
    #[cfg(target_os = "linux")]
    {
        0x20_000
    }
    #[cfg(target_os = "macos")]
    {
        0x0100
    }
}

#[derive(Debug)]
struct ScratchGuard(PathBuf);

impl Drop for ScratchGuard {
    fn drop(&mut self) {
        let _removed = fs::remove_dir_all(&self.0);
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use std::collections::BTreeMap;

    use super::{
        SMALL_FILE_LIMIT, SourceProbeError, StreamRuns, complete_inspection,
        decimal_seconds_to_ticks, normalize_probe, parse_ratio, round_ratio,
        run_ffprobe_with_timeout, sample_source, sample_spans, to_edit_ticks,
    };
    use serde_json::json;
    use std::{fs, os::unix::fs::PermissionsExt, time::Duration};
    use tempfile::TempDir;

    #[test]
    fn small_sources_hash_every_byte_and_large_sources_use_eighteen_windows() {
        assert_eq!(sample_spans(12), vec![(0, 12)]);
        let spans = sample_spans(SMALL_FILE_LIMIT + 1);
        assert_eq!(spans.len(), 18);
        assert_eq!(spans.first(), Some(&(0, 1024 * 1024)));
        assert_eq!(
            spans.last().map(|span| span.0 + span.1),
            Some(SMALL_FILE_LIMIT + 1)
        );
    }

    #[test]
    fn rational_conversion_rounds_ties_to_even() {
        assert_eq!(round_ratio(1, 2).expect("half"), 0);
        assert_eq!(round_ratio(3, 2).expect("one and half"), 2);
        assert_eq!(round_ratio(-3, 2).expect("negative one and half"), -2);
        assert_eq!(decimal_seconds_to_ticks("1.001").expect("decimal"), 90_090);
        assert_eq!(parse_ratio("1001/30000").expect("ratio"), (1001, 30000));
        assert!(matches!(
            parse_ratio("1/0"),
            Err(SourceProbeError::InvalidProbe(_))
        ));
    }

    /// Fold a list of packets exactly as the streamed reader does, so a test
    /// that writes packets out by hand still exercises the real path.
    fn folded(packets: &[(u64, i128, i128)]) -> BTreeMap<u64, StreamRuns> {
        let mut runs = BTreeMap::<u64, StreamRuns>::new();
        for (index, pts, duration) in packets {
            runs.entry(*index)
                .or_default()
                .observe(*pts, *duration)
                .expect("a handful of packets is under every bound");
        }
        runs
    }

    /// A recording longer than the old sixteen-megabyte JSON ceiling.
    ///
    /// The file is made by stream-copying a short clip end to end, so it costs
    /// no encoding and has the packet count of a real hour: about 126,000
    /// video packets and as many again of audio. Under the previous probe this
    /// produced roughly ninety megabytes of ffprobe JSON and was refused with
    /// `OutputLimit` — which meant ClipMill could not import an hour-long
    /// recording, the exact thing it is for.
    ///
    /// Ignored by default because it needs the pinned FFmpeg and takes a few
    /// seconds; `gate-ingest` runs it.
    #[test]
    #[ignore = "needs the pinned FFmpeg sidecar; run through gate-ingest"]
    fn an_hour_long_recording_is_probed_rather_than_refused() {
        let (ffprobe, ffmpeg) = pinned_tools();
        let scratch = tempfile::tempdir().expect("a scratch directory");
        let long = scratch.path().join("long.mp4");
        let short = scratch.path().join("short.mp4");
        assert!(
            std::process::Command::new(&ffmpeg)
                .args([
                    "-y",
                    "-f",
                    "lavfi",
                    "-i",
                    "testsrc=size=320x180:rate=30:duration=20"
                ])
                .args(["-f", "lavfi", "-i", "sine=frequency=440:duration=20"])
                .args([
                    "-c:v",
                    "libx264",
                    "-preset",
                    "ultrafast",
                    "-c:a",
                    "aac",
                    "-shortest"
                ])
                .arg(&short)
                .output()
                .expect("ffmpeg runs")
                .status
                .success()
        );
        assert!(
            std::process::Command::new(&ffmpeg)
                .args(["-y", "-stream_loop", "210", "-i"])
                .arg(&short)
                .args(["-c", "copy"])
                .arg(&long)
                .output()
                .expect("ffmpeg runs")
                .status
                .success()
        );

        let (metadata, packets) =
            run_ffprobe_with_timeout(&ffprobe, scratch.path(), &long, super::PROBE_TIMEOUT)
                .expect("an hour-long recording is probed");
        let normalized = normalize_probe(&metadata, &packets).expect("it normalizes");

        // Over two hundred thousand packets read, and the state kept is one run
        // per stream: the whole point of the change.
        assert!(packets.len() >= 2, "both streams produced packets");
        for stream in packets.values() {
            assert_eq!(
                stream.runs.len(),
                1,
                "a stream-copied loop has one continuous run"
            );
        }
        let duration = normalized["container"]["duration_ticks"]
            .as_i64()
            .expect("the container carries a duration");
        assert!(
            duration > 90_000 * 3_600,
            "the probe reported {duration} ticks, under an hour"
        );
    }

    /// The pinned sidecars.
    ///
    /// Panics rather than returning `None` when they are absent. The test that
    /// calls this is `#[ignore]`d, so it runs only when somebody asked for it —
    /// and a test that quietly passes by doing nothing is worse than one that
    /// fails, because the silence is indistinguishable from success.
    fn pinned_tools() -> (std::path::PathBuf, std::path::PathBuf) {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.cache/bin");
        let ffprobe = root.join("ffprobe");
        let ffmpeg = root.join("ffmpeg");
        assert!(
            ffprobe.is_file() && ffmpeg.is_file(),
            "the pinned FFmpeg sidecar is missing from {}; run ./tools/fetch-ffmpeg.sh",
            root.display()
        );
        (ffprobe, ffmpeg)
    }

    #[test]
    fn the_fold_keeps_one_run_for_a_continuous_stream_however_long() {
        // A hundred thousand packets in order: one run, two distinct durations
        // never reached, and nothing retained per packet.
        let mut stream = StreamRuns::default();
        for index in 0..100_000_i128 {
            stream.observe(index * 40, 40).expect("under every bound");
        }
        assert_eq!(stream.runs.len(), 1);
        assert_eq!(stream.runs[0], (0, 99_999 * 40 + 40));
        assert!(!stream.is_variable());
    }

    #[test]
    fn a_timeline_that_is_nothing_but_jumps_is_refused() {
        // Every packet jumping backwards is not a recording. The bound is on
        // nonsense, not on length, so it must be reachable only this way.
        let mut stream = StreamRuns::default();
        let outcome = (0..super::MAX_RUNS_PER_STREAM as i128 + 2)
            .try_for_each(|index| stream.observe(-index * 40, 40));
        assert!(matches!(outcome, Err(SourceProbeError::OutputLimit)));
    }

    #[test]
    fn mapping_preserves_gaps_splits_resets_and_tracks_audio_offset() {
        let packets = folded(&[
            (0, 100, 40),
            (0, 140, 40),
            (0, 300, 40),
            (0, 10, 40),
            (1, 200, 20),
            (1, 220, 20),
        ]);
        let normalized = normalize_probe(
            &json!({
            "format": {"format_name": "matroska,webm", "duration": "1.000"},
            "streams": [
                {
                    "index": 0,
                    "codec_type": "video",
                    "codec_name": "h264",
                    "time_base": "1/1000",
                    "start_pts": 100,
                    "duration_ts": 500,
                    "width": 1280,
                    "height": 720,
                    "avg_frame_rate": "30/1",
                    "r_frame_rate": "24/1"
                },
                {
                    "index": 1,
                    "codec_type": "audio",
                    "codec_name": "aac",
                    "time_base": "1/1000",
                    "start_pts": 200,
                    "duration_ts": 300,
                    "sample_rate": "48000",
                    "channels": 2,
                    "channel_layout": "stereo"
                }
            ],
            "chapters": []
            }),
            &packets,
        )
        .expect("normalize synthetic probe");
        let segments = normalized["mapping"]["segments"]
            .as_array()
            .expect("mapping segments");
        assert_eq!(
            segments
                .iter()
                .filter(|segment| segment["stream_index"] == 0)
                .count(),
            2,
            "backward reset creates a new segment"
        );
        assert_eq!(
            segments
                .iter()
                .find(|segment| segment["segment_id"] == "stream-0-segment-0")
                .expect("first video segment")["edit_end_ticks"],
            21_600,
            "the positive presentation gap remains in edit time"
        );
        assert_eq!(normalized["streams"][1]["start_offset_ticks"], 18_000);
        assert_eq!(normalized["streams"][0]["video"]["vfr"], true);
    }

    #[test]
    fn source_and_edit_tick_conversion_roundtrips_within_one_tick() {
        for source_tick in [-90_001_i128, -1, 0, 1, 1_001, 90_001] {
            let edit = to_edit_ticks(source_tick, 1001, 30_000).expect("to edit");
            let restored = round_ratio(edit * 30_000, 1001 * 90_000).expect("to source");
            let edit_again = to_edit_ticks(restored, 1001, 30_000).expect("back to edit");
            assert!((edit_again - edit).abs() <= 1);
        }
    }

    #[test]
    fn probe_deadline_terminates_a_stuck_sidecar() {
        let temp = TempDir::new().expect("tempdir");
        let sidecar = temp.path().join("stuck-ffprobe");
        fs::write(&sidecar, "#!/bin/sh\nsleep 5\n").expect("write fake sidecar");
        fs::set_permissions(&sidecar, fs::Permissions::from_mode(0o700))
            .expect("make fake sidecar executable");
        let source = temp.path().join("source.bin");
        fs::write(&source, b"source").expect("write source");
        let error =
            run_ffprobe_with_timeout(&sidecar, temp.path(), &source, Duration::from_millis(50))
                .expect_err("stuck sidecar times out");
        assert!(matches!(error, SourceProbeError::Timeout));
    }

    #[test]
    fn mutation_during_probe_is_retryably_detected() {
        let temp = TempDir::new().expect("tempdir");
        let scratch = temp.path().join("scratch");
        fs::create_dir(&scratch).expect("scratch");
        let sidecar = temp.path().join("mutating-ffprobe");
        fs::write(
            &sidecar,
            concat!(
                "#!/bin/sh\n",
                "for last in \"$@\"; do :; done\n",
                "printf x >> \"$last\"\n",
                "printf '%s' '{\"format\":{\"format_name\":\"test\",\"duration\":\"1.0\"},",
                "\"streams\":[{\"index\":0,\"codec_type\":\"video\",",
                "\"codec_name\":\"rawvideo\",\"time_base\":\"1/1000\",",
                "\"start_pts\":0,\"duration_ts\":1000,\"width\":1,\"height\":1}],",
                "\"packets\":[]}'\n"
            ),
        )
        .expect("write mutating sidecar");
        fs::set_permissions(&sidecar, fs::Permissions::from_mode(0o700))
            .expect("make sidecar executable");
        let source = temp.path().join("source.bin");
        fs::write(&source, b"source").expect("write source");
        let sampled = sample_source(source.to_str().expect("UTF-8 path")).expect("sample source");
        let error = complete_inspection(&sidecar, &scratch, sampled)
            .expect_err("mutation invalidates observation");
        assert!(matches!(error, SourceProbeError::SourceChanged));
    }
}
