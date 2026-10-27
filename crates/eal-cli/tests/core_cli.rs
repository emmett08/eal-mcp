//! Exercise the pure semantic service through its actual dependency-free binary.
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
        let root = std::env::temp_dir().join(format!(
            "eal-core-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        Self(root)
    }
    fn write(&self, name: &str, source: &str) {
        std::fs::write(self.0.join(name), source).unwrap();
    }
    fn run(&self, arguments: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_eal-core"))
            .arg("--workspace")
            .arg(&self.0)
            .args(arguments)
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

fn output_json(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn native_evaluation_retains_supplied_observation_and_scope() {
    let workspace = Workspace::new();
    let case = native_case::case();
    workspace.write("source.eal", case["source"].as_str().unwrap());
    workspace.write(
        "records.json",
        &serde_json::to_string(&case["records"]).unwrap(),
    );
    workspace.write(
        "context.json",
        &serde_json::to_string(&case["context"]).unwrap(),
    );
    let assessment = output_json(&workspace.run(&[
        "evaluate",
        "source.eal",
        "--now",
        case["now"].as_str().unwrap(),
        "--context",
        "@context.json",
        "--records",
        "records.json",
    ]));
    assert_eq!(assessment["claims"]["working"]["status"], "supported");
    assert_eq!(assessment["claims"]["downstream"]["status"], "supported");
    assert_eq!(assessment["evidence"]["observed"]["status"], "available");
    assert_eq!(assessment["valid"], true);
}

#[test]
fn compile_resolves_only_workspace_eal_imports_and_validate_reports_errors() {
    let workspace = Workspace::new();
    workspace.write(
        "library.eal",
        "language \"EAL/4\"\nenvironment lab {\nrequire true\n}\n",
    );
    workspace.write("source.eal", "language \"EAL/4\"\nimport \"library.eal\" as shared\nclaim c {\nstatement \"Scoped claim\"\nenvironment shared.lab\n}\n");
    let compiled = output_json(&workspace.run(&["compile", "source.eal"]));
    assert!(compiled["environments"].get("shared.lab").is_some());
    assert!(compiled["imports"].get("library.eal").is_some());
    assert_eq!(
        output_json(&workspace.run(&["validate", "source.eal"]))["valid"],
        true
    );
    workspace.write(
        "bad.eal",
        "language \"EAL/4\"\nclaim c {\nstatement \"Scope missing\"\nenvironment absent\n}\n",
    );
    let invalid = workspace.run(&["validate", "bad.eal"]);
    assert_eq!(invalid.status.code(), Some(1));
    let invalid: Value = serde_json::from_slice(&invalid.stdout).unwrap();
    assert_eq!(invalid["valid"], false);
    workspace.write("library.txt", "language \"EAL/4\"\n");
    workspace.write(
        "source.eal",
        "language \"EAL/4\"\nimport \"library.txt\" as shared\n",
    );
    let invalid = workspace.run(&["compile", "source.eal"]);
    assert_eq!(invalid.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&invalid.stderr).contains(".eal"));
}

#[test]
fn grounded_and_host_limits_use_native_paths() {
    let workspace = Workspace::new();
    workspace.write(
        "graph.json",
        r#"{"arguments":["a","b","c"],"attacks":[["a","b"],["b","c"]]}"#,
    );
    let graph = output_json(&workspace.run(&["grounded", "graph.json"]));
    assert_eq!(graph["accepted"], json!(["a", "c"]));
    workspace.write("limits.toml", "[limits]\nsource_bytes = 4\n");
    workspace.write("source.eal", "language \"EAL/4\"\n");
    let invalid = workspace.run(&["--limits", "limits.toml", "compile", "source.eal"]);
    assert_eq!(invalid.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("byte budget"));
    workspace.write("limits.toml", "[limits]\njson_nodes = 0\n");
    assert_eq!(
        workspace
            .run(&["--limits", "limits.toml", "describe"])
            .status
            .code(),
        Some(2)
    );
}

#[test]
fn malformed_json_and_implicit_time_are_rejected_before_assessment() {
    let workspace = Workspace::new();
    workspace.write("source.eal", "language \"EAL/4\"\n");
    assert_eq!(
        workspace.run(&["evaluate", "source.eal"]).status.code(),
        Some(2)
    );
    workspace.write("records.json", r#"{"a":1,"\u0061":2}"#);
    let invalid = workspace.run(&[
        "evaluate",
        "source.eal",
        "--now",
        "2026-09-23T12:00:00Z",
        "--records",
        "records.json",
    ]);
    assert_eq!(invalid.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("Duplicate"));
}

#[test]
fn planning_and_checked_formatting_preserve_source_bound_semantics() {
    let workspace = Workspace::new();
    let case = native_case::case();
    workspace.write("source.eal", case["source"].as_str().unwrap());
    let plan = output_json(&workspace.run(&[
        "plan",
        "source.eal",
        "--claim",
        case["claim"].as_str().unwrap(),
    ]));
    assert_eq!(plan["claim"], "working");
    assert_eq!(plan["calls"].as_array().unwrap().len(), 1);
    let formatted = workspace.run(&["format", "source.eal"]);
    assert!(
        formatted.status.success(),
        "{}",
        String::from_utf8_lossy(&formatted.stderr)
    );
    workspace.write(
        "formatted.eal",
        std::str::from_utf8(&formatted.stdout).unwrap(),
    );
    assert_eq!(
        output_json(&workspace.run(&[
            "plan",
            "formatted.eal",
            "--claim",
            case["claim"].as_str().unwrap()
        ])),
        plan
    );
    assert_eq!(
        workspace.run(&["format", "formatted.eal"]).stdout,
        formatted.stdout
    );
    assert_eq!(
        workspace.run(&["plan", "source.eal"]).status.code(),
        Some(2)
    );
    assert_eq!(
        workspace
            .run(&["compile", "source.eal", "--claim", "root"])
            .status
            .code(),
        Some(2)
    );
}

#[test]
fn optional_engineering_methods_require_an_explicit_profile() {
    let workspace = Workspace::new();
    let builtin = output_json(&workspace.run(&["describe"]));
    let optional = output_json(&workspace.run(&["--methods", "engineering", "describe"]));
    assert_eq!(builtin["methods"].as_object().unwrap().len(), 8);
    assert_eq!(optional["methods"].as_object().unwrap().len(), 12);
    assert!(builtin["methods"].get("engineering/rms/1").is_none());
    assert!(optional["methods"].get("engineering/rms/1").is_some());
    workspace.write("optional.eal", "language \"EAL/4\"\nenvironment lab {\nrequire true\n}\ntool sensor {\nversion \"1\"\n}\nevidence samples {\ntool sensor\nkind measurement_series\nenvironment lab\nmax_age 60\nrequire true\n}\nreasoning rms {\nmethod \"engineering/rms/1\"\nrationale \"Compute the RMS of the supplied samples.\"\nrequire \"rms\" <= 4\n}\nclaim result {\nstatement \"The registered method computes the result.\"\nenvironment lab\n}\nargument computation = [evidence samples] via rms => result\n");
    assert_eq!(
        workspace.run(&["validate", "optional.eal"]).status.code(),
        Some(1)
    );
    assert_eq!(
        output_json(&workspace.run(&["--methods", "engineering", "validate", "optional.eal"]))["valid"],
        true
    );
    assert_eq!(
        workspace
            .run(&["--methods", "unknown", "describe"])
            .status
            .code(),
        Some(2)
    );
}

#[cfg(unix)]
#[test]
fn import_symlinks_cannot_escape_the_selected_workspace() {
    use std::os::unix::fs::symlink;
    let workspace = Workspace::new();
    let outside = Workspace::new();
    outside.write("library.eal", "language \"EAL/4\"\n");
    symlink(
        outside.0.join("library.eal"),
        workspace.0.join("library.eal"),
    )
    .unwrap();
    workspace.write(
        "source.eal",
        "language \"EAL/4\"\nimport \"library.eal\" as outside\n",
    );
    let invalid = workspace.run(&["compile", "source.eal"]);
    assert_eq!(invalid.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("escapes"));
}
