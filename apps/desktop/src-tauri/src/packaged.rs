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

/// AppImage remounts its payload at a new path on each launch. Settings uses
/// the original image, which can reopen its own CLI for a connected client.
#[cfg(target_os = "linux")]
pub(crate) fn appimage_path(executable: &std::path::Path) -> Option<PathBuf> {
    let mount = PathBuf::from(std::env::var_os("APPDIR")?);
    let image = PathBuf::from(std::env::var_os("APPIMAGE")?);
    installed_image(executable, &mount, image)
}

#[cfg(any(target_os = "linux", test))]
fn installed_image(
    executable: &std::path::Path,
    mount: &std::path::Path,
    image: PathBuf,
) -> Option<PathBuf> {
    (mount.is_absolute() && executable.starts_with(mount) && image.is_absolute() && image.is_file())
        .then_some(image)
}

#[cfg(target_os = "linux")]
pub(crate) fn forward_appimage_cli() -> std::io::Result<()> {
    use std::os::unix::process::CommandExt as _;

    let mut args = std::env::args_os().skip(1);
    let Some(first) = args.next() else {
        return Ok(());
    };
    if first != "mcp" && first != "--cli" {
        return Ok(());
    }
    let executable = std::env::current_exe()?;
    if appimage_path(&executable).is_none() {
        return Ok(());
    }
    let mut cli = std::process::Command::new(executable.with_file_name("clipmill"));
    if first == "mcp" {
        cli.arg(first);
    }
    if let Some(original_directory) = std::env::var_os("OWD") {
        cli.current_dir(original_directory);
    }
    // Keep stdin/stdout, signals and the client's exact argument boundaries.
    Err(cli.args(args).exec())
}

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
        // On Windows Tauri gives this folder in its `\\?\` form. The daemon
        // passes it on to uv, Python and FFmpeg, so use the ordinary form.
        let resources = dunce::simplified(&app.path().resource_dir().ok()?).join("resources");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_clients_keep_the_same_command_across_distinct_mounts() -> std::io::Result<()> {
        let root = std::env::temp_dir();
        let image = root.join(format!("ClipMill portable {}.AppImage", ulid::Ulid::new()));
        std::fs::write(&image, b"fixture")?;
        for folder in ["first mount", "second mount"] {
            let mount = root.join(folder);
            let executable = mount.join("usr/bin/clipmill-studio");
            assert_eq!(
                installed_image(&executable, &mount, image.clone()),
                Some(image.clone())
            );
            // An installed Studio can inherit another AppImage's environment.
            assert!(
                installed_image(&root.join("usr/bin/clipmill-studio"), &mount, image.clone())
                    .is_none()
            );
        }
        std::fs::remove_file(image)
    }
}
