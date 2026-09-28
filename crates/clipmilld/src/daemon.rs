use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::{
    fs,
    future::Future,
    io::{self, Read},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use tokio::{
    net::{UnixListener, UnixStream},
    sync::{Semaphore, oneshot},
    task::JoinSet,
    time::timeout,
};

use clipmill_artifacts::ArtifactPath;

use crate::{
    ArtifactCoordinator, Config, DaemonError,
    artifacts::{ArtifactActor, ArtifactHandle},
    collector::{self, CleanUp, Collector},
    db::DbActor,
    device::{DeviceProfiler, verify_profile},
    ipc::{FrameError, handle_connection},
    jobs::{EventHub, ResourceCapacity, Scheduler},
    lock::DaemonLock,
    service::Service,
    shm::{ShmBroker, handle_shm_connection},
    sources::SourceInspector,
    worker::{WorkerError, WorkerService},
};

const MAX_CONNECTIONS: usize = 64;
const DRAIN_TIMEOUT: Duration = Duration::from_secs(5);
const SOCKET_PROBE_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Debug)]
pub struct Daemon {
    listener: UnixListener,
    worker_listener: UnixListener,
    shm_listener: UnixListener,
    service: Service,
    worker_service: WorkerService,
    artifacts: ArtifactActor,
    database: DbActor,
    scheduler: Scheduler,
    epoch: String,
    artifact_gc_grace: Duration,
    /// Clean-ups asked for through the service, answered by the collection
    /// loop `serve_until` starts.
    clean_ups: tokio::sync::mpsc::Receiver<CleanUp>,
    socket: SocketGuard,
    worker_socket: SocketGuard,
    shm_socket: SocketGuard,
    _lock: DaemonLock,
}

impl Daemon {
    #[allow(clippy::too_many_lines)]
    pub async fn start(config: Config) -> Result<Self, DaemonError> {
        let started_unix_millis = unix_millis()?;
        prepare_directories(&config)?;
        let daemon_lock = DaemonLock::acquire(&config.paths.lock)?;
        let database = DbActor::start(&config.paths.database, &config.paths.backups_dir)?;
        let (artifacts, recovery) = match ArtifactActor::start(&config.paths.artifacts_dir) {
            Ok(value) => value,
            Err(error) => {
                database.shutdown().await?;
                return Err(error);
            }
        };
        tracing::info!(
            committed = recovery.committed_loaded,
            legacy = recovery.legacy_loaded,
            staging_quarantined = recovery.staging_quarantined,
            objects_quarantined = recovery.objects_quarantined,
            "artifact store recovered"
        );
        let sources = match SourceInspector::new(
            config.ffprobe.clone(),
            config.paths.probe_scratch_dir.clone(),
        ) {
            Ok(sources) => sources,
            Err(error) => {
                artifacts.shutdown().await?;
                database.shutdown().await?;
                return Err(DaemonError::Ipc(format!(
                    "cannot initialize source inspector: {error}"
                )));
            }
        };
        let media_runner = match crate::media::MediaRunner::new(
            config.ffprobe.clone(),
            config.paths.media_scratch_dir.clone(),
        ) {
            Ok(runner) => runner,
            Err(error) => {
                artifacts.shutdown().await?;
                database.shutdown().await?;
                return Err(DaemonError::Ipc(format!(
                    "cannot initialize media runner: {error}"
                )));
            }
        };
        // Read once at startup: the registry is pinned configuration, and a
        // daemon that re-read it per task could key two artifacts of one job
        // against different model identities. It has to exist before the
        // profiler, which measures nothing about a model it cannot identify.
        let models = Arc::new(
            match crate::models::ModelRegistry::load_with_custom(
                &config.models_dir,
                &config.paths.custom_models_dir,
            ) {
                Ok(registry) => {
                    for summary in registry.summaries() {
                        tracing::info!(model = summary, "pinned model");
                    }
                    tracing::info!(
                        models = registry.len(),
                        licenses = ?registry.license_classes(),
                        "pinned model registry loaded"
                    );
                    registry
                }
                Err(error) => {
                    tracing::error!(%error, "pinned model registry is unreadable");
                    artifacts.shutdown().await?;
                    database.shutdown().await?;
                    return Err(DaemonError::ModelRegistry(error.to_string()));
                }
            },
        );
        let device_profiler = match DeviceProfiler::new(
            &config.ffprobe,
            &config.paths.device_attestation_key,
            &config.paths.device_profile_scratch_dir,
            &config.paths.speech_benchmark,
            Arc::clone(&models),
        ) {
            Ok(profiler) => profiler,
            Err(error) => {
                artifacts.shutdown().await?;
                database.shutdown().await?;
                return Err(DaemonError::Ipc(format!(
                    "cannot initialize device profiler: {error}"
                )));
            }
        };
        let live_scheduler_capacity = match device_profiler.scheduler_capacity().await {
            Ok(capacity) => capacity,
            Err(error) => {
                artifacts.shutdown().await?;
                database.shutdown().await?;
                return Err(DaemonError::Ipc(format!(
                    "cannot measure scheduler capacity: {error}"
                )));
            }
        };
        let hardware_fingerprint = match device_profiler.hardware_fingerprint().await {
            Ok(fingerprint) => fingerprint,
            Err(error) => {
                artifacts.shutdown().await?;
                database.shutdown().await?;
                return Err(DaemonError::Ipc(format!(
                    "cannot fingerprint scheduler device: {error}"
                )));
            }
        };
        let scheduler_capacity = startup_profile_capacity(
            &database.handle(),
            &artifacts.handle(),
            &hardware_fingerprint,
            live_scheduler_capacity,
        )
        .await;
        let daemon_epoch = ulid::Ulid::new().to_string();
        let events = EventHub::new();
        match database
            .handle()
            .recover_jobs(daemon_epoch.clone(), started_unix_millis)
            .await
        {
            Ok(recovered) => events.publish_all(recovered),
            Err(error) => {
                artifacts.shutdown().await?;
                database.shutdown().await?;
                return Err(DaemonError::Ipc(format!(
                    "cannot recover durable jobs: {error}"
                )));
            }
        }

        let socket_result = async {
            let control = bind_private_socket(config.paths.socket.clone()).await?;
            let worker = bind_private_socket(config.paths.worker_socket.clone()).await?;
            let shm = bind_private_socket(config.paths.shm_socket.clone()).await?;
            Ok::<_, DaemonError>((control, worker, shm))
        }
        .await;
        let ((listener, socket), (worker_listener, worker_socket), (shm_listener, shm_socket)) =
            match socket_result {
                Ok(sockets) => sockets,
                Err(error) => {
                    artifacts.shutdown().await?;
                    database.shutdown().await?;
                    return Err(error);
                }
            };
        // Read before the runner is handed to the scheduler. The one leased
        // stage that decodes video is given this path on its lease rather than
        // finding a decoder itself.
        let decoder = media_runner.ffmpeg().to_path_buf();
        // One counter for the whole process, offered every task either lease
        // path starts, and read by Health and by Settings.
        let policy = Arc::new(crate::policy::LocalLockPolicy::new());
        // A packaged app's engine enrols its workers' keys here, before the
        // worker service reads the keys it trusts.
        let engine = config.bundle.as_ref().map(|bundle| {
            crate::engine::Engine::prepare(
                crate::engine::EnginePaths {
                    uv: bundle.uv.clone(),
                    shipped: bundle.resources_dir.join("engine"),
                    engine_dir: config.paths.engine_dir.clone(),
                    logs_dir: config.paths.logs_dir.clone(),
                    identity_dir: config.paths.worker_identity_dir.clone(),
                    trust_dir: config.paths.worker_trust_dir.clone(),
                    worker_socket: config.paths.worker_socket.clone(),
                    shm_socket: config.paths.shm_socket.clone(),
                },
                Arc::clone(&policy),
            )
        });
        let scheduler = Scheduler::start(
            database.handle(),
            artifacts.handle(),
            events.clone(),
            daemon_epoch.clone(),
            sources.clone(),
            device_profiler.clone(),
            media_runner.clone(),
            crate::render::RenderResources {
                fonts: config.fonts_dir.clone(),
                assets: config.paths.assets_dir.clone(),
                emoji: config.emoji_dir.clone(),
            },
            Arc::clone(&models),
            scheduler_capacity,
            config.builtin_fixture_executor,
            Arc::clone(&policy),
        );
        let shm = ShmBroker::default();
        let roster = crate::worker::new_roster();
        let worker_service = match WorkerService::new(
            database.handle(),
            artifacts.handle(),
            events.clone(),
            scheduler.handle(),
            daemon_epoch.clone(),
            &config.paths.worker_trust_dir,
            shm,
            Arc::clone(&models),
            config.paths.artifacts_dir.clone(),
            config.weights_dir.clone(),
            decoder.clone(),
            Arc::clone(&policy),
            Arc::clone(&roster),
        ) {
            Ok(service) => service,
            Err(error) => {
                scheduler.shutdown().await;
                artifacts.shutdown().await?;
                database.shutdown().await?;
                return Err(DaemonError::Ipc(format!(
                    "cannot initialize worker trust: {error}"
                )));
            }
        };
        if !socket.path.exists() || !worker_socket.path.exists() || !shm_socket.path.exists() {
            scheduler.shutdown().await;
            artifacts.shutdown().await?;
            database.shutdown().await?;
            return Err(DaemonError::Ipc(
                "one or more daemon sockets disappeared during startup".to_owned(),
            ));
        }
        let library = crate::library::ModelLibrary::start(
            Arc::clone(&models),
            crate::library::LibraryPaths {
                weights: config.weights_dir.clone(),
                custom: config.paths.custom_models_dir.clone(),
                choices: config.paths.model_choices.clone(),
            },
            Some(database.handle()),
            Arc::clone(&policy),
            crate::device::total_memory().await,
        );
        let (collector, clean_ups) = Collector::new();
        let service = Service::with_scheduler(
            database.handle(),
            started_unix_millis,
            events,
            scheduler.handle(),
            sources,
            artifacts.handle(),
            device_profiler,
            Arc::clone(&models),
            crate::storage::StorageDirs {
                data: config.paths.data_dir.clone(),
                artifacts: config.paths.artifacts_dir.clone(),
                state: config.paths.state_dir.clone(),
                weights: config.weights_dir.clone(),
                backups: config.paths.backups_dir.clone(),
                scratch: vec![
                    config.paths.probe_scratch_dir.clone(),
                    config.paths.media_scratch_dir.clone(),
                    config.paths.device_profile_scratch_dir.clone(),
                ],
            },
            config.artifact_gc_grace,
            Arc::clone(&policy),
            roster,
            decoder,
        )
        .with_library(library)
        .with_collector(collector)
        .with_fonts(config.fonts_dir.clone());
        let service = match engine {
            Some(engine) => service.with_engine(engine),
            None => service,
        };
        // The person's own pictures and sounds. A folder that cannot be made
        // leaves the rest of the daemon working; bringing one in says why.
        let service = match crate::assets::AssetStore::new(
            config.paths.assets_dir.clone(),
            media_runner.clone(),
        ) {
            Ok(store) => service.with_assets(store),
            Err(error) => {
                tracing::warn!(%error, "no asset folder");
                service
            }
        };

        service.recover_youtube_imports().await.map_err(|error| {
            DaemonError::Ipc(format!("cannot recover YouTube imports: {error}"))
        })?;
        service
            .recover_youtube_publishing()
            .await
            .map_err(|error| {
                DaemonError::Ipc(format!("cannot recover YouTube publishing: {error}"))
            })?;
        service.resume_export_batches().await;
        Ok(Self {
            listener,
            worker_listener,
            shm_listener,
            service,
            worker_service,
            artifacts,
            database,
            scheduler,
            epoch: daemon_epoch,
            artifact_gc_grace: config.artifact_gc_grace,
            clean_ups,
            socket,
            worker_socket,
            shm_socket,
            _lock: daemon_lock,
        })
    }

    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.socket.path
    }

    #[must_use]
    pub fn worker_socket_path(&self) -> &Path {
        &self.worker_socket.path
    }

    #[must_use]
    pub fn shm_socket_path(&self) -> &Path {
        &self.shm_socket.path
    }

    #[must_use]
    pub fn artifact_coordinator(&self) -> ArtifactCoordinator {
        ArtifactCoordinator::new(self.artifacts.handle(), self.database.handle())
    }

    /// The document a command log started from, plus every logged step.
    ///
    /// This is an in-process lifecycle interface for archival and for drills
    /// that must verify replay against a restarted daemon; like the artifact
    /// coordinator it is deliberately not part of the control API.
    pub async fn edit_log(&self, doc_id: String) -> Result<EditLog, DaemonError> {
        let (initial_document, entries) = self
            .database
            .handle()
            .get_edit_log(doc_id)
            .await
            .map_err(|error| DaemonError::Ipc(error.to_string()))?;
        Ok(EditLog {
            initial_document,
            commands: entries
                .into_iter()
                .map(|entry| (entry.revision, entry.command_json, entry.inverse_json))
                .collect(),
        })
    }

    #[allow(clippy::too_many_lines)]
    pub async fn serve_until<F>(self, shutdown: F) -> Result<(), DaemonError>
    where
        F: Future<Output = ()> + Send,
    {
        let Self {
            listener,
            worker_listener,
            shm_listener,
            service,
            worker_service,
            artifacts,
            database,
            scheduler,
            epoch: daemon_epoch,
            artifact_gc_grace,
            clean_ups,
            mut socket,
            mut worker_socket,
            mut shm_socket,
            _lock: daemon_lock,
        } = self;
        let semaphore = Arc::new(Semaphore::new(MAX_CONNECTIONS));
        let worker_semaphore = Arc::new(Semaphore::new(MAX_CONNECTIONS));
        let shm_semaphore = Arc::new(Semaphore::new(MAX_CONNECTIONS));
        let mut connections: JoinSet<Result<(), FrameError>> = JoinSet::new();
        let mut worker_connections: JoinSet<Result<(), WorkerError>> = JoinSet::new();
        let mut shm_connections: JoinSet<Result<(), String>> = JoinSet::new();
        let mut serve_error = None;
        let (maintenance_stop, maintenance_stopped) = oneshot::channel();
        let mut maintenance = tokio::spawn(collector::run(
            artifacts.handle(),
            database.handle(),
            artifact_gc_grace,
            clean_ups,
            maintenance_stopped,
        ));
        tokio::pin!(shutdown);
        let requested = service.shutdown_requested();
        // The engine's workers connect to the socket this loop accepts on;
        // until it runs they wait in the listener's backlog or retry.
        if let Some(engine) = service.engine() {
            engine.start();
        }

        loop {
            tokio::select! {
                biased;
                () = &mut shutdown => break,
                () = requested.notified() => break,
                joined = connections.join_next(), if !connections.is_empty() => {
                    if let Some(result) = joined {
                        log_connection_result(result);
                    }
                }
                joined = worker_connections.join_next(), if !worker_connections.is_empty() => {
                    if let Some(result) = joined {
                        log_worker_result(result);
                    }
                }
                joined = shm_connections.join_next(), if !shm_connections.is_empty() => {
                    if let Some(result) = joined {
                        log_shm_result(result);
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
                accepted = worker_listener.accept() => {
                    let (stream, _address) = match accepted {
                        Ok(accepted) => accepted,
                        Err(error) => {
                            serve_error = Some(DaemonError::Ipc(error.to_string()));
                            break;
                        }
                    };
                    let Ok(permit) = Arc::clone(&worker_semaphore).try_acquire_owned() else {
                        tracing::warn!("rejecting worker connection because the limit is reached");
                        drop(stream);
                        continue;
                    };
                    let worker_service = worker_service.clone();
                    worker_connections.spawn(async move {
                        let _permit = permit;
                        worker_service.handle_connection(stream).await
                    });
                }
                accepted = shm_listener.accept() => {
                    let (stream, _address) = match accepted {
                        Ok(accepted) => accepted,
                        Err(error) => {
                            serve_error = Some(DaemonError::Ipc(error.to_string()));
                            break;
                        }
                    };
                    let Ok(permit) = Arc::clone(&shm_semaphore).try_acquire_owned() else {
                        tracing::warn!("rejecting shared-memory connection because the limit is reached");
                        drop(stream);
                        continue;
                    };
                    let worker_service = worker_service.clone();
                    shm_connections.spawn(async move {
                        let _permit = permit;
                        let stream = stream.into_std().map_err(|error| error.to_string())?;
                        tokio::task::spawn_blocking(move || {
                            handle_shm_connection(stream, &worker_service.shm_broker())
                                .map_err(|error| error.to_string())
                        })
                        .await
                        .map_err(|error| error.to_string())?
                    });
                }
            }
        }

        // Unlinking first prevents new connects while leaving already accepted
        // streams alive for the bounded drain below.
        socket.remove();
        worker_socket.remove();
        shm_socket.remove();
        drop(listener);
        drop(worker_listener);
        drop(shm_listener);
        worker_service.stop_scheduling();
        // Ask the daemon's own workers to leave now, so each releases its
        // lease over a connection the drain below still serves.
        if let Some(engine) = service.engine() {
            engine.stop().await;
        }
        service.stop_youtube_imports().await;
        service.stop_youtube_publishing().await;
        service.stop_model_library().await;
        scheduler.shutdown().await;
        let _stop_sent = maintenance_stop.send(());
        if timeout(DRAIN_TIMEOUT, &mut maintenance).await.is_err() {
            maintenance.abort();
            let _joined = maintenance.await;
        }

        let drain = async {
            loop {
                if connections.is_empty()
                    && worker_connections.is_empty()
                    && shm_connections.is_empty()
                {
                    break;
                }
                tokio::select! {
                    joined = connections.join_next(), if !connections.is_empty() => {
                        if let Some(result) = joined { log_connection_result(result); }
                    }
                    joined = worker_connections.join_next(), if !worker_connections.is_empty() => {
                        if let Some(result) = joined { log_worker_result(result); }
                    }
                    joined = shm_connections.join_next(), if !shm_connections.is_empty() => {
                        if let Some(result) = joined { log_shm_result(result); }
                    }
                }
            }
        };
        if timeout(DRAIN_TIMEOUT, drain).await.is_err() {
            connections.abort_all();
            worker_connections.abort_all();
            shm_connections.abort_all();
            while connections.join_next().await.is_some() {}
            while worker_connections.join_next().await.is_some() {}
            while shm_connections.join_next().await.is_some() {}
        }
        if let Ok(recovered) = database
            .handle()
            .recover_jobs(daemon_epoch, unix_millis().unwrap_or(u64::MAX))
            .await
        {
            service.event_hub().publish_all(recovered);
        }
        drop(worker_service);
        drop(service);
        let artifact_result = artifacts.shutdown().await;
        let database_result = database.shutdown().await;
        drop(socket);
        drop(worker_socket);
        drop(shm_socket);
        drop(daemon_lock);
        artifact_result?;
        database_result?;
        serve_error.map_or(Ok(()), Err)
    }
}

async fn startup_profile_capacity(
    database: &crate::db::DbHandle,
    artifacts: &ArtifactHandle,
    hardware_fingerprint: &str,
    live: ResourceCapacity,
) -> ResourceCapacity {
    let Ok(Some(record)) = database
        .current_device_profile(hardware_fingerprint.to_owned())
        .await
    else {
        return live;
    };
    let (Some(profile_json), Some(artifact_id)) = (record.profile_json, record.artifact_id) else {
        tracing::warn!("active device profile omitted durable payload state");
        return live;
    };
    let Ok(verified) = verify_profile(&profile_json, Some(hardware_fingerprint)) else {
        tracing::warn!(%artifact_id, "active device profile signature is invalid");
        return live;
    };
    let Ok(lease) = artifacts.open(artifact_id).await else {
        tracing::warn!(%artifact_id, "active device profile artifact is unavailable");
        return live;
    };
    let Ok(path) = "profile.json".parse::<ArtifactPath>() else {
        return live;
    };
    let mut artifact_profile = String::new();
    let verified_payload = lease
        .open_verified(&path)
        .ok()
        .and_then(|mut file| file.read_to_string(&mut artifact_profile).ok())
        .is_some()
        && artifact_profile == profile_json;
    if !verified_payload {
        tracing::warn!(%artifact_id, "active device profile artifact failed verification");
        return live;
    }
    // Disk comes from the live measurement, not the profile: a profile records
    // what a machine *is*, and free space is what it *has right now*.
    let measured = ResourceCapacity::for_device(
        &verified.platform_os,
        verified.logical_cores,
        verified.total_memory_bytes,
        verified.available_memory_bytes,
        live.disk_bytes,
    )
    .with_available_backends(&verified.available_backends);
    tracing::info!(
        %artifact_id,
        generation = verified.measurement_generation,
        backends = ?verified.available_backends,
        "scheduler consumed verified cached device profile"
    );
    ResourceCapacity {
        cpu_threads: measured.cpu_threads.min(live.cpu_threads).max(1),
        ram_bytes: measured.ram_bytes.min(live.ram_bytes),
        disk_bytes: measured.disk_bytes.min(live.disk_bytes),
        accelerator_mask: measured.accelerator_mask,
        vram_bytes: measured.vram_bytes,
    }
}

fn log_connection_result(result: Result<Result<(), FrameError>, tokio::task::JoinError>) {
    match result {
        Ok(Ok(())) => {}
        Ok(Err(error)) => tracing::debug!(%error, "IPC connection closed with error"),
        Err(error) => tracing::warn!(%error, "IPC connection task failed"),
    }
}

fn log_worker_result(result: Result<Result<(), WorkerError>, tokio::task::JoinError>) {
    match result {
        Ok(Ok(())) => {}
        Ok(Err(error)) => {
            tracing::debug!(result_code = error.code(), "worker connection closed");
        }
        Err(_error) => tracing::warn!(result_code = "TASK_FAILED", "worker connection task failed"),
    }
}

fn log_shm_result(result: Result<Result<(), String>, tokio::task::JoinError>) {
    match result {
        Ok(Ok(())) => {}
        Ok(Err(_error)) => {
            tracing::debug!(
                result_code = "PROTOCOL_ERROR",
                "shared-memory connection closed"
            );
        }
        Err(_error) => tracing::warn!(
            result_code = "TASK_FAILED",
            "shared-memory connection task failed"
        ),
    }
}

fn prepare_directories(config: &Config) -> Result<(), DaemonError> {
    for path in [
        &config.paths.data_dir,
        &config.paths.state_dir,
        &config.paths.backups_dir,
        &config.paths.artifacts_dir,
        &config.paths.probe_scratch_dir,
        &config.paths.media_scratch_dir,
        &config.paths.device_profile_scratch_dir,
        &config.paths.worker_trust_dir,
        &config.paths.custom_models_dir,
        &config.paths.run_dir,
    ] {
        fs::create_dir_all(path).map_err(|source| DaemonError::io(path, source))?;
        set_private_permissions(path, 0o700)?;
    }

    for socket in [
        &config.paths.socket,
        &config.paths.worker_socket,
        &config.paths.shm_socket,
    ] {
        let socket_parent = socket
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
    }
    Ok(())
}

async fn bind_private_socket(path: PathBuf) -> Result<(UnixListener, SocketGuard), DaemonError> {
    recover_stale_socket(&path).await?;
    let listener = UnixListener::bind(&path).map_err(|source| DaemonError::io(&path, source))?;
    let socket = SocketGuard::new(path);
    set_private_permissions(&socket.path, 0o600)?;
    Ok((listener, socket))
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

/// An edit document's durable history: the document it started from and every
/// logged `(revision, command, inverse)` step.
#[derive(Clone, Debug)]
pub struct EditLog {
    pub initial_document: String,
    pub commands: Vec<(u64, String, String)>,
}
