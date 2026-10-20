//! Native crash, transaction and concurrent reservation checks. No model runs.
#![cfg(unix)]
use eal_coordination::{
    bus::{JsonBus, Mailbox, MailboxLimits},
    ledger::{EffectLedger, EffectStatus, canonical_json},
    state::State,
};
use eal_runtime::private_sqlite::PrivateDatabase;
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{Arc, Barrier},
    time::Duration,
};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "eal-coordination-persistence-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn request() -> Value {
    json!({"message_id":"request","sender":"reviewer","recipient":"assessor","kind":"request"})
}
fn reply(id: &str) -> Value {
    json!({"message_id":id,"sender":"assessor","recipient":"reviewer","kind":"outcome","reply_to":"request"})
}

#[test]
fn concurrent_reservations_authorize_one_new_effect_and_preserve_number_identity() {
    let directory = Directory::new();
    let ledger = Arc::new(EffectLedger::new(directory.path("effects.sqlite")).unwrap());
    let barrier = Arc::new(Barrier::new(4));
    let handles = (0..4)
        .map(|_| {
            let ledger = Arc::clone(&ledger);
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                ledger
                    .reserve_effect("run/assessor/request", &request())
                    .unwrap()
                    .status
            })
        })
        .collect::<Vec<_>>();
    let results = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        results
            .iter()
            .filter(|status| **status == EffectStatus::New)
            .count(),
        1
    );
    assert_eq!(
        results
            .iter()
            .filter(|status| **status == EffectStatus::Unknown)
            .count(),
        3
    );
    assert_ne!(
        canonical_json(&json!(1)).unwrap(),
        canonical_json(&json!(1.0)).unwrap()
    );
    assert_ne!(
        canonical_json(&json!(true)).unwrap(),
        canonical_json(&json!(1)).unwrap()
    );
    ledger.reserve_effect("typed", &json!({"n":1})).unwrap();
    assert_eq!(
        ledger
            .reserve_effect("typed", &json!({"n":1.0}))
            .unwrap_err(),
        "Inbound identity collision: canonical payload changed"
    );
}

#[test]
fn interrupted_outbox_commit_rolls_back_result_and_every_reply() {
    let directory = Directory::new();
    let path = directory.path("effects.sqlite");
    let ledger = EffectLedger::new(&path).unwrap();
    ledger.reserve_effect("effect", &request()).unwrap();
    let database = PrivateDatabase::new(&path, Duration::from_secs(10)).unwrap();
    database.with_transaction(|db|db.execute_batch("CREATE TRIGGER fail_outbox BEFORE INSERT ON effect_outbox WHEN NEW.position = 1 BEGIN SELECT RAISE(ABORT, 'simulated commit failure'); END;").map_err(|e|e.to_string())).unwrap();
    assert!(
        ledger
            .complete_effect(
                "effect",
                &json!({"retained":"first response"}),
                &[reply("one"), reply("two")]
            )
            .unwrap_err()
            .contains("simulated commit failure")
    );
    let replay = EffectLedger::new(path)
        .unwrap()
        .reserve_effect("effect", &request())
        .unwrap();
    assert_eq!(replay.status, EffectStatus::Unknown);
    assert!(replay.result.is_none());
    assert!(replay.outbox.is_empty());
}

#[test]
fn callback_writes_are_unlocked_and_only_unconfirmed_suffix_retries() {
    let directory = Directory::new();
    let ledger = EffectLedger::new(directory.path("effects.sqlite")).unwrap();
    ledger.reserve_effect("effect", &request()).unwrap();
    let committed = ledger
        .complete_effect(
            "effect",
            &json!({"raw":"retained"}),
            &[reply("one"), reply("two")],
        )
        .unwrap();
    let mut sent = Vec::new();
    let error = ledger
        .drain_outbox(
            |message| {
                EffectLedger::new(ledger.path())
                    .unwrap()
                    .reserve_effect(
                        &format!("side/{}", message["message_id"].as_str().unwrap()),
                        &json!(true),
                    )
                    .unwrap();
                sent.push(message.clone());
                if sent.len() == 2 {
                    Err("second confirmation lost".into())
                } else {
                    Ok(())
                }
            },
            |_| panic!("unconfirmed suffix must block ACK"),
        )
        .unwrap_err();
    assert_eq!(error, "second confirmation lost");
    let mut resent = Vec::new();
    let mut acknowledged = Vec::new();
    let report = ledger
        .drain_outbox(
            |message| {
                resent.push(message.clone());
                Ok(())
            },
            |key| {
                acknowledged.push(key.to_owned());
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(resent, [reply("two")]);
    assert_eq!(acknowledged, ["effect"]);
    assert_eq!(report.confirmed_sends[0].message_id, "two");
    let mut changed = committed;
    changed.outbox[0]["kind"] = json!("caller mutation");
    let replay = ledger.reserve_effect("effect", &request()).unwrap();
    assert_eq!(replay.outbox[0], reply("one"));
    assert!(replay.acknowledged);
    let report = ledger
        .drain_outbox(
            |_| panic!("no resend after ACK"),
            |_| panic!("no repeat ACK after confirmation"),
        )
        .unwrap();
    assert!(report.confirmed_sends.is_empty());
}

#[test]
fn unknown_effect_never_sends_or_acknowledges_without_explicit_reconciliation() {
    let directory = Directory::new();
    let path = directory.path("effects.sqlite");
    EffectLedger::new(&path)
        .unwrap()
        .reserve_effect("effect", &request())
        .unwrap();
    let ledger = EffectLedger::new(path).unwrap();
    let replay = ledger.reserve_effect("effect", &request()).unwrap();
    assert_eq!(replay.status, EffectStatus::Unknown);
    assert_eq!(replay.diagnosis.as_deref(), Some("effect_outcome_unknown"));
    let report = ledger
        .drain_outbox(
            |_| panic!("unknown must not send"),
            |_| panic!("unknown must not ACK"),
        )
        .unwrap();
    assert!(report.acknowledged_effects.is_empty());
    ledger
        .complete_effect(
            "effect",
            &json!({"raw_response":"independently retained","admission":"rejected"}),
            &[],
        )
        .unwrap();
    let report = ledger
        .drain_outbox(|_| panic!("empty outbox"), |_| Ok(()))
        .unwrap();
    assert_eq!(report.acknowledged_effects, ["effect"]);
}

#[test]
fn schema_files_remain_separate_from_observation_storage_and_state_views_are_detached() {
    let directory = Directory::new();
    let path = directory.path("coordination.sqlite");
    let state = State::with_clock(&path, Arc::new(|| "2040-01-01T00:00:00Z".into())).unwrap();
    assert!(state.config().is_err());
    state.initialize(&json!({"run_id":"run"})).unwrap();
    assert!(state.initialize(&json!({"run_id":"other"})).is_err());
    state
        .issue(&json!({"finding_id":"finding","check":{"ok":false}}))
        .unwrap();
    let mut detached = state.finding("finding").unwrap();
    detached["check"]["ok"] = json!(true);
    assert_eq!(state.finding("finding").unwrap()["check"]["ok"], false);
    state
        .reserve_call("assessor", "mcp", "eal_sources", 1)
        .unwrap();
    assert_eq!(
        state
            .reserve_call("assessor", "mcp", "eal_sources", 1)
            .unwrap_err(),
        "mcp call budget exhausted for assessor"
    );
    assert_eq!(state.calls().unwrap().len(), 1);
    state.cancel().unwrap();
    assert!(state.cancelled().unwrap());
    assert_eq!(
        state.events().unwrap()[0],
        json!({"sequence":1,"at":"2040-01-01T00:00:00Z","kind":"cancelled","data":{}})
    );
    let database = PrivateDatabase::new(&path, Duration::from_secs(10)).unwrap();
    let tables = database
        .with_connection(|db| {
            let mut query = db
                .prepare("SELECT name FROM sqlite_schema WHERE type='table' ORDER BY name")
                .map_err(|e| e.to_string())?;
            query
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())
        })
        .unwrap();
    assert_eq!(
        tables,
        [
            "calls",
            "effect_outbox",
            "effects",
            "events",
            "findings",
            "run_config"
        ]
    );
    assert!(!directory.path("coordination.sqlite.binding-key").exists());
    let bus = JsonBus::new(directory.path("mailbox.sqlite")).unwrap();
    bus.create(
        "run",
        &["reviewer".into(), "assessor".into()],
        MailboxLimits::default(),
    )
    .unwrap();
    assert_eq!(bus.status("run").unwrap()["usage"]["messages"], 0);
}

#[test]
fn child_exits_after_durable_remote_send_before_confirmation() {
    let Some(path) = std::env::var_os("EAL_TEST_OUTBOX_CRASH_DIRECTORY") else {
        return;
    };
    let path = PathBuf::from(path);
    let ledger = EffectLedger::new(path.join("effects.sqlite")).unwrap();
    let bus = JsonBus::new(path.join("mailbox.sqlite")).unwrap();
    ledger
        .drain_outbox(
            |message| {
                bus.send("run", message, "assessor")?;
                std::process::exit(73)
            },
            |_| panic!("child must exit before ACK"),
        )
        .unwrap();
    panic!("child must exit after send");
}

#[test]
fn abrupt_exit_replays_stable_delivery_without_repeating_external_effect() {
    let directory = Directory::new();
    let ledger = EffectLedger::new(directory.path("effects.sqlite")).unwrap();
    let bus = JsonBus::new(directory.path("mailbox.sqlite")).unwrap();
    bus.create(
        "run",
        &["reviewer".into(), "assessor".into()],
        MailboxLimits::default(),
    )
    .unwrap();
    bus.send("run", &request(), "reviewer").unwrap();
    bus.receive("run", "assessor", 8).unwrap();
    ledger
        .reserve_effect("run/assessor/request", &request())
        .unwrap();
    ledger
        .complete_effect(
            "run/assessor/request",
            &json!({"raw_response":"already committed"}),
            &[reply("reply")],
        )
        .unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "child_exits_after_durable_remote_send_before_confirmation",
        ])
        .env("EAL_TEST_OUTBOX_CRASH_DIRECTORY", &directory.0)
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(73),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let reopened = EffectLedger::new(ledger.path()).unwrap();
    let replay = reopened
        .reserve_effect("run/assessor/request", &request())
        .unwrap();
    assert_eq!(replay.status, EffectStatus::Committed);
    assert!(!replay.acknowledged);
    let mut confirmations = Vec::new();
    reopened
        .drain_outbox(
            |message| {
                confirmations.push(bus.send("run", message, "assessor")?);
                Ok(())
            },
            |_| bus.acknowledge("run", "request", "assessor").map(|_| ()),
        )
        .unwrap();
    assert_eq!(confirmations.len(), 1);
    assert_eq!(confirmations[0]["duplicate"], true);
    assert_eq!(bus.status("run").unwrap()["usage"]["messages"], 2);
    assert!(
        reopened
            .reserve_effect("run/assessor/request", &request())
            .unwrap()
            .acknowledged
    );
}
