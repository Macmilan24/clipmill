//! ClipMill desktop shell: window management and daemon communication.
//! `clipmilld` owns durable project state and media work. The shell supervises the
//! daemon and can restart without losing durable state.

mod daemon;
mod media;
mod models;
mod views;
mod youtube;

use std::{sync::Arc, time::Duration};

use serde::Serialize;
use tauri::{Emitter, State};
use tauri_plugin_dialog::DialogExt;

pub use daemon::{ConnectionState, DaemonClient, DaemonLinkError, DaemonSupervisor};
pub use media::{MediaProtocol, SCHEME as MEDIA_SCHEME};
pub use views::{DocumentView, JobView, ProjectView, SourceView, TaskView};

/// Event name the renderer subscribes to for connection transitions.
const STATE_EVENT: &str = "daemon://state";
/// Event name carrying one task transition from the daemon's durable log.
const TASK_EVENT: &str = "daemon://task-events";
const POLL_INTERVAL: Duration = Duration::from_secs(2);
/// How long to wait before resubscribing after the event stream drops. Short,
/// because the gap is exactly the window in which a running stage looks stalled.
const RESUBSCRIBE_DELAY: Duration = Duration::from_millis(500);

/// Task transition passed through to the renderer. Preserve progress units and
/// counts because stages measure different quantities that cannot be averaged.
#[derive(Debug, Serialize)]
struct TaskEventView {
    #[serde(rename = "eventId")]
    event_id: u64,
    #[serde(rename = "jobId")]
    job_id: String,
    #[serde(rename = "taskId")]
    task_id: String,
    state: i32,
    attempt: u32,
    #[serde(rename = "waitReason")]
    wait_reason: String,
    #[serde(rename = "failureClass")]
    failure_class: i32,
    #[serde(rename = "atUnixMillis")]
    at_unix_millis: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    progress: Option<ProgressView>,
}

#[derive(Debug, Serialize)]
struct ProgressView {
    unit: String,
    done: u64,
    /// Zero means the stage knows how far it has come and not how far there is
    /// to go. A bar drawn from that would be inventing the denominator.
    total: u64,
}

/// The device profile as the daemon returned it. The document is passed through
/// as canonical JSON rather than reshaped here: the renderer parses it with the
/// generated `clipmill.device_profile.v1` type, so the schema stays the only
/// contract between them.
#[derive(Debug, Serialize)]
struct DeviceProfileView {
    #[serde(rename = "artifactId")]
    artifact_id: String,
    #[serde(rename = "profileJson")]
    profile_json: String,
}

#[tauri::command]
async fn daemon_state(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
) -> Result<ConnectionState, String> {
    Ok(supervisor.state().await)
}

/// Force a supervision pass now instead of waiting for the next tick. The
/// renderer calls this when the user asks to retry.
#[tauri::command]
async fn reconnect_daemon(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
) -> Result<ConnectionState, String> {
    supervisor.reconcile().await;
    Ok(supervisor.state().await)
}

#[tauri::command]
async fn device_profile(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    remeasure: bool,
) -> Result<DeviceProfileView, String> {
    let profile = supervisor
        .client()
        .device_profile(remeasure)
        .await
        .map_err(|error| error.to_string())?;
    Ok(DeviceProfileView {
        artifact_id: profile.artifact_id,
        profile_json: profile.profile_json,
    })
}

/// Forward renderer commands to the daemon and convert replies to view records.
/// The daemon owns policy and durable state; errors become displayable strings.
#[tauri::command]
async fn list_projects(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
) -> Result<Vec<views::ProjectView>, String> {
    supervisor
        .client()
        .list_projects()
        .await
        .map(|projects| projects.into_iter().map(Into::into).collect())
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn create_project(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    name: String,
) -> Result<String, String> {
    supervisor
        .client()
        .create_project(&name)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn rename_project(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    project_id: String,
    name: String,
) -> Result<views::ProjectView, String> {
    supervisor
        .client()
        .rename_project(&project_id, &name)
        .await
        .map(Into::into)
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn delete_project(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    project_id: String,
) -> Result<(), String> {
    supervisor
        .client()
        .delete_project(&project_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn cancel_job(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    job_id: String,
) -> Result<views::JobView, String> {
    supervisor
        .client()
        .cancel_job(&job_id)
        .await
        .map(Into::into)
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn list_sources(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    project_id: String,
) -> Result<Vec<views::SourceView>, String> {
    supervisor
        .client()
        .list_sources(&project_id)
        .await
        .map(|sources| sources.into_iter().map(Into::into).collect())
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn list_jobs(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    project_id: String,
) -> Result<Vec<views::JobView>, String> {
    supervisor
        .client()
        .list_jobs(&project_id)
        .await
        .map(|jobs| jobs.into_iter().map(Into::into).collect())
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn get_job(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    job_id: String,
) -> Result<views::JobView, String> {
    supervisor
        .client()
        .get_job(&job_id)
        .await
        .map(Into::into)
        .map_err(|error| error.to_string())
}

/// Containers the analysis pipeline can open, offered as the dialog's filter.
const SOURCE_EXTENSIONS: [&str; 6] = ["mp4", "mov", "mkv", "webm", "m4v", "avi"];

/// Open a native source picker. The renderer has no direct dialog permission;
/// this command returns only the path selected in the host-owned dialog.
#[tauri::command]
async fn choose_source_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (reply, chosen) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_title("Choose a video to analyze")
        .add_filter("Video", &SOURCE_EXTENSIONS)
        .pick_file(move |path| {
            let _sent = reply.send(path);
        });
    let path = chosen
        .await
        .map_err(|_| "the file dialog closed".to_owned())?;
    // A path the shell cannot express as text is one the daemon could not be
    // told about either, so it is refused here rather than half-way through a
    // registration.
    Ok(path.map(|value| value.to_string()))
}

/// Pictures and sounds an asset may be, offered as the dialog's filters.
const IMAGE_EXTENSIONS: [&str; 4] = ["png", "jpg", "jpeg", "webp"];
const AUDIO_EXTENSIONS: [&str; 7] = ["mp3", "wav", "flac", "m4a", "aac", "ogg", "oga"];

/// Bring a picture or a sound in: the host opens the picker, and the daemon
/// copies the chosen file into the asset folder by its hash. The renderer
/// never names a path. `None` when the person closed the picker.
#[tauri::command]
async fn import_asset(
    app: tauri::AppHandle,
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    kind: String,
    license: String,
) -> Result<Option<views::AssetView>, String> {
    let (title, filter, extensions): (&str, &str, &[&str]) = match kind.as_str() {
        "image" => ("Choose a picture", "Picture", &IMAGE_EXTENSIONS),
        "audio" => ("Choose a sound", "Sound", &AUDIO_EXTENSIONS),
        _ => return Err("choose a picture or a sound".to_owned()),
    };
    let (reply, chosen) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_title(title)
        .add_filter(filter, extensions)
        .pick_file(move |path| {
            let _sent = reply.send(path);
        });
    let Some(path) = chosen
        .await
        .map_err(|_| "the file dialog closed".to_owned())?
    else {
        return Ok(None);
    };
    supervisor
        .client()
        .import_asset(path.to_string(), license)
        .await
        .map(|asset| Some(asset.into()))
        .map_err(|error| error.to_string())
}

/// Every asset of a kind, newest first.
#[tauri::command]
async fn list_assets(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    kind: String,
) -> Result<Vec<views::AssetView>, String> {
    supervisor
        .client()
        .list_assets(kind)
        .await
        .map(|assets| assets.into_iter().map(Into::into).collect())
        .map_err(|error| error.to_string())
}

/// Open a native export-folder picker. The renderer has no direct dialog
/// permission; this command returns the directory selected by the user.
#[tauri::command]
async fn choose_export_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (reply, chosen) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_title("Choose where exports are written")
        .pick_folder(move |path| {
            let _sent = reply.send(path);
        });
    let path = chosen
        .await
        .map_err(|_| "the folder dialog closed".to_owned())?;
    Ok(path.map(|value| value.to_string()))
}

/// Register a local file as this project's source, which probes it.
#[tauri::command]
async fn register_source(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    project_id: String,
    absolute_path: String,
) -> Result<views::RegisteredSourceView, String> {
    supervisor
        .client()
        .register_source(&project_id, &absolute_path)
        .await
        .map_err(|error| error.to_string())
        .and_then(|registered| {
            views::RegisteredSourceView::try_from(registered).map_err(ToOwned::to_owned)
        })
}

/// Locate the same recording after it has moved, preserving its source id.
#[tauri::command]
async fn relink_source(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    project_id: String,
    source_id: String,
    absolute_path: String,
) -> Result<views::RegisteredSourceView, String> {
    supervisor
        .client()
        .register_source_with_id(&project_id, &absolute_path, &source_id)
        .await
        .map_err(|error| error.to_string())
        .and_then(|registered| {
            views::RegisteredSourceView::try_from(registered).map_err(ToOwned::to_owned)
        })
}

#[tauri::command]
async fn get_source(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    source_id: String,
) -> Result<views::SourceDetailsView, String> {
    supervisor
        .client()
        .get_source(&source_id)
        .await
        .map_err(|error| error.to_string())?
        .try_into()
        .map_err(str::to_owned)
}

#[tauri::command]
async fn start_youtube_import(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    project_id: String,
    url: String,
    rights_confirmed: bool,
    max_height: Option<u32>,
) -> Result<views::YoutubeImportView, String> {
    supervisor
        .client()
        .start_youtube_import(
            &project_id,
            &url,
            rights_confirmed,
            max_height.unwrap_or(1080),
        )
        .await
        .map(Into::into)
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn get_youtube_import(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    import_id: String,
) -> Result<views::YoutubeImportView, String> {
    supervisor
        .client()
        .get_youtube_import(&import_id)
        .await
        .map(Into::into)
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn list_youtube_imports(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    project_id: String,
) -> Result<Vec<views::YoutubeImportView>, String> {
    supervisor
        .client()
        .list_youtube_imports(&project_id)
        .await
        .map(|items| items.into_iter().map(Into::into).collect())
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn update_youtube_import(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    import_id: String,
    action: String,
) -> Result<views::YoutubeImportView, String> {
    supervisor
        .client()
        .update_youtube_import(&import_id, &action)
        .await
        .map(Into::into)
        .map_err(|error| error.to_string())
}

/// Submit the analysis DAG. The reply is the job, so a screen can watch it.
#[tauri::command]
async fn submit_analyze(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    project_id: String,
    request: views::AnalyzeRequest,
) -> Result<views::JobView, String> {
    supervisor
        .client()
        .submit_analyze(&project_id, request.into_payload())
        .await
        .map(Into::into)
        .map_err(|error| error.to_string())
}

/// What a media artifact holds, so a screen can name a file when it asks the
/// media protocol for one. The bytes arrive over that protocol, never here.
#[tauri::command]
async fn resolve_media(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    project_id: String,
    artifact_id: String,
) -> Result<views::MediaArtifactView, String> {
    supervisor
        .client()
        .resolve_media(&project_id, &artifact_id)
        .await
        .map(Into::into)
        .map_err(|error| error.to_string())
}

/// How much disk this installation is using, by category.
#[tauri::command]
async fn storage_stats(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
) -> Result<views::StorageStatsView, String> {
    supervisor
        .client()
        .storage_stats()
        .await
        .map(Into::into)
        .map_err(|error| error.to_string())
}

/// One published document, whole. The daemon decides whether this project may
/// read it and which file the artifact's kind carries; this reassembles however
/// many chunks it took.
#[tauri::command]
async fn read_document(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    project_id: String,
    artifact_id: String,
) -> Result<views::DocumentView, String> {
    let (kind, json) = supervisor
        .client()
        .read_document(&project_id, &artifact_id)
        .await
        .map_err(|error| error.to_string())?;
    Ok(views::DocumentView {
        artifact_id,
        kind,
        json,
    })
}

/// Apply one command to a document, and hand back the inverse.
#[tauri::command]
async fn apply_edit_command(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    doc_id: String,
    expected_revision: u64,
    command_json: String,
) -> Result<views::AppliedCommandView, String> {
    supervisor
        .client()
        .apply_edit_command(&doc_id, expected_revision, command_json)
        .await
        .map(Into::into)
        .map_err(|error| error.to_string())
}

/// Every edit document a project holds, oldest first.
#[tauri::command]
async fn list_edit_docs(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    project_id: String,
) -> Result<Vec<views::EditDocView>, String> {
    supervisor
        .client()
        .list_edit_docs(&project_id)
        .await
        .map(|docs| docs.into_iter().map(Into::into).collect())
        .map_err(|error| error.to_string())
}

/// Captions under a look being tried, without saving it.
#[tauri::command]
async fn preview_captions(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    doc_id: String,
    style_ref: String,
    options_json: String,
) -> Result<views::CaptionPreviewView, String> {
    supervisor
        .client()
        .preview_captions(&doc_id, &style_ref, &options_json)
        .await
        .map(|reply| views::CaptionPreviewView {
            ass: reply.ass,
            revision: reply.revision,
        })
        .map_err(|error| error.to_string())
}

/// Ask for attention when a long run finishes while the window is behind
/// others: the Dock icon bounces once, and nothing else changes.
#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri injects the calling window by value"
)]
fn request_attention(window: tauri::WebviewWindow) {
    // Nothing to do when the platform will not bounce anything.
    let _ = window.request_user_attention(Some(tauri::UserAttentionType::Informational));
}

/// Where the camera would point at each moment, for vertical thumbnails.
#[tauri::command]
async fn thumbnail_framing(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    project_id: String,
    face_track_artifact_id: String,
    moments: Vec<u64>,
) -> Result<Vec<f64>, String> {
    let request = clipmill_contracts::proto::ipc::v1::ThumbnailFramingRequest {
        project_id,
        face_track_artifact_id,
        moments,
    };
    supervisor
        .client()
        .thumbnail_framing(request)
        .await
        .map_err(|error| error.to_string())
}

/// The faces seen over a span, for picking who the camera follows.
#[tauri::command]
async fn list_faces(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    project_id: String,
    face_track_artifact_id: String,
    start_ticks: u64,
    end_ticks: u64,
) -> Result<Vec<views::FaceSightingView>, String> {
    let request = clipmill_contracts::proto::ipc::v1::ListFacesRequest {
        project_id,
        face_track_artifact_id,
        start_ticks,
        end_ticks,
    };
    supervisor
        .client()
        .list_faces(request)
        .await
        .map(|sightings| sightings.into_iter().map(Into::into).collect())
        .map_err(|error| error.to_string())
}

/// A document's whole edit history, oldest first.
#[tauri::command]
async fn list_edit_history(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    doc_id: String,
) -> Result<Vec<views::EditHistoryEntryView>, String> {
    supervisor
        .client()
        .list_edit_history(&doc_id)
        .await
        .map(|entries| {
            entries
                .into_iter()
                .map(|entry| views::EditHistoryEntryView {
                    revision: entry.revision,
                    command_json: entry.command_json,
                    inverse_json: entry.inverse_json,
                    applied_unix_millis: entry.applied_unix_millis,
                })
                .collect()
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn get_edit_doc(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    doc_id: String,
) -> Result<views::EditDocDetailView, String> {
    supervisor
        .client()
        .get_edit_doc(&doc_id)
        .await
        .map(Into::into)
        .map_err(|error| error.to_string())
}

/// What the player must draw for a document.
#[tauri::command]
async fn preview_plan(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    project_id: String,
    doc_id: String,
) -> Result<views::PreviewPlanView, String> {
    supervisor
        .client()
        .preview_plan(&project_id, &doc_id)
        .await
        .map(Into::into)
        .map_err(|error| error.to_string())
}

/// What an export would do, without doing it.
#[tauri::command]
async fn plan_export(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    request: views::ExportRequestInput,
) -> Result<views::ExportPlanView, String> {
    supervisor
        .client()
        .plan_export(request.into())
        .await
        .map(Into::into)
        .map_err(|error| error.to_string())
}

/// Perform an export. Answers with the job to watch.
#[tauri::command]
async fn export_clip(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    request: views::ExportRequestInput,
) -> Result<views::QueuedExportView, String> {
    supervisor
        .client()
        .export_clip(request.into())
        .await
        .map(Into::into)
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn submit_export_batch(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    requests: Vec<views::ExportRequestInput>,
) -> Result<views::ExportBatchView, String> {
    supervisor
        .client()
        .submit_export_batch(requests.into_iter().map(Into::into).collect())
        .await
        .map_err(|error| error.to_string())?
        .try_into()
}
#[tauri::command]
async fn list_export_batches(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
) -> Result<Vec<views::ExportBatchView>, String> {
    supervisor
        .client()
        .list_export_batches()
        .await
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(TryInto::try_into)
        .collect()
}
#[tauri::command]
async fn update_export_batch_item(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    batch_id: String,
    index: u32,
    action: String,
) -> Result<views::ExportBatchView, String> {
    supervisor
        .client()
        .update_export_batch_item(batch_id, index, action)
        .await
        .map_err(|error| error.to_string())?
        .try_into()
}

/// Reveal an existing regular file in the platform file manager. Canonicalize and
/// validate the path first, then pass it as a single argument without a shell
/// (`open -R` on macOS, a folder opener elsewhere).
#[tauri::command]
async fn reveal_path(path: String) -> Result<(), String> {
    let target = std::path::Path::new(&path)
        .canonicalize()
        .map_err(|error| format!("{path}: {error}"))?;
    let metadata = std::fs::metadata(&target).map_err(|error| format!("{path}: {error}"))?;
    if !metadata.is_file() {
        return Err(format!("{path} is not a file"));
    }
    let status = tokio::task::spawn_blocking(move || reveal_command(&target).status())
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| format!("cannot open the file manager: {error}"))?;
    if !status.success() {
        return Err(format!("the file manager refused: {status}"));
    }
    Ok(())
}

/// The platform's reveal, as a command with the path as its one argument.
fn reveal_command(target: &std::path::Path) -> std::process::Command {
    #[cfg(target_os = "macos")]
    {
        let mut command = std::process::Command::new("/usr/bin/open");
        command.arg("-R").arg(target);
        command
    }
    #[cfg(not(target_os = "macos"))]
    {
        // No reveal-in-folder on the free desktops' common opener; the
        // folder is what can be shown.
        let mut command = std::process::Command::new("xdg-open");
        command.arg(target.parent().unwrap_or(target));
        command
    }
}

/// Pack a project's work into a zip.
#[tauri::command]
async fn export_archive(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    project_id: String,
    destination_dir: String,
) -> Result<views::ArchiveView, String> {
    supervisor
        .client()
        .export_archive(&project_id, &destination_dir)
        .await
        .map(Into::into)
        .map_err(|error| error.to_string())
}

/// Whether an analysis could run right now, stage by stage, and what to do
/// about the ones that could not.
#[tauri::command]
async fn readiness(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
) -> Result<views::ReadinessView, String> {
    supervisor
        .client()
        .readiness()
        .await
        .map(Into::into)
        .map_err(|error| error.to_string())
}

/// Whether this installation is offline, and the evidence for it.
#[tauri::command]
async fn local_lock(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
) -> Result<views::LocalLockView, String> {
    supervisor
        .client()
        .local_lock()
        .await
        .map(Into::into)
        .map_err(|error| error.to_string())
}

/// The crop path for a span, as a proposal. Nothing is written, so the
/// Inspector may ask again every time a boundary moves.
///
/// `track_id` follows that face instead of the one the gate would choose,
/// `two_up` solves both portraits of a two-person layout, and the aspect is
/// the clip's frame, 9:16 when absent.
#[tauri::command]
#[allow(
    clippy::too_many_arguments,
    reason = "a Tauri command's arguments are its JSON fields"
)]
async fn solve_crop_path(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    project_id: String,
    face_track_artifact_id: String,
    start_ticks: u64,
    end_ticks: u64,
    track_id: Option<u32>,
    two_up: Option<bool>,
    aspect_width: Option<u32>,
    aspect_height: Option<u32>,
) -> Result<views::CropPathView, String> {
    let (aspect_width, aspect_height) = match (aspect_width, aspect_height) {
        (Some(width), Some(height)) if width > 0 && height > 0 => (width, height),
        _ => (9, 16),
    };
    let request = clipmill_contracts::proto::ipc::v1::SolveCropPathRequest {
        project_id,
        face_track_artifact_id,
        start_ticks,
        end_ticks,
        aspect_width,
        aspect_height,
        weights: None,
        track_id: track_id.unwrap_or(0),
        follow_track: track_id.is_some(),
        two_up: two_up.unwrap_or(false),
    };
    supervisor
        .client()
        .solve_crop_path(request)
        .await
        .map(Into::into)
        .map_err(|error| error.to_string())
}

/// Turn an approved candidate into an edit document.
///
/// One call rather than assemble-then-create: two would leave a window where a
/// clip is half approved, and nothing downstream could tell that from a crash.
#[tauri::command]
async fn direct_clip(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    request: views::DirectClipInput,
) -> Result<views::DirectedClipView, String> {
    supervisor
        .client()
        .direct_clip(request.into())
        .await
        .map(Into::into)
        .map_err(|error| error.to_string())
}

/// The clip approving would build, drawn as the Editor draws it — built and
/// not saved, so the Inspector shows what an approval makes.
#[tauri::command]
async fn preview_direct(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    request: views::DirectClipInput,
) -> Result<views::PreviewPlanView, String> {
    supervisor
        .client()
        .preview_direct(request.into())
        .await
        .map(Into::into)
        .map_err(|error| error.to_string())
}

/// Record what somebody decided about a clip, durably.
#[tauri::command]
async fn set_clip_decision(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    project_id: String,
    source_id: String,
    candidate_id: String,
    decision: String,
) -> Result<views::ClipDecisionView, String> {
    let request = clipmill_contracts::proto::ipc::v1::SetClipDecisionRequest {
        project_id,
        source_id,
        candidate_id: candidate_id.clone(),
        decision: views::decision_code(&decision),
    };
    let reply = supervisor
        .client()
        .set_clip_decision(request)
        .await
        .map_err(|error| error.to_string())?;
    Ok(views::ClipDecisionView {
        candidate_id,
        decision: views::decision_word(reply.decision),
        decided_unix_millis: reply.decided_unix_millis,
    })
}

#[tauri::command]
async fn list_clip_decisions(
    supervisor: State<'_, Arc<DaemonSupervisor>>,
    project_id: String,
    source_id: String,
) -> Result<Vec<views::ClipDecisionView>, String> {
    supervisor
        .client()
        .list_clip_decisions(&project_id, &source_id)
        .await
        .map(|records| records.into_iter().map(Into::into).collect())
        .map_err(|error| error.to_string())
}

/// Boot the shell. The socket path is resolved through the daemon's own
/// configuration so the two can never disagree about where to meet.
/// Forward the daemon's task events to the renderer, across restarts.
///
/// The cursor is what makes a reconnect honest: the daemon replays what
/// happened while the socket was down, so a stage that finished during the gap
/// arrives finished rather than staying where the renderer last saw it.
async fn stream_task_events(supervisor: Arc<DaemonSupervisor>, app: tauri::AppHandle) {
    let mut cursor = 0_u64;
    loop {
        let handle = app.clone();
        let result = supervisor
            .client()
            .stream_task_events(cursor, |event| {
                cursor = cursor.max(event.event_id);
                let view = TaskEventView {
                    event_id: event.event_id,
                    job_id: event.job_id,
                    task_id: event.task_id,
                    state: event.state,
                    attempt: event.attempt,
                    wait_reason: event.wait_reason,
                    failure_class: event.failure_class,
                    at_unix_millis: event.at_unix_millis,
                    progress: event.progress.map(|progress| ProgressView {
                        unit: progress.unit,
                        done: progress.done,
                        total: progress.total,
                    }),
                };
                if let Err(error) = handle.emit(TASK_EVENT, &view) {
                    tracing::warn!(%error, "cannot publish a task event");
                }
            })
            .await;
        if let Err(error) = result {
            tracing::debug!(%error, cursor, "task event stream ended");
        }
        tokio::time::sleep(RESUBSCRIBE_DELAY).await;
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "Tauri command registration is kept together in the app builder"
)]
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = clipmilld::Config::resolve(None, None)?;
    let socket = config.paths.socket.clone();
    tracing::info!(socket = %socket.display(), "resolved daemon socket");

    let supervisor = Arc::new(DaemonSupervisor::new(DaemonClient::new(socket)));
    let background = Arc::clone(&supervisor);
    // The media door. It holds the store root so it can derive an object
    // directory from a content address; it receives no path from the daemon.
    let media = Arc::new(
        media::MediaProtocol::new(Arc::clone(&supervisor), config.paths.artifacts_dir.clone())
            .with_fonts(config.fonts_dir.clone())
            .with_assets(config.paths.assets_dir.clone()),
    );

    tauri::Builder::default()
        // Registered for `choose_source_file` alone. The renderer is granted no
        // permission to reach the plugin, so a page cannot open a dialog — it
        // can only ask this host to open one.
        .plugin(tauri_plugin_dialog::init())
        .manage(supervisor)
        .register_asynchronous_uri_scheme_protocol(
            media::SCHEME,
            move |_app, request, responder| {
                let media = Arc::clone(&media);
                tauri::async_runtime::spawn(async move {
                    responder.respond(media.serve(request).await);
                });
            },
        )
        .invoke_handler(tauri::generate_handler![
            models::list_models,
            models::download_models,
            models::cancel_model_download,
            models::remove_model,
            models::verify_model,
            models::set_model_choice,
            models::inspect_hub_model,
            models::add_custom_model,
            models::forget_model,
            models::clean_storage,
            models::open_storage_location,
            youtube::youtube_publishing_status,
            youtube::choose_youtube_client_config,
            youtube::connect_youtube_channel,
            youtube::update_youtube_connection,
            youtube::draft_youtube_metadata,
            youtube::start_youtube_upload,
            youtube::get_youtube_upload,
            youtube::list_youtube_uploads,
            youtube::update_youtube_upload,
            youtube::publish_youtube_upload,
            youtube::open_youtube_page,
            daemon_state,
            reconnect_daemon,
            device_profile,
            list_projects,
            create_project,
            list_sources,
            list_jobs,
            get_job,
            read_document,
            resolve_media,
            import_asset,
            list_assets,
            storage_stats,
            choose_source_file,
            register_source,
            relink_source,
            preview_captions,
            list_edit_history,
            list_faces,
            preview_direct,
            request_attention,
            thumbnail_framing,
            get_source,
            start_youtube_import,
            get_youtube_import,
            list_youtube_imports,
            update_youtube_import,
            submit_analyze,
            apply_edit_command,
            list_edit_docs,
            get_edit_doc,
            preview_plan,
            plan_export,
            export_clip,
            submit_export_batch,
            list_export_batches,
            update_export_batch_item,
            export_archive,
            local_lock,
            readiness,
            choose_export_folder,
            reveal_path,
            solve_crop_path,
            direct_clip,
            set_clip_decision,
            list_clip_decisions,
            rename_project,
            delete_project,
            cancel_job
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            let events = Arc::clone(&background);
            let events_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    // reconcile() reports only edges, so a steady connection
                    // stays silent instead of spamming the renderer.
                    if let Some(state) = background.reconcile().await
                        && let Err(error) = handle.emit(STATE_EVENT, &state)
                    {
                        tracing::warn!(%error, "cannot publish daemon state");
                    }
                    tokio::time::sleep(POLL_INTERVAL).await;
                }
            });
            tauri::async_runtime::spawn(stream_task_events(events, events_handle));
            Ok(())
        })
        .run(tauri::generate_context!())?;
    Ok(())
}
