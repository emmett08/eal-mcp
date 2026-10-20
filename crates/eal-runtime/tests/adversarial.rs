//! Independent adversarial checks of real process and persistent-state boundaries.
#![cfg(unix)]

use eal_core::{ExecutionLimits, canonical, composition, evaluation, methods};
use eal_runtime::{
    acquisition::{PinnedFile, ToolBinding, ToolRegistry},
    collection::EvidenceRuntime,
    lease::WorkspaceAcquisition,
    process::{CommandRequest, ExecutionError, StartupHandshake, Supervisor},
    reuse::ObservationRebinder,
    scheduler::CollectionScheduler,
    service::{ReasoningService, ServiceConfig},
    store::{ObservationFilter, RunStore},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    os::{
        fd::AsFd,
        unix::fs::{OpenOptionsExt, PermissionsExt, symlink},
    },
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "eal-adversarial-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn write(&self, name: &str, contents: impl AsRef<[u8]>) -> PathBuf {
        let path = self.0.join(name);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
            .unwrap();
        file.write_all(contents.as_ref()).unwrap();
        path
    }
    fn database(&self) -> PathBuf {
        self.0.join("runs.sqlite3")
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn supervisor() -> Supervisor {
    Supervisor::new(PathBuf::from(env!("CARGO_BIN_EXE_eal-command-supervisor")))
}
fn request(
    workspace: &Workspace,
    script: &str,
    output: usize,
    timeout: Duration,
) -> CommandRequest {
    CommandRequest {
        argv: vec!["/bin/sh".into(), "-c".into(), script.into()],
        environment: BTreeMap::new(),
        cwd: workspace.0.clone(),
        input: Vec::new(),
        timeout,
        max_output_bytes: output,
    }
}

#[test]
fn dropping_the_original_lease_cannot_unlock_a_live_descriptor_clone() {
    let workspace = Workspace::new();
    let coordinator = WorkspaceAcquisition::new(&workspace.0).unwrap();
    let lease = coordinator.hold(Duration::ZERO).unwrap();
    let inherited_copy = lease.as_fd().try_clone_to_owned().unwrap();
    drop(lease);
    assert!(
        coordinator.hold(Duration::ZERO).is_err(),
        "Drop must close, never LOCK_UN, a shared open-file description"
    );
    drop(inherited_copy);
    // Another parallel test can be between fork and exec with a transient
    // CLOEXEC copy. It must disappear promptly; no live copy may be unlocked.
    coordinator.hold(Duration::from_secs(1)).unwrap();
}

#[test]
fn isolated_final_descriptor_close_releases_without_any_grace_period() {
    let workspace = Workspace::new();
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "runtime_process_fixture", "--nocapture"])
        .env("EAL_ADVERSARIAL_FIXTURE", "lease-clone")
        .env("EAL_ADVERSARIAL_WORKSPACE", &workspace.0)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn collector_exit_and_signal_status_are_distinct_from_supervisor_success() {
    let workspace = Workspace::new();
    let result = supervisor()
        .run(
            request(
                &workspace,
                "printf 'out'; printf 'err' >&2; exit 23",
                64,
                Duration::from_secs(2),
            ),
            None,
        )
        .unwrap();
    assert_eq!(result.returncode, 23);
    assert_eq!(result.stdout, b"out");
    assert_eq!(result.stderr, b"err");
    assert!(!result.timed_out);
    assert!(result.execution_error.is_none());
    let signalled = supervisor()
        .run(
            request(&workspace, "kill -TERM $$", 64, Duration::from_secs(2)),
            None,
        )
        .unwrap();
    assert_eq!(signalled.returncode, -15);
    assert!(!signalled.timed_out);
}

#[test]
fn one_combined_output_allowance_distinguishes_exact_fit_from_excess() {
    let workspace = Workspace::new();
    let script = "printf 'abc'; printf 'de' >&2";
    let exact = supervisor()
        .run(request(&workspace, script, 5, Duration::from_secs(2)), None)
        .unwrap();
    assert_eq!(exact.stdout, b"abc");
    assert_eq!(exact.stderr, b"de");
    assert_eq!(exact.returncode, 0);
    assert!(exact.execution_error.is_none());
    let excess = supervisor()
        .run(request(&workspace, script, 4, Duration::from_secs(2)), None)
        .unwrap();
    assert_eq!(excess.stdout.len() + excess.stderr.len(), 4);
    assert!(excess.execution_error.is_some());
    assert!(
        !excess.timed_out,
        "Output exhaustion must retain priority over cleanup timing"
    );
}

#[test]
fn an_unconsumed_large_stdin_obeys_the_original_absolute_deadline() {
    let workspace = Workspace::new();
    let mut command = request(
        &workspace,
        "exec /bin/sleep 5",
        64,
        Duration::from_millis(200),
    );
    command.input = vec![b'x'; 1024 * 1024];
    let started = Instant::now();
    let result = supervisor().run(command, None).unwrap();
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "Blocked stdin exceeded the bounded cleanup allowance"
    );
    assert!(result.timed_out);
    assert!(result.execution_error.is_some());
    assert_eq!(result.returncode, -9);
}

#[test]
fn actual_companion_reports_timeout_when_large_configuration_arrives_after_deadline() {
    let workspace = Workspace::new();
    let mut command = request(
        &workspace,
        "printf launched > launched",
        64,
        Duration::from_nanos(1),
    );
    command
        .environment
        .insert("LARGE_CONFIGURATION".into(), "x".repeat(1024 * 1024));
    let started = Instant::now();
    let outcome = supervisor().run(command, None).unwrap();
    assert!(started.elapsed() < Duration::from_secs(2));
    assert_eq!(outcome.execution_error, Some(ExecutionError::Timeout));
    assert!(outcome.timed_out);
    assert!(!outcome.startup_completed);
    assert!(!workspace.0.join("launched").exists());
}

#[test]
fn a_supervisor_that_never_reads_configuration_cannot_block_its_caller() {
    let workspace = Workspace::new();
    let helper = workspace.write(
        "nonreading-supervisor",
        b"#!/bin/sh\necho $$ > companion.pid\nexec /bin/sleep 5\n",
    );
    fs::set_permissions(&helper, fs::Permissions::from_mode(0o700)).unwrap();
    for framed in [false, true] {
        let mut command = request(
            &workspace,
            "exit 0",
            64,
            if framed {
                Duration::from_secs(5)
            } else {
                Duration::from_millis(200)
            },
        );
        command
            .environment
            .insert("LARGE_CONFIGURATION".into(), "x".repeat(1024 * 1024));
        let started = Instant::now();
        let runner = Supervisor::new(helper.clone());
        let result = if framed {
            runner.run_framed(
                command,
                None,
                StartupHandshake {
                    expected_first_stdout_line: b"ready\n".to_vec(),
                    startup_timeout: Duration::from_millis(200),
                },
            )
        } else {
            runner.run(command, None)
        };
        let pid = fs::read_to_string(workspace.0.join("companion.pid"))
            .unwrap()
            .trim()
            .parse::<i32>()
            .unwrap();
        let _companion = KillOnDrop(rustix::process::Pid::from_raw(pid).unwrap());
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "Configuration transfer must be inside the initial deadline (framed={framed})"
        );
        assert!(
            result.is_err()
                || result.is_ok_and(|outcome| outcome.timed_out && !outcome.startup_completed)
        );
    }
}

struct KillOnDrop(rustix::process::Pid);
impl Drop for KillOnDrop {
    fn drop(&mut self) {
        let _ = rustix::process::kill_process(self.0, rustix::process::Signal::KILL);
    }
}

struct ChildFixture {
    child: Child,
    group: Option<rustix::process::Pid>,
}
impl Drop for ChildFixture {
    fn drop(&mut self) {
        let _ = self.child.kill();
        if let Some(group) = self.group {
            let _ = rustix::process::kill_process_group(group, rustix::process::Signal::KILL);
        }
        let _ = self.child.wait();
    }
}
fn wait_for_pid(path: &Path, child: &mut Child) -> rustix::process::Pid {
    let deadline = Instant::now() + Duration::from_secs(4);
    loop {
        if let Ok(contents) = fs::read_to_string(path)
            && let Ok(pid) = contents.trim().parse::<i32>()
        {
            return rustix::process::Pid::from_raw(pid).unwrap();
        }
        assert!(
            child.try_wait().unwrap().is_none(),
            "Fixture owner exited before recording its collector"
        );
        assert!(Instant::now() < deadline, "Fixture collector did not start");
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn caller_death_reaps_the_collector_group_before_a_new_acquisition_is_admitted() {
    let workspace = Workspace::new();
    let child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "runtime_process_fixture", "--nocapture"])
        .env("EAL_ADVERSARIAL_FIXTURE", "owner")
        .env("EAL_ADVERSARIAL_WORKSPACE", &workspace.0)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut fixture = ChildFixture { child, group: None };
    let group = wait_for_pid(&workspace.0.join("collector.pid"), &mut fixture.child);
    fixture.group = Some(group);
    let descendant = wait_for_pid(&workspace.0.join("descendant.pid"), &mut fixture.child);
    let coordinator = WorkspaceAcquisition::new(&workspace.0).unwrap();
    assert!(coordinator.hold(Duration::ZERO).is_err());
    fixture.child.kill().unwrap();
    fixture.child.wait().unwrap();
    let _next = coordinator.hold(Duration::from_secs(4)).unwrap();
    assert_eq!(
        rustix::process::test_kill_process_group(group),
        Err(rustix::io::Errno::SRCH),
        "Lease released before the complete collector group was reaped"
    );
    assert_eq!(
        rustix::process::test_kill_process(descendant),
        Err(rustix::io::Errno::SRCH)
    );
}

// This test executable is also a Rust fixture binary. Explicit child invocation
// selects this entrypoint; ordinary test execution performs no fixture effects.
#[test]
fn runtime_process_fixture() {
    let Ok(mode) = std::env::var("EAL_ADVERSARIAL_FIXTURE") else {
        return;
    };
    let workspace = Workspace(PathBuf::from(
        std::env::var_os("EAL_ADVERSARIAL_WORKSPACE").unwrap(),
    ));
    // Child fixtures must not remove their parent's workspace on exit.
    let path = workspace.0.clone();
    std::mem::forget(workspace);
    if mode == "owner" {
        let coordinator = WorkspaceAcquisition::new(&path).unwrap();
        let lease = coordinator.hold(Duration::ZERO).unwrap();
        let workspace = Workspace(path);
        let command = request(
            &workspace,
            "echo $$ > collector.pid; /bin/sh -c 'echo $$ > descendant.pid; while :; do /bin/sleep 1; done' & wait",
            64,
            Duration::from_secs(15),
        );
        let result = supervisor().run(command, Some(&lease));
        std::mem::forget(workspace);
        result.unwrap();
    } else if mode == "lease-clone" {
        let coordinator = WorkspaceAcquisition::new(&path).unwrap();
        for _ in 0..128 {
            let lease = coordinator.hold(Duration::ZERO).unwrap();
            let copy = lease.as_fd().try_clone_to_owned().unwrap();
            drop(lease);
            assert!(coordinator.hold(Duration::ZERO).is_err());
            drop(copy);
            drop(coordinator.hold(Duration::ZERO).unwrap());
        }
    } else if mode == "environment" {
        let mut binding = ToolBinding::command("collector", "1", vec!["/bin/sh".into()]);
        binding.inherit_env = Some(vec!["EAL_ADVERSARIAL_VALUE".into()]);
        let digest = binding
            .process_environment_digest(&[7; 32], None)
            .unwrap()
            .unwrap();
        fs::write(path.join("environment.digest"), digest).unwrap();
    } else {
        panic!("Unknown explicit runtime fixture mode");
    }
}

fn assert_store_refuses(store: &RunStore) {
    assert!(store.get("existing", Some("test")).is_err());
    assert!(
        store
            .put("test", &json!({"new":true}), Some("new"))
            .is_err()
    );
    assert!(store.list(None, 100).is_err());
    assert!(
        store
            .find_observations(&ObservationFilter::new("e", "env", "request", "binding"))
            .is_err()
    );
}

#[test]
fn live_stores_and_their_clones_refuse_replaced_symlinked_or_hardlinked_identities() {
    for object in ["database", "key"] {
        for attack in ["replacement", "symlink", "hardlink", "mode"] {
            let workspace = Workspace::new();
            let store = RunStore::new(workspace.database()).unwrap();
            store
                .put("test", &json!({"retained":true}), Some("existing"))
                .unwrap();
            let clone = store.clone();
            let target = if object == "database" {
                workspace.database()
            } else {
                workspace.0.join("runs.sqlite3.binding-key")
            };
            let original = fs::read(&target).unwrap();
            match attack {
                "replacement" => {
                    fs::rename(&target, workspace.0.join("retired")).unwrap();
                    let mut file = OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .mode(0o600)
                        .open(&target)
                        .unwrap();
                    file.write_all(&original).unwrap();
                }
                "symlink" => {
                    fs::rename(&target, workspace.0.join("retired")).unwrap();
                    symlink(workspace.0.join("retired"), &target).unwrap();
                }
                "hardlink" => fs::hard_link(&target, workspace.0.join("alias")).unwrap(),
                "mode" => fs::set_permissions(&target, fs::Permissions::from_mode(0o640)).unwrap(),
                _ => unreachable!(),
            }
            assert_store_refuses(&store);
            assert_store_refuses(&clone);
            assert!(!workspace.0.join("new").exists(), "{object} {attack}");
        }
    }
}

#[test]
fn in_place_key_changes_and_sidecar_links_are_checked_before_every_operation() {
    let workspace = Workspace::new();
    let store = RunStore::new(workspace.database()).unwrap();
    store.put("test", &json!({}), Some("existing")).unwrap();
    let key_path = workspace.0.join("runs.sqlite3.binding-key");
    let mut bytes = fs::read(&key_path).unwrap();
    bytes[0] ^= 0x80;
    fs::write(key_path, bytes).unwrap();
    assert_store_refuses(&store);
    for suffix in ["-wal", "-shm", "-journal"] {
        let workspace = Workspace::new();
        let store = RunStore::new(workspace.database()).unwrap();
        store.put("test", &json!({}), Some("existing")).unwrap();
        let target = workspace.write("external", b"not a SQLite sidecar");
        symlink(target, workspace.0.join(format!("runs.sqlite3{suffix}"))).unwrap();
        assert_store_refuses(&store);
    }
}

const SOURCE: &str = r#"language "EAL/3"
environment lab { require true
}
tool collector { version "1"
}
evidence observed { tool collector
  environment lab
  kind test
  max_age 1
  require ok
}
reasoning check { method "structured/1"
  rationale "A bounded synthetic observation."
}
claim ready { statement "The supplied observation meets its declared predicate."
  environment lab
}
argument route = [evidence observed] via check => ready
"#;

fn old_record(program: &eal_core::semantic::Program, execution: Option<&str>) -> Value {
    let limits = &program.limits;
    let context = json!({});
    let acquisition = json!({"tool":"collector","tool_version":"1","input":{},"context":context});
    let request = json!({"evidence_id":"observed","environment":"lab","tool":"collector","tool_version":"1","input":{},"context":context});
    let value = json!({"ok":true});
    let mut record = json!({"schema":"EAL/observation-record/1","evidence_id":"observed","source_digest":program.source_digest,
        "tool":"collector","tool_version":"1","tool_binding_digest":"0".repeat(64),"evidence_kind":"test","environment":"lab",
        "environment_fingerprint":evaluation::environment_fingerprint("lab",&context,limits).unwrap(),
        "input_digest":canonical::canonical_digest(&json!({}),limits).unwrap(),"input":{},"context":context,
        "acquisition_request":acquisition,"acquisition_request_digest":canonical::canonical_digest(&acquisition,limits).unwrap(),
        "request_digest":canonical::canonical_digest(&request,limits).unwrap(),"collected_at":"1970-01-01T00:00:00Z",
        "observed_at":"1970-01-01T00:00:00Z","started_at":"1970-01-01T00:00:00Z","ingested_at":"1970-01-01T00:00:01Z",
        "run_id":"original","status":"ok","value":value,"data_digest":canonical::canonical_digest(&value,limits).unwrap()});
    if let Some(execution) = execution {
        record["process_environment_digest"] = json!(execution);
    }
    record
}

#[test]
fn reuse_rebinds_source_without_refreshing_observation_time_or_ingestion_time() {
    let workspace = Workspace::new();
    let store = RunStore::new(workspace.database()).unwrap();
    let original_program =
        composition::compile(SOURCE, None, "", ExecutionLimits::default()).unwrap();
    let edited = composition::compile(
        &format!("{SOURCE}\n// Explicit source revision\n"),
        None,
        "",
        ExecutionLimits::default(),
    )
    .unwrap();
    let original = old_record(&original_program, None);
    store
        .put("observation", &original, Some("original"))
        .unwrap();
    let rebound = ObservationRebinder::new(store.clone())
        .prepare_record(
            &edited,
            &json!({}),
            "observed",
            &original,
            Some(&"0".repeat(64)),
            None,
        )
        .unwrap();
    for field in [
        "collected_at",
        "observed_at",
        "started_at",
        "ingested_at",
        "value",
        "data_digest",
    ] {
        assert_eq!(rebound[field], original[field], "{field}");
    }
    assert_eq!(rebound["source_digest"], edited.source_digest);
    assert_ne!(rebound["run_id"], original["run_id"]);
    assert_eq!(
        store.get("original", Some("observation")).unwrap(),
        original
    );
    let assessment = evaluation::evaluate(
        &edited,
        &json!({"observed":rebound}),
        "2026-10-20T12:00:00Z",
        &json!({}),
        methods::default_registry(),
        None,
    )
    .unwrap();
    assert_eq!(assessment["evidence"]["observed"]["status"], "unavailable");
    assert_eq!(
        assessment["evidence"]["observed"]["availability_issues"],
        json!(["stale_observation"])
    );
    assert_ne!(assessment["claims"]["ready"]["status"], "supported");
}

#[test]
fn changed_effective_environment_and_pinned_bytes_cannot_authorize_reuse() {
    let workspace = Workspace::new();
    let mut digests = Vec::new();
    for value in ["first", "second"] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "runtime_process_fixture", "--nocapture"])
            .env("EAL_ADVERSARIAL_FIXTURE", "environment")
            .env("EAL_ADVERSARIAL_WORKSPACE", &workspace.0)
            .env("EAL_ADVERSARIAL_VALUE", value)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        digests.push(fs::read_to_string(workspace.0.join("environment.digest")).unwrap());
    }
    assert_ne!(digests[0], digests[1]);
    let store = RunStore::new(workspace.database()).unwrap();
    let program = composition::compile(SOURCE, None, "", ExecutionLimits::default()).unwrap();
    let original = old_record(&program, Some(&digests[0]));
    store
        .put("observation", &original, Some("original"))
        .unwrap();
    let rebinder = ObservationRebinder::new(store);
    let binding = "0".repeat(64);
    assert!(
        rebinder
            .prepare_record(
                &program,
                &json!({}),
                "observed",
                &original,
                Some(&binding),
                Some(&digests[1])
            )
            .is_err()
    );
    assert!(
        rebinder
            .candidates(
                &program,
                &json!({}),
                "observed",
                &binding,
                Some(&digests[1]),
                100
            )
            .unwrap()
            .is_empty()
    );
    let pin = workspace.write("collector-artifact", b"original bytes");
    let mut command = ToolBinding::command("collector", "1", vec!["/bin/sh".into()]);
    command.inherit_env = Some(Vec::new());
    command.pinned_files = vec![PinnedFile {
        path: "collector-artifact".into(),
        sha256: format!("{:x}", Sha256::digest(b"original bytes")),
    }];
    assert!(command.binding_digest(&[7; 32], Some(&workspace.0)).is_ok());
    fs::write(pin, b"changed bytes").unwrap();
    assert!(
        command
            .binding_digest(&[7; 32], Some(&workspace.0))
            .is_err()
    );
}

fn runtime(workspace: &Workspace, store: RunStore, binding: ToolBinding) -> EvidenceRuntime {
    EvidenceRuntime::new(
        workspace.0.clone(),
        ToolRegistry::new(BTreeMap::from([("collector".into(), binding)])).unwrap(),
        store,
        methods::default_registry().clone(),
        CollectionScheduler::default(),
        supervisor(),
    )
    .unwrap()
}

#[test]
fn file_collection_and_later_reuse_preserve_the_original_measurement_age() {
    let workspace = Workspace::new();
    let store = RunStore::new(workspace.database()).unwrap();
    let program = composition::compile(SOURCE, None, "", ExecutionLimits::default()).unwrap();
    let edited = composition::compile(
        &format!("{SOURCE}\n// Reviewed source edit\n"),
        None,
        "",
        ExecutionLimits::default(),
    )
    .unwrap();
    let envelope = json!({"value":{"ok":true},"observed_at":"1970-01-01T00:00:00Z","context":{},
        "request":{"tool":"collector","tool_version":"1","input":{},"context":{}}});
    workspace.write("observation.json", serde_json::to_vec(&envelope).unwrap());
    let runtime = runtime(
        &workspace,
        store.clone(),
        ToolBinding::json_file("collector", "1", "observation.json"),
    );
    let collection = runtime.collect(&program, &json!({}), None).unwrap();
    let original = &collection["records"]["observed"];
    assert_eq!(original["status"], "ok");
    assert_eq!(original["collected_at"], "1970-01-01T00:00:00Z");
    assert_ne!(original["ingested_at"], original["collected_at"]);
    let bindings = runtime.current_binding_digests(&edited);
    let rebound = ObservationRebinder::new(store.clone())
        .prepare_record(
            &edited,
            &json!({}),
            "observed",
            original,
            bindings["observed"].as_str(),
            None,
        )
        .unwrap();
    assert_eq!(rebound["collected_at"], original["collected_at"]);
    assert_eq!(rebound["ingested_at"], original["ingested_at"]);
    let assessment = evaluation::evaluate(
        &edited,
        &json!({"observed":rebound}),
        "2026-10-20T12:00:00Z",
        &json!({}),
        methods::default_registry(),
        Some(&bindings),
    )
    .unwrap();
    assert_eq!(
        assessment["evidence"]["observed"]["availability_issues"],
        json!(["stale_observation"])
    );
    assert_eq!(
        store.list(Some("observation"), 100).unwrap().len(),
        1,
        "Preparing reuse must not write another observation"
    );
}

#[test]
fn collector_artifact_changes_during_acquisition_fail_and_cannot_be_reused() {
    for during_call in [false, true] {
        let workspace = Workspace::new();
        let store = RunStore::new(workspace.database()).unwrap();
        let program = composition::compile(SOURCE, None, "", ExecutionLimits::default()).unwrap();
        let pin = workspace.write("artifact", b"reviewed bytes");
        let script = if during_call {
            "printf x >> called; printf changed > artifact; printf '{\"value\":{\"ok\":true}}'"
        } else {
            "printf x >> called; printf '{\"value\":{\"ok\":true}}'"
        };
        let mut binding = ToolBinding::command(
            "collector",
            "1",
            vec!["/bin/sh".into(), "-c".into(), script.into()],
        );
        binding.inherit_env = Some(Vec::new());
        binding.timeout_seconds = 2.0;
        binding.pinned_files = vec![PinnedFile {
            path: "artifact".into(),
            sha256: format!("{:x}", Sha256::digest(b"reviewed bytes")),
        }];
        let runtime = runtime(&workspace, store.clone(), binding);
        let collection = runtime.collect(&program, &json!({}), None).unwrap();
        let original = &collection["records"]["observed"];
        assert_eq!(original["status"], if during_call { "error" } else { "ok" });
        if during_call {
            assert!(original.get("value").is_none());
        } else {
            fs::write(pin, b"changed bytes").unwrap();
        }
        let current = runtime.current_binding_digests(&program);
        assert!(current["observed"].is_null());
        assert!(
            ObservationRebinder::new(store.clone())
                .prepare_record(
                    &program,
                    &json!({}),
                    "observed",
                    original,
                    current["observed"].as_str(),
                    None
                )
                .is_err()
        );
        assert_eq!(
            fs::read(workspace.0.join("called")).unwrap(),
            b"x",
            "Reuse validation must never relaunch the collector"
        );
        assert_eq!(store.list(Some("observation"), 100).unwrap().len(), 1);
    }
}

#[test]
fn one_coordinator_and_its_clones_refuse_a_replaced_still_leased_inode() {
    let workspace = Workspace::new();
    let coordinator = WorkspaceAcquisition::new(&workspace.0).unwrap();
    let clone = coordinator.clone();
    let lease = coordinator.hold(Duration::ZERO).unwrap();
    fs::rename(coordinator.path(), workspace.0.join("retired-lock")).unwrap();
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(coordinator.path())
        .unwrap();
    assert!(coordinator.hold(Duration::ZERO).is_err());
    assert!(clone.hold(Duration::ZERO).is_err());
    drop(lease);
    assert!(
        coordinator.hold(Duration::ZERO).is_err(),
        "Identity replacement requires an explicit new coordinator"
    );
}

fn service(workspace: &Workspace, limits: ExecutionLimits) -> ReasoningService {
    ReasoningService::new(ServiceConfig {
        workspace: workspace.0.clone(),
        registry_path: None,
        database_path: Some(workspace.database()),
        method_registry: methods::default_registry().clone(),
        method_executor: None,
        scheduler: CollectionScheduler::default(),
        supervisor: supervisor(),
        limits,
        lease_timeout: Duration::from_secs(1),
    })
    .unwrap()
}

#[test]
fn maximum_configured_source_budget_does_not_overflow_small_import_reads() {
    let workspace = Workspace::new();
    workspace.write(
        "library.eal",
        b"language \"EAL/3\"\nenvironment imported { require true\n}\n",
    );
    let service = service(
        &workspace,
        ExecutionLimits {
            source_bytes: usize::MAX,
            ..ExecutionLimits::default()
        },
    );
    let program = service
        .parse("language \"EAL/3\"\nimport \"library.eal\" as lib\n")
        .unwrap();
    assert!(program.environments.contains_key("lib.imported"));
}

#[test]
fn service_import_base_is_workspace_root_and_nested_imports_keep_definition_origin() {
    let workspace = Workspace::new();
    fs::create_dir(workspace.0.join("nested")).unwrap();
    workspace.write(
        "dependency.eal",
        b"language \"EAL/3\"\nenvironment root { require true\n}\n",
    );
    workspace.write(
        "nested/dependency.eal",
        b"language \"EAL/3\"\nenvironment nested { require true\n}\n",
    );
    workspace.write(
        "nested/library.eal",
        b"language \"EAL/3\"\nimport \"dependency.eal\" as data\n",
    );
    let service = service(&workspace, ExecutionLimits::default());
    let nested = service
        .parse("language \"EAL/3\"\nimport \"nested/library.eal\" as lib\n")
        .unwrap();
    assert!(nested.environments.contains_key("lib.data.nested"));
    assert_eq!(
        nested.source_files["lib.data.nested"],
        "nested/dependency.eal"
    );
    let source_file = workspace.write(
        "nested/main.eal",
        b"language \"EAL/3\"\nimport \"dependency.eal\" as data\n",
    );
    let root = service
        .parse(&fs::read_to_string(source_file).unwrap())
        .unwrap();
    assert!(root.environments.contains_key("data.root"));
    assert!(!root.environments.contains_key("data.nested"));
    assert_eq!(root.source_files["data.root"], "dependency.eal");
}

#[test]
fn collector_output_budget_and_later_assessment_record_budget_are_independent() {
    let workspace = Workspace::new();
    const NOW: &str = "2026-10-20T12:00:00Z";
    const PAYLOAD_BYTES: usize = 5 * 1024 * 1024;
    workspace.write("tools.toml",b"[tools.collector]\nkind='json_file'\nversion='1'\npath='observation.json'\nmax_output_bytes=8388608\n");
    let envelope = json!({"value":{"ok":true,"payload":"x".repeat(PAYLOAD_BYTES)},"observed_at":NOW,"context":{},
        "request":{"tool":"collector","tool_version":"1","input":{},"context":{}}});
    workspace.write("observation.json", serde_json::to_vec(&envelope).unwrap());
    let service = ReasoningService::new(ServiceConfig {
        workspace: workspace.0.clone(),
        registry_path: Some(workspace.0.join("tools.toml")),
        database_path: Some(workspace.database()),
        method_registry: methods::default_registry().clone(),
        method_executor: None,
        scheduler: CollectionScheduler::default(),
        supervisor: supervisor(),
        limits: ExecutionLimits::default(),
        lease_timeout: Duration::from_secs(1),
    })
    .unwrap();
    assert_eq!(service.limits().record_bytes, 4 * 1024 * 1024);
    let collection = service.collect(SOURCE, &json!({}), None).unwrap();
    let original = &collection["records"]["observed"];
    assert_eq!(original["status"], "ok");
    assert_eq!(
        original["value"]["payload"].as_str().unwrap().len(),
        PAYLOAD_BYTES
    );
    assert_eq!(
        service
            .store()
            .get(original["run_id"].as_str().unwrap(), Some("observation"))
            .unwrap(),
        *original
    );
    let assessment = service
        .reason(
            SOURCE,
            &json!({}),
            collection["collection_id"].as_str(),
            Some(NOW),
        )
        .unwrap();
    assert_eq!(assessment["evidence"]["observed"]["status"], "unavailable");
    assert!(
        assessment["evidence"]["observed"]["availability_issues"]
            .as_array()
            .unwrap()
            .contains(&json!("invalid_observation"))
    );
    assert!(
        assessment["evidence"]["observed"]["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| reason.as_str().unwrap().contains("byte limit"))
    );
    assert_ne!(assessment["claims"]["ready"]["status"], "supported");
}
