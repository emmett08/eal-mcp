use eal_model_host::{
    contract::{Catalogue, CompletionGuard, RunOptions, ScriptGenerator, Session, SessionError},
    host::{run_policy, validate_integer_tokens},
    policy::{self, Action},
};
use serde_json::{Value, json};

fn tools() -> Catalogue {
    [("eal_validate".into(),json!({"inputSchema":{"type":"object","properties":{"source":{"type":"string"}},"required":["source"],"additionalProperties":false}})),
     ("eal_reason".into(),json!({"inputSchema":{"type":"object","properties":{}}})),
     ("eal_collect_claim".into(),json!({"inputSchema":{"type":"object","properties":{}}}))].into_iter().collect()
}
fn state() -> serde_json::Map<String, Value> {
    json!({"source":"A","context":{"location":"B"},"now":"2026-01-02T03:04:05Z","claim":"outcome"})
        .as_object()
        .unwrap()
        .clone()
}

#[test]
fn authoritative_source_context_time_claim_and_ids_are_immutable() {
    let mut state = state();
    state.insert("collection_id".into(), json!("native-id"));
    for (key, value) in [
        ("source", json!("B")),
        ("context", json!({"location":"C"})),
        ("now", json!("2027-01-01T00:00:00Z")),
        ("claim", json!("different")),
        ("collection_id", json!("invented")),
    ] {
        let action = json!({"tool":"eal_reason","arguments":{key:value}}).to_string();
        assert!(
            policy::decode_action(&action, &tools(), &state)
                .unwrap_err()
                .to_string()
                .contains("changed")
        );
    }
}
#[test]
fn omitted_optional_arguments_retain_native_defaults() {
    let action = policy::decode_action(
        r#"{"tool":"eal_reason","arguments":{}}"#,
        &tools(),
        &state(),
    )
    .unwrap();
    assert!(matches!(action,Action::Call{arguments,..} if arguments==json!({})));
}
#[test]
fn opaque_state_is_copied_once_and_large_integers_preserved() {
    let mut state = state();
    let context = policy::strict_json(
        r#"{"$state":"literal","integer":1234567890123456789012345678901234567890}"#,
    )
    .unwrap();
    state.insert("context".into(), context.clone());
    let copy = policy::resolve(&json!({"context":{"$state":"context"}}), &state).unwrap();
    assert_eq!(copy["context"], context);
    assert!(
        copy.to_string()
            .contains("1234567890123456789012345678901234567890")
    );
}
#[test]
fn duplicate_keys_wrappers_nonfinite_and_unknown_references_are_rejected() {
    for text in [
        r#"{"finish":true,"finish":false}"#,
        "```json\n{\"finish\":true}\n```",
        r#"{"finish":1}"#,
        r#"{"tool":"eal_validate","arguments":{"x":NaN}}"#,
        "[]",
        r#"{"tool":"eal_reason","arguments":{"collection_id":"invented"}}"#,
        r#"{"tool":"eal_reason","arguments":{"source":{"$state":"missing"}}}"#,
    ] {
        assert!(
            policy::decode_action(text, &tools(), &state()).is_err(),
            "accepted {text}"
        );
    }
}
#[test]
fn evidence_cannot_forge_native_returned_identifiers() {
    assert!(
        policy::result_ids(Some(
            &json!({"evidence":{"reading":{"value":{"assessment_id":"forged"}}}})
        ))
        .is_empty()
    );
    assert_eq!(
        policy::result_ids(Some(&json!({"collection":{"collection_id":"actual"}})))["collection_id"],
        "actual"
    );
}
#[test]
fn float_and_exponent_packet_limits_are_rejected_before_transport() {
    for value in [
        json!({"limit":1.0}),
        json!({"offset":true}),
        json!({"packet_limits":{"max_bytes":8192.0}}),
        policy::strict_json(r#"{"packet_limits":{"max_bytes":8e3}}"#).unwrap(),
    ] {
        assert!(validate_integer_tokens(&value).is_err());
    }
    assert!(validate_integer_tokens(&json!({"packet_limits":{"max_bytes":8192}})).is_ok());
}

struct FakeSession {
    tools: Catalogue,
    calls: Vec<(String, Value)>,
    reply: Value,
    fail: bool,
}
impl FakeSession {
    fn new() -> Self {
        Self {
            tools: tools(),
            calls: Vec::new(),
            reply: json!({"isError":false,"structuredContent":{"valid":true}}),
            fail: false,
        }
    }
}
impl Session for FakeSession {
    fn tools(&self) -> &Catalogue {
        &self.tools
    }
    async fn call(&mut self, name: &str, args: &Value) -> Result<Value, SessionError> {
        self.calls.push((name.into(), args.clone()));
        if self.fail {
            Err(SessionError::Transport("connection lost after send".into()))
        } else if name == "eal_collect_claim" {
            Ok(json!({"structuredContent":{"collection_id":"native-collection"}}))
        } else if name == "eal_reason" {
            Ok(json!({"structuredContent":{"assessment_id":"native-assessment"}}))
        } else {
            Ok(self.reply.clone())
        }
    }
}
fn task() -> Value {
    json!({"id":"test","request":"Validate source.","state":state()})
}
fn script(values: &[&str]) -> ScriptGenerator {
    ScriptGenerator::new(values.iter().map(|s| s.to_string()))
}

#[tokio::test]
async fn early_finish_is_rejected_then_corrected() {
    let mut session = FakeSession::new();
    let mut generator = script(&[
        r#"{"finish":true}"#,
        r#"{"tool":"eal_validate","arguments":{"source":{"$state":"source"}}}"#,
        r#"{"finish":true}"#,
    ]);
    let result = run_policy(
        &mut session,
        &task(),
        &mut generator,
        &RunOptions::default(),
    )
    .await
    .unwrap();
    assert_eq!(result["stop"], "finished");
    assert!(result["attempts"][0]["error"].is_string());
    assert_eq!(session.calls.len(), 1);
}
#[tokio::test]
async fn duplicate_calls_are_not_replayed() {
    let mut session = FakeSession::new();
    let call = r#"{"tool":"eal_validate","arguments":{"source":{"$state":"source"}}}"#;
    let mut generator = script(&[call, call, r#"{"finish":true}"#]);
    let result = run_policy(
        &mut session,
        &task(),
        &mut generator,
        &RunOptions::default(),
    )
    .await
    .unwrap();
    assert_eq!(session.calls.len(), 1);
    assert!(
        result["attempts"][1]["error"]
            .as_str()
            .unwrap()
            .contains("already completed")
    );
}
#[tokio::test]
async fn uncertain_transport_stops_after_one_send() {
    let mut session = FakeSession::new();
    session.fail = true;
    let mut generator =
        script(&[r#"{"tool":"eal_validate","arguments":{"source":{"$state":"source"}}}"#]);
    let result = run_policy(
        &mut session,
        &task(),
        &mut generator,
        &RunOptions::default(),
    )
    .await
    .unwrap();
    assert_eq!(result["stop"], "transport_error");
    assert_eq!(session.calls.len(), 1);
}
#[tokio::test]
async fn filtered_exposure_is_enforced_at_proposal_boundary() {
    let mut session = FakeSession::new();
    let mut task = task();
    task["allowed_tools"] = json!(["eal_validate"]);
    let mut generator = script(&[
        r#"{"tool":"eal_reason","arguments":{}}"#,
        r#"{"tool":"eal_validate","arguments":{"source":{"$state":"source"}}}"#,
        r#"{"finish":true}"#,
    ]);
    let result = run_policy(&mut session, &task, &mut generator, &RunOptions::default())
        .await
        .unwrap();
    assert_eq!(session.calls.len(), 1);
    assert_eq!(session.calls[0].0, "eal_validate");
    assert_eq!(result["allowed_tools"], json!(["eal_validate"]));
}
#[tokio::test]
async fn tool_error_cannot_finish_successfully() {
    let mut session = FakeSession::new();
    session.reply = json!({"isError":true,"content":[{"type":"text","text":"missing result"}]});
    let mut generator = script(&[
        r#"{"tool":"eal_validate","arguments":{"source":{"$state":"source"}}}"#,
        r#"{"finish":true}"#,
        r#"{"stop":"failed"}"#,
    ]);
    let result = run_policy(
        &mut session,
        &task(),
        &mut generator,
        &RunOptions::default(),
    )
    .await
    .unwrap();
    assert_eq!(result["stop"], "failed");
    assert!(result["attempts"][1]["error"].is_string());
}
#[tokio::test]
async fn enhanced_completion_requires_operator_requested_assessment_and_actual_collection_id() {
    let mut session = FakeSession::new();
    let mut task = task();
    task["requires_assessment"] = true.into();
    let mut generator = script(&[
        r#"{"tool":"eal_collect_claim","arguments":{}}"#,
        r#"{"finish":true}"#,
        r#"{"tool":"eal_reason","arguments":{"collection_id":{"$state":"collection_id"}}}"#,
        r#"{"finish":true}"#,
    ]);
    let options = RunOptions {
        completion_guard: CompletionGuard::AssessmentRequested,
        ..Default::default()
    };
    let result = run_policy(&mut session, &task, &mut generator, &options)
        .await
        .unwrap();
    assert_eq!(result["stop"], "finished");
    assert_eq!(session.calls.len(), 2);
    assert_eq!(session.calls[1].1["collection_id"], "native-collection");
    assert!(
        result["attempts"][1]["error"]
            .as_str()
            .unwrap()
            .contains("incomplete")
    );
    assert_eq!(result["state_ids"]["assessment_id"], "native-assessment");
}
#[tokio::test]
async fn baseline_allows_collection_only_success_when_requested() {
    let mut session = FakeSession::new();
    let mut generator = script(&[
        r#"{"tool":"eal_collect_claim","arguments":{}}"#,
        r#"{"finish":true}"#,
    ]);
    let result = run_policy(
        &mut session,
        &task(),
        &mut generator,
        &RunOptions::default(),
    )
    .await
    .unwrap();
    assert_eq!(result["stop"], "finished");
    assert_eq!(session.calls.len(), 1);
}
#[tokio::test]
async fn stale_initial_assessment_cannot_complete_new_collection() {
    let mut session = FakeSession::new();
    let mut task = task();
    task["requires_assessment"] = true.into();
    task["state"]["assessment_id"] = json!("old-assessment");
    let mut generator = script(&[
        r#"{"tool":"eal_collect_claim","arguments":{}}"#,
        r#"{"finish":true}"#,
        r#"{"stop":"failed"}"#,
    ]);
    let options = RunOptions {
        completion_guard: CompletionGuard::AssessmentRequested,
        ..Default::default()
    };
    let result = run_policy(&mut session, &task, &mut generator, &options)
        .await
        .unwrap();
    assert_eq!(result["stop"], "failed");
    assert!(
        result["attempts"][1]["error"]
            .as_str()
            .unwrap()
            .contains("incomplete")
    );
}
#[tokio::test]
async fn canonical_call_identity_blocks_reordered_duplicate_arguments() {
    let mut session = FakeSession::new();
    let mut generator = script(&[
        r#"{"tool":"eal_validate","arguments":{"source":"A","context":{"location":"B"}}}"#,
        r#"{"tool":"eal_validate","arguments":{"context":{"location":"B"},"source":"A"}}"#,
        r#"{"finish":true}"#,
    ]);
    let options = RunOptions {
        canonical_call_identity: true,
        ..Default::default()
    };
    let result = run_policy(&mut session, &task(), &mut generator, &options)
        .await
        .unwrap();
    assert_eq!(session.calls.len(), 1);
    assert!(
        result["attempts"][1]["error"]
            .as_str()
            .unwrap()
            .contains("already completed")
    );
}

fn observational_tools() -> Catalogue {
    let mut catalogue = tools();
    catalogue.insert("eal_reason".into(), json!({"inputSchema":{"type":"object","required":["source","context"],"properties":{"source":{"type":"string"},"context":{"type":"object"},"collection_id":{"anyOf":[{"type":"string","minLength":1},{"type":"null"}]}},"additionalProperties":false}}));
    catalogue.insert("eal_compile_aspic".into(), json!({"inputSchema":{"type":"object","required":["source","context","collection_id","goal"],"properties":{"collection_id":{"type":"string","minLength":1}}}}));
    catalogue.insert("eal_packet".into(), json!({"inputSchema":{"type":"object","required":["assessment_id"],"properties":{"assessment_id":{"type":"string","minLength":1}}}}));
    catalogue.insert("custom_query".into(), json!({"inputSchema":{"type":"object","required":["query"],"properties":{"query":{"type":"string"},"assessment_id":{"type":"string"}}}}));
    catalogue.insert("both_ids".into(), json!({"inputSchema":{"type":"object","required":["collection_id","assessment_id"],"properties":{}}}));
    catalogue
}

#[test]
fn observational_scope_uses_actual_required_ids_and_explicit_reason_policy_only() {
    use eal_model_host::host::evidence_guided_catalogue;
    let catalogue = observational_tools();
    let mut state = state();
    let initial = evidence_guided_catalogue(&catalogue, &state);
    assert!(!initial.contains_key("eal_reason")); // Explicit scope beyond native optional semantics.
    assert!(!initial.contains_key("eal_compile_aspic"));
    assert!(!initial.contains_key("eal_packet"));
    assert!(!initial.contains_key("both_ids"));
    assert!(initial.contains_key("custom_query")); // Authored query and optional assessment ID stay legal.
    assert!(initial.contains_key("eal_validate")); // Authored source is not a routing prerequisite.
    for invalid in [
        Value::Null,
        json!(""),
        json!(123),
        json!({"id":"collection"}),
    ] {
        state.insert("collection_id".into(), invalid);
        let offered = evidence_guided_catalogue(&catalogue, &state);
        assert!(!offered.contains_key("eal_reason"));
        assert!(!offered.contains_key("eal_compile_aspic"));
    }
    state.insert("collection_id".into(), json!("native-collection"));
    let after_collection = evidence_guided_catalogue(&catalogue, &state);
    assert!(after_collection.contains_key("eal_reason"));
    assert!(after_collection.contains_key("eal_compile_aspic"));
    assert!(!after_collection.contains_key("both_ids"));
    state.insert("assessment_id".into(), json!("native-assessment"));
    let after_assessment = evidence_guided_catalogue(&catalogue, &state);
    assert!(after_assessment.contains_key("eal_packet"));
    assert!(after_assessment.contains_key("both_ids"));
    assert_eq!(after_assessment["eal_reason"], catalogue["eal_reason"]);
}

struct RecordingGenerator {
    script: ScriptGenerator,
    messages: Vec<Vec<Value>>,
}
impl RecordingGenerator {
    fn new(actions: &[&str]) -> Self {
        Self {
            script: script(actions),
            messages: Vec::new(),
        }
    }
}
impl eal_model_host::contract::Generator for RecordingGenerator {
    fn generate(
        &mut self,
        messages: &[Value],
        max_new_tokens: usize,
    ) -> anyhow::Result<eal_model_host::contract::Generation> {
        self.messages.push(messages.to_vec());
        eal_model_host::contract::Generator::generate(&mut self.script, messages, max_new_tokens)
    }
}

#[tokio::test]
async fn evidence_guidance_off_preserves_baseline_messages_and_optional_collection_omission() {
    let mut session = FakeSession::new();
    session.tools = observational_tools();
    let expected_messages = policy::messages_for(&task(), &session.tools, &state(), &[]).unwrap();
    let mut generator = RecordingGenerator::new(&[
        r#"{"tool":"eal_reason","arguments":{"source":{"$state":"source"},"context":{"$state":"context"}}}"#,
        r#"{"finish":true}"#,
    ]);
    let run = run_policy(
        &mut session,
        &task(),
        &mut generator,
        &RunOptions::default(),
    )
    .await
    .unwrap();
    assert_eq!(run["stop"], "finished");
    assert_eq!(generator.messages[0], expected_messages);
    assert_eq!(
        session.calls[0].1,
        json!({"source":"A","context":{"location":"B"}})
    );
    assert!(run["attempts"][0].get("visible_tools").is_none());
    assert!(run.get("discovered_catalogue").is_none());
}

#[tokio::test]
async fn evidence_guidance_recomputes_after_native_ids_without_replacing_calls_or_arguments() {
    let mut session = FakeSession::new();
    session.tools = observational_tools();
    let original_catalogue = session.tools.clone();
    let mut generator = RecordingGenerator::new(&[
        r#"{"tool":"eal_reason","arguments":{"source":{"$state":"source"},"context":{"$state":"context"}}}"#,
        r#"{"tool":"eal_collect_claim","arguments":{"source":{"$state":"source"},"context":{"$state":"context"},"claim":{"$state":"claim"}}}"#,
        r#"{"tool":"eal_reason","arguments":{"source":{"$state":"source"},"context":{"$state":"context"},"collection_id":{"$state":"collection_id"}}}"#,
        r#"{"finish":true}"#,
    ]);
    let options = RunOptions {
        max_steps: 4,
        evidence_guided: true,
        ..Default::default()
    };
    let run = run_policy(&mut session, &task(), &mut generator, &options)
        .await
        .unwrap();
    let offered = |step: usize, name: &str| {
        run["attempts"][step]["visible_tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == name)
    };
    assert_eq!(run["stop"], "finished");
    assert!(!offered(0, "eal_reason"));
    assert!(!offered(1, "eal_compile_aspic"));
    assert!(offered(2, "eal_reason"));
    assert!(offered(2, "eal_compile_aspic"));
    assert!(offered(3, "eal_packet"));
    assert!(
        run["attempts"][0]["error"]
            .as_str()
            .unwrap()
            .contains("unavailable")
    );
    assert_eq!(session.calls.len(), 2);
    assert_eq!(
        session.calls[0],
        (
            "eal_collect_claim".into(),
            json!({"source":"A","context":{"location":"B"},"claim":"outcome"})
        )
    );
    assert_eq!(
        session.calls[1],
        (
            "eal_reason".into(),
            json!({"source":"A","context":{"location":"B"},"collection_id":"native-collection"})
        )
    );
    assert_eq!(session.tools, original_catalogue);
    assert_eq!(
        run["discovered_catalogue"],
        serde_json::to_value(original_catalogue).unwrap()
    );
    assert_eq!(
        run["host_variant"]["evidence_guided"]["native_schema_changed"],
        false
    );
}

#[tokio::test]
async fn evidence_guidance_intersects_operator_scope_and_retains_generation_failure_menu() {
    let mut session = FakeSession::new();
    session.tools = observational_tools();
    let mut task = task();
    task["allowed_tools"] = json!(["eal_collect_claim", "eal_packet"]);
    let mut generator = RecordingGenerator::new(&[]);
    let options = RunOptions {
        evidence_guided: true,
        ..Default::default()
    };
    let run = run_policy(&mut session, &task, &mut generator, &options)
        .await
        .unwrap();
    assert_eq!(run["stop"], "generation_error");
    assert_eq!(
        run["attempts"][0]["visible_tools"],
        json!(["eal_collect_claim"])
    );
    assert_eq!(
        run["allowed_tools"],
        json!(["eal_collect_claim", "eal_packet"])
    );
    assert!(session.calls.is_empty());
}

#[cfg(unix)]
fn reject_invalid_cli_bundle_before_native_spawn(bad_files: bool) {
    use std::{fs, os::unix::fs::PermissionsExt, process::Command};
    let temporary = tempfile::tempdir().unwrap();
    let base = temporary.path().join("base");
    let adapter = temporary.path().join("adapter");
    if bad_files {
        fs::create_dir(&base).unwrap();
        fs::create_dir(&adapter).unwrap();
        // The real pin rejects these bytes before any tensor can be loaded.
        fs::write(base.join("config.json"), b"{}").unwrap();
    }
    let workspace = temporary.path().join("workspace");
    fs::create_dir(&workspace).unwrap();
    let marker = workspace.join("native-spawned.marker");
    let executable = temporary.path().join("operator-native-sentinel");
    // This operator-owned test executable has no interpolated paths or inputs.
    fs::write(
        &executable,
        b"#!/bin/sh\nprintf invoked > native-spawned.marker\nexit 93\n",
    )
    .unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
    let task = temporary.path().join("task.json");
    fs::write(&task, serde_json::to_vec(&json!({"request":"Validate supplied source.","state":{"source":"language \"EAL/4\"\n","context":{}}})).unwrap()).unwrap();
    let trace = temporary.path().join("failure.json");
    let binary = env!("CARGO_BIN_EXE_eal-model-host");

    let verification = Command::new(binary)
        .arg("verify")
        .arg("--base")
        .arg(&base)
        .arg("--adapter")
        .arg(&adapter)
        .output()
        .unwrap();
    assert!(!verification.status.success());
    assert!(verification.stdout.is_empty());
    assert!(String::from_utf8_lossy(&verification.stderr).contains("pinned base model"));

    let run = Command::new(binary)
        .arg("run")
        .arg("--task")
        .arg(&task)
        .arg("--base")
        .arg(&base)
        .arg("--adapter")
        .arg(&adapter)
        .arg("--native-executable")
        .arg(&executable)
        .arg("--workspace")
        .arg(&workspace)
        .arg("--output")
        .arg(&trace)
        .output()
        .unwrap();
    assert!(!run.status.success());
    let retained: Value = serde_json::from_slice(&fs::read(trace).unwrap()).unwrap();
    assert_eq!(retained["status"], "failed");
    assert!(
        retained["error"]
            .as_str()
            .unwrap()
            .contains("pinned base model")
    );
    assert!(retained.get("run").is_none());
    assert_eq!(retained["configuration"]["prefix_cache"], true);
    assert!(
        !marker.exists(),
        "invalid bundle must never start the native executable"
    );
}

#[cfg(unix)]
#[test]
fn cli_missing_bundle_fails_closed_and_retains_failure_without_starting_mcp() {
    reject_invalid_cli_bundle_before_native_spawn(false);
}

#[cfg(unix)]
#[test]
fn cli_bad_bundle_fails_closed_and_retains_failure_without_starting_mcp() {
    reject_invalid_cli_bundle_before_native_spawn(true);
}
