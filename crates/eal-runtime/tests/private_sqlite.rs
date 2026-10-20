//! Private repository boundaries: no secret RunStore schema or shared connections.
#![cfg(unix)]
use eal_runtime::private_sqlite::{PrivateDatabase, PrivateLock, ensure_immutable_file};
use std::{
    cell::Cell,
    fs::{self, OpenOptions},
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    time::Duration,
};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("eal-private-sqlite-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::set_permissions(&self.0, fs::Permissions::from_mode(0o700));
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn private_file(path: &Path, contents: &[u8]) {
    use std::io::Write;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .unwrap();
    file.write_all(contents).unwrap();
}
fn database(path: &Path) -> PrivateDatabase {
    PrivateDatabase::new(path, Duration::from_secs(1)).unwrap()
}
fn initialize(database: &PrivateDatabase) {
    database
        .with_transaction(|db| {
            db.execute_batch("CREATE TABLE entries(id INTEGER PRIMARY KEY, value TEXT);")
                .map_err(|e| e.to_string())
        })
        .unwrap();
}
fn count(database: &PrivateDatabase) -> i64 {
    database
        .with_connection(|db| {
            db.query_row("SELECT count(*) FROM entries", [], |row| row.get(0))
                .map_err(|e| e.to_string())
        })
        .unwrap()
}

#[test]
fn transactions_rollback_errors_and_enforce_foreign_keys_and_full_sync() {
    let directory = Directory::new();
    let path = directory.path("state.sqlite");
    let db = database(&path);
    initialize(&db);
    let clone = db.clone();
    let error: Result<(), String> = db.with_transaction(|connection| {
        connection
            .execute("INSERT INTO entries(value) VALUES('uncommitted')", [])
            .map_err(|e| e.to_string())?;
        Err("interrupted repository operation".into())
    });
    assert_eq!(error.unwrap_err(), "interrupted repository operation");
    assert_eq!(count(&clone), 0);
    let flags: (i64, i64) = db
        .with_connection(|connection| {
            Ok((
                connection
                    .pragma_query_value(None, "foreign_keys", |row| row.get(0))
                    .map_err(|e| e.to_string())?,
                connection
                    .pragma_query_value(None, "synchronous", |row| row.get(0))
                    .map_err(|e| e.to_string())?,
            ))
        })
        .unwrap();
    assert_eq!(flags, (1, 2));
    db.with_transaction(|connection| {
        connection
            .execute_batch("CREATE TABLE children(parent INTEGER REFERENCES entries(id));")
            .map_err(|e| e.to_string())
    })
    .unwrap();
    assert!(
        db.with_transaction(|connection| connection
            .execute("INSERT INTO children VALUES(99)", [])
            .map(|_| ())
            .map_err(|e| e.to_string()))
            .unwrap_err()
            .contains("FOREIGN KEY constraint failed")
    );
    assert_eq!(fs::metadata(&path).unwrap().mode() & 0o777, 0o600);
    assert!(!directory.path("state.sqlite.binding-key").exists());
}

#[test]
fn replacement_before_commit_rolls_back_the_anchored_database_and_preserves_replacement() {
    let directory = Directory::new();
    let path = directory.path("state.sqlite");
    let retained = directory.path("retained.sqlite");
    let db = database(&path);
    initialize(&db);
    let error = db
        .with_transaction(|connection| {
            connection
                .execute("INSERT INTO entries(value) VALUES('must rollback')", [])
                .map_err(|e| e.to_string())?;
            fs::rename(&path, &retained).map_err(|e| e.to_string())?;
            private_file(&path, b"replacement sentinel");
            Ok(())
        })
        .unwrap_err();
    assert!(error.contains("replaced"), "{error}");
    assert_eq!(fs::read(&path).unwrap(), b"replacement sentinel");
    assert_eq!(count(&database(&retained)), 0);
    assert!(db.with_connection(|_| Ok(())).is_err());
}

#[test]
fn successful_reads_are_not_returned_after_path_replacement() {
    let directory = Directory::new();
    let path = directory.path("state.sqlite");
    let db = database(&path);
    initialize(&db);
    let error = db
        .with_connection(|connection| {
            let _: i64 = connection
                .query_row("SELECT count(*) FROM entries", [], |row| row.get(0))
                .map_err(|e| e.to_string())?;
            fs::rename(&path, directory.path("retained.sqlite")).map_err(|e| e.to_string())?;
            private_file(&path, b"replacement sentinel");
            Ok("must not be returned")
        })
        .unwrap_err();
    assert!(error.contains("replaced"));
    assert_eq!(fs::read(&path).unwrap(), b"replacement sentinel");
}

#[test]
fn sidecar_links_nonregular_files_and_permission_tamper_precede_callbacks() {
    for mutation in [
        "mode",
        "hardlink",
        "wal-link",
        "journal-fifo",
        "directory-mode",
    ] {
        let directory = Directory::new();
        let path = directory.path("state.sqlite");
        let db = database(&path);
        initialize(&db);
        match mutation {
            "mode" => fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap(),
            "hardlink" => fs::hard_link(&path, directory.path("alias.sqlite")).unwrap(),
            "wal-link" => {
                private_file(&directory.path("untouched"), b"sentinel");
                std::os::unix::fs::symlink(
                    directory.path("untouched"),
                    directory.path("state.sqlite-wal"),
                )
                .unwrap();
            }
            "journal-fifo" => rustix::fs::mkfifoat(
                rustix::fs::CWD,
                directory.path("state.sqlite-journal"),
                rustix::fs::Mode::from_raw_mode(0o600),
            )
            .unwrap(),
            "directory-mode" => {
                fs::set_permissions(&directory.0, fs::Permissions::from_mode(0o777)).unwrap()
            }
            _ => unreachable!(),
        }
        let called = Cell::new(false);
        assert!(
            db.with_connection(|_| {
                called.set(true);
                Ok(())
            })
            .is_err(),
            "{mutation}"
        );
        assert!(!called.get(), "{mutation}");
        if mutation == "wal-link" {
            assert_eq!(fs::read(directory.path("untouched")).unwrap(), b"sentinel");
        }
    }
}

#[test]
fn lock_clones_share_exclusion_and_refuse_replacement_of_a_held_path() {
    let directory = Directory::new();
    let path = directory.path("lifecycle.lock");
    let lock = PrivateLock::new(&path).unwrap();
    let clone = lock.clone();
    let guard = lock.hold(Duration::from_secs(1)).unwrap();
    drop(lock);
    assert!(clone.hold(Duration::from_millis(10)).is_err());
    drop(guard);
    let guard = clone.hold(Duration::from_secs(1)).unwrap();
    fs::rename(&path, directory.path("retained.lock")).unwrap();
    private_file(&path, b"");
    let error = match clone.hold(Duration::from_millis(10)) {
        Ok(_) => panic!("replacement admitted"),
        Err(error) => error,
    };
    assert!(error.contains("replaced"));
    drop(guard);
}

#[test]
fn immutable_publication_never_truncates_repairs_or_follows_untrusted_files() {
    let directory = Directory::new();
    let path = directory.path("schema.json");
    let content = b"{\"schema\":\"EAL/test/1\"}\n";
    assert_eq!(ensure_immutable_file(&path, content).unwrap(), path);
    let original = fs::metadata(&path).unwrap();
    ensure_immutable_file(&path, content).unwrap();
    assert_eq!(fs::metadata(&path).unwrap().ino(), original.ino());
    assert_eq!(original.mode() & 0o777, 0o600);
    assert_eq!(
        ensure_immutable_file(&path, b"different").unwrap_err(),
        "Private immutable file content differs"
    );
    assert_eq!(fs::read(&path).unwrap(), content);
    // A pre-existing partial publication remains explicit failure; opening it
    // must never silently repair or truncate an existing immutable artifact.
    let partial = directory.path("partial.json");
    private_file(&partial, b"{");
    assert!(ensure_immutable_file(&partial, content).is_err());
    assert_eq!(fs::read(&partial).unwrap(), b"{");
    for kind in ["symlink", "hardlink", "fifo", "mode"] {
        let selected = directory.path(&format!("{kind}.json"));
        match kind {
            "symlink" => std::os::unix::fs::symlink(&path, &selected).unwrap(),
            "hardlink" => fs::hard_link(&path, &selected).unwrap(),
            "fifo" => rustix::fs::mkfifoat(
                rustix::fs::CWD,
                &selected,
                rustix::fs::Mode::from_raw_mode(0o600),
            )
            .unwrap(),
            "mode" => {
                private_file(&selected, content);
                fs::set_permissions(&selected, fs::Permissions::from_mode(0o644)).unwrap();
            }
            _ => unreachable!(),
        }
        assert!(ensure_immutable_file(&selected, content).is_err(), "{kind}");
        if kind == "hardlink" {
            fs::remove_file(selected).unwrap();
        }
        assert_eq!(fs::read(&path).unwrap(), content);
    }
}
