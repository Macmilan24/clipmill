//! ClipMill's components, as the renderer sees them: the model workers a
//! packaged app installs and keeps running. Every decision is the daemon's;
//! each command is one request and one reshaped answer.

use std::sync::Arc;

use clipmill_contracts::proto::ipc::v1::{self as ipc, request, response};
use serde::Serialize;
use tauri::State;

use crate::{DaemonClient, DaemonSupervisor};

type Host<'a> = State<'a, Arc<DaemonSupervisor>>;

/// The components and where each stands.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentsView {
    /// False in a development checkout, where workers are started by hand.
    managed: bool,
    parts: Vec<ComponentView>,
    python_version: String,
    /// Why they cannot be installed here at all, when they cannot.
    unavailable: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ComponentView {
    name: String,
    title: String,
    family: String,
    /// missing, queued, installing, installed, outdated or failed.
    state: String,
    detail: String,
    installed_bytes: u64,
    download_bytes: u64,
    /// stopped, starting, running or waiting.
    process: String,
    restarts: u32,
    log_path: String,
    /// Run by the daemon when a task needs it, never kept running.
    tool: bool,
}

impl From<ipc::EngineResponse> for ComponentsView {
    fn from(value: ipc::EngineResponse) -> Self {
        Self {
            managed: value.managed,
            parts: value
                .parts
                .into_iter()
                .map(|part| ComponentView {
                    name: part.name,
                    title: part.title,
                    family: part.family,
                    state: part.state,
                    detail: part.detail,
                    installed_bytes: part.installed_bytes,
                    download_bytes: part.download_bytes,
                    process: part.process,
                    restarts: part.restarts,
                    log_path: part.log_path,
                    tool: part.tool,
                })
                .collect(),
            python_version: value.python_version,
            unavailable: value.unavailable,
        }
    }
}

async fn engine_call(client: &DaemonClient, body: request::Body) -> Result<ComponentsView, String> {
    match client.call(body).await.map_err(|error| error.to_string())? {
        response::Body::Engine(engine) => Ok(engine.into()),
        _ => Err("The engine returned no components.".to_owned()),
    }
}

#[tauri::command]
pub async fn list_components(supervisor: Host<'_>) -> Result<ComponentsView, String> {
    engine_call(
        supervisor.client(),
        request::Body::GetEngine(ipc::GetEngineRequest {}),
    )
    .await
}

/// Install components. A network operation; the page says so before calling.
#[tauri::command]
pub async fn install_components(
    supervisor: Host<'_>,
    parts: Vec<String>,
) -> Result<ComponentsView, String> {
    engine_call(
        supervisor.client(),
        request::Body::InstallEngine(ipc::InstallEngineRequest { parts }),
    )
    .await
}

#[tauri::command]
pub async fn cancel_component_install(supervisor: Host<'_>) -> Result<ComponentsView, String> {
    engine_call(
        supervisor.client(),
        request::Body::CancelEngineInstall(ipc::CancelEngineInstallRequest {}),
    )
    .await
}
