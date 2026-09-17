//! Milestone 1's exit, run for real.
//!
//! From an older project while a newer one exists, choose a clip several
//! minutes into the recording; play and scrub it; correct a name; trim both
//! ends; undo and redo; restart the daemon; export; and decode what came
//! out to check it holds the footage that was asked for, at the timing that
//! was asked for, with the correction in both caption outputs and the
//! revision that was reviewed. Everything goes through the shell's own
//! daemon link and media door — the bridge the renderer stands on — against
//! a spawned `clipmilld` and the real worker fleet. Nothing is stubbed.
//!
//! The recording is built by `tools/fixtures/make-long-recording.sh`: a
//! spoken talk from the platform synthesizer, and a picture whose colour
//! names its own source second. That is what lets the end of this test read
//! the delivered frames and say which second of the recording each one came
//! from, through a proxy, a crop, and an encoder, without OCR.
//!
//! Needs a built `clipmilld`, the pinned FFmpeg sidecars and font, the
//! worker environments and the fetched weights, and macOS for the voice —
//! which is why it is `#[ignore]` and driven by `just gate-milestone-1`.
#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
// One scenario, told in order: splitting it into functions would hide the
// order, which is the point. Ticks here are seconds times ninety thousand,
// far inside every integer and every mantissa they are cast between.
#![allow(
    clippy::too_many_lines,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss
)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Arc,
    time::{Duration, Instant},
};

use clipmill_contracts::proto::ipc::v1::{
    AnalyzeSourcePayloadV1, ClipCutV1, ClipDurationV1, DirectClipRequest, EditDoc, ExportRequestV1,
    GetPreviewPlanResponse, GetReadinessResponse, Job, JobState,
};
use clipmill_shell::{DaemonClient, DaemonSupervisor, MEDIA_SCHEME as SCHEME, MediaProtocol};
use serde_json::{Value, json};

const TICKS: i64 = 90_000;
/// The whole analysis of an eleven-minute talk on the portable ASR path.
const ANALYSIS_TIMEOUT: Duration = Duration::from_mins(20);
const EXPORT_TIMEOUT: Duration = Duration::from_mins(5);
/// How far a decoded frame may sit from the second it was asked for. The
/// picture names time to a tenth; a frame period is a thirtieth.
const TIMING_TOLERANCE_SECONDS: f64 = 0.25;
/// A clip that begins this far in is "several minutes into the source".
const SEVERAL_MINUTES: i64 = 3 * 60 * TICKS;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repository root")
}

fn daemon_binary() -> PathBuf {
    let mut path = std::env::current_exe().expect("test executable path");
    path.pop(); // deps/
    path.pop(); // debug/
    path.join("clipmilld")
}

fn tool(name: &str) -> PathBuf {
    let path = repo_root().join(".cache/bin").join(name);
    assert!(
        path.is_file(),
        "pinned {name} missing at {}; run ./tools/fetch-ffmpeg.sh",
        path.display()
    );
    path
}

fn log_file(directory: &Path, name: &str) -> fs::File {
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(directory.join(name))
        .expect("a log file")
}

/// A daemon spawned the way the shell spawns one, in a private data directory.
struct DaemonUnderTest {
    child: Child,
    socket: PathBuf,
}

impl DaemonUnderTest {
    fn start(directory: &Path) -> Self {
        let binary = daemon_binary();
        assert!(
            binary.is_file(),
            "clipmilld is not built at {}; run `cargo build --workspace` first",
            binary.display()
        );
        let socket = directory.join("d.sock");
        let child = Command::new(binary)
            .arg("--data-dir")
            .arg(directory)
            .arg("--socket")
            .arg(&socket)
            .arg("--ffprobe")
            .arg(tool("ffprobe"))
            // The registry is named relative to the repository by default, and
            // this test's working directory is the shell crate.
            .env("CLIPMILL_MODELS_DIR", repo_root().join("models/registry"))
            // Appended, so a restarted daemon's log follows the first one's.
            .stdout(Stdio::from(log_file(directory, "daemon.log")))
            .stderr(Stdio::from(log_file(directory, "daemon.log")))
            .spawn()
            .expect("spawn clipmilld");
        Self { child, socket }
    }

    fn client(&self) -> DaemonClient {
        DaemonClient::new(self.socket.clone())
    }

    /// Stop it the way the shell does, and confirm it is gone.
    fn stop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for DaemonUnderTest {
    fn drop(&mut self) {
        self.stop();
    }
}

/// The worker fleet, launched by the same script `just workers` runs.
///
/// Stopped with TERM rather than KILL: the script forwards TERM to every
/// worker it started, and a KILL would leave five Python processes behind.
struct Workers {
    child: Child,
}

impl Workers {
    fn enrol(directory: &Path) {
        let status = Command::new(repo_root().join("tools/run-workers.sh"))
            .arg("--enrol-only")
            .arg("--data-dir")
            .arg(directory)
            .current_dir(repo_root())
            .stdout(Stdio::null())
            .status()
            .expect("enrol the workers");
        assert!(status.success(), "worker enrolment failed");
    }

    fn launch(directory: &Path) -> Self {
        let child = Command::new(repo_root().join("tools/run-workers.sh"))
            .arg("--data-dir")
            .arg(directory)
            .current_dir(repo_root())
            .stdout(Stdio::from(log_file(directory, "workers.log")))
            .stderr(Stdio::from(log_file(directory, "workers.log")))
            .spawn()
            .expect("launch the workers");
        Self { child }
    }
}

impl Drop for Workers {
    fn drop(&mut self) {
        let _ = Command::new("kill")
            .arg("-TERM")
            .arg(self.child.id().to_string())
            .status();
        let _ = self.child.wait();
    }
}

async fn wait_for_health(client: &DaemonClient, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if client.health().await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("the daemon never opened its socket");
}

async fn wait_for_readiness(client: &DaemonClient, timeout: Duration) -> GetReadinessResponse {
    let deadline = Instant::now() + timeout;
    loop {
        let report = client.readiness().await.expect("read readiness");
        if report.ready {
            return report;
        }
        assert!(
            Instant::now() < deadline,
            "the workers never all connected: {:#?}",
            report
                .stages
                .iter()
                .filter(|stage| !stage.ready)
                .map(|stage| &stage.remedy)
                .collect::<Vec<_>>()
        );
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

async fn wait_for_job(client: &DaemonClient, job_id: &str, timeout: Duration) -> Job {
    let started = Instant::now();
    loop {
        let job = client.get_job(job_id).await.expect("read the job");
        if matches!(
            JobState::try_from(job.state),
            Ok(JobState::Succeeded | JobState::Failed | JobState::Cancelled)
        ) {
            return job;
        }
        assert!(
            started.elapsed() < timeout,
            "job {job_id} did not settle within {timeout:?}; running: {:?}",
            job.tasks
                .iter()
                .filter(|task| task.state == 3)
                .map(|task| &task.kind)
                .collect::<Vec<_>>()
        );
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

/// The artifact a run's task of this kind published.
fn output_of(job: &Job, kind: &str) -> String {
    job.tasks
        .iter()
        .find(|task| task.kind == kind && !task.output_artifact_id.is_empty())
        .unwrap_or_else(|| panic!("the run published no {kind}"))
        .output_artifact_id
        .clone()
}

/// The recording: named by the drill, or built here on a machine that can.
fn recording(directory: &Path) -> PathBuf {
    if let Some(named) = std::env::var_os("CLIPMILL_M1_RECORDING") {
        let path = PathBuf::from(named);
        assert!(path.is_file(), "{} is not a file", path.display());
        return path;
    }
    let path = directory.join("talk.mp4");
    let status = Command::new(repo_root().join("tools/fixtures/make-long-recording.sh"))
        .arg(&path)
        .arg("--ffmpeg")
        .arg(tool("ffmpeg"))
        .current_dir(repo_root())
        .stdout(Stdio::null())
        .status()
        .expect("build the recording");
    assert!(status.success(), "the recording could not be built");
    path
}

/// A project with the recording registered and analysed, settled.
struct Analysed {
    project_id: String,
    source_id: String,
    source_fingerprint: String,
    job: Job,
}

async fn analyse(client: &DaemonClient, name: &str, recording: &Path) -> Analysed {
    let project_id = client.create_project(name).await.expect("a project");
    let registered = client
        .register_source(&project_id, recording.to_str().expect("a UTF-8 path"))
        .await
        .expect("register the recording");
    let source = registered.source.expect("the registered source");
    let job = client
        .submit_analyze(
            &project_id,
            AnalyzeSourcePayloadV1 {
                key_version: "clipmill.analyze-source.v1".to_owned(),
                source_id: source.source_id.clone(),
                language: String::new(),
                duration: Some(ClipDurationV1 {
                    min_ticks: 15 * TICKS as u64,
                    max_ticks: 90 * TICKS as u64,
                }),
                count: 5,
                diversity_milli: 0,
            },
        )
        .await
        .expect("submit the analysis");
    let settled = wait_for_job(client, &job.job_id, ANALYSIS_TIMEOUT).await;
    assert_eq!(
        settled.state,
        JobState::Succeeded as i32,
        "the analysis of {name} failed: {}",
        settled.failure_detail
    );
    Analysed {
        project_id,
        source_id: source.source_id,
        source_fingerprint: source.source_fingerprint,
        job: settled,
    }
}

/// The ranked candidate that starts several minutes in, highest first.
async fn candidate_minutes_in(client: &DaemonClient, run: &Analysed) -> (String, i64, i64) {
    let ranking_id = output_of(&run.job, "rank-candidates");
    let (kind, json) = client
        .read_document(&run.project_id, &ranking_id)
        .await
        .expect("read the ranking");
    assert_eq!(kind, "ranking.set.v1");
    let ranking: Value = serde_json::from_str(&json).expect("the ranking parses");
    let cohort = ranking["cohort"].as_array().expect("a cohort");
    assert!(!cohort.is_empty(), "the run ranked nothing");
    let chosen = cohort
        .iter()
        .find(|entry| {
            entry["boundary"]["chosen"]["start_ticks"]
                .as_i64()
                .unwrap_or(0)
                >= SEVERAL_MINUTES
        })
        .unwrap_or_else(|| panic!("no ranked clip starts several minutes in: {json}"));
    (
        chosen["candidate_id"]
            .as_str()
            .expect("a candidate id")
            .to_owned(),
        chosen["boundary"]["chosen"]["start_ticks"]
            .as_i64()
            .unwrap(),
        chosen["boundary"]["chosen"]["end_ticks"].as_i64().unwrap(),
    )
}

/// The guests of the synthesized talk, by first name — see
/// `tools/fixtures/make-long-recording.sh`.
const GUEST_NAMES: [&str; 6] = ["Priya", "Marisol", "Ingrid", "Teodora", "Yuki", "Beatrix"];

/// A caption word without the punctuation the recogniser attached to it.
fn bare(text: &str) -> &str {
    text.trim_end_matches(|c: char| !c.is_alphanumeric())
}

/// How many times a word is said in a caption file, as a whole word.
fn count_word(text: &str, word: &str) -> usize {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|token| *token == word)
        .count()
}

fn document(doc: &EditDoc) -> Value {
    serde_json::from_str(&doc.document_json).expect("the document parses")
}

/// The words of one presentation, flattened, as (`word_id`, text).
fn words_of(doc: &Value, presentation: &str) -> Vec<(String, String)> {
    doc["captions"][presentation]
        .as_array()
        .map(|cues| {
            cues.iter()
                .flat_map(|cue| cue["lines"].as_array().unwrap().iter())
                .flat_map(|line| line["words"].as_array().unwrap().iter())
                .map(|word| {
                    (
                        word["word_id"].as_str().unwrap_or("").to_owned(),
                        word["text"].as_str().unwrap().to_owned(),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn segment_of(doc: &Value) -> (String, i64, i64) {
    let segment = &doc["video"]["segments"][0];
    (
        segment["segment_id"].as_str().unwrap().to_owned(),
        segment["in_ticks"].as_i64().unwrap(),
        segment["out_ticks"].as_i64().unwrap(),
    )
}

fn media_request(path: &str, range: Option<&str>) -> tauri::http::Request<Vec<u8>> {
    let mut builder = tauri::http::Request::builder().uri(format!("{SCHEME}://localhost{path}"));
    if let Some(value) = range {
        builder = builder.header(tauri::http::header::RANGE, value);
    }
    builder.body(Vec::new()).expect("a media request")
}

/// The proxy the plan names, played from its head and then scrubbed into.
///
/// A player seeking sends a byte range, and the door has to honour it from
/// the file the plan named — the source's proxy, from the run the clip was
/// directed from — not the newest proxy in the store.
async fn play_and_scrub(
    client: &DaemonClient,
    protocol: &MediaProtocol,
    project: &str,
    plan: &GetPreviewPlanResponse,
    fingerprint: &str,
) {
    let proxy = plan
        .proxies
        .iter()
        .find(|proxy| proxy.source_fingerprint == fingerprint)
        .expect("the plan names a proxy for the clip's source");
    assert!(proxy.width > 0 && proxy.height > 0 && proxy.rate_num > 0);
    let inventory = client
        .resolve_media(project, &proxy.artifact_id)
        .await
        .expect("resolve the proxy");
    let file = inventory
        .files
        .iter()
        .find(|file| file.path == proxy.file)
        .expect("the proxy artifact serves the file the plan named");
    assert_eq!(file.media_type, "video/mp4");
    let route = format!("/{project}/{}/{}", proxy.artifact_id, proxy.file);

    let head = protocol
        .serve(media_request(&route, Some("bytes=0-1023")))
        .await;
    assert_eq!(head.status(), 206, "the proxy's head did not stream");
    assert_eq!(head.body().len(), 1024);
    // A scrub: the player lands in the middle of the file and asks from there.
    let middle = file.bytes / 2;
    let scrubbed = protocol
        .serve(media_request(
            &route,
            Some(&format!("bytes={middle}-{}", middle + 4095)),
        ))
        .await;
    assert_eq!(
        scrubbed.status(),
        206,
        "a seek into the proxy did not stream"
    );
    assert_eq!(scrubbed.body().len(), 4096);
    assert_eq!(
        scrubbed
            .headers()
            .get(tauri::http::header::CONTENT_RANGE)
            .and_then(|value| value.to_str().ok()),
        Some(format!("bytes {middle}-{}/{}", middle + 4095, file.bytes).as_str())
    );
}

/// The source second a decoded frame came from, read from its colour.
///
/// Inverts what `make-long-recording.sh` wrote: luma names the minute, Cb
/// the second within it to a third of a second, and Cr the second within
/// each ten to a twentieth. Cb picks the ten-second band and Cr refines it,
/// so an encoder's rounding on any one plane moves the answer by less than
/// a frame rather than by a band.
fn source_second_of(y: f64, cb: f64, cr: f64) -> f64 {
    let minute = ((y - 16.0) / 219.0 * 15.0).round();
    let coarse = (cb - 16.0) / 224.0 * 60.0;
    let fine = (cr - 16.0) / 224.0 * 10.0;
    let band = (0..6)
        .map(|k| f64::from(k) * 10.0)
        .min_by(|left, right| {
            ((left + fine) - coarse)
                .abs()
                .total_cmp(&((right + fine) - coarse).abs())
        })
        .unwrap();
    minute * 60.0 + band + fine
}

/// Mean Y, Cb, Cr of a patch at the centre of the frame at this time.
///
/// The centre of the delivered frame is inside the picture under either
/// layout — letterboxed or filled — and away from the captions along the
/// bottom, so the patch is the flat colour and nothing else.
fn decode_patch(file: &Path, at_seconds: f64) -> (f64, f64, f64) {
    let output = Command::new(tool("ffmpeg"))
        .args(["-hide_banner", "-loglevel", "info", "-ss"])
        .arg(format!("{at_seconds:.3}"))
        .arg("-i")
        .arg(file)
        .args([
            "-frames:v",
            "1",
            "-vf",
            "crop=iw/5:ih/10:(iw-iw/5)/2:(ih-ih/10)/2,signalstats,metadata=print:file=-",
            "-f",
            "null",
            "-",
        ])
        .output()
        .expect("decode a frame");
    let printed = String::from_utf8_lossy(&output.stdout);
    let read = |key: &str| -> f64 {
        printed
            .lines()
            .find_map(|line| {
                line.trim()
                    .strip_prefix(&format!("lavfi.signalstats.{key}="))
            })
            .unwrap_or_else(|| panic!("no {key} for {} at {at_seconds}: {printed}", file.display()))
            .trim()
            .parse()
            .expect("a number")
    };
    (read("YAVG"), read("UAVG"), read("VAVG"))
}

fn source_second_at(file: &Path, at_seconds: f64) -> f64 {
    let (y, cb, cr) = decode_patch(file, at_seconds);
    source_second_of(y, cb, cr)
}

fn probe_duration_seconds(file: &Path) -> f64 {
    let output = Command::new(tool("ffprobe"))
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "csv=p=0",
        ])
        .arg(file)
        .output()
        .expect("probe the clip");
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .expect("a duration")
}

/// A file inside a published artifact, by the store's own layout — the same
/// derivation the media door makes.
fn artifact_file(artifacts_dir: &Path, artifact_id: &str, name: &str) -> PathBuf {
    let digest = artifact_id
        .strip_prefix("sha256:")
        .expect("a sha256 address");
    artifacts_dir
        .join("objects/sha256")
        .join(&digest[..2])
        .join(digest)
        .join(name)
}

async fn apply(client: &DaemonClient, doc: &EditDoc, command: Value) -> (EditDoc, Value) {
    let reply = client
        .apply_edit_command(&doc.doc_id, doc.revision, command.to_string())
        .await
        .expect("apply the command");
    let applied = reply.doc.expect("the document after the command");
    assert_eq!(applied.revision, doc.revision + 1);
    let inverse: Value = serde_json::from_str(&reply.inverse_command_json).expect("an inverse");
    (applied, inverse)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires a built workspace, the pinned sidecars, the worker environments and weights, and macOS"]
async fn an_older_projects_clip_survives_the_whole_workflow() {
    let directory = PathBuf::from(format!("/tmp/cm-m1-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).expect("test directory");
    let recording = recording(&directory);

    // ---- The daemon, and what it says before the workers arrive. ----
    Workers::enrol(&directory);
    let mut daemon = DaemonUnderTest::start(&directory);
    let client = daemon.client();
    wait_for_health(&client, Duration::from_secs(30)).await;

    let before = client.readiness().await.expect("readiness");
    assert!(!before.ready, "nothing has connected yet");
    assert!(before.decoder_present, "the pinned decoder is installed");
    assert!(before.workers.is_empty());
    for stage in &before.stages {
        assert!(stage.model_present, "{} has its weights here", stage.stage);
        assert!(!stage.worker_present);
        assert!(
            stage.remedy.contains("just workers"),
            "{} says what to run: {}",
            stage.stage,
            stage.remedy
        );
    }

    let workers = Workers::launch(&directory);
    let ready = wait_for_readiness(&client, Duration::from_mins(2)).await;
    assert_eq!(ready.workers.len(), 5, "five families announced themselves");

    // ---- Two projects: the older one analysed first, then a newer one. ----
    let older = analyse(&client, "older talk", &recording).await;
    let newer = analyse(&client, "newer talk", &recording).await;
    assert_ne!(older.job.job_id, newer.job.job_id);
    let projects = client.list_projects().await.expect("projects");
    let position = |id: &str| projects.iter().position(|p| p.project_id == id).unwrap();
    assert!(
        position(&older.project_id) != position(&newer.project_id),
        "two projects"
    );

    // ---- Choose a clip several minutes into the older project's source. ----
    let (candidate_id, chosen_start, chosen_end) = candidate_minutes_in(&client, &older).await;
    let directed = client
        .direct_clip(DirectClipRequest {
            project_id: older.project_id.clone(),
            source_id: older.source_id.clone(),
            candidate_id: candidate_id.clone(),
            cut: ClipCutV1::Chosen as i32,
            style_ref: String::new(),
            start_ticks: 0,
            end_ticks: 0,
            variation: false,
            approve: true,
            job_id: older.job.job_id.clone(),
        })
        .await
        .expect("direct the clip");
    assert!(!directed.reopened, "the first approval builds the document");
    let doc = directed.doc.expect("a document");
    assert_eq!(doc.project_id, older.project_id);
    assert_eq!(doc.source_id, older.source_id);
    assert_eq!(doc.candidate_id, candidate_id);
    assert_eq!(doc.job_id, older.job.job_id);
    assert_eq!(directed.start_ticks as i64, chosen_start);
    assert_eq!(directed.end_ticks as i64, chosen_end);
    let (segment_id, in_ticks, out_ticks) = segment_of(&document(&doc));
    assert_eq!((in_ticks, out_ticks), (chosen_start, chosen_end));
    assert!(
        in_ticks >= SEVERAL_MINUTES,
        "the clip starts {} s in",
        in_ticks / TICKS
    );

    // Approving it again reopens it; the newer project has no document.
    let again = client
        .direct_clip(DirectClipRequest {
            project_id: older.project_id.clone(),
            source_id: older.source_id.clone(),
            candidate_id: candidate_id.clone(),
            cut: ClipCutV1::Chosen as i32,
            style_ref: String::new(),
            start_ticks: 0,
            end_ticks: 0,
            variation: false,
            approve: true,
            job_id: older.job.job_id.clone(),
        })
        .await
        .expect("direct the clip again");
    assert!(again.reopened);
    assert_eq!(again.doc.expect("the same document").doc_id, doc.doc_id);
    assert_eq!(
        client
            .list_edit_docs(&older.project_id)
            .await
            .unwrap()
            .len(),
        1
    );
    assert!(
        client
            .list_edit_docs(&newer.project_id)
            .await
            .unwrap()
            .is_empty()
    );

    // ---- Play and scrub it, through the media door. ----
    let plan = client
        .preview_plan(&older.project_id, &doc.doc_id)
        .await
        .expect("the preview plan");
    assert_eq!(plan.revision, doc.revision);
    assert_eq!(plan.segments.len(), 1);
    let planned = &plan.segments[0];
    assert_eq!(planned.segment_id, segment_id);
    assert_eq!((planned.in_ticks, planned.out_ticks), (in_ticks, out_ticks));
    assert_eq!(planned.first_frame, 0);
    assert_eq!(planned.end_frame, plan.frame_count);
    let source = plan
        .sources
        .iter()
        .find(|source| source.source_fingerprint == older.source_fingerprint)
        .expect("the plan names the clip's source");
    assert_eq!(source.source_id, older.source_id);
    assert_eq!((source.display_width, source.display_height), (1280, 720));
    let proxy = plan
        .proxies
        .iter()
        .find(|proxy| proxy.source_fingerprint == older.source_fingerprint)
        .expect("the plan names the source's proxy");
    assert_eq!(
        proxy.artifact_id,
        output_of(&older.job, "ingest-proxy"),
        "the proxy is the one the clip's own run derived"
    );
    assert!(proxy.coverage_start_ticks <= in_ticks && out_ticks <= proxy.coverage_end_ticks);
    let supervisor = Arc::new(DaemonSupervisor::new(daemon.client()));
    let protocol = MediaProtocol::new(Arc::clone(&supervisor), directory.join("artifacts"));
    play_and_scrub(
        &client,
        &protocol,
        &older.project_id,
        &plan,
        &older.source_fingerprint,
    )
    .await;

    // ---- Correct a name. ----
    // The talk's guest is surnamed with something the recogniser tends to
    // hear its own way; whatever it heard, the correction is addressed to
    // the word and must land in both presentations.
    let before_correction = document(&doc);
    let reading = words_of(&before_correction, "cues");
    let burn_in = words_of(&before_correction, "burn_in");
    assert!(!reading.is_empty(), "the clip carries captions");
    assert!(
        reading.iter().all(|(id, _)| !id.is_empty()),
        "every word carries an identity"
    );
    // The talk names its guest by a first name the recogniser gets right and
    // a surname it hears its own way; either is a name to correct. A clip
    // from a stretch with no name in it corrects its longest word instead —
    // the mechanics are the same, and the point is the round trip.
    let (word_id, heard) = reading
        .iter()
        .find(|(_, text)| GUEST_NAMES.contains(&bare(text)))
        .or_else(|| reading.iter().max_by_key(|(_, text)| bare(text).len()))
        .cloned()
        .expect("a word in the clip");
    let heard_bare = bare(&heard).to_owned();
    // Spelled the way the guest spells it, keeping whatever punctuation the
    // recogniser attached: a correction is to the word, not to the cue.
    let corrected_text = format!("Okonkwo{}", &heard[heard_bare.len()..]);
    assert_ne!(heard, corrected_text);
    let heard_before = reading
        .iter()
        .filter(|(_, text)| bare(text) == heard_bare)
        .count();
    let (doc, undo_correction) = apply(
        &client,
        &doc,
        json!({"op": "set_word_text", "word_id": word_id, "text": corrected_text}),
    )
    .await;
    let after_correction = document(&doc);
    let corrected = |presentation: &str| {
        words_of(&after_correction, presentation)
            .into_iter()
            .find(|(id, _)| *id == word_id)
            .map(|(_, text)| text)
    };
    assert_eq!(corrected("cues"), Some(corrected_text.clone()));
    if !burn_in.is_empty() {
        assert_eq!(
            corrected("burn_in"),
            Some(corrected_text.clone()),
            "the burned-in grouping carries the same correction"
        );
    }
    assert_eq!(
        undo_correction["op"], "set_word_text",
        "the inverse puts the heard word back"
    );
    assert_eq!(undo_correction["text"], heard);
    let plan = client
        .preview_plan(&older.project_id, &doc.doc_id)
        .await
        .expect("the plan after the correction");
    assert_eq!(plan.revision, doc.revision);
    assert!(
        plan.cues
            .iter()
            .flat_map(|cue| cue.lines.iter())
            .flat_map(|line| line.words.iter())
            .any(|word| word.word_id == word_id && word.text == corrected_text),
        "the player shows the correction"
    );
    let corrected_bare = bare(&corrected_text).to_owned();

    // ---- Trim both ends. ----
    // Where a person trims: in to the start of the second sentence, out to
    // the end of the second-to-last. Both are word boundaries — a cut inside
    // a word is one the export refuses, and the editor snaps to a word for
    // the same reason — and what remains is whole sentences, which is what a
    // person cutting a clip down keeps. The document's cues are in program
    // ticks; the trim speaks source ticks, the segment's own window.
    let sentences = document(&doc)["captions"]["cues"]
        .as_array()
        .expect("reading cues")
        .clone();
    assert!(
        sentences.len() >= 4,
        "the clip has sentences to trim between: {}",
        sentences.len()
    );
    let head_cut = sentences[1]["start_ticks"].as_i64().unwrap();
    let keep_until = sentences[sentences.len() - 2]["end_ticks"]
        .as_i64()
        .unwrap();
    let trimmed_in = in_ticks + head_cut;
    let trimmed_out = in_ticks + keep_until;
    assert!(head_cut > 0 && trimmed_out < out_ticks && trimmed_out > trimmed_in);
    let (doc, _) = apply(
        &client,
        &doc,
        json!({"op": "trim", "segment_id": segment_id, "in_ticks": trimmed_in, "out_ticks": out_ticks}),
    )
    .await;
    let (doc, undo_tail) = apply(
        &client,
        &doc,
        json!({"op": "trim", "segment_id": segment_id, "in_ticks": trimmed_in, "out_ticks": trimmed_out}),
    )
    .await;
    assert_eq!(
        segment_of(&document(&doc)),
        (segment_id.clone(), trimmed_in, trimmed_out)
    );
    let trimmed = document(&doc);
    let remaining = trimmed["captions"]["cues"].as_array().unwrap();
    assert_eq!(
        remaining.len(),
        sentences.len() - 2,
        "one sentence gone from each end"
    );
    assert_eq!(
        remaining[0]["start_ticks"], 0,
        "the first remaining sentence now opens the clip"
    );
    assert_eq!(
        remaining[remaining.len() - 1]["end_ticks"],
        trimmed_out - trimmed_in,
        "the last remaining sentence now closes it"
    );
    for presentation in ["cues", "burn_in"] {
        assert!(
            trimmed["captions"][presentation]
                .as_array()
                .unwrap()
                .iter()
                .all(|cue| cue["start_ticks"].as_i64().unwrap() >= 0
                    && cue["end_ticks"].as_i64().unwrap() <= trimmed_out - trimmed_in),
            "every {presentation} cue lies inside the trimmed program"
        );
    }

    // ---- Undo the tail trim, then redo it. ----
    let (doc, redo_tail) = apply(&client, &doc, undo_tail).await;
    assert_eq!(
        segment_of(&document(&doc)),
        (segment_id.clone(), trimmed_in, out_ticks),
        "undo restores the tail"
    );
    let (doc, _) = apply(&client, &doc, redo_tail).await;
    assert_eq!(
        segment_of(&document(&doc)),
        (segment_id.clone(), trimmed_in, trimmed_out),
        "redo cuts it again"
    );
    let reviewed_revision = doc.revision;
    assert_eq!(reviewed_revision, 5, "correction, two trims, undo, redo");

    // ---- Restart. ----
    daemon.stop();
    let mut daemon = DaemonUnderTest::start(&directory);
    let client = daemon.client();
    wait_for_health(&client, Duration::from_secs(30)).await;
    // The fleet reconnects on its own; the roster is real state, not a memory.
    wait_for_readiness(&client, Duration::from_mins(1)).await;

    let restored = client
        .list_edit_docs(&older.project_id)
        .await
        .expect("documents after the restart");
    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].doc_id, doc.doc_id);
    assert_eq!(restored[0].revision, reviewed_revision);
    assert_eq!(
        segment_of(&document(&restored[0])),
        (segment_id.clone(), trimmed_in, trimmed_out)
    );
    let jobs = client
        .list_jobs(&older.project_id)
        .await
        .expect("jobs after the restart");
    assert!(
        jobs.iter()
            .any(|job| job.job_id == older.job.job_id && job.state == JobState::Succeeded as i32),
        "the run's real state came back"
    );
    let reopened = client
        .direct_clip(DirectClipRequest {
            project_id: older.project_id.clone(),
            source_id: older.source_id.clone(),
            candidate_id: candidate_id.clone(),
            cut: ClipCutV1::Chosen as i32,
            style_ref: String::new(),
            start_ticks: 0,
            end_ticks: 0,
            variation: false,
            approve: true,
            job_id: older.job.job_id.clone(),
        })
        .await
        .expect("reopen after the restart");
    assert!(reopened.reopened);
    assert_eq!(
        (reopened.start_ticks as i64, reopened.end_ticks as i64),
        (trimmed_in, trimmed_out),
        "a reopened clip reports where its segment stands now"
    );

    // ---- Export the reviewed revision. ----
    // Delivered where the drill asked, so the milestone is a file somebody
    // can watch rather than a passing test; under the data directory
    // otherwise, and gone with it.
    let destination = std::env::var_os("CLIPMILL_M1_DELIVERED_DIR")
        .map_or_else(|| directory.join("delivered"), PathBuf::from);
    let _ = fs::remove_dir_all(&destination);
    fs::create_dir_all(&destination).expect("destination");
    let request = ExportRequestV1 {
        doc_id: doc.doc_id.clone(),
        destination_dir: destination.to_str().unwrap().to_owned(),
        naming_pattern: String::new(),
        source_attestation: "own_content".to_owned(),
        gates_passed: vec!["duration_60s".to_owned()],
        ai_assistance: vec!["asr_captions".to_owned()],
        index: 1,
        date: "2026-09-17".to_owned(),
        title: "older clip".to_owned(),
        expected_revision: Some(reviewed_revision),
    };
    let planned = client
        .plan_export(request.clone())
        .await
        .expect("plan the export");
    assert_eq!(planned.revision, reviewed_revision);
    let validation = planned.validation.expect("a validation");
    assert!(
        validation.passes,
        "the plan blocks nothing: {:?}",
        validation.findings
    );
    // A stale review is refused, before anything is rendered.
    let stale = client
        .export_clip(ExportRequestV1 {
            expected_revision: Some(reviewed_revision - 1),
            ..request.clone()
        })
        .await;
    assert!(
        stale.is_err(),
        "an export of a revision nobody reviewed was accepted"
    );

    let queued = client.export_clip(request).await.expect("queue the export");
    assert_eq!(queued.revision, reviewed_revision);
    assert!(!queued.ir_artifact_id.is_empty());
    let delivered = wait_for_job(&client, &queued.job_id, EXPORT_TIMEOUT).await;
    assert_eq!(
        delivered.state,
        JobState::Succeeded as i32,
        "the export failed: {}",
        delivered.failure_detail
    );
    let package_id = output_of(&delivered, "deliver-export");
    let (kind, package_json) = client
        .read_document(&older.project_id, &package_id)
        .await
        .expect("read the package");
    assert_eq!(kind, "export.package.v1");
    let package: Value = serde_json::from_str(&package_json).expect("the package parses");
    assert_eq!(package["doc_id"], doc.doc_id);
    let files = package["files"].as_array().expect("delivered files");
    let file_named = |role: &str| -> PathBuf {
        let name = files
            .iter()
            .find(|file| file["role"] == role)
            .unwrap_or_else(|| panic!("no {role} in the package"))["name"]
            .as_str()
            .unwrap();
        let path = Path::new(&queued.destination_dir).join(name);
        assert!(path.is_file(), "{} was not delivered", path.display());
        path
    };
    let clip = file_named("clip");
    let srt = file_named("subtitles_srt");
    let vtt = file_named("subtitles_vtt");
    let manifest_path = file_named("render_manifest");
    file_named("metadata");
    file_named("checksums");
    file_named("thumbnail");

    // ---- The revision that was reviewed is the one that was rendered. ----
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    assert_eq!(manifest["ir_artifact_id"], queued.ir_artifact_id);
    let rendered_segment = &manifest["program"]["segments"][0];
    assert_eq!(rendered_segment["segment_id"], segment_id);
    assert_eq!(rendered_segment["in_ticks"], trimmed_in);
    assert_eq!(rendered_segment["out_ticks"], trimmed_out);
    assert_eq!(
        rendered_segment["source_fingerprint"],
        older.source_fingerprint
    );
    assert_eq!(
        client.list_edit_docs(&older.project_id).await.unwrap()[0].revision,
        reviewed_revision,
        "exporting changed nothing"
    );

    // ---- The footage and its timing, read back from the delivered frames. ----
    let expected_seconds = (trimmed_out - trimmed_in) as f64 / TICKS as f64;
    let duration = probe_duration_seconds(&clip);
    assert!(
        (duration - expected_seconds).abs() < 0.2,
        "the clip runs {duration:.2} s; the trimmed program is {expected_seconds:.2} s"
    );
    let clip_start = trimmed_in as f64 / TICKS as f64;
    let mut sampled = Vec::new();
    for at in [0.5, expected_seconds / 2.0, expected_seconds - 0.5] {
        let from = source_second_at(&clip, at);
        let wanted = clip_start + at;
        assert!(
            (from - wanted).abs() <= TIMING_TOLERANCE_SECONDS,
            "the frame {at:.2} s into the clip came from source second {from:.2}; \
             the trimmed clip asked for {wanted:.2}"
        );
        sampled.push(format!("{at:.2}s→{from:.2}s"));
    }
    // The reading is sensitive: the same colour read back from the source at
    // the wrong second names the wrong second.
    let elsewhere = source_second_at(&recording, clip_start - 30.0);
    assert!(
        (elsewhere - (clip_start - 30.0)).abs() <= TIMING_TOLERANCE_SECONDS
            && (elsewhere - clip_start).abs() > 10.0
    );

    // ---- Both caption outputs carry the correction. ----
    // The word may be said more than once in the clip; exactly one saying of
    // it was corrected, so each output carries the correction once and the
    // heard word one time fewer than the document did.
    let srt_text = fs::read_to_string(&srt).unwrap();
    let vtt_text = fs::read_to_string(&vtt).unwrap();
    let ass = artifact_file(
        &directory.join("artifacts"),
        package["render_artifact_id"].as_str().unwrap(),
        "clip.ass",
    );
    let ass_text = fs::read_to_string(&ass).expect("the burned-in track the encoder was handed");
    for (name, text) in [("SRT", &srt_text), ("VTT", &vtt_text), ("ASS", &ass_text)] {
        assert_eq!(
            count_word(text, &corrected_bare),
            1,
            "the {name} carries the correction once"
        );
        assert_eq!(
            count_word(text, &heard_bare),
            heard_before - 1,
            "the {name} says the heard word one time fewer"
        );
    }
    // And the captions were retimed with the trim: the first burned-in cue
    // does not begin before the picture does.
    assert!(
        manifest["caption_windows"]
            .as_array()
            .unwrap()
            .iter()
            .all(|window| window["first_frame"].as_i64().unwrap() >= 0
                && window["end_frame"].as_i64().unwrap()
                    <= manifest["program"]["frame_count"].as_i64().unwrap()),
        "every burned-in cue lies inside the delivered program"
    );

    println!(
        "milestone-1: clip {}–{} s of the older project's recording, \"{heard}\" corrected to \
         \"{corrected_text}\" in both groupings, trimmed to {:.2}–{:.2} s, revision \
         {reviewed_revision} exported to {}; sampled frames {} (clip second → source second)",
        in_ticks / TICKS,
        out_ticks / TICKS,
        clip_start,
        trimmed_out as f64 / TICKS as f64,
        clip.display(),
        sampled.join(", ")
    );

    drop(protocol);
    drop(workers);
    daemon.stop();
    let _ = fs::remove_dir_all(&directory);
}
