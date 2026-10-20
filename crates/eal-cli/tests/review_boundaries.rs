//! Review checks through the actual binary, including raised host budgets.
use serde_json::Value;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "eal-reviewed-cli-{}-{}",
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

fn regular_error(output: Output) {
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "Failed commands must not emit partial JSON"
    );
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        !stderr.contains("panicked") && !stderr.contains("stack overflow"),
        "{stderr}"
    );
}

#[test]
fn deeply_nested_records_and_context_are_regular_cli_errors_with_raised_limits() {
    let workspace = Workspace::new();
    workspace.write("source.eal", "language \"EAL/3\"\n");
    workspace.write(
        "limits.toml",
        "[limits]\njson_depth = 9223372036854775807\n",
    );
    workspace.write(
        "deep.json",
        &format!("{{\"payload\":{}0{}}}", "[".repeat(5000), "]".repeat(5000)),
    );
    regular_error(workspace.run(&[
        "--limits",
        "limits.toml",
        "evaluate",
        "source.eal",
        "--now",
        "2026-09-23T12:00:00Z",
        "--records",
        "deep.json",
    ]));
    regular_error(workspace.run(&[
        "--limits",
        "limits.toml",
        "evaluate",
        "source.eal",
        "--now",
        "2026-09-23T12:00:00Z",
        "--context",
        "@deep.json",
    ]));
}

#[test]
fn compile_keeps_literal_span_payloads_and_emits_flat_semantic_locations() {
    let workspace = Workspace::new();
    workspace.write("source.eal", "language \"EAL/3\"\ntool t { version \"1\"\n}\nenvironment e { require true\n}\nevidence x { tool t\nenvironment e\nmax_age 1\nkind test\ninput {\"span\":{\"start\":{\"line\":90},\"end\":{\"column\":91}}}\nrequire true\n}\nstrict missing reviewed \"review/span\"\n");
    let output = workspace.run(&["compile", "source.eal"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let program: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        program["evidence"]["x"]["input"]["span"]["start"]["line"],
        90
    );
    assert!(program["locations"]["x"].get("line").is_some());
    assert!(program["locations"]["x"].get("start").is_none());
    assert!(
        program["argumentation_directives"][0]["span"]
            .get("line")
            .is_some()
    );
    assert!(
        program["argumentation_directives"][0]["span"]
            .get("start")
            .is_none()
    );
}

#[test]
fn non_finite_numeric_fields_are_diagnosed_without_successful_null_substitution() {
    let workspace = Workspace::new();
    workspace.write("source.eal", "language \"EAL/3\"\ntool t { version \"1\"\n}\nenvironment e { require true\n}\nevidence x { tool t\nenvironment e\nmax_age 1e999\nkind test\nrequire true\n}\n");
    regular_error(workspace.run(&["compile", "source.eal"]));
    let output = workspace.run(&["validate", "source.eal"]);
    assert_eq!(output.status.code(), Some(1));
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["valid"], false);
    assert!(
        result["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["code"] == "invalid_age")
    );
}
