//! Real native workers, strict custom protocol peers and process failure boundaries.
use eal_core::{
    ExecutionLimits,
    methods::{
        MethodContract, MethodError, NativeMethodIdentity, assess_mode, default_registry,
        engineering_registry,
    },
};
use eal_runtime::{
    methods::{MethodExecutor, WorkerCommand},
    process::Supervisor,
};
use serde_json::json;
use std::{collections::BTreeMap, path::PathBuf};

struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("eal-methods-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn executor(path: &Workspace) -> MethodExecutor {
    let supervisor = Supervisor::new(companion::path());
    let worker = WorkerCommand::new(
        vec![env!("CARGO_BIN_EXE_eal-method-worker").into()],
        BTreeMap::new(),
        path.0.clone(),
    )
    .unwrap();
    MethodExecutor::new(supervisor, worker).unwrap()
}
fn contract() -> MethodContract {
    MethodContract::native(
        "example/protocol/1",
        "protocol",
        json!({"type":"object","properties":{},"required":[],"additionalProperties":false}),
        json!({"type":"object","properties":{},"required":[],"additionalProperties":false}),
        json!({"type":"object","properties":{"answer":{"type":"integer"}},"required":["answer"],"additionalProperties":false}),
        NativeMethodIdentity {
            version: "test-command/1".into(),
            artifact_sha256: "f".repeat(64),
        },
        |_, _| Err("In-process callback must not run".into()),
    )
}
fn shell(path: &Workspace, body: &str) -> WorkerCommand {
    WorkerCommand::new(
        vec!["/bin/sh".into(), "-c".into(), body.into()],
        BTreeMap::new(),
        path.0.clone(),
    )
    .unwrap()
}
const READY: &str = "printf '%s\\n' '{\"protocol\":\"eal-method-worker/1\",\"ready\":true}'; IFS= read -r request; ";

#[test]
fn native_optional_methods_are_isolated_and_keep_budget_failure_categories() {
    let workspace = Workspace::new();
    let executor = executor(&workspace);
    let registry = eal_aspic::registry::with_aspic(&engineering_registry()).unwrap();
    let isolated = executor.supervise_registry(&registry).unwrap();
    assert_ne!(registry.fingerprint(), isolated.fingerprint());
    assert_eq!(
        isolated.get("structured/1").unwrap().describe(),
        default_registry().get("structured/1").unwrap().describe()
    );
    assert_eq!(
        isolated.get("engineering/rms/1").unwrap().describe()["execution"],
        "host_registered_executor"
    );
    let result = assess_mode(
        "engineering/rms/1",
        &[json!({"id":"sample","kind":"measurement_series","value":{"origin":0,"samples":[3,3]}})],
        &[],
        &isolated,
    );
    assert_eq!(result["status"], "supported", "{result}");
    assert!((result["details"]["rms"].as_f64().unwrap() - 3.0).abs() < 1e-12);
    let negative = assess_mode(
        "engineering/api-load-criteria/1",
        &[
            json!({"id":"empty","kind":"api_report","value":{"request_count":0,"p95_ms":null,"error_rate_percent":null}}),
        ],
        &[],
        &isolated,
    );
    assert_eq!(negative["status"], "supported", "{negative}");
    assert_eq!(negative["details"]["passes"], false);
    assert_eq!(negative["details"]["fails"], true);
    assert_eq!(
        executor
            .supervise_registry(&isolated)
            .unwrap()
            .fingerprint(),
        isolated.fingerprint()
    );
    let payload = json!({"theory":{"premises":[{"atom":"p","kind":"axiom"},{"atom":"q","kind":"axiom"}],"rules":[],"contraries":[],"goal":"p"}});
    let tight = ExecutionLimits {
        formal_arguments: 1,
        ..Default::default()
    };
    let error = executor
        .execute(eal_aspic::registry::aspic_contract(), &payload, &tight)
        .unwrap_err();
    assert!(matches!(error, MethodError::Incomplete(_)), "{error}");
    let complete = executor
        .execute(
            eal_aspic::registry::aspic_contract(),
            &payload,
            &ExecutionLimits::default(),
        )
        .unwrap();
    assert_eq!(complete["query_accepted"], true);
}

#[test]
fn strict_responses_explicit_environments_and_per_registry_callbacks() {
    let workspace = Workspace::new();
    let executor = executor(&workspace);
    let first = executor
        .register_command(
            default_registry(),
            contract(),
            shell(
                &workspace,
                &format!("{READY}printf '%s\\n' '{{\"ok\":true,\"output\":{{\"answer\":11}}}}'"),
            ),
        )
        .unwrap();
    let second = executor
        .register_command(
            default_registry(),
            contract(),
            shell(
                &workspace,
                &format!("{READY}printf '%s\\n' '{{\"ok\":true,\"output\":{{\"answer\":22}}}}'"),
            ),
        )
        .unwrap();
    let evidence = [json!({"id":"test","kind":"protocol","value":{}})];
    assert_eq!(
        assess_mode("example/protocol/1", &evidence, &[], &first)["details"]["answer"],
        11
    );
    assert_eq!(
        assess_mode("example/protocol/1", &evidence, &[], &second)["details"]["answer"],
        22
    );
    assert_ne!(first.fingerprint(), second.fingerprint());
    for response in [
        "{\"ok\":true,\"ok\":true,\"output\":{\"answer\":1}}",
        "{\"ok\":true,\"output\":{\"answer\":true}}",
        "{\"ok\":true,\"output\":{\"answer\":1},\"extra\":false}",
        "{\"ok\":true,\"output\":{\"answer\":1}}\\n{}",
    ] {
        let registry = executor
            .register_command(
                default_registry(),
                contract(),
                shell(&workspace, &format!("{READY}printf '%s\\n' '{response}'")),
            )
            .unwrap();
        assert_eq!(
            assess_mode("example/protocol/1", &evidence, &[], &registry)["status"],
            "unsupported",
            "{response}"
        );
    }
    let registry=executor.register_command(default_registry(),contract(),shell(&workspace,&format!("{READY}printf '{{\"ok\":true,\"output\":{{\"answer\":%s}}}}\\n' \"$(ulimit -v)\""))).unwrap();
    let output = assess_mode("example/protocol/1", &evidence, &[], &registry);
    assert_eq!(output["details"]["answer"], 256 * 1024, "{output}");
}

#[test]
fn execution_timeout_and_output_overflow_do_not_produce_partial_success() {
    let workspace = Workspace::new();
    let executor = executor(&workspace);
    let evidence = [json!({"id":"test","kind":"protocol","value":{}})];
    let mut startup = contract();
    startup.timeout_seconds = 0.5;
    let delayed = executor
        .register_command(
            default_registry(),
            startup,
            shell(
                &workspace,
                &format!(
                    "sleep 1; {READY}printf '%s\\n' '{{\"ok\":true,\"output\":{{\"answer\":1}}}}'"
                ),
            ),
        )
        .unwrap();
    let completed = assess_mode("example/protocol/1", &evidence, &[], &delayed);
    assert_eq!(
        completed["status"], "supported",
        "startup has its own five-second deadline: {completed}"
    );
    let mut spec = contract();
    spec.timeout_seconds = 0.05;
    let registry = executor
        .register_command(
            default_registry(),
            spec,
            shell(
                &workspace,
                &format!(
                    "{READY}sleep 2; printf '%s\\n' '{{\"ok\":true,\"output\":{{\"answer\":1}}}}'"
                ),
            ),
        )
        .unwrap();
    let result = assess_mode("example/protocol/1", &evidence, &[], &registry);
    assert_eq!(result["status"], "incomplete", "{result}");
    assert_eq!(result["details"], json!({}));
    let mut spec = contract();
    spec.max_output_bytes = 1024;
    let registry = executor
        .register_command(
            default_registry(),
            spec,
            shell(
                &workspace,
                &format!("{READY}i=0; while [ \"$i\" -lt 2048 ]; do printf x; i=$((i+1)); done"),
            ),
        )
        .unwrap();
    let result = assess_mode("example/protocol/1", &evidence, &[], &registry);
    assert_eq!(result["status"], "unsupported", "{result}");
    assert_eq!(result["details"], json!({}));
}

#[test]
fn unknown_unbound_methods_and_changed_custom_artifacts_are_rejected() {
    use std::os::unix::fs::PermissionsExt;
    let workspace = Workspace::new();
    let executor = executor(&workspace);
    assert!(
        executor
            .supervise_registry(&default_registry().with_method(contract()).unwrap())
            .is_err()
    );
    let path = workspace.0.join("registered-method");
    std::fs::write(
        &path,
        format!("#!/bin/sh\n{READY}printf '%s\\n' '{{\"ok\":true,\"output\":{{\"answer\":1}}}}'\n"),
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    let command = WorkerCommand::new(
        vec![path.to_str().unwrap().into()],
        BTreeMap::new(),
        workspace.0.clone(),
    )
    .unwrap();
    let registry = executor
        .register_command(default_registry(), contract(), command)
        .unwrap();
    assert_eq!(
        executor
            .supervise_registry(&registry)
            .unwrap()
            .fingerprint(),
        registry.fingerprint()
    );
    std::fs::write(&path, "#!/bin/sh\nexit 0\n").unwrap();
    let output = assess_mode(
        "example/protocol/1",
        &[json!({"id":"case","kind":"protocol","value":{}})],
        &[],
        &registry,
    );
    assert_eq!(output["status"], "unsupported");
    assert!(
        output["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r.as_str().unwrap().contains("registered artifact identity"))
    );
}

#[test]
fn direct_invalid_contract_limits_are_rejected_before_execution() {
    let workspace = Workspace::new();
    let executor = executor(&workspace);
    let mut invalid = contract();
    invalid.timeout_seconds = f64::NAN;
    assert!(matches!(
        executor.execute(&invalid, &json!({}), &ExecutionLimits::default()),
        Err(MethodError::Invalid(_))
    ));
    let mut invalid = contract();
    invalid.max_output_bytes = usize::MAX;
    assert!(matches!(
        executor.execute(&invalid, &json!({}), &ExecutionLimits::default()),
        Err(MethodError::Invalid(_))
    ));
}

#[test]
fn fifo_executables_are_rejected_without_waiting_for_a_writer() {
    let workspace = Workspace::new();
    let path = workspace.0.join("not-an-executable");
    rustix::fs::mkfifoat(
        rustix::fs::CWD,
        &path,
        rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR,
    )
    .unwrap();
    let (sender, receiver) = std::sync::mpsc::channel();
    let cwd = workspace.0.clone();
    let task = std::thread::spawn(move || {
        let result = WorkerCommand::new(vec![path.to_str().unwrap().into()], BTreeMap::new(), cwd);
        sender.send(result.is_err()).unwrap();
    });
    assert!(
        receiver
            .recv_timeout(std::time::Duration::from_secs(1))
            .expect("FIFO metadata admission must not block")
    );
    task.join().unwrap();
}

#[path = "common/companion.rs"]
mod companion;
