//! Batch intents live beside edit documents in the single SQLite writer. A job
//! acceptance and its link can be replayed independently using a stable request
//! ID, so a crash between them cannot turn an item into a second export.
use super::{StoreError, remember, replay};
use clipmill_contracts::proto::ipc::v1::{
    ExportBatchItemV1, ExportBatchResponse, ExportBatchV1, ExportClipResponse, ExportRequestV1,
    Response, response,
};
use prost::Message;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

pub(super) const CREATE_V12_TABLES: &str = "
 CREATE TABLE export_batches (
   batch_id TEXT PRIMARY KEY, request_id TEXT NOT NULL UNIQUE,
   request_hash BLOB NOT NULL, created_unix_millis INTEGER NOT NULL
 ) STRICT;
 CREATE TABLE export_batch_items (
   batch_id TEXT NOT NULL REFERENCES export_batches(batch_id) ON DELETE CASCADE,
   item_index INTEGER NOT NULL, doc_id TEXT NOT NULL REFERENCES edit_docs(doc_id) ON DELETE CASCADE,
   request BLOB NOT NULL, state TEXT NOT NULL CHECK(state IN ('pending','queued','failed','cancelled')),
   attempt INTEGER NOT NULL DEFAULT 0, queued BLOB, error TEXT NOT NULL DEFAULT '',
   PRIMARY KEY(batch_id,item_index)
 ) STRICT, WITHOUT ROWID;
";

#[derive(Debug)]
pub(crate) enum BatchCommand {
    Create {
        request_id: String,
        request_hash: [u8; 32],
        batch: ExportBatchV1,
    },
    List,
    Link {
        batch_id: String,
        index: u32,
        attempt: u32,
        queued: Option<ExportClipResponse>,
        error: String,
    },
    Update {
        request_id: String,
        request_hash: [u8; 32],
        completed_unix_millis: u64,
        batch_id: String,
        index: u32,
        action: String,
    },
}

#[allow(
    clippy::too_many_lines,
    reason = "one serialized batch command dispatch with transactional branches"
)]
pub(super) fn execute(
    connection: &mut Connection,
    command: BatchCommand,
) -> Result<Vec<ExportBatchV1>, StoreError> {
    match command {
        BatchCommand::List => list(connection),
        BatchCommand::Create {
            request_id,
            request_hash,
            batch,
        } => {
            let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            if let Some(bytes) = replay(&tx, &request_id, &request_hash)? {
                return decode_receipt(&bytes);
            }
            let existing: Option<(String, Vec<u8>)> = tx
                .query_row(
                    "SELECT batch_id,request_hash FROM export_batches WHERE request_id=?1",
                    [&request_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()?;
            if let Some((id, hash)) = existing {
                if hash != request_hash {
                    return Err(StoreError::Conflict);
                }
                return Ok(vec![read(&tx, &id)?]);
            }
            tx.execute(
                "INSERT INTO export_batches VALUES (?1,?2,?3,?4)",
                params![
                    batch.batch_id,
                    request_id,
                    request_hash.as_slice(),
                    i64::try_from(batch.created_unix_millis).map_err(|_| {
                        StoreError::InvalidData("batch timestamp exceeds SQLite range")
                    })?
                ],
            )?;
            for item in &batch.items {
                let request = item
                    .request
                    .as_ref()
                    .ok_or(StoreError::InvalidData("batch item has no request"))?;
                tx.execute("INSERT INTO export_batch_items(batch_id,item_index,doc_id,request,state) VALUES(?1,?2,?3,?4,'pending')", params![batch.batch_id,item.index,request.doc_id,request.encode_to_vec()])?;
            }
            let saved = read(&tx, &batch.batch_id)?;
            remember(
                &tx,
                &request_id,
                &request_hash,
                &receipt(&request_id, &saved),
                batch.created_unix_millis,
            )?;
            tx.commit()?;
            Ok(vec![saved])
        }
        BatchCommand::Link {
            batch_id,
            index,
            attempt,
            queued,
            error,
        } => {
            let state = if queued.is_some() { "queued" } else { "failed" };
            // A cancelled item or a later explicit attempt cannot be overwritten
            // by a late completion from an older coordinator.
            connection.execute("UPDATE export_batch_items SET state=?1,queued=COALESCE(?2,queued),error=?3 WHERE batch_id=?4 AND item_index=?5 AND attempt=?6 AND state='pending'", params![state, queued.map(|q|q.encode_to_vec()),error,batch_id,index,attempt])?;
            Ok(vec![read(connection, &batch_id)?])
        }
        BatchCommand::Update {
            request_id,
            request_hash,
            completed_unix_millis,
            batch_id,
            index,
            action,
        } => {
            let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            if let Some(bytes) = replay(&tx, &request_id, &request_hash)? {
                return decode_receipt(&bytes);
            }
            let changed = match action.as_str() {
                "retry" => tx.execute("UPDATE export_batch_items SET state='pending',attempt=attempt+1,error='' WHERE batch_id=?1 AND item_index=?2 AND state IN ('failed','cancelled','queued')", params![batch_id,index])?,
                "cancel" => tx.execute("UPDATE export_batch_items SET state='cancelled',error='Cancelled by you' WHERE batch_id=?1 AND item_index=?2 AND state IN ('pending','queued','failed')", params![batch_id,index])?,
                _ => return Err(StoreError::InvalidData("unknown batch action")),
            };
            if changed == 0 {
                return Err(StoreError::Conflict);
            }
            let batch = read(&tx, &batch_id)?;
            let bytes = Response {
                request_id: request_id.clone(),
                body: Some(response::Body::ExportBatch(ExportBatchResponse {
                    batch: Some(batch.clone()),
                })),
            }
            .encode_to_vec();
            remember(
                &tx,
                &request_id,
                &request_hash,
                &bytes,
                completed_unix_millis,
            )?;
            tx.commit()?;
            Ok(vec![batch])
        }
    }
}

fn list(connection: &Connection) -> Result<Vec<ExportBatchV1>, StoreError> {
    let mut statement = connection.prepare(
        "SELECT batch_id FROM export_batches ORDER BY created_unix_millis DESC,batch_id DESC",
    )?;
    let ids = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    ids.iter().map(|id| read(connection, id)).collect()
}

fn read(connection: &Connection, id: &str) -> Result<ExportBatchV1, StoreError> {
    let created = connection
        .query_row(
            "SELECT created_unix_millis FROM export_batches WHERE batch_id=?1",
            [id],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .ok_or(StoreError::NotFound)?;
    let mut statement = connection.prepare("SELECT b.item_index,b.request,b.state,b.attempt,b.queued,b.error,d.project_id FROM export_batch_items b JOIN edit_docs d ON b.doc_id=d.doc_id WHERE b.batch_id=?1 ORDER BY b.item_index")?;
    let items = statement
        .query_map([id], |row| {
            Ok((
                row.get::<_, u32>(0)?,
                row.get::<_, Vec<u8>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, u32>(3)?,
                row.get::<_, Option<Vec<u8>>>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(
            |(index, request, state, attempt, queued, error, project_id)| {
                Ok(ExportBatchItemV1 {
                    index,
                    project_id,
                    request: Some(
                        ExportRequestV1::decode(request.as_slice())
                            .map_err(|_| StoreError::InvalidData("invalid saved export request"))?,
                    ),
                    state,
                    attempt,
                    queued: queued
                        .map(|bytes| {
                            ExportClipResponse::decode(bytes.as_slice()).map_err(|_| {
                                StoreError::InvalidData("invalid saved export receipt")
                            })
                        })
                        .transpose()?,
                    error,
                })
            },
        )
        .collect::<Result<Vec<_>, StoreError>>()?;
    Ok(ExportBatchV1 {
        batch_id: id.to_owned(),
        created_unix_millis: u64::try_from(created)
            .map_err(|_| StoreError::InvalidData("negative batch timestamp"))?,
        items,
    })
}

fn receipt(request_id: &str, batch: &ExportBatchV1) -> Vec<u8> {
    Response {
        request_id: request_id.to_owned(),
        body: Some(response::Body::ExportBatch(ExportBatchResponse {
            batch: Some(batch.clone()),
        })),
    }
    .encode_to_vec()
}
fn decode_receipt(bytes: &[u8]) -> Result<Vec<ExportBatchV1>, StoreError> {
    let response = Response::decode(bytes)
        .map_err(|_| StoreError::InvalidData("invalid saved batch receipt"))?;
    match response.body {
        Some(response::Body::ExportBatch(reply)) => Ok(vec![
            reply
                .batch
                .ok_or(StoreError::InvalidData("empty batch receipt"))?,
        ]),
        _ => Err(StoreError::Conflict),
    }
}
