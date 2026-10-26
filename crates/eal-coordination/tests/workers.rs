#![cfg(unix)]
//! Actual supervised commands with exact retained bytes; no model execution is inferred.
use eal_coordination::workers::{CommandWorker, CommandWorkerConfig, WorkerError};
use eal_runtime::{lease::WorkspaceAcquisition, process::Supervisor};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    time::{Duration, Instant},
};
struct Workspace {
    root: PathBuf,
    work: PathBuf,
    run: PathBuf,
}
impl Workspace {
    fn new() -> Self {
        let root =
            std::env::temp_dir().join(format!("eal-coordination-worker-{}", uuid::Uuid::new_v4()));
        let work = root.join("work");
        let run = root.join("run");
        std::fs::create_dir_all(&work).unwrap();
        std::fs::create_dir_all(&run).unwrap();
        Self { root, work, run }
    }
    fn worker(
        &self,
        argv: Vec<String>,
        mode: &str,
        cap: usize,
        environment: BTreeMap<String, String>,
    ) -> CommandWorker {
        CommandWorker::new(
            Supervisor::new(supervisor()),
            CommandWorkerConfig {
                argv,
                execution_mode: mode.into(),
                timeout_seconds: 2,
                max_output_bytes: cap,
                environment,
                working_directory: self.work.clone(),
                lease_directory: self.run.clone(),
            },
        )
        .unwrap()
    }
    fn shell(&self, body: &str) -> CommandWorker {
        self.worker(
            vec!["/bin/sh".into(), "-c".into(), body.into()],
            "software_test",
            8192,
            BTreeMap::new(),
        )
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
fn supervisor() -> PathBuf {
    let directory = PathBuf::from(env!("CARGO_BIN_EXE_eal-coordinate"))
        .parent()
        .unwrap()
        .to_path_buf();
    let path = directory.join("eal-command-supervisor");
    assert!(
        path.is_file(),
        "Build native workspace companions before integration tests"
    );
    path
}
fn failure(
    result: Result<eal_coordination::workers::WorkerResult, WorkerError>,
) -> eal_coordination::workers::WorkerInterrupted {
    match result.unwrap_err() {
        WorkerError::Interrupted(error) => *error,
        other => panic!("Expected retained interruption: {other}"),
    }
}
#[test]
fn fresh_processes_keep_exact_context_and_use_a_distinct_coordination_lease() {
    let fixture = Workspace::new();
    let workspace_lock = WorkspaceAcquisition::new(&fixture.work).unwrap();
    let _held = workspace_lock.hold(Duration::from_secs(1)).unwrap();
    let cat = fixture.worker(
        vec!["/bin/cat".into()],
        "software_test",
        8192,
        BTreeMap::new(),
    );
    let context = json!({"private":"alpha","literal":"$(touch forbidden)","integer":123456789});
    let first = cat
        .execute(&context, "call-1", Duration::from_secs(1), None)
        .unwrap();
    assert_eq!(first.response, context);
    assert_eq!(
        first.raw_response,
        b"{\"integer\": 123456789, \"literal\": \"$(touch forbidden)\", \"private\": \"alpha\"}\n"
    );
    assert!(!fixture.work.join("forbidden").exists());
    assert_eq!(first.evidence["returncode"], 0);
    assert_eq!(first.evidence["model"], Value::Null);
    assert!(first.execution_id.starts_with("command:call-1:"));
    assert!(
        first.evidence["supervisor_pid"]
            .as_u64()
            .is_some_and(|pid| pid != u64::from(std::process::id()))
    );
    let pid = fixture.shell("printf '{\"pid\":%s}' \"$$\"");
    let a = pid
        .execute(&json!({}), "pid-a", Duration::from_secs(1), None)
        .unwrap();
    let b = pid
        .execute(&json!({}), "pid-b", Duration::from_secs(1), None)
        .unwrap();
    assert_ne!(a.response["pid"], b.response["pid"]);
}
#[test]
fn strict_response_failures_retain_first_exact_stdout_and_actual_exit() {
    let fixture = Workspace::new();
    for raw in [
        "{\"answer\":1,\"answer\":2}",
        "{\"answer\":NaN}",
        "{\"answer\":1e9999}",
        "{}\n{}",
        "[]",
        "not JSON",
        "",
    ] {
        let adapter = fixture.worker(
            vec!["/bin/sh".into(), "-c".into(), "printf '%s' \"$RAW\"".into()],
            "software_test",
            8192,
            BTreeMap::from([("RAW".into(), raw.into())]),
        );
        let error = failure(adapter.execute(&json!({}), "invalid", Duration::from_secs(1), None));
        assert_eq!(error.reason, "invalid_response");
        assert_eq!(error.raw_response, raw.as_bytes());
        assert_eq!(error.evidence["returncode"], 0);
    }
    let adapter = fixture.worker(
        vec![
            "/bin/sh".into(),
            "-c".into(),
            "printf '{\"partial\":true}'; printf 'failure %s\\n' \"$API_KEY\" >&2; exit 17".into(),
        ],
        "software_test",
        8192,
        BTreeMap::from([("API_KEY".into(), "synthetic-worker-credential".into())]),
    );
    let error = failure(adapter.execute(&json!({}), "exit-17", Duration::from_secs(1), None));
    assert_eq!(error.reason, "nonzero_exit");
    assert_eq!(error.evidence["returncode"], 17);
    assert_eq!(error.raw_response, b"{\"partial\":true}");
    assert_eq!(error.evidence["stderr"], "failure [redacted]\n");
    assert!(!error.to_string().contains("synthetic-worker-credential"));
}
#[test]
fn live_modes_require_external_receipts_without_inventing_model_or_usage() {
    let fixture = Workspace::new();
    for mode in ["provider", "actual_host_agent"] {
        let receipt = json!({"response":{"answer":7},"execution":{"execution_id":"external-42","model":null,"usage":null}});
        let adapter = fixture.worker(
            vec![
                "/bin/sh".into(),
                "-c".into(),
                "printf '%s' \"$RAW\"; printf 'private-provider-diagnostic' >&2".into(),
            ],
            mode,
            8192,
            BTreeMap::from([("RAW".into(), receipt.to_string())]),
        );
        let result = adapter
            .execute(&json!({}), "live", Duration::from_secs(1), None)
            .unwrap();
        assert_eq!(result.execution_id, "external-42");
        assert_eq!(result.response, json!({"answer":7}));
        assert_eq!(
            result.evidence["receipt_verification"],
            "operator_command_assertion"
        );
        assert_eq!(result.evidence["model"], Value::Null);
        assert_eq!(result.evidence["usage"], Value::Null);
        assert!(result.evidence.get("stderr").is_none());
        assert!(
            !result
                .evidence
                .to_string()
                .contains("private-provider-diagnostic")
        );
        for execution in [
            json!({"execution_id":"live","model":null,"usage":null}),
            json!({"execution_id":"command:live:1","model":null,"usage":null}),
            json!({"execution_id":"external","model":null,"usage":{"input_tokens":true}}),
            json!({"execution_id":"external","model":null,"usage":{"cost":0.1}}),
        ] {
            let raw = json!({"response":{},"execution":execution}).to_string();
            let adapter = fixture.worker(
                vec!["/bin/sh".into(), "-c".into(), "printf '%s' \"$RAW\"".into()],
                mode,
                8192,
                BTreeMap::from([("RAW".into(), raw.clone())]),
            );
            let error = failure(adapter.execute(&json!({}), "live", Duration::from_secs(1), None));
            assert_eq!(error.reason, "invalid_response");
            assert_eq!(error.raw_response, raw.as_bytes());
        }
    }
}
#[test]
fn combined_output_deadline_and_cancellation_remain_distinct_without_replay() {
    let fixture = Workspace::new();
    let adapter = fixture.worker(
        vec![
            "/bin/sh".into(),
            "-c".into(),
            "printf 'xxxxxxxxxxxxxxxx'; printf 'yyyyyyyyyyyyyyyy' >&2".into(),
        ],
        "software_test",
        20,
        BTreeMap::new(),
    );
    let error = failure(adapter.execute(&json!({}), "cap", Duration::from_secs(1), None));
    assert_eq!(error.reason, "output_limit");
    assert_eq!(
        error.evidence["stdout_bytes"].as_u64().unwrap()
            + error.evidence["stderr_bytes"].as_u64().unwrap(),
        20
    );
    assert_eq!(error.evidence["output_truncated"], true);
    let adapter = fixture.worker(
        vec!["/bin/sleep".into(), "30".into()],
        "software_test",
        8192,
        BTreeMap::new(),
    );
    let started = Instant::now();
    let error = failure(adapter.execute(
        &json!({"large":"x".repeat(1_000_000)}),
        "timeout",
        Duration::from_millis(100),
        None,
    ));
    assert_eq!(error.reason, "timeout");
    assert!(started.elapsed() < Duration::from_secs(3));
    let adapter = fixture.shell("printf x >> effect; sleep 30");
    let marker = fixture.work.join("effect");
    let stop = || Ok(marker.exists());
    let error =
        failure(adapter.execute(&json!({}), "cancelled", Duration::from_secs(2), Some(&stop)));
    assert_eq!(error.reason, "cancelled");
    assert_eq!(std::fs::read(&marker).unwrap(), b"x");
    let stopped = || Ok(true);
    let error = failure(adapter.execute(
        &json!({}),
        "preflight",
        Duration::from_secs(1),
        Some(&stopped),
    ));
    assert_eq!(error.reason, "cancelled");
    assert_eq!(error.evidence["supervisor_pid"], Value::Null);
    assert_eq!(std::fs::read(&marker).unwrap(), b"x");
    let broken = || Err("private-cancellation-controller".into());
    let error = failure(adapter.execute(
        &json!({}),
        "callback",
        Duration::from_secs(1),
        Some(&broken),
    ));
    assert_eq!(error.reason, "supervisor_failure");
    assert!(
        !error
            .to_string()
            .contains("private-cancellation-controller")
    );
}

#[test]
fn opaque_environment_values_cannot_reappear_inside_a_redaction_marker() {
    let fixture = Workspace::new();
    let adapter = fixture.worker(
        vec![
            "/bin/sh".into(),
            "-c".into(),
            "printf 'redacted second-secret' >&2; printf '{}'".into(),
        ],
        "software_test",
        8192,
        BTreeMap::from([
            ("A".into(), "redacted".into()),
            ("B".into(), "second-secret".into()),
        ]),
    );
    let result = adapter
        .execute(&json!({}), "marker", Duration::from_secs(1), None)
        .unwrap();
    let diagnostic = result.evidence["stderr"].as_str().unwrap();
    assert!(!diagnostic.contains("redacted"));
    assert!(!diagnostic.contains("second-secret"));
}

#[test]
fn prelaunch_cancellation_does_not_wait_for_an_inherited_run_lease() {
    let fixture = Workspace::new();
    let adapter = fixture.shell("printf '{}' ");
    let acquisition = WorkspaceAcquisition::new(&fixture.run).unwrap();
    let _held = acquisition.hold(Duration::from_secs(1)).unwrap();
    let stop = || Ok(true);
    let started = Instant::now();
    let error = failure(adapter.execute(
        &json!({}),
        "already-cancelled",
        Duration::from_secs(1),
        Some(&stop),
    ));
    assert_eq!(error.reason, "cancelled");
    assert_eq!(error.evidence["supervisor_pid"], Value::Null);
    assert!(started.elapsed() < Duration::from_millis(250));
}

#[test]
fn positive_budget_expiring_during_admission_retains_a_timeout_without_launching() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let fixture = Workspace::new();
    let adapter = fixture.shell("printf x >> executed; printf '{}'");
    let checks = AtomicUsize::new(0);
    let slow_admission = || {
        checks.fetch_add(1, Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(40));
        Ok(false)
    };
    let error = failure(adapter.execute(
        &json!({}),
        "admission-timeout",
        Duration::from_millis(10),
        Some(&slow_admission),
    ));
    assert_eq!(error.reason, "timeout");
    assert_eq!(error.execution_id, "command:admission-timeout");
    assert!(error.raw_response.is_empty());
    assert_eq!(error.evidence["execution_error"], "timeout");
    assert_eq!(error.evidence["supervisor_pid"], Value::Null);
    assert_eq!(error.evidence["returncode"], Value::Null);
    assert_eq!(checks.load(Ordering::SeqCst), 1);
    assert!(!fixture.work.join("executed").exists());

    let invalid = adapter.execute(
        &json!({}),
        "invalid-zero-budget",
        Duration::ZERO,
        Some(&slow_admission),
    );
    assert!(
        matches!(invalid, Err(WorkerError::Invalid(message)) if message == "remaining_seconds must be positive")
    );
    assert_eq!(checks.load(Ordering::SeqCst), 1);
    assert!(!fixture.work.join("executed").exists());
}
