//! Exercise the shared lifecycle deadline and cleanup with the real process supervisor.
#![cfg(unix)]

use eal_process::CancellationToken;
use eal_tool_runtime::{
    SourceFile, ToolchainProfile,
    runner::{DockerRunner, ExecutionMode, FixtureRunner},
};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    time::{Duration, Instant},
};

struct Daemon {
    root: tempfile::TempDir,
    runner: DockerRunner,
}

impl Daemon {
    fn new(
        timeout_seconds: u64,
        create_delay: f64,
        upload_delay: f64,
        foreign_owner: bool,
    ) -> Self {
        let root = tempfile::tempdir().unwrap();
        let docker = root.path().join("docker");
        fs::write(
            &docker,
            include_str!("fixtures/snapshot_lifecycle_docker.py"),
        )
        .unwrap();
        fs::set_permissions(&docker, fs::Permissions::from_mode(0o755)).unwrap();
        fs::write(
            root.path().join("config.json"),
            serde_json::to_vec(&json!({
                "create_delay": create_delay,
                "upload_delay": upload_delay,
                "foreign_owner": foreign_owner,
            }))
            .unwrap(),
        )
        .unwrap();
        let profile = ToolchainProfile {
            schema: eal_tool_runtime::profile::TOOLCHAIN_SCHEMA.into(),
            language: "python".into(),
            docker: docker.to_string_lossy().into_owned(),
            image: "synthetic-lifecycle:1".into(),
            build_argv: vec![],
            run_argv: vec!["python3".into(), "/src/main.py".into()],
            inherit_env: vec![],
            runtime_network: "none".into(),
            timeout_seconds,
            memory_mb: 64,
            output_bytes: 65_536,
        };
        let binary = PathBuf::from(env!("CARGO_BIN_EXE_eal-tool-run"));
        let supervisor = eal_process::companion_in(binary.parent().unwrap());
        assert!(
            supervisor.is_file(),
            "Build eal-process binaries before focused tool-runtime tests"
        );
        let runner = DockerRunner::new(profile, supervisor).unwrap();
        Self { root, runner }
    }

    fn execute(
        &self,
        cancellation: &CancellationToken,
    ) -> Result<eal_tool_runtime::runner::RunOutcome, eal_tool_runtime::Error> {
        self.runner.execute(
            self.root.path(),
            &[SourceFile {
                path: "main.py".into(),
                content: "print('unused by synthetic daemon')\n".into(),
            }],
            &[],
            b"bounded-lifecycle-test\n",
            ExecutionMode::Fixture,
            cancellation,
        )
    }

    fn events(&self) -> Vec<Value> {
        fs::read_to_string(self.root.path().join("events.jsonl"))
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    fn assert_cleaned(&self, worker_started: bool) {
        let events = self.events();
        let operations: Vec<_> = events
            .iter()
            .map(|event| event["operation"].as_str().unwrap())
            .collect();
        let created = events
            .iter()
            .find(|event| event["operation"] == "create")
            .unwrap();
        let removed = events
            .iter()
            .find(|event| event["operation"] == "remove snapshot")
            .unwrap();
        assert_eq!(removed["name"], created["name"]);
        assert_eq!(removed["volumes"], true);
        let ownership = operations
            .iter()
            .position(|operation| *operation == "ownership")
            .unwrap();
        let removal = operations
            .iter()
            .position(|operation| *operation == "remove snapshot")
            .unwrap();
        assert!(ownership < removal, "{events:?}");
        assert_eq!(operations.contains(&"worker"), worker_started);
        assert_eq!(operations.contains(&"remove worker"), worker_started);
        if worker_started {
            let worker_removal = operations
                .iter()
                .position(|operation| *operation == "remove worker")
                .unwrap();
            assert!(worker_removal < removal, "{events:?}");
        }
        assert!(!self.root.path().join("snapshot.json").exists());
    }

    fn assert_process_reaped(&self, operation: &str) {
        let event = self
            .events()
            .into_iter()
            .find(|event| event["operation"] == operation)
            .unwrap();
        let pid = rustix::process::Pid::from_raw(event["pid"].as_i64().unwrap() as i32).unwrap();
        assert_eq!(
            rustix::process::test_kill_process(pid),
            Err(rustix::io::Errno::SRCH)
        );
    }
}

#[test]
fn setup_longer_than_fifteen_seconds_can_complete_within_the_profile_budget() {
    let daemon = Daemon::new(20, 16.0, 0.0, false);
    let started = Instant::now();
    let outcome = daemon.execute(&CancellationToken::default()).unwrap();
    assert!(started.elapsed() >= Duration::from_secs(16));
    assert_eq!(outcome.returncode, 0);
    assert_eq!(outcome.execution_error, None);
    assert_eq!(outcome.stdout, b"worker reached\n");
    daemon.assert_cleaned(true);
}

#[test]
fn setup_stages_share_the_execution_deadline_and_clean_up_after_exhaustion() {
    let daemon = Daemon::new(1, 1.75, 1.75, false);
    let started = Instant::now();
    let error = daemon.execute(&CancellationToken::default()).unwrap_err();
    let message = error.to_string();
    assert!(message.contains("source upload failed"), "{message}");
    assert!(message.contains("timeout"), "{message}");
    assert!(started.elapsed() < Duration::from_secs(6));
    assert!(
        !daemon
            .events()
            .iter()
            .any(|event| event["operation"] == "fixture upload")
    );
    daemon.assert_cleaned(false);
    daemon.assert_process_reaped("source upload");
}

#[test]
fn cancellation_during_setup_reaps_the_command_and_cleans_the_owned_snapshot() {
    let daemon = Daemon::new(20, 0.0, 30.0, false);
    let cancellation = CancellationToken::default();
    std::thread::scope(|scope| {
        let running = scope.spawn(|| daemon.execute(&cancellation));
        let deadline = Instant::now() + Duration::from_secs(5);
        while !daemon
            .events()
            .iter()
            .any(|event| event["operation"] == "source upload")
        {
            if Instant::now() >= deadline {
                cancellation.cancel();
                panic!("source upload did not start");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        cancellation.cancel();
        let message = running.join().unwrap().unwrap_err().to_string();
        assert!(message.contains("source upload failed"), "{message}");
        assert!(message.contains("cancelled"), "{message}");
    });
    assert!(
        !daemon
            .events()
            .iter()
            .any(|event| event["operation"] == "fixture upload")
    );
    daemon.assert_cleaned(false);
    daemon.assert_process_reaped("source upload");
}

#[test]
fn exhausted_setup_never_removes_a_snapshot_with_foreign_ownership() {
    let daemon = Daemon::new(1, 30.0, 0.0, true);
    let message = daemon
        .execute(&CancellationToken::default())
        .unwrap_err()
        .to_string();
    assert!(message.contains("snapshot creation failed"), "{message}");
    assert!(message.contains("ownership was not confirmed"), "{message}");
    assert!(
        !daemon
            .events()
            .iter()
            .any(|event| event["operation"] == "remove snapshot")
    );
    assert!(daemon.root.path().join("snapshot.json").exists());
    daemon.assert_process_reaped("create");
}
