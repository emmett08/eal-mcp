//! Private SQLite file identity and short owned transactions for host repositories.
use crate::store::files;
use rusqlite::{Connection, OpenFlags, Transaction, TransactionBehavior};
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

/// A handle captures identities rather than retaining a shared connection.
#[derive(Clone, Debug)]
pub struct PrivateDatabase {
    path: PathBuf,
    parent: files::Identity,
    database: files::Identity,
    timeout: Duration,
}
impl PrivateDatabase {
    pub fn new(path: impl AsRef<Path>, timeout: Duration) -> Result<Self, String> {
        if timeout.is_zero() || timeout > Duration::from_secs(30) {
            return Err("Private database timeout must be positive and at most 30 seconds".into());
        }
        let path = files::absolute(path.as_ref())?;
        let parent_path = path
            .parent()
            .ok_or("Private database requires a directory")?;
        let parent = files::create_directory(parent_path, "Private database directory")?;
        let lock_path = files::suffix(&path, ".initialisation.lock");
        let lock = PrivateLock::new(&lock_path)?;
        let _guard = lock.hold(timeout)?;
        files::check_directory(parent_path, parent, "Private database directory")?;
        let database = files::create_sqlite_file(&path, "Private database file")?;
        let value = Self {
            path,
            parent,
            database,
            timeout,
        };
        value.check_files()?;
        files::sync_directory(
            value.path.parent().unwrap(),
            parent,
            "Private database directory",
        )?;
        Ok(value)
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    fn check_files(&self) -> Result<(), String> {
        files::check_directory(
            self.path.parent().unwrap(),
            self.parent,
            "Private database directory",
        )?;
        files::check_sqlite_file(&self.path, Some(self.database), "Private database file")?;
        files::check_sqlite_sidecars(&self.path)
    }
    fn connect(&self) -> Result<Connection, String> {
        self.check_files()?;
        let connection = Connection::open_with_flags(
            &self.path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX
                | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(|e| e.to_string())?;
        connection
            .busy_timeout(self.timeout)
            .map_err(|e| e.to_string())?;
        connection
            .execute_batch("PRAGMA foreign_keys=ON; PRAGMA synchronous=FULL;")
            .map_err(|e| e.to_string())?;
        self.check_files()?;
        Ok(connection)
    }
    /// The closure owns no persistent handle; callbacks causing external effects
    /// must run after this operation has returned.
    pub fn with_connection<T>(
        &self,
        operation: impl FnOnce(&Connection) -> Result<T, String>,
    ) -> Result<T, String> {
        let connection = self.connect()?;
        let result = operation(&connection)?;
        self.check_files()?;
        Ok(result)
    }
    /// Commit a bounded repository operation after checking captured identities.
    pub fn with_transaction<T>(
        &self,
        operation: impl FnOnce(&Transaction<'_>) -> Result<T, String>,
    ) -> Result<T, String> {
        let mut connection = self.connect()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|e| e.to_string())?;
        let result = operation(&transaction)?;
        self.check_files()?;
        transaction.commit().map_err(|e| e.to_string())?;
        self.check_files()?;
        Ok(result)
    }
}

/// An independently named lock for a host lifecycle, not an acquisition grant.
#[derive(Clone, Debug)]
pub struct PrivateLock {
    path: PathBuf,
    parent: files::Identity,
    identity: files::Identity,
}
pub struct PrivateLockGuard {
    _file: File,
}

/// Publish immutable host material beside private state. Existing content must
/// agree byte for byte; it is never truncated or repaired.
pub fn ensure_immutable_file(path: impl AsRef<Path>, contents: &[u8]) -> Result<PathBuf, String> {
    let path = files::absolute(path.as_ref())?;
    let directory = path
        .parent()
        .ok_or("Private publication requires a directory")?;
    let parent = files::create_directory(directory, "Private publication directory")?;
    let (mut file, identity, created) =
        match files::create_exclusive(&path, "Private immutable file") {
            Ok(file) => {
                let identity = files::validate_file(&path, &file, None, "Private immutable file")?;
                (file, identity, true)
            }
            Err(_) => {
                let (file, identity) =
                    files::open_private(&path, false, false, None, "Private immutable file")?;
                (file, identity, false)
            }
        };
    if created {
        file.write_all(contents).map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
    } else {
        let mut retained = Vec::new();
        (&mut file)
            .take(contents.len() as u64 + 1)
            .read_to_end(&mut retained)
            .map_err(|e| e.to_string())?;
        if retained != contents {
            return Err("Private immutable file content differs".into());
        }
    }
    files::validate_file(&path, &file, Some(identity), "Private immutable file")?;
    files::check_directory(directory, parent, "Private publication directory")?;
    if created {
        files::sync_directory(directory, parent, "Private publication directory")?;
    }
    Ok(path)
}
impl PrivateLock {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = files::absolute(path.as_ref())?;
        let parent = files::create_directory(
            path.parent().ok_or("Lock requires a directory")?,
            "Private lock directory",
        )?;
        let (_, identity) = files::open_private(&path, true, true, None, "Private lifecycle lock")?;
        Ok(Self {
            path,
            parent,
            identity,
        })
    }
    pub fn hold(&self, timeout: Duration) -> Result<PrivateLockGuard, String> {
        files::check_directory(
            self.path.parent().unwrap(),
            self.parent,
            "Private lock directory",
        )?;
        let (file, _) = files::open_private(
            &self.path,
            true,
            false,
            Some(self.identity),
            "Private lifecycle lock",
        )?;
        files::lock(&file, timeout, "private lifecycle lock")?;
        files::validate_file(
            &self.path,
            &file,
            Some(self.identity),
            "Private lifecycle lock",
        )?;
        files::check_directory(
            self.path.parent().unwrap(),
            self.parent,
            "Private lock directory",
        )?;
        Ok(PrivateLockGuard { _file: file })
    }
}
