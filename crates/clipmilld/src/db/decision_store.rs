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
//! that grows without a reader.

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
    connection.execute(
        "INSERT INTO clip_decisions
             (project_id, source_id, candidate_id, decision, decided_unix_millis)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(project_id, source_id, candidate_id) DO UPDATE SET
             decision = excluded.decision,
             decided_unix_millis = excluded.decided_unix_millis",
        params![
            project_id,
            source_id,
            candidate_id,
            decision.as_str(),
            i64::try_from(now).unwrap_or(i64::MAX)
        ],
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
        "SELECT candidate_id, decision, decided_unix_millis
           FROM clip_decisions
          WHERE project_id = ?1 AND source_id = ?2
          ORDER BY decided_unix_millis DESC, candidate_id ASC",
    )?;
    let rows = statement.query_map(params![project_id, source_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
        ))
    })?;
    let mut found = Vec::new();
    for row in rows {
        let (candidate_id, decision, decided) = row?;
        // A row whose decision word is not one of the three cannot happen while
        // the CHECK holds, and is skipped rather than guessed at if it ever does.
        if let Some(decision) = Decision::parse(&decision) {
            found.push(DecisionRecord {
                candidate_id,
                decision,
                decided_unix_millis: u64::try_from(decided).unwrap_or(0),
            });
        }
    }
    Ok(found)
}
