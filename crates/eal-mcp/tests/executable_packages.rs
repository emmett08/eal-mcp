//! Portable source/TOML/tool packages exercise the real MCP host and runtime.
#![cfg(unix)]
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Package {
    root: PathBuf,
    package: PathBuf,
}
impl Package {
    fn copy(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "eal-executable-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let package = root.join("moved-package");
        std::fs::create_dir_all(package.join("tools")).unwrap();
        let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let source = repository.join("examples/executable").join(name);
        for file in ["case.eal", "tools.toml"] {
            std::fs::copy(source.join(file), package.join(file)).unwrap();
        }
        for file in std::fs::read_dir(source.join("tools")).unwrap() {
            let file = file.unwrap();
            std::fs::copy(file.path(), package.join("tools").join(file.file_name())).unwrap();
        }
        Self { root, package }
    }
    fn scenario(&self, name: &str) {
        let path = self.package.join("case.eal");
        let source = std::fs::read_to_string(&path).unwrap();
        assert!(source.contains("\"scenario\":\"positive\""));
        std::fs::write(
            path,
            source.replace(
                "\"scenario\":\"positive\"",
                &format!("\"scenario\":\"{name}\""),
            ),
        )
        .unwrap();
    }
    fn run(&self, claim: &str) -> Value {
        self.run_at(claim, None)
    }
    fn run_at(&self, claim: &str, now: Option<&str>) -> Value {
        let mut command = Command::new(env!("CARGO_BIN_EXE_eal-host"));
        command
            .current_dir(&self.root)
            .env("PREPARATION_TEST_CREDENTIAL", "inherited-secret-sentinel")
            .arg("run")
            .arg(self.package.join("case.eal"))
            .arg("--tools")
            .arg(self.package.join("tools.toml"))
            .arg("--workspace")
            .arg(self.root.join("run"))
            .arg("--claim")
            .arg(claim);
        if let Some(now) = now {
            command.args(["--now", now]);
        }
        let output = command.output().unwrap();
        let response: Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
            panic!(
                "{e}: stdout={} stderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
        });
        assert!(
            output.status.success(),
            "{response}; stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(response["is_error"], false, "{response}");
        response["result"].clone()
    }
}
impl Drop for Package {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn immutable_finite_model_runs_through_host_without_acquired_observations() {
    let package = Package::copy("finite-inventory");
    std::fs::write(
        package.package.join("case.eal"),
        include_str!("../../../examples/finite-probability/case.eal"),
    )
    .unwrap();
    std::fs::write(package.package.join("tools.toml"), "[tools]\n").unwrap();
    for now in ["2026-10-28T12:00:00Z", "2026-10-30T12:00:00Z"] {
        let result = package.run_at("consequence", Some(now));
        assert_eq!(
            result["assessment"]["claims"]["consequence"]["status"], "supported",
            "{result}"
        );
        assert_eq!(result["collection"]["records"], serde_json::json!({}));
        assert_eq!(result["assessment"]["evidence"], serde_json::json!({}));
        let computation = &result["assessment"]["arguments"]["proof"]["reasoning_result"];
        assert_eq!(computation["input_origin"], "authored_program");
        assert_eq!(
            computation["details"]["posterior_exact"]["numerator"],
            "152"
        );
        assert_eq!(
            computation["details"]["posterior_exact"]["denominator"],
            "201"
        );
        assert!(computation.get("evidence_id").is_none());
    }
}

#[test]
fn moved_inventory_runs_real_mcp_source_and_command_without_registration() {
    let package = Package::copy("finite-inventory");
    let result = package.run("inventory_verified");
    assert_eq!(result["schema"], "EAL/execution/1");
    assert_eq!(
        result["assessment"]["claims"]["inventory_verified"]["status"], "supported",
        "{result}"
    );
    let records = result["collection"]["records"].as_object().unwrap();
    assert_eq!(records.len(), 4, "one snapshot, one audit, two pure checks");
    assert_eq!(
        records["snapshot"]["value"]["items"],
        serde_json::json!(["Alpha", "Beta", "Gamma"])
    );
    assert_eq!(records["audit"]["value"]["count"], 3);
    assert_eq!(records["audit"]["value"]["distinct"], true);
    let computation = &result["assessment"]["arguments"]["inventory_argument"]["reasoning_result"];
    assert_eq!(computation["details"]["entailed"], true);
    assert_eq!(computation["input_origin"], "authored_program");
    assert!(computation.get("evidence_id").is_none());
    assert!(!result.to_string().contains("engarg_support_graph"));
    assert!(
        !package
            .package
            .join("argument.execution-plan.json")
            .exists()
    );
    assert!(!package.package.join("registration.json").exists());
}

#[test]
fn inventory_negatives_errors_and_original_age_do_not_become_support() {
    for scenario in [
        "duplicate",
        "incomplete",
        "manifest-mismatch",
        "stale",
        "error",
    ] {
        let package = Package::copy("finite-inventory");
        package.scenario(scenario);
        let result = package.run("inventory_verified");
        assert_ne!(
            result["assessment"]["claims"]["inventory_verified"]["status"], "supported",
            "{scenario}: {result}"
        );
        if scenario == "duplicate" {
            assert_eq!(result["collection"]["records"]["audit"]["status"], "ok");
            assert_eq!(
                result["collection"]["records"]["audit"]["value"]["distinct"],
                false
            );
        }
        if scenario == "stale" {
            assert_eq!(
                result["collection"]["records"]["snapshot"]["collected_at"],
                "2000-01-01T00:00:00Z"
            );
        }
        if scenario == "error" {
            assert_ne!(result["collection"]["records"]["snapshot"]["status"], "ok");
        }
    }
}

#[test]
fn altered_audit_output_fails_independent_eal_check() {
    let package = Package::copy("finite-inventory");
    let path = package.package.join("tools/audit.py");
    let code = std::fs::read_to_string(&path).unwrap();
    std::fs::write(path, code.replace("\"count\": len(items)", "\"count\": 4")).unwrap();
    let result = package.run("inventory_verified");
    assert_eq!(
        result["collection"]["records"]["audit"]["value"]["count"],
        4
    );
    assert_ne!(
        result["assessment"]["claims"]["inventory_verified"]["status"],
        "supported"
    );
}

#[test]
fn seven_modes_share_one_acquired_observation_and_compute_expected_values() {
    let package = Package::copy("reasoning-modes");
    let result = package.run("all_modes");
    for mode in [
        "deductive",
        "inductive",
        "abductive",
        "causal",
        "counterfactual",
        "analogical",
        "practical",
    ] {
        assert_eq!(
            result["assessment"]["claims"][format!("{mode}_claim")]["status"],
            "supported",
            "{mode}: {result}"
        );
    }
    assert_eq!(
        result["assessment"]["claims"]["all_modes"]["status"],
        "supported"
    );
    let arguments = &result["assessment"]["arguments"];
    assert_eq!(
        arguments["causal_argument"]["reasoning_result"]["details"]["estimate"],
        8.0
    );
    assert_eq!(
        arguments["counterfactual_argument"]["reasoning_result"]["details"]["difference"],
        6
    );
    let posterior =
        arguments["abductive_argument"]["reasoning_result"]["details"]["best_posterior"]
            .as_f64()
            .unwrap();
    assert!((posterior - 2.0 / 3.0).abs() < 1e-12);
    let lower = arguments["inductive_argument"]["reasoning_result"]["details"]["lower"]
        .as_f64()
        .unwrap();
    assert!((lower - 0.8256343384950865).abs() < 1e-8);
    let records = result["collection"]["records"].as_object().unwrap();
    assert_eq!(
        records
            .values()
            .filter(|r| r["tool"] == "instrument")
            .count(),
        1
    );
}

#[test]
fn native_applicability_challenge_and_scoped_review_failures_propagate() {
    for scenario in [
        "negative",
        "challenge",
        "wrong-review-scope",
        "wrong-reviewer",
        "old-nested-review",
        "malformed-review-reasons",
        "malformed-review-time",
        "inconclusive",
        "query-conflict",
        "stale",
        "error",
    ] {
        let package = Package::copy("reasoning-modes");
        package.scenario(scenario);
        let result = package.run("all_modes");
        assert_ne!(
            result["assessment"]["claims"]["all_modes"]["status"], "supported",
            "{scenario}: {result}"
        );
        if scenario == "challenge" {
            assert_ne!(
                result["assessment"]["claims"]["counterfactual_claim"]["status"],
                "supported"
            );
            assert_eq!(
                result["assessment"]["evidence"]["fault"]["status"],
                "available"
            );
        }
    }
}

#[test]
fn malformed_authored_formula_fails_before_source_execution() {
    let package = Package::copy("reasoning-modes");
    let source = package.package.join("case.eal");
    let text = std::fs::read_to_string(&source).unwrap();
    let replacement = text.replace(
        "\"premises\":[\"sample_good\",\"effect_positive\"]",
        "\"premises\":[true]",
    );
    assert_ne!(text, replacement);
    std::fs::write(&source, replacement).unwrap();
    let tool = package.package.join("tools/observations.py");
    let text = std::fs::read_to_string(&tool).unwrap();
    std::fs::write(
        &tool,
        format!("from pathlib import Path\nPath('source-was-called').write_text('called')\n{text}"),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_eal-host"))
        .arg("run")
        .arg(&source)
        .arg("--tools")
        .arg(package.package.join("tools.toml"))
        .arg("--workspace")
        .arg(package.root.join("run"))
        .arg("--claim")
        .arg("all_modes")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["is_error"], true);
    assert!(!package.package.join("source-was-called").exists());
    assert!(!package.root.join("run/source-was-called").exists());
}

#[test]
fn prepared_tools_move_and_execute_without_the_preparation_receipt() {
    let mut package = Package::copy("finite-inventory");
    let server = package.package.join("tools/snapshot.py");
    let source = std::fs::read_to_string(&server).unwrap();
    std::fs::write(&server, source.replace("    elif method == \"tools/call\":",
        "    elif method == \"tools/call\":\n        open('invocations','a').write('called\\n')")).unwrap();
    let manifest = package.package.join("tools.toml");
    let original = std::fs::read_to_string(&manifest).unwrap().replacen(
        "inherit_env = [\"PATH\"]",
        "inherit_env = [\"PATH\", \"PREPARATION_TEST_CREDENTIAL\"]",
        1,
    );
    let original = format!(
        "{original}\n[tools.snapshot_source.env]\nPRIVATE_LITERAL = \"operator-literal-sentinel\"\n"
    );
    std::fs::write(&manifest, original).unwrap();
    let prepared = package.package.join("prepared.tools.toml");
    let output = Command::new(env!("CARGO_BIN_EXE_eal-host"))
        .env("PREPARATION_TEST_CREDENTIAL", "inherited-secret-sentinel")
        .arg("prepare")
        .arg(package.package.join("case.eal"))
        .arg("--tools")
        .arg(package.package.join("tools.toml"))
        .arg("--workspace")
        .arg(package.root.join("prepare-run"))
        .arg("--claim")
        .arg("inventory_verified")
        .arg("--output-tools")
        .arg(&prepared)
        .output()
        .unwrap();
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(output.status.success(), "{response}");
    assert_eq!(response["result"]["operation_calls"], 0);
    assert!(!package.package.join("invocations").exists());
    assert!(!response.to_string().contains("inherited-secret-sentinel"));
    assert!(!response.to_string().contains("operator-literal-sentinel"));
    let generated = std::fs::read_to_string(&prepared).unwrap();
    assert!(!generated.contains("inherited-secret-sentinel"));
    assert!(generated.contains("PREPARATION_TEST_CREDENTIAL"));
    assert!(generated.contains("operator-literal-sentinel"));
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(
        std::fs::metadata(&prepared).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let pins: toml::Value = toml::from_str(&generated).unwrap();
    assert!(
        pins["tools"]["snapshot_source"]["direct"]["input_schema_sha256"]
            .as_str()
            .is_some()
    );
    assert_eq!(
        pins["tools"]["snapshot_source"]["direct"]["server_identity"]["name"].as_str(),
        Some("synthetic-snapshot")
    );
    std::fs::rename(prepared, package.package.join("tools.toml")).unwrap();
    std::fs::remove_dir_all(package.root.join("prepare-run")).unwrap();
    let moved = package.root.join("prepared-package");
    std::fs::rename(&package.package, &moved).unwrap();
    package.package = moved;
    let result = package.run("inventory_verified");
    assert_eq!(
        result["assessment"]["claims"]["inventory_verified"]["status"],
        "supported"
    );
    assert_eq!(
        result["collection"]["records"]["snapshot"]["mcp_interface"]["server"]["name"],
        "synthetic-snapshot"
    );
    assert_eq!(
        std::fs::read_to_string(package.package.join("invocations")).unwrap(),
        "called\n"
    );
}
