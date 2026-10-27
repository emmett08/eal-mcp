//! Pinned pure checks and native registration correspondence without acquisition.
#![cfg(unix)]
use eal_coordination::{
    contracts::{ServerSpec, TaskSpec},
    host::HostPaths,
    profiles,
};
use eal_runtime::catalogue::RegisterOptions;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("eal-coordination-profile-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn server(&self, engineering: bool) -> ServerSpec {
        ServerSpec::from_value(&json!({"workspace":self.0,"registry":null,"database":null,"limits":null,"methods":if engineering {Some("engineering")} else {None}}), &self.0).unwrap()
    }
    fn write(&self, source: &str) {
        std::fs::write(self.0.join("source.eal"), source).unwrap();
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn host() -> HostPaths {
    HostPaths::from_directory(
        Path::new(env!("CARGO_BIN_EXE_eal-coordinate"))
            .parent()
            .unwrap(),
    )
}
fn task(requirements: Value, api: bool) -> TaskSpec {
    TaskSpec::from_value(&json!({"task_id":"bounded","question":"What does this registered sample support?","entry_id":"sample","claims":["ready","challenged"],"profile":if api {"api_load/1"} else {"registered_claims/1"},"requirements":requirements})).unwrap()
}
fn api_requirements() -> Value {
    json!({"context":{"service":"orders-api","build_id":"build-1","dataset":"synthetic"},
        "workload":{"service":"orders-api","build_id":"build-1","dataset":"synthetic","run_id":"run-1","concurrent_clients":10},
        "criteria":{"minimum_requests":100,"maximum_sample_p95_ms":200,"maximum_error_rate_percent":1,"evidence_max_age_seconds":86400},
        "claim_outputs":{"ready":"passes","challenged":"fails"}})
}
fn api_source() -> String {
    format!(
        "language \"EAL/4\"\nenvironment lab {{\nrequire true\n}}\ntool report {{\nversion \"1\"\n}}\nevidence sample {{\ntool report\nkind api_report\nenvironment lab\nmax_age 86400\ninput {}\nrequire true\n}}\nreasoning passing {{\nmethod \"engineering/api-load-criteria/1\"\nrationale \"Check the pinned finite sample.\"\nrequire passes == true\n}}\nreasoning failing {{\nmethod \"engineering/api-load-criteria/1\"\nrationale \"Check the pinned finite sample failure.\"\nrequire fails == true\n}}\nclaim ready {{\nstatement \"The sample passes.\"\nenvironment lab\n}}\nclaim challenged {{\nstatement \"The sample fails.\"\nenvironment lab\n}}\nargument pass = [evidence sample] via passing => ready\nargument fail = [evidence sample] via failing => challenged\n",
        json!({"service":"orders-api","build_id":"build-1","dataset":"synthetic","run_id":"run-1","concurrent_clients":10,"report_sha256":"a".repeat(64)})
    )
}
fn register(server: &ServerSpec, requirements: &Value, host: &HostPaths) {
    profiles::knowledge_with_host(server, host)
        .unwrap()
        .catalogue
        .register(
            "source.eal",
            RegisterOptions {
                entry_id: Some("sample"),
                context: Some(&requirements["context"]),
                ..Default::default()
            },
        )
        .unwrap();
}

#[test]
fn api_profile_checks_complete_workload_freshness_routes_and_native_method_identity() {
    let workspace = Workspace::new();
    let host = host();
    let server = workspace.server(true);
    let requirements = api_requirements();
    workspace.write(&api_source());
    register(&server, &requirements, &host);
    let selected = task(requirements.clone(), true);
    let anchor = profiles::pin_task_with_host(&server, &selected, &host).unwrap();
    assert_eq!(
        profiles::check_task_with_host(&server, &anchor, &host).unwrap()["ok"],
        true
    );
    for (section, field, value, code) in [
        (
            "workload",
            "run_id",
            json!("other-run"),
            "workload_mismatch",
        ),
        (
            "workload",
            "concurrent_clients",
            json!(20),
            "workload_mismatch",
        ),
        (
            "criteria",
            "maximum_sample_p95_ms",
            json!(201),
            "criteria_mismatch",
        ),
        (
            "criteria",
            "evidence_max_age_seconds",
            json!(42),
            "criteria_mismatch",
        ),
    ] {
        let mut changed = requirements.clone();
        changed[section][field] = value;
        let anchor = profiles::pin_task_with_host(&server, &task(changed, true), &host).unwrap();
        assert_eq!(
            profiles::check_task_with_host(&server, &anchor, &host).unwrap()["code"],
            code,
            "{field}"
        );
    }
    let mut removed_method = server.clone();
    removed_method.methods = None;
    assert_eq!(
        profiles::check_task_with_host(&removed_method, &anchor, &host).unwrap()["code"],
        "criteria_mismatch"
    );
    workspace.write(&(api_source() + "argument bypass = [evidence sample] via failing => ready\n"));
    let anchor = profiles::pin_task_with_host(&server, &selected, &host).unwrap();
    assert_eq!(
        profiles::check_task_with_host(&server, &anchor, &host).unwrap()["code"],
        "criteria_mismatch"
    );
    let knowledge = profiles::knowledge_with_host(&server, &host).unwrap();
    assert!(
        knowledge
            .service
            .store()
            .list(Some("observation"), 50)
            .unwrap()
            .is_empty()
    );
    assert!(
        knowledge
            .service
            .store()
            .list(Some("assessment"), 50)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn generic_profile_preserves_type_sensitive_context_and_pinned_source_revision() {
    let workspace = Workspace::new();
    let host = host();
    let server = workspace.server(false);
    let source = "language \"EAL/4\"\nenvironment lab {\nrequire true\n}\nclaim ready {\nstatement \"A bounded sample.\"\nenvironment lab\n}\nclaim challenged {\nstatement \"Another bounded sample.\"\nenvironment lab\n}\n";
    workspace.write(source);
    let requirements = json!({"context":{"count":1}});
    register(&server, &requirements, &host);
    let selected = task(requirements, false);
    let anchor = profiles::pin_task_with_host(&server, &selected, &host).unwrap();
    assert_eq!(
        profiles::check_task_with_host(&server, &anchor, &host).unwrap()["ok"],
        true
    );
    let mut changed = selected.clone();
    changed.requirements["context"]["count"] = json!(true);
    let mismatched = profiles::pin_task_with_host(&server, &changed, &host).unwrap();
    assert_eq!(
        profiles::check_task_with_host(&server, &mismatched, &host).unwrap()["code"],
        "context_mismatch"
    );
    workspace.write(&(source.to_owned() + "// revised source\n"));
    assert_eq!(
        profiles::check_task_with_host(&server, &anchor, &host).unwrap()["code"],
        "source_changed"
    );
}
