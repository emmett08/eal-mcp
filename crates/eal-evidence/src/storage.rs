//! Private atomic source records with cross-process single-acquisition locks.
use crate::{
    execution::SourceRecord,
    plan::{canonical_digest, valid_digest},
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

mod records;

static NEXT: AtomicU64 = AtomicU64::new(0);

#[derive(Clone)]
pub struct SourceStore {
    root: PathBuf,
    maximum_records: usize,
    maximum_bytes: usize,
    key: Arc<[u8; 32]>,
}
impl std::fmt::Debug for SourceStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceStore")
            .field("root", &self.root)
            .field("maximum_records", &self.maximum_records)
            .field("maximum_bytes", &self.maximum_bytes)
            .finish_non_exhaustive()
    }
}
pub(crate) struct SourceLock {
    file: File,
}
impl Drop for SourceLock {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            let _ = rustix::fs::flock(&self.file, rustix::fs::FlockOperation::Unlock);
        }
    }
}

impl SourceStore {
    /// Each physical reuse index and each immutable observation is bounded by
    /// maximum_bytes. maximum_records independently bounds source identities
    /// and retained observations; capacity exhaustion never evicts raw records.
    pub fn open(path: &Path, maximum_records: usize, maximum_bytes: usize) -> Result<Self, String> {
        if maximum_records == 0
            || maximum_records > 1_000_000
            || maximum_bytes == 0
            || maximum_bytes > 16 * 1024 * 1024
        {
            return Err("source store requires bounded positive record limits".into());
        }
        let absolute = std::path::absolute(path).map_err(|_| "source store path is unavailable")?;
        match fs::symlink_metadata(&absolute) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err("source store must be a regular directory".into());
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if let Some(parent) = absolute.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|_| "source store parent could not be created")?;
                }
                let mut builder = fs::DirBuilder::new();
                #[cfg(unix)]
                {
                    use std::os::unix::fs::DirBuilderExt;
                    builder.mode(0o700);
                }
                if let Err(error) = builder.create(&absolute) {
                    if error.kind() != std::io::ErrorKind::AlreadyExists {
                        return Err("source store could not be created".into());
                    }
                    let metadata = fs::symlink_metadata(&absolute).map_err(
                        |_| "concurrent source store initialization could not be inspected",
                    )?;
                    if metadata.file_type().is_symlink() || !metadata.is_dir() {
                        return Err("source store must be a regular directory".into());
                    }
                }
            }
            Err(_) => return Err("source store could not be inspected".into()),
        }
        let root = absolute
            .canonicalize()
            .map_err(|_| "source store could not be resolved")?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if fs::metadata(&root)
                .map_err(|_| "source store could not be inspected")?
                .permissions()
                .mode()
                & 0o077
                != 0
            {
                return Err("source store must be private to its owner".into());
            }
        }
        let mut key_file = open_private(&root.join("store.key"), true)?;
        #[cfg(unix)]
        rustix::fs::flock(&key_file, rustix::fs::FlockOperation::LockExclusive)
            .map_err(|_| "source store key lock failed")?;
        let mut key = [0u8; 32];
        if key_file
            .metadata()
            .map_err(|_| "source key metadata unavailable")?
            .len()
            == 0
        {
            getrandom::fill(&mut key).map_err(|_| "source key entropy unavailable")?;
            key_file
                .write_all(&key)
                .map_err(|_| "source store key could not be written")?;
            key_file
                .sync_all()
                .map_err(|_| "source store key could not be synced")?;
        } else {
            if key_file
                .metadata()
                .map_err(|_| "source key metadata unavailable")?
                .len()
                != 32
            {
                return Err("source store key has invalid length".into());
            }
            key_file
                .read_exact(&mut key)
                .map_err(|_| "source store key could not be read")?;
        }
        #[cfg(unix)]
        rustix::fs::flock(&key_file, rustix::fs::FlockOperation::Unlock)
            .map_err(|_| "source store key unlock failed")?;
        private_directory(&root.join("observations"))?;
        Ok(Self {
            root,
            maximum_records,
            maximum_bytes,
            key: Arc::new(key),
        })
    }
    pub fn directory(&self) -> &Path {
        &self.root
    }

    pub(crate) async fn lock(&self, key: &str, deadline: Instant) -> Result<SourceLock, String> {
        if !valid_digest(key) {
            return Err("source lock identity is invalid".into());
        }
        let path = self.root.join(format!("{key}.lock"));
        let publication = open_private(&self.root.join("store.lock"), true)?;
        #[cfg(unix)]
        rustix::fs::flock(&publication, rustix::fs::FlockOperation::LockExclusive)
            .map_err(|_| "source store reservation lock failed")?;
        if !path.exists() {
            let locks = fs::read_dir(&self.root)
                .map_err(|_| "source store is unavailable")?
                .filter_map(Result::ok)
                .filter(|entry| {
                    entry
                        .path()
                        .extension()
                        .is_some_and(|extension| extension == "lock")
                        && entry.file_name() != "store.lock"
                })
                .take(self.maximum_records + 1)
                .count();
            if locks >= self.maximum_records {
                return Err("source store identity budget is exhausted".into());
            }
        }
        let file = open_private(&path, true)?;
        drop(publication);
        #[cfg(unix)]
        loop {
            match rustix::fs::flock(&file, rustix::fs::FlockOperation::NonBlockingLockExclusive) {
                Ok(()) => return Ok(SourceLock { file }),
                Err(error) if error == rustix::io::Errno::WOULDBLOCK => {
                    if Instant::now() >= deadline {
                        return Err("source lock deadline expired".into());
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                }
                Err(_) => return Err("source lock could not be acquired".into()),
            }
        }
        #[cfg(not(unix))]
        {
            let _ = (file, deadline);
            Err("atomic source sharing requires supported file locks".into())
        }
    }

    fn seal_bytes(&self, record: &SourceRecord) -> Result<String, String> {
        let mut value =
            serde_json::to_value(record).map_err(|_| "source record could not be encoded")?;
        value
            .as_object_mut()
            .ok_or("source record requires object fields")?
            .remove("integrity");
        canonical_digest(&value)
    }
    pub(crate) fn seal(&self, record: &mut SourceRecord) -> Result<(), String> {
        record.observation_id = record.observation_identity()?;
        let mut mac = Hmac::<Sha256>::new_from_slice(self.key.as_slice())
            .map_err(|_| "source store integrity key is invalid")?;
        mac.update(self.seal_bytes(record)?.as_bytes());
        record.integrity = mac
            .finalize()
            .into_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        Ok(())
    }

    pub(crate) fn begin_acquisition(&self, key: &str) -> Result<(), String> {
        if !valid_digest(key) {
            return Err("source identity is invalid".into());
        }
        let path = self.root.join(format!("{key}.pending"));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(path)
            .map_err(|_| "previous source acquisition is uncertain; reconcile before retry")?;
        file.write_all(b"source acquisition outcome is not yet reconciled\n")
            .map_err(|_| "source acquisition marker could not be written")?;
        file.sync_all()
            .map_err(|_| "source acquisition marker could not be synced")?;
        Ok(())
    }
    /// Reconciliation is an explicit operator action, never an automatic retry.
    pub fn reconcile_source(&self, key: &str) -> Result<(), String> {
        if !valid_digest(key) {
            return Err("source identity is invalid".into());
        }
        let lock = self.root.join(format!("{key}.lock"));
        if !lock.exists() {
            return Ok(());
        }
        let held = open_private(&lock, false)?;
        #[cfg(unix)]
        rustix::fs::flock(&held, rustix::fs::FlockOperation::NonBlockingLockExclusive)
            .map_err(|_| "source acquisition remains active and cannot be reconciled")?;
        self.complete_acquisition(key)
    }
    pub(crate) fn complete_acquisition(&self, key: &str) -> Result<(), String> {
        match fs::remove_file(self.root.join(format!("{key}.pending"))) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err("source acquisition reconciliation failed".into()),
        }
    }
}

fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let digit = |byte: u8| match byte {
                b'0'..=b'9' => Some(byte - b'0'),
                b'a'..=b'f' => Some(byte - b'a' + 10),
                _ => None,
            };
            Ok(digit(pair[0]).ok_or("invalid source signature")? * 16
                + digit(pair[1]).ok_or("invalid source signature")?)
        })
        .collect()
}

fn open_private(path: &Path, create: bool) -> Result<File, String> {
    #[cfg(unix)]
    {
        use rustix::fs::{Mode, OFlags, open};
        let flags = OFlags::RDWR
            | OFlags::NOFOLLOW
            | OFlags::CLOEXEC
            | OFlags::NONBLOCK
            | if create {
                OFlags::CREATE
            } else {
                OFlags::empty()
            };
        let fd = open(path, flags, Mode::RUSR | Mode::WUSR)
            .map_err(|_| "source store leaf must be a private regular file")?;
        let file = File::from(fd);
        use std::os::unix::fs::MetadataExt;
        let metadata = file
            .metadata()
            .map_err(|_| "source store file metadata is unavailable")?;
        if !metadata.is_file() || metadata.nlink() != 1 || metadata.mode() & 0o077 != 0 {
            return Err("source store leaf must be a private regular file".into());
        }
        Ok(file)
    }
    #[cfg(not(unix))]
    {
        let _ = (path, create);
        Err("source store requires supported file identity checks".into())
    }
}

fn private_directory(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return Err("observation store must be a private regular directory".into());
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            if let Err(error) = builder.create(path)
                && error.kind() != std::io::ErrorKind::AlreadyExists
            {
                return Err("observation store could not be created".into());
            }
        }
        Err(_) => return Err("observation store could not be inspected".into()),
    }
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "observation store could not be inspected")?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("observation store must be a private regular directory".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err("observation store must be private to its owner".into());
        }
    }
    Ok(())
}
