//! Channel connections and explicit publication of an immutable approved render.
mod metadata;
mod transfer;
mod verified;

use super::{Reply, Service, error_reply, response_reply, store_error_reply, unix_millis};
use crate::db::{Publication, PublishingCommand, StoreError};
use clipmill_contracts::proto::ipc::v1::{
    ConfigureYoutubePublishingRequest, DraftYoutubeMetadataRequest, ErrorCode,
    ListYoutubeUploadsResponse, StartYoutubeUploadRequest, UpdateYoutubeConnectionRequest,
    UpdateYoutubeUploadRequest, YoutubeConnectResponse, YoutubeConnectionV1,
    YoutubePublishingStatusResponse, YoutubeUploadResponse, YoutubeVideoMetadataV1, response,
};
use clipmill_youtube::{AuthorizedClient, DesktopClient, Error, Metadata};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicBool, Ordering},
};
use tokio::sync::{Mutex, watch};

const CLIENT_KEY: &str = "desktop-client";
struct Authorization {
    connection_id: String,
    request_id: String,
    url: String,
    cancel: watch::Sender<bool>,
}
type UploadTasks = HashMap<String, (String, tokio::task::JoinHandle<()>)>;
#[derive(Default)]
pub(crate) struct PublishingRuntime {
    authorization: Mutex<Option<Authorization>>,
    running: Mutex<UploadTasks>,
    stopping: AtomicBool,
}
impl std::fmt::Debug for PublishingRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PublishingRuntime([redacted])")
    }
}

impl Service {
    pub(crate) async fn recover_youtube_publishing(&self) -> Result<(), StoreError> {
        self.database
            .publishing(PublishingCommand::Recover { now: now() })
            .await?;
        // A disconnect may have been interrupted after its durable fence but
        // before Keychain deletion. It never reconnects or resumes network work.
        for connection in self
            .database
            .publishing(PublishingCommand::Connections)
            .await?
            .connections
        {
            if connection.state != "connected" {
                let _ = clipmill_youtube::forget_connection(&connection.connection_id).await;
            }
        }
        Ok(())
    }
    pub(crate) async fn stop_youtube_publishing(&self) {
        self.publishing.stopping.store(true, Ordering::SeqCst);
        if let Some(auth) = self.publishing.authorization.lock().await.take() {
            let _ = auth.cancel.send(true);
        }
        let tasks = std::mem::take(&mut *self.publishing.running.lock().await);
        for upload_id in tasks.keys() {
            if self.publication(upload_id).await.is_ok() {
                // A normal Action writes the durable pause flag; no forged
                // progress checkpoint can override a concurrent user pause.
                let _ = self
                    .database
                    .publishing(PublishingCommand::Action {
                        request_id: format!("shutdown-{}", ulid::Ulid::new()),
                        request_hash: [0; 32],
                        id: upload_id.clone(),
                        action: "pause".into(),
                        now: now(),
                    })
                    .await;
            }
        }
        let drain = async {
            for (_, (_, task)) in tasks {
                let _ = task.await;
            }
        };
        let _ = tokio::time::timeout(std::time::Duration::from_secs(5), drain).await;
    }
    pub(super) async fn pause_youtube_project(&self, project_id: &str) {
        let _ = self
            .database
            .publishing(PublishingCommand::PauseProject {
                project_id: project_id.into(),
                now: now(),
            })
            .await;
    }
    pub(super) async fn youtube_publishing_status(&self, id: String) -> Reply {
        let available = clipmill_youtube::vault::supported();
        let configured = if available {
            match clipmill_youtube::vault::optional(CLIENT_KEY) {
                Ok(bytes) => bytes.is_some(),
                Err(error) => return api_error(id, &error),
            }
        } else {
            false
        };
        match self
            .database
            .publishing(PublishingCommand::Connections)
            .await
        {
            Ok(saved) => response_reply(
                id,
                response::Body::YoutubePublishingStatus(YoutubePublishingStatusResponse {
                    configured,
                    connections: saved.connections,
                    available,
                }),
            ),
            Err(error) => store_error_reply(id, &error),
        }
    }
    pub(super) async fn configure_youtube_publishing(
        &self,
        id: String,
        request: ConfigureYoutubePublishingRequest,
    ) -> Reply {
        let path = std::path::PathBuf::from(request.client_config_path);
        if !path.is_absolute() {
            return error_reply(
                id,
                ErrorCode::InvalidArgument,
                "Choose the Desktop app client JSON file using the file picker.",
            );
        }
        let loaded = tokio::task::spawn_blocking(move || {
            use std::io::Read;
            let file = std::fs::File::open(&path)
                .map_err(|_| Error::Invalid("The selected client file could not be read"))?;
            let metadata = file
                .metadata()
                .map_err(|_| Error::Invalid("The selected client file could not be read"))?;
            if !metadata.is_file() || metadata.len() > 64 * 1024 {
                return Err(Error::Invalid(
                    "Choose a Desktop app client JSON file under 64 KiB",
                ));
            }
            let mut bytes = Vec::new();
            file.take(64 * 1024 + 1)
                .read_to_end(&mut bytes)
                .map_err(|_| Error::Invalid("The selected client file could not be read"))?;
            DesktopClient::parse(&bytes)?;
            clipmill_youtube::vault::put(CLIENT_KEY, &bytes)
        })
        .await;
        match loaded {
            Ok(Ok(())) => self.youtube_publishing_status(id).await,
            Ok(Err(error)) => api_error(id, &error),
            Err(_) => error_reply(
                id,
                ErrorCode::Internal,
                "Client configuration could not be saved.",
            ),
        }
    }
    pub(super) async fn connect_youtube_channel(&self, id: String) -> Reply {
        let mut active = self.publishing.authorization.lock().await;
        if self.publishing.stopping.load(Ordering::SeqCst) {
            return error_reply(
                id,
                ErrorCode::Unavailable,
                "The app is closing. Connect after restarting.",
            );
        }
        if let Some(auth) = active.as_ref() {
            if auth.request_id == id {
                return connect_reply(id, &auth.connection_id, &auth.url);
            }
            return error_reply(
                id,
                ErrorCode::Conflict,
                "Finish or cancel the channel connection already in progress.",
            );
        }
        let config = match clipmill_youtube::vault::get(CLIENT_KEY)
            .and_then(|bytes| DesktopClient::parse(&bytes))
        {
            Ok(value) => value,
            Err(error) => return api_error(id, &error),
        };
        let (url, pending) = match clipmill_youtube::authorize(&config).await {
            Ok(value) => value,
            Err(error) => return api_error(id, &error),
        };
        let record = YoutubeConnectionV1 {
            connection_id: format!("ytc_{}", ulid::Ulid::new()),
            channel_id: String::new(),
            title: String::new(),
            state: "connecting".into(),
            error: String::new(),
            created_unix_millis: now(),
            updated_unix_millis: now(),
        };
        if let Err(error) = self
            .database
            .publishing(PublishingCommand::SaveConnection {
                record: record.clone(),
                expected_state: None,
            })
            .await
        {
            return store_error_reply(id, &error);
        }
        let (cancel, mut stop) = watch::channel(false);
        *active = Some(Authorization {
            connection_id: record.connection_id.clone(),
            request_id: id.clone(),
            url: url.clone(),
            cancel,
        });
        self.policy.note_task_start("youtube-connect");
        let service = self.clone();
        let connection_id = record.connection_id.clone();
        tokio::spawn(async move {
            let result = tokio::select! {biased; result=clipmill_youtube::complete_authorization(&config,pending)=>Some(result),_=stop.changed()=>None};
            service
                .finish_youtube_connection(record, config, result)
                .await;
        });
        connect_reply(id, &connection_id, &url)
    }
    async fn finish_youtube_connection(
        &self,
        mut record: YoutubeConnectionV1,
        config: DesktopClient,
        result: Option<Result<clipmill_youtube::Token, Error>>,
    ) {
        let connected = async {
            let token = result.ok_or(Error::Invalid("Connection cancelled"))??;
            clipmill_youtube::store_connection(&record.connection_id, &config, &token).await?;
            let channel = AuthorizedClient::load(&record.connection_id)
                .await?
                .channel()
                .await?;
            Ok::<_, Error>(channel)
        }
        .await;
        match connected {
            Ok(channel) => {
                record.channel_id = channel.id;
                record.title = channel.title;
                "connected".clone_into(&mut record.state);
            }
            Err(error) => {
                "failed".clone_into(&mut record.state);
                record.error = error.to_string();
            }
        }
        record.updated_unix_millis = now();
        let saved = self
            .database
            .publishing(PublishingCommand::SaveConnection {
                record: record.clone(),
                expected_state: Some("connecting".into()),
            })
            .await;
        if saved.is_err() || record.state != "connected" {
            let _ = clipmill_youtube::forget_connection(&record.connection_id).await;
        }
        let mut auth = self.publishing.authorization.lock().await;
        if auth
            .as_ref()
            .is_some_and(|value| value.connection_id == record.connection_id)
        {
            auth.take();
        }
    }
    pub(super) async fn update_youtube_connection(
        &self,
        id: String,
        request: UpdateYoutubeConnectionRequest,
    ) -> Reply {
        if !matches!(request.action.as_str(), "cancel" | "disconnect") {
            return error_reply(
                id,
                ErrorCode::InvalidArgument,
                "Choose Cancel or Disconnect.",
            );
        }
        let mut record = match self.connection(&request.connection_id).await {
            Ok(value) => value,
            Err(error) => return store_error_reply(id, &error),
        };
        "disconnected".clone_into(&mut record.state);
        record.error.clear();
        record.updated_unix_millis = now();
        if let Err(error) = self
            .database
            .publishing(PublishingCommand::SaveConnection {
                record,
                expected_state: None,
            })
            .await
        {
            return store_error_reply(id, &error);
        }
        let _ = self
            .database
            .publishing(PublishingCommand::PauseConnection {
                connection_id: request.connection_id.clone(),
                now: now(),
            })
            .await;
        if let Some(auth) = self.publishing.authorization.lock().await.as_ref()
            && auth.connection_id == request.connection_id
        {
            let _ = auth.cancel.send(true);
        }
        // An in-flight request may still report success. Preserve its receipt,
        // then remove credentials; restart retries this local deletion if needed.
        let service = self.clone();
        let connection_id = request.connection_id;
        tokio::spawn(async move {
            loop {
                if !service
                    .publishing
                    .running
                    .lock()
                    .await
                    .values()
                    .any(|entry| entry.0 == connection_id)
                {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
            let _ = clipmill_youtube::forget_connection(&connection_id).await;
        });
        self.youtube_publishing_status(id).await
    }
    pub(super) async fn start_youtube_upload(
        &self,
        id: String,
        hash: [u8; 32],
        request: StartYoutubeUploadRequest,
    ) -> Reply {
        match self.database.replay_request(id.clone(), hash).await {
            Ok(Some(bytes)) => {
                return Reply {
                    bytes,
                    outcome: super::Outcome::Success,
                };
            }
            Ok(None) => {}
            Err(error) => return store_error_reply(id, &error),
        }
        if !request.rights_confirmed {
            return error_reply(
                id,
                ErrorCode::PolicyDenied,
                "Confirm that you have permission to upload this clip to the selected channel.",
            );
        }
        let Some(metadata) = request.metadata else {
            return error_reply(
                id,
                ErrorCode::InvalidArgument,
                "Review the title, description, audience and disclosure before uploading.",
            );
        };
        if let Err(error) = metadata_of(&metadata).validate() {
            return api_error(id, &error);
        }
        let connection = match self.connection(&request.connection_id).await {
            Ok(value) if value.state == "connected" => value,
            Ok(_) => {
                return error_reply(
                    id,
                    ErrorCode::Unavailable,
                    "Reconnect the selected channel before uploading.",
                );
            }
            Err(error) => return store_error_reply(id, &error),
        };
        let resolved = match self
            .verified_youtube_export(&request.export_job_id, request.expected_revision)
            .await
        {
            Ok(value) => value,
            Err(error) => return api_error(id, &error),
        };
        let record = Publication {
            view: clipmill_contracts::proto::ipc::v1::YoutubeUploadV1 {
                upload_id: format!("ytu_{}", ulid::Ulid::new()),
                project_id: resolved.project_id,
                doc_id: resolved.doc_id,
                revision: request.expected_revision,
                export_job_id: request.export_job_id,
                ir_artifact_id: resolved.ir_id,
                render_artifact_id: resolved.render_id,
                connection_id: connection.connection_id,
                channel_id: connection.channel_id,
                channel_title: connection.title,
                metadata: Some(metadata),
                state: "queued".into(),
                acknowledged_bytes: 0,
                total_bytes: resolved.bytes,
                video_id: String::new(),
                visibility: "private".into(),
                error_code: String::new(),
                error: String::new(),
                created_unix_millis: now(),
                updated_unix_millis: now(),
            },
            sha256: resolved.sha256,
            generation: 0,
            session_started: false,
            final_possible: false,
            publish_intent: false,
            paused: false,
            intent_epoch: 0,
            reset_session: false,
        };
        // Keep the verification lease until the same transaction roots the file.
        let result = self
            .database
            .publishing(PublishingCommand::Create {
                request_id: id.clone(),
                request_hash: hash,
                record,
            })
            .await;
        drop(resolved.lease);
        match result {
            Ok(mut saved) => {
                if let Some(record) = saved.uploads.pop() {
                    self.dispatch_publication(&record).await;
                }
                Reply {
                    bytes: saved.receipt.unwrap_or_default(),
                    outcome: super::Outcome::Success,
                }
            }
            Err(error) => store_error_reply(id, &error),
        }
    }
    pub(super) async fn get_youtube_upload(&self, id: String, upload_id: String) -> Reply {
        match self.publication(&upload_id).await {
            Ok(record) => upload_reply(id, record),
            Err(error) => store_error_reply(id, &error),
        }
    }
    pub(super) async fn list_youtube_uploads(&self, id: String, project_id: String) -> Reply {
        match self
            .database
            .publishing(PublishingCommand::Uploads { project_id })
            .await
        {
            Ok(saved) => response_reply(
                id,
                response::Body::ListYoutubeUploads(ListYoutubeUploadsResponse {
                    uploads: saved
                        .uploads
                        .into_iter()
                        .map(|record| record.view)
                        .collect(),
                }),
            ),
            Err(error) => store_error_reply(id, &error),
        }
    }
    pub(super) async fn update_youtube_upload(
        &self,
        id: String,
        hash: [u8; 32],
        request: UpdateYoutubeUploadRequest,
    ) -> Reply {
        if !matches!(request.action.as_str(), "pause" | "resume" | "reconcile") {
            return error_reply(
                id,
                ErrorCode::InvalidArgument,
                "Choose Pause, Resume or Reconcile.",
            );
        }
        self.publication_action(id, hash, request.upload_id, request.action)
            .await
    }
    pub(super) async fn publish_youtube_upload(
        &self,
        id: String,
        hash: [u8; 32],
        upload_id: String,
    ) -> Reply {
        self.publication_action(id, hash, upload_id, "publish".into())
            .await
    }
    async fn publication_action(
        &self,
        id: String,
        hash: [u8; 32],
        upload_id: String,
        action: String,
    ) -> Reply {
        match self
            .database
            .publishing(PublishingCommand::Action {
                request_id: id.clone(),
                request_hash: hash,
                id: upload_id,
                action,
                now: now(),
            })
            .await
        {
            Ok(mut saved) => {
                if let Some(record) = saved.uploads.pop() {
                    self.dispatch_publication(&record).await;
                }
                Reply {
                    bytes: saved.receipt.unwrap_or_default(),
                    outcome: super::Outcome::Success,
                }
            }
            Err(error) => store_error_reply(id, &error),
        }
    }
    async fn publication(&self, id: &str) -> Result<Publication, StoreError> {
        self.database
            .publishing(PublishingCommand::Read { id: id.into() })
            .await?
            .uploads
            .pop()
            .ok_or(StoreError::NotFound)
    }
    async fn connection(&self, id: &str) -> Result<YoutubeConnectionV1, StoreError> {
        self.database
            .publishing(PublishingCommand::Connections)
            .await?
            .connections
            .into_iter()
            .find(|record| record.connection_id == id)
            .ok_or(StoreError::NotFound)
    }
    async fn dispatch_publication(&self, record: &Publication) {
        if record.paused
            || !matches!(
                record.view.state.as_str(),
                "queued" | "reconciling" | "publishing"
            )
        {
            return;
        }
        let mut running = self.publishing.running.lock().await;
        if self.publishing.stopping.load(Ordering::SeqCst)
            || running.contains_key(&record.view.upload_id)
        {
            return;
        }
        let service = self.clone();
        let id = record.view.upload_id.clone();
        let task = tokio::spawn(async move {
            loop {
                service.run_publication(&id).await;
                if !service.continue_publication(&id).await {
                    break;
                }
            }
        });
        running.insert(
            record.view.upload_id.clone(),
            (record.view.connection_id.clone(), task),
        );
    }
    // The durable intent is read while holding the same lock used by dispatch.
    // An action either becomes visible here or dispatches after this task retires.
    async fn continue_publication(&self, id: &str) -> bool {
        let mut running = self.publishing.running.lock().await;
        let eligible = !self.publishing.stopping.load(Ordering::SeqCst)
            && self.publication(id).await.is_ok_and(|record| {
                !record.paused
                    && matches!(
                        record.view.state.as_str(),
                        "queued" | "reconciling" | "publishing"
                    )
            });
        if !eligible {
            running.remove(id);
        }
        eligible
    }
    async fn run_publication(&self, id: &str) {
        let Ok(mut claimed) = self
            .database
            .publishing(PublishingCommand::Claim {
                id: id.into(),
                now: now(),
            })
            .await
        else {
            return;
        };
        let Some(mut record) = claimed.uploads.pop() else {
            return;
        };
        if let Some((connection_id, _)) = self.publishing.running.lock().await.get_mut(id) {
            connection_id.clone_from(&record.view.connection_id);
        }
        self.policy.note_task_start("youtube-upload");
        let bound_connection = record.view.connection_id.clone();
        let result = async {
            let connection = self
                .connection(&record.view.connection_id)
                .await
                .map_err(|_| Error::MissingCredential)?;
            if connection.state != "connected" {
                return Err(Error::MissingCredential);
            }
            let client = AuthorizedClient::load(&record.view.connection_id).await?;
            if record.reset_session {
                // Only this upload's sole running worker may consume this
                // explicit retry intent; an ambiguous final send cannot reset.
                if record.final_possible {
                    return Err(Error::Protocol);
                }
                clipmill_youtube::forget_session(&record.view.upload_id)?;
                record = self
                    .database
                    .publishing(PublishingCommand::ResetExpiredSession {
                        id: record.view.upload_id.clone(),
                        generation: record.generation,
                    })
                    .await
                    .map_err(|_| Error::Protocol)?
                    .uploads
                    .pop()
                    .ok_or(Error::Protocol)?;
            }
            self.transfer_publication(&client, &mut record).await
        }
        .await;
        if let Err(error) = result {
            self.fail_publication(&mut record, &error, &bound_connection)
                .await;
        }
    }
    async fn checkpoint_publication(&self, record: &mut Publication) -> Result<(), Error> {
        record.view.updated_unix_millis = now();
        *record = self
            .database
            .publishing(PublishingCommand::Checkpoint {
                record: record.clone(),
            })
            .await
            .map_err(|_| {
                Error::Invalid(
                    "Upload state could not be saved. Resume to reconcile the existing session.",
                )
            })?
            .uploads
            .pop()
            .ok_or(Error::Protocol)?;
        Ok(())
    }
    async fn fail_publication(
        &self,
        record: &mut Publication,
        error: &Error,
        bound_connection: &str,
    ) {
        if matches!(error, Error::Authorization | Error::MissingCredential)
            && let Ok(mut connection) = self.connection(bound_connection).await
        {
            "failed".clone_into(&mut connection.state);
            "Reconnect this channel to continue.".clone_into(&mut connection.error);
            connection.updated_unix_millis = now();
            let _ = self
                .database
                .publishing(PublishingCommand::SaveConnection {
                    record: connection,
                    expected_state: Some("connected".into()),
                })
                .await;
        }
        let (state, code) = match error {
            Error::Authorization | Error::MissingCredential => ("auth_required", "reconnect"),
            Error::ExpiredSession | Error::Protocol if record.final_possible => {
                ("completion_uncertain", "completion_uncertain")
            }
            Error::ExpiredSession => ("failed", "session_expired"),
            _ => ("failed", "upload_failed"),
        };
        state.clone_into(&mut record.view.state);
        code.clone_into(&mut record.view.error_code);
        record.view.error = error.to_string();
        let _ = self.checkpoint_publication(record).await;
    }
    pub(super) async fn draft_youtube_metadata(
        &self,
        id: String,
        request: DraftYoutubeMetadataRequest,
    ) -> Reply {
        match self.youtube_metadata_draft(&request).await {
            Ok(value) => response_reply(id, response::Body::DraftYoutubeMetadata(value)),
            Err(error) => api_error(id, &error),
        }
    }
}
fn api_error(id: String, error: &Error) -> Reply {
    let code = if matches!(error, Error::Invalid(_)) {
        ErrorCode::InvalidArgument
    } else {
        ErrorCode::Unavailable
    };
    error_reply(id, code, error.to_string())
}
fn connect_reply(id: String, connection_id: &str, url: &str) -> Reply {
    response_reply(
        id,
        response::Body::YoutubeConnect(YoutubeConnectResponse {
            connection_id: connection_id.into(),
            authorization_url: url.into(),
        }),
    )
}
fn upload_reply(id: String, record: Publication) -> Reply {
    response_reply(
        id,
        response::Body::YoutubeUpload(YoutubeUploadResponse {
            record: Some(record.view),
        }),
    )
}
fn metadata_of(value: &YoutubeVideoMetadataV1) -> Metadata {
    Metadata {
        title: value.title.clone(),
        description: value.description.clone(),
        tags: value.tags.clone(),
        made_for_kids: value.made_for_kids,
        contains_synthetic_media: value.contains_synthetic_media,
    }
}
fn now() -> u64 {
    unix_millis().unwrap_or(0)
}
