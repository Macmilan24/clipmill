#![cfg(unix)]

use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::Parser;
use clipmill_core::WorkerId;
use clipmilld::Config;
use ed25519_dalek::SigningKey;
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(
    name = "clipmill-worker-keygen",
    version,
    about = "Provision a local Phase 0 worker identity"
)]
struct Arguments {
    /// ClipMill data directory whose local trust store receives the public key.
    #[arg(long)]
    data_dir: Option<PathBuf>,
    /// Private identity output. Defaults under state/worker-identities.
    #[arg(long)]
    identity: Option<PathBuf>,
    /// Reuse a preallocated typed worker ID.
    #[arg(long)]
    worker_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct IdentityFile {
    key_version: &'static str,
    private_key: String,
    worker_id: String,
}

fn main() -> ExitCode {
    match run(Arguments::parse()) {
        Ok((worker_id, identity)) => {
            println!(
                "provisioned {worker_id}; private identity: {}",
                identity.display()
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("clipmill-worker-keygen: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(arguments: Arguments) -> Result<(String, PathBuf), String> {
    let config = Config::resolve(arguments.data_dir, None).map_err(|error| error.to_string())?;
    let worker_id = match arguments.worker_id {
        Some(worker_id) => worker_id
            .parse::<WorkerId>()
            .map_err(|error| error.to_string())?
            .to_string(),
        None => WorkerId::new().to_string(),
    };
    let identities = config.paths.state_dir.join("worker-identities");
    create_private_directory(&config.paths.state_dir)?;
    create_private_directory(&config.paths.worker_trust_dir)?;
    create_private_directory(&identities)?;
    let identity_path = arguments
        .identity
        .unwrap_or_else(|| identities.join(format!("{worker_id}.json")));
    let identity_parent = identity_path
        .parent()
        .ok_or_else(|| "identity path has no parent".to_owned())?;
    create_private_directory(identity_parent)?;

    let mut seed = [0_u8; 32];
    getrandom::fill(&mut seed).map_err(|error| error.to_string())?;
    let signing_key = SigningKey::from_bytes(&seed);
    let public_hex = hex::encode(signing_key.verifying_key().to_bytes());
    let identity = IdentityFile {
        key_version: "clipmill.worker.identity.v1",
        private_key: hex::encode(seed),
        worker_id: worker_id.clone(),
    };
    let mut identity_bytes =
        serde_json::to_vec_pretty(&identity).map_err(|error| error.to_string())?;
    identity_bytes.push(b'\n');
    write_new_private(&identity_path, &identity_bytes)?;
    let trust_path = config
        .paths
        .worker_trust_dir
        .join(format!("{worker_id}.pub"));
    write_new_private(&trust_path, format!("{public_hex}\n").as_bytes())?;
    sync_directory(identity_parent)?;
    sync_directory(&config.paths.worker_trust_dir)?;
    Ok((worker_id, identity_path))
}

fn create_private_directory(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|error| format!("{}: {error}", path.display()))?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|error| format!("{}: {error}", path.display()))
}

fn write_new_private(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut options = OpenOptions::new();
    options.create_new(true).write(true).mode(0o600);
    let mut file = options
        .open(path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    file.write_all(bytes)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("{}: {error}", path.display()))
}

fn sync_directory(path: &Path) -> Result<(), String> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| format!("{}: {error}", path.display()))
}
