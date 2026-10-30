//! Exercise the installed command boundary with an empty executable search path.
#[path = "../../../tests/native_case.rs"]
mod native_case;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "eal-aspic-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn write(&self, name: &str, value: &str) {
        std::fs::write(self.0.join(name), value).unwrap();
    }
    fn json(&self, name: &str, value: &Value) {
        self.write(name, &serde_json::to_string(value).unwrap());
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_eal-core"))
            .arg("--workspace")
            .arg(&self.0)
            .args(args)
            .env("PATH", "")
            .output()
            .unwrap()
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn successful(output: Output) -> Value {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn explicit_theory_and_recomputed_export_reject_edited_labels() {
    let workspace = Workspace::new();
    let payload = native_case::theory();
    workspace.json("theory.json", &payload);
    let mut result = successful(workspace.run(&["aspic", "theory.json"]));
    assert_eq!(result["theory"], payload["theory"]);
    assert_eq!(result["formal"]["query_status"], "accepted");
    workspace.json("result.json", &result);
    let view = successful(workspace.run(&["export-aspic", "result.json"]));
    assert_eq!(view["schema"], "aspic-view/3");
    assert_eq!(
        view["validation"],
        json!({"formal_result":"recomputed","provenance":"supplied"})
    );
    assert_eq!(
        view["defeats"].as_array().unwrap().len(),
        result["formal"]["defeats"].as_array().unwrap().len()
    );
    result["formal"]["arguments"][0]["label"] = json!("out");
    workspace.json("result.json", &result);
    let rejected = workspace.run(&["export-aspic", "result.json"]);
    assert_eq!(rejected.status.code(), Some(2));
    assert!(rejected.stdout.is_empty());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("recomputation"));
}

#[test]
fn no_stable_extension_and_exhausted_search_have_distinct_machine_results() {
    let workspace = Workspace::new();
    let payload = json!({"theory":{
        "premises":[{"atom":"a","kind":"ordinary","rank":500},{"atom":"b","kind":"ordinary","rank":500},{"atom":"c","kind":"ordinary","rank":500}],
        "rules":[],"contraries":[{"attacker":"a","target":"b"},{"attacker":"b","target":"c"},{"attacker":"c","target":"a"}],"goal":"a",
        "options":{"semantics":"stable","query_mode":"sceptical"}
    }});
    workspace.json("theory.json", &payload);
    let result = successful(workspace.run(&["aspic", "theory.json"]));
    assert_eq!(result["formal"]["query_status"], "no_extension");
    assert_eq!(result["formal"]["query_accepted"], false);
    assert_eq!(result["formal"]["query_rejected"], false);
    workspace.write("limits.toml", "[limits]\nextension_search = 1\n");
    let exhausted = workspace.run(&["--limits", "limits.toml", "aspic", "theory.json"]);
    assert_eq!(exhausted.status.code(), Some(1));
    let exhausted: Value = serde_json::from_slice(&exhausted.stdout).unwrap();
    assert_eq!(exhausted["status"], "incomplete");
    assert_eq!(exhausted["complete"], false);
    assert_eq!(exhausted["details"], json!({}));
    assert!(exhausted.get("formal").is_none());
    assert!(exhausted.get("query_accepted").is_none());
}

#[test]
fn source_snapshot_compilation_and_export_preserve_formal_and_authored_results() {
    let workspace = Workspace::new();
    let case = native_case::case();
    workspace.write("source.eal", case["source"].as_str().unwrap());
    workspace.json("records.json", &case["records"]);
    workspace.json("context.json", &case["context"]);
    let arguments = [
        "compile-aspic",
        "source.eal",
        "--claim",
        case["goal"].as_str().unwrap(),
        "--now",
        case["now"].as_str().unwrap(),
        "--records",
        "records.json",
        "--context",
        "@context.json",
    ];
    let result = successful(workspace.run(&arguments));
    assert_eq!(result["profile"], "EAL/4-compiled-aspic/5");
    assert_eq!(result["formal"]["query_status"], "accepted");
    assert_eq!(result["claim_status"], "supported");
    assert_eq!(result["authored_claim_status"], "supported");
    workspace.json("result.json", &result);
    let view = successful(workspace.run(&["export-aspic", "result.json"]));
    assert_eq!(view["snapshot_digest"], result["snapshot_digest"]);
    assert_eq!(view["goal_claim"], case["goal"]);
    let repeated = successful(workspace.run(&arguments));
    assert_eq!(repeated, result);
    workspace.write("limits.toml", "[limits]\nformal_arguments = 1\n");
    let exhausted = workspace.run(&[
        "--limits",
        "limits.toml",
        "compile-aspic",
        "source.eal",
        "--claim",
        case["goal"].as_str().unwrap(),
        "--now",
        case["now"].as_str().unwrap(),
        "--context",
        "@context.json",
        "--records",
        "records.json",
    ]);
    assert_eq!(exhausted.status.code(), Some(1));
    let incomplete: Value = serde_json::from_slice(&exhausted.stdout).unwrap();
    assert_eq!(incomplete["status"], "incomplete");
    assert!(incomplete.get("theory").is_none());
}

#[test]
fn methods_are_explicit_and_solver_options_are_checked_before_execution() {
    let workspace = Workspace::new();
    assert_eq!(
        successful(workspace.run(&["describe"]))["methods"]
            .as_object()
            .unwrap()
            .len(),
        9
    );
    assert_eq!(
        successful(workspace.run(&["--methods", "aspic", "describe"]))["methods"]
            .as_object()
            .unwrap()
            .len(),
        10
    );
    assert_eq!(
        successful(workspace.run(&["--methods", "all", "describe"]))["methods"]
            .as_object()
            .unwrap()
            .len(),
        14
    );
    for arguments in [
        vec!["compile-aspic", "absent.eal", "--claim", "c"],
        vec![
            "compile-aspic",
            "absent.eal",
            "--now",
            "2026-09-23T12:00:00Z",
        ],
        vec!["aspic", "absent.json", "--semantics", "stable"],
        vec![
            "compile-aspic",
            "absent.eal",
            "--claim",
            "c",
            "--now",
            "2026-09-23T12:00:00Z",
            "--semantics",
            "approximate",
        ],
    ] {
        let output = workspace.run(&arguments);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("usage:"));
    }
}

#[test]
fn registered_typed_theory_binds_queries_and_propagates_incomplete_assessment() {
    let workspace = Workspace::new();
    let case = native_case::typed_case();
    workspace.write("source.eal", case["source"].as_str().unwrap());
    workspace.json("records.json", &case["records"]);
    workspace.json("context.json", &case["context"]);
    for (budget, exit, complete, status) in
        [(128, 0, true, "supported"), (1, 1, false, "unsupported")]
    {
        workspace.write(
            "limits.toml",
            &format!("[limits]\nformal_arguments = {budget}\n"),
        );
        let output = workspace.run(&[
            "--methods",
            "aspic",
            "--limits",
            "limits.toml",
            "evaluate",
            "source.eal",
            "--now",
            native_case::NOW,
            "--context",
            "@context.json",
            "--records",
            "records.json",
        ]);
        assert_eq!(
            output.status.code(),
            Some(exit),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["complete"], complete);
        assert_eq!(result["claims"]["accepted"]["status"], status);
        if !complete {
            assert_eq!(
                result["arguments"]["formal_arg"]["reasoning_result"]["details"],
                json!({})
            );
        }
    }
}
