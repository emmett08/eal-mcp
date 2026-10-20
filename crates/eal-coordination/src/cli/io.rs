use serde_json::Value;
use std::{
    fs::File,
    io::{Read, Write},
    path::{Component, Path},
};

pub(super) fn read(path: &Path, maximum: usize) -> Result<Vec<u8>, String> {
    let path = path.canonicalize().map_err(|error| error.to_string())?;
    let file = open(&path, false)?;
    if !file
        .metadata()
        .map_err(|error| error.to_string())?
        .is_file()
    {
        return Err("Input must be a regular file".into());
    }
    let mut bytes = Vec::new();
    file.take((maximum as u64).saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() > maximum {
        return Err(format!("Input exceeds its {maximum} byte budget"));
    }
    Ok(bytes)
}

pub(super) fn json(path: &Path) -> Result<Value, String> {
    let bytes = read(path, 4 * 1024 * 1024)?;
    super::strict_json(&bytes)
}

pub(super) fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| error.to_string())?
            .join(path)
    };
    let parent = absolute
        .parent()
        .ok_or("Output must have a parent directory")?;
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let parent = parent.canonicalize().map_err(|error| error.to_string())?;
    let path = parent.join(absolute.file_name().ok_or("Output must name a file")?);
    let mut file = open(&path, true)?;
    if !file
        .metadata()
        .map_err(|error| error.to_string())?
        .is_file()
    {
        return Err("Output must be a regular file".into());
    }
    let mut encoded = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    encoded.push(b'\n');
    file.set_len(0).map_err(|error| error.to_string())?;
    file.write_all(&encoded).map_err(|error| error.to_string())
}

#[cfg(unix)]
fn open(path: &Path, write: bool) -> Result<File, String> {
    use rustix::fs::{Mode, OFlags, open, openat};
    let mut directory = open(
        "/",
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| error.to_string())?;
    for component in path
        .parent()
        .ok_or("File requires a parent directory")?
        .components()
    {
        if component == Component::RootDir {
            continue;
        }
        let Component::Normal(component) = component else {
            return Err("File path must be canonical".into());
        };
        directory = openat(
            &directory,
            component,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|error| error.to_string())?;
    }
    let flags = if write {
        OFlags::WRONLY | OFlags::CREATE
    } else {
        OFlags::RDONLY
    };
    Ok(File::from(
        openat(
            &directory,
            path.file_name().ok_or("Path must name a file")?,
            flags | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::from_raw_mode(0o600),
        )
        .map_err(|error| error.to_string())?,
    ))
}

#[cfg(not(unix))]
fn open(path: &Path, write: bool) -> Result<File, String> {
    if std::fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err("Configured file must not be a symlink".into());
    }
    std::fs::OpenOptions::new()
        .read(!write)
        .write(write)
        .create(write)
        .truncate(false)
        .open(path)
        .map_err(|error| error.to_string())
}
