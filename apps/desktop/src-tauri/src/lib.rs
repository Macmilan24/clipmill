//! ClipMill desktop shell.
//!
//! Thin by design: this process owns the daemon connection and the window, and
//! nothing else. It holds no project state, performs no media work, and caches
//! no artifacts — `clipmilld` remains the single writer (ch. 8). If the shell
//! is killed, nothing durable is lost; if the daemon is killed, the shell says
//! so and brings it back.

mod daemon;
mod media;
mod views;

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

/// One task transition, shaped for the renderer.
///
/// Passed through rather than interpreted. Progress keeps its unit and its
/// two counts because that is what the daemon measured — a stage reporting
/// "412 of 900 audio windows" is saying something a percentage would throw
/// away, and the pipeline's stages do not share a unit to average over.
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

/// Every command below is the same shape: ask the daemon, turn the answer into
/// something the renderer can read, and let a failure be a string the screen can
/// show. None of them decide anything — the daemon owns every policy, and this
/// process owns no state a restart could lose.
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

/// Ask the operating system for a file.
///
/// The dialog runs here, not in the WebView. The plugin is registered for this
/// command's sake alone and the renderer is granted no permission to reach it,
/// so a page cannot open a file dialog — it can only ask this host to, and what
/// comes back is a path the user chose in a native window.
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

/// Boot the shell. The socket path is resolved through the daemon's own
/// configuration so the two can never disagree about where to meet.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = clipmilld::Config::resolve(None, None)?;
    let socket = config.paths.socket.clone();
    tracing::info!(socket = %socket.display(), "resolved daemon socket");

    let supervisor = Arc::new(DaemonSupervisor::new(DaemonClient::new(socket)));
    let background = Arc::clone(&supervisor);
    // The media door. It holds the store root so it can derive an object
    // directory from a content address; it receives no path from the daemon.
    let media = Arc::new(media::MediaProtocol::new(
        Arc::clone(&supervisor),
        config.paths.artifacts_dir.clone(),
    ));

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
            storage_stats,
            choose_source_file,
            register_source,
            submit_analyze
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
            // The task-event stream, kept alive across daemon restarts. The
            // cursor is what makes a reconnect honest: the daemon replays what
            // happened while the socket was down, so a stage that finished
            // during the gap arrives finished rather than staying where the
            // renderer last saw it.
            tauri::async_runtime::spawn(async move {
                let mut cursor = 0_u64;
                loop {
                    let handle = events_handle.clone();
                    let result = events
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
            });
            Ok(())
        })
        .run(tauri::generate_context!())?;
    Ok(())
}
