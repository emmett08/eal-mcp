//! Validation must preserve SQLite's POSIX locks across independent processes.
#![cfg(unix)]

use eal_runtime::{private_sqlite::PrivateDatabase, store::RunStore};
use rusqlite::{Connection, ErrorCode, OpenFlags, TransactionBehavior};
use serde_json::json;
use std::{
    fs,
    os::unix::fs::DirBuilderExt,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("eal-sqlite-locks-{}", uuid::Uuid::new_v4()));
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        Self(path)
    }
    fn database(&self) -> PathBuf {
        self.0.join("state.sqlite3")
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn competing_writer_helper() {
    let Some(path) = std::env::var_os("EAL_SQLITE_LOCK_DATABASE") else {
        return;
    };
    let connection = Connection::open_with_flags(
        PathBuf::from(path),
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )
    .unwrap();
    connection.busy_timeout(Duration::ZERO).unwrap();
    let result = connection.execute_batch("BEGIN IMMEDIATE");
    if std::env::var("EAL_SQLITE_EXPECT_BLOCKED").unwrap() == "true" {
        let error = result.expect_err("another process acquired the parent's SQLite write lock");
        assert_eq!(error.sqlite_error_code(), Some(ErrorCode::DatabaseBusy));
    } else {
        result.unwrap();
        connection.execute_batch("ROLLBACK").unwrap();
    }
}

fn writer_is_blocked(path: &Path, expected: bool) {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "competing_writer_helper", "--nocapture"])
        .env("EAL_SQLITE_LOCK_DATABASE", path)
        .env("EAL_SQLITE_EXPECT_BLOCKED", expected.to_string())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "child writer failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
}

#[test]
fn run_store_reads_preserve_another_connections_wal_write_lock() {
    let directory = Directory::new();
    let path = directory.database();
    let store = RunStore::new(&path).unwrap();
    store
        .put("test", &json!({"retained": true}), Some("retained"))
        .unwrap();
    let mut connection = Connection::open(&path).unwrap();
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .unwrap();
    writer_is_blocked(&path, true);

    // Opening and dropping another descriptor for -shm here would release the
    // first connection's WAL lock, despite its transaction remaining live.
    assert_eq!(store.list(Some("test"), 100).unwrap().len(), 1);
    writer_is_blocked(&path, true);
    assert_eq!(
        store.get("retained", Some("test")).unwrap(),
        json!({"retained": true})
    );
    writer_is_blocked(&path, true);

    transaction.commit().unwrap();
    writer_is_blocked(&path, false);
}

#[test]
fn private_database_construction_and_reads_preserve_live_sqlite_locks() {
    for journal in ["DELETE", "WAL"] {
        let directory = Directory::new();
        let path = directory.database();
        let database = PrivateDatabase::new(&path, Duration::from_secs(1)).unwrap();
        database
            .with_transaction(|connection| {
                connection
                    .execute_batch("CREATE TABLE entries(id INTEGER PRIMARY KEY)")
                    .map_err(|e| e.to_string())
            })
            .unwrap();
        let mut connection = Connection::open(&path).unwrap();
        connection
            .pragma_update(None, "journal_mode", journal)
            .unwrap();
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        writer_is_blocked(&path, true);

        // Reconstructing a handle must not open/close the existing database
        // inode or sidecars while another connection in this process owns locks.
        let reopened = PrivateDatabase::new(&path, Duration::from_secs(1)).unwrap();
        writer_is_blocked(&path, true);
        let count: i64 = reopened
            .with_connection(|connection| {
                connection
                    .query_row("SELECT count(*) FROM entries", [], |row| row.get(0))
                    .map_err(|e| e.to_string())
            })
            .unwrap();
        assert_eq!(count, 0);
        writer_is_blocked(&path, true);

        transaction.commit().unwrap();
        writer_is_blocked(&path, false);
    }
}

#[test]
fn run_store_survives_concurrent_wal_checkpoint_and_connection_cleanup() {
    use std::sync::{Arc, Barrier};

    let directory = Directory::new();
    let path = directory.database();
    let store = RunStore::new(&path).unwrap();
    store
        .put("test", &json!({"retained": true}), Some("retained"))
        .unwrap();
    let start = Arc::new(Barrier::new(3));
    let readers = (0..2)
        .map(|_| {
            let store = store.clone();
            let start = Arc::clone(&start);
            std::thread::spawn(move || {
                start.wait();
                for _ in 0..64 {
                    assert_eq!(
                        store.get("retained", Some("test")).unwrap(),
                        json!({"retained": true})
                    );
                    assert_eq!(store.list(Some("test"), 10).unwrap().len(), 1);
                }
            })
        })
        .collect::<Vec<_>>();
    start.wait();
    for _ in 0..64 {
        let connection = Connection::open(&path).unwrap();
        connection.busy_timeout(Duration::from_secs(5)).unwrap();
        let (busy, _, _): (i64, i64, i64) = connection
            .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .unwrap();
        assert!(
            busy == 0 || busy == 1,
            "unexpected SQLite checkpoint result {busy}"
        );
        // Dropping the last connection lets SQLite unlink its WAL/SHM files
        // while a different thread validates or opens the next connection.
        drop(connection);
    }
    for reader in readers {
        reader.join().unwrap();
    }
    for suffix in ["-wal", "-shm"] {
        let mut sidecar = path.as_os_str().to_owned();
        sidecar.push(suffix);
        assert!(
            !PathBuf::from(sidecar).exists(),
            "last connection must clean up {suffix}"
        );
    }
    let reopened = RunStore::new(&path).unwrap();
    assert_eq!(
        reopened.get("retained", Some("test")).unwrap(),
        json!({"retained": true})
    );
    let connection = Connection::open(&path).unwrap();
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
}
