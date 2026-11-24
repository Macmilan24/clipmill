//! What a person decided about a clip, kept where it survives a kill.
//!
//! Rejecting a clip is work. It is small work, and a user does it a dozen times
//! in a session while barely noticing, which is exactly why losing it is worse
//! than losing something large — nobody remembers what they rejected, so nobody
//! can redo it. The decisions therefore live in the same durable store as
//! projects and jobs rather than in the renderer's memory, and the gate proves
//! it by killing the daemon between the decision and the read.
//!
//! A decision is keyed by candidate rather than by clip, and that matters. The
//! candidate id survives a re-run of ranking over the same evidence, so a user
//! who re-analyzes a recording does not find their rejections quietly reset.
//! What it does not survive is re-analysis that changes the evidence, and that
//! is correct: those are different candidates, whatever they are called.
//!
//! There is exactly one row per candidate. Changing your mind replaces the
//! decision rather than appending to it, because the question "what did I decide
//! about this clip" has one answer and a history nobody asked for is a table
//! that grows without a reader. Taking a decision back removes the row: an
//! undecided clip is one nobody has an answer for, which is what no row says.

use rusqlite::{Connection, params};

pub(super) const CREATE_V9_TABLES: &str = "
    CREATE TABLE clip_decisions (
        project_id TEXT NOT NULL REFERENCES projects(project_id) ON DELETE CASCADE,
        source_id TEXT NOT NULL,
        candidate_id TEXT NOT NULL CHECK(length(candidate_id) > 0),
        decision TEXT NOT NULL CHECK(decision IN ('rejected', 'kept', 'approved')),
        decided_unix_millis INTEGER NOT NULL CHECK(decided_unix_millis >= 0),
        PRIMARY KEY(project_id, source_id, candidate_id)
    ) STRICT, WITHOUT ROWID;

    CREATE INDEX clip_decisions_by_source
        ON clip_decisions(project_id, source_id, decided_unix_millis DESC);
";

pub(super) const CREATE_V17_TABLES: &str = "
    ALTER TABLE clip_decisions ADD COLUMN reason TEXT NOT NULL DEFAULT '' CHECK(length(reason) <= 240);
    ALTER TABLE clip_decisions ADD COLUMN subject TEXT NOT NULL DEFAULT '' CHECK(length(subject) <= 200);
";

/// What a person decided about one candidate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Decision {
    /// Not this one. Kept as a decision rather than a deletion so a re-run does
    /// not put it back in front of somebody who already said no.
    Rejected,
    /// Maybe later — the answer that stops a good clip being lost to a session
    /// that ran out of time.
    Kept,
    /// Send it to the editor.
    Approved,
}

impl Decision {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Rejected => "rejected",
            Self::Kept => "kept",
            Self::Approved => "approved",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "rejected" => Some(Self::Rejected),
            "kept" => Some(Self::Kept),
            "approved" => Some(Self::Approved),
            _ => None,
        }
    }
}

/// One decision, as it is read back.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DecisionRecord {
    pub candidate_id: String,
    pub decision: Decision,
    pub decided_unix_millis: u64,
    pub reason: String,
}

/// Record a decision, replacing whatever was there.
pub(crate) fn set(
    connection: &Connection,
    project_id: &str,
    source_id: &str,
    candidate_id: &str,
    decision: Decision,
    now: u64,
) -> rusqlite::Result<()> {
    set_with_reason(
        connection,
        project_id,
        source_id,
        candidate_id,
        decision,
        now,
        "",
    )
}

pub(crate) fn set_with_reason(
    connection: &Connection,
    project_id: &str,
    source_id: &str,
    candidate_id: &str,
    decision: Decision,
    now: u64,
    reason: &str,
) -> rusqlite::Result<()> {
    connection.execute(
        "INSERT INTO clip_decisions
             (project_id, source_id, candidate_id, decision, decided_unix_millis,reason)
         VALUES (?1, ?2, ?3, ?4, ?5,?6)
         ON CONFLICT(project_id, source_id, candidate_id) DO UPDATE SET
             reason = CASE WHEN clip_decisions.decision = excluded.decision AND excluded.reason = ''
                      THEN clip_decisions.reason ELSE excluded.reason END,
             decision = excluded.decision,
             decided_unix_millis = excluded.decided_unix_millis",
        params![
            project_id,
            source_id,
            candidate_id,
            decision.as_str(),
            i64::try_from(now).unwrap_or(i64::MAX),
            reason
        ],
    )?;
    Ok(())
}

/// Record a decision with what it was about. An empty subject leaves the one
/// stored before, so a clip that has since left the analysis keeps its name.
#[expect(
    clippy::too_many_arguments,
    reason = "one column per value, as the decision row stores them"
)]
pub(crate) fn set_with_subject(
    connection: &Connection,
    project_id: &str,
    source_id: &str,
    candidate_id: &str,
    decision: Decision,
    now: u64,
    reason: &str,
    subject: &str,
) -> rusqlite::Result<()> {
    set_with_reason(
        connection,
        project_id,
        source_id,
        candidate_id,
        decision,
        now,
        reason,
    )?;
    if !subject.is_empty() {
        connection.execute(
            "UPDATE clip_decisions SET subject = ?4
              WHERE project_id = ?1 AND source_id = ?2 AND candidate_id = ?3",
            params![project_id, source_id, candidate_id, subject],
        )?;
    }
    Ok(())
}

/// Take a decision back, so the clip is undecided again.
///
/// A row that is not there is already the answer, so clearing a clip nobody
/// decided about is not an error — a retried undo must not fail on the second
/// attempt because the first one landed.
pub(crate) fn clear(
    connection: &Connection,
    project_id: &str,
    source_id: &str,
    candidate_id: &str,
) -> rusqlite::Result<()> {
    connection.execute(
        "DELETE FROM clip_decisions
          WHERE project_id = ?1 AND source_id = ?2 AND candidate_id = ?3",
        params![project_id, source_id, candidate_id],
    )?;
    Ok(())
}

/// Every decision for a source, newest first.
pub(crate) fn list(
    connection: &Connection,
    project_id: &str,
    source_id: &str,
) -> rusqlite::Result<Vec<DecisionRecord>> {
    let mut statement = connection.prepare(
        "SELECT candidate_id, decision, decided_unix_millis, reason
           FROM clip_decisions
          WHERE project_id = ?1 AND source_id = ?2
          ORDER BY decided_unix_millis DESC, candidate_id ASC",
    )?;
    let rows = statement.query_map(params![project_id, source_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;
    let mut found = Vec::new();
    for row in rows {
        let (candidate_id, decision, decided, reason) = row?;
        // A row whose decision word is not one of the three cannot happen while
        // the CHECK holds, and is skipped rather than guessed at if it ever does.
        if let Some(decision) = Decision::parse(&decision) {
            found.push(DecisionRecord {
                reason,
                candidate_id,
                decision,
                decided_unix_millis: u64::try_from(decided).unwrap_or(0),
            });
        }
    }
    Ok(found)
}

/// A review submitted through the engine. The subject comes from the ranking
/// the user viewed, including when no edit document has been created yet.
#[derive(Debug)]
pub(crate) struct Review {
    pub request_id: String,
    pub hash: [u8; 32],
    pub project_id: String,
    pub source_id: String,
    pub candidate_id: String,
    pub clip_id: String,
    pub decision: Option<Decision>,
    pub reason: String,
    pub subject: String,
    pub now: u64,
}

pub(super) fn save_review(
    connection: &mut Connection,
    review: &Review,
) -> Result<Vec<u8>, super::StoreError> {
    let db = connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    if let Some(bytes) = super::replay(&db, &review.request_id, &review.hash)? {
        return Ok(bytes);
    }
    let (choice, reason) = if let Some(choice) = review.decision {
        set_with_reason(
            &db,
            &review.project_id,
            &review.source_id,
            &review.candidate_id,
            choice,
            review.now,
            &review.reason,
        )?;
        let reason: String = db.query_row(
            "UPDATE clip_decisions SET subject=?4 WHERE project_id=?1 AND source_id=?2 AND candidate_id=?3 RETURNING reason",
            params![review.project_id, review.source_id, review.candidate_id,review.subject],|row|row.get(0)
        )?;
        (choice.as_str(), reason)
    } else {
        clear(
            &db,
            &review.project_id,
            &review.source_id,
            &review.candidate_id,
        )?;
        ("undecided", String::new())
    };
    let result = serde_json::json!({"clip_id":review.clip_id,"decision":choice,"reason":reason})
        .to_string()
        .into_bytes();
    super::remember(&db, &review.request_id, &review.hash, &result, review.now)?;
    db.commit()?;
    Ok(result)
}

/// Recent choices and editing actions within this project. Only the command's
/// verb goes into a brief, so an edit's file paths and asset IDs stay private.
pub(super) fn taste(connection: &Connection, project_id: &str) -> rusqlite::Result<Vec<String>> {
    let mut stmt = connection.prepare("
        WITH examples AS (
            SELECT d.decided_unix_millis AS ts, d.candidate_id AS id,
                d.decision || ' ' || coalesce(nullif(d.subject,''), coalesce(json_extract(e.document,'$.plan.form'),'clip') ||
                coalesce(' (“' || nullif(json_extract(e.document,'$.title'),'') || '”)','')) ||
                CASE WHEN d.reason<>'' THEN ' — ' || d.reason ELSE '' END AS detail
            FROM clip_decisions d LEFT JOIN edit_docs e ON e.doc_id=(
                SELECT doc_id FROM edit_docs WHERE project_id=d.project_id AND source_id=d.source_id
                    AND candidate_id=d.candidate_id ORDER BY updated_unix_millis DESC, doc_id LIMIT 1)
            WHERE d.project_id=?1 AND (e.doc_id IS NOT NULL OR d.reason<>'' OR d.subject<>'')
            UNION ALL
            SELECT c.applied_unix_millis, e.doc_id || ':' || c.revision,
                'Edited ' || coalesce(json_extract(e.document,'$.plan.form'),'clip') ||
                ' with ' || json_extract(c.command,'$.op')
            FROM edit_commands c JOIN edit_docs e ON e.doc_id=c.doc_id WHERE e.project_id=?1
        )
        SELECT substr(detail,1,240) FROM examples WHERE detail IS NOT NULL ORDER BY ts DESC,id LIMIT 12")?;
    stmt.query_map([project_id], |r| r.get(0))?.collect()
}
