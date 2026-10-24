use eal_coordination::{
    lab_store::{LabError, LabStore},
    protocol::{self, MAX_BYTES, MAX_DEPTH},
};
use serde_json::json;
use std::{path::PathBuf, sync::Arc};

struct Sandbox(PathBuf);
impl Sandbox {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("eal-lab-store-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn concurrent_senders_atomically_spend_one_remaining_message() {
    let sandbox = Sandbox::new();
    let store = LabStore::new(sandbox.0.join("lab.sqlite")).unwrap();
    store
        .create_run(&json!({"run_id":"r","agents":["a","b"],"max_messages":1}))
        .unwrap();
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let handles=(0..2).map(|id| {
        let store=store.clone();let barrier=Arc::clone(&barrier);
        std::thread::spawn(move||{
            let message=protocol::admit_envelope(&json!({"message_id":format!("m{id}"),"run_id":"r","sender":"a","recipient":"b","version":"1","kind":"request","payload":{"id":id}}),MAX_BYTES,MAX_DEPTH).unwrap();
            barrier.wait(); store.send_admitted(&message,"a")
        })
    }).collect::<Vec<_>>();
    let results = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(results.iter().filter(|v| v.is_ok()).count(), 1);
    assert_eq!(results.iter().filter(|v|matches!(v,Err(LabError{message,..}) if message=="communication budget exhausted")).count(),1);
    assert_eq!(store.inspect("r").unwrap()["usage"]["messages"], 1);
}

#[test]
fn lab_database_has_original_tables_and_retains_no_runstore_key() {
    let sandbox = Sandbox::new();
    let path = sandbox.0.join("lab.sqlite");
    let store = LabStore::new(&path).unwrap();
    store
        .create_run(&json!({"run_id":"r","agents":["a","b"]}))
        .unwrap();
    let db = rusqlite::Connection::open(&path).unwrap();
    let mut query = db
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .unwrap();
    let tables = query
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(
        tables,
        ["calls", "messages", "protocols", "runs", "sqlite_sequence"]
    );
    assert!(
        !std::fs::read_dir(&sandbox.0).unwrap().any(|e| e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains("binding"))
    );
}
