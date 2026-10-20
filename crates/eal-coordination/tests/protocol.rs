//! Independent checks of exact canonical wire identity and checked native admission.
use eal_coordination::protocol::{self, MAX_BYTES, MAX_DEPTH};
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
};

fn envelope(payload: Value) -> Value {
    json!({"message_id":"m","run_id":"r","sender":"a","recipient":"b","version":"1","kind":"request","payload":payload})
}

#[test]
fn canonical_wire_sorts_every_object_and_preserves_literal_private_keys() {
    let value = envelope(
        json!({"z":{"y":2,"a":1},"a":{"$serde_json::private::Number":"literal","$serde_json::private::RawValue":"ordinary"},"bool":true}),
    );
    let admitted = protocol::admit_envelope(&value, MAX_BYTES, MAX_DEPTH).unwrap();
    assert_eq!(
        admitted.wire(),
        r#"message "m" run "r" from "a" to "b" version "1" kind "request" payload {"a":{"$serde_json::private::Number":"literal","$serde_json::private::RawValue":"ordinary"},"bool":true,"z":{"a":1,"y":2}};"#
    );
    assert_eq!(admitted.to_value(), value);
    assert_eq!(
        protocol::parse_message(admitted.wire(), MAX_BYTES, MAX_DEPTH)
            .unwrap()
            .to_value()
            .unwrap(),
        value
    );
}

#[test]
fn every_decode_path_rejects_fractional_exponent_and_duplicate_payloads() {
    for payload in [
        r#"{"n":1.0}"#,
        r#"{"n":1e0}"#,
        r#"{"n":-0.0}"#,
        r#"{"k":1,"\u006b":2}"#,
    ] {
        let wire = format!(
            "message \"m\" run \"r\" from \"a\" to \"b\" version \"1\" kind \"request\" payload {payload};"
        );
        assert!(protocol::parse_message(&wire, MAX_BYTES, MAX_DEPTH).is_err());
        let json = format!(
            r#"{{"message_id":"m","run_id":"r","sender":"a","recipient":"b","version":"1","kind":"request","payload":{payload}}}"#
        );
        assert!(protocol::encode_message(&json, MAX_BYTES, MAX_DEPTH).is_err());
        assert!(protocol::admit_message(&json, MAX_BYTES, MAX_DEPTH).is_err());
    }
}

#[test]
fn standalone_persistent_codec_is_strict_recovers_and_flushes_each_reply() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mas-protocol-codec"))
        .arg("serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    for operation in ["admit", "invalid", "admit"] {
        let request = json!({"operation":operation,"source":envelope(json!({"exact":123})).to_string(),"max_bytes":MAX_BYTES,"max_depth":MAX_DEPTH});
        writeln!(input, "{request}").unwrap();
        input.flush().unwrap();
        let mut response = String::new();
        assert!(output.read_line(&mut response).unwrap() > 0);
        let value: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(value["ok"], operation == "admit");
        assert!(child.try_wait().unwrap().is_none());
    }
    drop(input);
    assert!(child.wait().unwrap().success());
}

#[test]
fn checked_public_models_reject_deep_values_before_cloning_and_drop_safely() {
    use eal_coordination::contracts::TaskSpec;
    std::thread::Builder::new()
        .stack_size(256 * 1024)
        .spawn(|| {
            let mut nested = Value::Null;
            for _ in 0..20_000 {
                nested = Value::Array(vec![nested]);
            }
            let mut task = TaskSpec {
                task_id: "t".into(),
                question: "q".into(),
                entry_id: "e".into(),
                claims: vec!["c".into()],
                profile: "registered-assessment".into(),
                requirements: Value::Null,
            };
            task.requirements = json!({"nested":null});
            task.requirements["nested"] = nested;
            assert_eq!(
                task.to_value(),
                Err("requirements exceeds structural bounds".into())
            );
            let mut payload = serde_json::Map::new();
            payload.insert("nested".into(), std::mem::take(&mut task.requirements));
            let message = protocol::Message {
                message_id: "m".into(),
                run_id: "r".into(),
                sender: "a".into(),
                recipient: "b".into(),
                version: "1".into(),
                kind: "request".into(),
                payload,
            };
            assert_eq!(message.to_value().unwrap_err().code, "DEPTH");
        })
        .unwrap()
        .join()
        .unwrap();
}

#[cfg(unix)]
#[test]
fn server_spec_rejects_non_utf8_paths_without_lossy_identity() {
    use eal_coordination::contracts::ServerSpec;
    use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::PathBuf};
    let server = ServerSpec {
        workspace: PathBuf::from(OsString::from_vec(b"/tmp/non-utf8-\xff".to_vec())),
        registry: None,
        database: None,
        methods: None,
        limits: None,
    };
    assert!(server.to_value().is_err());
}

#[test]
fn finding_aggregation_uses_the_durable_record_ceiling() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/coordination/reference.json"
    ))
    .unwrap();
    let anchor = &fixture["contracts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["op"] == "finding")
        .unwrap()["value"];
    let check = json!({"blob":"x".repeat(4 * 1024 * 1024 + 1)});
    let mut finding =
        eal_coordination::contracts::make_finding(anchor, &check, None, None, None, None).unwrap();
    let identity = finding
        .as_object_mut()
        .unwrap()
        .remove("finding_id")
        .unwrap();
    let limits = eal_core::ExecutionLimits {
        record_bytes: 64 * 1024 * 1024,
        json_nodes: 64 * 1024 * 1024,
        json_depth: 128,
        ..Default::default()
    };
    let digest = eal_core::canonical::canonical_digest(&finding, &limits).unwrap();
    assert_eq!(identity, format!("finding_{digest}"));
}
