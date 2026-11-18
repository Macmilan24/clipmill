//! Connected agents and the engine API's audit log.
//!
//! A token is shown to the person once, when the client is created, and kept
//! here only as its SHA-256: a copy of this database is not a set of working
//! credentials. Every engine API call is recorded with the client that made
//! it, so a person can see what an agent did and when.

use rusqlite::{Connection, OptionalExtension, params};

use super::StoreError;

pub(super) const CREATE_V15_TABLES: &str = "
 CREATE TABLE agent_clients (
   client_id TEXT PRIMARY KEY,
   name TEXT NOT NULL,
   token_hash BLOB NOT NULL UNIQUE,
   scopes TEXT NOT NULL,
   created_unix_millis INTEGER NOT NULL,
   revoked_unix_millis INTEGER NOT NULL DEFAULT 0,
   last_used_unix_millis INTEGER NOT NULL DEFAULT 0
 ) STRICT;
 CREATE TABLE engine_audit (
   audit_id INTEGER PRIMARY KEY,
   at_unix_millis INTEGER NOT NULL,
   client_id TEXT NOT NULL,
   operation TEXT NOT NULL,
   target TEXT NOT NULL,
   outcome TEXT NOT NULL CHECK(outcome IN ('ok','refused','failed')),
   detail TEXT NOT NULL
 ) STRICT;
 CREATE INDEX engine_audit_time ON engine_audit(at_unix_millis);
";

pub(super) const CREATE_V16_TABLES: &str = "
    CREATE TABLE engine_job_owners (
        job_id TEXT PRIMARY KEY REFERENCES jobs(job_id) ON DELETE CASCADE,
        client_id TEXT NOT NULL
    ) STRICT, WITHOUT ROWID;
";

/// The audit log keeps this many entries; older ones are dropped as new ones
/// arrive, so it cannot grow without bound on a busy agent.
const AUDIT_KEEP: i64 = 20_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AgentClient {
    pub client_id: String,
    pub name: String,
    pub scopes: Vec<String>,
    pub created_unix_millis: u64,
    pub revoked_unix_millis: u64,
    pub last_used_unix_millis: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AuditEntry {
    pub at_unix_millis: u64,
    pub client_id: String,
    pub client_name: String,
    pub operation: String,
    pub target: String,
    pub outcome: &'static str,
    pub detail: String,
}

#[derive(Debug)]
pub(crate) enum AgentCommand {
    Create {
        client_id: String,
        name: String,
        token_hash: [u8; 32],
        scopes: Vec<String>,
        created_unix_millis: u64,
    },
    Revoke {
        client_id: String,
        at_unix_millis: u64,
    },
    List,
    /// The live client a token belongs to, marked as used now.
    Resolve {
        token_hash: [u8; 32],
        at_unix_millis: u64,
    },
    OwnedJob {
        job_id: String,
        client_id: String,
    },
    Record(AuditEntry),
    Log {
        limit: u32,
    },
}

#[derive(Debug)]
pub(crate) enum AgentReply {
    Clients(Vec<AgentClient>),
    Client(Option<AgentClient>),
    OwnedJob(Option<String>),
    Recorded,
    Log(Vec<AuditEntry>),
}

fn millis(value: i64) -> u64 {
    u64::try_from(value).unwrap_or(0)
}

fn signed(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn client(row: &rusqlite::Row<'_>) -> rusqlite::Result<AgentClient> {
    let scopes: String = row.get(2)?;
    Ok(AgentClient {
        client_id: row.get(0)?,
        name: row.get(1)?,
        scopes: scopes.split_whitespace().map(str::to_owned).collect(),
        created_unix_millis: millis(row.get(3)?),
        revoked_unix_millis: millis(row.get(4)?),
        last_used_unix_millis: millis(row.get(5)?),
    })
}

const CLIENT_COLUMNS: &str =
    "client_id, name, scopes, created_unix_millis, revoked_unix_millis, last_used_unix_millis";

fn list(connection: &Connection) -> Result<Vec<AgentClient>, StoreError> {
    let mut statement = connection.prepare(&format!(
        "SELECT {CLIENT_COLUMNS} FROM agent_clients ORDER BY created_unix_millis, client_id"
    ))?;
    let rows = statement.query_map([], client)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

#[allow(
    clippy::too_many_lines,
    reason = "one arm per command, each a statement or two of SQL"
)]
pub(super) fn execute(
    connection: &mut Connection,
    command: AgentCommand,
) -> Result<AgentReply, StoreError> {
    match command {
        AgentCommand::Create {
            client_id,
            name,
            token_hash,
            scopes,
            created_unix_millis,
        } => {
            connection.execute(
                "INSERT INTO agent_clients (client_id, name, token_hash, scopes, created_unix_millis)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    client_id,
                    name,
                    token_hash.as_slice(),
                    scopes.join(" "),
                    signed(created_unix_millis)
                ],
            )?;
            Ok(AgentReply::Clients(list(connection)?))
        }
        AgentCommand::Revoke {
            client_id,
            at_unix_millis,
        } => {
            let changed = connection.execute(
                "UPDATE agent_clients SET revoked_unix_millis = ?2
                 WHERE client_id = ?1 AND revoked_unix_millis = 0",
                params![client_id, signed(at_unix_millis)],
            )?;
            let known: bool = connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM agent_clients WHERE client_id = ?1)",
                [&client_id],
                |row| row.get(0),
            )?;
            if changed == 0 && !known {
                return Err(StoreError::NotFound);
            }
            Ok(AgentReply::Clients(list(connection)?))
        }
        AgentCommand::List => Ok(AgentReply::Clients(list(connection)?)),
        AgentCommand::Resolve {
            token_hash,
            at_unix_millis,
        } => {
            let found = connection
                .query_row(
                    &format!(
                        "SELECT {CLIENT_COLUMNS} FROM agent_clients
                         WHERE token_hash = ?1 AND revoked_unix_millis = 0"
                    ),
                    [token_hash.as_slice()],
                    client,
                )
                .optional()?;
            if let Some(found) = &found {
                connection.execute(
                    "UPDATE agent_clients SET last_used_unix_millis = ?2 WHERE client_id = ?1",
                    params![found.client_id, signed(at_unix_millis)],
                )?;
            }
            Ok(AgentReply::Client(found))
        }
        AgentCommand::OwnedJob { job_id, client_id } => {
            let kind = connection
                .query_row(
                    "SELECT j.kind FROM jobs j JOIN engine_job_owners o
                ON o.job_id=j.job_id WHERE j.job_id=?1 AND o.client_id=?2",
                    params![job_id, client_id],
                    |r| r.get(0),
                )
                .optional()?;
            Ok(AgentReply::OwnedJob(kind))
        }
        AgentCommand::Record(entry) => {
            connection.execute(
                "INSERT INTO engine_audit (at_unix_millis, client_id, operation, target, outcome, detail)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    signed(entry.at_unix_millis),
                    entry.client_id,
                    entry.operation,
                    entry.target,
                    entry.outcome,
                    entry.detail
                ],
            )?;
            connection.execute(
                "DELETE FROM engine_audit WHERE audit_id <= (SELECT MAX(audit_id) FROM engine_audit) - ?1",
                [AUDIT_KEEP],
            )?;
            Ok(AgentReply::Recorded)
        }
        AgentCommand::Log { limit } => {
            let mut statement = connection.prepare(
                "SELECT a.at_unix_millis, a.client_id, COALESCE(c.name, a.client_id), a.operation,
                        a.target, a.outcome, a.detail
                 FROM engine_audit a LEFT JOIN agent_clients c ON c.client_id = a.client_id
                 ORDER BY a.audit_id DESC LIMIT ?1",
            )?;
            let rows = statement.query_map([i64::from(limit)], |row| {
                let outcome: String = row.get(5)?;
                Ok(AuditEntry {
                    at_unix_millis: millis(row.get(0)?),
                    client_id: row.get(1)?,
                    client_name: row.get(2)?,
                    operation: row.get(3)?,
                    target: row.get(4)?,
                    outcome: match outcome.as_str() {
                        "ok" => "ok",
                        "refused" => "refused",
                        _ => "failed",
                    },
                    detail: row.get(6)?,
                })
            })?;
            Ok(AgentReply::Log(rows.collect::<Result<_, _>>()?))
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    fn connection() -> Connection {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch(CREATE_V15_TABLES).unwrap();
        connection
    }

    #[test]
    fn a_token_resolves_until_its_client_is_revoked() {
        let mut db = connection();
        execute(
            &mut db,
            AgentCommand::Create {
                client_id: "agt_1".into(),
                name: "Claude Desktop".into(),
                token_hash: [7; 32],
                scopes: vec!["read".into(), "edit".into()],
                created_unix_millis: 1,
            },
        )
        .unwrap();
        let AgentReply::Client(Some(found)) = execute(
            &mut db,
            AgentCommand::Resolve {
                token_hash: [7; 32],
                at_unix_millis: 5,
            },
        )
        .unwrap() else {
            panic!("resolves");
        };
        assert_eq!(found.scopes, ["read", "edit"]);
        execute(
            &mut db,
            AgentCommand::Revoke {
                client_id: "agt_1".into(),
                at_unix_millis: 9,
            },
        )
        .unwrap();
        let AgentReply::Client(gone) = execute(
            &mut db,
            AgentCommand::Resolve {
                token_hash: [7; 32],
                at_unix_millis: 10,
            },
        )
        .unwrap() else {
            panic!("answers");
        };
        assert!(gone.is_none());
    }

    #[test]
    fn the_audit_log_reads_newest_first_with_the_clients_name() {
        let mut db = connection();
        execute(
            &mut db,
            AgentCommand::Create {
                client_id: "agt_1".into(),
                name: "Claude Desktop".into(),
                token_hash: [1; 32],
                scopes: vec!["read".into()],
                created_unix_millis: 1,
            },
        )
        .unwrap();
        for (at, operation) in [(2, "list_projects"), (3, "start_analysis")] {
            execute(
                &mut db,
                AgentCommand::Record(AuditEntry {
                    at_unix_millis: at,
                    client_id: "agt_1".into(),
                    client_name: String::new(),
                    operation: operation.into(),
                    target: String::new(),
                    outcome: if at == 3 { "refused" } else { "ok" },
                    detail: String::new(),
                }),
            )
            .unwrap();
        }
        let AgentReply::Log(log) = execute(&mut db, AgentCommand::Log { limit: 10 }).unwrap()
        else {
            panic!("log");
        };
        assert_eq!(log[0].operation, "start_analysis");
        assert_eq!(log[0].outcome, "refused");
        assert_eq!(log[1].client_name, "Claude Desktop");
    }
}
