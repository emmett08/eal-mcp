//! Independently authored executable packages exercise acquisition, not a plan sidecar.
use chrono::{Duration, Utc};
use eal_core::{
    ExecutionLimits, composition, evaluation, methods, planning::EvidencePlanner, semantic::Program,
};
use eal_runtime::{
    acquisition::{ToolBinding, ToolRegistry},
    collection::{EvidenceRuntime, acquisition_request},
    process::Supervisor,
    scheduler::CollectionScheduler,
    store::RunStore,
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf};
use uuid::Uuid;

const SOURCE: &str = r#"language "EAL/4"
environment lab {
require site == "bench"
}
tool snapshot {
version "1"
}
evidence measured {
tool snapshot
kind observation
environment lab
max_age 3600
input {}
require complete == true
}
evidence audit {
derive object("count",count(items))
kind calculation
environment lab
max_age 3600
input {"items":null}
bind "/items" from measured "/items"
require count == 3
}
evidence confirmation {
derive result
kind calculation
environment lab
max_age 3600
input {"result":null}
bind "/result" from audit ""
require count == 3
}
reasoning checked {
method "structured/1"
rationale "The finite snapshot has the declared count."
}
claim count_matches {
statement "The observed snapshot contains three items."
environment lab
}
argument support = [evidence audit, confirmation] via checked => count_matches
"#;

struct Case {
    dir: PathBuf,
    program: Program,
    runtime: EvidenceRuntime,
    context: Value,
}
impl Case {
    fn new(source: &str, value: Value, age_seconds: i64, available: bool) -> Self {
        let dir = std::env::temp_dir().join(format!("eal-dependencies-{}", Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let program = composition::compile(source, None, "", ExecutionLimits::default()).unwrap();
        let context = json!({"site":"bench"});
        let observed = (Utc::now() - Duration::seconds(age_seconds)).to_rfc3339();
        let envelope = json!({"observed_at":observed,"context":context,
            "request":acquisition_request(&program,"measured",&context).unwrap(),"value":value});
        std::fs::write(
            dir.join("snapshot.json"),
            serde_json::to_vec(&envelope).unwrap(),
        )
        .unwrap();
        let bindings = if available {
            BTreeMap::from([(
                "snapshot".into(),
                ToolBinding::json_file("snapshot", "1", "snapshot.json"),
            )])
        } else {
            BTreeMap::new()
        };
        let runtime = EvidenceRuntime::new(
            dir.clone(),
            ToolRegistry::new(bindings).unwrap(),
            RunStore::new(dir.join("runs.sqlite3")).unwrap(),
            methods::default_registry().clone(),
            CollectionScheduler::new(4).unwrap(),
            Supervisor::new(dir.join("unused-supervisor")),
        )
        .unwrap();
        Self {
            dir,
            program,
            runtime,
            context,
        }
    }
    fn collect(&self) -> Value {
        self.runtime
            .collect(
                &self.program,
                &self.context,
                Some(&["confirmation".into(), "audit".into()]),
            )
            .unwrap()
    }
    fn assess(&self, records: &Value) -> Value {
        evaluation::evaluate(
            &self.program,
            records,
            &Utc::now().to_rfc3339(),
            &self.context,
            methods::default_registry(),
            Some(&self.runtime.current_binding_digests(&self.program)),
        )
        .unwrap()
    }
}
impl Drop for Case {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}
fn snapshot() -> Value {
    json!({"complete":true,"items":["a","b","c"]})
}

#[test]
fn source_and_shared_derivations_execute_once_without_graph_tool_or_plan() {
    let case = Case::new(SOURCE, snapshot(), 30, true);
    let plan = EvidencePlanner::new(&case.program)
        .unwrap()
        .plan("count_matches")
        .unwrap();
    assert!(plan.closure.evidence.contains("measured"));
    let result = case.collect();
    let records = &result["records"];
    assert_eq!(records.as_object().unwrap().len(), 3);
    assert_eq!(records["audit"]["value"], json!({"count":3}));
    assert_eq!(records["confirmation"]["value"], json!({"count":3}));
    assert_eq!(
        records["audit"]["collected_at"],
        records["measured"]["collected_at"]
    );
    assert_eq!(
        records["confirmation"]["collected_at"],
        records["measured"]["collected_at"]
    );
    assert_eq!(
        records["audit"]["dependencies"][0]["run_id"],
        records["measured"]["run_id"]
    );
    assert_eq!(
        case.runtime
            .store()
            .list(Some("observation"), 100)
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        case.assess(records)["claims"]["count_matches"]["status"],
        "supported"
    );
}

#[test]
fn a_measured_negative_is_data_and_does_not_become_a_tool_failure() {
    let case = Case::new(
        SOURCE,
        json!({"complete":false,"items":["a","b","c"]}),
        0,
        true,
    );
    let result = case.collect();
    assert_eq!(result["records"]["measured"]["status"], "ok");
    assert_eq!(result["records"]["audit"]["status"], "ok");
    let assessed = case.assess(&result["records"]);
    assert_eq!(
        assessed["evidence"]["measured"]["availability_issues"],
        json!(["predicate_not_met"])
    );
    assert_eq!(assessed["evidence"]["audit"]["status"], "available");
}

#[test]
fn failed_and_missing_pointer_inputs_block_descendant_execution() {
    for (source, value, age, available) in [
        (SOURCE.to_owned(), snapshot(), 0, false),
        (
            SOURCE.replace("from measured \"/items\"", "from measured \"/absent\""),
            snapshot(),
            0,
            true,
        ),
    ] {
        let case = Case::new(&source, value, age, available);
        let result = case.collect();
        assert_eq!(result["records"]["audit"]["status"], "error");
        assert_eq!(result["records"]["confirmation"]["status"], "error");
        assert!(result["records"]["audit"].get("value").is_none());
        assert_eq!(
            case.assess(&result["records"])["claims"]["count_matches"]["status"],
            "unsupported"
        );
    }
}

#[test]
fn assessment_rejects_substituted_parents_and_forged_derived_values() {
    let case = Case::new(SOURCE, snapshot(), 0, true);
    let result = case.collect();
    for mode in ["parent", "derived", "input"] {
        let mut records = result["records"].clone();
        match mode {
            "parent" => records["measured"]["run_id"] = json!(Uuid::new_v4().to_string()),
            "derived" => {
                records["audit"]["value"] = json!({"count":7});
                records["audit"]["data_digest"] = json!(
                    eal_core::canonical::canonical_digest(
                        &records["audit"]["value"],
                        &case.program.limits
                    )
                    .unwrap()
                );
            }
            _ => records["audit"]["input"] = json!({"items":["other"]}),
        }
        assert_eq!(
            case.assess(&records)["claims"]["count_matches"]["status"],
            "unsupported",
            "{mode}"
        );
    }
}

#[test]
fn descendants_expire_with_original_measurements_after_computation() {
    let source = SOURCE.replacen(
        "kind calculation\nenvironment lab\nmax_age 3600",
        "kind calculation\nenvironment lab\nmax_age 10",
        1,
    );
    let case = Case::new(&source, snapshot(), 0, true);
    let result = case.collect();
    let assessment = evaluation::evaluate(
        &case.program,
        &result["records"],
        &(Utc::now() + Duration::seconds(20)).to_rfc3339(),
        &case.context,
        methods::default_registry(),
        Some(&case.runtime.current_binding_digests(&case.program)),
    )
    .unwrap();
    assert_eq!(
        assessment["evidence"]["audit"]["availability_issues"],
        json!(["stale_observation"])
    );
    assert_eq!(
        assessment["claims"]["count_matches"]["status"],
        "unsupported"
    );
}

#[test]
fn historical_calculation_preserves_old_measurements_and_explicit_assessment_time() {
    let case = Case::new(SOURCE, snapshot(), 4000, true);
    let result = case.collect();
    assert_eq!(result["records"]["audit"]["status"], "ok");
    assert_eq!(
        result["records"]["audit"]["collected_at"],
        result["records"]["measured"]["collected_at"]
    );
    assert_eq!(
        case.assess(&result["records"])["claims"]["count_matches"]["status"],
        "unsupported"
    );
}

#[test]
fn bound_inputs_reach_a_real_supervised_calculation_tool() {
    let source = SOURCE.replace(
        "evidence audit {\nderive object(\"count\",count(items))",
        "tool calculator {\nversion \"1\"\n}\nevidence audit {\ntool calculator",
    );
    let mut case = Case::new(&source, snapshot(), 30, true);
    let script = "import json,sys; r=json.load(sys.stdin); print(json.dumps({'value':{'count':len(r['input']['items'])}}))";
    case.runtime = EvidenceRuntime::new(
        case.dir.clone(),
        ToolRegistry::new(BTreeMap::from([
            (
                "snapshot".into(),
                ToolBinding::json_file("snapshot", "1", "snapshot.json"),
            ),
            (
                "calculator".into(),
                ToolBinding::command(
                    "calculator",
                    "1",
                    vec!["/usr/bin/python3".into(), "-c".into(), script.into()],
                ),
            ),
        ]))
        .unwrap(),
        RunStore::new(case.dir.join("command-runs.sqlite3")).unwrap(),
        methods::default_registry().clone(),
        CollectionScheduler::default(),
        Supervisor::new(companion::path()),
    )
    .unwrap();
    let result = case.collect();
    assert_eq!(
        result["records"]["audit"]["status"], "ok",
        "{}",
        result["records"]["audit"]
    );
    assert_eq!(
        result["records"]["audit"]["input"],
        json!({"items":["a","b","c"]})
    );
    assert_eq!(result["records"]["audit"]["value"], json!({"count":3}));
    assert!(result["records"]["audit"]["stdout_bytes"].as_u64().unwrap() > 0);
    assert_eq!(
        case.assess(&result["records"])["claims"]["count_matches"]["status"],
        "supported"
    );
}

#[test]
fn compatible_reuse_keeps_the_entire_parent_closure_and_original_times() {
    use eal_runtime::{
        catalogue::RegisterOptions,
        knowledge::EALKnowledgeBase,
        registered_assessment::RegisteredAssessmentOptions,
        service::{ReasoningService, ServiceConfig},
    };
    let case = Case::new(SOURCE, snapshot(), 30, true);
    std::fs::write(case.dir.join("case.eal"), SOURCE).unwrap();
    std::fs::write(
        case.dir.join("tools.toml"),
        "[tools.snapshot]\nkind='json_file'\nversion='1'\npath='snapshot.json'\n",
    )
    .unwrap();
    let service = std::sync::Arc::new(
        ReasoningService::new(ServiceConfig {
            workspace: case.dir.clone(),
            registry_path: Some(case.dir.join("tools.toml")),
            database_path: None,
            method_registry: methods::default_registry().clone(),
            method_executor: None,
            scheduler: CollectionScheduler::default(),
            supervisor: Supervisor::new(case.dir.join("unused-supervisor")),
            limits: ExecutionLimits::default(),
            lease_timeout: std::time::Duration::from_secs(1),
        })
        .unwrap(),
    );
    let knowledge = EALKnowledgeBase::new(service).unwrap();
    knowledge
        .register(
            "case.eal",
            RegisterOptions {
                entry_id: Some("case"),
                context: Some(&case.context),
                claims: Some(&["count_matches".into()]),
                label: None,
            },
        )
        .unwrap();
    let first = knowledge
        .assess(
            "case",
            "count_matches",
            RegisteredAssessmentOptions::default(),
        )
        .unwrap();
    let second = knowledge
        .assess(
            "case",
            "count_matches",
            RegisteredAssessmentOptions::default(),
        )
        .unwrap();
    assert_eq!(first["status"], "supported");
    assert_eq!(second["status"], "supported");
    assert_eq!(second["reused_count"], 3);
    assert_eq!(second["collected_count"], 0);
    assert_eq!(first["collection_id"], second["collection_id"]);
    assert_eq!(
        knowledge
            .service
            .store()
            .list(Some("observation"), 100)
            .unwrap()
            .len(),
        3
    );
    // Changing the concrete binding prevents replay of both source and descendants.
    let manifest = "[tools.snapshot]\nkind='json_file'\nversion='1'\npath='changed.json'\n";
    std::fs::write(case.dir.join("tools.toml"), manifest).unwrap();
    std::fs::copy(
        case.dir.join("snapshot.json"),
        case.dir.join("changed.json"),
    )
    .unwrap();
    let third = knowledge
        .assess(
            "case",
            "count_matches",
            RegisteredAssessmentOptions::default(),
        )
        .unwrap();
    assert_eq!(third["reused_count"], 0);
    assert_eq!(third["collected_count"], 3);
    assert_ne!(third["collection_id"], first["collection_id"]);
}

#[path = "common/companion.rs"]
mod companion;

#[test]
fn finite_count_distinctness_and_completeness_are_ordinary_separate_obligations() {
    for (items, complete, expected_distinct) in [
        (json!([]), true, true),
        (json!(["a"]), true, true),
        (json!(["a", "b", "c"]), true, true),
        (json!(["a", "a", "c"]), true, false),
        (json!(["a", "A"]), true, true),
        (json!(["a", "b", "c"]), false, true),
    ] {
        let count = items.as_array().unwrap().len();
        let source = SOURCE
            .replace(
                "derive object(\"count\",count(items))",
                "derive object(\"count\",count(items),\"distinct\",distinct(items))",
            )
            .replace(
                "require count == 3",
                &format!("require count == {count}\nrequire distinct == true"),
            )
            .replace(
                "[evidence audit, confirmation]",
                "[evidence measured, audit, confirmation]",
            )
            .replace(
                "contains three items",
                &format!("contains {count} distinct items"),
            );
        let case = Case::new(&source, json!({"complete":complete,"items":items}), 0, true);
        let result = case.collect();
        assert_eq!(result["records"]["audit"]["status"], "ok");
        assert_eq!(
            result["records"]["audit"]["value"],
            json!({"count":count,"distinct":expected_distinct})
        );
        assert_eq!(
            case.assess(&result["records"])["claims"]["count_matches"]["status"],
            if complete && expected_distinct {
                "supported"
            } else {
                "unsupported"
            }
        );
    }
}

#[test]
fn unresolved_parent_predicate_does_not_block_valid_bound_data() {
    for parent_value in [
        json!({"items":["a","b","c"]}),
        json!({"complete":"unknown","items":["a","b","c"]}),
    ] {
        let case = Case::new(SOURCE, parent_value, 0, true);
        let result = case.collect();
        let records = &result["records"];
        assert_eq!(records["measured"]["status"], "ok");
        assert_eq!(records["audit"]["status"], "ok", "{}", records["audit"]);
        assert_eq!(records["confirmation"]["status"], "ok");
        assert_eq!(records["audit"]["value"], json!({"count":3}));
        let verdicts = evaluation::assess_evidence_records(
            &case.program,
            &["confirmation".into()],
            records,
            Utc::now(),
            &case.context,
            Some(&case.runtime.current_binding_digests(&case.program)),
        )
        .unwrap();
        assert!(verdicts["measured"].data_available);
        assert!(!verdicts["measured"].complete);
        assert_eq!(
            verdicts["measured"].entry["availability_issues"],
            json!(["predicate_unresolved"])
        );
        assert!(verdicts["audit"].complete);
        assert!(verdicts["confirmation"].complete);
        let assessment = case.assess(records);
        assert_eq!(assessment["evidence"]["measured"]["status"], "unavailable");
        assert_eq!(assessment["claims"]["count_matches"]["status"], "supported");
        // The same incomplete predicate cannot conceal a corrupt parent payload.
        let mut corrupt = records.clone();
        corrupt["measured"]["value"]["items"] = json!([]);
        let checked = evaluation::assess_evidence_records(
            &case.program,
            &["confirmation".into()],
            &corrupt,
            Utc::now(),
            &case.context,
            Some(&case.runtime.current_binding_digests(&case.program)),
        )
        .unwrap();
        assert!(!checked["measured"].data_available);
        assert!(!checked["audit"].data_available);
        assert!(!checked["confirmation"].data_available);
    }
}

#[test]
fn native_dependency_review_and_derivation_failures_survive_packets_and_model_context() {
    use eal_runtime::{model_context::ModelContextBuilder, packets::AssessmentPacketBuilder};
    for scenario in [
        "missing",
        "missing_records",
        "stale",
        "derivation",
        "predicate",
        "expression",
        "review",
    ] {
        let mut source = SOURCE.replace(
            "[evidence audit, confirmation]",
            "[evidence measured, audit, confirmation]",
        );
        match scenario {
            "derivation" => source = source.replace("count(items)", "exact_mean(items)"),
            "predicate" => source = source.replace("require count == 3", "require absent == 3"),
            "expression" => {
                source =
                    source.replace("require count == 3", "require count / (count - count) == 3")
            }
            "review" => {
                source = source.replace("kind observation", "kind scoped_review").replace(
                    "input {}", "input {\"criterion\":\"finite inventory\",\"target\":\"count_matches\",\"scope\":{\"site\":\"bench\"}}",
                );
            }
            _ => {}
        }
        let case = Case::new(
            &source,
            snapshot(),
            if scenario == "stale" { 4000 } else { 0 },
            scenario != "missing",
        );
        let records = if scenario == "missing_records" {
            json!({})
        } else {
            case.collect()["records"].clone()
        };
        let mut assessed = case.assess(&records);
        assert_eq!(
            assessed["claims"]["count_matches"]["status"], "unsupported",
            "{scenario}: {assessed}"
        );
        let (target, expected) = match scenario {
            "predicate" | "expression" => ("audit", "predicate_unresolved"),
            "derivation" => ("audit", "invalid_observation"),
            "review" => ("measured", "invalid_observation"),
            _ => ("audit", "dependency_unavailable"),
        };
        assert!(
            assessed["evidence"][target]["availability_issues"]
                .as_array()
                .unwrap()
                .iter()
                .any(|issue| issue == expected),
            "{scenario}: {assessed}"
        );
        if scenario == "missing_records" {
            assert_eq!(
                assessed["evidence"]["measured"]["availability_issues"],
                json!(["missing_observation"])
            );
        }
        assessed["assessment_id"] = json!(
            case.runtime
                .store()
                .put("assessment", &assessed, None)
                .unwrap()
        );
        let packet = AssessmentPacketBuilder::default()
            .build(&assessed, None, None)
            .unwrap();
        assert!(
            packet["evidence"][target]["availability_issues"]
                .as_array()
                .unwrap()
                .iter()
                .any(|issue| issue == expected),
            "{scenario}: {packet}"
        );
        if scenario == "expression" {
            assert_eq!(
                packet["evidence"]["audit"]["predicate_failures"][0],
                json!({"path":"","issue":"invalid_expression"})
            );
        }
        let context = ModelContextBuilder::new()
            .build(&json!({
                "assessment_id":assessed["assessment_id"], "assessed_at":Utc::now().to_rfc3339(),
                "claim":"count_matches", "status":"unsupported", "packet":packet,
            }))
            .unwrap();
        assert_eq!(
            context["evidence"][target]["requirements"], "unresolved",
            "{scenario}: {context}"
        );
    }
}
