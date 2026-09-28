//! Where a packaged app's own files are, and how its daemon is started.
//!
//! A development checkout runs with `just app`, which points the daemon at
//! the checkout's sidecars and manifests through the environment. A packaged
//! app has no checkout: its resources (fonts, emoji, model manifests and the
//! components' packages) are in the bundle's resource folder, and its pinned
//! executables (the daemon, FFmpeg, FFprobe and uv) sit beside this one. The
//! component list is what marks a bundle as one; without it the shell behaves
//! exactly as in development.

use std::path::PathBuf;

use tauri::Manager as _;

use crate::daemon::Launch;

/// A packaged app's two folders.
#[derive(Debug, Clone)]
pub struct Layout {
    /// The bundle's `resources` folder.
    pub resources: PathBuf,
    /// The folder of the executables that ship beside this one.
    pub sidecars: PathBuf,
}

impl Layout {
    /// The layout of the bundle this shell runs from, when it runs from one.
    pub fn find(app: &tauri::AppHandle) -> Option<Self> {
        let resources = app.path().resource_dir().ok()?.join("resources");
        if !resources.join("engine").join("engine.json").is_file() {
            return None;
        }
        let sidecars = std::env::current_exe().ok()?.parent()?.to_path_buf();
        Some(Self {
            resources,
            sidecars,
        })
    }

    /// A pinned executable that ships beside the shell.
    fn sidecar(&self, name: &str) -> PathBuf {
        self.sidecars
            .join(format!("{name}{}", std::env::consts::EXE_SUFFIX))
    }

    /// How the daemon is started from this bundle: its resources, and the
    /// pinned FFprobe (FFmpeg beside it) and uv it ships with.
    pub fn launch(&self) -> Launch {
        Launch {
            arguments: vec![
                "--resources".into(),
                self.resources.clone().into_os_string(),
                "--ffprobe".into(),
                self.sidecar("ffprobe").into_os_string(),
                "--uv".into(),
                self.sidecar("uv").into_os_string(),
            ],
            packaged: true,
        }
    }

    pub fn fonts(&self) -> PathBuf {
        self.resources.join("fonts")
    }

    pub fn emoji(&self) -> PathBuf {
        self.resources.join("emoji")
    }
}
