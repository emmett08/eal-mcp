use eal_coordination::{
    lab_store::{LabError, LabStore},
    protocol::{self, MAX_BYTES, MAX_DEPTH},
};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

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
fn pinned_toolkit_transactions_and_error_order_match_exactly() {
    let fixture: Value = eal_core::json::parse(
        include_str!("../../../fixtures/coordination/lab-store-reference.json"),
        &eal_core::ExecutionLimits {
            record_bytes: 64 * 1024 * 1024,
            json_nodes: 1_000_000,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        fixture["store_sha256"],
        "f2510cbad31646c372d0006cb50be3f9d3f027a540c9142a0a4a09c1ea84b638"
    );
    let mut witnessed = 0;
    for scenario in fixture["scenarios"].as_array().unwrap() {
        let sandbox = Sandbox::new();
        let path = sandbox.0.join("lab.sqlite");
        let time = Arc::new(AtomicU64::new(1000.25f64.to_bits()));
        let selected = Arc::clone(&time);
        let store = LabStore::with_clock(
            &path,
            Arc::new(move || f64::from_bits(selected.load(Ordering::SeqCst))),
        )
        .unwrap();
        for step in scenario["steps"].as_array().unwrap() {
            witnessed += 1;
            let args = &step["arguments"];
            let string = |key: &str| args[key].as_str().unwrap();
            let actual = match step["operation"].as_str().unwrap() {
                "create_run" => store.create_run(args),
                "send" => store.send_admitted(
                    &protocol::admit_envelope(&args["envelope"], MAX_BYTES, MAX_DEPTH).unwrap(),
                    string("actor"),
                ),
                "receive" => store.receive(
                    string("run_id"),
                    string("recipient"),
                    args.get("limit").unwrap_or(&json!(10)),
                ),
                "ack" => store.ack(string("run_id"), string("message_id"), string("recipient")),
                "inspect" => store.inspect(string("run_id")),
                "reserve_call" => {
                    store.reserve_call(string("run_id"), string("actor"), string("call_id"))
                }
                "finish_call" => store.finish_call(
                    string("run_id"),
                    string("actor"),
                    string("call_id"),
                    &args["evidence"],
                ),
                "advance" => {
                    let now = f64::from_bits(time.load(Ordering::SeqCst));
                    time.store((now + args.as_f64().unwrap()).to_bits(), Ordering::SeqCst);
                    Ok(Value::Null)
                }
                "set_active" => {
                    rusqlite::Connection::open(&path)
                        .unwrap()
                        .execute(
                            "UPDATE runs SET active=? WHERE id='run'",
                            [args.as_str().unwrap()],
                        )
                        .unwrap();
                    Ok(Value::Null)
                }
                unknown => panic!("Unknown fixture operation {unknown}"),
            };
            match (actual, step.get("error")) {
                (Ok(value), None) => assert_eq!(
                    value, step["result"],
                    "{} {}",
                    scenario["name"], step["operation"]
                ),
                (Err(error), Some(expected)) => assert_eq!(
                    error.to_value(),
                    *expected,
                    "{} {}",
                    scenario["name"],
                    step["operation"]
                ),
                (actual, expected) => panic!(
                    "{} {}: {actual:?} != {expected:?}",
                    scenario["name"], step["operation"]
                ),
            }
        }
    }
    assert!(witnessed >= 70);
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
