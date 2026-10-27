//! A typed audit cannot inherit coverage from a filtered or differently qualified source.
use eal_evidence::{execution::*, plan::*, storage::SourceStore};
use serde_json::{Value, json};
use std::{sync::Arc, time::Instant};

const NOW: &str = "2026-10-25T12:00:00Z";
const FRESH: &str = "2026-10-25T11:59:30Z";
const OLD: &str = "2026-10-25T10:00:00Z";

#[derive(Clone)]
struct Fixture {
    mode: &'static str,
}
impl SourceExecutor for Fixture {
    fn bind<'a>(
        &'a self,
        step: &'a AcquisitionStep,
        arguments: Value,
    ) -> SourceFuture<'a, ResolvedAcquisition> {
        Box::pin(async move {
            Ok(ResolvedAcquisition {
                source: step.source.clone(),
                arguments,
                admission_metrics: SourceMetrics::default(),
            })
        })
    }
    fn execute<'a>(
        &'a self,
        request: &'a ResolvedAcquisition,
        _: Instant,
    ) -> SourceFuture<'a, SourceObservation> {
        Box::pin(async move {
            if request.source.capability_id == "source" {
                let complete = !matches!(self.mode, "incomplete" | "false_claim");
                let mut qualification = json!({"qualified_contract_sha256":"e".repeat(64), "qualified_quantities":[{"name":"items","unit":"json","output_pointer":"/items"}],
                    "qualified_coverage":[{"strength":"complete_enumeration","population":"members", "population_quantity":"items", "completeness_pointer":"/complete", "sample_size_pointer":null}]});
                if self.mode == "unqualified" {
                    qualification["qualified_coverage"][0]
                        .as_object_mut()
                        .unwrap()
                        .remove("population_quantity");
                }
                let first = if self.mode == "empty_id" { "" } else { "a" };
                let long_id = "b".repeat(65);
                let second = if self.mode == "duplicate" {
                    "a"
                } else if self.mode == "long_id" {
                    long_id.as_str()
                } else {
                    "b"
                };
                let identity = if self.mode == "wrong_identity" {
                    "other"
                } else {
                    "demo"
                };
                Ok(SourceObservation {
                    value: json!({"snapshot":identity, "items":[{"id":first, "category":"x"},{"id":second, "category":"y"}], "subset":[{"id":first, "category":"x"}], "complete":complete}),
                    observed_at: if self.mode == "stale" { OLD } else { FRESH }.into(),
                    provenance: qualification,
                    coverage: if complete || self.mode == "false_claim" {
                        CoverageStatus::CompleteEnumeration
                    } else {
                        CoverageStatus::Unspecified
                    },
                    metrics: SourceMetrics {
                        transport_requests: Some(1),
                        request_bytes: Some(2),
                        response_bytes: Some(3),
                    },
                })
            } else {
                let a = &request.arguments;
                let items = a["items"].as_array().unwrap();
                let mut count = items.len() as u64;
                if self.mode == "wrong_count" {
                    count += 1;
                }
                let audit = json!({"argument_items_pointer":"/items", "argument_complete_pointer":"/complete", "argument_identity_pointer":"/snapshot", "item_id_pointer":"/id", "result_count_pointer":"/count", "result_distinct_pointer":"/distinct", "result_complete_pointer":"/complete", "result_identity_pointer":"/snapshot", "maximum_items":32, "minimum_id_chars":1, "maximum_id_chars":64, "echoes":{"/formal_input":"/typed_input"}});
                let distinct = items
                    .iter()
                    .map(|item| item["id"].as_str().unwrap())
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    == items.len();
                Ok(SourceObservation {
                    value: json!({"snapshot":a["snapshot"], "complete":a["complete"], "count":count, "distinct":distinct, "typed_input":a["formal_input"], "unrelated":3}),
                    observed_at: FRESH.into(),
                    provenance: json!({"qualified_contract_sha256":"f".repeat(64), "qualified_logical_strengths":["exact_calculation"], "qualified_identifier_audit":audit}),
                    coverage: CoverageStatus::Unspecified,
                    metrics: SourceMetrics {
                        transport_requests: Some(1),
                        request_bytes: Some(2),
                        response_bytes: Some(3),
                    },
                })
            }
        })
    }
}

fn plan() -> EvidenceExecutionPlan {
    let source = |id: &str, digest: &str| json!({"provider_id":id,"capability_id":id,"capability_version":"1","input_schema_sha256":"a".repeat(64),"output_schema_sha256":"b".repeat(64),"qualified_contract_sha256":digest.repeat(64),"configuration_sha256":"c".repeat(64),"authentication_scope_sha256":"d".repeat(64),"scope":{"snapshot":"demo"}});
    let witness = json!({"kind":"identifier_audit_v1", "source_step_id":"source-projection", "source_acquisition_id":"source", "source_qualified_contract_sha256":"e".repeat(64), "population":"members", "population_quantity":"items", "source_raw_items_pointer":"/items", "source_raw_complete_pointer":"/complete", "source_items_pointer":"/items", "source_complete_pointer":"/complete", "source_identity_pointer":"/snapshot", "argument_items_pointer":"/items", "argument_complete_pointer":"/complete", "argument_identity_pointer":"/snapshot", "result_count_pointer":"/count", "result_distinct_pointer":"/distinct", "result_complete_pointer":"/complete", "result_identity_pointer":"/snapshot", "item_id_pointer":"/id", "identity_scope_key":"snapshot", "echoes":{"/formal_input":"/typed_input"}});
    let input = json!({"schema":"eal.evidence-execution-plan/1", "acquisitions":[
        {"id":"source", "source":source("source","e"), "arguments":{"snapshot":"demo"}},
        {"id":"audit", "source":source("audit","f"), "arguments":{"snapshot":null,"items":null,"complete":null,"formal_input":{"schema":"typed"}},
         "bindings":[{"output":{"step_id":"source-projection","pointer":"/snapshot"},"argument_pointer":"/snapshot"},
                     {"output":{"step_id":"source-projection","pointer":"/items"},"argument_pointer":"/items"},
                     {"output":{"step_id":"source-projection","pointer":"/complete"},"argument_pointer":"/complete"}],"coverage_proof":witness}],
        "derivations":[
          {"id":"source-projection","inputs":[{"step_id":"source","pointer":""}],"operation":{"kind":"project_value","fields":{"snapshot":"/snapshot","items":"/items","complete":"/complete"}}},
          {"id":"audit-projection","inputs":[{"step_id":"audit","pointer":""}],"operation":{"kind":"project_value","fields":{"count":"/count","distinct":"/distinct","complete":"/complete","snapshot":"/snapshot","typed_input":"/typed_input","unrelated":"/unrelated"}}}],
        "evaluations":[{"id":"count","requirement_id":"count","inputs":[{"step_id":"audit-projection","pointer":"/count"}],"criterion":{"kind":"equals","expected":2},"max_age_seconds":60,"required_coverage":"complete_enumeration","applicability":[{"step_id":"source-projection","pointer":"/complete"},{"step_id":"audit-projection","pointer":"/complete"}],"challenges":[]},
          {"id":"distinct","requirement_id":"distinct","inputs":[{"step_id":"audit-projection","pointer":"/distinct"}],"criterion":{"kind":"equals","expected":true},"max_age_seconds":60,"required_coverage":"complete_enumeration","applicability":[{"step_id":"source-projection","pointer":"/complete"},{"step_id":"audit-projection","pointer":"/complete"}],"challenges":[]}]});
    serde_json::from_value(input).unwrap()
}
async fn execute(plan: &EvidenceExecutionPlan, mode: &'static str) -> ExecutionReport {
    let directory = tempfile::tempdir().unwrap();
    let store = SourceStore::open(&directory.path().join("sources"), 100, 1_000_000).unwrap();
    EvidenceExecutor::new(
        Arc::new(Fixture { mode }),
        store,
        ExecutionLimits::default(),
    )
    .unwrap()
    .execute(plan, Some(NOW))
    .await
    .unwrap()
}

#[tokio::test]
async fn exact_audit_preserves_only_real_complete_source_lineage() {
    let plan = plan();
    let report = execute(&plan, "ok").await;
    let result = &report.evaluations[0];
    assert!(
        matches!(result.status, EvaluationStatus::Satisfied),
        "{result:?}"
    );
    assert_eq!(result.observed_at.as_deref(), Some(FRESH));
    assert_eq!(result.observation_ids.len(), 2);
    let negative = execute(&plan, "duplicate").await;
    assert!(matches!(
        negative.evaluations[0].status,
        EvaluationStatus::Satisfied
    ));
    assert!(matches!(
        negative.evaluations[1].status,
        EvaluationStatus::NotSatisfied
    ));
    for mode in [
        "incomplete",
        "false_claim",
        "wrong_identity",
        "empty_id",
        "long_id",
        "stale",
        "unqualified",
        "wrong_count",
    ] {
        let report = execute(&plan, mode).await;
        assert!(
            matches!(report.evaluations[0].status, EvaluationStatus::Pending),
            "{mode}: {:?}",
            report.evaluations[0]
        );
        if mode == "wrong_count" {
            assert_eq!(report.metrics.reported_provider_requests, 2);
            assert_eq!(report.metrics.reported_request_bytes, 4);
            assert_eq!(report.metrics.reported_response_bytes, 6);
        }
    }
}

#[tokio::test]
async fn a_projection_of_a_subset_cannot_borrow_whole_population_coverage() {
    let mut plan = plan();
    if let DerivationOperation::ProjectValue { fields } = &mut plan.derivations[0].operation {
        fields.insert("items".into(), "/subset".into());
    }
    let report = execute(&plan, "ok").await;
    assert!(matches!(
        report.evaluations[0].status,
        EvaluationStatus::Pending
    ));
}

#[test]
fn unchecked_outputs_or_derived_children_cannot_inherit_coverage() {
    let mut unrelated = plan();
    unrelated.evaluations[0].inputs[0].pointer = "/unrelated".into();
    assert!(unrelated.validate().is_err());
    let mut plan = plan();
    plan.derivations.push(DerivationStep {
        id: "unsafe".into(),
        inputs: vec![OutputReference {
            step_id: "audit-projection".into(),
            pointer: "/unrelated".into(),
        }],
        operation: DerivationOperation::Count,
    });
    assert!(plan.validate().is_err());
}

#[test]
fn witness_comparison_cannot_launder_unverified_audit_output() {
    let mut raw = serde_json::to_value(plan()).unwrap();
    let source = raw["acquisitions"][1]["source"].clone();
    raw["acquisitions"].as_array_mut().unwrap().push(json!({
        "id":"unsafe-child", "source":source, "arguments":{"item":null},
        "for_each":{"output":{"step_id":"source-projection","pointer":"/items"},
          "argument_pointer":"/item", "maximum_items":3,
          "stop_on_witness":{"pointer":"/unrelated", "equals_output":{"step_id":"audit-projection","pointer":"/unrelated"}}}
    }));
    let changed: EvidenceExecutionPlan = serde_json::from_value(raw).unwrap();
    assert!(
        changed
            .validate()
            .unwrap_err()
            .contains("further derivation or acquisition")
    );
}

#[test]
fn mixed_discovery_lineage_cannot_enter_the_audit_coverage_proof() {
    let mut raw = serde_json::to_value(plan()).unwrap();
    let source = raw["acquisitions"][0]["source"].clone();
    raw["acquisitions"].as_array_mut().unwrap().push(json!({
        "id":"other-source", "source":source, "arguments":{"snapshot":"demo"}
    }));
    raw["acquisitions"][1]["arguments"]["metadata"] = Value::Null;
    raw["acquisitions"][1]["bindings"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "output":{"step_id":"other-source","pointer":"/snapshot"},
            "argument_pointer":"/metadata",
            "role":{"kind":"discovery","scope_bindings":{"/snapshot":"/snapshot"}}
        }));
    let mixed: EvidenceExecutionPlan = serde_json::from_value(raw).unwrap();
    assert!(
        mixed
            .validate()
            .unwrap_err()
            .contains("three exact selection bindings")
    );
}

#[tokio::test]
async fn a_forged_identifier_pointer_cannot_change_the_qualified_operation() {
    let mut plan = plan();
    if let Some(CoverageProof::IdentifierAuditV1 {
        item_id_pointer, ..
    }) = &mut plan.acquisitions[1].coverage_proof
    {
        *item_id_pointer = "/category".into();
    }
    let report = execute(&plan, "ok").await;
    assert!(matches!(
        report.evaluations[0].status,
        EvaluationStatus::Pending
    ));
}

#[tokio::test]
async fn unscoped_review_is_pending_and_cannot_validate_an_unchecked_root() {
    let mut raw = serde_json::to_value(plan()).unwrap();
    let mut review = raw["evaluations"][0].clone();
    review["id"] = json!("unscoped-review");
    review["requirement_id"] = json!("review");
    review["inputs"] = json!([{"step_id":"audit","pointer":""}]);
    review["criterion"] = json!({"kind":"judgement","criterion":"independent review","scope":null});
    raw["evaluations"]
        .as_array_mut()
        .unwrap()
        .push(review.clone());
    let pending: EvidenceExecutionPlan = serde_json::from_value(raw.clone()).unwrap();
    let report = execute(&pending, "ok").await;
    assert!(matches!(
        report.evaluations[2].status,
        EvaluationStatus::Pending
    ));
    let index = raw["evaluations"].as_array().unwrap().len() - 1;
    raw["evaluations"][index]["criterion"]["scope"] = json!({"kind":"catalogue_snapshot", "subject":"demo", "selectors":{"snapshot":"demo"}, "identity_dimensions":["snapshot"]});
    let scoped: EvidenceExecutionPlan = serde_json::from_value(raw).unwrap();
    assert!(scoped.validate().is_err());
}
