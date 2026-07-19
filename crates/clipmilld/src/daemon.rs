use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::{
    fs,
    future::Future,
    io,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use tokio::{
    net::{UnixListener, UnixStream},
    sync::Semaphore,
    task::JoinSet,
    time::timeout,
};

use crate::{
    Config, DaemonError,
    db::DbActor,
    ipc::{FrameError, handle_connection},
    lock::DaemonLock,
    service::Service,
};

const MAX_CONNECTIONS: usize = 64;
const DRAIN_TIMEOUT: Duration = Duration::from_secs(5);
const SOCKET_PROBE_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Debug)]
pub struct Daemon {
    listener: UnixListener,
    service: Service,
    database: DbActor,
    socket: SocketGuard,
    _lock: DaemonLock,
}

impl Daemon {
    pub async fn start(config: Config) -> Result<Self, DaemonError> {
        let started_unix_millis = unix_millis()?;
        prepare_directories(&config)?;
        let daemon_lock = DaemonLock::acquire(&config.paths.lock)?;
        let database = DbActor::start(&config.paths.database)?;

        if let Err(error) = recover_stale_socket(&config.paths.socket).await {
            database.shutdown().await?;
            return Err(error);
        }
        let listener = match UnixListener::bind(&config.paths.socket) {
            Ok(listener) => listener,
            Err(source) => {
                database.shutdown().await?;
                return Err(DaemonError::io(&config.paths.socket, source));
            }
        };
        let socket = SocketGuard::new(config.paths.socket);
        if let Err(error) = set_private_permissions(&socket.path, 0o600) {
            database.shutdown().await?;
            return Err(error);
        }
        let service = Service::new(database.handle(), started_unix_millis);

        Ok(Self {
            listener,
            service,
            database,
            socket,
            _lock: daemon_lock,
        })
    }

    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.socket.path
    }

    pub async fn serve_until<F>(self, shutdown: F) -> Result<(), DaemonError>
    where
        F: Future<Output = ()> + Send,
    {
        let Self {
            listener,
            service,
            database,
            mut socket,
            _lock: daemon_lock,
        } = self;
        let semaphore = Arc::new(Semaphore::new(MAX_CONNECTIONS));
        let mut connections: JoinSet<Result<(), FrameError>> = JoinSet::new();
        let mut serve_error = None;
        tokio::pin!(shutdown);

        loop {
            tokio::select! {
                biased;
                () = &mut shutdown => break,
                joined = connections.join_next(), if !connections.is_empty() => {
                    if let Some(result) = joined {
                        log_connection_result(result);
                    }
                }
                accepted = listener.accept() => {
                    let (stream, _address) = match accepted {
                        Ok(accepted) => accepted,
                        Err(error) => {
                            serve_error = Some(DaemonError::Ipc(error.to_string()));
                            break;
                        }
                    };
                    let Ok(permit) = Arc::clone(&semaphore).try_acquire_owned() else {
                        tracing::warn!("rejecting IPC connection because the limit is reached");
                        drop(stream);
                        continue;
                    };
                    let service = service.clone();
                    connections.spawn(async move {
                        let _permit = permit;
                        handle_connection(stream, service).await
                    });
                }
            }
        }

        // Unlinking first prevents new connects while leaving already accepted
        // streams alive for the bounded drain below.
        socket.remove();
        drop(listener);

        let drain = async {
            while let Some(result) = connections.join_next().await {
                log_connection_result(result);
            }
        };
        if timeout(DRAIN_TIMEOUT, drain).await.is_err() {
            connections.abort_all();
            while connections.join_next().await.is_some() {}
        }
        drop(service);
        let database_result = database.shutdown().await;
        drop(socket);
        drop(daemon_lock);
        database_result?;
        serve_error.map_or(Ok(()), Err)
    }
}

fn log_connection_result(result: Result<Result<(), FrameError>, tokio::task::JoinError>) {
    match result {
        Ok(Ok(())) => {}
        Ok(Err(error)) => tracing::debug!(%error, "IPC connection closed with error"),
        Err(error) => tracing::warn!(%error, "IPC connection task failed"),
    }
}

fn prepare_directories(config: &Config) -> Result<(), DaemonError> {
    for path in [
        &config.paths.data_dir,
        &config.paths.state_dir,
        &config.paths.run_dir,
    ] {
        fs::create_dir_all(path).map_err(|source| DaemonError::io(path, source))?;
        set_private_permissions(path, 0o700)?;
    }

    let socket_parent = config
        .paths
        .socket
        .parent()
        .ok_or(DaemonError::InvalidPath("socket path has no parent"))?;
    if !socket_parent.exists() {
        fs::create_dir_all(socket_parent)
            .map_err(|source| DaemonError::io(socket_parent, source))?;
        set_private_permissions(socket_parent, 0o700)?;
    }
    if !socket_parent.is_dir() {
        return Err(DaemonError::InvalidPath(
            "socket parent exists but is not a directory",
        ));
    }
    Ok(())
}

async fn recover_stale_socket(path: &Path) -> Result<(), DaemonError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(source) => return Err(DaemonError::io(path, source)),
    };
    if !metadata.file_type().is_socket() {
        return Err(DaemonError::SocketPathOccupied(path.to_path_buf()));
    }

    match timeout(SOCKET_PROBE_TIMEOUT, UnixStream::connect(path)).await {
        Ok(Ok(stream)) => {
            drop(stream);
            Err(DaemonError::AlreadyRunning(path.to_path_buf()))
        }
        Ok(Err(error))
            if matches!(
                error.kind(),
                io::ErrorKind::ConnectionRefused | io::ErrorKind::NotFound
            ) =>
        {
            fs::remove_file(path).map_err(|source| DaemonError::io(path, source))?;
            Ok(())
        }
        Ok(Err(source)) => Err(DaemonError::io(path, source)),
        Err(_) => Err(DaemonError::AlreadyRunning(path.to_path_buf())),
    }
}

fn unix_millis() -> Result<u64, DaemonError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| DaemonError::Ipc(error.to_string()))?;
    u64::try_from(duration.as_millis()).map_err(|error| DaemonError::Ipc(error.to_string()))
}

fn set_private_permissions(path: &Path, mode: u32) -> Result<(), DaemonError> {
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
        .map_err(|source| DaemonError::io(path, source))
}

#[derive(Debug)]
struct SocketGuard {
    path: PathBuf,
    armed: bool,
}

impl SocketGuard {
    fn new(path: PathBuf) -> Self {
        Self { path, armed: true }
    }

    fn remove(&mut self) {
        if !self.armed {
            return;
        }
        match fs::remove_file(&self.path) {
            Ok(()) => self.armed = false,
            Err(error) if error.kind() == io::ErrorKind::NotFound => self.armed = false,
            Err(error) => {
                tracing::warn!(path = %self.path.display(), %error, "failed to remove daemon socket");
            }
        }
    }
}

impl Drop for SocketGuard {
    fn drop(&mut self) {
        self.remove();
    }
}
