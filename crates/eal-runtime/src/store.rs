//! Immutable SQLite records, private store identity and indexed observation reuse.
use rusqlite::{Connection, OpenFlags, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

pub(crate) mod files;
mod key;
mod payload;
mod schema;
pub use payload::MAX_STORED_RECORD_BYTES;
use payload::{decode, encode};

/// Clones retain identity, never a shared SQLite connection or its transaction.
#[derive(Clone)]
pub struct RunStore {
    path: PathBuf,
    parent: files::Identity,
    database: files::Identity,
    key_file: files::Identity,
    key: [u8; 32],
}
impl std::fmt::Debug for RunStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunStore")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}
#[derive(Clone, Copy, Debug)]
pub struct StoreEntry<'a> {
    pub kind: &'a str,
    pub payload: &'a Value,
    pub record_id: Option<&'a str>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredEntry {
    pub id: String,
    pub kind: String,
    pub created_at: String,
}
#[derive(Clone, Copy, Debug)]
pub struct ObservationFilter<'a> {
    pub evidence_id: &'a str,
    pub environment: &'a str,
    pub request_digest: &'a str,
    pub tool_binding_digest: &'a str,
    pub limit: usize,
}
impl<'a> ObservationFilter<'a> {
    pub fn new(
        evidence_id: &'a str,
        environment: &'a str,
        request_digest: &'a str,
        tool_binding_digest: &'a str,
    ) -> Self {
        Self {
            evidence_id,
            environment,
            request_digest,
            tool_binding_digest,
            limit: 100,
        }
    }
}

pub fn utc_now() -> String {
    let now = chrono::Utc::now();
    now.to_rfc3339_opts(
        if now.timestamp_subsec_micros() == 0 {
            chrono::SecondsFormat::Secs
        } else {
            chrono::SecondsFormat::Micros
        },
        true,
    )
}
impl RunStore {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = files::absolute(path.as_ref())?;
        let directory = path
            .parent()
            .ok_or("The run store must have a parent directory")?;
        let parent = files::create_directory(directory, "The run-store directory")?;
        let lock_path = files::suffix(&path, ".initialisation.lock");
        let (lock, lock_identity) = files::open_private(
            &lock_path,
            true,
            true,
            None,
            "The run-store initialisation lock",
        )?;
        files::lock(
            &lock,
            Duration::from_secs(30),
            "the run-store initialisation lock",
        )?;
        files::check_directory(directory, parent, "The run-store directory")?;
        files::validate_file(
            &lock_path,
            &lock,
            Some(lock_identity),
            "The run-store initialisation lock",
        )?;
        let (key, key_file) = key::load(&path, parent)?;
        let (_, database) = files::open_private(&path, true, true, None, "SQLite store file")?;
        let store = Self {
            path,
            parent,
            database,
            key_file,
            key,
        };
        store.check_files()?;
        store.initialise_schema()?;
        store.check_files()?;
        files::sync_directory(
            store.path.parent().unwrap(),
            parent,
            "The run-store directory",
        )?;
        Ok(store)
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub(crate) fn binding_key(&self) -> &[u8; 32] {
        &self.key
    }
    fn check_files(&self) -> Result<(), String> {
        files::check_directory(
            self.path.parent().unwrap(),
            self.parent,
            "The run-store directory",
        )?;
        let (key, _) = key::read(
            &files::suffix(&self.path, ".binding-key"),
            Some(self.key_file),
        )?;
        if key != self.key {
            return Err("The run-store binding key was modified".into());
        }
        files::open_private(
            &self.path,
            false,
            false,
            Some(self.database),
            "SQLite store file",
        )?;
        for suffix in ["-wal", "-shm", "-journal"] {
            let path = files::suffix(&self.path, suffix);
            match std::fs::symlink_metadata(&path) {
                Ok(_) => {
                    if let Err(error) =
                        files::open_private(&path, false, false, None, "SQLite store sidecar")
                        && !std::fs::symlink_metadata(&path)
                            .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound)
                    {
                        return Err(error);
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.to_string()),
            }
        }
        Ok(())
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
            .busy_timeout(Duration::from_secs(30))
            .map_err(|e| e.to_string())?;
        self.check_files()?;
        Ok(connection)
    }
    /// Internal repositories borrow one checked, owned connection; no handle
    /// escapes this boundary or remains shared between operations.
    pub(crate) fn with_connection<T>(
        &self,
        operation: impl FnOnce(&Connection) -> Result<T, String>,
    ) -> Result<T, String> {
        let connection = self.connect()?;
        let result = operation(&connection)?;
        self.check_files()?;
        Ok(result)
    }
    /// Repository updates commit only after validating the captured file identities.
    pub(crate) fn with_transaction<T>(
        &self,
        operation: impl FnOnce(&rusqlite::Transaction<'_>) -> Result<T, String>,
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
    pub fn put(
        &self,
        kind: &str,
        payload: &Value,
        record_id: Option<&str>,
    ) -> Result<String, String> {
        self.put_many(&[StoreEntry {
            kind,
            payload,
            record_id,
        }])
        .map(|mut ids| ids.remove(0))
    }
    /// Encode every entry first, then commit records and reuse-index rows atomically.
    pub fn put_many(&self, entries: &[StoreEntry<'_>]) -> Result<Vec<String>, String> {
        if entries.is_empty() {
            return Err("entries must not be empty".into());
        }
        let ids: Vec<String> = entries
            .iter()
            .map(|entry| {
                entry
                    .record_id
                    .filter(|id| !id.is_empty())
                    .map(str::to_owned)
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
            })
            .collect();
        let encoded: Vec<String> = entries
            .iter()
            .map(|entry| encode(entry.payload))
            .collect::<Result<_, _>>()?;
        let mut connection = self.connect()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|e| e.to_string())?;
        for ((entry, id), value) in entries.iter().zip(&ids).zip(&encoded) {
            transaction
                .execute(
                    "INSERT INTO records(id, kind, created_at, payload) VALUES (?, ?, ?, ?)",
                    params![id, entry.kind, utc_now(), value],
                )
                .map_err(|e| e.to_string())?;
            if entry.kind == "observation" {
                schema::index_observation(&transaction, id, entry.payload)?;
            }
        }
        self.check_files()?;
        transaction.commit().map_err(|e| e.to_string())?;
        self.check_files()?;
        Ok(ids)
    }
    pub fn put_batch(&self, entries: &[StoreEntry<'_>]) -> Result<Vec<String>, String> {
        self.put_many(entries)
    }
    pub fn get(&self, record_id: &str, kind: Option<&str>) -> Result<Value, String> {
        let connection = self.connect()?;
        let row: Option<(String, Option<String>)> = connection
            .query_row(
                "SELECT kind, CASE WHEN length(CAST(payload AS BLOB)) <= ? THEN payload ELSE NULL END FROM records WHERE id = ?",
                params![MAX_STORED_RECORD_BYTES as i64,record_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        self.check_files()?;
        match row {
            Some((actual, payload)) if kind.is_none_or(|kind| actual == kind) => {
                decode(&payload.ok_or("Stored record exceeds the byte budget")?)
            }
            _ => Err(format!(
                "No {} record with ID {}",
                kind.filter(|kind| !kind.is_empty()).unwrap_or("stored"),
                eal_core::expressions::diagnostic_repr(
                    &Value::String(record_id.into()),
                    &eal_core::ExecutionLimits::default()
                )?
            )),
        }
    }
    pub fn list(&self, kind: Option<&str>, limit: usize) -> Result<Vec<StoredEntry>, String> {
        check_limit(limit)?;
        let connection = self.connect()?;
        let (sql, parameters): (&str, Vec<rusqlite::types::Value>) = if let Some(kind) = kind {
            (
                "SELECT id, kind, created_at FROM records WHERE kind = ? ORDER BY rowid DESC LIMIT ?",
                vec![kind.to_owned().into(), (limit as i64).into()],
            )
        } else {
            (
                "SELECT id, kind, created_at FROM records ORDER BY rowid DESC LIMIT ?",
                vec![(limit as i64).into()],
            )
        };
        let mut statement = connection.prepare(sql).map_err(|e| e.to_string())?;
        let rows = statement
            .query_map(rusqlite::params_from_iter(parameters), |row| {
                Ok(StoredEntry {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    created_at: row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let result = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        self.check_files()?;
        Ok(result)
    }
    pub fn find_observations(&self, filter: &ObservationFilter<'_>) -> Result<Vec<Value>, String> {
        check_limit(filter.limit)?;
        let connection = self.connect()?;
        let mut statement=connection.prepare("SELECT CASE WHEN length(CAST(records.payload AS BLOB)) <= ? THEN records.payload ELSE NULL END FROM observation_index JOIN records ON records.id = observation_index.record_id WHERE evidence_id = ? AND environment = ? AND request_digest = ? AND tool_binding_digest = ? ORDER BY sequence DESC LIMIT ?").map_err(|e|e.to_string())?;
        let rows = statement
            .query_map(
                params![
                    MAX_STORED_RECORD_BYTES as i64,
                    filter.evidence_id,
                    filter.environment,
                    filter.request_digest,
                    filter.tool_binding_digest,
                    filter.limit as i64
                ],
                |row| row.get::<_, Option<String>>(0),
            )
            .map_err(|e| e.to_string())?;
        let result = rows
            .map(|row| {
                decode(
                    &row.map_err(|e| e.to_string())?
                        .ok_or("Stored record exceeds the byte budget")?,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.check_files()?;
        Ok(result)
    }
}
fn check_limit(limit: usize) -> Result<(), String> {
    if !(1..=1000).contains(&limit) {
        Err("limit must be between 1 and 1000".into())
    } else {
        Ok(())
    }
}
