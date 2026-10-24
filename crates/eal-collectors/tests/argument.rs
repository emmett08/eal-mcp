//! The supplied argument preserves positive, measured-negative and unavailable evidence.
use eal_core::{ExecutionLimits, composition, evaluation, methods, planning::EvidencePlanner};
use eal_runtime::{
    acquisition::{ToolBinding, ToolRegistry},
    collection::{EvidenceRuntime, acquisition_request},
    process::Supervisor,
    scheduler::CollectionScheduler,
    store::RunStore,
};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn example_plans_both_services_and_distinguishes_failed_observation_from_no_match() {
    let source = include_str!("../../../examples/catalog-sync-operator/case.eal");
    let program = composition::compile(source, None, "", ExecutionLimits::default()).unwrap();
    let plan = EvidencePlanner::new_with_registry(&program, methods::default_registry())
        .unwrap()
        .plan("rollout_observed")
        .unwrap();
    assert_eq!(plan.evidence_ids(), vec!["resources", "traces"]);
    let context = json!({"stage":"production"});
    let now = chrono::Utc::now().to_rfc3339();
    for (trace_value, same_inventory, expected_status) in [
        (Some(true), true, "supported"),
        (Some(true), false, "unsupported"),
        (Some(false), true, "unsupported"),
        (None, true, "unsupported"),
    ] {
        let root =
            std::env::temp_dir().join(format!("eal-collector-argument-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&root).unwrap();
        let registry = ToolRegistry::new(BTreeMap::from([
            (
                "production_resources".into(),
                ToolBinding::json_file("production_resources", "1", "resources.json"),
            ),
            (
                "production_traces".into(),
                ToolBinding::json_file("production_traces", "1", "traces.json"),
            ),
        ]))
        .unwrap();
        for (name, value) in [
            (
                "resources",
                json!({"all_present":true,"inventory_sha256":"REPLACE_WITH_APPROVED_INVENTORY_SHA256"}),
            ),
            (
                "traces",
                json!({"all_matched":trace_value,"inventory_sha256":if same_inventory{"REPLACE_WITH_APPROVED_INVENTORY_SHA256"}else{"different-inventory"}}),
            ),
        ] {
            let envelope = if name == "traces" && trace_value.is_none() {
                json!({"malformed":true})
            } else {
                json!({"observed_at":now,"context":context,"request":acquisition_request(&program,name,&context).unwrap(),"value":value})
            };
            std::fs::write(
                root.join(format!("{name}.json")),
                serde_json::to_vec(&envelope).unwrap(),
            )
            .unwrap();
        }
        let runtime = EvidenceRuntime::new(
            root.clone(),
            registry,
            RunStore::new(root.join("runs.sqlite3")).unwrap(),
            methods::default_registry().clone(),
            CollectionScheduler::new(1).unwrap(),
            Supervisor::new(root.join("unused-supervisor")),
        )
        .unwrap();
        let collection = runtime
            .collect(&program, &context, Some(&plan.evidence_ids()))
            .unwrap();
        let assessment =
            evaluation::evaluate_default(&program, &collection["records"], &now, &context).unwrap();
        assert_eq!(
            assessment["claims"]["rollout_observed"]["status"],
            expected_status
        );
        if trace_value.is_none() {
            assert_eq!(collection["records"]["traces"]["status"], "error");
            assert_eq!(assessment["evidence"]["traces"]["status"], "unavailable");
        } else {
            assert_eq!(collection["records"]["traces"]["status"], "ok");
        }
        drop(runtime);
        std::fs::remove_dir_all(root).unwrap();
    }
}
