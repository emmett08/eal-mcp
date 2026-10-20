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
