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
const POLL_INTERVAL: Duration = Duration::from_secs(2);

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
            Ok(())
        })
        .run(tauri::generate_context!())?;
    Ok(())
}
