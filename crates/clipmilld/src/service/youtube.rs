//! Explicit network acquisition, fenced by the durable import attempt.
use super::{Reply, Service, error_reply, response_reply, store_error_reply, unix_millis};
use crate::{
    db::YoutubeCommand,
    youtube_transport::{DownloadedVideo, YoutubeDownloader},
};
use clipmill_contracts::proto::ipc::v1::{
    ErrorCode, ListYoutubeImportsResponse, StartYoutubeImportRequest, UpdateYoutubeImportRequest,
    YoutubeImportResponse, YoutubeImportV1, response,
};
use clipmill_core::{ProjectId, SourceId};
use std::{
    collections::HashMap,
    fs,
    os::unix::fs::{DirBuilderExt, MetadataExt},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
use tokio::sync::{Mutex, Semaphore, mpsc, watch};

type Running = HashMap<(String, u32), (String, watch::Sender<bool>, tokio::task::JoinHandle<()>)>;

#[derive(Debug)]
pub(crate) struct YoutubeRuntime {
    downloader: YoutubeDownloader,
    root: PathBuf,
    running: Mutex<Running>,
    slots: Semaphore,
    stopping: AtomicBool,
}

impl YoutubeRuntime {
    pub(crate) fn new(downloader: YoutubeDownloader, root: PathBuf) -> Self {
        Self {
            downloader,
            root,
            running: Mutex::new(HashMap::new()),
            slots: Semaphore::new(2),
            stopping: AtomicBool::new(false),
        }
    }
}

impl Service {
    pub(crate) async fn recover_youtube_imports(&self) -> Result<(), crate::db::StoreError> {
        self.database
            .youtube_imports(YoutubeCommand::Recover {
                now: unix_millis().unwrap_or(0),
            })
            .await?;
        if let Some(runtime) = &self.youtube {
            let records = self
                .database
                .youtube_imports(YoutubeCommand::List {
                    project_id: String::new(),
                })
                .await?;
            let mut cleanup = Vec::new();
            for record in records {
                let retained = if record.state == "completed" {
                    // Completed records may also have older interrupted attempts.
                    // Only the source registered by the successful attempt stays.
                    let Ok(source) = self.database.get_source(record.source_id).await else {
                        continue;
                    };
                    PathBuf::from(source.observation.absolute_path)
                        .parent()
                        .map(Path::to_path_buf)
                } else {
                    None
                };
                cleanup.push((
                    runtime.root.join(record.project_id).join(record.import_id),
                    retained,
                ));
            }
            let _ = tokio::task::spawn_blocking(move || {
                for (directory, retained) in cleanup {
                    cleanup_abandoned_except(&directory, retained.as_deref());
                }
            })
            .await;
            // A deleted project's old helper may still hold its lease when the
            // deletion returns. Its rows are gone, so revisit managed directories
            // as well as saved imports after restart. Startup has not admitted
            // new requests yet, and database absence is checked before cleanup.
            let root = runtime.root.clone();
            let directories = tokio::task::spawn_blocking(move || managed_projects(&root))
                .await
                .unwrap_or_default();
            for (id, directory) in directories {
                if matches!(
                    self.database.get_project(id).await,
                    Err(crate::db::StoreError::NotFound)
                ) {
                    let _ = tokio::task::spawn_blocking(move || cleanup_project(&directory)).await;
                }
            }
        }
        Ok(())
    }

    pub(crate) async fn stop_youtube_imports(&self) {
        let Some(runtime) = &self.youtube else {
            return;
        };
        runtime.stopping.store(true, Ordering::SeqCst);
        let running = std::mem::take(&mut *runtime.running.lock().await);
        for (_, cancel, _) in running.values() {
            let _ = cancel.send(true);
        }
        let drain = async {
            for (_, (_, _, task)) in running {
                let _ = task.await;
            }
        };
        let _ = tokio::time::timeout(std::time::Duration::from_secs(10), drain).await;
    }

    pub(super) async fn cleanup_youtube_project(&self, project_id: &str) {
        let Some(runtime) = &self.youtube else {
            return;
        };
        let mut running = runtime.running.lock().await;
        let keys: Vec<_> = running
            .iter()
            .filter(|(_, entry)| entry.0 == project_id)
            .map(|(key, _)| key.clone())
            .collect();
        let tasks: Vec<_> = keys
            .into_iter()
            .filter_map(|key| running.remove(&key))
            .collect();
        drop(running);
        for (_, cancel, _) in &tasks {
            let _ = cancel.send(true);
        }
        let drain = async {
            for (_, _, task) in tasks {
                let _ = task.await;
            }
        };
        // If a helper cannot be reaped yet, keep its private files. The source
        // registration fence already prevents it from resurrecting this project.
        if tokio::time::timeout(std::time::Duration::from_secs(5), drain)
            .await
            .is_ok()
        {
            let directory = runtime.root.join(project_id);
            let _ = tokio::task::spawn_blocking(move || cleanup_project(&directory)).await;
        }
    }

    pub(super) async fn start_youtube_import(
        &self,
        id: String,
        hash: [u8; 32],
        asked: StartYoutubeImportRequest,
    ) -> Reply {
        let Some(max_height) = import_height(asked.max_height) else {
            return error_reply(
                id,
                ErrorCode::InvalidArgument,
                "Choose 360p, 720p or 1080p import quality.",
            );
        };
        if !asked.rights_confirmed {
            return error_reply(
                id,
                ErrorCode::InvalidArgument,
                "Confirm that you have permission to download and edit this video.",
            );
        }
        let Ok(project) = asked.project_id.parse::<ProjectId>() else {
            return error_reply(
                id,
                ErrorCode::InvalidArgument,
                "Choose a project for this import.",
            );
        };
        if let Err(error) = self.database.get_project(project.to_string()).await {
            return store_error_reply(id, &error);
        }
        let Some(runtime) = &self.youtube else {
            return error_reply(
                id,
                ErrorCode::Unavailable,
                "YouTube import is not available in this daemon.",
            );
        };
        if runtime.stopping.load(Ordering::SeqCst) {
            return error_reply(
                id,
                ErrorCode::Unavailable,
                "The app is closing. Reopen it to import.",
            );
        }
        let Some(video_id) = youtube_id(&asked.url) else {
            return error_reply(
                id,
                ErrorCode::InvalidArgument,
                "Paste a single YouTube video URL, such as https://youtu.be/VIDEO_ID.",
            );
        };
        let now = unix_millis().unwrap_or(0);
        let record = YoutubeImportV1 {
            import_id: format!("yt_{}", ulid::Ulid::new()),
            project_id: project.to_string(),
            canonical_url: format!("https://www.youtube.com/watch?v={video_id}"),
            video_id,
            title: String::new(),
            channel: String::new(),
            state: "queued".to_owned(),
            attempt: 0,
            downloaded_bytes: 0,
            total_bytes: None,
            source_id: String::new(),
            error_code: String::new(),
            error: String::new(),
            created_unix_millis: now,
            updated_unix_millis: now,
            max_height,
        };
        match self
            .database
            .youtube_imports(YoutubeCommand::Create {
                request_id: id.clone(),
                request_hash: hash,
                record,
            })
            .await
        {
            Ok(mut records) => {
                let Some(record) = records.pop() else {
                    return error_reply(id, ErrorCode::Internal, "The import could not be saved.");
                };
                self.dispatch_youtube(&record).await;
                import_reply(id, record)
            }
            Err(error) => store_error_reply(id, &error),
        }
    }

    pub(super) async fn get_youtube_import(&self, id: String, import_id: String) -> Reply {
        match self
            .database
            .youtube_imports(YoutubeCommand::Read { id: import_id })
            .await
        {
            Ok(mut records) => records.pop().map_or_else(
                || error_reply(id.clone(), ErrorCode::NotFound, "Import not found"),
                |record| import_reply(id.clone(), record),
            ),
            Err(error) => store_error_reply(id, &error),
        }
    }

    pub(super) async fn list_youtube_imports(&self, id: String, project_id: String) -> Reply {
        match self
            .database
            .youtube_imports(YoutubeCommand::List { project_id })
            .await
        {
            Ok(imports) => response_reply(
                id,
                response::Body::ListYoutubeImports(ListYoutubeImportsResponse { imports }),
            ),
            Err(error) => store_error_reply(id, &error),
        }
    }

    pub(super) async fn update_youtube_import(
        &self,
        id: String,
        hash: [u8; 32],
        asked: UpdateYoutubeImportRequest,
    ) -> Reply {
        if !matches!(asked.action.as_str(), "cancel" | "retry") {
            return error_reply(id, ErrorCode::InvalidArgument, "Choose cancel or retry.");
        }
        match self
            .database
            .youtube_imports(YoutubeCommand::Update {
                request_id: id.clone(),
                request_hash: hash,
                id: asked.import_id,
                action: asked.action.clone(),
                now: unix_millis().unwrap_or(0),
            })
            .await
        {
            Ok(mut records) => {
                let Some(record) = records.pop() else {
                    return error_reply(id, ErrorCode::NotFound, "Import not found");
                };
                if asked.action == "cancel" {
                    if let Some(runtime) = &self.youtube
                        && let Some((_, cancel, _)) = runtime
                            .running
                            .lock()
                            .await
                            .get(&(record.import_id.clone(), record.attempt))
                    {
                        let _ = cancel.send(true);
                    }
                } else {
                    self.dispatch_youtube(&record).await;
                }
                import_reply(id, record)
            }
            Err(error) => store_error_reply(id, &error),
        }
    }

    async fn dispatch_youtube(&self, record: &YoutubeImportV1) {
        let Some(runtime) = &self.youtube else {
            return;
        };
        if record.state != "queued" || runtime.stopping.load(Ordering::SeqCst) {
            return;
        }
        let mut running = runtime.running.lock().await;
        if runtime.stopping.load(Ordering::SeqCst) {
            return;
        }
        let key = (record.import_id.clone(), record.attempt);
        if running.contains_key(&key) {
            return;
        }
        let (cancel, receiver) = watch::channel(false);
        let service = self.clone();
        let project = record.project_id.clone();
        let record = record.clone();
        let task = tokio::spawn(async move {
            service.run_youtube_import(record, receiver).await;
        });
        running.insert(key, (project, cancel, task));
    }

    async fn run_youtube_import(
        &self,
        mut record: YoutubeImportV1,
        mut cancel: watch::Receiver<bool>,
    ) {
        let Some(runtime) = &self.youtube else {
            return;
        };
        let permit = tokio::select! {
            value = runtime.slots.acquire() => value.ok(),
            _ = cancel.changed() => None,
        };
        if permit.is_some() && !*cancel.borrow() {
            let claimed = self
                .database
                .youtube_imports(YoutubeCommand::Claim {
                    id: record.import_id.clone(),
                    attempt: record.attempt,
                    now: unix_millis().unwrap_or(0),
                })
                .await;
            if let Ok(mut records) = claimed
                && let Some(saved) = records.pop()
            {
                record = saved;
                self.download_youtube(&mut record, &mut cancel).await;
            }
        }
        runtime
            .running
            .lock()
            .await
            .remove(&(record.import_id, record.attempt));
    }

    #[allow(
        clippy::too_many_lines,
        reason = "single bounded download and source-inspection lifecycle"
    )]
    async fn download_youtube(
        &self,
        record: &mut YoutubeImportV1,
        cancel: &mut watch::Receiver<bool>,
    ) {
        let Some(runtime) = &self.youtube else {
            return;
        };
        if let Err(error) = runtime.downloader.check().await {
            self.fail_youtube(record, &error.code, &error.message).await;
            return;
        }
        if *cancel.borrow() {
            return;
        }
        let directory = runtime
            .root
            .join(&record.project_id)
            .join(&record.import_id)
            .join(format!("attempt-{}-{}", record.attempt, ulid::Ulid::new()));
        if private_directory(&runtime.root)
            .and_then(|()| private_directory(&runtime.root.join(&record.project_id)))
            .and_then(|()| {
                private_directory(
                    &runtime
                        .root
                        .join(&record.project_id)
                        .join(&record.import_id),
                )
            })
            .and_then(|()| fs::DirBuilder::new().mode(0o700).create(&directory))
            .is_err()
        {
            self.fail_youtube(
                record,
                "storage",
                "The managed import folder could not be created.",
            )
            .await;
            return;
        }
        let (updates, mut progress) = mpsc::channel(16);
        self.policy.note_task_start("youtube-import");
        let downloading = runtime.downloader.download(
            directory.clone(),
            record.canonical_url.clone(),
            updates,
            cancel.clone(),
            record.max_height,
        );
        tokio::pin!(downloading);
        let result = loop {
            tokio::select! {
                result = &mut downloading => break result,
                Some(update) = progress.recv() => {
                    let state = if update.state == "processing" { "processing" } else { "downloading" };
                    state.clone_into(&mut record.state);
                    if let Some(channel) = update.channel {
                        record.channel = safe_text(&channel, 200);
                    }
                    record.downloaded_bytes = update.downloaded_bytes;
                    record.total_bytes = update.total_bytes;
                    if let Some(title) = update.title {
                        record.title = safe_text(&title, 200);
                    }
                    record.updated_unix_millis = unix_millis().unwrap_or(0);
                    if self.database.youtube_imports(YoutubeCommand::Progress { record: record.clone() }).await.is_err()
                        && let Some((_, signal, _)) = runtime.running.lock().await.get(&(record.import_id.clone(), record.attempt)) {
                        let _ = signal.send(true);
                    }
                }
            }
        };
        let keep = match result {
            Ok(video) if !*cancel.borrow() => {
                let sync_path = video.path.clone();
                let sync_root = runtime.root.clone();
                let verified = valid_download(&directory, &video, &record.video_id);
                let durable = verified
                    && tokio::task::spawn_blocking(move || sync_download(&sync_path, &sync_root))
                        .await
                        .is_ok_and(|value| value.is_ok());
                if durable {
                    record.title = safe_text(&video.title, 200);
                    record.channel = safe_text(&video.channel, 200);
                    record.downloaded_bytes = video.byte_size;
                    record.total_bytes = Some(video.byte_size);
                    "registering".clone_into(&mut record.state);
                    record.updated_unix_millis = unix_millis().unwrap_or(0);
                    if self
                        .database
                        .youtube_imports(YoutubeCommand::Progress {
                            record: record.clone(),
                        })
                        .await
                        .is_ok()
                    {
                        self.register_youtube_download(record, &video.path, cancel)
                            .await
                    } else {
                        false
                    }
                } else {
                    self.fail_youtube(
                        record,
                        "invalid_download",
                        "The downloaded video could not be verified.",
                    )
                    .await;
                    false
                }
            }
            Ok(_) => {
                self.fail_youtube(
                    record,
                    "interrupted",
                    "Import interrupted. Retry when you are ready.",
                )
                .await;
                false
            }
            Err(error) => {
                self.fail_youtube(record, &error.code, &error.message).await;
                false
            }
        };
        if !keep {
            let _ = tokio::task::spawn_blocking(move || fs::remove_dir_all(directory)).await;
        }
    }

    async fn register_youtube_download(
        &self,
        record: &mut YoutubeImportV1,
        path: &Path,
        cancel: &mut watch::Receiver<bool>,
    ) -> bool {
        let Some(inspector) = &self.sources else {
            self.fail_youtube(
                record,
                "probe_unavailable",
                "Video inspection is unavailable. Restart the app and retry.",
            )
            .await;
            return false;
        };
        let inspect = async {
            let sampled = inspector
                .sample(path.to_string_lossy().into_owned())
                .await?;
            inspector
                .complete_cancellable(sampled, cancel.clone())
                .await
        };
        // Inspection owns and reaps its probe child before cancellation returns.
        let result = inspect.await;
        match result {
            Ok(inspection) if !*cancel.borrow() => match self
                .database
                .youtube_imports(YoutubeCommand::Complete {
                    id: record.import_id.clone(),
                    attempt: record.attempt,
                    source_id: SourceId::new().to_string(),
                    inspection: Box::new(inspection),
                    now: unix_millis().unwrap_or(0),
                })
                .await
            {
                Err(crate::db::StoreError::Conflict | crate::db::StoreError::NotFound) => false,
                // Preserve bytes if commit acknowledgement is uncertain. Never
                // delete a file that may already back a registered source.
                Ok(_) | Err(_) => true,
            },
            Err(_) if !*cancel.borrow() => {
                self.fail_youtube(
                    record,
                    "invalid_media",
                    "The downloaded file is not a usable video. Retry or choose a local file.",
                )
                .await;
                false
            }
            _ => {
                self.fail_youtube(
                    record,
                    "interrupted",
                    "Import interrupted. Retry when you are ready.",
                )
                .await;
                false
            }
        }
    }

    async fn fail_youtube(&self, record: &mut YoutubeImportV1, code: &str, message: &str) {
        let interrupted = self
            .youtube
            .as_ref()
            .is_some_and(|runtime| runtime.stopping.load(Ordering::SeqCst));
        if interrupted { "interrupted" } else { "failed" }.clone_into(&mut record.state);
        record.error_code = safe_text(code, 64);
        record.error = safe_text(message, 500);
        record.updated_unix_millis = unix_millis().unwrap_or(0);
        let _ = self
            .database
            .youtube_imports(YoutubeCommand::Progress {
                record: record.clone(),
            })
            .await;
    }
}

fn import_reply(id: String, record: YoutubeImportV1) -> Reply {
    response_reply(
        id,
        response::Body::YoutubeImport(YoutubeImportResponse {
            record: Some(record),
        }),
    )
}

fn import_height(value: u32) -> Option<u32> {
    match value {
        0 => Some(1080),
        360 | 720 | 1080 => Some(value),
        _ => None,
    }
}

fn private_directory(path: &Path) -> std::io::Result<()> {
    match fs::DirBuilder::new().mode(0o700).create(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            if fs::symlink_metadata(path)?.is_dir() {
                Ok(())
            } else {
                Err(std::io::Error::other("import directory is not a directory"))
            }
        }
        Err(error) => Err(error),
    }
}
fn valid_download(directory: &Path, video: &DownloadedVideo, id: &str) -> bool {
    video.video_id == id
        && video.path == directory.join("source.mkv")
        && video.duration_seconds.is_finite()
        && video.duration_seconds > 0.0
        && video.duration_seconds <= 21_600.0
        && fs::symlink_metadata(&video.path).is_ok_and(|meta| {
            meta.is_file()
                && meta.nlink() == 1
                && meta.len() == video.byte_size
                && meta.len() > 0
                && meta.len() <= 8 * 1024 * 1024 * 1024
        })
}
fn safe_text(value: &str, limit: usize) -> String {
    value
        .chars()
        .filter(|c| !c.is_control())
        .take(limit)
        .collect()
}

pub(super) fn youtube_id(url: &str) -> Option<String> {
    if url.len() > 2048 || url.chars().any(char::is_control) {
        return None;
    }
    let remainder = url.trim().strip_prefix("https://")?;
    let (host, path) = remainder.split_once('/')?;
    let path = path.split('#').next()?;
    let (route, query) = path.split_once('?').unwrap_or((path, ""));
    let id = match host {
        "youtu.be" => route,
        "youtube.com" | "www.youtube.com" | "m.youtube.com" if route == "watch" => {
            let mut ids = query.split('&').filter_map(|pair| pair.strip_prefix("v="));
            let id = ids.next()?;
            if ids.next().is_some() {
                return None;
            }
            id
        }
        "youtube.com" | "www.youtube.com" | "m.youtube.com" => route
            .strip_prefix("shorts/")
            .or_else(|| route.strip_prefix("embed/"))
            .or_else(|| route.strip_prefix("live/"))?,
        _ => return None,
    };
    (id.len() == 11
        && id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-'))
    .then(|| id.to_owned())
}

/// Cleanup only attempts whose helper-held lease can be acquired. Missing lock
/// files are left intact: absence is not proof that an orphan has stopped.
fn cleanup_abandoned(import_dir: &Path) {
    cleanup_abandoned_except(import_dir, None);
}

fn cleanup_abandoned_except(import_dir: &Path, retained: Option<&Path>) {
    use fs2::FileExt;
    use std::os::unix::fs::OpenOptionsExt;
    if !fs::symlink_metadata(import_dir).is_ok_and(|metadata| metadata.is_dir()) {
        return;
    }
    let Ok(entries) = fs::read_dir(import_dir) else {
        return;
    };
    for entry in entries.flatten() {
        if !entry.file_name().to_string_lossy().starts_with("attempt-")
            || !entry.file_type().is_ok_and(|kind| kind.is_dir())
        {
            continue;
        }
        let path = entry.path();
        if retained == Some(path.as_path()) {
            continue;
        }
        let Ok(lock) = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(nix::libc::O_NOFOLLOW)
            .open(path.join(".import.lock"))
        else {
            // A crash before helper spawn can leave an empty private attempt.
            // This only removes an empty directory; unknown contents stay.
            let _ = fs::remove_dir(path);
            continue;
        };
        if lock.try_lock_exclusive().is_ok() {
            let _ = fs::remove_dir_all(path);
        }
    }
}

/// Deletion can race an orphan helper from a previous daemon. Every attempt
/// retains its lease until that helper exits; empty parents are removed only
/// after all safe attempts have gone. External links are never traversed.
fn cleanup_project(directory: &Path) {
    if !fs::symlink_metadata(directory).is_ok_and(|meta| meta.is_dir()) {
        return;
    }
    if let Ok(entries) = fs::read_dir(directory) {
        for entry in entries.flatten() {
            if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                cleanup_abandoned(&entry.path());
                let _ = fs::remove_dir(entry.path());
            }
        }
    }
    let _ = fs::remove_dir(directory);
}

fn managed_projects(root: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let id = entry.file_name().into_string().ok()?;
            if id.parse::<ProjectId>().is_ok() && entry.file_type().is_ok_and(|kind| kind.is_dir())
            {
                Some((id, entry.path()))
            } else {
                None
            }
        })
        .collect()
}

/// A durable source row must not outlive an unflushed directory entry after a
/// power loss. The helper's file sync alone does not persist its new parents.
fn sync_download(path: &Path, root: &Path) -> std::io::Result<()> {
    fs::File::open(path)?.sync_all()?;
    let mut directory = path.parent();
    while let Some(parent) = directory {
        fs::File::open(parent)?.sync_all()?;
        if Some(parent) == root.parent() {
            return Ok(());
        }
        directory = parent.parent();
    }
    Err(std::io::Error::other(
        "import directory escaped managed storage",
    ))
}

#[cfg(test)]
mod tests;
