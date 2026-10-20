//! Independent crash witnesses through real process exit and a durable mailbox.
#![cfg(unix)]
use eal_coordination::{
    bus::{JsonBus, Mailbox, MailboxLimits},
    ledger::{EffectLedger, EffectStatus},
};
use serde_json::{Value, json};
use std::{fs, path::PathBuf, process::Command};

const EFFECT: &str = "synthetic-run/assessor/request";
fn incoming() -> Value {
    json!({"schema":"EAL/MAS-coordination/3","message_id":"request","kind":"request",
        "sender":"reviewer","recipient":"assessor","task_id":"synthetic-task"})
}
fn reply() -> Value {
    json!({"schema":"EAL/MAS-coordination/3","message_id":"reply","kind":"outcome",
        "sender":"assessor","recipient":"reviewer","reply_to":"request","synthetic":true})
}
fn result() -> Value {
    json!({"raw_response":"synthetic retained bytes", "accepted":true, "execution_mode":"software_test"})
}
struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("eal-coordination-crash-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn coordination_crash_child() {
    let Some(path) = std::env::var_os("EAL_COORDINATION_CRASH_TEST_PATH") else {
        return;
    };
    let path = PathBuf::from(path);
    let boundary = std::env::var("EAL_COORDINATION_CRASH_TEST_BOUNDARY").unwrap();
    let ledger = EffectLedger::new(path.join("effects.sqlite")).unwrap();
    let bus = JsonBus::new(path.join("mailbox.sqlite")).unwrap();
    ledger
        .drain_outbox(
            |message| {
                if boundary == "before_send" {
                    std::process::exit(86);
                }
                bus.send("run", message, "assessor")?;
                if boundary == "after_send" {
                    std::process::exit(86);
                }
                Ok(())
            },
            |key| {
                assert_eq!(key, EFFECT);
                if boundary == "before_ack" {
                    std::process::exit(86);
                }
                bus.acknowledge("run", "request", "assessor")?;
                if boundary == "after_ack" {
                    std::process::exit(86);
                }
                Ok(())
            },
        )
        .unwrap();
    panic!("Crash helper failed to reach the selected durable boundary");
}

#[test]
fn process_death_at_each_send_ack_boundary_resumes_delivery_without_reexecution() {
    for boundary in ["before_send", "after_send", "before_ack", "after_ack"] {
        let workspace = Workspace::new();
        let ledger_path = workspace.0.join("effects.sqlite");
        let bus_path = workspace.0.join("mailbox.sqlite");
        let ledger = EffectLedger::new(&ledger_path).unwrap();
        let bus = JsonBus::new(&bus_path).unwrap();
        bus.create(
            "run",
            &["reviewer".into(), "assessor".into()],
            MailboxLimits {
                max_messages: 2,
                ..Default::default()
            },
        )
        .unwrap();
        bus.send("run", &incoming(), "reviewer").unwrap();
        assert_eq!(
            bus.receive("run", "assessor", 8)
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            ledger.reserve_effect(EFFECT, &incoming()).unwrap().status,
            EffectStatus::New
        );
        // This durable marker represents the single independently performed
        // external effect. Recovery code has no callback that can perform it.
        fs::write(workspace.0.join("external-effect"), b"performed once").unwrap();
        ledger
            .complete_effect(EFFECT, &result(), &[reply()])
            .unwrap();
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "coordination_crash_child", "--nocapture"])
            .env("EAL_COORDINATION_CRASH_TEST_PATH", &workspace.0)
            .env("EAL_COORDINATION_CRASH_TEST_BOUNDARY", boundary)
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(86),
            "{boundary}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let restarted = EffectLedger::new(&ledger_path).unwrap();
        let persisted = restarted.reserve_effect(EFFECT, &incoming()).unwrap();
        assert_eq!(persisted.status, EffectStatus::Committed, "{boundary}");
        assert_eq!(persisted.result, Some(result()), "{boundary}");
        assert_eq!(persisted.outbox, [reply()], "{boundary}");
        assert!(!persisted.acknowledged, "{boundary}");
        let bus = JsonBus::new(&bus_path).unwrap();
        let mut confirmations = Vec::new();
        let report = restarted
            .drain_outbox(
                |message| {
                    confirmations.push(bus.send("run", message, "assessor")?);
                    Ok(())
                },
                |_| bus.acknowledge("run", "request", "assessor").map(|_| ()),
            )
            .unwrap();
        let retries = usize::from(matches!(boundary, "before_send" | "after_send"));
        assert_eq!(confirmations.len(), retries, "{boundary}");
        if retries == 1 {
            assert_eq!(
                confirmations[0]["duplicate"],
                boundary == "after_send",
                "{boundary}"
            );
        }
        assert_eq!(report.confirmed_sends.len(), retries, "{boundary}");
        assert_eq!(report.acknowledged_effects, [EFFECT], "{boundary}");
        assert_eq!(
            bus.status("run").unwrap()["usage"]["messages"],
            2,
            "{boundary}"
        );
        assert!(
            bus.receive("run", "assessor", 8)
                .unwrap()
                .as_array()
                .unwrap()
                .is_empty(),
            "{boundary}"
        );
        let delivered = bus.receive("run", "reviewer", 8).unwrap();
        assert_eq!(delivered.as_array().unwrap().len(), 1, "{boundary}");
        assert_eq!(delivered[0]["message"]["payload"], reply(), "{boundary}");
        assert!(
            restarted
                .reserve_effect(EFFECT, &incoming())
                .unwrap()
                .acknowledged,
            "{boundary}"
        );
        assert_eq!(
            fs::read(workspace.0.join("external-effect")).unwrap(),
            b"performed once",
            "{boundary}"
        );
    }
}
