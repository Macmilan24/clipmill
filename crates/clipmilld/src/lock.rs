use std::{
    fs::{File, OpenOptions},
    io::{Seek, SeekFrom, Write},
    path::Path,
};

use fs2::FileExt;

use crate::DaemonError;

#[derive(Debug)]
pub(crate) struct DaemonLock {
    file: File,
}

impl DaemonLock {
    pub(crate) fn acquire(path: &Path) -> Result<Self, DaemonError> {
        #[cfg(unix)]
        use std::os::unix::fs::OpenOptionsExt;

        let mut options = OpenOptions::new();
        options.create(true).read(true).write(true).truncate(false);
        #[cfg(unix)]
        options.mode(0o600);

        let mut file = options
            .open(path)
            .map_err(|source| DaemonError::io(path, source))?;
        match FileExt::try_lock_exclusive(&file) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                return Err(DaemonError::AlreadyRunning(path.to_path_buf()));
            }
            Err(source) => return Err(DaemonError::io(path, source)),
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            file.set_permissions(std::fs::Permissions::from_mode(0o600))
                .map_err(|source| DaemonError::io(path, source))?;
        }

        file.set_len(0)
            .and_then(|()| file.seek(SeekFrom::Start(0)).map(|_| ()))
            .and_then(|()| writeln!(file, "{}", std::process::id()))
            .and_then(|()| file.sync_all())
            .map_err(|source| DaemonError::io(path, source))?;

        Ok(Self { file })
    }
}

impl Drop for DaemonLock {
    fn drop(&mut self) {
        let _result = FileExt::unlock(&self.file);
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use tempfile::TempDir;

    use super::DaemonLock;
    use crate::DaemonError;

    #[test]
    fn lock_is_exclusive_and_repairs_private_permissions() {
        let temp = TempDir::new().expect("tempdir");
        let path = temp.path().join("daemon.lock");
        std::fs::write(&path, b"stale pid\n").expect("seed lock file");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o666))
                .expect("loosen lock permissions");
        }

        let lock = DaemonLock::acquire(&path).expect("first lock");
        assert!(matches!(
            DaemonLock::acquire(&path),
            Err(DaemonError::AlreadyRunning(_))
        ));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mode = std::fs::metadata(&path)
                .expect("lock metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
        }
        drop(lock);
        DaemonLock::acquire(&path).expect("lock is released");
    }
}
