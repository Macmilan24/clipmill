use std::{
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use clipmill_contracts::proto::ipc::v1::{
    CreateProjectResponse, DeleteProjectResponse, Project, Response, response,
};
use clipmill_core::{ArtifactId, ProjectId};
use prost::Message;
use rusqlite::{
    Connection, OpenFlags, OptionalExtension, Transaction, TransactionBehavior, backup::Backup,
    params,
};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};
use ulid::Ulid;

use crate::DaemonError;

const APPLICATION_ID: i64 = 0x434C_504D; // "CLPM"
const SCHEMA_VERSION: i64 = 2;
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
    pub(crate) fn start(path: &Path, backups_dir: &Path) -> Result<Self, DaemonError> {
        let (sender, mut receiver) = mpsc::channel(COMMAND_CAPACITY);
        let (ready_sender, ready_receiver) = std::sync::mpsc::sync_channel(1);
        let database_path = path.to_path_buf();
        let database_backups = backups_dir.to_path_buf();
        let actor_thread = thread::Builder::new()
            .name("clipmill-sqlite".to_owned())
            .spawn(move || {
                let connection = open_database(&database_path, &database_backups);
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
                                Command::AttachArtifactRoot {
                                    project_id,
                                    artifact_id,
                                    reply,
                                } => {
                                    let _result = reply.send(attach_artifact_root(
                                        &mut connection,
                                        &project_id,
                                        &artifact_id,
                                    ));
                                }
                                Command::ListArtifactRoots { reply } => {
                                    let _result = reply.send(list_artifact_roots(&connection));
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

    pub(crate) async fn attach_artifact_root(
        &self,
        project_id: ProjectId,
        artifact_id: ArtifactId,
    ) -> Result<(), StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::AttachArtifactRoot {
                project_id: project_id.to_string(),
                artifact_id: artifact_id.to_string(),
                reply,
            })
            .await
            .map_err(|_| StoreError::Stopped)?;
        received.await.map_err(|_| StoreError::Stopped)?
    }

    pub(crate) async fn list_artifact_roots(&self) -> Result<Vec<ArtifactId>, StoreError> {
        let (reply, received) = oneshot::channel();
        self.sender
            .send(Command::ListArtifactRoots { reply })
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
    AttachArtifactRoot {
        project_id: String,
        artifact_id: String,
        reply: oneshot::Sender<Result<(), StoreError>>,
    },
    ListArtifactRoots {
        reply: oneshot::Sender<Result<Vec<ArtifactId>, StoreError>>,
    },
    Shutdown {
        reply: oneshot::Sender<()>,
    },
}

fn open_database(path: &Path, backups_dir: &Path) -> Result<Connection, DaemonError> {
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

    let check: String = connection.query_row("PRAGMA quick_check(1)", [], |row| row.get(0))?;
    enforce_integrity_check(&check)?;
    migrate(&mut connection, backups_dir)?;
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

const CREATE_V1_TABLES: &str = "
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
";

const CREATE_V2_TABLES: &str = "
    CREATE TABLE project_artifact_roots (
        project_id TEXT NOT NULL
            REFERENCES projects(project_id) ON DELETE CASCADE,
        artifact_id TEXT NOT NULL
            CHECK(
                length(artifact_id) = 71
                AND substr(artifact_id, 1, 7) = 'sha256:'
                AND substr(artifact_id, 8) NOT GLOB '*[^0-9a-f]*'
            ),
        PRIMARY KEY(project_id, artifact_id)
    ) STRICT, WITHOUT ROWID;

    CREATE INDEX project_artifact_roots_by_artifact
        ON project_artifact_roots(artifact_id);
";

fn migrate(connection: &mut Connection, backups_dir: &Path) -> Result<(), DaemonError> {
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
        transaction.execute_batch(CREATE_V1_TABLES)?;
        transaction.execute_batch(CREATE_V2_TABLES)?;
        transaction
            .execute_batch("PRAGMA application_id = 1129074765; PRAGMA user_version = 2;")?;
        transaction.commit()?;
    } else if version == 1 {
        create_v1_backup(connection, backups_dir)?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute_batch(CREATE_V2_TABLES)?;
        transaction.execute_batch("PRAGMA user_version = 2;")?;
        transaction.commit()?;
    }
    Ok(())
}

fn create_v1_backup(source: &Connection, backups_dir: &Path) -> Result<PathBuf, DaemonError> {
    create_private_directory(backups_dir)?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| DaemonError::InvalidDuration(error.to_string()))?
        .as_millis();
    let base = format!("clipmill-v1-to-v2-{timestamp}-{}", Ulid::new());
    let temporary = backups_dir.join(format!("{base}.db.tmp"));
    let final_path = backups_dir.join(format!("{base}.db"));
    create_private_database_file(&temporary)?;

    let backup_result = (|| -> Result<(), DaemonError> {
        let mut destination = Connection::open_with_flags(
            &temporary,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        {
            let backup = Backup::new(source, &mut destination)?;
            backup.run_to_completion(128, Duration::from_millis(5), None)?;
        }
        let check: String = destination.query_row("PRAGMA quick_check(1)", [], |row| row.get(0))?;
        enforce_integrity_check(&check)?;
        drop(destination);
        File::open(&temporary)
            .and_then(|file| file.sync_all())
            .map_err(|source| DaemonError::io(&temporary, source))?;
        fs::rename(&temporary, &final_path)
            .map_err(|source| DaemonError::io(&final_path, source))?;
        set_private_file_permissions(&final_path)?;
        sync_directory(backups_dir)
    })();

    if backup_result.is_err() {
        let _cleanup = fs::remove_file(&temporary);
    }
    backup_result?;
    Ok(final_path)
}

fn create_private_directory(path: &Path) -> Result<(), DaemonError> {
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    fs::create_dir_all(path).map_err(|source| DaemonError::io(path, source))?;
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|source| DaemonError::io(path, source))?;
    Ok(())
}

fn set_private_file_permissions(path: &Path) -> Result<(), DaemonError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|source| DaemonError::io(path, source))?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), DaemonError> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|source| DaemonError::io(path, source))
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

fn attach_artifact_root(
    connection: &mut Connection,
    project_id: &str,
    artifact_id: &str,
) -> Result<(), StoreError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let project_exists: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM projects WHERE project_id = ?1)",
        [project_id],
        |row| row.get(0),
    )?;
    if !project_exists {
        return Err(StoreError::NotFound);
    }
    transaction.execute(
        "INSERT OR IGNORE INTO project_artifact_roots(project_id, artifact_id) VALUES (?1, ?2)",
        params![project_id, artifact_id],
    )?;
    transaction.commit()?;
    Ok(())
}

fn list_artifact_roots(connection: &Connection) -> Result<Vec<ArtifactId>, StoreError> {
    let mut statement = connection.prepare(
        "SELECT DISTINCT artifact_id FROM project_artifact_roots ORDER BY artifact_id ASC",
    )?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    let encoded = rows.collect::<Result<Vec<_>, _>>()?;
    encoded
        .into_iter()
        .map(|value| {
            value
                .parse()
                .map_err(|_| StoreError::InvalidData("artifact root id is invalid"))
        })
        .collect()
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

    use std::{fs, path::Path};

    use clipmill_core::{ArtifactId, Sha256Digest};
    use prost::Message;
    use rusqlite::{Connection, OpenFlags};
    use tempfile::TempDir;

    use super::{
        CREATE_V1_TABLES, ProjectRecord, SCHEMA_VERSION, SQLITE_MIN_VERSION, StoreError,
        attach_artifact_root, create_project, delete_project, enforce_integrity_check,
        enforce_sqlite_version, get_project, list_artifact_roots, list_projects, open_database,
    };
    use crate::DaemonError;

    fn database(temp: &TempDir) -> (std::path::PathBuf, Connection) {
        let path = temp.path().join("clipmill.db");
        let connection =
            open_database(&path, &temp.path().join("backups")).expect("database opens");
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
        let strict_roots: i64 = connection
            .query_row(
                "SELECT strict FROM pragma_table_list WHERE name = 'project_artifact_roots'",
                [],
                |row| row.get(0),
            )
            .expect("artifact roots table mode");
        let quick_check: String = connection
            .query_row("PRAGMA quick_check(1)", [], |row| row.get(0))
            .expect("quick check");
        assert_eq!(version, SCHEMA_VERSION);
        assert_eq!(mode, "wal");
        assert_eq!(foreign_keys, 1);
        assert_eq!(synchronous, 2);
        assert_eq!(busy_timeout, 5_000);
        assert_eq!(strict_projects, 1);
        assert_eq!(strict_roots, 1);
        assert_eq!(quick_check, "ok");
        let backup_count = fs::read_dir(temp.path().join("backups"))
            .map(Iterator::count)
            .unwrap_or_default();
        assert_eq!(backup_count, 0);
        drop(connection);
        open_database(&path, &temp.path().join("backups")).expect("repeat startup succeeds");
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
            .execute_batch("PRAGMA application_id = 1129074765; PRAGMA user_version = 3;")
            .expect("set version");
        drop(connection);
        assert!(open_database(&path, &temp.path().join("backups")).is_err());
    }

    #[test]
    fn v1_upgrade_creates_verified_private_backup_and_preserves_state() {
        let temp = TempDir::new().expect("tempdir");
        let path = temp.path().join("v1.db");
        let backups = temp.path().join("backups");
        let connection = Connection::open(&path).expect("open v1 database");
        connection
            .execute_batch(CREATE_V1_TABLES)
            .expect("v1 tables");
        connection
            .execute_batch(
                "INSERT INTO projects(project_id, name, created_unix_millis)
                 VALUES ('prj_01ARZ3NDEKTSV4RRFFQ69G5FAV', 'Before upgrade', 1);
                 PRAGMA application_id = 1129074765;
                 PRAGMA user_version = 1;",
            )
            .expect("v1 state");
        drop(connection);

        let upgraded = open_database(&path, &backups).expect("upgrade");
        let version: i64 = upgraded
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("version");
        let name: String = upgraded
            .query_row("SELECT name FROM projects", [], |row| row.get(0))
            .expect("project preserved");
        assert_eq!(version, 2);
        assert_eq!(name, "Before upgrade");
        drop(upgraded);

        let backup_paths = fs::read_dir(&backups)
            .expect("backups")
            .map(|entry| entry.expect("backup entry").path())
            .collect::<Vec<_>>();
        assert_eq!(backup_paths.len(), 1);
        assert_eq!(
            backup_paths[0].extension().and_then(|value| value.to_str()),
            Some("db")
        );
        let backup = Connection::open_with_flags(
            &backup_paths[0],
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .expect("open backup read-only");
        let backup_version: i64 = backup
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("backup version");
        let backup_check: String = backup
            .query_row("PRAGMA quick_check(1)", [], |row| row.get(0))
            .expect("backup quick check");
        assert_eq!(backup_version, 1);
        assert_eq!(backup_check, "ok");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&backup_paths[0])
                    .expect("backup metadata")
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
    }

    #[test]
    fn failed_v1_upgrade_keeps_v1_transactionally_intact() {
        let temp = TempDir::new().expect("tempdir");
        let path = temp.path().join("v1-failure.db");
        let backups = temp.path().join("backups");
        let connection = Connection::open(&path).expect("open v1 database");
        connection
            .execute_batch(CREATE_V1_TABLES)
            .expect("v1 tables");
        connection
            .execute_batch(
                "CREATE TABLE project_artifact_roots (marker TEXT NOT NULL);
                 INSERT INTO project_artifact_roots(marker) VALUES ('prior');
                 PRAGMA application_id = 1129074765;
                 PRAGMA user_version = 1;",
            )
            .expect("conflicting v1 state");
        drop(connection);

        assert!(open_database(&path, &backups).is_err());
        let prior = Connection::open(&path).expect("reopen v1");
        let version: i64 = prior
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("v1 version remains");
        let marker: String = prior
            .query_row("SELECT marker FROM project_artifact_roots", [], |row| {
                row.get(0)
            })
            .expect("prior table remains");
        assert_eq!(version, 1);
        assert_eq!(marker, "prior");
        assert_eq!(fs::read_dir(backups).expect("backup retained").count(), 1);
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

        assert!(open_database(&path, &temp.path().join("backups")).is_err());

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
    fn artifact_roots_are_idempotent_and_cascade_with_projects() {
        let temp = TempDir::new().expect("tempdir");
        let (_path, mut connection) = database(&temp);
        let project = project("prj_01ARZ3NDEKTSV4RRFFQ69G5FAV", "Rooted", 10);
        create_project(&mut connection, "create-rooted", &[1; 32], &project)
            .expect("create project");
        let artifact = ArtifactId::from_digest(Sha256Digest::from_bytes([0xab; 32]));
        attach_artifact_root(&mut connection, &project.project_id, &artifact.to_string())
            .expect("attach");
        attach_artifact_root(&mut connection, &project.project_id, &artifact.to_string())
            .expect("idempotent attach");
        assert_eq!(
            list_artifact_roots(&connection).expect("roots"),
            vec![artifact]
        );

        delete_project(
            &mut connection,
            "delete-rooted",
            &[2; 32],
            &project.project_id,
            20,
        )
        .expect("delete project");
        assert!(
            list_artifact_roots(&connection)
                .expect("roots after delete")
                .is_empty()
        );
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
