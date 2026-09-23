//! Narrow channel-publishing commands. Secrets and authorization URLs stay out of renderer views.

use std::sync::Arc;

use clipmill_contracts::proto::ipc::v1::{self as ipc, request, response};
use serde::{Deserialize, Serialize};
use tauri::State;
use tauri_plugin_dialog::DialogExt;

use crate::{DaemonClient, DaemonSupervisor};

type Host<'a> = State<'a, Arc<DaemonSupervisor>>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishingStatus {
    available: bool,
    configured: bool,
    connections: Vec<ConnectionView>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConnectionView {
    connection_id: String,
    channel_id: String,
    title: String,
    state: String,
    error: String,
    created_unix_millis: u64,
    updated_unix_millis: u64,
}
impl From<ipc::YoutubePublishingStatusResponse> for PublishingStatus {
    fn from(value: ipc::YoutubePublishingStatusResponse) -> Self {
        Self {
            available: value.available,
            configured: value.configured,
            connections: value
                .connections
                .into_iter()
                .map(|item| ConnectionView {
                    connection_id: item.connection_id,
                    channel_id: item.channel_id,
                    title: item.title,
                    state: item.state,
                    error: item.error,
                    created_unix_millis: item.created_unix_millis,
                    updated_unix_millis: item.updated_unix_millis,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    title: String,
    description: String,
    tags: Vec<String>,
    made_for_kids: bool,
    contains_synthetic_media: bool,
}
impl From<ipc::YoutubeVideoMetadataV1> for Metadata {
    fn from(value: ipc::YoutubeVideoMetadataV1) -> Self {
        Self {
            title: value.title,
            description: value.description,
            tags: value.tags,
            made_for_kids: value.made_for_kids,
            contains_synthetic_media: value.contains_synthetic_media,
        }
    }
}
impl From<Metadata> for ipc::YoutubeVideoMetadataV1 {
    fn from(value: Metadata) -> Self {
        Self {
            title: value.title,
            description: value.description,
            tags: value.tags,
            made_for_kids: value.made_for_kids,
            contains_synthetic_media: value.contains_synthetic_media,
        }
    }
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadView {
    upload_id: String,
    project_id: String,
    doc_id: String,
    revision: u64,
    export_job_id: String,
    ir_artifact_id: String,
    render_artifact_id: String,
    connection_id: String,
    channel_id: String,
    channel_title: String,
    metadata: Metadata,
    state: String,
    acknowledged_bytes: u64,
    total_bytes: u64,
    video_id: String,
    visibility: String,
    error_code: String,
    error: String,
    created_unix_millis: u64,
    updated_unix_millis: u64,
}
impl TryFrom<ipc::YoutubeUploadV1> for UploadView {
    type Error = String;
    fn try_from(value: ipc::YoutubeUploadV1) -> Result<Self, Self::Error> {
        Ok(Self {
            upload_id: value.upload_id,
            project_id: value.project_id,
            doc_id: value.doc_id,
            revision: value.revision,
            export_job_id: value.export_job_id,
            ir_artifact_id: value.ir_artifact_id,
            render_artifact_id: value.render_artifact_id,
            connection_id: value.connection_id,
            channel_id: value.channel_id,
            channel_title: value.channel_title,
            metadata: value.metadata.ok_or("Upload metadata was missing.")?.into(),
            state: value.state,
            acknowledged_bytes: value.acknowledged_bytes,
            total_bytes: value.total_bytes,
            video_id: value.video_id,
            visibility: value.visibility,
            error_code: value.error_code,
            error: value.error,
            created_unix_millis: value.created_unix_millis,
            updated_unix_millis: value.updated_unix_millis,
        })
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadRequest {
    export_job_id: String,
    connection_id: String,
    expected_revision: u64,
    metadata: Metadata,
    rights_confirmed: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataDraft {
    metadata: Metadata,
    render_artifact_id: String,
    revision: u64,
    transcript_excerpt: String,
}

async fn status_call(
    client: &DaemonClient,
    body: request::Body,
) -> Result<PublishingStatus, String> {
    match client.call(body).await.map_err(|error| error.to_string())? {
        response::Body::YoutubePublishingStatus(value) => Ok(value.into()),
        _ => Err("The engine returned no channel status.".to_owned()),
    }
}
async fn upload_call(client: &DaemonClient, body: request::Body) -> Result<UploadView, String> {
    match client.call(body).await.map_err(|error| error.to_string())? {
        response::Body::YoutubeUpload(value) => value
            .record
            .ok_or("The engine returned no upload.".to_owned())?
            .try_into(),
        _ => Err("The engine returned no upload.".to_owned()),
    }
}

#[tauri::command]
pub async fn youtube_publishing_status(supervisor: Host<'_>) -> Result<PublishingStatus, String> {
    status_call(
        supervisor.client(),
        request::Body::GetYoutubePublishingStatus(ipc::GetYoutubePublishingStatusRequest {}),
    )
    .await
}

#[tauri::command]
pub async fn choose_youtube_client_config(
    app: tauri::AppHandle,
    supervisor: Host<'_>,
) -> Result<Option<PublishingStatus>, String> {
    let (reply, chosen) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_title("Choose your Google Desktop OAuth client JSON")
        .add_filter("JSON", &["json"])
        .pick_file(move |path| {
            let _sent = reply.send(path);
        });
    let path = chosen
        .await
        .map_err(|_| "The file picker closed.".to_owned())?;
    let Some(path) = path else {
        return Ok(None);
    };
    status_call(
        supervisor.client(),
        request::Body::ConfigureYoutubePublishing(ipc::ConfigureYoutubePublishingRequest {
            client_config_path: path.to_string(),
        }),
    )
    .await
    .map(Some)
}

#[tauri::command]
pub async fn connect_youtube_channel(supervisor: Host<'_>) -> Result<String, String> {
    let reply = supervisor
        .client()
        .call(request::Body::ConnectYoutubeChannel(
            ipc::ConnectYoutubeChannelRequest {},
        ))
        .await
        .map_err(|error| error.to_string())?;
    let response::Body::YoutubeConnect(connection) = reply else {
        return Err("The engine returned no sign-in session.".to_owned());
    };
    if let Err(error) = open_google_authorization(&connection.authorization_url).await {
        let _cancelled = status_call(
            supervisor.client(),
            request::Body::UpdateYoutubeConnection(ipc::UpdateYoutubeConnectionRequest {
                connection_id: connection.connection_id,
                action: "cancel".to_owned(),
            }),
        )
        .await;
        return Err(error);
    }
    Ok(connection.connection_id)
}

#[tauri::command]
pub async fn update_youtube_connection(
    supervisor: Host<'_>,
    connection_id: String,
    action: String,
) -> Result<PublishingStatus, String> {
    if !matches!(action.as_str(), "cancel" | "disconnect") {
        return Err("Unknown connection action.".to_owned());
    }
    status_call(
        supervisor.client(),
        request::Body::UpdateYoutubeConnection(ipc::UpdateYoutubeConnectionRequest {
            connection_id,
            action,
        }),
    )
    .await
}

#[tauri::command]
pub async fn draft_youtube_metadata(
    supervisor: Host<'_>,
    export_job_id: String,
    expected_revision: u64,
) -> Result<MetadataDraft, String> {
    let reply = supervisor
        .client()
        .call(request::Body::DraftYoutubeMetadata(
            ipc::DraftYoutubeMetadataRequest {
                export_job_id,
                expected_revision,
            },
        ))
        .await
        .map_err(|error| error.to_string())?;
    let response::Body::DraftYoutubeMetadata(draft) = reply else {
        return Err("The engine returned no metadata draft.".to_owned());
    };
    Ok(MetadataDraft {
        metadata: draft
            .metadata
            .ok_or("The metadata draft was empty.")?
            .into(),
        render_artifact_id: draft.render_artifact_id,
        revision: draft.revision,
        transcript_excerpt: draft.transcript_excerpt,
    })
}

#[tauri::command]
pub async fn start_youtube_upload(
    supervisor: Host<'_>,
    request: UploadRequest,
) -> Result<UploadView, String> {
    upload_call(
        supervisor.client(),
        request::Body::StartYoutubeUpload(ipc::StartYoutubeUploadRequest {
            export_job_id: request.export_job_id,
            connection_id: request.connection_id,
            expected_revision: request.expected_revision,
            metadata: Some(request.metadata.into()),
            rights_confirmed: request.rights_confirmed,
        }),
    )
    .await
}
#[tauri::command]
pub async fn get_youtube_upload(
    supervisor: Host<'_>,
    upload_id: String,
) -> Result<UploadView, String> {
    upload_call(
        supervisor.client(),
        request::Body::GetYoutubeUpload(ipc::GetYoutubeUploadRequest { upload_id }),
    )
    .await
}
#[tauri::command]
pub async fn list_youtube_uploads(
    supervisor: Host<'_>,
    project_id: String,
) -> Result<Vec<UploadView>, String> {
    match supervisor
        .client()
        .call(request::Body::ListYoutubeUploads(
            ipc::ListYoutubeUploadsRequest { project_id },
        ))
        .await
        .map_err(|error| error.to_string())?
    {
        response::Body::ListYoutubeUploads(reply) => {
            reply.uploads.into_iter().map(TryInto::try_into).collect()
        }
        _ => Err("The engine returned no upload history.".to_owned()),
    }
}
#[tauri::command]
pub async fn update_youtube_upload(
    supervisor: Host<'_>,
    upload_id: String,
    action: String,
) -> Result<UploadView, String> {
    if !matches!(action.as_str(), "pause" | "resume" | "reconcile") {
        return Err("Unknown upload action.".to_owned());
    }
    upload_call(
        supervisor.client(),
        request::Body::UpdateYoutubeUpload(ipc::UpdateYoutubeUploadRequest { upload_id, action }),
    )
    .await
}
#[tauri::command]
pub async fn publish_youtube_upload(
    supervisor: Host<'_>,
    upload_id: String,
) -> Result<UploadView, String> {
    upload_call(
        supervisor.client(),
        request::Body::PublishYoutubeUpload(ipc::PublishYoutubeUploadRequest { upload_id }),
    )
    .await
}

fn authorized_google_url(raw: &str) -> Result<tauri::Url, String> {
    let url = tauri::Url::parse(raw)
        .map_err(|_| "Google returned an invalid sign-in address.".to_owned())?;
    if url.scheme() != "https"
        || url.host_str() != Some("accounts.google.com")
        || url.path() != "/o/oauth2/v2/auth"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || url.fragment().is_some()
    {
        return Err("The sign-in address was not an approved Google endpoint.".to_owned());
    }
    Ok(url)
}
async fn open_google_authorization(raw: &str) -> Result<(), String> {
    open_browser(authorized_google_url(raw)?.as_str()).await
}
async fn open_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut command = tokio::process::Command::new("/usr/bin/open");
    #[cfg(target_os = "linux")]
    let mut command = tokio::process::Command::new("xdg-open");
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            command
                .arg(url)
                .kill_on_drop(true)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status(),
        )
        .await;
        match result {
            Ok(Ok(status)) if status.success() => Ok(()),
            _ => Err("The system browser could not be opened. Start sign-in again.".to_owned()),
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _unused = url;
        Err("Opening the browser is unavailable on this platform.".to_owned())
    }
}

#[tauri::command]
pub async fn open_youtube_page(
    supervisor: Host<'_>,
    page: String,
    upload_id: Option<String>,
) -> Result<(), String> {
    let url = match page.as_str() {
        "setup" => "https://developers.google.com/identity/protocols/oauth2/native-app".to_owned(),
        "console" => {
            "https://console.cloud.google.com/apis/library/youtube.googleapis.com".to_owned()
        }
        "clients" => "https://console.cloud.google.com/auth/clients".to_owned(),
        "studio" => {
            let record = upload_call(
                supervisor.client(),
                request::Body::GetYoutubeUpload(ipc::GetYoutubeUploadRequest {
                    upload_id: upload_id.ok_or("Choose a saved upload first.")?,
                }),
            )
            .await?;
            let id = record.video_id;
            if id.len() != 11
                || !id
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
            {
                return Err("This upload has no verified YouTube video ID yet.".to_owned());
            }
            format!("https://studio.youtube.com/video/{id}/edit")
        }
        _ => return Err("Unknown YouTube page.".to_owned()),
    };
    open_browser(&url).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn browser_gate_accepts_only_the_exact_google_authorization_endpoint() {
        assert!(
            authorized_google_url(
                "https://accounts.google.com/o/oauth2/v2/auth?state=opaque&code_challenge=opaque"
            )
            .is_ok()
        );
        for url in [
            "http://accounts.google.com/o/oauth2/v2/auth",
            "https://accounts.google.com.evil.invalid/o/oauth2/v2/auth",
            "https://user@accounts.google.com/o/oauth2/v2/auth",
            "https://accounts.google.com:8443/o/oauth2/v2/auth",
            "https://accounts.google.com/o/oauth2/v2/auth#fragment",
            "https://accounts.google.com/other",
            "file:///tmp/client.json",
        ] {
            assert!(authorized_google_url(url).is_err());
        }
    }
    #[test]
    fn missing_upload_metadata_never_becomes_an_empty_ready_form() {
        assert!(UploadView::try_from(ipc::YoutubeUploadV1::default()).is_err());
    }
}
