//! What differs between the systems the daemon runs on, kept in one place so
//! the rest of the daemon reads the same everywhere.
//!
//! **Privacy.** On Unix every directory the daemon makes is 0700 and every
//! file it writes 0600. On Windows the data directory is in the user's local
//! application data, whose inherited access list admits only the user,
//! SYSTEM and administrators, so the functions here that tighten access leave
//! Windows files as they are rather than rewrite that list.
//!
//! **Subprocesses.** A tool the daemon runs gets a process group of its own,
//! so stopping it stops everything it started: on Unix the group is signalled;
//! on Windows the process tree is ended, and no console window opens.
//!
//! **Files.** A source is identified by its device and inode on Unix, and by
//! its volume serial number and file index on Windows.

use std::{
    fs::{DirBuilder, File, FileType, Metadata, OpenOptions},
    io,
    path::Path,
    process::{Command, Stdio},
};

/// Tighten a directory the daemon made to its user alone.
#[cfg_attr(not(unix), allow(clippy::unnecessary_wraps))]
pub(crate) fn restrict_dir(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(())
    }
}

/// Make the names in a directory durable, as a rename into it is not until
/// the directory itself is flushed. Windows cannot open a directory as a file
/// to flush it, and NTFS journals the names a directory holds.
#[cfg_attr(windows, allow(clippy::unnecessary_wraps))]
pub(crate) fn sync_dir(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        File::open(path)?.sync_all()
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(())
    }
}

/// Flush a file the daemon has finished writing and closed. Windows flushes
/// only through a handle that may write, so it is opened for writing there;
/// Unix flushes a read-only descriptor as well.
pub(crate) fn sync_file(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        File::open(path)?.sync_all()
    }
    #[cfg(not(unix))]
    {
        OpenOptions::new().write(true).open(path)?.sync_all()
    }
}

/// Tighten a file the daemon wrote to its user alone.
#[cfg_attr(not(unix), allow(clippy::unnecessary_wraps))]
pub(crate) fn restrict_file(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(())
    }
}

/// Open options that create a file only its user can read.
pub(crate) fn private_file(options: &mut OpenOptions) -> &mut OpenOptions {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(0o600)
    }
    #[cfg(not(unix))]
    {
        options
    }
}

/// A directory builder that creates directories only their user can open.
pub(crate) fn private_dir(builder: &mut DirBuilder) -> &mut DirBuilder {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt as _;
        builder.mode(0o700)
    }
    #[cfg(not(unix))]
    {
        builder
    }
}

/// Open a file without following a link at its final component, and without
/// blocking on a FIFO the path turned into after it was checked.
pub(crate) fn no_follow(options: &mut OpenOptions) -> &mut OpenOptions {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.custom_flags(nix::libc::O_NONBLOCK | nix::libc::O_NOFOLLOW)
    }
    #[cfg(windows)]
    {
        no_follow_link(options)
    }
}

/// Open a file without following a link at its final component.
pub(crate) fn no_follow_link(options: &mut OpenOptions) -> &mut OpenOptions {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.custom_flags(nix::libc::O_NOFOLLOW)
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt as _;
        // FILE_FLAG_OPEN_REPARSE_POINT: open a symbolic link or junction
        // itself rather than what it points at.
        options.custom_flags(0x0020_0000)
    }
}

/// A file that is not data: a device, a FIFO or a socket. Windows has none a
/// path can name that `is_file` does not already refuse.
pub(crate) fn is_special(file_type: FileType) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt as _;
        file_type.is_block_device()
            || file_type.is_char_device()
            || file_type.is_fifo()
            || file_type.is_socket()
    }
    #[cfg(not(unix))]
    {
        let _ = file_type;
        false
    }
}

/// Whether a file's permissions let anyone but its owner read or change it.
/// Windows files in the daemon's data folder inherit that folder's private
/// access list, so there are no bits here to look at.
pub(crate) fn is_shared(metadata: &Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        metadata.permissions().mode() & 0o077 != 0
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        false
    }
}

/// Which file an open handle is, as (device, inode) on Unix and (volume
/// serial number, file index) on Windows. Equal keys are the same file.
// Only Windows can fail here: it asks the file system.
#[cfg_attr(unix, allow(clippy::unnecessary_wraps))]
pub(crate) fn file_key(file: &File, metadata: &Metadata) -> io::Result<(u64, u64)> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt as _;
        let _ = file;
        Ok((metadata.dev(), metadata.ino()))
    }
    #[cfg(windows)]
    {
        let _ = metadata;
        let information = winapi_util::file::information(file)?;
        Ok((information.volume_serial_number(), information.file_index()))
    }
}

/// Which file a path names now, in the terms of [`file_key`], given the
/// metadata just read through that path.
#[cfg_attr(unix, allow(clippy::unnecessary_wraps))]
pub(crate) fn path_key(path: &Path, metadata: &Metadata) -> io::Result<(u64, u64)> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt as _;
        let _ = path;
        Ok((metadata.dev(), metadata.ino()))
    }
    #[cfg(windows)]
    {
        let _ = metadata;
        let file = File::open(path)?;
        let information = winapi_util::file::information(&file)?;
        Ok((information.volume_serial_number(), information.file_index()))
    }
}

/// How many names the file at `path` has, given the metadata just read
/// through that path.
#[cfg_attr(unix, allow(clippy::unnecessary_wraps))]
pub(crate) fn link_count(path: &Path, metadata: &Metadata) -> io::Result<u64> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt as _;
        let _ = path;
        Ok(metadata.nlink())
    }
    #[cfg(windows)]
    {
        let _ = metadata;
        let file = File::open(path)?;
        Ok(winapi_util::file::information(&file)?.number_of_links())
    }
}

/// Start a tool in a group of its own, so [`end_group`] reaches everything it
/// starts. On Windows it also gets no console window.
pub(crate) fn own_group(command: &mut Command) -> &mut Command {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt as _;
        command.process_group(0)
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        command.creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW)
    }
}

/// [`own_group`] for a Tokio command.
pub(crate) fn own_group_async(
    command: &mut tokio::process::Command,
) -> &mut tokio::process::Command {
    #[cfg(unix)]
    {
        command.process_group(0)
    }
    #[cfg(windows)]
    {
        command.creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW)
    }
}

/// Start a console program without a console window. The daemon has none
/// of its own on Windows, so without this each tool it runs would open one.
pub(crate) fn no_console(command: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command
}

/// [`no_console`] for a Tokio command.
pub(crate) fn no_console_async(
    command: &mut tokio::process::Command,
) -> &mut tokio::process::Command {
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

/// Ask one process to finish. False where no such request exists (Windows),
/// so the caller ends it at once instead of waiting for nothing.
pub(crate) fn ask_to_stop(pid: u32) -> bool {
    #[cfg(unix)]
    {
        Command::new("/bin/kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .env_clear()
            .status()
            .is_ok()
    }
    #[cfg(not(unix))]
    {
        let _ = pid;
        false
    }
}

/// Start a command with no environment but what the system itself needs. On
/// Windows a program started with none cannot always load its libraries, so
/// the system root is kept.
pub(crate) fn clear_environment(command: &mut Command) -> &mut Command {
    command.env_clear();
    #[cfg(windows)]
    for key in ["SystemRoot", "WINDIR"] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    command
}

/// [`clear_environment`] for a Tokio command.
pub(crate) fn clear_environment_async(
    command: &mut tokio::process::Command,
) -> &mut tokio::process::Command {
    command.env_clear();
    #[cfg(windows)]
    for key in ["SystemRoot", "WINDIR"] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    command
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Stop {
    /// Ask the group to finish. Windows has no such request for a process
    /// without a window, so there it ends the tree as [`Stop::Kill`] does.
    Terminate,
    Kill,
}

/// Stop a group started with [`own_group`], by the id of its first process.
pub(crate) fn end_group(pid: u32, stop: Stop) {
    #[cfg(unix)]
    {
        let signal = match stop {
            Stop::Terminate => "-TERM",
            Stop::Kill => "-KILL",
        };
        let _status = Command::new("/bin/kill")
            .args([signal, "--", &format!("-{pid}")])
            .env_clear()
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        let _ = stop;
        let _status = Command::new("taskkill")
            .args(["/T", "/F", "/PID", &pid.to_string()])
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

/// [`end_group`] without blocking the thread that awaits it.
pub(crate) async fn end_group_async(pid: u32, stop: Stop) {
    let mut command =
        tokio::process::Command::new(if cfg!(unix) { "/bin/kill" } else { "taskkill" });
    #[cfg(unix)]
    command.args([
        match stop {
            Stop::Terminate => "-TERM",
            Stop::Kill => "-KILL",
        },
        "--",
        &format!("-{pid}"),
    ]);
    #[cfg(windows)]
    {
        let _ = stop;
        command
            .args(["/T", "/F", "/PID", &pid.to_string()])
            .creation_flags(CREATE_NO_WINDOW);
    }
    let _status = command
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;
}

#[cfg(windows)]
const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
