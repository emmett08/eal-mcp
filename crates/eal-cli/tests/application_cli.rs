//! Public application entry point with real durable IDs and native companions.
#![cfg(unix)]
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

const SOURCE: &str = "language \"EAL/4\"\nenvironment lab {\n require site == \"bench\"\n}\ntool reader {\n version \"1\"\n}\nevidence reading {\n tool reader\n kind test\n environment lab\n max_age 3600\n input {\"sample\":1}\n require ok == true\n}\nreasoning measured {\n method \"structured/1\"\n rationale \"The supplied report supports the selected test result.\"\n}\nclaim works {\n statement \"The supplied test passed at this bench.\"\n environment lab\n}\nargument result = [evidence reading] via measured => works\n";
const NOW: &str = "2026-09-23T12:00:00Z";
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "eal-application-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        let out = Self(path);
        out.write("case.eal", SOURCE);
        out.write("context.json", r#"{"site":"bench"}"#);
        out.write(
            "tools.toml",
            "[tools.reader]\nkind='json_file'\nversion='1'\npath='observation.json'\n",
        );
        out.write("observation.json",&json!({"observed_at":NOW,"context":{"site":"bench"},
            "request":{"tool":"reader","tool_version":"1","input":{"sample":1},"context":{"site":"bench"}},
            "value":{"ok":true,"secret":"collector-secret"}}).to_string());
        out
    }
    fn write(&self, name: &str, text: &str) {
        std::fs::write(self.0.join(name), text).unwrap()
    }
    fn run(&self, args: &[&str]) -> Output {
        let eal = Path::new(env!("CARGO_BIN_EXE_eal"));
        let binaries = eal.parent().unwrap();
        let supervisor = binaries.join("eal-process-supervisor");
        let worker = binaries.join("eal-method-worker");
        assert!(
            supervisor.is_file() && worker.is_file(),
            "Build the workspace companion executables before running application CLI tests"
        );
        Command::new(eal)
            .current_dir(&self.0)
            .arg("--workspace")
            .arg(&self.0)
            .arg("--registry")
            .arg(self.0.join("tools.toml"))
            .arg("--supervisor")
            .arg(supervisor)
            .arg("--method-worker")
            .arg(worker)
            .args(args)
            .output()
            .unwrap()
    }
    fn pure(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_eal"))
            .current_dir(&self.0)
            .arg("--workspace")
            .arg(&self.0)
            .args([
                "--registry",
                "absent-registry.toml",
                "--supervisor",
                "absent-supervisor",
                "--method-worker",
                "absent-worker",
            ])
            .args(args)
            .output()
            .unwrap()
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
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("{e}: {}", String::from_utf8_lossy(&output.stdout)))
}

#[test]
fn register_find_collect_reason_packet_model_context_and_history_share_durable_state() {
    let workspace = Workspace::new();
    let entry = value(
        workspace.run(&[
            "register",
            "case.eal",
            "--entry-id",
            "demo",
            "--claim",
            "works",
            "--context",
            "@context.json",
        ]),
        0,
    );
    assert_eq!(entry["entry_id"], "demo");
    assert_eq!(entry["claims"], json!(["works"]));
    assert!(entry.get("source").is_none());
    let sources = value(workspace.run(&["sources"]), 0);
    assert_eq!(sources["entries"][0]["entry_id"], "demo");
    let found = value(
        workspace.run(&[
            "find",
            "supplied",
            "--claim",
            "works",
            "--path",
            "case.eal",
            "--context",
            "@context.json",
        ]),
        0,
    );
    assert_eq!(found["matches"][0]["source_digest"], entry["source_digest"]);
    let plan = value(workspace.run(&["plan", "case.eal", "--claim", "works"]), 0);
    assert_eq!(plan["evidence_ids"], json!(["reading"]));
    let collection = value(
        workspace.run(&[
            "collect",
            "case.eal",
            "--claim",
            "works",
            "--context",
            "@context.json",
        ]),
        0,
    );
    assert_eq!(collection["records"]["reading"]["collected_at"], NOW);
    let collection_id = collection["collection_id"].as_str().unwrap();
    let original_run = collection["records"]["reading"]["run_id"].as_str().unwrap();
    let assessment = value(
        workspace.run(&[
            "reason",
            "case.eal",
            "--context",
            "@context.json",
            "--collection",
            collection_id,
            "--now",
            NOW,
        ]),
        0,
    );
    assert_eq!(assessment["assessed_at"], NOW);
    assert_eq!(assessment["claims"]["works"]["status"], "supported");
    let assessment_id = assessment["assessment_id"].as_str().unwrap();
    let packet = value(
        workspace.run(&["packet", assessment_id, "--claim", "works"]),
        0,
    );
    assert_eq!(packet["schema"], "EAL/assessment-packet/2");
    assert_eq!(packet["assessment_id"], assessment_id);
    assert_eq!(packet["collection_id"], collection_id);
    assert_eq!(
        packet["claims"]["works"]["observation_ids"][0]["observation_id"],
        original_run
    );
    assert!(!packet.to_string().contains("collector-secret"));
    let explained = value(workspace.run(&["explain", assessment_id]), 0);
    assert_eq!(explained, assessment);
    let selected = value(
        workspace.run(&["explain", assessment_id, "--claim", "works"]),
        0,
    );
    assert_eq!(selected["result"]["status"], "supported");
    let prepared = value(
        workspace.run(&[
            "model-context",
            "demo",
            "--claim",
            "works",
            "--question",
            "Did the supplied test pass?",
            "--now",
            NOW,
        ]),
        0,
    );
    assert_eq!(prepared["schema"], "EAL/model-context/2");
    assert_eq!(prepared["assessment"]["reused_count"], 1);
    assert_eq!(prepared["assessment"]["collected_count"], 0);
    assert_eq!(
        prepared["messages"][1],
        json!({"role":"user","content":"Did the supplied test pass?"})
    );
    assert!(
        !prepared["messages"][0]["content"]
            .as_str()
            .unwrap()
            .contains("collector-secret")
    );
    assert!(
        !prepared["messages"][0]["content"]
            .as_str()
            .unwrap()
            .contains(original_run)
    );
    let history = value(workspace.run(&["history", "demo"]), 0);
    assert_eq!(
        history["assessments"][0]["assessment_id"],
        prepared["assessment"]["assessment_id"]
    );
    assert_eq!(history["assessments"][0]["assessed_at"], NOW);
    assert_eq!(
        history["assessments"][0]["claim_statuses"]["works"],
        "supported"
    );
    let full = value(
        workspace.run(&[
            "explain",
            prepared["assessment"]["full_explanation"]["assessment_id"]
                .as_str()
                .unwrap(),
        ]),
        0,
    );
    assert_eq!(
        full["collection_id"],
        prepared["assessment"]["collection_id"]
    );
    let fresh = value(
        workspace.run(&[
            "assess-known",
            "demo",
            "--claim",
            "works",
            "--now",
            NOW,
            "--reuse",
            "fresh",
        ]),
        0,
    );
    assert_eq!(fresh["reused_count"], 0);
    assert_eq!(fresh["collected_count"], 1);
    assert_eq!(fresh["assessed_at"], NOW);
    let exact = value(
        workspace.run(&[
            "history",
            "demo",
            "--source-digest",
            entry["source_digest"].as_str().unwrap(),
        ]),
        0,
    );
    assert_eq!(
        exact["assessments"][0]["assessment_id"],
        fresh["assessment_id"]
    );
}

#[test]
fn source_path_and_context_admission_return_regular_errors_before_collection() {
    let workspace = Workspace::new();
    let outside = Workspace::new();
    std::os::unix::fs::symlink(outside.0.join("case.eal"), workspace.0.join("escaped.eal"))
        .unwrap();
    for args in [
        vec!["validate", "escaped.eal"],
        vec!["register", "escaped.eal"],
        vec!["collect", "case.eal", "--context", "[]"],
        vec![
            "collect",
            "case.eal",
            "--context",
            r#"{"site":"bench","site":"field"}"#,
        ],
        vec![
            "collect",
            "case.eal",
            "--context",
            "{}",
            "--evidence",
            "absent",
        ],
        vec![
            "collect",
            "case.eal",
            "--context",
            "{}",
            "--claim",
            "works",
            "--evidence",
            "reading",
        ],
        vec!["sources", "--now", NOW],
    ] {
        let error = value(workspace.run(&args), 2);
        assert!(error["error"].is_string(), "{args:?}: {error}");
    }
    let invalid = SOURCE.replace("via measured", "via missing");
    workspace.write("invalid.eal", &invalid);
    assert_eq!(
        value(workspace.run(&["validate", "invalid.eal"]), 1)["valid"],
        false
    );
    let malformed = value(
        workspace.run(&[
            "collect",
            "case.eal",
            "--context",
            "@context.json",
            "--evidence",
            "reading",
            "--evidence",
            "reading",
        ]),
        2,
    );
    assert!(malformed["error"].is_string());
}

#[test]
fn pure_grounded_and_checked_export_do_not_create_a_store_and_reject_output_escape() {
    let workspace = Workspace::new();
    workspace.write(
        "graph.json",
        r#"{"arguments":["a","b","c"],"attacks":[["a","b"],["b","c"]]}"#,
    );
    let graph = value(workspace.pure(&["grounded", "graph.json"]), 0);
    assert_eq!(graph["accepted"], json!(["a", "c"]));
    workspace.write("limits.toml", "[limits]\njson_nodes=1000\n");
    let configured = value(
        workspace.pure(&["--limits", "limits.toml", "grounded", "graph.json"]),
        0,
    );
    assert_eq!(configured["accepted"], json!(["a", "c"]));
    assert!(!workspace.0.join(".eal").exists());
    let theory = json!({"premises":[{"atom":"ready","kind":"axiom"}],"rules":[],"contraries":[],"goal":"ready"});
    let formal = eal_aspic::solve_aspic(
        &json!({"theory":&theory}),
        &eal_core::ExecutionLimits::default(),
    )
    .unwrap();
    let supplied = json!({"theory":theory,"formal":formal});
    workspace.write("formal.json", &supplied.to_string());
    let exported = value(
        workspace.pure(&["export-aspic", "formal.json", "--output", "view.json"]),
        0,
    );
    assert_eq!(exported["formal_status"], "accepted");
    let view: Value =
        serde_json::from_slice(&std::fs::read(workspace.0.join("view.json")).unwrap()).unwrap();
    assert_eq!(view["schema"], "aspic-view/3");
    assert!(!workspace.0.join(".eal").exists());
    let outside = Workspace::new();
    let target = outside.0.join("must-not-be-created.json");
    std::os::unix::fs::symlink(&target, workspace.0.join("dangling.json")).unwrap();
    assert!(
        value(
            workspace.pure(&["export-aspic", "formal.json", "--output", "dangling.json"]),
            2
        )["error"]
            .is_string()
    );
    assert!(!target.exists());
    assert!(
        value(
            workspace.pure(&[
                "export-aspic",
                "formal.json",
                "--output",
                outside.0.join("outside.json").to_str().unwrap()
            ]),
            2
        )["error"]
            .is_string()
    );
    assert!(!outside.0.join("outside.json").exists());
    assert!(!workspace.0.join(".eal").exists());
}

#[test]
fn special_limits_are_rejected_before_a_blocking_read_or_database_creation() {
    use std::{
        process::Stdio,
        time::{Duration, Instant},
    };
    let workspace = Workspace::new();
    workspace.write("graph.json", r#"{"arguments":["a"],"attacks":[]}"#);
    let fifo = workspace.0.join("limits.fifo");
    rustix::fs::mkfifoat(
        rustix::fs::CWD,
        &fifo,
        rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR,
    )
    .unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_eal"))
        .current_dir(&workspace.0)
        .arg("--workspace")
        .arg(&workspace.0)
        .arg("--limits")
        .arg(&fifo)
        .args(["grounded", "graph.json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("The limits FIFO blocked before admission");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let error = value(child.wait_with_output().unwrap(), 2);
    assert!(error["error"].as_str().unwrap().contains("regular"));
    assert!(!workspace.0.join(".eal").exists());
}

#[test]
fn runtime_and_application_validation_ignore_unrelated_ambient_environment() {
    let workspace = Workspace::new();
    let mut results = Vec::new();
    for executable in [env!("CARGO_BIN_EXE_eal"), env!("CARGO_BIN_EXE_eal-runtime")] {
        let output = Command::new(executable)
            .current_dir(&workspace.0)
            .arg("--workspace")
            .arg(&workspace.0)
            .args(["validate", "case.eal"])
            .envs((0..260).map(|index| (format!("EAL_TEST_UNRELATED_{index}"), "unused")))
            .output()
            .unwrap();
        let result = value(output, 0);
        assert_eq!(result["valid"], true);
        results.push(result);
    }
    assert_eq!(results[0], results[1]);
}

#[test]
fn runtime_limits_remain_bounded_and_relative_to_the_workspace() {
    let workspace = Workspace::new();
    workspace.write("limits.toml", "[limits]\n");
    let output = Command::new(env!("CARGO_BIN_EXE_eal-runtime"))
        .current_dir(std::env::temp_dir())
        .arg("--workspace")
        .arg(&workspace.0)
        .args(["--limits", "limits.toml", "validate", "case.eal"])
        .output()
        .unwrap();
    assert_eq!(value(output, 0)["valid"], true);

    workspace.write("oversized.toml", &"x".repeat(16 * 1024 + 1));
    let output = Command::new(env!("CARGO_BIN_EXE_eal-runtime"))
        .current_dir(std::env::temp_dir())
        .arg("--workspace")
        .arg(&workspace.0)
        .args(["--limits", "oversized.toml", "validate", "case.eal"])
        .output()
        .unwrap();
    assert!(
        value(output, 2)["error"]
            .as_str()
            .unwrap()
            .contains("budget")
    );
}
