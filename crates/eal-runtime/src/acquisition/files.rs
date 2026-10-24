use super::AcquisitionError;
use std::{
    fs::File,
    io::Read,
    path::{Component, Path, PathBuf},
};

/// Resolve existing symlinks, retaining a normalised suffix for a missing file.
pub(super) fn resolve(path: &Path) -> Result<PathBuf, AcquisitionError> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut existing = absolute.as_path();
    let mut suffix = Vec::new();
    let mut base = loop {
        match existing.canonicalize() {
            Ok(value) => break value,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let last = existing
                    .components()
                    .next_back()
                    .ok_or_else(|| AcquisitionError::from(error))?;
                suffix.push(last.as_os_str().to_owned());
                existing = existing
                    .parent()
                    .ok_or("Cannot resolve the configured path")?;
            }
            Err(error) => return Err(error.into()),
        }
    };
    for part in suffix.into_iter().rev() {
        if part == ".." {
            base.pop();
        } else if part != "." {
            base.push(part);
        }
    }
    Ok(base)
}

pub fn bounded_path(workspace: &Path, name: &str) -> Result<PathBuf, AcquisitionError> {
    let root = workspace.canonicalize()?;
    let path = resolve(&root.join(name))?;
    if !path.starts_with(&root) {
        return Err(AcquisitionError::invalid(format!(
            "Path is outside the workspace: {name}"
        )));
    }
    Ok(path)
}

/// Every component is opened without following links after canonical resolution.
#[cfg(unix)]
fn open_checked(path: &Path) -> Result<File, AcquisitionError> {
    use rustix::fs::{Mode, OFlags, open, openat};
    let mut parent = open(
        "/",
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(std::io::Error::from)?;
    let parts = path
        .components()
        .filter_map(|c| {
            if let Component::Normal(c) = c {
                Some(c)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let (last, parts) = parts
        .split_last()
        .ok_or("File observations require a regular file")?;
    for part in parts {
        parent = openat(
            &parent,
            Path::new(part),
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(std::io::Error::from)?;
    }
    let descriptor = openat(
        &parent,
        Path::new(last),
        OFlags::RDONLY | OFlags::NONBLOCK | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(std::io::Error::from)?;
    Ok(File::from(descriptor))
}
#[cfg(not(unix))]
fn open_checked(path: &Path) -> Result<File, AcquisitionError> {
    if std::fs::symlink_metadata(path)?.file_type().is_symlink() {
        return Err("The configured file changed during acquisition".into());
    }
    Ok(File::open(path)?)
}

fn unchanged(path: &Path, before: &std::fs::Metadata) -> Result<(), AcquisitionError> {
    if path.canonicalize()? != path {
        return Err("The configured file changed during acquisition".into());
    }
    let after = std::fs::metadata(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if before.dev() != after.dev() || before.ino() != after.ino() {
            return Err("The configured file changed during acquisition".into());
        }
    }
    #[cfg(not(unix))]
    {
        if before.len() != after.len() || before.modified().ok() != after.modified().ok() {
            return Err("The configured file changed during acquisition".into());
        }
    }
    Ok(())
}

/// Read at most `limit + 1` bytes from an identified regular file.
///
/// Supply an absolute path with a canonical parent. The final component and
/// every resolved directory are opened without following links on POSIX hosts.
/// Special files are rejected before reading, and the path identity is checked
/// afterwards. The caller distinguishes an oversized result from other errors.
pub fn read_regular(
    path: &Path,
    limit: usize,
    not_regular: &str,
) -> Result<Vec<u8>, AcquisitionError> {
    let parent = path
        .parent()
        .filter(|_| path.is_absolute())
        .ok_or("Regular-file reads require an absolute path with a canonical parent")?;
    if parent.canonicalize()? != parent {
        return Err("Regular-file reads require an absolute path with a canonical parent".into());
    }
    let file = open_checked(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(AcquisitionError::invalid(not_regular));
    }
    let mut bytes = Vec::new();
    file.take((limit as u64).saturating_add(1))
        .read_to_end(&mut bytes)?;
    unchanged(path, &metadata)?;
    Ok(bytes)
}

pub(super) fn read_pinned(path: &Path, limit: usize) -> Result<Vec<u8>, AcquisitionError> {
    let file = open_checked(path)
        .map_err(|_| AcquisitionError::invalid("Pinned collector file is inaccessible"))?;
    let metadata = file
        .metadata()
        .map_err(|_| AcquisitionError::invalid("Pinned collector file is inaccessible"))?;
    if !metadata.is_file() || metadata.len() > limit as u64 {
        return Err("Pinned collector file must be a regular file at most 16 MiB".into());
    }
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| AcquisitionError::invalid("Pinned collector file is inaccessible"))?;
    if bytes.len() > limit {
        return Err("Pinned collector file must be a regular file at most 16 MiB".into());
    }
    unchanged(path, &metadata).map_err(|_| {
        AcquisitionError::invalid(
            "Pinned collector file identity differs from operator configuration",
        )
    })?;
    Ok(bytes)
}
