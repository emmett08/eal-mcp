//! Byte-bounded admission of regular adapter inputs with rejected leaf aliases.
use crate::Error;
use std::{fs::File, io::Read, path::Path};

pub fn read(path: &Path, limit: usize, label: &str) -> Result<Vec<u8>, Error> {
    let selected = std::fs::symlink_metadata(path)?;
    if !selected.is_file() {
        return Err(Error::InvalidInput(format!(
            "{label} requires a regular file"
        )));
    }
    if selected.len() > limit as u64 {
        return Err(Error::Budget(format!("{label} exceeds its byte limit")));
    }
    #[cfg(unix)]
    let file = File::from(
        rustix::fs::open(
            path,
            rustix::fs::OFlags::RDONLY
                | rustix::fs::OFlags::NOFOLLOW
                | rustix::fs::OFlags::NONBLOCK
                | rustix::fs::OFlags::CLOEXEC,
            rustix::fs::Mode::empty(),
        )
        .map_err(std::io::Error::from)?,
    );
    #[cfg(not(unix))]
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(Error::InvalidInput(format!(
            "{label} requires a regular file"
        )));
    }
    if metadata.len() > limit as u64 {
        return Err(Error::Budget(format!("{label} exceeds its byte limit")));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if selected.dev() != metadata.dev() || selected.ino() != metadata.ino() {
            return Err(Error::InvalidInput(format!(
                "{label} changed during admission"
            )));
        }
    }
    let mut bytes = Vec::new();
    file.take(limit.saturating_add(1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(Error::Budget(format!("{label} exceeds its byte limit")));
    }
    Ok(bytes)
}
