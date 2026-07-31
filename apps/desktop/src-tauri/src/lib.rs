//! ClipMill desktop shell.
//!
//! Thin by design: this process owns the daemon connection and the window, and
//! nothing else. It holds no project state, performs no media work, and caches
//! no artifacts — `clipmilld` remains the single writer (ch. 8). If the shell
//! is killed, nothing durable is lost; if the daemon is killed, the shell says
//! so and brings it back.

mod daemon;
mod media;

use std::{sync::Arc, time::Duration};

use serde::Serialize;
use tauri::{Emitter, State};

pub use daemon::{ConnectionState, DaemonClient, DaemonLinkError, DaemonSupervisor};
pub use media::SCHEME as MEDIA_SCHEME;

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
            device_profile
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
