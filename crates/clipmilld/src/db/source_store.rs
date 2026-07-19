use clipmill_contracts::proto::ipc::v1::{RegisterSourceResponse, Response, Source, response};
use clipmill_core::{ProjectId, SourceId};
use prost::Message;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

use super::{StoreError, remember, replay};
use crate::sources::{FileObservation, InspectedSource};

pub(super) const CREATE_V4_TABLES: &str = "
    CREATE TABLE sources (
        source_id TEXT PRIMARY KEY
            CHECK(length(source_id) = 30 AND substr(source_id, 1, 4) = 'src_'),
        project_id TEXT NOT NULL REFERENCES projects(project_id) ON DELETE CASCADE,
        source_fingerprint TEXT NOT NULL
            CHECK(
                length(source_fingerprint) = 71
                AND substr(source_fingerprint, 1, 7) = 'sha256:'
                AND substr(source_fingerprint, 8) NOT GLOB '*[^0-9a-f]*'
            ),
        source_map_json BLOB NOT NULL CHECK(length(source_map_json) > 0),
        source_map_artifact_id TEXT
            CHECK(
                source_map_artifact_id IS NULL OR (
                    length(source_map_artifact_id) = 71
                    AND substr(source_map_artifact_id, 1, 7) = 'sha256:'
                    AND substr(source_map_artifact_id, 8) NOT GLOB '*[^0-9a-f]*'
                )
            ),
        created_unix_millis INTEGER NOT NULL CHECK(created_unix_millis >= 0)
    ) STRICT;

    CREATE INDEX sources_by_project_created
        ON sources(project_id, created_unix_millis DESC, source_id DESC);

    CREATE TABLE source_file_observations (
        source_id TEXT PRIMARY KEY REFERENCES sources(source_id) ON DELETE CASCADE,
        absolute_path TEXT NOT NULL CHECK(length(absolute_path) > 0),
        byte_size INTEGER NOT NULL CHECK(byte_size >= 0),
        sample_sha256 TEXT NOT NULL
            CHECK(
                length(sample_sha256) = 71
                AND substr(sample_sha256, 1, 7) = 'sha256:'
                AND substr(sample_sha256, 8) NOT GLOB '*[^0-9a-f]*'
            ),
        device_id INTEGER NOT NULL CHECK(device_id >= 0),
        inode INTEGER NOT NULL CHECK(inode >= 0),
        modified_unix_nanos INTEGER NOT NULL CHECK(modified_unix_nanos >= 0)
    ) STRICT;

    CREATE INDEX source_observations_by_path
        ON source_file_observations(absolute_path, byte_size, sample_sha256);

    CREATE TABLE source_artifact_roots (
        source_id TEXT NOT NULL REFERENCES sources(source_id) ON DELETE CASCADE,
        artifact_id TEXT NOT NULL
            CHECK(
                length(artifact_id) = 71
                AND substr(artifact_id, 1, 7) = 'sha256:'
                AND substr(artifact_id, 8) NOT GLOB '*[^0-9a-f]*'
            ),
        PRIMARY KEY(source_id, artifact_id)
    ) STRICT, WITHOUT ROWID;

    CREATE INDEX source_artifact_roots_by_artifact
        ON source_artifact_roots(artifact_id);

    ALTER TABLE jobs ADD COLUMN source_id TEXT REFERENCES sources(source_id);
    CREATE INDEX jobs_by_source ON jobs(source_id) WHERE source_id IS NOT NULL;
";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SourceRecord {
    pub source_id: String,
    pub project_id: String,
    pub observation: FileObservation,
    pub source_fingerprint: String,
    pub source_map_json: Vec<u8>,
    pub source_map_artifact_id: String,
    pub created_unix_millis: u64,
}

impl From<SourceRecord> for Source {
    fn from(record: SourceRecord) -> Self {
        Self {
            source_id: record.source_id,
            project_id: record.project_id,
            absolute_path: record.observation.absolute_path,
            byte_size: record.observation.byte_size,
            sample_sha256: record.observation.sample_sha256,
            source_fingerprint: record.source_fingerprint,
            source_map_artifact_id: record.source_map_artifact_id,
            created_unix_millis: record.created_unix_millis,
        }
    }
}

pub(super) fn find_observation(
    connection: &Connection,
    project_id: &str,
    observation: &FileObservation,
) -> Result<Option<SourceRecord>, StoreError> {
    connection
        .query_row(
            "SELECT s.source_id, s.project_id, o.absolute_path, o.byte_size,
                    o.sample_sha256, o.device_id, o.inode, o.modified_unix_nanos,
                    s.source_fingerprint, s.source_map_json,
                    coalesce(s.source_map_artifact_id, ''), s.created_unix_millis
             FROM sources s
             JOIN source_file_observations o ON o.source_id = s.source_id
             WHERE s.project_id = ?1 AND o.absolute_path = ?2
               AND o.byte_size = ?3 AND o.sample_sha256 = ?4
             ORDER BY s.created_unix_millis DESC, s.source_id DESC
             LIMIT 1",
            params![
                project_id,
                observation.absolute_path,
                sqlite_u64(observation.byte_size, "source byte size")?,
                observation.sample_sha256,
            ],
            source_from_row,
        )
        .optional()
        .map_err(Into::into)
}

#[allow(clippy::too_many_lines)]
pub(super) fn register_source(
    connection: &mut Connection,
    request_id: &str,
    request_hash: &[u8; 32],
    project_id: &str,
    source_id: &str,
    inspection: &InspectedSource,
    created_unix_millis: u64,
) -> Result<Vec<u8>, StoreError> {
    source_id
        .parse::<SourceId>()
        .map_err(|_| StoreError::InvalidData("source id is invalid"))?;
    project_id
        .parse::<ProjectId>()
        .map_err(|_| StoreError::InvalidData("source project id is invalid"))?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(response) = replay(&transaction, request_id, request_hash)? {
        transaction.commit()?;
        return Ok(response);
    }
    let project_exists: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM projects WHERE project_id = ?1)",
        [project_id],
        |row| row.get(0),
    )?;
    if !project_exists {
        return Err(StoreError::NotFound);
    }
    let observation = &inspection.observation;
    let existing = transaction
        .query_row(
            "SELECT s.source_id, s.project_id, o.absolute_path, o.byte_size,
                    o.sample_sha256, o.device_id, o.inode, o.modified_unix_nanos,
                    s.source_fingerprint, s.source_map_json,
                    coalesce(s.source_map_artifact_id, ''), s.created_unix_millis
             FROM sources s JOIN source_file_observations o ON o.source_id = s.source_id
             WHERE s.project_id = ?1 AND o.absolute_path = ?2
               AND o.byte_size = ?3 AND o.sample_sha256 = ?4
             ORDER BY s.created_unix_millis DESC, s.source_id DESC
             LIMIT 1",
            params![
                project_id,
                observation.absolute_path,
                sqlite_u64(observation.byte_size, "source byte size")?,
                observation.sample_sha256,
            ],
            source_from_row,
        )
        .optional()?;
    if let Some(existing) = existing {
        let response = Response {
            request_id: request_id.to_owned(),
            body: Some(response::Body::RegisterSource(RegisterSourceResponse {
                source: Some(existing.into()),
                observation_cache_hit: true,
            })),
        }
        .encode_to_vec();
        remember(
            &transaction,
            request_id,
            request_hash,
            &response,
            created_unix_millis,
        )?;
        transaction.commit()?;
        return Ok(response);
    }
    let created = sqlite_u64(created_unix_millis, "source timestamp")?;
    transaction.execute(
        "INSERT INTO sources(
            source_id, project_id, source_fingerprint, source_map_json, created_unix_millis
         ) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            source_id,
            project_id,
            inspection.source_fingerprint,
            inspection.source_map_json,
            created,
        ],
    )?;
    transaction.execute(
        "INSERT INTO source_file_observations(
            source_id, absolute_path, byte_size, sample_sha256,
            device_id, inode, modified_unix_nanos
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            source_id,
            observation.absolute_path,
            sqlite_u64(observation.byte_size, "source byte size")?,
            observation.sample_sha256,
            sqlite_u64(observation.device_id, "source device id")?,
            sqlite_u64(observation.inode, "source inode")?,
            sqlite_u64(observation.modified_unix_nanos, "source modification time")?,
        ],
    )?;
    let source = get_source_tx(&transaction, source_id)?;
    let response = Response {
        request_id: request_id.to_owned(),
        body: Some(response::Body::RegisterSource(RegisterSourceResponse {
            source: Some(source.into()),
            observation_cache_hit: false,
        })),
    }
    .encode_to_vec();
    remember(
        &transaction,
        request_id,
        request_hash,
        &response,
        created_unix_millis,
    )?;
    transaction.commit()?;
    Ok(response)
}

pub(super) fn remember_observation_hit(
    connection: &mut Connection,
    request_id: &str,
    request_hash: &[u8; 32],
    source: &SourceRecord,
    completed_unix_millis: u64,
) -> Result<Vec<u8>, StoreError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(response) = replay(&transaction, request_id, request_hash)? {
        transaction.commit()?;
        return Ok(response);
    }
    let response = Response {
        request_id: request_id.to_owned(),
        body: Some(response::Body::RegisterSource(RegisterSourceResponse {
            source: Some(source.clone().into()),
            observation_cache_hit: true,
        })),
    }
    .encode_to_vec();
    remember(
        &transaction,
        request_id,
        request_hash,
        &response,
        completed_unix_millis,
    )?;
    transaction.commit()?;
    Ok(response)
}

pub(super) fn get_source(
    connection: &Connection,
    source_id: &str,
) -> Result<SourceRecord, StoreError> {
    source_id
        .parse::<SourceId>()
        .map_err(|_| StoreError::InvalidData("source id is invalid"))?;
    get_source_from(connection, source_id)
}

pub(super) fn list_sources(
    connection: &Connection,
    project_id: &str,
) -> Result<Vec<SourceRecord>, StoreError> {
    let exists: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM projects WHERE project_id = ?1)",
        [project_id],
        |row| row.get(0),
    )?;
    if !exists {
        return Err(StoreError::NotFound);
    }
    let mut statement = connection.prepare(
        "SELECT s.source_id, s.project_id, o.absolute_path, o.byte_size,
                o.sample_sha256, o.device_id, o.inode, o.modified_unix_nanos,
                s.source_fingerprint, s.source_map_json,
                coalesce(s.source_map_artifact_id, ''), s.created_unix_millis
         FROM sources s JOIN source_file_observations o ON o.source_id = s.source_id
         WHERE s.project_id = ?1
         ORDER BY s.created_unix_millis DESC, s.source_id DESC",
    )?;
    statement
        .query_map([project_id], source_from_row)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

fn get_source_from(connection: &Connection, source_id: &str) -> Result<SourceRecord, StoreError> {
    connection
        .query_row(
            "SELECT s.source_id, s.project_id, o.absolute_path, o.byte_size,
                    o.sample_sha256, o.device_id, o.inode, o.modified_unix_nanos,
                    s.source_fingerprint, s.source_map_json,
                    coalesce(s.source_map_artifact_id, ''), s.created_unix_millis
             FROM sources s JOIN source_file_observations o ON o.source_id = s.source_id
             WHERE s.source_id = ?1",
            [source_id],
            source_from_row,
        )
        .optional()?
        .ok_or(StoreError::NotFound)
}

fn get_source_tx(
    transaction: &rusqlite::Transaction<'_>,
    source_id: &str,
) -> Result<SourceRecord, StoreError> {
    transaction
        .query_row(
            "SELECT s.source_id, s.project_id, o.absolute_path, o.byte_size,
                    o.sample_sha256, o.device_id, o.inode, o.modified_unix_nanos,
                    s.source_fingerprint, s.source_map_json,
                    coalesce(s.source_map_artifact_id, ''), s.created_unix_millis
             FROM sources s JOIN source_file_observations o ON o.source_id = s.source_id
             WHERE s.source_id = ?1",
            [source_id],
            source_from_row,
        )
        .optional()?
        .ok_or(StoreError::NotFound)
}

fn source_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SourceRecord> {
    Ok(SourceRecord {
        source_id: row.get(0)?,
        project_id: row.get(1)?,
        observation: FileObservation {
            absolute_path: row.get(2)?,
            byte_size: sqlite_row_u64(row, 3)?,
            sample_sha256: row.get(4)?,
            device_id: sqlite_row_u64(row, 5)?,
            inode: sqlite_row_u64(row, 6)?,
            modified_unix_nanos: sqlite_row_u64(row, 7)?,
        },
        source_fingerprint: row.get(8)?,
        source_map_json: row.get(9)?,
        source_map_artifact_id: row.get(10)?,
        created_unix_millis: sqlite_row_u64(row, 11)?,
    })
}

fn sqlite_u64(value: u64, label: &'static str) -> Result<i64, StoreError> {
    i64::try_from(value).map_err(|_| StoreError::InvalidData(label))
}

fn sqlite_row_u64(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<u64> {
    let value = row.get::<_, i64>(index)?;
    u64::try_from(value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            index,
            rusqlite::types::Type::Integer,
            Box::new(error),
        )
    })
}
