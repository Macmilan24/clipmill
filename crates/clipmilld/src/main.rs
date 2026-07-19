#[cfg(unix)]
mod unix_main {
    use std::{path::PathBuf, process::ExitCode, time::Duration};

    use clap::Parser;
    use clipmilld::{Config, Daemon};
    use tracing_subscriber::EnvFilter;

    #[derive(Debug, Parser)]
    #[command(name = "clipmilld", version, about = "ClipMill local daemon")]
    struct Arguments {
        /// Root directory for durable ClipMill state.
        #[arg(long)]
        data_dir: Option<PathBuf>,
        /// Override the Unix-domain socket path.
        #[arg(long)]
        socket: Option<PathBuf>,
        /// Retain unreachable artifacts for this duration before collection.
        #[arg(long, value_parser = humantime::parse_duration)]
        artifact_gc_grace: Option<Duration>,
    }

    pub(crate) fn main() -> ExitCode {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        tracing_subscriber::fmt().with_env_filter(filter).init();

        let arguments = Arguments::parse();
        let runtime = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime,
            Err(error) => {
                eprintln!("clipmilld: cannot start async runtime: {error}");
                return ExitCode::FAILURE;
            }
        };
        match runtime.block_on(run(arguments)) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("clipmilld: {error}");
                ExitCode::FAILURE
            }
        }
    }

    async fn run(arguments: Arguments) -> Result<(), clipmilld::DaemonError> {
        let config = Config::resolve_with_gc(
            arguments.data_dir,
            arguments.socket,
            arguments.artifact_gc_grace,
        )?;
        let daemon = Daemon::start(config).await?;
        tracing::info!(socket = %daemon.socket_path().display(), "ClipMill daemon ready");
        daemon.serve_until(shutdown_signal()).await
    }

    async fn shutdown_signal() {
        let control_c = tokio::signal::ctrl_c();
        let terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate());
        match terminate {
            Ok(mut terminate) => {
                tokio::select! {
                    result = control_c => {
                        if let Err(error) = result {
                            tracing::warn!(%error, "failed to listen for Ctrl-C");
                        }
                    }
                    signal = terminate.recv() => {
                        if signal.is_none() {
                            tracing::warn!("SIGTERM listener closed unexpectedly");
                        }
                    }
                }
            }
            Err(error) => {
                tracing::warn!(%error, "failed to install SIGTERM listener");
                if let Err(error) = control_c.await {
                    tracing::warn!(%error, "failed to listen for Ctrl-C");
                }
            }
        }
    }
}

#[cfg(unix)]
fn main() -> std::process::ExitCode {
    unix_main::main()
}

#[cfg(not(unix))]
fn main() -> std::process::ExitCode {
    eprintln!("clipmilld: Windows named-pipe support has not landed yet");
    std::process::ExitCode::FAILURE
}
