//! Shared private-file policy for the store and descriptor-owned acquisition lock.
#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt};
use std::{
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Identity {
    device: u64,
    inode: u64,
}
fn identity(metadata: &fs::Metadata) -> Identity {
    #[cfg(unix)]
    {
        Identity {
            device: metadata.dev(),
            inode: metadata.ino(),
        }
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        Identity {
            device: 0,
            inode: 0,
        }
    }
}
pub(crate) fn absolute(path: &Path) -> Result<PathBuf, String> {
    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path)
    };
    if path.file_name().is_none() {
        return Err("A private file path must have a file name".into());
    }
    Ok(path)
}

pub(crate) fn check_sqlite_sidecars(database: &Path) -> Result<(), String> {
    for ending in ["-wal", "-shm", "-journal"] {
        let path = suffix(database, ending);
        check_sidecar_metadata(|| fs::symlink_metadata(&path))?;
    }
    Ok(())
}

/// SQLite may unlink a WAL/SHM inode after lstat resolved it but before the
/// kernel copies its metadata. Only that otherwise-private zero-link snapshot
/// is transient. Recheck names using metadata alone, never sidecar descriptors.
fn check_sidecar_metadata(
    mut inspect: impl FnMut() -> std::io::Result<fs::Metadata>,
) -> Result<(), String> {
    for _ in 0..4 {
        match inspect() {
            Ok(metadata) if removed_private_sidecar(&metadata) => std::thread::yield_now(),
            Ok(metadata) => return check_metadata(&metadata, "SQLite store sidecar").map(|_| ()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("SQLite store sidecar did not stabilise after unlink".into())
}
fn removed_private_sidecar(metadata: &fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        metadata.file_type().is_file()
            && metadata.uid() == rustix::process::getuid().as_raw()
            && metadata.mode() & 0o777 == 0o600
            && metadata.nlink() == 0
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        false
    }
}

/// Validate SQLite-owned paths without opening another descriptor. Closing any
/// descriptor for an inode releases this process's POSIX locks on it, including
/// SQLite's database and WAL shared-memory locks held by other connections.
/// See https://sqlite.org/howtocorrupt.html#posix_close_bug.
pub(crate) fn check_sqlite_file(
    path: &Path,
    expected: Option<Identity>,
    label: &str,
) -> Result<Identity, String> {
    let metadata = fs::symlink_metadata(path).map_err(|e| format!("{label}: {e}"))?;
    let found = check_metadata(&metadata, label)?;
    if expected.is_some_and(|expected| found != expected) {
        return Err(format!("{label} was replaced"));
    }
    Ok(found)
}

/// Create an absent database privately, or validate an existing database using
/// metadata alone. An existing file must never be opened and closed here, even
/// while its initialisation lock is held: another connection may own SQL locks.
pub(crate) fn create_sqlite_file(path: &Path, label: &str) -> Result<Identity, String> {
    let mut options = OpenOptions::new();
    options.read(true).write(true).create_new(true);
    #[cfg(unix)]
    options
        .mode(0o600)
        .custom_flags((rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits() as i32);
    match options.open(path) {
        Ok(file) => validate_file(path, &file, None, label),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            check_sqlite_file(path, None, label)
        }
        Err(error) => Err(format!("{label}: {error}")),
    }
}
pub(crate) fn private_directory(path: &Path, label: &str) -> Result<Identity, String> {
    #[cfg(not(unix))]
    {
        let _ = (path, label);
        Err("Private persistent storage requires POSIX ownership and permission checks on this platform".into())
    }
    #[cfg(unix)]
    {
        let metadata = fs::symlink_metadata(path).map_err(|e| format!("{label}: {e}"))?;
        if !metadata.file_type().is_dir() {
            return Err(format!("{label} must be a regular directory"));
        }
        if metadata.uid() != rustix::process::getuid().as_raw() || metadata.mode() & 0o022 != 0 {
            return Err(format!(
                "{label} must be owned by the process and not writable by others"
            ));
        }
        Ok(identity(&metadata))
    }
}
pub(crate) fn create_directory(path: &Path, label: &str) -> Result<Identity, String> {
    #[cfg(not(unix))]
    {
        private_directory(path, label)
    }
    #[cfg(unix)]
    {
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        builder.mode(0o700);
        builder.create(path).map_err(|e| format!("{label}: {e}"))?;
        private_directory(path, label)
    }
}
pub(crate) fn check_directory(path: &Path, expected: Identity, label: &str) -> Result<(), String> {
    if private_directory(path, label)? != expected {
        return Err(format!("{label} was replaced"));
    }
    Ok(())
}
fn check_metadata(metadata: &fs::Metadata, label: &str) -> Result<Identity, String> {
    if !metadata.file_type().is_file() {
        return Err(format!("{label} must be a private regular 0600 file"));
    }
    #[cfg(unix)]
    {
        if metadata.uid() != rustix::process::getuid().as_raw()
            || metadata.mode() & 0o777 != 0o600
            || metadata.nlink() != 1
        {
            return Err(format!(
                "{label} must be a private regular 0600 file with one link"
            ));
        }
        Ok(identity(metadata))
    }
    #[cfg(not(unix))]
    {
        Err("Private persistent storage requires POSIX ownership and permission checks on this platform".into())
    }
}
pub(crate) fn validate_file(
    path: &Path,
    file: &File,
    expected: Option<Identity>,
    label: &str,
) -> Result<Identity, String> {
    let opened = check_metadata(&file.metadata().map_err(|e| e.to_string())?, label)?;
    let named = check_metadata(
        &fs::symlink_metadata(path).map_err(|e| format!("{label}: {e}"))?,
        label,
    )?;
    if opened != named || expected.is_some_and(|expected| opened != expected) {
        return Err(format!("{label} was replaced"));
    }
    Ok(opened)
}
pub(crate) fn open_private(
    path: &Path,
    write: bool,
    create: bool,
    expected: Option<Identity>,
    label: &str,
) -> Result<(File, Identity), String> {
    if let Ok(metadata) = fs::symlink_metadata(path) {
        check_metadata(&metadata, label)?;
    }
    let mut options = OpenOptions::new();
    options.read(true).write(write);
    #[cfg(unix)]
    options
        .mode(0o600)
        .custom_flags((rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits() as i32);
    let file = if create {
        match options.create_new(true).open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => options
                .create_new(false)
                .open(path)
                .map_err(|e| format!("{label}: {e}"))?,
            Err(error) => return Err(format!("{label}: {error}")),
        }
    } else {
        options.open(path).map_err(|e| format!("{label}: {e}"))?
    };
    let identity = validate_file(path, &file, expected, label)?;
    Ok((file, identity))
}
pub(crate) fn create_exclusive(path: &Path, label: &str) -> Result<File, String> {
    let mut options = OpenOptions::new();
    options.read(true).write(true).create_new(true);
    #[cfg(unix)]
    options
        .mode(0o600)
        .custom_flags((rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits() as i32);
    let file = options
        .open(path)
        .map_err(|error| format!("{label}: {error}"))?;
    validate_file(path, &file, None, label)?;
    Ok(file)
}
pub(crate) fn sync_directory(path: &Path, expected: Identity, label: &str) -> Result<(), String> {
    check_directory(path, expected, label)?;
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options
        .custom_flags((rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::DIRECTORY).bits() as i32);
    let file = options.open(path).map_err(|e| e.to_string())?;
    if identity(&file.metadata().map_err(|e| e.to_string())?) != expected {
        return Err(format!("{label} was replaced"));
    }
    file.sync_all().map_err(|e| e.to_string())
}
pub(crate) fn lock(file: &File, timeout: Duration, label: &str) -> Result<(), String> {
    let started = Instant::now();
    loop {
        match file.try_lock() {
            Ok(()) => return Ok(()),
            Err(std::fs::TryLockError::WouldBlock) => {
                let remaining = timeout.saturating_sub(started.elapsed());
                if remaining.is_zero() {
                    return Err(format!("Timed out acquiring {label}"));
                }
                std::thread::sleep(remaining.min(Duration::from_millis(50)));
            }
            Err(std::fs::TryLockError::Error(error)) => {
                return Err(format!("Cannot acquire {label}: {error}"));
            }
        }
    }
}
pub(crate) fn suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

#[cfg(all(test, unix))]
mod sidecar_tests {
    use super::*;
    #[test]
    fn unlinked_private_sidecar_snapshots_are_rechecked_with_a_fixed_bound() {
        let root = std::env::temp_dir().join(format!(
            "eal-sidecar-metadata-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let path = root.join("metadata-fixture");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
            .unwrap();
        let linked = file.metadata().unwrap();
        fs::remove_file(&path).unwrap();
        let removed = file.metadata().unwrap();
        assert_eq!(removed.nlink(), 0);
        let mut calls = 0;
        check_sidecar_metadata(|| {
            calls += 1;
            if calls == 1 {
                Ok(removed.clone())
            } else {
                Err(std::io::ErrorKind::NotFound.into())
            }
        })
        .unwrap();
        assert_eq!(calls, 2);
        let mut calls = 0;
        check_sidecar_metadata(|| {
            calls += 1;
            Ok(if calls == 1 {
                removed.clone()
            } else {
                linked.clone()
            })
        })
        .unwrap();
        assert_eq!(calls, 2);
        let mut calls = 0;
        assert!(
            check_sidecar_metadata(|| {
                calls += 1;
                Ok(removed.clone())
            })
            .unwrap_err()
            .contains("did not stabilise")
        );
        assert_eq!(calls, 4);
        // Only normal unlink is retried. A permissive inode is rejected on its
        // first sample even if its next sample would disappear.
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o644))
            .unwrap();
        let unsafe_removed = file.metadata().unwrap();
        let mut calls = 0;
        assert!(
            check_sidecar_metadata(|| {
                calls += 1;
                Ok(unsafe_removed.clone())
            })
            .is_err()
        );
        assert_eq!(calls, 1);
        drop(file);
        fs::remove_dir(root).unwrap();
    }
}
