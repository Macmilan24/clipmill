//! The model library and storage clean-up, as the renderer sees them.
//!
//! Thin like the rest of the host: every decision is the daemon's, and each
//! command is one request and one reshaped answer. The only thing done here is
//! opening a storage folder, and even that is chosen by the category key the
//! daemon reported rather than by a path the page supplies.

use std::sync::Arc;

use clipmill_contracts::proto::ipc::v1::{self as ipc, request, response};
use serde::Serialize;
use tauri::State;

use crate::{DaemonClient, DaemonSupervisor, views::StorageStatsView};

type Host<'a> = State<'a, Arc<DaemonSupervisor>>;

/// Every model the daemon knows, where each stands, and who does each job.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryView {
    models: Vec<ModelView>,
    jobs: Vec<ModelJobView>,
    install_path: String,
    /// Absent when the filesystem would not say, which is not the same as full.
    #[serde(skip_serializing_if = "Option::is_none")]
    available_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    memory_total_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    memory_budget_bytes: Option<u64>,
    recommended_missing_bytes: u64,
    recommended_missing: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(
    clippy::struct_excessive_bools,
    reason = "a flat view the renderer reads field by field, as the wire carries it"
)]
struct ModelView {
    name: String,
    title: String,
    summary: String,
    capability: String,
    runtime: String,
    backend: String,
    quantization: String,
    license_spdx: String,
    source_repo: String,
    source_revision: String,
    download_bytes: u64,
    memory_bytes: u64,
    recommended: bool,
    supported: bool,
    unsupported_reason: String,
    memory_fit: String,
    custom: bool,
    install_state: String,
    installed_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    download: Option<DownloadView>,
    in_use: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadView {
    state: String,
    received_bytes: u64,
    total_bytes: u64,
    current_file: String,
    error: String,
    updated_unix_millis: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelJobView {
    capability: String,
    title: String,
    summary: String,
    model: String,
    selected_by: String,
    choice: String,
    models: Vec<String>,
}

impl From<ipc::ListModelsResponse> for LibraryView {
    fn from(value: ipc::ListModelsResponse) -> Self {
        let known = |bytes: u64| (bytes > 0).then_some(bytes);
        Self {
            models: value.models.into_iter().map(ModelView::from).collect(),
            jobs: value
                .jobs
                .into_iter()
                .map(|job| ModelJobView {
                    capability: job.capability,
                    title: job.title,
                    summary: job.summary,
                    model: job.model,
                    selected_by: job.selected_by,
                    choice: job.choice,
                    models: job.models,
                })
                .collect(),
            install_path: value.install_path,
            available_bytes: value.available_known.then_some(value.available_bytes),
            memory_total_bytes: known(value.memory_total_bytes),
            memory_budget_bytes: known(value.memory_budget_bytes),
            recommended_missing_bytes: value.recommended_missing_bytes,
            recommended_missing: value.recommended_missing,
        }
    }
}

impl From<ipc::ModelV1> for ModelView {
    fn from(model: ipc::ModelV1) -> Self {
        Self {
            name: model.name,
            title: model.title,
            summary: model.summary,
            capability: model.capability,
            runtime: model.runtime,
            backend: model.backend,
            quantization: model.quantization,
            license_spdx: model.license_spdx,
            source_repo: model.source_repo,
            source_revision: model.source_revision,
            download_bytes: model.download_bytes,
            memory_bytes: model.memory_bytes,
            recommended: model.recommended,
            supported: model.supported,
            unsupported_reason: model.unsupported_reason,
            memory_fit: model.memory_fit,
            custom: model.custom,
            install_state: model.install_state,
            installed_bytes: model.installed_bytes,
            download: model.download.map(|download| DownloadView {
                state: download.state,
                received_bytes: download.received_bytes,
                total_bytes: download.total_bytes,
                current_file: download.current_file,
                error: download.error,
                updated_unix_millis: download.updated_unix_millis,
            }),
            in_use: model.in_use,
        }
    }
}

/// What pinning a repository would pin, or why it cannot be.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectionView {
    repo: String,
    commit: String,
    capability: String,
    license: String,
    license_spdx: String,
    license_allowed: bool,
    problem: String,
    weight_choices: Vec<HubFileView>,
    files: Vec<HubFileView>,
    suggested_name: String,
    suggested_title: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HubFileView {
    path: String,
    bytes: u64,
    suggested_name: String,
    suggested_title: String,
}

impl From<ipc::HubModelFileV1> for HubFileView {
    fn from(file: ipc::HubModelFileV1) -> Self {
        Self {
            path: file.path,
            bytes: file.bytes,
            suggested_name: file.suggested_name,
            suggested_title: file.suggested_title,
        }
    }
}

impl From<ipc::InspectHubModelResponse> for InspectionView {
    fn from(value: ipc::InspectHubModelResponse) -> Self {
        Self {
            repo: value.repo,
            commit: value.commit,
            capability: value.capability,
            license: value.license,
            license_spdx: value.license_spdx,
            license_allowed: value.license_allowed,
            problem: value.problem,
            weight_choices: value
                .weight_choices
                .into_iter()
                .map(HubFileView::from)
                .collect(),
            files: value.files.into_iter().map(HubFileView::from).collect(),
            suggested_name: value.suggested_name,
            suggested_title: value.suggested_title,
        }
    }
}

/// What a clean-up freed, and the storage report measured after it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanView {
    freed_bytes: u64,
    removed_items: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    storage: Option<StorageStatsView>,
}

async fn library_call(client: &DaemonClient, body: request::Body) -> Result<LibraryView, String> {
    match client.call(body).await.map_err(|error| error.to_string())? {
        response::Body::ModelLibrary(library) => Ok(library.into()),
        _ => Err("The engine returned no model library.".to_owned()),
    }
}

#[tauri::command]
pub async fn list_models(supervisor: Host<'_>) -> Result<LibraryView, String> {
    library_call(
        supervisor.client(),
        request::Body::ListModels(ipc::ListModelsRequest {}),
    )
    .await
}

/// Queue downloads. A network operation; the page says so before calling.
#[tauri::command]
pub async fn download_models(
    supervisor: Host<'_>,
    names: Vec<String>,
) -> Result<LibraryView, String> {
    library_call(
        supervisor.client(),
        request::Body::DownloadModels(ipc::DownloadModelsRequest { names }),
    )
    .await
}

#[tauri::command]
pub async fn cancel_model_download(
    supervisor: Host<'_>,
    name: String,
) -> Result<LibraryView, String> {
    library_call(
        supervisor.client(),
        request::Body::CancelModelDownload(ipc::CancelModelDownloadRequest { name }),
    )
    .await
}

#[tauri::command]
pub async fn remove_model(supervisor: Host<'_>, name: String) -> Result<LibraryView, String> {
    library_call(
        supervisor.client(),
        request::Body::RemoveModel(ipc::RemoveModelRequest { name }),
    )
    .await
}

#[tauri::command]
pub async fn verify_model(supervisor: Host<'_>, name: String) -> Result<LibraryView, String> {
    library_call(
        supervisor.client(),
        request::Body::VerifyModel(ipc::VerifyModelRequest { name }),
    )
    .await
}

#[tauri::command]
pub async fn set_model_choice(
    supervisor: Host<'_>,
    capability: String,
    model: String,
) -> Result<LibraryView, String> {
    library_call(
        supervisor.client(),
        request::Body::SetModelChoice(ipc::SetModelChoiceRequest { capability, model }),
    )
    .await
}

/// Read a Hugging Face repository. A network operation, started by a person.
#[tauri::command]
pub async fn inspect_hub_model(
    supervisor: Host<'_>,
    repo: String,
    revision: String,
    capability: String,
) -> Result<InspectionView, String> {
    let reply = supervisor
        .client()
        .call(request::Body::InspectHubModel(
            ipc::InspectHubModelRequest {
                repo,
                revision,
                capability,
            },
        ))
        .await
        .map_err(|error| error.to_string())?;
    match reply {
        response::Body::InspectHubModel(inspection) => Ok(inspection.into()),
        _ => Err("The engine returned no inspection.".to_owned()),
    }
}

#[tauri::command]
#[allow(
    clippy::too_many_arguments,
    reason = "one field per value the renderer names; Tauri maps them by name"
)]
pub async fn add_custom_model(
    supervisor: Host<'_>,
    repo: String,
    commit: String,
    capability: String,
    weights_file: String,
    name: String,
    title: String,
) -> Result<LibraryView, String> {
    library_call(
        supervisor.client(),
        request::Body::AddCustomModel(ipc::AddCustomModelRequest {
            repo,
            commit,
            capability,
            weights_file,
            name,
            title,
        }),
    )
    .await
}

#[tauri::command]
pub async fn forget_model(supervisor: Host<'_>, name: String) -> Result<LibraryView, String> {
    library_call(
        supervisor.client(),
        request::Body::ForgetModel(ipc::ForgetModelRequest { name }),
    )
    .await
}

#[tauri::command]
pub async fn clean_storage(supervisor: Host<'_>, action: String) -> Result<CleanView, String> {
    if !matches!(action.as_str(), "unused_files" | "temporary" | "backups") {
        return Err("Unknown clean-up.".to_owned());
    }
    let reply = supervisor
        .client()
        .call(request::Body::CleanStorage(ipc::CleanStorageRequest {
            action,
        }))
        .await
        .map_err(|error| error.to_string())?;
    match reply {
        response::Body::CleanStorage(cleaned) => Ok(CleanView {
            freed_bytes: cleaned.freed_bytes,
            removed_items: cleaned.removed_items,
            storage: cleaned.storage.map(Into::into),
        }),
        _ => Err("The engine returned no clean-up report.".to_owned()),
    }
}

/// Open a storage category's folder in the file manager.
///
/// The page names a category, never a path: the folder is whichever one the
/// daemon reports for that key right now, and it must be a directory that
/// exists. A page cannot use this to open anything else.
#[tauri::command]
pub async fn open_storage_location(supervisor: Host<'_>, key: String) -> Result<(), String> {
    let report = supervisor
        .client()
        .storage_stats()
        .await
        .map_err(|error| error.to_string())?;
    let path = report
        .categories
        .into_iter()
        .find(|category| category.key == key)
        .map(|category| category.path)
        .ok_or_else(|| "The engine reports no such storage location.".to_owned())?;
    let target = std::path::Path::new(&path)
        .canonicalize()
        .map_err(|_| "Nothing is stored there yet.".to_owned())?;
    if !target.is_dir() {
        return Err("That storage location is not a folder.".to_owned());
    }
    let status = tokio::task::spawn_blocking(move || open_command(&target).status())
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| format!("cannot open the file manager: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("the file manager refused: {status}"))
    }
}

/// The platform's folder opener, with the folder as its one argument.
fn open_command(folder: &std::path::Path) -> std::process::Command {
    #[cfg(target_os = "macos")]
    {
        let mut command = std::process::Command::new("/usr/bin/open");
        command.arg(folder);
        command
    }
    #[cfg(not(target_os = "macos"))]
    {
        let mut command = std::process::Command::new("xdg-open");
        command.arg(folder);
        command
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use clipmill_contracts::proto::ipc::v1 as ipc;

    use super::LibraryView;

    /// The renderer keys off these names; an unknown figure stays absent
    /// rather than reading as zero.
    #[test]
    fn a_library_reaches_the_renderer_in_its_own_words() {
        let view = LibraryView::from(ipc::ListModelsResponse {
            models: vec![ipc::ModelV1 {
                name: "whisper-base".to_owned(),
                install_state: "partial".to_owned(),
                download: Some(ipc::ModelDownloadV1 {
                    state: "downloading".to_owned(),
                    received_bytes: 10,
                    total_bytes: 20,
                    ..ipc::ModelDownloadV1::default()
                }),
                ..ipc::ModelV1::default()
            }],
            available_known: false,
            memory_total_bytes: 0,
            memory_budget_bytes: 12,
            ..ipc::ListModelsResponse::default()
        });
        let json = serde_json::to_value(&view).expect("serializes");
        assert_eq!(json["models"][0]["installState"], "partial");
        assert_eq!(json["models"][0]["download"]["receivedBytes"], 10);
        assert!(json.get("availableBytes").is_none());
        assert!(json.get("memoryTotalBytes").is_none());
        assert_eq!(json["memoryBudgetBytes"], 12);
    }
}
