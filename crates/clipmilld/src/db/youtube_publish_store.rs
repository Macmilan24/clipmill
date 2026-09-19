//! Durable remote side effects. Receipts outlive their local project so a
//! deleted project cannot erase knowledge of a possibly created remote video.
use super::{StoreError, remember, replay};
use clipmill_contracts::proto::ipc::v1::{
    Response, YoutubeConnectionV1, YoutubeUploadResponse, YoutubeUploadV1, response,
};
use prost::Message;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

pub(super) const CREATE_V14_TABLES: &str = "
 CREATE TABLE youtube_connections (
   connection_id TEXT PRIMARY KEY, record BLOB NOT NULL
 ) STRICT;
 CREATE TABLE youtube_uploads (
   upload_id TEXT PRIMARY KEY,
   project_id TEXT NOT NULL,
   channel_id TEXT NOT NULL,
   doc_id TEXT NOT NULL,
   revision INTEGER NOT NULL,
   ir_artifact_id TEXT NOT NULL,
   record BLOB NOT NULL,
   sha256 TEXT NOT NULL,
   generation INTEGER NOT NULL DEFAULT 0,
   session_started INTEGER NOT NULL DEFAULT 0,
   final_possible INTEGER NOT NULL DEFAULT 0,
   publish_intent INTEGER NOT NULL DEFAULT 0,
   paused INTEGER NOT NULL DEFAULT 0,
   intent_epoch INTEGER NOT NULL DEFAULT 0,
   reset_session INTEGER NOT NULL DEFAULT 0,
   UNIQUE(channel_id,doc_id,revision,ir_artifact_id)
 ) STRICT;
 CREATE TABLE youtube_upload_roots (
   upload_id TEXT NOT NULL REFERENCES youtube_uploads(upload_id) ON DELETE CASCADE,
   artifact_id TEXT NOT NULL,
   PRIMARY KEY(upload_id,artifact_id)
 ) STRICT;
";

// These are independent monotonic side-effect facts and user intents; they
// deliberately survive transitions between the presentation states.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Publication {
    pub view: YoutubeUploadV1,
    pub sha256: String,
    pub generation: i64,
    pub session_started: bool,
    pub final_possible: bool,
    pub publish_intent: bool,
    pub paused: bool,
    pub intent_epoch: i64,
    pub reset_session: bool,
}

#[derive(Debug, Default)]
pub(crate) struct PublishingReply {
    pub connections: Vec<YoutubeConnectionV1>,
    pub uploads: Vec<Publication>,
    pub receipt: Option<Vec<u8>>,
    pub export_payload: Option<clipmill_contracts::proto::ipc::v1::ExportClipPayloadV1>,
}

#[derive(Debug)]
pub(crate) enum PublishingCommand {
    ExportPayload {
        job_id: String,
    },
    Connections,
    SaveConnection {
        record: YoutubeConnectionV1,
        expected_state: Option<String>,
    },
    Uploads {
        project_id: String,
    },
    Read {
        id: String,
    },
    Create {
        request_id: String,
        request_hash: [u8; 32],
        record: Publication,
    },
    Action {
        request_id: String,
        request_hash: [u8; 32],
        id: String,
        action: String,
        now: u64,
    },
    Claim {
        id: String,
        now: u64,
    },
    Checkpoint {
        record: Publication,
    },
    ResetExpiredSession {
        id: String,
        generation: i64,
    },
    Recover {
        now: u64,
    },
    PauseProject {
        project_id: String,
        now: u64,
    },
    PauseConnection {
        connection_id: String,
        now: u64,
    },
}

#[allow(
    clippy::too_many_lines,
    reason = "single serialized publishing state machine"
)]
pub(super) fn execute(
    connection: &mut Connection,
    command: PublishingCommand,
) -> Result<PublishingReply, StoreError> {
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let mut reply = PublishingReply::default();
    match command {
        PublishingCommand::ExportPayload { job_id } => {
            let bytes: Vec<u8> = tx
                .query_row(
                    "SELECT payload FROM jobs WHERE job_id=?1 AND kind='export-clip'",
                    [job_id],
                    |row| row.get(0),
                )
                .optional()?
                .ok_or(StoreError::NotFound)?;
            reply.export_payload = Some(
                clipmill_contracts::proto::ipc::v1::ExportClipPayloadV1::decode(bytes.as_slice())
                    .map_err(|_| StoreError::InvalidData("invalid export request"))?,
            );
        }
        PublishingCommand::Connections => reply.connections = connections(&tx)?,
        PublishingCommand::SaveConnection {
            record,
            expected_state,
        } => {
            if let Some(expected) = expected_state {
                let saved = connections(&tx)?
                    .into_iter()
                    .find(|saved| saved.connection_id == record.connection_id)
                    .ok_or(StoreError::NotFound)?;
                if saved.state != expected {
                    return Err(StoreError::Conflict);
                }
            }
            tx.execute("INSERT INTO youtube_connections VALUES(?1,?2) ON CONFLICT(connection_id) DO UPDATE SET record=excluded.record", params![record.connection_id,record.encode_to_vec()])?;
            reply.connections.push(record);
        }
        PublishingCommand::Uploads { project_id } => reply.uploads = uploads(&tx, &project_id)?,
        PublishingCommand::Read { id } => reply.uploads.push(read(&tx, &id)?),
        PublishingCommand::Create {
            request_id,
            request_hash,
            record,
        } => {
            if let Some(bytes) = replay(&tx, &request_id, &request_hash)? {
                let id = receipt_id(&bytes)?;
                reply.uploads.push(read(&tx, &id)?);
                reply.receipt = Some(bytes);
            } else {
                let existing: Option<String> = tx.query_row("SELECT upload_id FROM youtube_uploads WHERE channel_id=?1 AND doc_id=?2 AND revision=?3 AND ir_artifact_id=?4", params![record.view.channel_id,record.view.doc_id,i64::try_from(record.view.revision).map_err(|_| StoreError::InvalidData("revision is too large"))?,record.view.ir_artifact_id], |row| row.get(0)).optional()?;
                let saved = if let Some(id) = existing {
                    let prior = read(&tx, &id)?;
                    if prior.view.metadata != record.view.metadata
                        || prior.view.render_artifact_id != record.view.render_artifact_id
                    {
                        return Err(StoreError::PublishingConflict(
                            "This revision already has an upload for this channel. Open that upload instead of creating another one.",
                        ));
                    }
                    prior
                } else {
                    let exists: bool = tx.query_row(
                        "SELECT EXISTS(SELECT 1 FROM projects WHERE project_id=?1)",
                        [&record.view.project_id],
                        |row| row.get(0),
                    )?;
                    if !exists {
                        return Err(StoreError::NotFound);
                    }
                    tx.execute("INSERT INTO youtube_uploads VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)", params![record.view.upload_id,record.view.project_id,record.view.channel_id,record.view.doc_id,i64::try_from(record.view.revision).map_err(|_| StoreError::InvalidData("revision is too large"))?,record.view.ir_artifact_id,record.view.encode_to_vec(),record.sha256,record.generation,record.session_started,record.final_possible,record.publish_intent,record.paused,record.intent_epoch,record.reset_session])?;
                    for artifact in [&record.view.ir_artifact_id, &record.view.render_artifact_id] {
                        tx.execute(
                            "INSERT INTO youtube_upload_roots VALUES(?1,?2)",
                            params![record.view.upload_id, artifact],
                        )?;
                    }
                    record
                };
                reply.receipt = Some(remember_upload(
                    &tx,
                    &request_id,
                    &request_hash,
                    &saved.view,
                )?);
                reply.uploads.push(saved);
            }
        }
        PublishingCommand::Action {
            request_id,
            request_hash,
            id,
            action,
            now,
        } => {
            if let Some(bytes) = replay(&tx, &request_id, &request_hash)? {
                reply.uploads.push(read(&tx, &receipt_id(&bytes)?)?);
                reply.receipt = Some(bytes);
            } else {
                let mut record = read(&tx, &id)?;
                if matches!(action.as_str(), "resume" | "reconcile" | "publish") {
                    let channels = connections(&tx)?;
                    let connected = channels.iter().find(|channel| {
                        channel.connection_id == record.view.connection_id
                            && channel.state == "connected"
                            && channel.channel_id == record.view.channel_id
                    });
                    if connected.is_none() {
                        let replacement = channels
                            .iter()
                            .find(|channel| {
                                channel.state == "connected"
                                    && channel.channel_id == record.view.channel_id
                            })
                            .ok_or(StoreError::PublishingConflict(
                                "Reconnect the same YouTube channel, then resume this upload.",
                            ))?;
                        record
                            .view
                            .connection_id
                            .clone_from(&replacement.connection_id);
                    }
                }
                match action.as_str() {
                    "pause" if !matches!(record.view.state.as_str(), "private" | "public") => {
                        record.paused = true;
                        "paused".clone_into(&mut record.view.state);
                    }
                    "resume" | "reconcile"
                        if matches!(
                            record.view.state.as_str(),
                            "paused" | "failed" | "auth_required" | "completion_uncertain"
                        ) =>
                    {
                        record.paused = false;
                        record.reset_session |=
                            record.view.error_code == "session_expired" && !record.final_possible;
                        if record.publish_intent {
                            "publishing"
                        } else if record.session_started || record.final_possible {
                            "reconciling"
                        } else {
                            "queued"
                        }
                        .clone_into(&mut record.view.state);
                    }
                    "publish"
                        if !record.view.video_id.is_empty()
                            && matches!(
                                record.view.state.as_str(),
                                "private" | "publishing" | "public"
                            ) =>
                    {
                        if record.view.state != "public" {
                            record.publish_intent = true;
                            record.paused = false;
                            "publishing".clone_into(&mut record.view.state);
                        }
                    }
                    _ => {
                        return Err(StoreError::PublishingConflict(
                            "That action is not available for this upload's current state.",
                        ));
                    }
                }
                record.intent_epoch = record
                    .intent_epoch
                    .checked_add(1)
                    .ok_or(StoreError::Conflict)?;
                record.view.error.clear();
                record.view.error_code.clear();
                record.view.updated_unix_millis = now;
                save(&tx, &record)?;
                reply.receipt = Some(remember_upload(
                    &tx,
                    &request_id,
                    &request_hash,
                    &record.view,
                )?);
                reply.uploads.push(record);
            }
        }
        PublishingCommand::Claim { id, now } => {
            let mut record = read(&tx, &id)?;
            if record.paused
                || !matches!(
                    record.view.state.as_str(),
                    "queued" | "reconciling" | "publishing"
                )
            {
                return Err(StoreError::Conflict);
            }
            record.generation = record
                .generation
                .checked_add(1)
                .ok_or(StoreError::Conflict)?;
            if !record.publish_intent {
                "verifying".clone_into(&mut record.view.state);
            }
            record.view.updated_unix_millis = now;
            save(&tx, &record)?;
            reply.uploads.push(record);
        }
        PublishingCommand::ResetExpiredSession { id, generation } => {
            let mut record = read(&tx, &id)?;
            if record.generation != generation
                || !record.reset_session
                || record.final_possible
                || !record.view.video_id.is_empty()
                || record.paused
            {
                return Err(StoreError::Conflict);
            }
            record.session_started = false;
            record.reset_session = false;
            record.view.acknowledged_bytes = 0;
            record.view.error.clear();
            record.view.error_code.clear();
            save(&tx, &record)?;
            reply.uploads.push(record);
        }
        PublishingCommand::Checkpoint { mut record } => {
            let previous = read(&tx, &record.view.upload_id)?;
            if previous.generation != record.generation {
                return Err(StoreError::Conflict);
            }
            // Pause is local intent. It cannot erase an authoritative late
            // remote receipt, and a progress callback cannot silently unpause.
            record
                .view
                .connection_id
                .clone_from(&previous.view.connection_id);
            if record.intent_epoch != previous.intent_epoch
                && matches!(
                    previous.view.state.as_str(),
                    "queued" | "reconciling" | "publishing"
                )
                && record.view.state != "public"
            {
                record.view.state.clone_from(&previous.view.state);
            }
            record.intent_epoch = previous.intent_epoch;
            record.reset_session = previous.reset_session;
            record.paused = previous.paused;
            record.publish_intent |= previous.publish_intent;
            if record.paused && !matches!(record.view.state.as_str(), "private" | "public") {
                "paused".clone_into(&mut record.view.state);
            }
            if matches!(record.view.state.as_str(), "private" | "public")
                && record.view.video_id.is_empty()
            {
                return Err(StoreError::InvalidData(
                    "remote success requires a video receipt",
                ));
            }
            if !previous.view.video_id.is_empty() && previous.view.video_id != record.view.video_id
            {
                return Err(StoreError::InvalidData("remote video identity changed"));
            }
            if record.view.acknowledged_bytes > record.view.total_bytes {
                return Err(StoreError::InvalidData(
                    "upload acknowledged beyond its immutable file",
                ));
            }
            record.session_started |= previous.session_started;
            record.final_possible |= previous.final_possible;
            save(&tx, &record)?;
            if !record.view.video_id.is_empty() {
                // A durable remote receipt makes local upload bytes disposable.
                tx.execute(
                    "DELETE FROM youtube_upload_roots WHERE upload_id=?1",
                    [&record.view.upload_id],
                )?;
            }
            reply.uploads.push(record);
        }
        PublishingCommand::Recover { now } => {
            for mut connection in connections(&tx)? {
                if connection.state == "connecting" {
                    "interrupted".clone_into(&mut connection.state);
                    "Connection interrupted. Connect again when you are ready."
                        .clone_into(&mut connection.error);
                    connection.updated_unix_millis = now;
                    tx.execute(
                        "UPDATE youtube_connections SET record=?1 WHERE connection_id=?2",
                        params![connection.encode_to_vec(), connection.connection_id],
                    )?;
                }
            }
            for mut record in uploads(&tx, "")? {
                if matches!(
                    record.view.state.as_str(),
                    "queued"
                        | "verifying"
                        | "starting"
                        | "uploading"
                        | "reconciling"
                        | "publishing"
                ) {
                    record.paused = true;
                    "paused".clone_into(&mut record.view.state);
                    "The app stopped. Resume to check the existing upload before sending more data.".clone_into(&mut record.view.error);
                    record.view.updated_unix_millis = now;
                    save(&tx, &record)?;
                }
            }
        }
        PublishingCommand::PauseProject { project_id, now } => {
            pause_matching(&tx, now, |record| record.view.project_id == project_id)?;
        }
        PublishingCommand::PauseConnection { connection_id, now } => {
            pause_matching(&tx, now, |record| {
                record.view.connection_id == connection_id
            })?;
        }
    }
    tx.commit()?;
    Ok(reply)
}

fn pause_matching(
    connection: &Connection,
    now: u64,
    predicate: impl Fn(&Publication) -> bool,
) -> Result<(), StoreError> {
    for mut record in uploads(connection, "")? {
        if predicate(&record) && !matches!(record.view.state.as_str(), "private" | "public") {
            record.paused = true;
            "paused".clone_into(&mut record.view.state);
            record.view.updated_unix_millis = now;
            save(connection, &record)?;
        }
    }
    Ok(())
}
fn connections(connection: &Connection) -> Result<Vec<YoutubeConnectionV1>, StoreError> {
    let mut query =
        connection.prepare("SELECT record FROM youtube_connections ORDER BY rowid DESC")?;
    query
        .query_map([], |row| row.get::<_, Vec<u8>>(0))?
        .map(|bytes| {
            YoutubeConnectionV1::decode(bytes?.as_slice())
                .map_err(|_| StoreError::InvalidData("invalid channel connection"))
        })
        .collect()
}
fn uploads(connection: &Connection, project: &str) -> Result<Vec<Publication>, StoreError> {
    let mut query = connection.prepare(
        "SELECT upload_id FROM youtube_uploads WHERE ?1='' OR project_id=?1 ORDER BY rowid DESC",
    )?;
    query
        .query_map([project], |row| row.get::<_, String>(0))?
        .map(|id| read(connection, &id?))
        .collect()
}
fn read(connection: &Connection, id: &str) -> Result<Publication, StoreError> {
    let (bytes,sha256,generation,session_started,final_possible,publish_intent,paused,intent_epoch,reset_session):(Vec<u8>,String,i64,bool,bool,bool,bool,i64,bool)=connection.query_row("SELECT record,sha256,generation,session_started,final_possible,publish_intent,paused,intent_epoch,reset_session FROM youtube_uploads WHERE upload_id=?1",[id],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?,row.get(6)?,row.get(7)?,row.get(8)?))).optional()?.ok_or(StoreError::NotFound)?;
    let view = YoutubeUploadV1::decode(bytes.as_slice())
        .map_err(|_| StoreError::InvalidData("invalid upload record"))?;
    Ok(Publication {
        view,
        sha256,
        generation,
        session_started,
        final_possible,
        publish_intent,
        paused,
        intent_epoch,
        reset_session,
    })
}
fn save(connection: &Connection, record: &Publication) -> Result<(), StoreError> {
    connection.execute("UPDATE youtube_uploads SET record=?1,generation=?2,session_started=?3,final_possible=?4,publish_intent=?5,paused=?6,intent_epoch=?7,reset_session=?8 WHERE upload_id=?9",params![record.view.encode_to_vec(),record.generation,record.session_started,record.final_possible,record.publish_intent,record.paused,record.intent_epoch,record.reset_session,record.view.upload_id])?;
    Ok(())
}
fn remember_upload(
    connection: &rusqlite::Transaction<'_>,
    id: &str,
    hash: &[u8; 32],
    record: &YoutubeUploadV1,
) -> Result<Vec<u8>, StoreError> {
    let bytes = Response {
        request_id: id.into(),
        body: Some(response::Body::YoutubeUpload(YoutubeUploadResponse {
            record: Some(record.clone()),
        })),
    }
    .encode_to_vec();
    remember(connection, id, hash, &bytes, record.updated_unix_millis)?;
    Ok(bytes)
}
fn receipt_id(bytes: &[u8]) -> Result<String, StoreError> {
    match Response::decode(bytes)
        .map_err(|_| StoreError::InvalidData("invalid upload receipt"))?
        .body
    {
        Some(response::Body::YoutubeUpload(reply)) => reply
            .record
            .map(|record| record.upload_id)
            .ok_or(StoreError::InvalidData("upload receipt missing record")),
        _ => Err(StoreError::InvalidData("wrong upload receipt")),
    }
}

#[cfg(test)]
mod tests;
