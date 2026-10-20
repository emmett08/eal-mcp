#![cfg(unix)]
use eal_runtime::store::{ObservationFilter, RunStore, StoreEntry};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
    sync::{Arc, Barrier},
};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("eal-store-test-{}", uuid::Uuid::new_v4()));
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        Self(path)
    }
    fn database(&self) -> PathBuf {
        self.0.join("runs.sqlite3")
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut path = path.as_os_str().to_owned();
    path.push(suffix);
    PathBuf::from(path)
}
fn observation(value: i64) -> Value {
    json!({"status":"ok","evidence_id":"observed","environment":"lab","request_digest":"request","tool_binding_digest":"binding","value":value})
}
fn filter() -> ObservationFilter<'static> {
    ObservationFilter::new("observed", "lab", "request", "binding")
}

#[test]
fn immutable_records_listing_batch_and_reuse_match_source_order() {
    let directory = Directory::new();
    let store = RunStore::new(directory.database()).unwrap();
    store
        .put("observation", &observation(1), Some("first"))
        .unwrap();
    store
        .put("observation", &json!({"status":"error"}), Some("failed"))
        .unwrap();
    store
        .put(
            "observation",
            &json!({"status":"ok","evidence_id":"observed"}),
            Some("incomplete"),
        )
        .unwrap();
    let a = json!({"span":{"literal":true},"$serde_json::private::Number":"123","unicode":"Δ","number":123456789012345678901234567890u128});
    let b = observation(2);
    assert_eq!(
        store
            .put_many(&[
                StoreEntry {
                    kind: "collection",
                    payload: &a,
                    record_id: Some("batch")
                },
                StoreEntry {
                    kind: "observation",
                    payload: &b,
                    record_id: Some("second")
                }
            ])
            .unwrap(),
        ["batch", "second"]
    );
    assert_eq!(store.get("batch", Some("collection")).unwrap(), a);
    assert!(store.get("batch", Some("reasoning")).is_err());
    assert_eq!(
        store.find_observations(&filter()).unwrap(),
        [observation(2), observation(1)]
    );
    assert_eq!(
        store
            .list(None, 100)
            .unwrap()
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
        ["second", "batch", "incomplete", "failed", "first"]
    );
    assert_eq!(store.list(Some("collection"), 1).unwrap()[0].id, "batch");
    let before = store.get("first", None).unwrap();
    assert!(store.put("changed", &json!({}), Some("first")).is_err());
    assert_eq!(store.get("first", None).unwrap(), before);
    let mut detached = store.get("batch", None).unwrap();
    detached["span"] = json!(false);
    assert_eq!(store.get("batch", None).unwrap(), a);
    for limit in [0, 1001, usize::MAX] {
        assert!(store.list(None, limit).is_err());
        let mut f = filter();
        f.limit = limit;
        assert!(store.find_observations(&f).is_err());
    }
    assert!(store.put_many(&[]).is_err());
    assert!(
        store
            .get("absent", None)
            .unwrap_err()
            .contains("No stored record with ID 'absent'")
    );
}

#[test]
fn failed_batches_roll_back_both_record_and_index_rows() {
    let directory = Directory::new();
    let store = RunStore::new(directory.database()).unwrap();
    store
        .put("observation", &observation(1), Some("existing"))
        .unwrap();
    let a = observation(2);
    let b = json!({"collection":true});
    assert!(
        store
            .put_batch(&[
                StoreEntry {
                    kind: "observation",
                    payload: &a,
                    record_id: Some("rolled-back")
                },
                StoreEntry {
                    kind: "collection",
                    payload: &b,
                    record_id: Some("existing")
                }
            ])
            .is_err()
    );
    assert!(store.get("rolled-back", None).is_err());
    assert_eq!(
        store.find_observations(&filter()).unwrap(),
        [observation(1)]
    );
    assert_eq!(store.list(None, 100).unwrap().len(), 1);
    assert!(
        store
            .put_many(&[
                StoreEntry {
                    kind: "collection",
                    payload: &b,
                    record_id: Some("rolled-back-2")
                },
                StoreEntry {
                    kind: "observation",
                    payload: &json!([]),
                    record_id: Some("invalid")
                }
            ])
            .is_err()
    );
    assert!(store.get("rolled-back-2", None).is_err());
}

#[test]
fn simultaneous_initialisation_and_writes_share_one_key_and_schema() {
    let directory = Directory::new();
    let path = Arc::new(directory.database());
    let barrier = Arc::new(Barrier::new(8));
    let workers: Vec<_> = (0..8)
        .map(|index| {
            let path = path.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                let store = RunStore::new(path.as_path()).unwrap();
                store
                    .put(
                        "test",
                        &json!({"index":index}),
                        Some(&format!("thread-{index}")),
                    )
                    .unwrap();
            })
        })
        .collect();
    for worker in workers {
        worker.join().unwrap();
    }
    let store = RunStore::new(path.as_path()).unwrap();
    assert_eq!(store.list(None, 100).unwrap().len(), 8);
    assert_eq!(fs::read(suffix(&path, ".binding-key")).unwrap().len(), 32);
    let connection = rusqlite::Connection::open(path.as_path()).unwrap();
    assert_eq!(
        connection
            .query_row("PRAGMA journal_mode", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "wal"
    );
    assert_eq!(
        connection
            .query_row("SELECT value FROM store_metadata", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "1"
    );
    let lock = suffix(&path, ".initialisation.lock");
    let before = fs::symlink_metadata(&lock).unwrap();
    RunStore::new(path.as_path()).unwrap();
    let after = fs::symlink_metadata(&lock).unwrap();
    assert_eq!((before.dev(), before.ino()), (after.dev(), after.ino()));
    assert_eq!(after.mode() & 0o777, 0o600);
    fn send_sync<T: Send + Sync + Clone>() {}
    send_sync::<RunStore>();
}

#[test]
fn process_initialisation_helper() {
    if let Some(path) = std::env::var_os("EAL_TEST_STORE_DATABASE") {
        let store = RunStore::new(PathBuf::from(path)).unwrap();
        let id = std::env::var("EAL_TEST_STORE_ID").unwrap();
        store.put("process", &json!({"id":id}), Some(&id)).unwrap();
    }
}
#[test]
fn processes_initialise_and_commit_without_duplicate_index_setup() {
    let directory = Directory::new();
    let path = directory.database();
    let mut children: Vec<_> = (0..4)
        .map(|index| {
            std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "process_initialisation_helper", "--nocapture"])
                .env("EAL_TEST_STORE_DATABASE", &path)
                .env("EAL_TEST_STORE_ID", format!("process-{index}"))
                .stdout(std::process::Stdio::null())
                .spawn()
                .unwrap()
        })
        .collect();
    for child in &mut children {
        assert!(child.wait().unwrap().success());
    }
    assert_eq!(
        RunStore::new(path)
            .unwrap()
            .list(Some("process"), 100)
            .unwrap()
            .len(),
        4
    );
}

fn legacy(path: &Path) {
    let connection = rusqlite::Connection::open(path).unwrap();
    connection.execute_batch("CREATE TABLE records (id TEXT PRIMARY KEY, kind TEXT NOT NULL, created_at TEXT NOT NULL, payload TEXT NOT NULL)").unwrap();
    for (id, payload) in [
        ("first", observation(1)),
        ("second", observation(2)),
        ("failed", json!({"status":"error"})),
        ("incomplete", json!({"status":"ok"})),
    ] {
        connection
            .execute(
                "INSERT INTO records VALUES (?, 'observation', 'now', ?)",
                rusqlite::params![id, payload.to_string()],
            )
            .unwrap();
    }
    drop(connection);
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
}
#[test]
fn legacy_backfill_is_atomic_ordered_and_runs_once() {
    let directory = Directory::new();
    let path = directory.database();
    legacy(&path);
    let barrier = Arc::new(Barrier::new(4));
    let workers: Vec<_> = (0..4)
        .map(|_| {
            let path = path.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                RunStore::new(path).unwrap();
            })
        })
        .collect();
    for worker in workers {
        worker.join().unwrap();
    }
    let store = RunStore::new(&path).unwrap();
    assert_eq!(
        store.find_observations(&filter()).unwrap(),
        [observation(2), observation(1)]
    );
    assert_eq!(store.list(None, 100).unwrap().len(), 4);
}

#[test]
fn unsupported_version_rolls_back_schema_changes() {
    let directory = Directory::new();
    let path = directory.database();
    let store = RunStore::new(&path).unwrap();
    store
        .put("test", &json!({"retained":true}), Some("existing"))
        .unwrap();
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute_batch("UPDATE store_metadata SET value='corrupt'; DROP INDEX records_kind;")
        .unwrap();
    drop(connection);
    assert!(
        RunStore::new(&path)
            .unwrap_err()
            .contains("Unsupported observation index version")
    );
    let connection = rusqlite::Connection::open(path).unwrap();
    assert_eq!(
        connection
            .query_row("SELECT payload FROM records", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "{\"retained\": true}"
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE name='records_kind'",
                [],
                |row| row.get::<_, u64>(0)
            )
            .unwrap(),
        0
    );
}

#[test]
fn unsafe_initialisation_nodes_are_rejected_before_database_or_key_creation() {
    for kind in ["public", "directory", "symlink", "hardlink", "fifo"] {
        let directory = Directory::new();
        let path = directory.database();
        let lock = suffix(&path, ".initialisation.lock");
        match kind {
            "directory" => fs::create_dir(&lock).unwrap(),
            "public" => {
                fs::write(&lock, "unchanged").unwrap();
                fs::set_permissions(&lock, fs::Permissions::from_mode(0o644)).unwrap();
            }
            "fifo" => rustix::fs::mkfifoat(
                rustix::fs::CWD,
                &lock,
                rustix::fs::Mode::from_raw_mode(0o600),
            )
            .unwrap(),
            _ => {
                let target = directory.0.join("target");
                fs::write(&target, "unchanged").unwrap();
                fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).unwrap();
                if kind == "symlink" {
                    std::os::unix::fs::symlink(&target, &lock).unwrap();
                } else {
                    fs::hard_link(&target, &lock).unwrap();
                }
            }
        }
        assert!(RunStore::new(&path).is_err(), "{kind}");
        assert!(!path.exists());
        assert!(!suffix(&path, ".binding-key").exists());
    }
}

#[test]
fn private_identity_and_sidecars_are_revalidated_on_every_operation() {
    for kind in [
        "database-replace",
        "database-mode",
        "database-hardlink",
        "key-replace",
        "key-modify",
        "sidecar-fifo",
        "sidecar-symlink",
        "parent-replace",
    ] {
        let directory = Directory::new();
        let path = directory.database();
        let store = RunStore::new(&path).unwrap();
        store
            .put("test", &json!({"retained":true}), Some("existing"))
            .unwrap();
        let key = suffix(&path, ".binding-key");
        match kind {
            "database-replace" => {
                fs::rename(&path, suffix(&path, ".old")).unwrap();
                fs::write(&path, []).unwrap();
                fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
            }
            "database-mode" => {
                fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap()
            }
            "database-hardlink" => fs::hard_link(&path, suffix(&path, ".linked")).unwrap(),
            "key-replace" => {
                let bytes = fs::read(&key).unwrap();
                fs::rename(&key, suffix(&key, ".old")).unwrap();
                fs::write(&key, bytes).unwrap();
                fs::set_permissions(&key, fs::Permissions::from_mode(0o600)).unwrap();
            }
            "key-modify" => fs::write(&key, [0; 32]).unwrap(),
            "sidecar-fifo" => rustix::fs::mkfifoat(
                rustix::fs::CWD,
                suffix(&path, "-wal"),
                rustix::fs::Mode::from_raw_mode(0o600),
            )
            .unwrap(),
            "sidecar-symlink" => std::os::unix::fs::symlink(&key, suffix(&path, "-wal")).unwrap(),
            _ => {
                let old = suffix(&directory.0, ".old");
                fs::rename(&directory.0, &old).unwrap();
                fs::DirBuilder::new()
                    .mode(0o700)
                    .create(&directory.0)
                    .unwrap();
                fs::rename(old.join("runs.sqlite3"), &path).unwrap();
                fs::rename(old.join("runs.sqlite3.binding-key"), &key).unwrap();
                let _ = fs::remove_dir_all(old);
            }
        }
        assert!(store.get("existing", None).is_err(), "{kind}");
        assert!(store.list(None, 1).is_err(), "{kind}");
        assert!(store.find_observations(&filter()).is_err(), "{kind}");
        assert!(
            store.put("test", &json!({}), Some("new")).is_err(),
            "{kind}"
        );
    }
}

#[test]
fn strict_json_is_checked_when_reading_an_existing_database() {
    let directory = Directory::new();
    let path = directory.database();
    let store = RunStore::new(&path).unwrap();
    for (index, payload) in [
        "{\"duplicate\":1,\"duplicate\":2}",
        "{\"x\":NaN}",
        "{\"x\":1e9999}",
        "[] trailing",
    ]
    .into_iter()
    .enumerate()
    {
        let connection = rusqlite::Connection::open(&path).unwrap();
        connection
            .execute(
                "INSERT INTO records VALUES (?, 'test', 'now', ?)",
                rusqlite::params![format!("bad-{index}"), payload],
            )
            .unwrap();
        drop(connection);
        assert!(store.get(&format!("bad-{index}"), None).is_err());
    }
}

#[test]
fn aggregate_records_larger_than_default_observation_budgets_remain_durable() {
    let directory = Directory::new();
    let store = RunStore::new(directory.database()).unwrap();
    let payload =
        json!({"aggregate":"x".repeat(5*1024*1024),"observations":vec![json!({"value":1});60_000]});
    let id = store.put("collection", &payload, None).unwrap();
    assert_eq!(store.get(&id, Some("collection")).unwrap(), payload);
}
