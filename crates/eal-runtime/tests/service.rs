//! End-to-end durable service operations with imported synthetic observations.
#![cfg(unix)]
use eal_core::{ExecutionLimits, methods::default_registry};
use eal_runtime::{
    methods::{MethodExecutor, WorkerCommand},
    process::Supervisor,
    scheduler::CollectionScheduler,
    service::{FormalOptions, ReasoningService, ServiceConfig},
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf, time::Duration};

const SOURCE: &str = "language \"EAL/4\"\nenvironment lab {\n require true\n}\ntool runner {\n version \"1\"\n}\nevidence observed {\n tool runner\n kind test\n environment lab\n max_age 60\n input {}\n require passed == true\n}\nreasoning method {\n method \"structured/1\"\n rationale \"A passing synthetic test supports this bounded claim.\"\n}\nclaim working {\n statement \"The synthetic test passed.\"\n environment lab\n}\nargument support = [evidence observed] via method => working\n";
const NOW: &str = "2026-09-23T12:00:00Z";

struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("eal-service-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        let workspace = Self(path);
        workspace.write(
            "tools.toml",
            "[tools.runner]\nkind = 'json_file'\nversion = '1'\npath = 'observation.json'\n",
        );
        workspace.write(
            "observation.json",
            &json!({"observed_at":NOW,"context":{},
            "request":{"tool":"runner","tool_version":"1","input":{},"context":{}},
            "value":{"passed":true}})
            .to_string(),
        );
        workspace
    }
    fn write(&self, name: &str, text: &str) {
        std::fs::write(self.0.join(name), text).unwrap();
    }
    fn service(&self) -> ReasoningService {
        let supervisor = Supervisor::new(companion::path());
        let worker = WorkerCommand::new(
            vec![env!("CARGO_BIN_EXE_eal-method-worker").into()],
            BTreeMap::new(),
            self.0.clone(),
        )
        .unwrap();
        let executor = MethodExecutor::new(supervisor.clone(), worker).unwrap();
        ReasoningService::new(ServiceConfig {
            workspace: self.0.clone(),
            registry_path: Some(self.0.join("tools.toml")),
            database_path: None,
            method_registry: default_registry().clone(),
            method_executor: Some(executor),
            scheduler: CollectionScheduler::default(),
            supervisor,
            limits: ExecutionLimits::default(),
            lease_timeout: Duration::from_secs(1),
        })
        .unwrap()
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn collection(service: &ReasoningService) -> Value {
    service.collect(SOURCE, &json!({}), None).unwrap()
}

#[test]
fn imported_time_is_preserved_across_collection_reopen_assessment_and_explanation() {
    let workspace = Workspace::new();
    let service = workspace.service();
    assert_eq!(service.validate(SOURCE)["valid"], true);
    assert_eq!(
        service.plan(SOURCE, "working").unwrap()["evidence_ids"],
        json!(["observed"])
    );
    let collection = collection(&service);
    let record = &collection["records"]["observed"];
    assert_eq!(record["status"], "ok");
    assert_eq!(record["collected_at"], NOW);
    assert!(record.get("stdout").is_none());
    assert!(record.get("stderr").is_none());
    assert!(record["stdout_bytes"].as_u64().unwrap() > 0);
    let id = collection["collection_id"].as_str().unwrap();
    let assessment = service
        .reason(SOURCE, &json!({}), Some(id), Some(NOW))
        .unwrap();
    assert_eq!(assessment["claims"]["working"]["status"], "supported");
    let assessment_id = assessment["assessment_id"].as_str().unwrap();
    let reopened = workspace.service();
    assert_eq!(reopened.explain(assessment_id, None).unwrap(), assessment);
    assert_eq!(
        reopened.explain(assessment_id, Some("working")).unwrap()["result"],
        assessment["claims"]["working"]
    );
    let stale = reopened
        .reason(SOURCE, &json!({}), Some(id), Some("2026-09-23T12:01:01Z"))
        .unwrap();
    assert_eq!(stale["evidence"]["observed"]["status"], "unavailable");
    assert_ne!(stale["claims"]["working"]["status"], "supported");
}

#[test]
fn durable_collections_require_exact_source_context_and_current_binding() {
    let workspace = Workspace::new();
    let service = workspace.service();
    let collection = collection(&service);
    let id = collection["collection_id"].as_str().unwrap();
    assert!(
        service
            .reason(
                &format!("{SOURCE}// changed\n"),
                &json!({}),
                Some(id),
                Some(NOW)
            )
            .unwrap_err()
            .contains("source")
    );
    assert!(
        service
            .reason(SOURCE, &json!({"changed":true}), Some(id), Some(NOW))
            .unwrap_err()
            .contains("context")
    );
    workspace.write(
        "tools.toml",
        "[tools.runner]\nkind = 'json_file'\nversion = '1'\npath = 'other.json'\n",
    );
    let assessment = service
        .reason(SOURCE, &json!({}), Some(id), Some(NOW))
        .unwrap();
    assert_eq!(assessment["evidence"]["observed"]["status"], "unavailable");
    assert!(
        assessment["evidence"]["observed"]["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| reason.as_str().unwrap().contains("binding"))
    );
}

#[test]
fn stored_collection_compiles_and_solves_through_the_native_worker() {
    let workspace = Workspace::new();
    let service = workspace.service();
    let collection = collection(&service);
    let mut options = FormalOptions::new("working");
    options.now = Some(NOW);
    let result = service
        .compile_aspic(
            SOURCE,
            &json!({}),
            collection["collection_id"].as_str().unwrap(),
            options,
        )
        .unwrap();
    assert_eq!(result["authored_claim_status"], "supported");
    assert_eq!(result["claim_status"], "supported");
    assert_eq!(result["formal"]["query_status"], "accepted");
    assert_eq!(result["collection_id"], collection["collection_id"]);
}

#[test]
fn unselected_invalid_names_and_import_escapes_fail_before_observation_writes() {
    let workspace = Workspace::new();
    let service = workspace.service();
    assert!(
        service
            .collect(SOURCE, &json!({}), Some(&["absent".into()]))
            .is_err()
    );
    assert!(
        service
            .store()
            .list(Some("observation"), 50)
            .unwrap()
            .is_empty()
    );
    assert!(
        service
            .parse("language \"EAL/4\"\nimport \"../outside.eal\" as outside\n")
            .is_err()
    );
    assert!(service.reason(SOURCE, &json!([]), None, Some(NOW)).is_err());
}

#[path = "common/companion.rs"]
mod companion;
