// Release builds on Windows must not open a console window behind the app.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::ExitCode;

use tracing_subscriber::EnvFilter;

fn main() -> ExitCode {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    match clipmill_shell::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("clipmill-shell: {error}");
            ExitCode::FAILURE
        }
    }
}
