//! Explicit installed-path verification using operator-supplied model assets.
use serde_json::{Value, json};
use std::{env, fs, path::PathBuf, process::Command};

fn configured_path(name: &str) -> PathBuf {
    env::var_os(name)
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("set {name} for this explicitly requested smoke test"))
}

#[test]
#[ignore = "requires pinned local weights and built native companions; see docs/local-model-host.md"]
fn pinned_rust_model_validates_native_source_and_preserves_existing_trace() {
    let binary = configured_path("EAL_MODEL_HOST_BIN");
    let base = configured_path("EAL_MODEL_BASE");
    let adapter = configured_path("EAL_MODEL_ADAPTER");
    let native = configured_path("EAL_MODEL_NATIVE");
    let temporary = tempfile::tempdir().unwrap();
    let task_path = temporary.path().join("task.json");
    let workspace = temporary.path().join("workspace");
    let output = temporary.path().join("run.json");
    let source = include_str!("../../../examples/executable/finite-inventory/case.eal");
    fs::write(
        &task_path,
        serde_json::to_vec(&json!({
            "id":"native-validation-smoke",
            "request":"Use eal_validate to check the supplied EAL source and return its diagnostics without acquiring observations.",
            "state":{"source":source,"context":{},"claim":"inventory_verified",
                "now":"2026-11-08T12:00:00Z"},
            "allowed_tools":["eal_validate"]
        })).unwrap(),
    ).unwrap();
    let mut command = Command::new(binary);
    command
        .args(["run", "--base"])
        .arg(base)
        .arg("--adapter")
        .arg(adapter)
        .arg("--native-executable")
        .arg(native)
        .arg("--workspace")
        .arg(workspace)
        .arg("--task")
        .arg(task_path)
        .arg("--output")
        .arg(&output)
        .env("RAYON_NUM_THREADS", "4")
        .env("TOKENIZERS_PARALLELISM", "false");
    let result = command.output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let retained = fs::read(&output).unwrap();
    let report: Value = serde_json::from_slice(&retained).unwrap();
    assert_eq!(report["bundle"]["status"], "verified");
    assert_eq!(report["configuration"]["precision"], "f32");
    assert_eq!(report["run"]["stop"], "finished", "{report}");
    assert_eq!(report["run"]["tool_dispatches"], 1);
    let calls = report["run"]["calls"].as_array().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0]["tool"], "eal_validate");
    assert_eq!(calls[0]["arguments"]["source"], source);
    assert_eq!(calls[0]["response"]["structuredContent"]["valid"], true);
    let repeat = command.output().unwrap();
    assert!(
        !repeat.status.success(),
        "existing output must stop a repeated invocation"
    );
    assert_eq!(fs::read(&output).unwrap(), retained);
}
