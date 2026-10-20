//! Exact production-source oracle; no Python or external processes run here.
use eal_coordination::ledger::canonical_json;
use serde_json::Value;

fn reference() -> Value {
    eal_core::json::parse(
        include_str!("../../../fixtures/coordination/reference.json"),
        &eal_core::ExecutionLimits::default(),
    )
    .unwrap()
}

fn compare(case: &str, actual: Result<Value, String>, expected: &Value) {
    if let Some(error) = expected.get("error") {
        assert_eq!(actual, Err(error.as_str().unwrap().to_owned()), "{case}");
    } else {
        assert_eq!(actual, Ok(expected["value"].clone()), "{case}");
    }
}

#[test]
fn canonical_bytes_match_all_pinned_coordination_cases() {
    let fixture = reference();
    let cases = fixture["canonical"].as_array().unwrap();
    assert_eq!(cases.len(), 13);
    for case in cases {
        let raw = case["input"].as_str().unwrap();
        let input = eal_core::json::parse(raw, &eal_core::ExecutionLimits::default()).unwrap();
        assert_eq!(
            canonical_json(&input).unwrap(),
            case["expected"].as_str().unwrap(),
            "{raw}"
        );
    }
}

#[test]
fn all_fixed_task_anchor_finding_and_answer_contracts_match() {
    use eal_coordination::{
        contracts::{TaskSpec, make_finding, validate_anchor},
        profiles::validate_answer,
    };
    let fixture = reference();
    let cases = fixture["contracts"].as_array().unwrap();
    assert_eq!(cases.len(), 52);
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let input = &case["value"];
        let actual = match case["op"].as_str().unwrap() {
            "task" => TaskSpec::from_value(input).and_then(|task| task.to_value()),
            "anchor" => validate_anchor(input).and_then(|task| task.to_value()),
            "finding" => make_finding(
                input,
                &case["check"],
                Some(&case["assessments"]),
                None,
                None,
                None,
            ),
            "answer" => validate_answer(&case["anchor"], &case["finding"], input).map(Value::Array),
            other => panic!("Unknown contract fixture operation {other}"),
        };
        if name == "anchor-invalid-time" {
            // The native ISO parser has its own diagnostic. Keep both exact
            // texts visible and require the same rejection before digest use.
            assert_eq!(case["expected"]["error"], "month must be in 1..12");
            assert_eq!(actual, Err("Invalid ISO-8601 date".to_owned()));
        } else {
            compare(name, actual, &case["expected"]);
        }
    }
}

#[cfg(unix)]
mod persistent {
    use super::*;
    use eal_coordination::{
        bus::{JsonBus, Mailbox, MailboxLimits},
        ledger::EffectLedger,
        state::State,
    };
    use serde::Serialize;
    use serde_json::json;
    use std::{cell::RefCell, path::PathBuf, sync::Arc};

    struct Workspace(PathBuf);
    impl Workspace {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "eal-coordination-reference-{}",
                uuid::Uuid::new_v4()
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Workspace {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    enum Instance {
        Ledger(EffectLedger),
        State(State),
        Bus(JsonBus),
    }
    fn value<T: Serialize>(result: Result<T, String>) -> Result<Value, String> {
        result.and_then(|value| serde_json::to_value(value).map_err(|error| error.to_string()))
    }
    fn text<'a>(value: &'a Value, key: &str) -> &'a str {
        value[key].as_str().unwrap()
    }
    fn number(value: &Value, key: &str, default: usize) -> usize {
        value
            .get(key)
            .map_or(default, |value| value.as_u64().unwrap() as usize)
    }

    impl Instance {
        fn new(group: &str, path: &std::path::Path, clock: &str) -> Self {
            match group {
                "ledger" => Self::Ledger(EffectLedger::new(path).unwrap()),
                "state" => {
                    let clock = clock.to_owned();
                    Self::State(State::with_clock(path, Arc::new(move || clock.clone())).unwrap())
                }
                "bus" => Self::Bus(JsonBus::new(path).unwrap()),
                other => panic!("Unknown fixture group {other}"),
            }
        }

        fn run(
            &self,
            step: &Value,
            path: &std::path::Path,
            callbacks: &RefCell<Vec<Value>>,
        ) -> Result<Value, String> {
            let op = text(step, "op");
            if op == "reserve_effect" {
                return match self {
                    Self::Ledger(ledger) => {
                        value(ledger.reserve_effect(text(step, "key"), &step["payload"]))
                    }
                    Self::State(state) => {
                        value(state.reserve_effect(text(step, "key"), &step["payload"]))
                    }
                    Self::Bus(_) => panic!("Ledger fixture operation on bus"),
                };
            }
            if op == "complete_effect" {
                let outbox = step["outbox"].as_array().unwrap();
                return match self {
                    Self::Ledger(ledger) => {
                        value(ledger.complete_effect(text(step, "key"), &step["result"], outbox))
                    }
                    Self::State(state) => {
                        value(state.complete_effect(text(step, "key"), &step["result"], outbox))
                    }
                    Self::Bus(_) => panic!("Ledger fixture operation on bus"),
                };
            }
            if op == "drain" {
                let send = |message: &Value| {
                    callbacks.borrow_mut().push(json!({"send":message}));
                    if step.get("fail_send") == message.get("message_id") {
                        return Err("Synthetic send confirmation unavailable".to_owned());
                    }
                    if step["reentrant"] == true {
                        EffectLedger::new(path)?.reserve_effect(
                            &format!("side/{}", text(message, "message_id")),
                            &json!({"synthetic":true}),
                        )?;
                    }
                    Ok(())
                };
                let ack = |key: &str| {
                    callbacks.borrow_mut().push(json!({"ack":key}));
                    if step["fail_ack"].as_str() == Some(key) {
                        return Err("Synthetic ACK confirmation unavailable".to_owned());
                    }
                    Ok(())
                };
                return match self {
                    Self::Ledger(ledger) => value(ledger.drain_outbox(send, ack)),
                    Self::State(state) => value(state.drain_outbox(send, ack)),
                    Self::Bus(_) => panic!("Ledger fixture operation on bus"),
                };
            }
            match self {
                Self::State(state) => match op {
                    "config" => state.config(),
                    "initialize" => value(state.initialize(&step["config"])),
                    "event" => value(state.event(text(step, "kind"), &step["content"])),
                    "events" => value(state.events()),
                    "issue" => value(state.issue(&step["finding"])),
                    "finding" => state.finding(text(step, "identity")),
                    "reserve_call" => value(state.reserve_call(
                        text(step, "actor"),
                        text(step, "category"),
                        text(step, "operation"),
                        number(step, "maximum", 0),
                    )),
                    "calls" => value(state.calls()),
                    "cancel" => value(state.cancel()),
                    "cancelled" => value(state.cancelled()),
                    "effect" => value(state.effect(text(step, "key"))),
                    "effects" => value(state.effects()),
                    other => panic!("Unknown State fixture operation {other}"),
                },
                Self::Bus(bus) => match op {
                    "create" => bus.create(
                        text(step, "run_id"),
                        &step["participants"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|value| value.as_str().unwrap().to_owned())
                            .collect::<Vec<_>>(),
                        MailboxLimits {
                            max_messages: number(step, "max_messages", 8),
                            max_bytes: number(step, "max_bytes", 65536),
                            max_calls: number(step, "max_calls", 2),
                        },
                    ),
                    "send" => bus.send(text(step, "run_id"), &step["payload"], text(step, "actor")),
                    "receive" => bus.receive(
                        text(step, "run_id"),
                        text(step, "recipient"),
                        number(step, "limit", 8),
                    ),
                    "acknowledge" => bus.acknowledge(
                        text(step, "run_id"),
                        text(step, "message_id"),
                        text(step, "recipient"),
                    ),
                    "reserve" => bus.reserve(
                        text(step, "run_id"),
                        text(step, "actor"),
                        text(step, "call_id"),
                    ),
                    "finish" => bus.finish(
                        text(step, "run_id"),
                        text(step, "actor"),
                        text(step, "call_id"),
                        &step["evidence"],
                    ),
                    "status" => bus.status(text(step, "run_id")),
                    other => panic!("Unknown bus fixture operation {other}"),
                },
                Self::Ledger(_) => panic!("Unknown ledger fixture operation {op}"),
            }
        }
    }

    #[test]
    fn every_durable_step_matches_full_snapshot_and_callback_order() {
        let fixture = reference();
        let scenarios = fixture["scenarios"].as_array().unwrap();
        assert_eq!(scenarios.len(), 26);
        let mut steps = 0;
        let mut counts = [0; 3];
        for case in scenarios {
            let group = text(case, "group");
            counts[match group {
                "ledger" => 0,
                "state" => 1,
                "bus" => 2,
                other => panic!("Unknown group {other}"),
            }] += 1;
            let workspace = Workspace::new();
            let path = workspace.0.join("state.sqlite");
            let clock = text(&fixture, "clock");
            let mut instance = Instance::new(group, &path, clock);
            for (index, step) in case["steps"].as_array().unwrap().iter().enumerate() {
                steps += 1;
                let label = format!(
                    "{group}/{} step {index} {}",
                    text(case, "name"),
                    text(step, "op")
                );
                let callbacks = RefCell::new(Vec::new());
                let actual = if step["op"] == "reopen" {
                    instance = Instance::new(group, &path, clock);
                    Ok(Value::Null)
                } else {
                    instance.run(step, &path, &callbacks)
                };
                compare(&label, actual, &step["expected"]);
                if step["op"] == "drain" {
                    assert_eq!(
                        Value::Array(callbacks.into_inner()),
                        step["expected"]["callbacks"],
                        "{label} callbacks"
                    );
                }
            }
        }
        assert_eq!(counts, [14, 4, 8]);
        assert_eq!(steps, 175);
    }
}
