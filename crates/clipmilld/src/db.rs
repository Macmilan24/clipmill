use std::{
    fs::{self, OpenOptions},
    path::Path,
    thread,
    time::Duration,
};

use clipmill_contracts::proto::ipc::v1::{
    CreateProjectResponse, DeleteProjectResponse, Project, Response, response,
};
use prost::Message;
use rusqlite::{
    Connection, OpenFlags, OptionalExtension, Transaction, TransactionBehavior, params,
};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

use crate::DaemonError;

const APPLICATION_ID: i64 = 0x434C_504D; // "CLPM"
const SCHEMA_VERSION: i64 = 1;
const SQLITE_MIN_VERSION: i32 = 3_051_003;
const COMMAND_CAPACITY: usize = 128;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectRecord {
    pub project_id: String,
    pub name: String,
    pub created_unix_millis: u64,
}

impl From<ProjectRecord> for Project {
    fn from(record: ProjectRecord) -> Self {
        Self {
            project_id: record.project_id,
            name: record.name,
            created_unix_millis: record.created_unix_millis,
        }
    }
}

#[derive(Debug, Error)]
pub(crate) enum StoreError {
    #[error("request id was already used with a different request body")]
    Conflict,
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("database contains invalid data: {0}")]
    InvalidData(&'static str),
    #[error("project was not found")]
    NotFound,
    #[error("database actor stopped")]
    Stopped,
}

#[derive(Clone, Debug)]
pub(crate) struct DbHandle {
    sender: mpsc::Sender<Command>,
}

#[derive(Debug)]
pub(crate) struct DbActor {
    handle: DbHandle,
    thread: Option<thread::JoinHandle<()>>,
}

impl DbActor {
    pub(crate) fn start(path: &Path) -> Result<Self, DaemonError> {
        let (sender, mut receiver) = mpsc::channel(COMMAND_CAPACITY);
        let (ready_sender, ready_receiver) = std::sync::mpsc::sync_channel(1);
        let database_path = path.to_path_buf();
        let actor_thread = thread::Builder::new()
            .name("clipmill-sqlite".to_owned())
            .spawn(move || {
                let connection = open_database(&database_path);
                match connection {
                    Ok(mut connection) => {
                        let _result = ready_sender.send(Ok(()));
                        while let Some(command) = receiver.blocking_recv() {
                            match command {
                                Command::Create {
                                    request_id,
                                    request_hash,
                                    project,
                                    reply,
                                } => {
                                    let _result = reply.send(create_project(
                                        &mut connection,
                                        &request_id,
                                        &request_hash,
                                        &project,
                                    ));
                                }
                                Command::Delete {
                                    request_id,
                                    request_hash,
                                    project_id,
                                    completed_unix_millis,
                                    reply,
                                } => {
                                    let _result = reply.send(delete_project(
                                        &mut connection,
                                        &request_id,
                                        &request_hash,
                                        &project_id,
                                        completed_unix_millis,
                                    ));
                                }
                                Command::Get { project_id, reply } => {
                                    let _result = reply.send(get_project(&connection, &project_id));
                                }
                                Command::List { reply } => {
                                    let _result = reply.send(list_projects(&connection));
                                }
                                Command::Shutdown { reply } => {
                                    let _result = reply.send(());
                                    break;
                                }
                            }
                        }
                    }
                    Err(error) => {
                        let _result = ready_sender.send(Err(error));
                    }
                }
            })
            .map_err(|source| DaemonError::io(path, source))?;

        match ready_receiver.recv() {
            Ok(Ok(())) => Ok(Self {
                handle: DbHandle { sender },
                thread: Some(actor_thread),
            }),
            Ok(Err(error)) => {
                let _result = actor_thread.join();
                Err(error)
            }
            Err(_) => {
                let _result = actor_thread.join();
                Err(DaemonError::DatabaseActorStopped)
            }
        }
    }

    pub(crate) fn handle(&self) -> DbHandle {
        self.handle.clone()
    }

    pub(crate) async fn shutdown(mut self) -> Result<(), DaemonError> {
        let (reply, received) = oneshot::channel();
        self.handle
            .sender
            .send(Command::Shutdown { reply })
            .await
            .map_err(|_| DaemonError::DatabaseActorStopped)?;
        received
            .await
            .map_err(|_| DaemonError::DatabaseActorStopped)?;

        if let Some(actor_thread) = self.thread.take() {
            tokio::task::spawn_blocking(move || actor_thread.join())
                .await
                .map_err(|_| DaemonError::DatabaseActorPanicked)?
                .map_err(|_| DaemonError::DatabaseActorPanicked)?;
        }
        Ok(())
    }
}

impl DbHandle {
    pub(crate) async fn create_project(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        project: ProjectRecord,
    ) -> Result<Vec<u8>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::Create {
                request_id,
                request_hash,
                project,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn delete_project(
        &self,
        request_id: String,
        request_hash: [u8; 32],
        project_id: String,
        completed_unix_millis: u64,
    ) -> Result<Vec<u8>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::Delete {
                request_id,
                request_hash,
                project_id,
                completed_unix_millis,
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn get_project(
        &self,
        project_id: String,
    ) -> Result<ProjectRecord, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::Get { project_id, reply })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn list_projects(&self) -> Result<Vec<ProjectRecord>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::List { reply })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }
}

#[derive(Debug)]
enum Command {
    Create {
        request_id: String,
        request_hash: [u8; 32],
        project: ProjectRecord,
        reply: oneshot::Sender<Result<Vec<u8>, StoreError>>,
    },
    Delete {
        request_id: String,
        request_hash: [u8; 32],
        project_id: String,
        completed_unix_millis: u64,
        reply: oneshot::Sender<Result<Vec<u8>, StoreError>>,
    },
    Get {
        project_id: String,
        reply: oneshot::Sender<Result<ProjectRecord, StoreError>>,
    },
    List {
        reply: oneshot::Sender<Result<Vec<ProjectRecord>, StoreError>>,
    },
    Shutdown {
        reply: oneshot::Sender<()>,
    },
}

fn open_database(path: &Path) -> Result<Connection, DaemonError> {
    let found_version = rusqlite::version_number();
    enforce_sqlite_version(found_version, rusqlite::version())?;

    create_private_database_file(path)?;
    let mut connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    connection.busy_timeout(Duration::from_secs(5))?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;

    let journal_mode: String =
        connection.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
    if !journal_mode.eq_ignore_ascii_case("wal") {
        return Err(DaemonError::UnexpectedJournalMode {
            found: journal_mode,
        });
    }
    connection.execute_batch("PRAGMA synchronous = FULL;")?;

    migrate(&mut connection)?;
    let check: String = connection.query_row("PRAGMA quick_check(1)", [], |row| row.get(0))?;
    enforce_integrity_check(&check)?;
    Ok(connection)
}

fn enforce_sqlite_version(found: i32, label: &str) -> Result<(), DaemonError> {
    if found < SQLITE_MIN_VERSION {
        return Err(DaemonError::SqliteTooOld {
            found: label.to_owned(),
        });
    }
    Ok(())
}

fn enforce_integrity_check(result: &str) -> Result<(), DaemonError> {
    if result != "ok" {
        return Err(DaemonError::IntegrityCheckFailed {
            result: result.to_owned(),
        });
    }
    Ok(())
}

fn create_private_database_file(path: &Path) -> Result<(), DaemonError> {
    #[cfg(unix)]
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

    if !path.exists() {
        let mut options = OpenOptions::new();
        options.create_new(true).read(true).write(true);
        #[cfg(unix)]
        options.mode(0o600);
        options
            .open(path)
            .map_err(|source| DaemonError::io(path, source))?;
    }
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .map_err(|source| DaemonError::io(path, source))?;
    Ok(())
}

fn migrate(connection: &mut Connection) -> Result<(), DaemonError> {
    let application_id: i64 =
        connection.query_row("PRAGMA application_id", [], |row| row.get(0))?;
    if application_id != 0 && application_id != APPLICATION_ID {
        return Err(DaemonError::UnexpectedApplicationId {
            found: application_id,
        });
    }

    let version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if version > SCHEMA_VERSION {
        return Err(DaemonError::UnsupportedSchema {
            found: version,
            supported: SCHEMA_VERSION,
        });
    }
    if version == 0 {
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute_batch(
            "
            CREATE TABLE projects (
                project_id TEXT PRIMARY KEY
                    CHECK(length(project_id) = 30 AND substr(project_id, 1, 4) = 'prj_'),
                name TEXT NOT NULL
                    CHECK(length(name) BETWEEN 1 AND 200),
                created_unix_millis INTEGER NOT NULL
                    CHECK(created_unix_millis >= 0)
            ) STRICT;

            CREATE TABLE request_dedup (
                request_id TEXT PRIMARY KEY
                    CHECK(length(request_id) BETWEEN 1 AND 128),
                request_hash BLOB NOT NULL
                    CHECK(length(request_hash) = 32),
                response_blob BLOB NOT NULL,
                completed_unix_millis INTEGER NOT NULL
                    CHECK(completed_unix_millis >= 0)
            ) STRICT;

            PRAGMA application_id = 1129074765;
            PRAGMA user_version = 1;
            ",
        )?;
        transaction.commit()?;
    }
    Ok(())
}

fn create_project(
    connection: &mut Connection,
    request_id: &str,
    request_hash: &[u8; 32],
    project: &ProjectRecord,
) -> Result<Vec<u8>, StoreError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(response) = replay(&transaction, request_id, request_hash)? {
        transaction.commit()?;
        return Ok(response);
    }

    let created = i64::try_from(project.created_unix_millis)
        .map_err(|_| StoreError::InvalidData("project timestamp exceeds SQLite integer range"))?;
    transaction.execute(
        "INSERT INTO projects(project_id, name, created_unix_millis) VALUES (?1, ?2, ?3)",
        params![project.project_id, project.name, created],
    )?;

    let response = Response {
        request_id: request_id.to_owned(),
        body: Some(response::Body::CreateProject(CreateProjectResponse {
            project: Some(project.clone().into()),
        })),
    }
    .encode_to_vec();
    remember(
        &transaction,
        request_id,
        request_hash,
        &response,
        project.created_unix_millis,
    )?;
    transaction.commit()?;
    Ok(response)
}

fn delete_project(
    connection: &mut Connection,
    request_id: &str,
    request_hash: &[u8; 32],
    project_id: &str,
    completed_unix_millis: u64,
) -> Result<Vec<u8>, StoreError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(response) = replay(&transaction, request_id, request_hash)? {
        transaction.commit()?;
        return Ok(response);
    }

    let deleted =
        transaction.execute("DELETE FROM projects WHERE project_id = ?1", [project_id])?;
    if deleted == 0 {
        return Err(StoreError::NotFound);
    }

    let response = Response {
        request_id: request_id.to_owned(),
        body: Some(response::Body::DeleteProject(DeleteProjectResponse {})),
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

fn replay(
    transaction: &Transaction<'_>,
    request_id: &str,
    request_hash: &[u8; 32],
) -> Result<Option<Vec<u8>>, StoreError> {
    let cached: Option<(Vec<u8>, Vec<u8>)> = transaction
        .query_row(
            "SELECT request_hash, response_blob FROM request_dedup WHERE request_id = ?1",
            [request_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    match cached {
        Some((cached_hash, response)) if cached_hash == request_hash => Ok(Some(response)),
        Some(_) => Err(StoreError::Conflict),
        None => Ok(None),
    }
}

fn remember(
    transaction: &Transaction<'_>,
    request_id: &str,
    request_hash: &[u8; 32],
    response: &[u8],
    completed_unix_millis: u64,
) -> Result<(), StoreError> {
    let completed = i64::try_from(completed_unix_millis)
        .map_err(|_| StoreError::InvalidData("request timestamp exceeds SQLite integer range"))?;
    transaction.execute(
        "INSERT INTO request_dedup(request_id, request_hash, response_blob, completed_unix_millis)
         VALUES (?1, ?2, ?3, ?4)",
        params![request_id, request_hash.as_slice(), response, completed],
    )?;
    Ok(())
}

fn get_project(connection: &Connection, project_id: &str) -> Result<ProjectRecord, StoreError> {
    connection
        .query_row(
            "SELECT project_id, name, created_unix_millis FROM projects WHERE project_id = ?1",
            [project_id],
            project_from_row,
        )
        .optional()?
        .ok_or(StoreError::NotFound)
}

fn list_projects(connection: &Connection) -> Result<Vec<ProjectRecord>, StoreError> {
    let mut statement = connection.prepare(
        "SELECT project_id, name, created_unix_millis
         FROM projects
         ORDER BY created_unix_millis DESC, project_id DESC",
    )?;
    let rows = statement.query_map([], project_from_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn project_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProjectRecord> {
    let created: i64 = row.get(2)?;
    let created_unix_millis = u64::try_from(created).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            2,
            rusqlite::types::Type::Integer,
            Box::new(error),
        )
    })?;
    Ok(ProjectRecord {
        project_id: row.get(0)?,
        name: row.get(1)?,
        created_unix_millis,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use std::path::Path;

    use prost::Message;
    use rusqlite::Connection;
    use tempfile::TempDir;

    use super::{
        ProjectRecord, SCHEMA_VERSION, SQLITE_MIN_VERSION, StoreError, create_project,
        delete_project, enforce_integrity_check, enforce_sqlite_version, get_project,
        list_projects, open_database,
    };
    use crate::DaemonError;

    fn database(temp: &TempDir) -> (std::path::PathBuf, Connection) {
        let path = temp.path().join("clipmill.db");
        let connection = open_database(&path).expect("database opens");
        (path, connection)
    }

    fn project(id: &str, name: &str, at: u64) -> ProjectRecord {
        ProjectRecord {
            project_id: id.to_owned(),
            name: name.to_owned(),
            created_unix_millis: at,
        }
    }

    #[test]
    fn database_is_migrated_and_configured() {
        let temp = TempDir::new().expect("tempdir");
        let (path, connection) = database(&temp);
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("version");
        let mode: String = connection
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .expect("journal mode");
        let foreign_keys: i64 = connection
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("foreign keys");
        let synchronous: i64 = connection
            .query_row("PRAGMA synchronous", [], |row| row.get(0))
            .expect("synchronous mode");
        let busy_timeout: i64 = connection
            .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
            .expect("busy timeout");
        let strict_projects: i64 = connection
            .query_row(
                "SELECT strict FROM pragma_table_list WHERE name = 'projects'",
                [],
                |row| row.get(0),
            )
            .expect("projects table mode");
        let quick_check: String = connection
            .query_row("PRAGMA quick_check(1)", [], |row| row.get(0))
            .expect("quick check");
        assert_eq!(version, SCHEMA_VERSION);
        assert_eq!(mode, "wal");
        assert_eq!(foreign_keys, 1);
        assert_eq!(synchronous, 2);
        assert_eq!(busy_timeout, 5_000);
        assert_eq!(strict_projects, 1);
        assert_eq!(quick_check, "ok");
        drop(connection);
        open_database(&path).expect("repeat startup succeeds");
    }

    #[test]
    fn sqlite_version_and_integrity_results_are_enforced() {
        assert!(enforce_sqlite_version(SQLITE_MIN_VERSION, "minimum").is_ok());
        assert!(matches!(
            enforce_sqlite_version(SQLITE_MIN_VERSION - 1, "too-old"),
            Err(DaemonError::SqliteTooOld { .. })
        ));
        assert!(enforce_integrity_check("ok").is_ok());
        assert!(matches!(
            enforce_integrity_check("database disk image is malformed"),
            Err(DaemonError::IntegrityCheckFailed { .. })
        ));
    }

    #[test]
    fn newer_schema_is_refused() {
        let temp = TempDir::new().expect("tempdir");
        let path = temp.path().join("newer.db");
        let connection = Connection::open(&path).expect("open raw database");
        connection
            .execute_batch("PRAGMA application_id = 1129074765; PRAGMA user_version = 2;")
            .expect("set version");
        drop(connection);
        assert!(open_database(&path).is_err());
    }

    #[test]
    fn failed_migration_rolls_back_to_the_prior_schema() {
        let temp = TempDir::new().expect("tempdir");
        let path = temp.path().join("migration-failure.db");
        let connection = Connection::open(&path).expect("open raw database");
        connection
            .execute_batch(
                "CREATE TABLE projects (marker TEXT NOT NULL);\n\
                 INSERT INTO projects(marker) VALUES ('prior');",
            )
            .expect("create prior schema");
        drop(connection);

        assert!(open_database(&path).is_err());

        let connection = Connection::open(&path).expect("reopen prior database");
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("schema version");
        let application_id: i64 = connection
            .query_row("PRAGMA application_id", [], |row| row.get(0))
            .expect("application id");
        let marker: String = connection
            .query_row("SELECT marker FROM projects", [], |row| row.get(0))
            .expect("prior row");
        let dedup_tables: i64 = connection
            .query_row(
                "SELECT count(*) FROM sqlite_master\n\
                 WHERE type = 'table' AND name = 'request_dedup'",
                [],
                |row| row.get(0),
            )
            .expect("dedup table count");
        assert_eq!(version, 0);
        assert_eq!(application_id, 0);
        assert_eq!(marker, "prior");
        assert_eq!(dedup_tables, 0);
    }

    #[test]
    fn project_crud_and_ordering_are_deterministic() {
        let temp = TempDir::new().expect("tempdir");
        let (_path, mut connection) = database(&temp);
        let first = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAV", "First", 10);
        let second = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAW", "First", 20);
        create_project(&mut connection, "create-1", &[1; 32], &first).expect("create first");
        create_project(&mut connection, "create-2", &[2; 32], &second).expect("create second");

        assert_eq!(
            get_project(&connection, &first.project_id).expect("get"),
            first
        );
        assert_eq!(
            list_projects(&connection).expect("list"),
            vec![second, first]
        );

        delete_project(
            &mut connection,
            "delete-1",
            &[3; 32],
            "prj_01ARZ3NDEKTSV4RRFFQ69G5FAV",
            30,
        )
        .expect("delete");
        assert!(matches!(
            get_project(&connection, "prj_01ARZ3NDEKTSV4RRFFQ69G5FAV"),
            Err(StoreError::NotFound)
        ));
    }

    #[test]
    fn successful_mutation_replays_exact_bytes_and_rejects_collision() {
        let temp = TempDir::new().expect("tempdir");
        let (_path, mut connection) = database(&temp);
        let original = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAV", "Original", 10);
        let different = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAW", "Different", 20);

        let first =
            create_project(&mut connection, "same", &[9; 32], &original).expect("first response");
        let replayed = create_project(&mut connection, "same", &[9; 32], &different)
            .expect("replayed response");
        assert_eq!(first, replayed);
        assert!(clipmill_contracts::proto::ipc::v1::Response::decode(first.as_slice()).is_ok());
        assert!(matches!(
            create_project(&mut connection, "same", &[8; 32], &different),
            Err(StoreError::Conflict)
        ));
        assert_eq!(list_projects(&connection).expect("list"), vec![original]);
    }

    #[test]
    fn failed_mutation_rolls_back_without_claiming_request_id() {
        let temp = TempDir::new().expect("tempdir");
        let (_path, mut connection) = database(&temp);
        let original = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAV", "Original", 10);
        create_project(&mut connection, "original", &[1; 32], &original).expect("create original");

        let duplicate = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAV", "Duplicate", 20);
        assert!(matches!(
            create_project(&mut connection, "retryable", &[2; 32], &duplicate),
            Err(StoreError::Database(_))
        ));

        let retry = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAW", "Retry", 30);
        create_project(&mut connection, "retryable", &[2; 32], &retry)
            .expect("request id remains available after rollback");
        assert_eq!(list_projects(&connection).expect("list").len(), 2);
    }

    #[test]
    fn database_file_is_private() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let temp = TempDir::new().expect("tempdir");
            let (path, _connection) = database(&temp);
            let mode = std::fs::metadata(Path::new(&path))
                .expect("metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
        }
    }
}
