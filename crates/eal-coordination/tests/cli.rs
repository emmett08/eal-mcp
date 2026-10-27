//! Real native CLI lifecycle with manually authored software responses.
#![cfg(unix)]
use base64::{Engine, engine::general_purpose::STANDARD};
use eal_coordination::{contracts::ServerSpec, host::HostPaths, profiles};
use eal_runtime::catalogue::RegisterOptions;
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("eal-coordinate-cli-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn write(&self, name: &str, bytes: impl AsRef<[u8]>) {
        std::fs::write(self.0.join(name), bytes).unwrap();
    }
    fn json(&self, name: &str, value: &Value) {
        self.write(name, serde_json::to_vec_pretty(value).unwrap());
    }
    fn run(&self, arguments: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_eal-coordinate"))
            .current_dir(&self.0)
            .args(arguments)
            .output()
            .unwrap()
    }
    fn setup(&self) -> Value {
        self.setup_with_methods(None, false)
    }
    fn setup_with_methods(&self, methods: Option<&str>, matching_context: bool) -> Value {
        let host = HostPaths::from_directory(
            Path::new(env!("CARGO_BIN_EXE_eal-coordinate"))
                .parent()
                .unwrap(),
        );
        let server = ServerSpec::from_value(&json!({"workspace":self.0,"registry":null,"database":null,"methods":methods,"limits":null}), &self.0).unwrap();
        self.write("source.eal", b"language \"EAL/4\"\nenvironment lab {\nrequire true\n}\nclaim ready {\nstatement \"A bounded registered claim.\"\nenvironment lab\n}\n");
        let knowledge = profiles::knowledge_with_host(&server, &host).unwrap();
        knowledge
            .catalogue
            .register(
                "source.eal",
                RegisterOptions {
                    entry_id: Some("sample"),
                    context: Some(&json!({"dataset":"synthetic"})),
                    ..Default::default()
                },
            )
            .unwrap();
        let task = json!({"task_id":"review","question":"What does the selected sample support?","entry_id":"sample","claims":["ready"],"profile":"registered_claims/1","requirements":{"context":{"dataset":if matching_context {"synthetic"} else {"another-sample"}}}});
        self.json(
            "spec.json",
            &json!({"server":server.to_value().unwrap(),"task":task}),
        );
        task
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn value(output: Output, code: i32) -> Value {
    assert_eq!(
        output.status.code(),
        Some(code),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn engineering_gateway_argv_works_in_an_empty_worker_environment_through_actual_mcp() {
    let workspace = Workspace::new();
    workspace.setup_with_methods(Some("engineering"), true);
    value(
        workspace.run(&["start", "--run", "run", "--spec", "spec.json"]),
        0,
    );
    let reserved = value(
        workspace.run(&[
            "reserve",
            "--run",
            "run",
            "--role",
            "assessor",
            "--execution-mode",
            "software_test",
        ]),
        0,
    );
    let argv = reserved["context"]["gateway"]["argv"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect::<Vec<_>>();
    let finding = value(
        Command::new(argv[0])
            .args(&argv[1..])
            .current_dir(&workspace.0)
            .env_clear()
            .output()
            .unwrap(),
        0,
    );
    assert_eq!(finding["check"]["ok"], true, "{}", finding["check"]);
    assert_eq!(finding["assessments"]["ready"]["status"], "unsupported");
    assert_eq!(finding["collected_count"], 0);
    assert_eq!(
        finding["mcp"]["calls"]
            .as_array()
            .unwrap()
            .iter()
            .map(|call| call["name"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "eal_sources",
            "eal_assess_known",
            "eal_packet",
            "eal_explain"
        ]
    );
    assert!(
        finding["mcp"]["calls"]
            .as_array()
            .unwrap()
            .iter()
            .all(|call| call["outcome"] == "success")
    );
}

#[test]
fn manual_two_role_handoff_reopens_exact_first_responses_and_exports_typed_unassessed_answer() {
    let workspace = Workspace::new();
    let task = workspace.setup();
    assert_eq!(
        value(
            workspace.run(&["start", "--run", "run", "--spec", "spec.json"]),
            0
        )["status"],
        "ready_assessor"
    );
    let reserved = value(
        workspace.run(&[
            "reserve",
            "--run",
            "run",
            "--role",
            "assessor",
            "--execution-mode",
            "software_test",
            "--context-out",
            "assessor-context.json",
        ]),
        0,
    );
    assert_eq!(reserved["new"], true);
    assert_eq!(
        serde_json::from_slice::<Value>(
            &std::fs::read(workspace.0.join("assessor-context.json")).unwrap()
        )
        .unwrap(),
        reserved["context"]
    );
    assert_eq!(
        Path::new(reserved["context"]["gateway"]["argv"][0].as_str().unwrap())
            .file_name()
            .unwrap(),
        "eal-coordinate"
    );
    assert_eq!(
        serde_json::from_slice::<Value>(
            &std::fs::read(reserved["context"]["response_schema"].as_str().unwrap()).unwrap()
        )
        .unwrap(),
        serde_json::from_str::<Value>(eal_coordination::RESPONSE_SCHEMA).unwrap()
    );
    let call = reserved["call_id"].as_str().unwrap();
    let finding = value(
        workspace.run(&["query", "--run", "run", "--call-id", call]),
        0,
    );
    assert_eq!(finding["check"]["code"], "context_mismatch");
    assert_eq!(finding["mcp"]["calls"], json!([]));
    let request = &reserved["context"]["received_messages"][0];
    let response = json!({"message":{"schema":"EAL/MAS-coordination/3","kind":"disposition","message_id":"assessor-first","task_id":task["task_id"],"task_digest":finding["task_digest"],"sender":"assessor","recipient":"reviewer","reply_to":request["message_id"],"attempt":0,"finding_id":finding["finding_id"],"code":"context_mismatch","outcome":"rejected"},"consumed_message_ids":[request["message_id"]]});
    let mut raw = serde_json::to_vec_pretty(&response).unwrap();
    raw.push(b'\n');
    workspace.write("assessor.json", &raw);
    let retained = value(
        workspace.run(&[
            "record",
            "--run",
            "run",
            "--call-id",
            call,
            "--response",
            "assessor.json",
            "--execution-id",
            "software-fixture/assessor",
            "--execution-mode",
            "software_test",
        ]),
        0,
    );
    assert_eq!(retained["accepted"], true);
    assert_eq!(
        STANDARD
            .decode(retained["raw_base64"].as_str().unwrap())
            .unwrap(),
        raw
    );
    let duplicate = value(
        workspace.run(&[
            "reserve",
            "--run",
            "run",
            "--role",
            "assessor",
            "--execution-mode",
            "software_test",
        ]),
        0,
    );
    assert_eq!(duplicate["new"], false);
    assert_eq!(
        value(
            workspace.run(&[
                "record",
                "--run",
                "run",
                "--call-id",
                call,
                "--response",
                "assessor.json",
                "--execution-id",
                "software-fixture/assessor",
                "--execution-mode",
                "software_test"
            ]),
            0
        ),
        retained
    );
    let reviewer = value(
        workspace.run(&[
            "reserve",
            "--run",
            "run",
            "--role",
            "reviewer",
            "--execution-mode",
            "software_test",
        ]),
        0,
    );
    assert_eq!(
        reviewer["context"]["received_messages"],
        json!([response["message"]])
    );
    let reviewer_call = reviewer["call_id"].as_str().unwrap();
    let looked_up = value(
        workspace.run(&[
            "lookup",
            "--run",
            "run",
            "--call-id",
            reviewer_call,
            "--finding-id",
            finding["finding_id"].as_str().unwrap(),
        ]),
        0,
    );
    assert_eq!(looked_up, finding);
    let answer = json!({"task_digest":finding["task_digest"],"finding_id":finding["finding_id"],"claims":{"ready":{"assessment_id":null,"status":null,"assessed_at":null}},"scope":{"context":task["requirements"]["context"],"requirements":task["requirements"]},"conclusion":"out_of_scope","qualification":"conditional_on_registered_argument"});
    workspace.json(
        "reviewer.json",
        &json!({"answer":answer,"consumed_message_ids":["assessor-first"]}),
    );
    assert_eq!(
        value(
            workspace.run(&[
                "record",
                "--run",
                "run",
                "--call-id",
                reviewer_call,
                "--response",
                "reviewer.json",
                "--execution-id",
                "software-fixture/reviewer",
                "--execution-mode",
                "software_test"
            ]),
            0
        )["accepted"],
        true
    );
    let status = value(workspace.run(&["resume", "--run", "run"]), 0);
    assert_eq!(status["status"], "completed_unassessed");
    assert_eq!(status["answer"], answer);
    let export = value(
        workspace.run(&[
            "export",
            "--run",
            "run",
            "--output",
            "retained/evidence.json",
        ]),
        0,
    );
    assert_eq!(
        export,
        serde_json::from_slice::<Value>(
            &std::fs::read(workspace.0.join("retained/evidence.json")).unwrap()
        )
        .unwrap()
    );
    let assessor = export["effects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|effect| effect["key"] == "worker:assessor")
        .unwrap();
    assert_eq!(
        STANDARD
            .decode(assessor["result"]["raw_base64"].as_str().unwrap())
            .unwrap(),
        raw
    );
    workspace.json("assessor.json", &json!({"repaired":true}));
    let replacement = workspace.run(&[
        "record",
        "--run",
        "run",
        "--call-id",
        call,
        "--response",
        "assessor.json",
        "--execution-id",
        "software-fixture/assessor",
        "--execution-mode",
        "software_test",
    ]);
    assert_eq!(replacement.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&replacement.stderr).contains("First response"));
    assert_eq!(
        value(workspace.run(&["cancel", "--run", "run"]), 1)["status"],
        "cancelled"
    );
}

#[test]
fn execute_retains_failed_first_command_and_never_relaunches_it() {
    let workspace = Workspace::new();
    workspace.setup();
    value(
        workspace.run(&["start", "--run", "run", "--spec", "spec.json"]),
        0,
    );
    workspace.json(
        "private.json",
        &json!({"PRIVATE_TOKEN":"private-cli-value"}),
    );
    let argv = json!([
        "/bin/sh",
        "-c",
        "printf x > launched; printf '%s' \"$PRIVATE_TOKEN\" >&2; printf '%s' '{}'"
    ])
    .to_string();
    let result = value(
        workspace.run(&[
            "execute",
            "--run",
            "run",
            "--execution-mode",
            "software_test",
            "--worker-argv",
            &argv,
            "--worker-environment",
            "private.json",
        ]),
        1,
    );
    assert_eq!(result["status"], "failed");
    assert!(workspace.0.join("launched").is_file());
    let export = value(
        workspace.run(&["export", "--run", "run", "--output", "evidence.json"]),
        0,
    );
    assert!(!export.to_string().contains("private-cli-value"));
    let assessor = export["effects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|effect| effect["key"] == "worker:assessor")
        .unwrap();
    assert_eq!(assessor["result"]["execution_mode"], "software_test");
    assert_eq!(
        STANDARD
            .decode(assessor["result"]["raw_base64"].as_str().unwrap())
            .unwrap(),
        b"{}"
    );
    std::fs::remove_file(workspace.0.join("launched")).unwrap();
    assert_eq!(
        value(
            workspace.run(&[
                "execute",
                "--run",
                "run",
                "--execution-mode",
                "software_test",
                "--worker-argv",
                &argv
            ]),
            1
        )["status"],
        "failed"
    );
    assert!(!workspace.0.join("launched").exists());
}

#[test]
fn schema_and_usage_are_pure_and_invalid_start_documents_do_not_create_runs() {
    let workspace = Workspace::new();
    let schema = value(workspace.run(&["--response-schema"]), 0);
    assert_eq!(schema["$id"], "EAL/coordination-worker-response/1");
    let invalid = workspace.run(&[
        "reserve",
        "--run",
        "absent",
        "--role",
        "invented",
        "--execution-mode",
        "software_test",
    ]);
    assert_eq!(invalid.status.code(), Some(2));
    assert!(!workspace.0.join("absent").exists());
    workspace.write(
        "duplicate.json",
        b"{\"server\":{},\"server\":{},\"task\":{}}",
    );
    assert_eq!(
        workspace
            .run(&["start", "--run", "absent", "--spec", "duplicate.json"])
            .status
            .code(),
        Some(1)
    );
    assert!(!workspace.0.join("absent").exists());
    workspace.json(
        "demo.json",
        &json!({"server":{},"tasks":[{"task_id":"one"},{"task_id":"two"}]}),
    );
    assert_eq!(
        workspace
            .run(&["start", "--run", "absent", "--demo", "demo.json"])
            .status
            .code(),
        Some(1)
    );
    assert!(!workspace.0.join("absent").exists());
}
