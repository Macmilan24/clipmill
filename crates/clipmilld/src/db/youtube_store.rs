//! Durable acquisition intents; attempt/state fences make late callbacks inert.
use super::{StoreError, remember, replay, source_store};
use crate::sources::InspectedSource;
use clipmill_contracts::proto::ipc::v1::{
    Response, YoutubeImportResponse, YoutubeImportV1, response,
};
use prost::Message;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

pub(super) const CREATE_V13_TABLES: &str = "
 CREATE TABLE youtube_imports (
   import_id TEXT PRIMARY KEY,
   project_id TEXT NOT NULL REFERENCES projects(project_id) ON DELETE CASCADE,
   video_id TEXT NOT NULL, canonical_url TEXT NOT NULL,
   rights_confirmed INTEGER NOT NULL CHECK(rights_confirmed=1),
   state TEXT NOT NULL CHECK(state IN ('queued','downloading','processing','registering','completed','failed','cancelled','interrupted')),
   attempt INTEGER NOT NULL, record BLOB NOT NULL,
   UNIQUE(project_id,video_id)
 ) STRICT;
";

#[derive(Debug)]
pub(crate) enum YoutubeCommand {
    Create {
        request_id: String,
        request_hash: [u8; 32],
        record: YoutubeImportV1,
    },
    Read {
        id: String,
    },
    List {
        project_id: String,
    },
    Update {
        request_id: String,
        request_hash: [u8; 32],
        id: String,
        action: String,
        now: u64,
    },
    Claim {
        id: String,
        attempt: u32,
        now: u64,
    },
    Progress {
        record: YoutubeImportV1,
    },
    Complete {
        id: String,
        attempt: u32,
        source_id: String,
        inspection: Box<InspectedSource>,
        now: u64,
    },
    Recover {
        now: u64,
    },
}

#[allow(
    clippy::too_many_lines,
    reason = "serialized transactional import state machine"
)]
pub(super) fn execute(
    connection: &mut Connection,
    command: YoutubeCommand,
) -> Result<Vec<YoutubeImportV1>, StoreError> {
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let result = match command {
        YoutubeCommand::Read { id } => vec![read(&tx, &id)?],
        YoutubeCommand::List { project_id } => list(&tx, &project_id)?,
        YoutubeCommand::Create {
            request_id,
            request_hash,
            mut record,
        } => {
            if let Some(bytes) = replay(&tx, &request_id, &request_hash)? {
                return receipt_record(&bytes);
            }
            if record.max_height == 0 {
                record.max_height = 1080;
            }
            let existing = tx
                .query_row(
                    "SELECT import_id FROM youtube_imports WHERE project_id=?1 AND video_id=?2",
                    params![record.project_id, record.video_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;
            let saved = if let Some(id) = existing {
                let saved = read(&tx, &id)?;
                if saved.max_height != record.max_height {
                    return Err(StoreError::ImportQualityConflict);
                }
                saved
            } else {
                tx.execute(
                    "INSERT INTO youtube_imports VALUES(?1,?2,?3,?4,1,?5,?6,?7)",
                    params![
                        record.import_id,
                        record.project_id,
                        record.video_id,
                        record.canonical_url,
                        record.state,
                        record.attempt,
                        record.encode_to_vec()
                    ],
                )?;
                record
            };
            remember(
                &tx,
                &request_id,
                &request_hash,
                &receipt(&request_id, &saved),
                saved.updated_unix_millis,
            )?;
            vec![saved]
        }
        YoutubeCommand::Update {
            request_id,
            request_hash,
            id,
            action,
            now,
        } => {
            if let Some(bytes) = replay(&tx, &request_id, &request_hash)? {
                return receipt_record(&bytes);
            }
            let mut record = read(&tx, &id)?;
            match action.as_str() {
                "cancel"
                    if matches!(
                        record.state.as_str(),
                        "queued" | "downloading" | "processing" | "registering"
                    ) =>
                {
                    "cancelled".clone_into(&mut record.state);
                    "cancelled".clone_into(&mut record.error_code);
                    "Import cancelled".clone_into(&mut record.error);
                }
                "retry"
                    if matches!(
                        record.state.as_str(),
                        "failed" | "cancelled" | "interrupted"
                    ) =>
                {
                    record.attempt = record.attempt.checked_add(1).ok_or(StoreError::Conflict)?;
                    "queued".clone_into(&mut record.state);
                    record.downloaded_bytes = 0;
                    record.total_bytes = None;
                    record.error.clear();
                    record.error_code.clear();
                }
                _ => return Err(StoreError::Conflict),
            }
            record.updated_unix_millis = now;
            save(&tx, &record)?;
            remember(
                &tx,
                &request_id,
                &request_hash,
                &receipt(&request_id, &record),
                now,
            )?;
            vec![record]
        }
        YoutubeCommand::Claim { id, attempt, now } => {
            let mut record = read(&tx, &id)?;
            if record.state != "queued" || record.attempt != attempt {
                return Err(StoreError::Conflict);
            }
            "downloading".clone_into(&mut record.state);
            record.updated_unix_millis = now;
            save(&tx, &record)?;
            vec![record]
        }
        YoutubeCommand::Progress { record } => {
            let saved = read(&tx, &record.import_id)?;
            if saved.attempt != record.attempt
                || !matches!(
                    saved.state.as_str(),
                    "downloading" | "processing" | "registering"
                )
            {
                return Err(StoreError::Conflict);
            }
            if !matches!(
                record.state.as_str(),
                "downloading" | "processing" | "registering" | "failed" | "interrupted"
            ) {
                return Err(StoreError::Conflict);
            }
            save(&tx, &record)?;
            vec![record]
        }
        YoutubeCommand::Complete {
            id,
            attempt,
            source_id,
            inspection,
            now,
        } => {
            let mut record = read(&tx, &id)?;
            if record.attempt != attempt || record.state != "registering" {
                return Err(StoreError::Conflict);
            }
            // One commit: cancellation cannot race source insertion, and a
            // crash cannot leave a registered source disconnected from its import.
            source_store::insert_inspected_source(
                &tx,
                &record.project_id,
                &source_id,
                &inspection,
                now,
            )?;
            record.source_id = source_id;
            "completed".clone_into(&mut record.state);
            record.updated_unix_millis = now;
            save(&tx, &record)?;
            vec![record]
        }
        YoutubeCommand::Recover { now } => {
            for mut record in list(&tx, "")? {
                if matches!(
                    record.state.as_str(),
                    "queued" | "downloading" | "processing" | "registering"
                ) {
                    "interrupted".clone_into(&mut record.state);
                    "interrupted".clone_into(&mut record.error_code);
                    "The app stopped during this import. Retry when you are ready."
                        .clone_into(&mut record.error);
                    record.updated_unix_millis = now;
                    save(&tx, &record)?;
                }
            }
            Vec::new()
        }
    };
    tx.commit()?;
    Ok(result)
}

fn save(connection: &Connection, record: &YoutubeImportV1) -> Result<(), StoreError> {
    connection.execute(
        "UPDATE youtube_imports SET state=?1,attempt=?2,record=?3 WHERE import_id=?4",
        params![
            record.state,
            record.attempt,
            record.encode_to_vec(),
            record.import_id
        ],
    )?;
    if !record.title.is_empty() {
        // The shell creates this precise placeholder before metadata exists.
        // A name the user supplied is never replaced by remote metadata.
        connection.execute(
            "UPDATE projects SET name=?1 WHERE project_id=?2 AND name=?3",
            params![
                record.title,
                record.project_id,
                format!("YouTube · {}", record.video_id)
            ],
        )?;
    }
    Ok(())
}
fn read(connection: &Connection, id: &str) -> Result<YoutubeImportV1, StoreError> {
    let bytes: Vec<u8> = connection
        .query_row(
            "SELECT record FROM youtube_imports WHERE import_id=?1",
            [id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or(StoreError::NotFound)?;
    let mut record = YoutubeImportV1::decode(bytes.as_slice())
        .map_err(|_| StoreError::InvalidData("invalid saved YouTube import"))?;
    if record.max_height == 0 {
        record.max_height = 1080;
    }
    Ok(record)
}
fn list(connection: &Connection, project: &str) -> Result<Vec<YoutubeImportV1>, StoreError> {
    let mut query = connection.prepare(
        "SELECT import_id FROM youtube_imports WHERE ?1='' OR project_id=?1 ORDER BY rowid DESC",
    )?;
    let ids = query
        .query_map([project], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    ids.iter().map(|id| read(connection, id)).collect()
}
fn receipt(id: &str, record: &YoutubeImportV1) -> Vec<u8> {
    Response {
        request_id: id.to_owned(),
        body: Some(response::Body::YoutubeImport(YoutubeImportResponse {
            record: Some(record.clone()),
        })),
    }
    .encode_to_vec()
}
fn receipt_record(bytes: &[u8]) -> Result<Vec<YoutubeImportV1>, StoreError> {
    match Response::decode(bytes).ok().and_then(|reply| reply.body) {
        Some(response::Body::YoutubeImport(reply)) => reply
            .record
            .map(|record| vec![record])
            .ok_or(StoreError::InvalidData("missing import receipt")),
        _ => Err(StoreError::InvalidData("invalid import receipt")),
    }
}

#[cfg(test)]
mod tests;
