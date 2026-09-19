//! Bounded process transport for the explicitly requested `YouTube` importer.
//! No shell command contains a URL, credential or renderer-supplied path.

use std::{path::PathBuf, process::Stdio, time::Duration};

use serde::Deserialize;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    process::{Child, Command},
    sync::{mpsc, watch},
};

const DOWNLOAD_LIMIT: Duration = Duration::from_hours(2);
const OUTPUT_LIMIT: u64 = 16 * 1024 * 1024;
const MAX_BYTES: u64 = 8 * 1024 * 1024 * 1024;

struct ProcessGroup(Option<u32>);

impl Drop for ProcessGroup {
    fn drop(&mut self) {
        // Tokio task cancellation drops the future without reaching its normal
        // asynchronous termination path. Kill descendants too in that case.
        if let Some(pid) = self.0.take() {
            let _ = std::process::Command::new("/bin/kill")
                .args(["-KILL", &format!("-{pid}")])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct YoutubeDownloader {
    helper: PathBuf,
    ffmpeg: PathBuf,
}

#[derive(Clone, Debug)]
pub(crate) struct DownloadProgress {
    pub state: String,
    pub title: Option<String>,
    pub channel: Option<String>,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
}

#[derive(Debug)]
pub(crate) struct DownloadedVideo {
    pub path: PathBuf,
    pub video_id: String,
    pub title: String,
    pub channel: String,
    pub duration_seconds: f64,
    pub byte_size: u64,
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub(crate) struct DownloadError {
    pub code: String,
    pub message: String,
}

impl DownloadError {
    fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
enum Event {
    Ready,
    Metadata {
        title: String,
        channel: String,
    },
    Progress {
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
    },
    Processing,
    Complete {
        file_name: String,
        byte_size: u64,
        video_id: String,
        title: String,
        channel: String,
        duration_seconds: f64,
    },
    Error {
        code: String,
        message: String,
    },
}

impl YoutubeDownloader {
    pub(crate) fn new(helper: PathBuf, ffmpeg: PathBuf) -> Self {
        Self { helper, ffmpeg }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(&self.helper);
        command
            .arg("--ffmpeg")
            .arg(&self.ffmpeg)
            .env_clear()
            .env("PATH", std::env::var_os("PATH").unwrap_or_default())
            .env("LANG", "en_US.UTF-8")
            .env("YTDLP_NO_PLUGINS", "1")
            .env("PYTHONNOUSERSITE", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        if let Some(node) = std::env::var_os("CLIPMILL_NODE") {
            command.env("CLIPMILL_NODE", node);
        }
        // The helper's FFmpeg and JavaScript descendants inherit this group.
        command.process_group(0);
        command
    }

    pub(crate) async fn check(&self) -> Result<(), DownloadError> {
        let (updates, _receiver) = mpsc::channel(1);
        let (_cancel, receiver) = watch::channel(false);
        self.run(
            self.command().arg("--check"),
            None,
            updates,
            receiver,
            Duration::from_secs(10),
        )
        .await
        .map(|_| ())
    }

    pub(crate) async fn download(
        &self,
        operation_dir: PathBuf,
        canonical_url: String,
        updates: mpsc::Sender<DownloadProgress>,
        cancel: watch::Receiver<bool>,
        max_height: u32,
    ) -> Result<DownloadedVideo, DownloadError> {
        if !matches!(max_height, 360 | 720 | 1080) {
            return Err(DownloadError::new(
                "invalid_quality",
                "Choose 360p, 720p or 1080p for the download.",
            ));
        }
        let mut command = self.command();
        command.arg("--max-height").arg(max_height.to_string());
        command
            .arg("--url")
            .arg(canonical_url)
            .arg("--destination")
            .arg(&operation_dir);
        self.run(
            &mut command,
            Some(operation_dir),
            updates,
            cancel,
            DOWNLOAD_LIMIT,
        )
        .await?
        .ok_or_else(protocol_error)
    }

    #[allow(clippy::too_many_lines)]
    async fn run(
        &self,
        command: &mut Command,
        directory: Option<PathBuf>,
        updates: mpsc::Sender<DownloadProgress>,
        mut cancel: watch::Receiver<bool>,
        deadline: Duration,
    ) -> Result<Option<DownloadedVideo>, DownloadError> {
        if *cancel.borrow() {
            return Err(cancelled());
        }
        let mut child = command.spawn().map_err(|_| {
            DownloadError::new(
                "setup_required",
                "YouTube import needs setup. Run just setup-youtube, then retry.",
            )
        })?;
        let process_group = child.id();
        let mut group_guard = ProcessGroup(process_group);
        let Some(stdout) = child.stdout.take() else {
            terminate(&mut child, process_group).await;
            return Err(protocol_error());
        };
        let mut lines = BufReader::new(stdout.take(OUTPUT_LIMIT + 1)).lines();
        let task = async {
            let mut complete = None;
            let mut ready = false;
            let mut bytes_read = 0_u64;
            let mut downloaded_bytes = 0_u64;
            while let Some(line) = lines.next_line().await.map_err(|_| protocol_error())? {
                bytes_read += line.len() as u64 + 1;
                if line.len() > 8192 || bytes_read > OUTPUT_LIMIT {
                    return Err(protocol_error());
                }
                let event: Event = serde_json::from_str(&line).map_err(|_| protocol_error())?;
                let progress = match event {
                    Event::Ready => {
                        ready = true;
                        None
                    }
                    Event::Metadata { title, channel } => Some(DownloadProgress {
                        state: "downloading".into(),
                        title: Some(clean_text(&title, 300)),
                        channel: Some(clean_text(&channel, 200)),
                        downloaded_bytes,
                        total_bytes: None,
                    }),
                    Event::Progress {
                        downloaded_bytes: value,
                        total_bytes,
                    } => {
                        if value > MAX_BYTES
                            || total_bytes.is_some_and(|v| v > MAX_BYTES || v < value)
                        {
                            return Err(protocol_error());
                        }
                        downloaded_bytes = downloaded_bytes.max(value);
                        Some(DownloadProgress {
                            state: "downloading".into(),
                            title: None,
                            channel: None,
                            downloaded_bytes,
                            total_bytes,
                        })
                    }
                    Event::Processing => Some(DownloadProgress {
                        state: "processing".into(),
                        title: None,
                        channel: None,
                        downloaded_bytes,
                        total_bytes: None,
                    }),
                    Event::Complete {
                        file_name,
                        byte_size,
                        video_id,
                        title,
                        channel,
                        duration_seconds,
                    } => {
                        if complete.is_some()
                            || file_name != "source.mkv"
                            || byte_size == 0
                            || byte_size > MAX_BYTES
                            || video_id.len() != 11
                            || !video_id
                                .bytes()
                                .all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))
                            || !duration_seconds.is_finite()
                            || duration_seconds <= 0.0
                            || duration_seconds > 21_600.0
                        {
                            return Err(protocol_error());
                        }
                        let path = directory
                            .as_ref()
                            .ok_or_else(protocol_error)?
                            .join(file_name);
                        let metadata =
                            std::fs::symlink_metadata(&path).map_err(|_| protocol_error())?;
                        if !metadata.is_file() || metadata.len() != byte_size {
                            return Err(protocol_error());
                        }
                        complete = Some(DownloadedVideo {
                            path,
                            video_id,
                            title: clean_text(&title, 300),
                            channel: clean_text(&channel, 200),
                            duration_seconds,
                            byte_size,
                        });
                        None
                    }
                    Event::Error { code, message } => {
                        return Err(safe_helper_error(&code, &message));
                    }
                };
                if let Some(progress) = progress {
                    // A slow DB/UI must not block reading a child's pipe. Newer
                    // byte counts supersede dropped progress; completion is separate.
                    let _sent = updates.try_send(progress);
                }
            }
            if child.wait().await.map_err(|_| protocol_error())?.success()
                && ((directory.is_some() && complete.is_some()) || (directory.is_none() && ready))
            {
                Ok(complete)
            } else {
                Err(protocol_error())
            }
        };
        let result = tokio::select! {
            result = tokio::time::timeout(deadline, task) => result.unwrap_or_else(|_| Err(DownloadError::new("timeout", "The YouTube import timed out. Check your connection and retry."))),
            _ = cancel.changed() => Err(cancelled()),
        };
        // Also reap descendants if the helper exited with an error, or left a
        // pipe open. The process group belongs only to this import attempt.
        if result.is_err() {
            terminate(&mut child, process_group).await;
        }
        group_guard.0 = None;
        result
    }
}

fn clean_text(value: &str, limit: usize) -> String {
    value
        .chars()
        .filter(|c| !c.is_control())
        .take(limit)
        .collect()
}

fn safe_helper_error(code: &str, message: &str) -> DownloadError {
    const ALLOWED: &[&str] = &[
        "setup_required",
        "invalid_url",
        "invalid_quality",
        "invalid_video",
        "live_video",
        "invalid_duration",
        "duration_limit",
        "protected_video",
        "size_limit",
        "storage_error",
        "disk_full",
        "restricted_video",
        "unavailable",
        "download_failed",
    ];
    if ALLOWED.contains(&code) && !message.contains("://") && message.len() <= 400 {
        DownloadError::new(code, &clean_text(message, 400))
    } else {
        protocol_error()
    }
}

fn protocol_error() -> DownloadError {
    DownloadError::new(
        "download_failed",
        "The importer did not produce a verified source file. Retry the import or repair its setup.",
    )
}

fn cancelled() -> DownloadError {
    DownloadError::new("cancelled", "YouTube import cancelled.")
}

async fn terminate(child: &mut Child, process_group: Option<u32>) {
    if let Some(pid) = process_group {
        let group = format!("-{pid}");
        let _signal = Command::new("/bin/kill")
            .args(["-TERM", &group])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await;
        tokio::time::sleep(Duration::from_millis(150)).await;
        let _signal = Command::new("/bin/kill")
            .args(["-KILL", &group])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await;
    }
    let _kill = child.start_kill();
    let _reaped = child.wait().await;
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::{fs, os::unix::fs::PermissionsExt, path::Path};

    fn helper(root: &Path, script: &str) -> YoutubeDownloader {
        let path = root.join("helper.sh");
        fs::write(&path, format!("#!/bin/sh\n{script}\n")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        YoutubeDownloader::new(path, PathBuf::from("/fixed/ffmpeg"))
    }

    #[tokio::test]
    async fn readiness_needs_both_protocol_and_successful_exit() {
        let root = tempfile::tempdir().unwrap();
        helper(root.path(), "printf '%s\\n' '{\"event\":\"ready\"}'")
            .check()
            .await
            .unwrap();
        assert!(
            helper(
                root.path(),
                "printf '%s\\n' '{\"event\":\"ready\"}'; exit 1"
            )
            .check()
            .await
            .is_err()
        );
        assert!(helper(root.path(), "exit 0").check().await.is_err());
    }

    #[tokio::test]
    async fn completed_response_cannot_escape_the_attempt_directory() {
        let root = tempfile::tempdir().unwrap();
        let script = "printf '%s\\n' '{\"event\":\"complete\",\"file_name\":\"../outside.mkv\",\"byte_size\":4,\"video_id\":\"NYFGCESmikA\",\"title\":\"Title\",\"channel\":\"Creator\",\"duration_seconds\":4}'";
        let (tx, _rx) = mpsc::channel(4);
        let (_cancel, rx) = watch::channel(false);
        let result = helper(root.path(), script)
            .download(root.path().into(), "url".into(), tx, rx, 1080)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn completion_is_not_success_without_the_final_regular_file() {
        let root = tempfile::tempdir().unwrap();
        let script = "printf '%s\\n' '{\"event\":\"complete\",\"file_name\":\"source.mkv\",\"byte_size\":4,\"video_id\":\"NYFGCESmikA\",\"title\":\"Title\",\"channel\":\"Creator\",\"duration_seconds\":4}'";
        let downloader = helper(root.path(), script);
        for present in [false, true] {
            if present {
                fs::write(root.path().join("source.mkv"), b"data").unwrap();
            }
            let (tx, _rx) = mpsc::channel(4);
            let (_cancel, rx) = watch::channel(false);
            let result = downloader
                .download(root.path().into(), "url".into(), tx, rx, 1080)
                .await;
            assert_eq!(result.is_ok(), present);
        }
    }

    #[tokio::test]
    async fn cancellation_reaps_the_merger_process_group() {
        let root = tempfile::tempdir().unwrap();
        let pid_path = root.path().join("descendant.pid");
        let script = format!("sleep 60 &\necho $! > '{}'\nwait", pid_path.display());
        let downloader = helper(root.path(), &script);
        let (tx, _rx) = mpsc::channel(4);
        let (cancel, rx) = watch::channel(false);
        let directory = root.path().to_path_buf();
        let task = tokio::spawn(async move {
            downloader
                .download(directory, "url".into(), tx, rx, 1080)
                .await
        });
        for _ in 0..500 {
            if pid_path.exists() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        let descendant = fs::read_to_string(pid_path).unwrap();
        cancel.send(true).unwrap();
        let result = tokio::time::timeout(Duration::from_secs(3), task)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result.unwrap_err().code, "cancelled");
        let status = Command::new("/bin/kill")
            .args(["-0", descendant.trim()])
            .stderr(Stdio::null())
            .status()
            .await
            .unwrap();
        assert!(!status.success(), "download child survived cancellation");
    }

    #[tokio::test]
    async fn aborting_the_transport_future_also_kills_descendants() {
        let root = tempfile::tempdir().unwrap();
        let pid_path = root.path().join("descendant.pid");
        let script = format!("sleep 60 &\necho $! > '{}'\nwait", pid_path.display());
        let downloader = helper(root.path(), &script);
        let (tx, _rx) = mpsc::channel(4);
        let (_cancel, rx) = watch::channel(false);
        let directory = root.path().to_path_buf();
        let task = tokio::spawn(async move {
            downloader
                .download(directory, "url".into(), tx, rx, 1080)
                .await
        });
        for _ in 0..500 {
            if pid_path.exists() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        let descendant = fs::read_to_string(pid_path).unwrap();
        task.abort();
        let _ = task.await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        let status = Command::new("/bin/kill")
            .args(["-0", descendant.trim()])
            .stderr(Stdio::null())
            .status()
            .await
            .unwrap();
        assert!(!status.success(), "download child survived task abort");
    }

    #[test]
    fn untrusted_helper_errors_cannot_expose_signed_urls() {
        let error = safe_helper_error("download_failed", "https://media.invalid?token=secret");
        assert!(!error.message.contains("secret"));
    }
}
