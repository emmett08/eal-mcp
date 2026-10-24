use eal_core::{ExecutionLimits, composition, evaluation, methods};
use eal_runtime::{
    acquisition::{ToolBinding, ToolRegistry},
    collection::{EvidenceRuntime, acquisition_request},
    process::Supervisor,
    reuse::ObservationRebinder,
    scheduler::CollectionScheduler,
    store::{RunStore, StoreEntry},
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf};
use uuid::Uuid;

const SOURCE: &str = "language \"EAL/3\"\nenvironment lab {\nrequire site == \"bench\"\n}\ntool runner {\nversion \"1\"\n}\nevidence measured {\ntool runner\nkind test\nenvironment lab\nmax_age 60\ninput {\"value\":7}\nrequire passed == true\n}\nreasoning observed {\nmethod \"structured/1\"\nrationale \"The observation supports this bounded claim.\"\n}\nclaim works {\nstatement \"The selected test passes.\"\nenvironment lab\n}\nargument result = [evidence measured] via observed => works\n";
struct Case {
    dir: PathBuf,
    runtime: EvidenceRuntime,
    program: eal_core::semantic::Program,
    context: Value,
    record: Value,
}
impl Case {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!("eal-reuse-{}", Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let program = composition::compile(SOURCE, None, "", ExecutionLimits::default()).unwrap();
        let context = json!({"site":"bench"});
        let envelope = json!({"observed_at":"2020-01-02T03:04:05Z","context":context,"request":acquisition_request(&program,"measured",&context).unwrap(),"value":{"passed":true}});
        std::fs::write(
            dir.join("value.json"),
            serde_json::to_vec(&envelope).unwrap(),
        )
        .unwrap();
        let binding = ToolBinding::json_file("runner", "1", "value.json");
        let runtime = EvidenceRuntime::new(
            dir.clone(),
            ToolRegistry::new(BTreeMap::from([("runner".into(), binding)])).unwrap(),
            RunStore::new(dir.join("runs.sqlite3")).unwrap(),
            methods::default_registry().clone(),
            CollectionScheduler::default(),
            Supervisor::new(dir.join("unused-supervisor")),
        )
        .unwrap();
        let record =
            runtime.collect(&program, &context, None).unwrap()["records"]["measured"].clone();
        assert_eq!(record["status"], "ok");
        Self {
            dir,
            runtime,
            program,
            context,
            record,
        }
    }
    fn binding(&self, p: &eal_core::semantic::Program) -> String {
        self.runtime.current_binding_digests(p)["measured"]
            .as_str()
            .unwrap()
            .to_owned()
    }
}
impl Drop for Case {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn source_revision_reuse_is_prepared_without_write_and_preserves_original_time_and_origin() {
    let case = Case::new();
    let revised = composition::compile(
        &SOURCE.replace(
            "The observation supports this bounded claim.",
            "Reviewed revised rationale.",
        ),
        None,
        "",
        ExecutionLimits::default(),
    )
    .unwrap();
    let rebinder = ObservationRebinder::new(case.runtime.store().clone());
    let binding = case.binding(&revised);
    let new = rebinder
        .prepare_record(
            &revised,
            &case.context,
            "measured",
            &case.record,
            Some(&binding),
            None,
        )
        .unwrap();
    assert_ne!(new["run_id"], case.record["run_id"]);
    assert_eq!(new["source_digest"], revised.source_digest);
    for field in [
        "collected_at",
        "started_at",
        "ingested_at",
        "stdout_digest",
        "stderr_digest",
        "stdout_bytes",
        "stderr_bytes",
        "data_digest",
    ] {
        assert_eq!(new[field], case.record[field], "{field}");
    }
    assert_eq!(new["origin_run_id"], case.record["run_id"]);
    assert_eq!(new["reused_from_run_id"], case.record["run_id"]);
    assert!(new["rebound_at"].is_string());
    assert_eq!(
        case.runtime
            .store()
            .list(Some("observation"), 100)
            .unwrap()
            .len(),
        1
    );
    let collection = json!({"source_digest":revised.source_digest,"context":case.context,"records":{"measured":new}});
    case.runtime
        .store()
        .put_many(&[
            StoreEntry {
                kind: "observation",
                payload: &new,
                record_id: new["run_id"].as_str(),
            },
            StoreEntry {
                kind: "collection",
                payload: &collection,
                record_id: None,
            },
        ])
        .unwrap();
    let again = rebinder
        .prepare_record(
            &revised,
            &case.context,
            "measured",
            &new,
            Some(&binding),
            None,
        )
        .unwrap();
    assert_eq!(again["origin_run_id"], case.record["run_id"]);
    assert_eq!(again["reused_from_run_id"], new["run_id"]);
    assert_eq!(
        case.runtime
            .store()
            .get(case.record["run_id"].as_str().unwrap(), Some("observation"))
            .unwrap(),
        case.record
    );
    let assessed = evaluation::evaluate_default(
        &revised,
        &json!({"measured":new}),
        "2026-09-23T12:00:00Z",
        &case.context,
    )
    .unwrap();
    assert_eq!(assessed["claims"]["works"]["status"], "unsupported");
    assert_eq!(
        assessed["evidence"]["measured"]["availability_issues"],
        json!(["stale_observation"])
    );
}

#[test]
fn reuse_checks_exact_stored_record_acquisition_and_typed_identity_before_write() {
    let case = Case::new();
    let rebinder = ObservationRebinder::new(case.runtime.store().clone());
    let binding = case.binding(&case.program);
    for pointer in ["/collected_at", "/stdout_bytes", "/value/passed"] {
        let mut edited = case.record.clone();
        let original = edited.pointer_mut(pointer).unwrap();
        *original = match pointer {
            "/collected_at" => json!("2026-09-23T12:00:00Z"),
            "/stdout_bytes" => json!(original.as_u64().unwrap() as f64),
            _ => json!(false),
        };
        assert!(
            rebinder
                .prepare_record(
                    &case.program,
                    &case.context,
                    "measured",
                    &edited,
                    Some(&binding),
                    None
                )
                .unwrap_err()
                .contains("differs from stored acquisition")
        );
    }
    let renamed = composition::compile(
        &SOURCE.replace("measured", "another_id"),
        None,
        "",
        ExecutionLimits::default(),
    )
    .unwrap();
    assert!(
        rebinder
            .prepare_record(
                &renamed,
                &case.context,
                "another_id",
                &case.record,
                Some(&binding),
                None
            )
            .unwrap_err()
            .contains("evidence_id differs")
    );
    assert!(
        rebinder
            .prepare_record(
                &case.program,
                &json!({"site":"elsewhere"}),
                "measured",
                &case.record,
                Some(&binding),
                None
            )
            .unwrap_err()
            .contains("environment_fingerprint differs")
    );
    assert!(
        rebinder
            .prepare_record(
                &case.program,
                &case.context,
                "measured",
                &case.record,
                Some(&"0".repeat(64)),
                None
            )
            .unwrap_err()
            .contains("tool_binding_digest differs")
    );
    assert!(
        rebinder
            .prepare_record(
                &case.program,
                &case.context,
                "measured",
                &case.record,
                None,
                None
            )
            .unwrap_err()
            .contains("No current tool binding")
    );
    assert_eq!(
        case.runtime
            .store()
            .list(Some("observation"), 100)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn candidates_are_newest_first_acquisition_compatible_and_leave_freshness_to_assessment() {
    let case = Case::new();
    let rebinder = ObservationRebinder::new(case.runtime.store().clone());
    let binding = case.binding(&case.program);
    let prepared = rebinder
        .prepare_record(
            &case.program,
            &case.context,
            "measured",
            &case.record,
            Some(&binding),
            None,
        )
        .unwrap();
    case.runtime
        .store()
        .put("observation", &prepared, prepared["run_id"].as_str())
        .unwrap();
    let candidates = rebinder
        .candidates(
            &case.program,
            &case.context,
            "measured",
            &binding,
            None,
            100,
        )
        .unwrap();
    assert_eq!(
        candidates
            .iter()
            .map(|r| r["run_id"].clone())
            .collect::<Vec<_>>(),
        vec![prepared["run_id"].clone(), case.record["run_id"].clone()]
    );
    assert_eq!(candidates[0]["collected_at"], "2020-01-02T03:04:05Z");
    assert!(
        rebinder
            .candidates(&case.program, &case.context, "measured", &binding, None, 0)
            .is_err()
    );
    assert!(
        rebinder
            .candidates(
                &case.program,
                &case.context,
                "measured",
                &"0".repeat(64),
                None,
                100
            )
            .unwrap()
            .is_empty()
    );
    let renamed = composition::compile(
        &SOURCE.replace("measured", "another_id"),
        None,
        "",
        ExecutionLimits::default(),
    )
    .unwrap();
    assert!(
        rebinder
            .candidates(&renamed, &case.context, "another_id", &binding, None, 100)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn legacy_streams_are_removed_and_command_environment_identity_is_required() {
    let case = Case::new();
    let rebinder = ObservationRebinder::new(case.runtime.store().clone());
    let binding = case.binding(&case.program);
    let mut legacy = case.record.clone();
    legacy["run_id"] = json!(Uuid::new_v4().to_string());
    legacy["stdout"] = json!("private legacy stream");
    legacy["stderr"] = json!("private legacy diagnostic");
    legacy["process_environment_digest"] = json!("c".repeat(64));
    case.runtime
        .store()
        .put("observation", &legacy, legacy["run_id"].as_str())
        .unwrap();
    assert!(
        rebinder
            .prepare_record(
                &case.program,
                &case.context,
                "measured",
                &legacy,
                Some(&binding),
                None
            )
            .unwrap_err()
            .contains("Current process environment identity is required")
    );
    let new = rebinder
        .prepare_record(
            &case.program,
            &case.context,
            "measured",
            &legacy,
            Some(&binding),
            Some(&"c".repeat(64)),
        )
        .unwrap();
    assert!(new.get("stdout").is_none());
    assert!(new.get("stderr").is_none());
    assert_eq!(new["stdout_digest"], legacy["stdout_digest"]);
    assert!(
        rebinder
            .prepare_record(
                &case.program,
                &case.context,
                "measured",
                &legacy,
                Some(&binding),
                Some(&"d".repeat(64))
            )
            .unwrap_err()
            .contains("process_environment_digest differs")
    );
    assert!(
        rebinder
            .candidates(
                &case.program,
                &case.context,
                "measured",
                &binding,
                Some(&"c".repeat(64)),
                100
            )
            .unwrap()
            .iter()
            .any(|record| record["run_id"] == legacy["run_id"])
    );
    assert_eq!(
        case.runtime
            .store()
            .get(legacy["run_id"].as_str().unwrap(), Some("observation"))
            .unwrap(),
        legacy
    );
}

#[test]
fn whole_stored_identity_does_not_spend_one_payloads_node_allowance_on_metadata() {
    let case = Case::new();
    let mut program = case.program.clone();
    program.limits.json_nodes = 100;
    let binding = case.binding(&program);
    let mut original = case.record.clone();
    original["run_id"] = json!(Uuid::new_v4().to_string());
    original["collector_metadata"] = json!(vec![0; 100]);
    case.runtime
        .store()
        .put("observation", &original, original["run_id"].as_str())
        .unwrap();
    assert!(
        ObservationRebinder::new(case.runtime.store().clone())
            .prepare_record(
                &program,
                &case.context,
                "measured",
                &original,
                Some(&binding),
                None
            )
            .is_ok()
    );
}
