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
    time::{Duration, Instant},
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
    /// Synchronous initialisation has a finite 30-second lock-wait budget.
    /// Request handlers should use `open_until` to keep blocking filesystem work
    /// outside their async executor and share the caller's absolute deadline.
    pub fn open(path: &Path, maximum_records: usize, maximum_bytes: usize) -> Result<Self, String> {
        Self::open_sync_until(
            path,
            maximum_records,
            maximum_bytes,
            Instant::now() + Duration::from_secs(30),
        )
    }

    /// Initialise the store within the caller's monotonic deadline. A timed-out
    /// filesystem worker retains the key lock until its admitted I/O finishes.
    pub async fn open_until(
        path: &Path,
        maximum_records: usize,
        maximum_bytes: usize,
        deadline: Instant,
    ) -> Result<Self, String> {
        let path = path.to_owned();
        run_io(deadline, move || {
            Self::open_sync_until(&path, maximum_records, maximum_bytes, deadline)
        })
        .await
        .map_err(|error| error.to_string())
    }

    fn open_sync_until(
        path: &Path,
        maximum_records: usize,
        maximum_bytes: usize,
        deadline: Instant,
    ) -> Result<Self, String> {
        check_deadline(deadline)?;
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
        check_deadline(deadline)?;
        let key_lock = lock_file_until(open_private(&root.join("store.key"), true)?, deadline)?;
        let mut key_file = &key_lock.file;
        check_deadline(deadline)?;
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
        drop(key_lock);
        check_deadline(deadline)?;
        private_directory(&root.join("observations"))?;
        check_deadline(deadline)?;
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

    pub(crate) async fn lock(
        &self,
        key: &str,
        deadline: Instant,
    ) -> Result<Arc<SourceLock>, StoreIoError> {
        let key = key.to_owned();
        let store = self.clone();
        run_io(deadline, move || store.lock_sync(&key, deadline)).await
    }
    fn lock_sync(&self, key: &str, deadline: Instant) -> Result<Arc<SourceLock>, String> {
        check_deadline(deadline)?;
        if !valid_digest(key) {
            return Err("source lock identity is invalid".into());
        }
        let publication =
            lock_file_until(open_private(&self.root.join("store.lock"), true)?, deadline)?;
        let path = self.root.join(format!("{key}.lock"));
        if !path.exists() {
            let mut count = 0;
            for entry in fs::read_dir(&self.root).map_err(|_| "source store is unavailable")? {
                check_deadline(deadline)?;
                let entry = entry.map_err(|_| "source store entry is unavailable")?;
                if entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "lock")
                    && entry.file_name() != "store.lock"
                {
                    count += 1;
                }
                if count >= self.maximum_records {
                    return Err("source store identity budget is exhausted".into());
                }
            }
        }
        let file = open_private(&path, true)?;
        drop(publication);
        lock_file_until(file, deadline)
    }
    pub(crate) async fn load_until(
        &self,
        key: &str,
        lease: &Arc<SourceLock>,
        deadline: Instant,
    ) -> Result<Option<SourceRecord>, StoreIoError> {
        let store = self.clone();
        let key = key.to_owned();
        let lease = lease.clone();
        run_io(deadline, move || {
            let _lease = lease;
            check_deadline(deadline)?;
            let result = store.load(&key, deadline)?;
            check_deadline(deadline)?;
            Ok(result)
        })
        .await
    }
    pub(crate) async fn begin_until(
        &self,
        key: &str,
        lease: &Arc<SourceLock>,
        deadline: Instant,
    ) -> Result<(), StoreIoError> {
        let store = self.clone();
        let key = key.to_owned();
        let lease = lease.clone();
        run_io(deadline, move || {
            let _lease = lease;
            check_deadline(deadline)?;
            store.begin_acquisition(&key)?;
            check_deadline(deadline)
        })
        .await
    }
    pub(crate) async fn publish_until(
        &self,
        mut record: SourceRecord,
        lease: &Arc<SourceLock>,
        deadline: Instant,
    ) -> Result<SourceRecord, StoreIoError> {
        let store = self.clone();
        let lease = lease.clone();
        run_io(deadline, move || {
            let _lease = lease;
            check_deadline(deadline)?;
            store.seal(&mut record)?;
            check_deadline(deadline)?;
            let record = store.save(&record, deadline)?;
            check_deadline(deadline)?;
            store.complete_acquisition(&record.source_id)?;
            check_deadline(deadline)?;
            Ok(record)
        })
        .await
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

#[derive(Debug)]
pub(crate) enum StoreIoError {
    Deadline,
    Storage(String),
}
impl std::fmt::Display for StoreIoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deadline => f.write_str(STORE_DEADLINE),
            Self::Storage(message) => f.write_str(message),
        }
    }
}
const STORE_DEADLINE: &str = "source store deadline exhausted";
fn check_deadline(deadline: Instant) -> Result<(), String> {
    if Instant::now() >= deadline {
        Err(STORE_DEADLINE.into())
    } else {
        Ok(())
    }
}
fn lock_file_until(file: File, deadline: Instant) -> Result<Arc<SourceLock>, String> {
    #[cfg(unix)]
    loop {
        check_deadline(deadline)?;
        match rustix::fs::flock(&file, rustix::fs::FlockOperation::NonBlockingLockExclusive) {
            Ok(()) => return Ok(Arc::new(SourceLock { file })),
            Err(error) if error == rustix::io::Errno::WOULDBLOCK => {
                std::thread::sleep(std::time::Duration::from_millis(2))
            }
            Err(_) => return Err("source store lock could not be acquired".into()),
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (file, deadline);
        Err("atomic source sharing requires supported file locks".into())
    }
}
async fn run_io<T: Send + 'static>(
    deadline: Instant,
    operation: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, StoreIoError> {
    if Instant::now() >= deadline {
        return Err(StoreIoError::Deadline);
    }
    // Blocking workers retain their source lease if a caller reaches its
    // deadline. Reconciliation and another write cannot race unfinished IO.
    let result = tokio::time::timeout_at(
        tokio::time::Instant::from_std(deadline),
        tokio::task::spawn_blocking(operation),
    )
    .await;
    match result {
        Err(_) => Err(StoreIoError::Deadline),
        Ok(Err(_)) => Err(StoreIoError::Storage("source store worker failed".into())),
        Ok(Ok(Err(message))) if message == STORE_DEADLINE => Err(StoreIoError::Deadline),
        Ok(Ok(Err(message))) => Err(StoreIoError::Storage(message)),
        Ok(Ok(Ok(value))) => {
            if Instant::now() >= deadline {
                Err(StoreIoError::Deadline)
            } else {
                Ok(value)
            }
        }
    }
}

#[cfg(test)]
mod io_tests {
    use super::*;
    #[tokio::test]
    async fn an_expired_io_worker_retains_its_source_lease_until_it_finishes() {
        let directory = tempfile::tempdir().unwrap();
        let store = SourceStore::open(&directory.path().join("cache"), 10, 1_048_576).unwrap();
        let key = "a".repeat(64);
        let lease = store
            .lock(&key, Instant::now() + std::time::Duration::from_secs(2))
            .await
            .unwrap();
        store.begin_acquisition(&key).unwrap();
        let worker_lease = lease.clone();
        let (done, wait) = tokio::sync::oneshot::channel();
        let result = run_io(
            Instant::now() + std::time::Duration::from_millis(20),
            move || {
                let _lease = worker_lease;
                std::thread::sleep(std::time::Duration::from_millis(150));
                drop(_lease);
                let _ = done.send(());
                Ok(())
            },
        )
        .await;
        assert!(matches!(result, Err(StoreIoError::Deadline)));
        drop(lease);
        assert!(store.reconcile_source(&key).is_err());
        wait.await.unwrap();
        store.reconcile_source(&key).unwrap();
    }
}
