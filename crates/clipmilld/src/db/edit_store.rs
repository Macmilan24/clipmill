//! Edit documents and their command logs.
//!
//! Both live in SQLite because they are project state: small, mutable, and
//! authored by the user (book ch. 10's two-lifecycle rule). The immutable
//! artifact store holds the *snapshots* a render consumes, never the document
//! being edited.
//!
//! A command is applied and logged in one transaction against the single
//! writer, so an acknowledged edit is durable by the time the caller sees it
//! and the log can never describe a document that was never reached.

use clipmill_contracts::proto::ipc::v1::{
    ApplyEditCommandResponse, CreateEditDocResponse, EditDoc, Response, response,
};
use clipmill_core::{EditDocId, ProjectId};
use clipmill_edit_ir::{EditCommand, EditDocument};
use prost::Message;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

use super::{StoreError, remember, replay};

pub(super) const CREATE_V6_TABLES: &str = "
    CREATE TABLE edit_docs (
        doc_id TEXT PRIMARY KEY
            CHECK(length(doc_id) = 30 AND substr(doc_id, 1, 4) = 'edt_'),
        project_id TEXT NOT NULL REFERENCES projects(project_id) ON DELETE CASCADE,
        revision INTEGER NOT NULL CHECK(revision >= 0),
        initial_document TEXT NOT NULL CHECK(length(initial_document) > 0),
        document TEXT NOT NULL CHECK(length(document) > 0),
        created_unix_millis INTEGER NOT NULL CHECK(created_unix_millis >= 0),
        updated_unix_millis INTEGER NOT NULL CHECK(updated_unix_millis >= 0)
    ) STRICT;

    CREATE INDEX edit_docs_by_project_created
        ON edit_docs(project_id, created_unix_millis DESC, doc_id DESC);

    CREATE TABLE edit_commands (
        doc_id TEXT NOT NULL REFERENCES edit_docs(doc_id) ON DELETE CASCADE,
        revision INTEGER NOT NULL CHECK(revision >= 1),
        command TEXT NOT NULL CHECK(length(command) > 0),
        inverse TEXT NOT NULL CHECK(length(inverse) > 0),
        applied_unix_millis INTEGER NOT NULL CHECK(applied_unix_millis >= 0),
        PRIMARY KEY(doc_id, revision)
    ) STRICT, WITHOUT ROWID;
";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EditDocRecord {
    pub doc_id: String,
    pub project_id: String,
    pub revision: u64,
    pub document_json: String,
    pub created_unix_millis: u64,
    pub updated_unix_millis: u64,
}

impl From<EditDocRecord> for EditDoc {
    fn from(value: EditDocRecord) -> Self {
        Self {
            doc_id: value.doc_id,
            project_id: value.project_id,
            revision: value.revision,
            document_json: value.document_json,
            created_unix_millis: value.created_unix_millis,
            updated_unix_millis: value.updated_unix_millis,
        }
    }
}

/// One logged step: the command as applied and the command that undoes it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EditCommandRecord {
    pub revision: u64,
    pub command_json: String,
    pub inverse_json: String,
}

pub(super) fn create_edit_doc(
    connection: &mut Connection,
    request_id: &str,
    request_hash: &[u8; 32],
    project_id: &str,
    document_json: &str,
    now: u64,
) -> Result<Vec<u8>, StoreError> {
    project_id
        .parse::<ProjectId>()
        .map_err(|_| StoreError::InvalidData("edit document project id is invalid"))?;
    let document = if document_json.trim().is_empty() {
        EditDocument::default()
    } else {
        EditDocument::from_canonical_json(document_json.as_bytes())
            .map_err(|_| StoreError::InvalidData("initial edit document is not valid"))?
    };
    let canonical = canonical_document(&document)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(response) = replay(&transaction, request_id, request_hash)? {
        transaction.commit()?;
        return Ok(response);
    }
    let project_exists: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM projects WHERE project_id = ?1 AND is_system = 0)",
        [project_id],
        |row| row.get(0),
    )?;
    if !project_exists {
        return Err(StoreError::NotFound);
    }
    let doc_id = EditDocId::new().to_string();
    let now_sql = sqlite_u64(now)?;
    transaction.execute(
        "INSERT INTO edit_docs(
            doc_id, project_id, revision, initial_document, document,
            created_unix_millis, updated_unix_millis
         ) VALUES (?1, ?2, 0, ?3, ?3, ?4, ?4)",
        params![doc_id, project_id, canonical, now_sql],
    )?;
    let record = EditDocRecord {
        doc_id,
        project_id: project_id.to_owned(),
        revision: 0,
        document_json: canonical,
        created_unix_millis: now,
        updated_unix_millis: now,
    };
    let response = Response {
        request_id: request_id.to_owned(),
        body: Some(response::Body::CreateEditDoc(CreateEditDocResponse {
            doc: Some(record.into()),
        })),
    }
    .encode_to_vec();
    remember(&transaction, request_id, request_hash, &response, now)?;
    transaction.commit()?;
    Ok(response)
}

pub(super) fn apply_edit_command(
    connection: &mut Connection,
    request_id: &str,
    request_hash: &[u8; 32],
    doc_id: &str,
    expected_revision: u64,
    command_json: &str,
    now: u64,
) -> Result<Vec<u8>, StoreError> {
    let command = EditCommand::from_canonical_json(command_json.as_bytes())
        .map_err(|_| StoreError::InvalidData("edit command is not valid"))?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(response) = replay(&transaction, request_id, request_hash)? {
        transaction.commit()?;
        return Ok(response);
    }
    let row: Option<(String, i64, String)> = transaction
        .query_row(
            "SELECT project_id, revision, document FROM edit_docs WHERE doc_id = ?1",
            [doc_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;
    let Some((project_id, revision, document_json)) = row else {
        return Err(StoreError::NotFound);
    };
    let revision = u64_from_i64(revision)?;
    if revision != expected_revision {
        // The client edited a document it had not seen the latest state of.
        // Rebasing silently would discard whichever edit lost the race.
        return Err(StoreError::Conflict);
    }
    let mut document = EditDocument::from_canonical_json(document_json.as_bytes())
        .map_err(|_| StoreError::InvalidData("stored edit document is not valid"))?;
    let inverse = command
        .apply(&mut document)
        .map_err(|_| StoreError::InvalidData("edit command cannot apply to this document"))?;
    let canonical = canonical_document(&document)?;
    let inverse_json = String::from_utf8(
        inverse
            .to_canonical_json()
            .map_err(|_| StoreError::InvalidData("inverse command is not serializable"))?,
    )
    .map_err(|_| StoreError::InvalidData("inverse command is not UTF-8"))?;
    let command_canonical = String::from_utf8(
        command
            .to_canonical_json()
            .map_err(|_| StoreError::InvalidData("edit command is not serializable"))?,
    )
    .map_err(|_| StoreError::InvalidData("edit command is not UTF-8"))?;
    let next_revision = revision
        .checked_add(1)
        .ok_or(StoreError::InvalidData("edit revision overflow"))?;
    let now_sql = sqlite_u64(now)?;
    transaction.execute(
        "INSERT INTO edit_commands(doc_id, revision, command, inverse, applied_unix_millis)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            doc_id,
            sqlite_u64(next_revision)?,
            command_canonical,
            inverse_json,
            now_sql
        ],
    )?;
    transaction.execute(
        "UPDATE edit_docs SET revision = ?1, document = ?2, updated_unix_millis = ?3
         WHERE doc_id = ?4",
        params![sqlite_u64(next_revision)?, canonical, now_sql, doc_id],
    )?;
    let record = EditDocRecord {
        doc_id: doc_id.to_owned(),
        project_id,
        revision: next_revision,
        document_json: canonical,
        created_unix_millis: now,
        updated_unix_millis: now,
    };
    let response = Response {
        request_id: request_id.to_owned(),
        body: Some(response::Body::ApplyEditCommand(ApplyEditCommandResponse {
            doc: Some(record.into()),
            inverse_command_json: inverse_json,
        })),
    }
    .encode_to_vec();
    remember(&transaction, request_id, request_hash, &response, now)?;
    transaction.commit()?;
    Ok(response)
}

/// Every document in a project, oldest first.
///
/// Ordered by creation so "the newest" is the last one, which is what an editor
/// opening after approving a clip is asking for. The index this reads was
/// created with the table for exactly this query.
pub(super) fn list_edit_docs(
    connection: &Connection,
    project_id: &str,
) -> Result<Vec<EditDocRecord>, StoreError> {
    let mut statement = connection.prepare(
        "SELECT doc_id, project_id, revision, document, created_unix_millis, updated_unix_millis
           FROM edit_docs
          WHERE project_id = ?1
          ORDER BY created_unix_millis ASC, doc_id ASC",
    )?;
    let rows = statement.query_map([project_id], |row| {
        Ok(EditDocRecord {
            doc_id: row.get(0)?,
            project_id: row.get(1)?,
            revision: row.get::<_, i64>(2)?.try_into().unwrap_or(0),
            document_json: row.get(3)?,
            created_unix_millis: row.get::<_, i64>(4)?.try_into().unwrap_or(0),
            updated_unix_millis: row.get::<_, i64>(5)?.try_into().unwrap_or(0),
        })
    })?;
    let mut found = Vec::new();
    for row in rows {
        found.push(row?);
    }
    Ok(found)
}

pub(super) fn get_edit_doc(
    connection: &Connection,
    doc_id: &str,
) -> Result<EditDocRecord, StoreError> {
    connection
        .query_row(
            "SELECT doc_id, project_id, revision, document,
                    created_unix_millis, updated_unix_millis
             FROM edit_docs WHERE doc_id = ?1",
            [doc_id],
            |row| {
                Ok(EditDocRecord {
                    doc_id: row.get(0)?,
                    project_id: row.get(1)?,
                    revision: sql_u64(row, 2)?,
                    document_json: row.get(3)?,
                    created_unix_millis: sql_u64(row, 4)?,
                    updated_unix_millis: sql_u64(row, 5)?,
                })
            },
        )
        .optional()?
        .ok_or(StoreError::NotFound)
}

/// The document the log started from, plus every logged step. Replaying the
/// commands over the initial document must reproduce the live one exactly.
pub(super) fn get_edit_log(
    connection: &Connection,
    doc_id: &str,
) -> Result<(String, Vec<EditCommandRecord>), StoreError> {
    let initial: Option<String> = connection
        .query_row(
            "SELECT initial_document FROM edit_docs WHERE doc_id = ?1",
            [doc_id],
            |row| row.get(0),
        )
        .optional()?;
    let Some(initial) = initial else {
        return Err(StoreError::NotFound);
    };
    let mut statement = connection.prepare(
        "SELECT revision, command, inverse FROM edit_commands
         WHERE doc_id = ?1 ORDER BY revision",
    )?;
    let entries = statement
        .query_map([doc_id], |row| {
            Ok(EditCommandRecord {
                revision: sql_u64(row, 0)?,
                command_json: row.get(1)?,
                inverse_json: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok((initial, entries))
}

fn canonical_document(document: &EditDocument) -> Result<String, StoreError> {
    let bytes = document
        .to_canonical_json()
        .map_err(|_| StoreError::InvalidData("edit document is not serializable"))?;
    String::from_utf8(bytes).map_err(|_| StoreError::InvalidData("edit document is not UTF-8"))
}

fn sqlite_u64(value: u64) -> Result<i64, StoreError> {
    i64::try_from(value).map_err(|_| StoreError::InvalidData("value exceeds SQLite integer range"))
}

fn u64_from_i64(value: i64) -> Result<u64, StoreError> {
    u64::try_from(value).map_err(|_| StoreError::InvalidData("stored revision is negative"))
}

fn sql_u64(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<u64> {
    let value: i64 = row.get(index)?;
    u64::try_from(value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            index,
            rusqlite::types::Type::Integer,
            Box::new(error),
        )
    })
}
