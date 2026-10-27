#![cfg(unix)]
//! Actual supervised ticket stages with independent artifacts and failure paths.
use eal_coordination_contracts::ticket::{
    Artifact, STAGE_RESULT_SCHEMA, Stage, StageResult, StageStatus, TICKET_SCHEMA, TicketSpec,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    path::PathBuf,
    process::{Command, Output, Stdio},
    time::{Duration, Instant},
};

struct Fixture {
    root: PathBuf,
    work: PathBuf,
    run: PathBuf,
    spec: PathBuf,
    counter: PathBuf,
    artifacts: Vec<Artifact>,
}
impl Fixture {
    fn new(block: Option<Stage>, fail: Option<Stage>, delay: bool) -> Self {
        let root = std::env::temp_dir().join(format!("eal-ticket-test-{}", uuid::Uuid::new_v4()));
        let work = root.join("work");
        std::fs::create_dir_all(&work).unwrap();
        let work = work.canonicalize().unwrap();
        let run = root.join("journal");
        let spec_path = root.join("ticket.json");
        let counter = work.join("calls");
        let mut artifacts = Vec::new();
        let mut responses = Vec::new();
        let source_text = r#"language "EAL/3" profile "execution/1"
environment lab {
  require true
}
tool runner {
  version "1"
}
evidence observation {
  tool runner
  kind test
  environment lab
  max_age 60
  input {}
  require passed == true
}
evidence unused {
  tool runner
  kind test
  environment lab
  max_age 60
  input {}
  require passed == true
}
reasoning measured {
  method "structured/1"
  rationale "The retained observation decides the selected test predicate."
}
claim selected_claim {
  statement "The selected test predicate holds."
  environment lab
}
claim stronger_target {
  statement "The independent second predicate holds."
  environment lab
}
argument selected_support = [evidence observation] via measured => selected_claim
argument unused_support = [evidence unused] via measured => stronger_target
"#;
        let source_digest = format!("{:x}", Sha256::digest(source_text.as_bytes()));
        let program = eal_core::composition::compile(
            source_text,
            None,
            "ticket.eal",
            eal_core::ExecutionLimits::default(),
        )
        .unwrap();
        let executable_digest = eal_core::semantic_format::executable_digest(
            &program,
            eal_core::methods::default_registry(),
        )
        .unwrap();
        let context_fingerprint =
            eal_core::canonical::canonical_digest(&json!({}), &Default::default()).unwrap();
        let collection = json!({"source_digest":source_digest,"context":{},"collection_id":"collection_fixture","records":{"observation":{"status":"ok","value":{"passed":false}}}});
        let assessment = json!({"source_digest":source_digest,"context_fingerprint":context_fingerprint,"collection_id":"collection_fixture","assessment_id":"assessment_fixture","valid":true,"complete":true,"claims":{"selected_claim":{"status":"unsupported"}},"evidence":{"observation":{"status":"unavailable","availability_issues":["predicate_not_met"]},"unused":{"status":"unavailable","availability_issues":["missing_observation"]}}});
        let execution = json!({"is_error":false,"result":{"schema":"EAL/execution/1","source_digest":source_digest,"executable_digest":executable_digest,"claim":"selected_claim","collection":collection,"assessment":assessment}});
        let tools = "[tools.runner]\nkind='command'\nversion='1'\nargv=['unused']\n";
        let tools_digest = format!("{:x}", Sha256::digest(tools.as_bytes()));
        let precise=eal_collector_contracts::strict_json_exact(br#"{"precision":1.0000000000000000000001,"literal":{"$serde_json::private::Number":"1"}}"#,Default::default()).unwrap();
        for stage in Stage::ALL {
            let stage_artifacts = stage
                .required_artifacts()
                .iter()
                .map(|kind| {
                    let path = work.join(format!("{kind}.json"));
                    let data = match *kind {
                        "build" => json!({"schema":"engarg.executable-package/1","files":{"case.eal":source_digest,"tools.toml":tools_digest},"executable_digest":executable_digest,"claims":{"current_claim":"selected_claim"}}),
                        "collection" => collection.clone(),
                        "assessment" => assessment.clone(),
                        "execution" => execution.clone(),
                        _ => json!({"artifact":kind}),
                    };
                    let bytes = match *kind { "source"=>source_text.as_bytes().to_vec(),"tools"=>tools.as_bytes().to_vec(),_=>serde_json::to_vec(&data).unwrap() };
                    std::fs::write(&path, &bytes).unwrap();
                    Artifact {
                        kind: (*kind).into(),
                        path,
                        sha256: format!("{:x}", Sha256::digest(&bytes)),
                        bytes: bytes.len() as u64,
                    }
                })
                .collect::<Vec<_>>();
            artifacts.extend(stage_artifacts.clone());
            responses.push(StageResult {
                schema_version: STAGE_RESULT_SCHEMA.into(),
                ticket_id: "fixture-ticket".into(),
                stage,
                status: if block == Some(stage) {
                    StageStatus::Blocked
                } else {
                    StageStatus::Completed
                },
                artifacts: if block == Some(stage) {
                    Vec::new()
                } else {
                    stage_artifacts
                },
                outcome: Some(if stage == Stage::Execute {
                    let mut summary=json!({"complete":true,"valid":true,"claim_status":"unsupported","claim":"selected_claim","source_digest":source_digest,"executable_digest":executable_digest,"collection_id":"collection_fixture","assessment_id":"assessment_fixture"});
                    summary["precision"]=precise["precision"].clone(); summary["literal"]=precise["literal"].clone(); summary
                } else {
                    json!({"reason":"fixture"})
                }),
            });
        }
        let mut script = format!(
            "#!/bin/sh\nn=0\nif test -f '{}'; then n=$(/bin/cat '{}'); fi\nn=$((n+1))\nprintf '%s' \"$n\" > '{}'\n/bin/cat > '{}/request-'\"$n\"'.json'\n",
            counter.display(),
            counter.display(),
            counter.display(),
            work.display()
        );
        if delay {
            script.push_str("/bin/sleep 2\n");
        }
        script.push_str("case \"$n\" in\n");
        for (index, response) in responses.into_iter().enumerate() {
            let body=serde_json::to_string(&json!({"response":response,"execution":{"execution_id":format!("fixture-execution-{index}"),"model":null,"usage":null}})).unwrap();
            script.push_str(&format!("{}) ", index + 1));
            if fail == Some(Stage::ALL[index]) {
                script.push_str("printf 'partial'; exit 17;;\n");
            } else {
                script.push_str(&format!("printf '%s\\n' '{body}';;\n"));
            }
        }
        script.push_str("*) exit 99;;\nesac\n");
        let script_path = work.join("worker.sh");
        std::fs::write(&script_path, &script).unwrap();
        let spec = TicketSpec {
            schema_version: TICKET_SCHEMA.into(),
            ticket_id: "fixture-ticket".into(),
            workspace: work.clone(),
            worker_argv: vec![
                std::fs::canonicalize("/bin/sh")
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                script_path.to_string_lossy().into_owned(),
            ],
            environment: Vec::new(),
            timeout_seconds: 20,
            stage_timeout_seconds: 5,
            context: precise,
            inputs: vec![Artifact {
                kind: "worker_script".into(),
                path: script_path,
                sha256: format!("{:x}", Sha256::digest(script.as_bytes())),
                bytes: script.len() as u64,
            }],
        };
        std::fs::write(&spec_path, serde_json::to_vec(&spec).unwrap()).unwrap();
        Self {
            root,
            work,
            run,
            spec: spec_path,
            counter,
            artifacts,
        }
    }
    fn command(&self, operation: &str) -> Command {
        let mut command = Command::new(binary());
        command.args([operation, "--run"]).arg(&self.run);
        if operation == "ticket-run" {
            command.arg("--spec").arg(&self.spec);
        }
        command
    }
    fn execute(&self, operation: &str) -> (Output, Value) {
        let output = self.command(operation).output().unwrap();
        let value = serde_json::from_slice(&output.stdout)
            .unwrap_or_else(|_| panic!("{}", String::from_utf8_lossy(&output.stderr)));
        (output, value)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_eal-coordinate")
}

#[test]
fn native_ticket_orders_stages_and_completes_an_unsupported_claim_without_reexecution() {
    let fixture = Fixture::new(None, None, false);
    let (output, result) = fixture.execute("ticket-run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(result["status"], "completed");
    assert_eq!(result["outcome"]["claim_status"], "unsupported");
    assert_eq!(
        result["outcome"]["precision"].to_string(),
        "1.0000000000000000000001"
    );
    assert_eq!(result["stages"].as_array().unwrap().len(), 4);
    assert_eq!(std::fs::read_to_string(&fixture.counter).unwrap(), "4");
    for (index, stage) in Stage::ALL.into_iter().enumerate() {
        let request: Value = serde_json::from_slice(
            &std::fs::read(fixture.work.join(format!("request-{}.json", index + 1))).unwrap(),
        )
        .unwrap();
        assert_eq!(request["stage"], stage.as_str());
        assert_eq!(request["ticket_id"], "fixture-ticket");
        let exact = eal_collector_contracts::strict_json_exact(
            &std::fs::read(fixture.work.join(format!("request-{}.json", index + 1))).unwrap(),
            Default::default(),
        )
        .unwrap();
        assert_eq!(
            exact["context"]["precision"].to_string(),
            "1.0000000000000000000001"
        );
        assert!(exact["context"]["literal"].is_object());
    }
    assert_eq!(fixture.execute("ticket-run").1["status"], "completed");
    assert_eq!(std::fs::read_to_string(&fixture.counter).unwrap(), "4");
}

#[test]
fn blocked_integration_and_failed_workers_stop_later_stages_and_never_repeat() {
    for blocked in [true, false] {
        let fixture = Fixture::new(
            blocked.then_some(Stage::Integrate),
            (!blocked).then_some(Stage::Integrate),
            false,
        );
        let (output, result) = fixture.execute("ticket-run");
        assert!(!output.status.success());
        assert_eq!(result["status"], if blocked { "blocked" } else { "failed" });
        assert_eq!(result["stages"].as_array().unwrap().len(), 2);
        assert_eq!(std::fs::read_to_string(&fixture.counter).unwrap(), "2");
        assert_eq!(fixture.execute("ticket-run").1["status"], result["status"]);
        assert_eq!(std::fs::read_to_string(&fixture.counter).unwrap(), "2");
    }
}

#[test]
fn artifact_changes_and_outside_workspace_bindings_are_rejected() {
    let fixture = Fixture::new(None, None, false);
    assert!(fixture.execute("ticket-run").0.status.success());
    std::fs::write(&fixture.artifacts[0].path, b"changed").unwrap();
    assert!(
        !fixture
            .command("ticket-status")
            .output()
            .unwrap()
            .status
            .success()
    );
    let fixture = Fixture::new(None, None, false);
    let mut spec: Value = serde_json::from_slice(&std::fs::read(&fixture.spec).unwrap()).unwrap();
    spec["inputs"][0]["path"] = json!("/etc/passwd");
    std::fs::write(&fixture.spec, serde_json::to_vec(&spec).unwrap()).unwrap();
    assert!(
        !fixture
            .command("ticket-run")
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(!fixture.counter.exists());
}

#[test]
fn an_interrupted_controller_keeps_unknown_work_reserved() {
    let fixture = Fixture::new(None, None, true);
    let mut child = fixture
        .command("ticket-run")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let started = Instant::now();
    while !fixture.counter.exists() {
        assert!(started.elapsed() < Duration::from_secs(5));
        std::thread::sleep(Duration::from_millis(10));
    }
    child.kill().unwrap();
    child.wait().unwrap();
    let (output, result) = fixture.execute("ticket-run");
    assert!(!output.status.success());
    assert_eq!(result["status"], "uncertain");
    assert_eq!(std::fs::read_to_string(&fixture.counter).unwrap(), "1");
}

#[test]
fn ticket_response_receipts_are_checked_against_exact_retained_bytes() {
    let fixture = Fixture::new(None, None, false);
    assert!(fixture.execute("ticket-run").0.status.success());
    let db = rusqlite::Connection::open(fixture.run.join("ticket.sqlite")).unwrap();
    db.execute("UPDATE effects SET result = json_set(result,'$.execution_id','different') WHERE effect_key='stage:author'",[]).unwrap();
    assert!(
        !fixture
            .command("ticket-status")
            .output()
            .unwrap()
            .status
            .success()
    );
}

#[test]
fn cancelling_a_ticket_interrupts_the_active_stage_and_prevents_another_launch() {
    let fixture = Fixture::new(None, None, true);
    let child = fixture
        .command("ticket-run")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let started = Instant::now();
    while !fixture.counter.exists() {
        assert!(started.elapsed() < Duration::from_secs(5));
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(fixture.execute("ticket-cancel").1["status"], "cancelled");
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["status"], "cancelled");
    assert_eq!(std::fs::read_to_string(&fixture.counter).unwrap(), "1");
    assert_eq!(fixture.execute("ticket-run").1["status"], "cancelled");
}

#[test]
fn frozen_application_context_is_checked_against_the_exact_dispatched_request() {
    let fixture = Fixture::new(None, None, false);
    assert!(fixture.execute("ticket-run").0.status.success());
    let request = fixture.run.join("author-request.json");
    let mut value = eal_collector_contracts::strict_json_exact(
        &std::fs::read(&request).unwrap(),
        Default::default(),
    )
    .unwrap();
    value["context"] = json!({"different":"context"});
    let bytes = serde_json::to_vec(&value).unwrap();
    std::fs::write(&request, &bytes).unwrap();
    let digest = format!("{:x}", Sha256::digest(&bytes));
    let db = rusqlite::Connection::open(fixture.run.join("ticket.sqlite")).unwrap();
    db.execute("UPDATE effects SET payload = json_set(payload,'$.request.sha256',?1,'$.request.bytes',?2) WHERE effect_key='stage:author'",rusqlite::params![digest,bytes.len() as u64]).unwrap();
    assert!(
        !fixture
            .command("ticket-status")
            .output()
            .unwrap()
            .status
            .success()
    );
}

#[test]
fn invalid_claim_status_and_cross_artifact_execution_identities_are_rejected() {
    for (before, after) in [
        (
            r#""claim_status":"unsupported""#,
            r#""claim_status":"banana""#,
        ),
        (
            r#""collection_id":"collection_fixture""#,
            r#""collection_id":"different_collection""#,
        ),
        (
            r#""claim":"selected_claim""#,
            r#""claim":"stronger_target""#,
        ),
    ] {
        let fixture = Fixture::new(None, None, false);
        let script_path = fixture.work.join("worker.sh");
        let script = std::fs::read_to_string(&script_path).unwrap();
        assert!(script.contains(before));
        let script = script.replace(before, after);
        std::fs::write(&script_path, &script).unwrap();
        let mut spec: TicketSpec =
            serde_json::from_slice(&std::fs::read(&fixture.spec).unwrap()).unwrap();
        spec.inputs[0].sha256 = format!("{:x}", Sha256::digest(script.as_bytes()));
        spec.inputs[0].bytes = script.len() as u64;
        std::fs::write(&fixture.spec, serde_json::to_vec(&spec).unwrap()).unwrap();
        let (output, result) = fixture.execute("ticket-run");
        assert!(!output.status.success());
        assert_eq!(result["status"], "failed");
        assert_eq!(result["stages"].as_array().unwrap().len(), 4);
    }
}

#[test]
fn oversized_execution_receipt_keeps_a_small_inspectable_failure_index() {
    let fixture = Fixture::new(None, None, false);
    let script_path = fixture.work.join("worker.sh");
    let script = std::fs::read_to_string(&script_path).unwrap();
    let script = script.replace(
        r#""usage":null"#,
        &format!(r#""usage":{{"input_tokens":{}}}"#, "9".repeat(100_000)),
    );
    std::fs::write(&script_path, &script).unwrap();
    let mut spec: TicketSpec =
        serde_json::from_slice(&std::fs::read(&fixture.spec).unwrap()).unwrap();
    spec.inputs[0].sha256 = format!("{:x}", Sha256::digest(script.as_bytes()));
    spec.inputs[0].bytes = script.len() as u64;
    std::fs::write(&fixture.spec, serde_json::to_vec(&spec).unwrap()).unwrap();
    let (output, result) = fixture.execute("ticket-run");
    assert!(!output.status.success());
    assert_eq!(result["status"], "failed");
    assert_eq!(fixture.execute("ticket-status").1["status"], "failed");
    assert!(
        std::fs::metadata(fixture.run.join("ticket.sqlite"))
            .unwrap()
            .len()
            < 1024 * 1024
    );
}

#[test]
fn rehashed_collection_payload_cannot_replace_the_retained_execution_result() {
    let fixture = Fixture::new(None, None, false);
    let collection = fixture
        .artifacts
        .iter()
        .find(|artifact| artifact.kind == "collection")
        .unwrap();
    let mut value: Value =
        serde_json::from_slice(&std::fs::read(&collection.path).unwrap()).unwrap();
    value["records"]["observation"]["value"]["passed"] = json!(true);
    let bytes = serde_json::to_vec(&value).unwrap();
    std::fs::write(&collection.path, &bytes).unwrap();
    let replacement = Artifact {
        sha256: format!("{:x}", Sha256::digest(&bytes)),
        bytes: bytes.len() as u64,
        ..collection.clone()
    };
    let script_path = fixture.work.join("worker.sh");
    let script = std::fs::read_to_string(&script_path).unwrap();
    let before = serde_json::to_string(collection).unwrap();
    let after = serde_json::to_string(&replacement).unwrap();
    assert!(script.contains(&before));
    let script = script.replace(&before, &after);
    std::fs::write(&script_path, &script).unwrap();
    let mut spec: TicketSpec =
        serde_json::from_slice(&std::fs::read(&fixture.spec).unwrap()).unwrap();
    spec.inputs[0].sha256 = format!("{:x}", Sha256::digest(script.as_bytes()));
    spec.inputs[0].bytes = script.len() as u64;
    std::fs::write(&fixture.spec, serde_json::to_vec(&spec).unwrap()).unwrap();
    let (output, result) = fixture.execute("ticket-run");
    assert!(!output.status.success());
    assert_eq!(result["status"], "failed");
    assert_eq!(result["stages"].as_array().unwrap().len(), 4);
}
