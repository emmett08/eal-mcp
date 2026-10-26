//! Native session protocol, graph routing and durable uncertainty checks.
use eal_coordination_contracts::{
    ClientMessage, DebateConfig, Dispatch, PeerValue, RoundResult, SCHEMA_VERSION, ServerMessage,
    SessionSpec, Topology,
};
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!("eal-debate-session-{}", uuid::Uuid::new_v4())))
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_eal-coordinate")
}
fn process(directory: &Path) -> Child {
    Command::new(binary())
        .args(["debate-session", "--run"])
        .arg(directory)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
}
fn spec(name: &str) -> SessionSpec {
    SessionSpec {
        schema_version: SCHEMA_VERSION.into(),
        debate: DebateConfig {
            participants: 4,
            rounds: 1,
            topology: Topology::preset(name, 4).unwrap(),
            adaptation: Default::default(),
        },
        max_payload_bytes: 1024,
    }
}
fn send(input: &mut ChildStdin, message: &ClientMessage) {
    serde_json::to_writer(&mut *input, message).unwrap();
    input.write_all(b"\n").unwrap();
    input.flush().unwrap();
}
fn receive(reader: &mut impl BufRead) -> ServerMessage {
    let mut line = String::new();
    assert!(reader.read_line(&mut line).unwrap() > 0);
    serde_json::from_str(&line).unwrap()
}
fn dispatch(reader: &mut impl BufRead) -> Dispatch {
    let ServerMessage::Dispatch { dispatch } = receive(reader) else {
        panic!("expected dispatch")
    };
    dispatch
}
fn values(round: usize) -> Vec<PeerValue> {
    (0..4)
        .rev()
        .map(|participant| PeerValue {
            participant,
            round,
            payload: json!({"marker":format!("{participant}:{round}")}),
        })
        .collect()
}
fn inspect(directory: &Path, operation: &str) -> Value {
    let output = Command::new(binary())
        .args([operation, "--run"])
        .arg(directory)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn native_session_executes_every_topology_and_retains_graph_dispatches() {
    for topology in [
        "fully_connected",
        "grouped",
        "chain",
        "ring",
        "star",
        "tree",
        "layered",
        "sparse",
        "bipartite",
    ] {
        let directory = Directory::new();
        let mut child = process(&directory.0);
        let mut input = child.stdin.take().unwrap();
        let mut output = BufReader::new(child.stdout.take().unwrap());
        let selected = spec(topology);
        let edges = selected.debate.graphs().unwrap()[0].edges.clone();
        send(&mut input, &ClientMessage::Start { spec: selected });
        let first = dispatch(&mut output);
        assert_eq!(first.round, 0);
        assert_eq!(first.calls.len(), 4);
        assert!(
            first.calls.iter().all(|call| call.peers.is_empty()
                && call.previous.is_none()
                && call.edges.is_empty())
        );
        send(
            &mut input,
            &ClientMessage::Results {
                result: RoundResult {
                    round: 0,
                    responses: values(0),
                },
            },
        );
        let peer = dispatch(&mut output);
        assert_eq!(peer.round, 1);
        for call in &peer.calls {
            let expected = edges
                .iter()
                .filter(|edge| edge.to == call.participant)
                .map(|edge| edge.from)
                .collect::<Vec<_>>();
            assert_eq!(call.incoming, expected, "{topology}");
            assert_eq!(call.edges, edges, "{topology}");
            assert_eq!(
                call.previous,
                Some(json!({"marker":format!("{}:0",call.participant)}))
            );
            assert_eq!(
                call.peers
                    .iter()
                    .map(|peer| peer.participant)
                    .collect::<Vec<_>>(),
                expected
            );
            assert!(call.peers.iter().all(|peer| peer.round == 0
                && peer.payload == json!({"marker":format!("{}:0",peer.participant)})));
        }
        send(
            &mut input,
            &ClientMessage::Results {
                result: RoundResult {
                    round: 1,
                    responses: values(1),
                },
            },
        );
        assert!(matches!(
            receive(&mut output),
            ServerMessage::Completed {
                rounds: 2,
                calls: 8
            }
        ));
        drop(input);
        assert!(child.wait().unwrap().success());
        let status = inspect(&directory.0, "debate-status");
        assert_eq!(status["status"], "completed");
        assert_eq!(status["committed_calls"], 8);
        assert_eq!(status["uncertain_calls"], 0);
        let export = inspect(&directory.0, "debate-export");
        assert_eq!(export["effects"].as_array().unwrap().len(), 8);
        assert!(
            export["effects"]
                .as_array()
                .unwrap()
                .iter()
                .all(|effect| effect["result"].get("payload").is_none()
                    && effect["result"]["payload_sha256"].as_str().unwrap().len() == 64)
        );
        assert_eq!(
            export["config"]["spec"]["debate"]["topology"]["kind"],
            topology
        );
    }
}

#[test]
fn malformed_result_rejects_whole_round_and_preserves_the_exact_frame() {
    let directory = Directory::new();
    let mut child = process(&directory.0);
    let mut input = child.stdin.take().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    send(&mut input, &ClientMessage::Start { spec: spec("ring") });
    dispatch(&mut output);
    let mut responses = values(0);
    responses[0].participant = responses[1].participant;
    let message = ClientMessage::Results {
        result: RoundResult {
            round: 0,
            responses,
        },
    };
    let mut bytes = serde_json::to_vec(&message).unwrap();
    bytes.push(b'\n');
    input.write_all(&bytes).unwrap();
    input.flush().unwrap();
    assert!(matches!(receive(&mut output), ServerMessage::Failed { .. }));
    drop(input);
    assert!(!child.wait().unwrap().success());
    let status = inspect(&directory.0, "debate-status");
    assert_eq!(status["status"], "failed");
    assert_eq!(status["committed_calls"], 0);
    assert_eq!(status["uncertain_calls"], 4);
    let export = inspect(&directory.0, "debate-export");
    let frame = export["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|event| {
            event["kind"] == "session_frame"
                && event["data"]["direction"] == "in"
                && event["data"]["index"] == 2
        })
        .unwrap();
    assert_eq!(
        std::fs::read(directory.0.join(frame["data"]["path"].as_str().unwrap())).unwrap(),
        bytes
    );
    let output = Command::new(binary())
        .args(["debate-session", "--run"])
        .arg(&directory.0)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("empty directory"));
}

#[test]
fn input_loss_keeps_uncertain_reservations_and_frame_corruption_is_detected() {
    let directory = Directory::new();
    let mut child = process(&directory.0);
    let mut input = child.stdin.take().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    send(
        &mut input,
        &ClientMessage::Start {
            spec: spec("chain"),
        },
    );
    dispatch(&mut output);
    drop(input);
    assert!(matches!(receive(&mut output), ServerMessage::Failed { .. }));
    assert!(!child.wait().unwrap().success());
    let export = inspect(&directory.0, "debate-export");
    assert_eq!(export["status"]["uncertain_calls"], 4);
    std::fs::write(directory.0.join("frame-0000-in.bin"), b"tampered\n").unwrap();
    let output = Command::new(binary())
        .args(["debate-status", "--run"])
        .arg(&directory.0)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("frame content differs"));
}

#[test]
fn invalid_start_is_retained_and_status_does_not_require_a_frozen_valid_spec() {
    let directory = Directory::new();
    let mut child = process(&directory.0);
    let mut input = child.stdin.take().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    input
        .write_all(b"{\"kind\":\"start\",\"invalid\":true}\n")
        .unwrap();
    input.flush().unwrap();
    assert!(matches!(receive(&mut output), ServerMessage::Failed { .. }));
    drop(input);
    assert!(!child.wait().unwrap().success());
    let export = inspect(&directory.0, "debate-export");
    assert!(export["config"].is_null());
    assert_eq!(export["status"]["status"], "failed");
    assert_eq!(export["status"]["uncertain_calls"], 0);
}

#[test]
fn opaque_payloads_preserve_decimal_lexemes_negative_zero_and_literal_number_keys() {
    let directory = Directory::new();
    let mut child = process(&directory.0);
    let mut input = child.stdin.take().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    send(&mut input, &ClientMessage::Start { spec: spec("ring") });
    dispatch(&mut output);
    let payload = eal_collector_contracts::strict_json_exact(
        br#"{"decimal":1.0000000000000000000001,"signed":-0,"literal":{"$serde_json::private::Number":"1"}}"#,
        Default::default(),
    ).unwrap();
    let responses = (0..4)
        .map(|participant| PeerValue {
            participant,
            round: 0,
            payload: payload.clone(),
        })
        .collect();
    send(
        &mut input,
        &ClientMessage::Results {
            result: RoundResult {
                round: 0,
                responses,
            },
        },
    );
    let routed = dispatch(&mut output);
    let expected = serde_json::to_string(&payload).unwrap();
    for call in &routed.calls {
        assert_eq!(
            serde_json::to_string(call.previous.as_ref().unwrap()).unwrap(),
            expected
        );
        assert_eq!(
            serde_json::to_string(&call.peers[0].payload).unwrap(),
            expected
        );
    }
    send(
        &mut input,
        &ClientMessage::Results {
            result: RoundResult {
                round: 1,
                responses: values(1),
            },
        },
    );
    assert!(matches!(
        receive(&mut output),
        ServerMessage::Completed { calls: 8, .. }
    ));
    drop(input);
    assert!(child.wait().unwrap().success());
    let export = inspect(&directory.0, "debate-export");
    use sha2::{Digest, Sha256};
    let expected_digest = format!("{:x}", Sha256::digest(expected.as_bytes()));
    assert!(
        export["effects"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|effect| effect["result"]["round"] == 0)
            .all(|effect| effect["result"]["payload_sha256"] == expected_digest)
    );
}

#[test]
fn corrupt_frozen_configuration_is_reported_instead_of_becoming_a_missing_start() {
    let directory = Directory::new();
    let mut child = process(&directory.0);
    let mut input = child.stdin.take().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    send(&mut input, &ClientMessage::Start { spec: spec("ring") });
    dispatch(&mut output);
    drop(input);
    assert!(matches!(receive(&mut output), ServerMessage::Failed { .. }));
    assert!(!child.wait().unwrap().success());
    let database = rusqlite::Connection::open(directory.0.join("debate.sqlite")).unwrap();
    database
        .execute(
            "UPDATE run_config SET content = ? WHERE id = 1",
            ["{broken"],
        )
        .unwrap();
    drop(database);
    let output = Command::new(binary())
        .args(["debate-status", "--run"])
        .arg(&directory.0)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Invalid stored coordination JSON"));
}

#[test]
fn deleted_frozen_configuration_is_detected_after_admission() {
    let directory = Directory::new();
    let mut child = process(&directory.0);
    let mut input = child.stdin.take().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    send(&mut input, &ClientMessage::Start { spec: spec("ring") });
    dispatch(&mut output);
    drop(input);
    assert!(matches!(receive(&mut output), ServerMessage::Failed { .. }));
    assert!(!child.wait().unwrap().success());
    let database = rusqlite::Connection::open(directory.0.join("debate.sqlite")).unwrap();
    database.execute("DELETE FROM run_config", []).unwrap();
    drop(database);
    let output = Command::new(binary())
        .args(["debate-status", "--run"])
        .arg(&directory.0)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("No coordination run exists"));
}
