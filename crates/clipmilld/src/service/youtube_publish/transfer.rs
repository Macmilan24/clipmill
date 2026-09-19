use super::{AuthorizedClient, Error, Metadata, Publication, Service, metadata_of};
use clipmill_youtube::{Channel, SessionHandle, UploadReply, VideoStatus};
use std::future::Future;
use std::sync::atomic::Ordering;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

const CHUNK_BYTES: usize = 8 * 1024 * 1024;

/// Only the engine is generic; production always constructs the fixed-provider
/// `AuthorizedClient`. Test doubles cannot install a renderer-controlled endpoint.
pub(super) trait UploadApi: Send + Sync {
    type Session: Send + Sync;
    fn channel(&self) -> impl Future<Output = Result<Channel, Error>> + Send;
    fn existing(&self, id: &str) -> Result<Option<Self::Session>, Error>;
    fn begin(
        &self,
        id: &str,
        metadata: &Metadata,
        total: u64,
    ) -> impl Future<Output = Result<Self::Session, Error>> + Send;
    fn position(
        &self,
        session: &Self::Session,
        total: u64,
    ) -> impl Future<Output = Result<UploadReply, Error>> + Send;
    fn chunk(
        &self,
        session: &Self::Session,
        start: u64,
        total: u64,
        bytes: Vec<u8>,
    ) -> impl Future<Output = Result<UploadReply, Error>> + Send;
    fn video(
        &self,
        id: &str,
        channel: &str,
    ) -> impl Future<Output = Result<VideoStatus, Error>> + Send;
    fn publish(
        &self,
        id: &str,
        channel: &str,
    ) -> impl Future<Output = Result<VideoStatus, Error>> + Send;
}
impl UploadApi for AuthorizedClient {
    type Session = SessionHandle;
    async fn channel(&self) -> Result<Channel, Error> {
        self.channel().await
    }
    fn existing(&self, id: &str) -> Result<Option<SessionHandle>, Error> {
        self.existing_session(id)
    }
    async fn begin(
        &self,
        id: &str,
        metadata: &Metadata,
        total: u64,
    ) -> Result<SessionHandle, Error> {
        self.begin_private(id, metadata, total).await
    }
    async fn position(&self, session: &SessionHandle, total: u64) -> Result<UploadReply, Error> {
        self.position(session, total).await
    }
    async fn chunk(
        &self,
        session: &SessionHandle,
        start: u64,
        total: u64,
        bytes: Vec<u8>,
    ) -> Result<UploadReply, Error> {
        self.chunk(session, start, total, bytes).await
    }
    async fn video(&self, id: &str, channel: &str) -> Result<VideoStatus, Error> {
        self.video(id, channel).await
    }
    async fn publish(&self, id: &str, channel: &str) -> Result<VideoStatus, Error> {
        self.publish(id, channel).await
    }
}

impl Service {
    pub(super) async fn transfer_publication(
        &self,
        client: &impl UploadApi,
        record: &mut Publication,
    ) -> Result<(), Error> {
        let bound_connection = record.view.connection_id.clone();
        if client.channel().await?.id != record.view.channel_id {
            return Err(Error::Invalid(
                "The connected channel changed. Reconnect the originally approved channel.",
            ));
        }
        if self.publication_paused(record).await? || record.view.connection_id != bound_connection {
            return Ok(());
        }
        if !record.view.video_id.is_empty() {
            return self.reconcile_publication(client, record).await;
        }
        let (_lease, file) = self.verified_publication_file(record).await?;
        self.transfer_verified_bytes(client, record, file).await
    }
    pub(super) async fn transfer_verified_bytes(
        &self,
        client: &impl UploadApi,
        record: &mut Publication,
        mut file: tokio::fs::File,
    ) -> Result<(), Error> {
        let bound_connection = record.view.connection_id.clone();
        let session = if let Some(session) = client.existing(&record.view.upload_id)? {
            session
        } else {
            // Losing a persisted session is not permission to initiate another
            // remote insert, even if the local byte counter says zero.
            if record.session_started || record.final_possible {
                return Err(Error::ExpiredSession);
            }
            "starting".clone_into(&mut record.view.state);
            self.checkpoint_publication(record).await?;
            if self.publication_paused(record).await?
                || record.view.connection_id != bound_connection
            {
                return Ok(());
            }
            let metadata = record.view.metadata.as_ref().ok_or(Error::Protocol)?;
            client
                .begin(
                    &record.view.upload_id,
                    &metadata_of(metadata),
                    record.view.total_bytes,
                )
                .await?
        };
        record.session_started = true;
        self.checkpoint_publication(record).await?;
        if record.paused
            || record.view.connection_id != bound_connection
            || self.publishing.stopping.load(Ordering::SeqCst)
        {
            return Ok(());
        }
        // Server acknowledgement is authoritative on every entry, including the
        // crash between Keychain session persistence and its SQLite checkpoint.
        let response = client.position(&session, record.view.total_bytes).await?;
        if self.apply_upload_reply(record, response).await? {
            return Ok(());
        }
        loop {
            if self.publication_paused(record).await?
                || record.view.connection_id != bound_connection
            {
                return Ok(());
            }
            let start = record.view.acknowledged_bytes;
            let remaining = record
                .view
                .total_bytes
                .checked_sub(start)
                .ok_or(Error::Protocol)?;
            if remaining == 0 {
                // A 308 response can acknowledge the whole file without proving
                // video creation. Only an actual video receipt finishes upload.
                record.final_possible = true;
                self.checkpoint_publication(record).await?;
                return Err(Error::Protocol);
            }
            let count =
                usize::try_from(remaining.min(CHUNK_BYTES as u64)).map_err(|_| Error::Protocol)?;
            file.seek(std::io::SeekFrom::Start(start))
                .await
                .map_err(|_| Error::Invalid("The approved video could not be read."))?;
            let mut bytes = vec![0_u8; count];
            file.read_exact(&mut bytes).await.map_err(|_| {
                Error::Invalid("The approved video became unavailable during upload.")
            })?;
            if start + count as u64 == record.view.total_bytes {
                record.final_possible = true;
            }
            "uploading".clone_into(&mut record.view.state);
            self.checkpoint_publication(record).await?;
            if record.paused || record.view.connection_id != bound_connection {
                return Ok(());
            }
            // Let an in-flight request settle after Pause. Its response may be
            // the only receipt for a video the server has already created.
            let response = client
                .chunk(&session, start, record.view.total_bytes, bytes)
                .await?;
            if matches!(&response, UploadReply::Incomplete { acknowledged } if *acknowledged <= start)
            {
                // Stop a non-progressing session. An explicit resume asks the
                // server for its position again instead of looping media PUTs.
                return Err(Error::Protocol);
            }
            if self.apply_upload_reply(record, response).await? {
                return Ok(());
            }
        }
    }
    async fn apply_upload_reply(
        &self,
        record: &mut Publication,
        response: UploadReply,
    ) -> Result<bool, Error> {
        match response {
            UploadReply::Incomplete { acknowledged } => {
                if acknowledged > record.view.total_bytes {
                    return Err(Error::Protocol);
                }
                record.view.acknowledged_bytes = acknowledged;
                "uploading".clone_into(&mut record.view.state);
                self.checkpoint_publication(record).await?;
                Ok(false)
            }
            UploadReply::Complete {
                video_id,
                channel_id,
                privacy,
            } => {
                if channel_id != record.view.channel_id || video_id.is_empty() {
                    return Err(Error::Protocol);
                }
                record.view.video_id = video_id;
                record.view.acknowledged_bytes = record.view.total_bytes;
                if !matches!(privacy.as_str(), "private" | "unlisted" | "public") {
                    return Err(Error::Protocol);
                }
                (if privacy == "public" {
                    "public"
                } else {
                    "private"
                })
                .clone_into(&mut record.view.state);
                record.view.visibility = privacy;
                record.view.error.clear();
                record.view.error_code.clear();
                self.checkpoint_publication(record).await?;
                Ok(true)
            }
        }
    }
    async fn publication_paused(&self, record: &mut Publication) -> Result<bool, Error> {
        let saved = self
            .publication(&record.view.upload_id)
            .await
            .map_err(|_| Error::Protocol)?;
        if saved.generation != record.generation {
            return Err(Error::Protocol);
        }
        *record = saved;
        Ok(record.paused || self.publishing.stopping.load(Ordering::SeqCst))
    }
    async fn publish_visibility(
        &self,
        client: &impl UploadApi,
        record: &mut Publication,
        bound_connection: &str,
    ) -> Result<Option<VideoStatus>, Error> {
        "publishing".clone_into(&mut record.view.state);
        self.checkpoint_publication(record).await?;
        if record.paused
            || record.view.connection_id != bound_connection
            || self.publishing.stopping.load(Ordering::SeqCst)
        {
            return Ok(None);
        }
        client
            .publish(&record.view.video_id, &record.view.channel_id)
            .await
            .map(Some)
    }
    async fn reconcile_publication(
        &self,
        client: &impl UploadApi,
        record: &mut Publication,
    ) -> Result<(), Error> {
        let bound_connection = record.view.connection_id.clone();
        let mut remote = client
            .video(&record.view.video_id, &record.view.channel_id)
            .await?;
        if remote.id != record.view.video_id || remote.channel_id != record.view.channel_id {
            return Err(Error::Protocol);
        }
        if record.publish_intent && remote.privacy != "public" {
            if remote.status != "processed" {
                return Err(Error::Invalid(
                    "YouTube is still processing or has refused this video. Review it in YouTube Studio before publishing.",
                ));
            }
            if self.publication_paused(record).await?
                || record.view.connection_id != bound_connection
            {
                return Ok(());
            }
            let Some(published) = self
                .publish_visibility(client, record, &bound_connection)
                .await?
            else {
                return Ok(());
            };
            remote = published;
        }
        if remote.id != record.view.video_id || remote.channel_id != record.view.channel_id {
            return Err(Error::Protocol);
        }
        record.view.visibility = remote.privacy;
        if record.view.visibility == "public" {
            "public"
        } else {
            "private"
        }
        .clone_into(&mut record.view.state);
        record.view.error.clear();
        record.view.error_code.clear();
        self.checkpoint_publication(record).await
    }
}

#[cfg(test)]
mod tests;
