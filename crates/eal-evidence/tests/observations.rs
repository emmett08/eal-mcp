//! Physical reuse and immutable observation identity have separate lifetimes.
use eal_collector_contracts::AcquisitionRequest;
use eal_evidence::{execution::*, plan::*, projection::*, storage::SourceStore};
use serde_json::{Value, json};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Instant,
};
const OLD: &str = "2026-10-25T11:59:30Z";
const NEW: &str = "2026-10-25T12:02:30Z";
const NOW: &str = "2026-10-25T12:02:50Z";
struct Versioned {
    calls: AtomicUsize,
    same_measurement: bool,
}
impl SourceExecutor for Versioned {
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
        _: &'a ResolvedAcquisition,
        _: Instant,
    ) -> SourceFuture<'a, SourceObservation> {
        Box::pin(async move {
            let call = self.calls.fetch_add(1, Ordering::SeqCst);
            let repeated = self.same_measurement || call == 0;
            Ok(SourceObservation {
                value: json!(if self.same_measurement { 7 } else { 7 + call }),
                observed_at: if repeated { OLD } else { NEW }.into(),
                provenance: json!({"provider":"retained-instrument","instrument_revision":"1"}),
                coverage: CoverageStatus::CompleteEnumeration,
                metrics: SourceMetrics {
                    transport_requests: Some(1),
                    request_bytes: Some(10),
                    response_bytes: Some(100 + call as u64),
                },
            })
        })
    }
}
fn acquisition(id: &str) -> AcquisitionStep {
    AcquisitionStep {
        id: id.into(),
        source: SourceIdentity {
            provider_id: "instrument".into(),
            capability_id: "quantity".into(),
            capability_version: "1".into(),
            input_schema_sha256: "a".repeat(64),
            output_schema_sha256: "b".repeat(64),
            qualified_contract_sha256: "c".repeat(64),
            configuration_sha256: "d".repeat(64),
            authentication_scope_sha256: "e".repeat(64),
            scope: json!({"instrument":"dev"}),
            historical_interval: None,
        },
        arguments: json!({"anchor":7}),
        bindings: vec![],
        for_each: None,
        availability: SourceAvailability::Configured,
        reuse: ReusePolicy::FreshOrAcquire,
        max_age_seconds: Some(300),
        coverage_proof: None,
    }
}
fn evaluation(id: &str, step: &str) -> RequirementEvaluation {
    RequirementEvaluation {
        id: id.into(),
        requirement_id: id.into(),
        inputs: vec![OutputReference {
            step_id: step.into(),
            pointer: "".into(),
        }],
        criterion: EvaluationCriterion::Numeric {
            comparison: NumericComparison::AtMost,
            threshold: json!(10),
            value_unit: "1".into(),
            threshold_unit: "1".into(),
        },
        max_age_seconds: Some(300),
        required_coverage: RequiredCoverage::CompleteEnumeration,
        applicability: vec![],
        challenges: vec![],
    }
}
fn plan() -> EvidenceExecutionPlan {
    EvidenceExecutionPlan {
        schema: PLAN_SCHEMA.into(),
        acquisitions: vec![acquisition("read")],
        derivations: vec![],
        evaluations: vec![evaluation("quantity", "read")],
        scheduling: SchedulingPolicy::default(),
    }
}
fn runtime(source: Arc<Versioned>, store: SourceStore) -> EvidenceExecutor<Versioned> {
    EvidenceExecutor::new(source, store, ExecutionLimits::default()).unwrap()
}
#[tokio::test]
async fn refreshing_a_physical_source_retains_both_raw_observations_and_projected_references() {
    let temp = tempfile::tempdir().unwrap();
    let store = SourceStore::open(&temp.path().join("sources"), 10, 1_048_576).unwrap();
    let runtime = runtime(
        Arc::new(Versioned {
            calls: AtomicUsize::new(0),
            same_measurement: false,
        }),
        store.clone(),
    );
    let mut plan = plan();
    let first = runtime.execute(&plan, Some(NOW)).await.unwrap();
    let old = &first.evaluations[0].observation_ids[0];
    assert_eq!(first.observations[old].value, json!(7));
    plan.acquisitions[0].reuse = ReusePolicy::Refresh;
    let next = runtime.execute(&plan, Some(NOW)).await.unwrap();
    let new = &next.evaluations[0].observation_ids[0];
    assert_ne!(old, new);
    assert_eq!(
        first.evaluations[0].source_ids,
        next.evaluations[0].source_ids
    );
    assert_eq!(
        store.load_observation(old).unwrap().unwrap().value,
        json!(7)
    );
    assert_eq!(
        store.load_observation(new).unwrap().unwrap().value,
        json!(8)
    );
    assert_eq!(
        store.load_observation(old).unwrap().unwrap().observed_at,
        OLD
    );
    assert_eq!(
        store.load_observation(new).unwrap().unwrap().observed_at,
        NEW
    );
    let request = AcquisitionRequest {
        evidence_id: "quantity".into(),
        environment: "dev".into(),
        tool: "support_graph".into(),
        tool_version: "1".into(),
        input: json!({"schema":SUPPORT_ACQUISITION_SCHEMA,"case_digest":"f".repeat(64),"execution_plan_sha256":first.plan_sha256,"requirement_id":"quantity"}),
        context: json!({"engarg_export_digest":"f".repeat(64)}),
    };
    let envelope = project_requirement(&request, &first).unwrap();
    assert_eq!(envelope.value["observation_ids"], json!([old]));
    let refs = &envelope.details.unwrap()["observations"];
    assert_eq!(refs[0]["observation_id"], *old);
    assert_eq!(
        refs[0]["value_sha256"],
        first.observations[old].value_sha256
    );
    assert_eq!(refs[0]["observed_at"], OLD);
    assert_eq!(refs[0]["snapshot"], format!("observations/{old}.json"));
}
#[tokio::test]
async fn one_execution_keeps_old_and_refreshed_measurements_of_the_same_source_separate() {
    let temp = tempfile::tempdir().unwrap();
    let store = SourceStore::open(&temp.path().join("sources"), 10, 1_048_576).unwrap();
    let runtime = runtime(
        Arc::new(Versioned {
            calls: AtomicUsize::new(0),
            same_measurement: false,
        }),
        store.clone(),
    );
    let seeded = runtime.execute(&plan(), Some(NOW)).await.unwrap();
    let old_id = seeded.evaluations[0].observation_ids[0].clone();
    let mut old = acquisition("old");
    old.reuse = ReusePolicy::RetainedOnly;
    let mut strict = acquisition("strict");
    strict.max_age_seconds = Some(60);
    strict.arguments["anchor"] = Value::Null;
    strict.bindings = vec![DependencyBinding {
        role: Default::default(),
        output: OutputReference {
            step_id: "old".into(),
            pointer: "".into(),
        },
        argument_pointer: "/anchor".into(),
    }];
    let mut plan = plan();
    plan.acquisitions = vec![old, strict];
    plan.evaluations = vec![evaluation("old", "old"), evaluation("new", "strict")];
    let report = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert!(report.complete, "{:?}", report.evaluations);
    assert_eq!(report.sources.len(), 1);
    assert_eq!(report.observations.len(), 2);
    assert_eq!(report.steps["old"].value, Some(json!(7)));
    assert_eq!(report.steps["strict"].value, Some(json!(8)));
    assert_eq!(report.evaluations[0].observation_ids, vec![old_id.clone()]);
    assert_eq!(report.evaluations[1].observation_ids.len(), 2);
    assert_eq!(report.observations[&old_id].value, json!(7));
    assert_eq!(
        report
            .observations
            .values()
            .find(|record| record.value == json!(8))
            .unwrap()
            .observed_at,
        NEW
    );
    assert_eq!(report.metrics.source_requests, 1);
    for id in report.observations.keys() {
        assert!(store.load_observation(id).unwrap().is_some());
    }
}
#[tokio::test]
async fn repeated_measurement_with_different_transport_cost_keeps_the_first_snapshot_bytes() {
    let temp = tempfile::tempdir().unwrap();
    let store = SourceStore::open(&temp.path().join("sources"), 1, 1_048_576).unwrap();
    let runtime = runtime(
        Arc::new(Versioned {
            calls: AtomicUsize::new(0),
            same_measurement: true,
        }),
        store.clone(),
    );
    let mut plan = plan();
    let first = runtime.execute(&plan, Some(NOW)).await.unwrap();
    let id = first.evaluations[0].observation_ids[0].clone();
    let path = store
        .directory()
        .join("observations")
        .join(format!("{id}.json"));
    let bytes = std::fs::read(&path).unwrap();
    plan.acquisitions[0].reuse = ReusePolicy::Refresh;
    let again = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert!(again.complete);
    assert_eq!(again.evaluations[0].observation_ids, vec![id.clone()]);
    assert_eq!(std::fs::read(path).unwrap(), bytes);
    assert_eq!(
        serde_json::to_value(&again.observations[&id]).unwrap(),
        serde_json::to_value(store.load_observation(&id).unwrap().unwrap()).unwrap()
    );
    assert_eq!(again.observations[&id].metrics.response_bytes, Some(100));
    assert_eq!(again.metrics.reported_response_bytes, 101);
    assert_eq!(
        std::fs::read_dir(store.directory().join("observations"))
            .unwrap()
            .count(),
        1
    );
}
#[tokio::test]
async fn observation_capacity_and_integrity_fail_without_overwriting_retained_raw_data() {
    let temp = tempfile::tempdir().unwrap();
    let store = SourceStore::open(&temp.path().join("sources"), 1, 1_048_576).unwrap();
    let runtime = runtime(
        Arc::new(Versioned {
            calls: AtomicUsize::new(0),
            same_measurement: false,
        }),
        store.clone(),
    );
    let mut plan = plan();
    let first = runtime.execute(&plan, Some(NOW)).await.unwrap();
    let id = first.evaluations[0].observation_ids[0].clone();
    let path = store
        .directory()
        .join("observations")
        .join(format!("{id}.json"));
    let original = std::fs::read(&path).unwrap();
    plan.acquisitions[0].reuse = ReusePolicy::Refresh;
    let full = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert!(
        full.steps["read"]
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("observation capacity")
    );
    assert!(full.steps["read"].value.is_none());
    assert!(matches!(full.steps["read"].status, StepStatus::Unavailable));
    assert_eq!(full.evaluations[0].status, EvaluationStatus::Pending);
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(
        store.load_observation(&id).unwrap().unwrap().value,
        json!(7)
    );
    assert_eq!(
        std::fs::read_dir(store.directory().join("observations"))
            .unwrap()
            .count(),
        1
    );
    plan.acquisitions[0].reuse = ReusePolicy::RetainedOnly;
    let independent = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(independent.metrics.source_requests, 0);
    assert_eq!(independent.steps["read"].value, Some(json!(7)));
    assert_eq!(
        independent.evaluations[0].status,
        EvaluationStatus::Satisfied
    );
    let mut corrupted: Value = serde_json::from_slice(&original).unwrap();
    corrupted["provenance"]["instrument_revision"] = json!("forged");
    std::fs::write(&path, serde_json::to_vec(&corrupted).unwrap()).unwrap();
    assert!(store.load_observation(&id).is_err());
    plan.acquisitions[0].reuse = ReusePolicy::FreshOrAcquire;
    let invalid = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(invalid.evaluations[0].status, EvaluationStatus::Pending);
    assert_eq!(invalid.metrics.source_requests, 0);
}
