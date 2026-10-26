//! Bounded regular-file identities for ticket inputs and stage outputs.
use eal_coordination_contracts::ticket::Artifact;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::File,
    io::Read,
    path::{Component, Path},
};

pub(super) const MAX_ARTIFACT_BYTES: u64 = 16 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 64 * 1024 * 1024;

pub(super) fn read(path: &Path, maximum: u64) -> Result<Vec<u8>, String> {
    let file = open(path)?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("ticket artifact must be a regular file".into());
    }
    let mut bytes = Vec::new();
    file.take(maximum + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > maximum {
        return Err("ticket artifact exceeds its byte bound".into());
    }
    Ok(bytes)
}

#[cfg(unix)]
fn open(path: &Path) -> Result<File, String> {
    use rustix::fs::{Mode, OFlags, open, openat};
    if !path.is_absolute() {
        return Err("ticket artifact paths must be absolute".into());
    }
    let mut directory = open(
        "/",
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|e| e.to_string())?;
    for component in path
        .parent()
        .ok_or("artifact requires a parent")?
        .components()
    {
        match component {
            Component::RootDir => {}
            Component::Normal(name) => {
                directory = openat(
                    &directory,
                    name,
                    OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                    Mode::empty(),
                )
                .map_err(|e| e.to_string())?
            }
            _ => return Err("ticket artifact paths must be canonical".into()),
        }
    }
    Ok(File::from(
        openat(
            &directory,
            path.file_name().ok_or("artifact requires a filename")?,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|e| e.to_string())?,
    ))
}
#[cfg(not(unix))]
fn open(path: &Path) -> Result<File, String> {
    if std::fs::symlink_metadata(path)
        .map_err(|e| e.to_string())?
        .file_type()
        .is_symlink()
    {
        return Err("ticket artifacts must not be symlinks".into());
    }
    File::open(path).map_err(|e| e.to_string())
}

pub(super) fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Executables have a separate bound from application artifacts and are streamed.
pub(super) fn executable_identity(path: &Path) -> Result<(u64, String), String> {
    let mut file = open(path)?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("ticket worker executable must be a regular file".into());
    }
    let mut total = 0u64;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        total += count as u64;
        if total > 256 * 1024 * 1024 {
            return Err("ticket worker executable exceeds its 256MiB bound".into());
        }
        hash.update(&buffer[..count]);
    }
    Ok((total, format!("{:x}", hash.finalize())))
}

pub(super) fn validate(workspace: &Path, artifacts: &[Artifact]) -> Result<(), String> {
    if artifacts.len() > 128 {
        return Err("too many ticket artifacts".into());
    }
    let mut kinds = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut total = 0u64;
    for artifact in artifacts {
        if artifact.kind.is_empty()
            || artifact.kind.len() > 128
            || !artifact
                .kind
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
            || !kinds.insert(&artifact.kind)
            || !paths.insert(&artifact.path)
        {
            return Err("ticket artifacts require distinct bounded kinds and paths".into());
        }
        if !artifact.path.starts_with(workspace)
            || artifact.path == workspace
            || !artifact.path.is_absolute()
            || artifact.path.to_str().is_none_or(|path| path.len() > 4096)
            || artifact.path.canonicalize().map_err(|e| e.to_string())? != artifact.path
        {
            return Err("ticket artifacts must be canonical files in the ticket workspace".into());
        }
        if artifact.sha256.len() != 64
            || !artifact
                .sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || artifact.bytes > MAX_ARTIFACT_BYTES
        {
            return Err("ticket artifact identity is invalid".into());
        }
        total = total
            .checked_add(artifact.bytes)
            .ok_or("ticket artifact byte overflow")?;
        if total > MAX_TOTAL_BYTES {
            return Err("ticket artifacts exceed their aggregate byte bound".into());
        }
        let bytes = read(&artifact.path, MAX_ARTIFACT_BYTES)?;
        if bytes.len() as u64 != artifact.bytes || digest(&bytes) != artifact.sha256 {
            return Err("ticket artifact changed from its retained identity".into());
        }
    }
    Ok(())
}

pub(super) fn json(bytes: &[u8]) -> Result<Value, String> {
    eal_collector_contracts::strict_json_exact(
        bytes,
        eal_collector_contracts::JsonLimits {
            max_bytes: 4 * 1024 * 1024,
            max_depth: 64,
            max_nodes: 1_000_000,
        },
    )
}
