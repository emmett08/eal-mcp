use super::*;
use std::io::{Read, Write};

pub(super) fn load(
    path: &Path,
    parent: files::Identity,
) -> Result<([u8; 32], files::Identity), String> {
    let key_path = files::suffix(path, ".binding-key");
    if !key_path.try_exists().map_err(|e| e.to_string())? {
        let temporary = files::suffix(
            &key_path,
            &format!(".{}.tmp", uuid::Uuid::new_v4().simple()),
        );
        let mut file = files::create_exclusive(&temporary, "The run-store temporary binding key")?;
        let result = (|| {
            let mut candidate = [0u8; 32];
            getrandom::fill(&mut candidate).map_err(|e| e.to_string())?;
            file.write_all(&candidate).map_err(|e| e.to_string())?;
            file.sync_all().map_err(|e| e.to_string())?;
            match std::fs::hard_link(&temporary, &key_path) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
                Err(error) => Err(error.to_string()),
            }
        })();
        let cleanup = std::fs::remove_file(&temporary).map_err(|e| e.to_string());
        result?;
        cleanup?;
        files::sync_directory(path.parent().unwrap(), parent, "The run-store directory")?;
    }
    read(&key_path, None)
}
pub(super) fn read(
    path: &Path,
    expected: Option<files::Identity>,
) -> Result<([u8; 32], files::Identity), String> {
    let (file, identity) =
        files::open_private(path, false, false, expected, "The run-store binding key")?;
    let mut key = Vec::new();
    file.take(33)
        .read_to_end(&mut key)
        .map_err(|e| e.to_string())?;
    if key.len() != 32 {
        return Err("The run-store binding key must contain exactly 32 bytes".into());
    }
    Ok((
        key.try_into().map_err(|_| "Invalid binding key length")?,
        identity,
    ))
}
