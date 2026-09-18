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
//!
//! A directed document also knows which clip it is. The source it was cut from
//! and the candidate it was built for sit beside it as columns, so "the
//! document for this clip" is a lookup rather than a guess at the newest — and
//! directing the same clip again finds that document and hands it back, edits
//! included, instead of building a second one nobody asked for.

use clipmill_contracts::proto::ipc::v1::{
    ApplyEditCommandResponse, CreateEditDocResponse, DirectClipResponse, EditDoc, Response,
    response,
};
use clipmill_core::{EditDocId, ProjectId};
use clipmill_edit_ir::{EditCommand, EditDocument};
use prost::Message;
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use super::{Decision, StoreError, decision_store, remember, replay};

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

/// Which clip a document is: the source it was cut from and the candidate it
/// was built for.
///
/// Nullable, because a document handed in whole through `CreateEditDoc` names
/// no candidate and never will. The backfill recovers what it can for rows
/// that predate the columns: the candidate is in the document's own rationale,
/// and the source is the registered recording whose fingerprint the first
/// segment names. A row the backfill cannot place stays null and is listed
/// as a document with no clip, which is what it is.
pub(super) const CREATE_V10_TABLES: &str = "
    ALTER TABLE edit_docs ADD COLUMN source_id TEXT;
    ALTER TABLE edit_docs ADD COLUMN candidate_id TEXT;

    UPDATE edit_docs
       SET candidate_id = json_extract(document, '$.rationale.candidate_id')
     WHERE candidate_id IS NULL
       AND json_type(document, '$.rationale.candidate_id') = 'text';

    UPDATE edit_docs
       SET source_id = (
           SELECT s.source_id FROM sources s
            WHERE s.project_id = edit_docs.project_id
              AND s.source_fingerprint =
                  json_extract(edit_docs.document, '$.video.segments[0].source_fingerprint')
            ORDER BY s.created_unix_millis ASC, s.source_id ASC
            LIMIT 1
       )
     WHERE source_id IS NULL;

    CREATE INDEX edit_docs_by_clip
        ON edit_docs(project_id, source_id, candidate_id, created_unix_millis DESC, doc_id DESC)
        WHERE candidate_id IS NOT NULL;
";

/// The analysis run a document was cut from.
///
/// Nullable: a document handed in whole names no run, and one directed
/// before runs were recorded cannot be told which. Nothing is backfilled —
/// the document does not say, and a guess at the newest run would be exactly
/// the substitution the column exists to remove.
pub(super) const CREATE_V11_TABLES: &str = "
    ALTER TABLE edit_docs ADD COLUMN job_id TEXT;
";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EditDocRecord {
    pub doc_id: String,
    pub project_id: String,
    /// The source the document was cut from and the candidate it was built
    /// for; both `None` for a document handed in whole rather than directed.
    pub source_id: Option<String>,
    pub candidate_id: Option<String>,
    /// The analysis run it was cut from, when the director knew it.
    pub job_id: Option<String>,
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
            source_id: value.source_id.unwrap_or_default(),
            candidate_id: value.candidate_id.unwrap_or_default(),
            job_id: value.job_id.unwrap_or_default(),
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

/// Which clip a hand-created document is, when the caller says.
///
/// Optional as a whole and in parts: a document may name a source and no
/// candidate (a clip nobody ranked), or a candidate and no run.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct DocumentOrigin {
    pub source: Option<String>,
    pub candidate: Option<String>,
    pub run: Option<String>,
}

pub(super) fn create_edit_doc(
    connection: &mut Connection,
    request_id: &str,
    request_hash: &[u8; 32],
    project_id: &str,
    document_json: &str,
    origin: &DocumentOrigin,
    now: u64,
) -> Result<Vec<u8>, StoreError> {
    project_id
        .parse::<ProjectId>()
        .map_err(|_| StoreError::InvalidData("edit document project id is invalid"))?;
    if origin.candidate.is_some() && origin.source.is_none() {
        return Err(StoreError::InvalidData(
            "a document naming a candidate must name the source it was cut from",
        ));
    }
    let mut document = if document_json.trim().is_empty() {
        EditDocument::default()
    } else {
        EditDocument::from_canonical_json(document_json.as_bytes())
            .map_err(|_| StoreError::InvalidData("initial edit document is not valid"))?
    };
    // Every stored document names its words, so a correction can be
    // addressed to one; a document handed in without ids is given them here,
    // once, before anything is logged against it.
    document.assign_word_ids();
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
            created_unix_millis, updated_unix_millis, source_id, candidate_id, job_id
         ) VALUES (?1, ?2, 0, ?3, ?3, ?4, ?4, ?5, ?6, ?7)",
        params![
            doc_id,
            project_id,
            canonical,
            now_sql,
            origin.source,
            origin.candidate,
            origin.run
        ],
    )?;
    let record = EditDocRecord {
        doc_id,
        project_id: project_id.to_owned(),
        source_id: origin.source.clone(),
        candidate_id: origin.candidate.clone(),
        job_id: origin.run.clone(),
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

/// Which clip a directed document is for: the project, the recording in it,
/// and the candidate the director built from — and the run that minted the
/// candidate, when the caller named one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClipIdentity {
    pub project: String,
    pub source: String,
    pub candidate: String,
    pub run: Option<String>,
}

/// What `direct_edit_doc` is asked to do beyond storing a document.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DirectOptions {
    /// Build a second document beside an existing one rather than reopening
    /// it.
    pub variation: bool,
    /// Record the approval in the same transaction.
    pub approve: bool,
}

/// What came back: the document, and whether it already existed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Directed {
    pub record: EditDocRecord,
    pub reopened: bool,
}

/// The document a clip already has, reopened — or nothing, and no write.
///
/// The half of directing that needs no evidence. A clip's saved edit is the
/// answer to directing it again whether or not the analysis it was cut from
/// can still be loaded — a re-analysis may have renumbered the candidates, a
/// stage may fail to verify — so the service asks this first and assembles
/// only when there is nothing to reopen. The approval, when asked for, lands
/// in the same transaction; a retry of a lost reply is the same reply.
pub(super) fn reopen_edit_doc(
    connection: &mut Connection,
    request_id: &str,
    request_hash: &[u8; 32],
    identity: &ClipIdentity,
    approve: bool,
    now: u64,
) -> Result<Option<Vec<u8>>, StoreError> {
    identity
        .project
        .parse::<ProjectId>()
        .map_err(|_| StoreError::InvalidData("edit document project id is invalid"))?;
    if identity.source.is_empty() || identity.candidate.is_empty() {
        return Err(StoreError::InvalidData(
            "a directed document names its source and its candidate",
        ));
    }
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(response) = replay(&transaction, request_id, request_hash)? {
        transaction.commit()?;
        return Ok(Some(response));
    }
    let Some(record) = newest_for_clip(&transaction, identity)? else {
        // Nothing to reopen and nothing written: the transaction held no
        // change, and the request id stays free for the creation to claim.
        transaction.commit()?;
        return Ok(None);
    };
    if approve {
        decision_store::set(
            &transaction,
            &identity.project,
            &identity.source,
            &identity.candidate,
            Decision::Approved,
            now,
        )?;
    }
    let directed = Directed {
        record,
        reopened: true,
    };
    let response = Response {
        request_id: request_id.to_owned(),
        body: Some(response::Body::DirectClip(direct_response(&directed)?)),
    }
    .encode_to_vec();
    remember(&transaction, request_id, request_hash, &response, now)?;
    transaction.commit()?;
    Ok(Some(response))
}

/// The document for a clip: found if the clip has one, created if not.
///
/// One transaction does three things that must not be separable. It looks for
/// a document this candidate already has and, unless a variation was asked
/// for, hands that back untouched — the edits somebody made to it are the
/// reason it is the answer. Otherwise it stores the document the director
/// assembled. And if the caller is approving, it records the decision in the
/// same write, so a clip is never approved without a document to open and
/// never has a document without the approval that made it.
///
/// The reply is stored under the request id like every mutation, so a retry
/// after a lost response gets the same bytes rather than a second document.
pub(super) fn direct_edit_doc(
    connection: &mut Connection,
    request_id: &str,
    request_hash: &[u8; 32],
    identity: &ClipIdentity,
    document_json: &str,
    options: DirectOptions,
    now: u64,
) -> Result<Vec<u8>, StoreError> {
    identity
        .project
        .parse::<ProjectId>()
        .map_err(|_| StoreError::InvalidData("edit document project id is invalid"))?;
    if identity.source.is_empty() || identity.candidate.is_empty() {
        return Err(StoreError::InvalidData(
            "a directed document names its source and its candidate",
        ));
    }
    let mut document =
        EditDocument::from_canonical_json(document_json.as_bytes()).map_err(|error| {
            // The director's own document, refused by the document's own
            // rules: the reason belongs in the log, since the reply cannot
            // carry it and nobody can act on "not valid".
            tracing::warn!(%error, "directed edit document refused");
            StoreError::InvalidData("directed edit document is not valid")
        })?;
    document.assign_word_ids();
    let canonical = canonical_document(&document)?;

    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(response) = replay(&transaction, request_id, request_hash)? {
        transaction.commit()?;
        return Ok(response);
    }
    let project_exists: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM projects WHERE project_id = ?1 AND is_system = 0)",
        [&identity.project],
        |row| row.get(0),
    )?;
    if !project_exists {
        return Err(StoreError::NotFound);
    }

    let existing = if options.variation {
        None
    } else {
        newest_for_clip(&transaction, identity)?
    };
    let directed = if let Some(record) = existing {
        Directed {
            record,
            reopened: true,
        }
    } else {
        let doc_id = EditDocId::new().to_string();
        let now_sql = sqlite_u64(now)?;
        transaction.execute(
            "INSERT INTO edit_docs(
                doc_id, project_id, revision, initial_document, document,
                created_unix_millis, updated_unix_millis, source_id, candidate_id, job_id
             ) VALUES (?1, ?2, 0, ?3, ?3, ?4, ?4, ?5, ?6, ?7)",
            params![
                doc_id,
                identity.project,
                canonical,
                now_sql,
                identity.source,
                identity.candidate,
                identity.run
            ],
        )?;
        Directed {
            record: EditDocRecord {
                doc_id,
                project_id: identity.project.clone(),
                source_id: Some(identity.source.clone()),
                candidate_id: Some(identity.candidate.clone()),
                job_id: identity.run.clone(),
                revision: 0,
                document_json: canonical,
                created_unix_millis: now,
                updated_unix_millis: now,
            },
            reopened: false,
        }
    };
    if options.approve {
        decision_store::set(
            &transaction,
            &identity.project,
            &identity.source,
            &identity.candidate,
            Decision::Approved,
            now,
        )?;
    }

    let response = Response {
        request_id: request_id.to_owned(),
        body: Some(response::Body::DirectClip(direct_response(&directed)?)),
    }
    .encode_to_vec();
    remember(&transaction, request_id, request_hash, &response, now)?;
    transaction.commit()?;
    Ok(response)
}

/// The reply for a directed document, read off the document itself.
///
/// The cut and the decisions come from the stored document rather than from
/// what the director assembled this time, because for a reopened document the
/// two differ — a trim moved the segment, and the segment is what the caller
/// is about to open.
fn direct_response(directed: &Directed) -> Result<DirectClipResponse, StoreError> {
    let document = EditDocument::from_canonical_json(directed.record.document_json.as_bytes())
        .map_err(|_| StoreError::InvalidData("stored edit document is not valid"))?;
    let (start_ticks, end_ticks) = document
        .video
        .segments
        .first()
        .zip(document.video.segments.last())
        .map(|(first, last)| (first.in_ticks, last.out_ticks))
        .unwrap_or_default();
    Ok(DirectClipResponse {
        doc: Some(directed.record.clone().into()),
        start_ticks: u64::try_from(start_ticks).unwrap_or(0),
        end_ticks: u64::try_from(end_ticks).unwrap_or(0),
        decisions: document
            .rationale
            .map(|rationale| rationale.decisions)
            .unwrap_or_default(),
        reopened: directed.reopened,
    })
}

/// The newest document a clip has, if it has one.
///
/// Newest by creation, because when a clip has several — an approval and a
/// variation taken afterwards — the one somebody made last is the one they
/// were last working on.
fn newest_for_clip(
    transaction: &Transaction<'_>,
    identity: &ClipIdentity,
) -> Result<Option<EditDocRecord>, StoreError> {
    transaction
        .query_row(
            "SELECT doc_id, project_id, revision, document,
                    created_unix_millis, updated_unix_millis, source_id, candidate_id, job_id
               FROM edit_docs
              WHERE project_id = ?1 AND source_id = ?2 AND candidate_id = ?3
              ORDER BY created_unix_millis DESC, doc_id DESC
              LIMIT 1",
            params![identity.project, identity.source, identity.candidate],
            record_from_row,
        )
        .optional()
        .map_err(StoreError::from)
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
    let Some(current) = transaction
        .query_row(
            "SELECT doc_id, project_id, revision, document,
                    created_unix_millis, updated_unix_millis, source_id, candidate_id, job_id
               FROM edit_docs WHERE doc_id = ?1",
            [doc_id],
            record_from_row,
        )
        .optional()?
    else {
        return Err(StoreError::NotFound);
    };
    if current.revision != expected_revision {
        // The client edited a document it had not seen the latest state of.
        // Rebasing silently would discard whichever edit lost the race.
        return Err(StoreError::Conflict);
    }
    let revision = current.revision;
    let mut document = EditDocument::from_canonical_json(current.document_json.as_bytes())
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
        revision: next_revision,
        document_json: canonical,
        created_unix_millis: now,
        updated_unix_millis: now,
        ..current
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
        "SELECT doc_id, project_id, revision, document,
                created_unix_millis, updated_unix_millis, source_id, candidate_id, job_id
           FROM edit_docs
          WHERE project_id = ?1
          ORDER BY created_unix_millis ASC, doc_id ASC",
    )?;
    let rows = statement.query_map([project_id], record_from_row)?;
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
                    created_unix_millis, updated_unix_millis, source_id, candidate_id, job_id
             FROM edit_docs WHERE doc_id = ?1",
            [doc_id],
            record_from_row,
        )
        .optional()?
        .ok_or(StoreError::NotFound)
}

/// One row of `edit_docs`, in the column order every read here uses.
fn record_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<EditDocRecord> {
    Ok(EditDocRecord {
        doc_id: row.get(0)?,
        project_id: row.get(1)?,
        revision: sql_u64(row, 2)?,
        document_json: row.get(3)?,
        created_unix_millis: sql_u64(row, 4)?,
        updated_unix_millis: sql_u64(row, 5)?,
        source_id: row.get(6)?,
        candidate_id: row.get(7)?,
        job_id: row.get(8)?,
    })
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

/// Give every stored document's words their identity.
///
/// The log is the truth and the live document is derived from it, so the
/// migration is done the way the log is checked: the initial document is
/// given ids and every logged command is replayed over it, which yields a
/// live document whose corrections landed in both presentations — a
/// correction logged before ids existed reached one grouping only, and the
/// replay is where it reaches the other. A log that no longer replays (none
/// should) falls back to giving the live document ids as it stands, so no
/// document is left without them. Runs inside the schema transaction, so a
/// store is either wholly migrated or not at all.
pub(super) fn migrate_word_ids(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    let mut statement = transaction.prepare("SELECT doc_id FROM edit_docs ORDER BY doc_id")?;
    let doc_ids = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    drop(statement);
    for doc_id in doc_ids {
        let (initial_json, entries) = get_edit_log(transaction, &doc_id)?;
        let live_json: String = transaction.query_row(
            "SELECT document FROM edit_docs WHERE doc_id = ?1",
            [&doc_id],
            |row| row.get(0),
        )?;
        let mut initial = EditDocument::from_canonical_json(initial_json.as_bytes())
            .map_err(|_| StoreError::InvalidData("stored initial edit document is not valid"))?;
        let mut live = EditDocument::from_canonical_json(live_json.as_bytes())
            .map_err(|_| StoreError::InvalidData("stored edit document is not valid"))?;
        if !initial.assign_word_ids() && live.words_are_identified() {
            continue;
        }
        let migrated_live = replay_log(&initial, &entries).unwrap_or_else(|| {
            live.assign_word_ids();
            live
        });
        transaction.execute(
            "UPDATE edit_docs SET initial_document = ?1, document = ?2 WHERE doc_id = ?3",
            params![
                canonical_document(&initial)?,
                canonical_document(&migrated_live)?,
                doc_id
            ],
        )?;
    }
    Ok(())
}

/// The log applied over a document, or nothing if any step refuses.
fn replay_log(initial: &EditDocument, entries: &[EditCommandRecord]) -> Option<EditDocument> {
    let mut document = initial.clone();
    for entry in entries {
        let command = EditCommand::from_canonical_json(entry.command_json.as_bytes()).ok()?;
        command.apply(&mut document).ok()?;
    }
    Some(document)
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
