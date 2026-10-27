//! Independent offline lifecycle checks. Deliberate context mismatch means no
//! MCP session, collector, provider or model is allowed to run.
#![cfg(unix)]
use base64::{Engine, engine::general_purpose::STANDARD};
use eal_coordination::{
    contracts::{ServerSpec, TaskSpec},
    coordinator::Coordinator,
    host::HostPaths,
    profiles::knowledge_with_host,
};
use eal_runtime::catalogue::RegisterOptions;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

const SOURCE: &str = r#"language "EAL/3"
environment lab {
  require site == "bench"
}
claim ready {
  statement "This registered claim is a synthetic lifecycle fixture."
  environment lab
}
"#;
struct Fixture {
    directory: PathBuf,
    coordinator: Coordinator,
}
impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "eal-coordination-lifecycle-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir(&directory).unwrap();
        let workspace = directory.join("workspace");
        fs::create_dir(&workspace).unwrap();
        fs::write(workspace.join("case.eal"), SOURCE).unwrap();
        let server = ServerSpec {
            workspace: workspace.clone(),
            registry: None,
            database: Some(workspace.join("runs.sqlite3")),
            methods: Some("builtin".into()),
            limits: None,
        };
        // Supply a coherent private runtime directory for companion admission.
        // These are identity-only fixtures: the context mismatch must stop all
        // MCP calls before any executable can launch.
        let binaries = directory.join("bin");
        fs::create_dir(&binaries).unwrap();
        use std::os::unix::fs::{PermissionsExt, symlink};
        fs::set_permissions(&binaries, fs::Permissions::from_mode(0o700)).unwrap();
        fs::copy(std::env::current_exe().unwrap(), binaries.join("eal-mcp")).unwrap();
        for name in [
            "eal-coordinate",
            "eal-process-supervisor",
            "eal-method-worker",
        ] {
            symlink("eal-mcp", binaries.join(name)).unwrap();
        }
        let host = HostPaths::from_directory(&binaries);
        knowledge_with_host(&server, &host)
            .unwrap()
            .catalogue
            .register(
                "case.eal",
                RegisterOptions {
                    entry_id: Some("synthetic"),
                    context: Some(&json!({"site":"bench"})),
                    claims: Some(&["ready".into()]),
                    label: Some("Synthetic lifecycle fixture"),
                },
            )
            .unwrap();
        let task = TaskSpec::from_value(&json!({
            "task_id":"synthetic-task", "question":"A synthetic context mismatch",
            "entry_id":"synthetic", "claims":["ready"], "profile":"registered_claims/1",
            "requirements":{"context":{"site":"elsewhere"}}
        }))
        .unwrap();
        let coordinator =
            Coordinator::start_with_host(directory.join("run"), server, task, None, None, host)
                .unwrap();
        Self {
            directory,
            coordinator,
        }
    }
    fn reopen(&self) -> Coordinator {
        Coordinator::open_with_host(&self.coordinator.directory, self.coordinator.host.clone())
            .unwrap()
    }
    fn reserve(&self, role: &str) -> Value {
        self.coordinator
            .reserve_worker(role, "software_test")
            .unwrap()
    }
    fn finding(&self, reserved: &Value) -> Value {
        let finding = self
            .coordinator
            .query(reserved["call_id"].as_str().unwrap())
            .unwrap();
        assert_eq!(finding["check"]["code"], "context_mismatch");
        assert_eq!(finding["mcp"]["calls"], json!([]));
        finding
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}
fn text<'a>(value: &'a Value, name: &str) -> &'a str {
    value[name].as_str().unwrap()
}
fn raw(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).unwrap();
    bytes.push(b'\n');
    bytes
}
fn assessor_frame(coordinator: &Coordinator, finding: &Value) -> Value {
    let code = text(&finding["check"], "code");
    json!({"message":{
        "schema":"EAL/MAS-coordination/3", "kind":"disposition", "message_id":"synthetic-handoff",
        "task_id":coordinator.task.task_id, "task_digest":finding["task_digest"],
        "sender":"assessor", "recipient":"reviewer", "reply_to":"request", "attempt":0,
        "finding_id":finding["finding_id"], "code":code,
        "outcome":if matches!(code,"evidence_unavailable"|"effect_outcome_unknown") { "incomplete" } else { "rejected" }
    }, "consumed_message_ids":["request"]})
}
fn answer_frame(coordinator: &Coordinator, finding: &Value, conclusion: &str) -> Value {
    json!({"answer":{
        "task_digest":coordinator.anchor["task_digest"], "finding_id":finding["finding_id"],
        "claims":{"ready":{"assessment_id":null,"status":null,"assessed_at":null}},
        "scope":{"context":coordinator.task.requirements["context"],"requirements":coordinator.task.requirements},
        "conclusion":conclusion,"qualification":"conditional_on_registered_argument"
    }, "consumed_message_ids":["synthetic-handoff"]})
}
fn assert_no_mcp(coordinator: &Coordinator) {
    assert!(
        coordinator
            .state
            .calls()
            .unwrap()
            .iter()
            .all(|row| row["category"] != "mcp")
    );
}

#[test]
fn two_roles_retain_the_addressed_finding_and_complete_unassessed() {
    let fixture = Fixture::new();
    let coordinator = &fixture.coordinator;
    let assessor = fixture.reserve("assessor");
    assert_eq!(assessor["new"], true);
    let repeated = fixture
        .reopen()
        .reserve_worker("assessor", "software_test")
        .unwrap();
    assert_eq!(repeated["new"], false);
    assert_eq!(repeated["context"], assessor["context"]);
    let finding = fixture.finding(&assessor);
    assert!(
        coordinator
            .lookup(text(&assessor, "call_id"), text(&finding, "finding_id"))
            .is_err()
    );
    let frame = assessor_frame(coordinator, &finding);
    let recorded = coordinator
        .record(
            text(&assessor, "call_id"),
            &raw(&frame),
            "synthetic/assessor",
            "software_test",
            Some(&frame),
            None,
        )
        .unwrap();
    assert_eq!(recorded["accepted"], true);
    let reviewer = fixture.reserve("reviewer");
    assert_eq!(
        reviewer["context"]["received_messages"],
        json!([frame["message"]])
    );
    let retrieved = coordinator
        .lookup(text(&reviewer, "call_id"), text(&finding, "finding_id"))
        .unwrap();
    assert_eq!(retrieved, finding);
    let frame = answer_frame(coordinator, &finding, "out_of_scope");
    assert_eq!(
        coordinator
            .record(
                text(&reviewer, "call_id"),
                &raw(&frame),
                "synthetic/reviewer",
                "software_test",
                Some(&frame),
                None
            )
            .unwrap()["accepted"],
        true
    );
    let restarted = fixture.reopen();
    let status = restarted.resume().unwrap();
    assert_eq!(status["status"], "completed_unassessed");
    assert_eq!(status["answer"]["claims"]["ready"]["status"], Value::Null);
    assert_eq!(
        status["answer"]["qualification"],
        "conditional_on_registered_argument"
    );
    assert!(
        restarted
            .query(text(&assessor, "call_id"))
            .unwrap_err()
            .contains("first response")
    );
    assert_no_mcp(&restarted);
}

#[test]
fn invalid_first_raw_bytes_are_durable_and_replay_cannot_repair_them() {
    let fixture = Fixture::new();
    let coordinator = &fixture.coordinator;
    let assessor = fixture.reserve("assessor");
    let id = text(&assessor, "call_id");
    assert!(
        coordinator
            .record(id, b"{}", " ", "software_test", None, None)
            .is_err()
    );
    for id_whitespace in ["\u{1c}", "\u{1d}", "\u{1e}", "\u{1f}"] {
        assert!(
            coordinator
                .record(id, b"{}", id_whitespace, "software_test", None, None)
                .is_err(),
            "Whitespace-only identity must not consume the first attempt"
        );
    }
    assert!(
        coordinator
            .record(id, b"{}", "synthetic/first", "provider", None, None)
            .is_err()
    );
    assert_eq!(
        coordinator
            .state
            .effect("worker:assessor")
            .unwrap()
            .unwrap()["status"],
        "unknown"
    );
    let bytes = b"\xff\x00{not-json}\n";
    let result = coordinator
        .record(
            id,
            bytes,
            "synthetic/first",
            "software_test",
            None,
            Some(&json!({"first":true})),
        )
        .unwrap();
    assert_eq!(result["accepted"], false);
    assert_eq!(result["outcome"], "rejected");
    assert_eq!(STANDARD.decode(text(&result, "raw_base64")).unwrap(), bytes);
    assert_eq!(result["raw_digest"], format!("{:x}", Sha256::digest(bytes)));
    let replay = fixture
        .reopen()
        .record(
            id,
            bytes,
            "synthetic/first",
            "software_test",
            Some(&json!({"repaired":true})),
            Some(&json!({"first":false})),
        )
        .unwrap();
    assert_eq!(replay, result);
    assert!(
        coordinator
            .record(id, b"{}\n", "synthetic/first", "software_test", None, None)
            .unwrap_err()
            .contains("First response cannot be replaced")
    );
    let retained = fixture
        .reopen()
        .state
        .effect("worker:assessor")
        .unwrap()
        .unwrap();
    assert_eq!(retained["result"], result);
    assert_eq!(retained["acknowledged"], true);
    assert_no_mcp(coordinator);
}

#[test]
fn cancellation_blocks_new_effects_but_retains_the_first_arriving_response() {
    let fixture = Fixture::new();
    let coordinator = &fixture.coordinator;
    let assessor = fixture.reserve("assessor");
    let finding = fixture.finding(&assessor);
    let bytes = raw(&assessor_frame(coordinator, &finding));
    assert_eq!(coordinator.cancel().unwrap()["status"], "cancelled");
    assert!(
        coordinator
            .query(text(&assessor, "call_id"))
            .unwrap_err()
            .contains("cancelled")
    );
    assert!(
        coordinator
            .reserve_worker("reviewer", "software_test")
            .unwrap_err()
            .contains("cancelled")
    );
    let result = coordinator
        .record(
            text(&assessor, "call_id"),
            &bytes,
            "synthetic/late",
            "software_test",
            None,
            None,
        )
        .unwrap();
    assert_eq!(result["accepted"], false);
    assert!(text(&result["diagnostics"][0], "detail").contains("cancelled"));
    assert_eq!(STANDARD.decode(text(&result, "raw_base64")).unwrap(), bytes);
    let restarted = fixture.reopen();
    assert_eq!(restarted.resume().unwrap()["status"], "cancelled");
    assert_eq!(
        restarted.state.effect("worker:assessor").unwrap().unwrap()["status"],
        "committed"
    );
    assert_no_mcp(&restarted);
}

#[test]
fn cached_correspondence_changes_do_not_replace_the_committed_assessment_receipt() {
    let fixture = Fixture::new();
    let coordinator = &fixture.coordinator;
    let assessor = fixture.reserve("assessor");
    let original = fixture.finding(&assessor);
    let committed = coordinator.state.effect("assessment").unwrap().unwrap();
    let source = coordinator.server.workspace.join("case.eal");
    fs::write(
        &source,
        format!("{SOURCE}\n// Synthetic source revision.\n"),
    )
    .unwrap();
    let changed = coordinator.query(text(&assessor, "call_id")).unwrap();
    assert_eq!(changed["check"]["code"], "source_changed");
    assert_ne!(changed["finding_id"], original["finding_id"]);
    assert_eq!(changed["mcp"]["calls"], json!([]));
    assert_eq!(
        coordinator.state.effect("assessment").unwrap().unwrap(),
        committed
    );
    fs::write(&source, SOURCE).unwrap();
    assert_eq!(
        coordinator.query(text(&assessor, "call_id")).unwrap(),
        original
    );
    assert_eq!(
        fixture
            .reopen()
            .state
            .effect("assessment")
            .unwrap()
            .unwrap(),
        committed
    );
    assert_no_mcp(coordinator);
}

#[test]
fn unknown_effect_remains_unknown_after_source_change_with_null_counts() {
    let fixture = Fixture::new();
    let coordinator = &fixture.coordinator;
    let assessor = fixture.reserve("assessor");
    coordinator
        .state
        .reserve_effect(
            "assessment",
            &json!({"task_digest":coordinator.anchor["task_digest"]}),
        )
        .unwrap();
    let first = coordinator.query(text(&assessor, "call_id")).unwrap();
    assert_eq!(first["check"]["code"], "effect_outcome_unknown");
    assert_eq!(first["collected_count"], Value::Null);
    assert_eq!(first["reused_count"], Value::Null);
    fs::write(
        coordinator.server.workspace.join("case.eal"),
        format!("{SOURCE}\n// Synthetic source revision.\n"),
    )
    .unwrap();
    let unknown = coordinator.query(text(&assessor, "call_id")).unwrap();
    assert_eq!(unknown, first);
    assert_eq!(
        coordinator.state.effect("assessment").unwrap().unwrap()["status"],
        "unknown"
    );
    let frame = assessor_frame(coordinator, &unknown);
    assert_eq!(
        coordinator
            .record(
                text(&assessor, "call_id"),
                &raw(&frame),
                "synthetic/assessor",
                "software_test",
                None,
                None
            )
            .unwrap()["accepted"],
        true
    );
    let reviewer = fixture.reserve("reviewer");
    assert_eq!(
        coordinator
            .lookup(text(&reviewer, "call_id"), text(&unknown, "finding_id"))
            .unwrap(),
        unknown
    );
    let frame = answer_frame(coordinator, &unknown, "unresolved");
    assert_eq!(
        coordinator
            .record(
                text(&reviewer, "call_id"),
                &raw(&frame),
                "synthetic/reviewer",
                "software_test",
                None,
                None
            )
            .unwrap()["accepted"],
        true
    );
    let status = fixture.reopen().resume().unwrap();
    assert_eq!(status["status"], "completed_unassessed");
    assert_eq!(status["answer"]["conclusion"], "unresolved");
    assert_no_mcp(coordinator);
}
