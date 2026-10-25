//! Independent contracts for shared acquisition, lineage, deterministic evaluation and limits.
use eal_collector_contracts::AcquisitionRequest;
use eal_evidence::{execution::*, plan::*, projection::*, storage::SourceStore};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

const AT: &str = "2026-10-25T11:59:30Z";
const NOW: &str = "2026-10-25T12:00:00Z";
#[derive(Default)]
struct Mock {
    calls: AtomicUsize,
    fail: AtomicBool,
    active: AtomicUsize,
    peak: AtomicUsize,
    delay_ms: u64,
}
impl SourceExecutor for Mock {
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
        _deadline: Instant,
    ) -> SourceFuture<'a, SourceObservation> {
        Box::pin(async move {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
            self.peak.fetch_max(active, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(
                self.delay_ms
                    + request
                        .arguments
                        .get("delay_ms")
                        .and_then(Value::as_u64)
                        .unwrap_or(0),
            ))
            .await;
            self.active.fetch_sub(1, Ordering::SeqCst);
            if self.fail.load(Ordering::SeqCst) {
                return Err(SourceError::acquisition("independent source failure"));
            }
            let value = match request.arguments["operation"].as_str() {
                Some("search") => {
                    json!({"ids":request.arguments.get("ids").cloned().unwrap_or_else(||json!(["a","b","c"]))})
                }
                Some("detail") => {
                    let latency = match request.arguments["id"].as_str() {
                        Some("a") => 10,
                        Some("b") => 20,
                        _ => 30,
                    };
                    json!({"id":request.arguments["id"],"witness":request.arguments["id"]=="b","metrics":{"latency_ms":latency},"nested":{"samples":[request.arguments["id"]]},"attributes":{"resource_uid":if request.arguments["id"]=="b"{"resource-17"}else{"other-resource"}}})
                }
                _ => request.arguments.get("value").cloned().unwrap_or(json!(25)),
            };
            Ok(SourceObservation {
                value,
                observed_at: if request.arguments.get("current_time") == Some(&Value::Bool(true)) {
                    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)
                } else {
                    AT.into()
                },
                provenance: json!({"operator":"test-provider","method":"measurement/1"}),
                coverage: CoverageStatus::CompleteEnumeration,
                metrics: SourceMetrics::default(),
            })
        })
    }
}
fn source() -> SourceIdentity {
    SourceIdentity {
        provider_id: "provider".into(),
        capability_id: "measure".into(),
        capability_version: "1".into(),
        input_schema_sha256: "a".repeat(64),
        output_schema_sha256: "b".repeat(64),
        qualified_contract_sha256: "e".repeat(64),
        configuration_sha256: "c".repeat(64),
        authentication_scope_sha256: "d".repeat(64),
        scope: json!({"cluster":"dev"}),
        historical_interval: None,
    }
}
fn acquisition(id: &str, value: Value) -> AcquisitionStep {
    AcquisitionStep {
        id: id.into(),
        source: source(),
        arguments: json!({"value":value}),
        bindings: vec![],
        for_each: None,
        availability: SourceAvailability::Configured,
        reuse: ReusePolicy::FreshOrAcquire,
        max_age_seconds: None,
    }
}
fn reference(id: &str, pointer: &str) -> OutputReference {
    OutputReference {
        step_id: id.into(),
        pointer: pointer.into(),
    }
}
fn evaluation(id: &str, step: &str, threshold: Value) -> RequirementEvaluation {
    RequirementEvaluation {
        id: format!("evaluation-{id}"),
        requirement_id: id.into(),
        inputs: vec![reference(step, "")],
        criterion: EvaluationCriterion::Numeric {
            comparison: NumericComparison::AtMost,
            threshold,
            value_unit: "ms".into(),
            threshold_unit: "ms".into(),
        },
        max_age_seconds: Some(60),
        required_coverage: RequiredCoverage::Unspecified,
        applicability: vec![],
        challenges: vec![],
    }
}
fn plan(
    acquisitions: Vec<AcquisitionStep>,
    evaluations: Vec<RequirementEvaluation>,
) -> EvidenceExecutionPlan {
    EvidenceExecutionPlan {
        schema: PLAN_SCHEMA.into(),
        acquisitions,
        derivations: vec![],
        evaluations,
        scheduling: SchedulingPolicy::default(),
    }
}
fn runtime(mock: Arc<Mock>, store: SourceStore) -> EvidenceExecutor<Mock> {
    EvidenceExecutor::new(mock, store, ExecutionLimits::default()).unwrap()
}
fn store(temp: &tempfile::TempDir) -> SourceStore {
    SourceStore::open(&temp.path().join("sources"), 100, 1_048_576).unwrap()
}

#[tokio::test]
async fn one_measurement_serves_independent_thresholds_and_later_requirement_ids() {
    let temp = tempfile::tempdir().unwrap();
    let mock = Arc::new(Mock::default());
    let runtime = runtime(mock.clone(), store(&temp));
    let mut plan = plan(
        vec![acquisition("measure", json!(25))],
        vec![
            evaluation("strict", "measure", json!(20)),
            evaluation("equal", "measure", json!(25)),
            evaluation("loose", "measure", json!(30)),
        ],
    );
    let first = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(mock.calls.load(Ordering::SeqCst), 1);
    assert_eq!(first.metrics.generated_code_count, 0);
    assert_eq!(
        first
            .evaluations
            .iter()
            .map(|e| &e.status)
            .collect::<Vec<_>>(),
        vec![
            &EvaluationStatus::NotSatisfied,
            &EvaluationStatus::Satisfied,
            &EvaluationStatus::Satisfied
        ]
    );
    for evaluated in &first.evaluations {
        assert_eq!(evaluated.source_ids, first.evaluations[0].source_ids);
        assert_eq!(evaluated.observed_at.as_deref(), Some(AT));
    }
    plan.evaluations = vec![evaluation("new-prose-and-threshold", "measure", json!(99))];
    let second = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(second.metrics.source_requests, 0);
    assert_eq!(second.metrics.reused_sources, 1);
    assert_eq!(
        second.evaluations[0].source_ids,
        first.evaluations[0].source_ids
    );
    assert_eq!(second.evaluations[0].observed_at.as_deref(), Some(AT));
}
#[tokio::test]
async fn equivalent_concurrent_steps_and_executors_acquire_once() {
    let temp = tempfile::tempdir().unwrap();
    let shared = store(&temp);
    let mock = Arc::new(Mock {
        delay_ms: 40,
        ..Default::default()
    });
    let first = runtime(mock.clone(), shared.clone());
    let second = runtime(mock.clone(), shared);
    let plan = plan(
        vec![
            acquisition("first", json!(25)),
            acquisition("second", json!(25)),
        ],
        vec![
            evaluation("one", "first", json!(30)),
            evaluation("two", "second", json!(30)),
        ],
    );
    let (a, b) = tokio::join!(
        first.execute(&plan, Some(NOW)),
        second.execute(&plan, Some(NOW))
    );
    let (a, b) = (a.unwrap(), b.unwrap());
    assert!(a.complete && b.complete);
    assert_eq!(mock.calls.load(Ordering::SeqCst), 1);
    assert_eq!(a.evaluations[0].source_ids, a.evaluations[1].source_ids);
    assert_eq!(a.evaluations[0].source_ids, b.evaluations[1].source_ids);
}
#[tokio::test]
async fn cache_identity_changes_with_every_physical_source_boundary() {
    let temp = tempfile::tempdir().unwrap();
    let mock = Arc::new(Mock::default());
    let runtime = runtime(mock.clone(), store(&temp));
    let baseline = acquisition("measure", json!(25));
    let mutations: Vec<fn(&mut AcquisitionStep)> = vec![
        (|s| s.source.provider_id = "other-provider".into()),
        (|s| s.source.capability_version = "2".into()),
        (|s| s.source.input_schema_sha256 = "1".repeat(64)),
        (|s| s.source.output_schema_sha256 = "2".repeat(64)),
        (|s| s.source.qualified_contract_sha256 = "3".repeat(64)),
        (|s| s.source.configuration_sha256 = "4".repeat(64)),
        (|s| s.source.authentication_scope_sha256 = "5".repeat(64)),
        (|s| s.source.scope = json!({"cluster":"production"})),
        (|s| s.arguments = json!({"value":26})),
        (|s| {
            s.source.historical_interval = Some(HistoricalInterval {
                start: "2026-10-25T11:00:00Z".into(),
                end: NOW.into(),
            })
        }),
    ];
    let first = runtime
        .execute(
            &plan(
                vec![baseline.clone()],
                vec![evaluation("r", "measure", json!(100))],
            ),
            Some(NOW),
        )
        .await
        .unwrap();
    for mutate in &mutations {
        let mut step = baseline.clone();
        mutate(&mut step);
        let next = runtime
            .execute(
                &plan(vec![step], vec![evaluation("r", "measure", json!(100))]),
                Some(NOW),
            )
            .await
            .unwrap();
        assert_ne!(
            next.evaluations[0].source_ids,
            first.evaluations[0].source_ids
        );
    }
    assert_eq!(mock.calls.load(Ordering::SeqCst), mutations.len() + 1);
}
#[tokio::test]
async fn fractional_staleness_is_pending_and_retains_original_lineage_after_failure() {
    let temp = tempfile::tempdir().unwrap();
    let mock = Arc::new(Mock::default());
    let runtime = runtime(mock.clone(), store(&temp));
    let mut plan = plan(
        vec![acquisition("measure", json!(25))],
        vec![evaluation("r", "measure", json!(30))],
    );
    plan.evaluations[0].max_age_seconds = Some(30);
    let original = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert!(original.complete);
    mock.fail.store(true, Ordering::SeqCst);
    let stale = runtime
        .execute(&plan, Some("2026-10-25T12:00:00.000000001Z"))
        .await
        .unwrap();
    assert_eq!(stale.evaluations[0].status, EvaluationStatus::Pending);
    assert_eq!(
        stale.evaluations[0].source_ids,
        original.evaluations[0].source_ids
    );
    assert_eq!(stale.evaluations[0].observed_at.as_deref(), Some(AT));
    assert!(
        stale.evaluations[0]
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("stale")
    );
    // The failed source is uncertain and is not automatically called again.
    let again = runtime
        .execute(&plan, Some("2026-10-25T12:00:00.000000002Z"))
        .await
        .unwrap();
    assert_eq!(again.metrics.source_requests, 0);
    assert_eq!(mock.calls.load(Ordering::SeqCst), 2);
}
#[tokio::test]
async fn bounded_fanout_uses_real_outputs_and_does_not_establish_absence() {
    let temp = tempfile::tempdir().unwrap();
    let mock = Arc::new(Mock::default());
    let runtime = runtime(mock.clone(), store(&temp));
    let mut search = acquisition("search", Value::Null);
    search.arguments = json!({"operation":"search"});
    let mut detail = acquisition("details", Value::Null);
    detail.arguments = json!({"operation":"detail","id":null});
    detail.for_each = Some(ForEachBinding {
        output: reference("search", "/ids"),
        argument_pointer: "/id".into(),
        maximum_items: 1,
        stop_on_witness: None,
    });
    let mut eval = evaluation("r", "details", json!(0));
    eval.criterion = EvaluationCriterion::AnyTrue;
    eval.required_coverage = RequiredCoverage::Witness;
    let mut plan = plan(vec![search, detail], vec![eval]);
    plan.derivations.push(DerivationStep {
        id: "witnesses".into(),
        inputs: vec![reference("details", "")],
        operation: DerivationOperation::Extract {
            pointer: "/0/witness".into(),
        },
    });
    plan.evaluations[0].inputs = vec![reference("witnesses", "")];
    let output = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(mock.calls.load(Ordering::SeqCst), 2);
    assert!(output.steps["details"].truncated);
    assert_eq!(
        output.steps["details"].value.as_ref().unwrap()[0]["id"],
        "a"
    );
    assert_eq!(output.evaluations[0].status, EvaluationStatus::Pending);
}
#[tokio::test]
async fn explicit_witness_can_stop_detail_reads_but_cannot_skip_applicability() {
    let temp = tempfile::tempdir().unwrap();
    let mock = Arc::new(Mock::default());
    let runtime = runtime(mock.clone(), store(&temp));
    let mut search = acquisition("search", Value::Null);
    search.arguments = json!({"operation":"search"});
    let mut detail = acquisition("details", Value::Null);
    detail.arguments = json!({"operation":"detail","id":null});
    detail.for_each = Some(ForEachBinding {
        output: reference("search", "/ids"),
        argument_pointer: "/id".into(),
        maximum_items: 3,
        stop_on_witness: Some(WitnessPredicate {
            pointer: "/witness".into(),
            equals: json!(true),
        }),
    });
    let challenge = acquisition("challenge", json!(false));
    let mut evaluation = evaluation("r", "details", json!(0));
    evaluation.inputs = vec![reference("details", "/1/witness")];
    evaluation.criterion = EvaluationCriterion::AnyTrue;
    evaluation.required_coverage = RequiredCoverage::Witness;
    evaluation.challenges = vec![reference("challenge", "")];
    let output = runtime
        .execute(
            &plan(vec![search, detail, challenge], vec![evaluation]),
            Some(NOW),
        )
        .await
        .unwrap();
    assert_eq!(mock.calls.load(Ordering::SeqCst), 4);
    assert_eq!(
        output.steps["details"]
            .value
            .as_ref()
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(output.evaluations[0].status, EvaluationStatus::Pending);
    assert!(
        output.evaluations[0]
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("challenge")
    );
}
#[tokio::test]
async fn graph_is_validated_before_acquisition_and_failure_is_not_a_negative() {
    let temp = tempfile::tempdir().unwrap();
    let mock = Arc::new(Mock::default());
    let runtime = runtime(mock.clone(), store(&temp));
    let mut plan = plan(
        vec![acquisition("measure", json!(false))],
        vec![evaluation("r", "measure", json!(0))],
    );
    plan.evaluations[0].criterion = EvaluationCriterion::Equals {
        expected: json!(true),
    };
    let negative = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(
        negative.evaluations[0].status,
        EvaluationStatus::NotSatisfied
    );
    let mut broken = plan.clone();
    broken.acquisitions[0].bindings.push(DependencyBinding {
        output: reference("absent", ""),
        argument_pointer: "/value".into(),
    });
    assert!(runtime.execute(&broken, Some(NOW)).await.is_err());
    assert_eq!(mock.calls.load(Ordering::SeqCst), 1);
    plan.acquisitions[0].reuse = ReusePolicy::Refresh;
    mock.fail.store(true, Ordering::SeqCst);
    // A retained measured negative is still usable while its age fits.
    let failed_refresh = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(
        failed_refresh.evaluations[0].status,
        EvaluationStatus::NotSatisfied
    );
    plan.acquisitions[0].arguments = json!({"value":true});
    let failure = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(failure.evaluations[0].status, EvaluationStatus::Pending);
    assert!(matches!(
        failure.steps["measure"].status,
        StepStatus::Failed
    ));
}
#[tokio::test]
async fn cache_tampering_cannot_change_original_time_or_completeness() {
    let temp = tempfile::tempdir().unwrap();
    let shared = store(&temp);
    let mock = Arc::new(Mock::default());
    let runtime = runtime(mock, shared.clone());
    let plan = plan(
        vec![acquisition("measure", json!(25))],
        vec![evaluation("r", "measure", json!(30))],
    );
    let original = runtime.execute(&plan, Some(NOW)).await.unwrap();
    let key = &original.evaluations[0].source_ids[0];
    let path = shared.directory().join(format!("{key}.json"));
    let mut record: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    record["observed_at"] = json!(NOW);
    record["coverage"] = json!({"kind":"witness"});
    std::fs::write(&path, serde_json::to_vec(&record).unwrap()).unwrap();
    let tampered = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(tampered.evaluations[0].status, EvaluationStatus::Pending);
    assert!(
        tampered.steps["measure"]
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("integrity")
    );
}
#[tokio::test]
async fn concurrency_and_request_budgets_are_enforced() {
    let temp = tempfile::tempdir().unwrap();
    let mock = Arc::new(Mock {
        delay_ms: 30,
        ..Default::default()
    });
    let runtime = EvidenceExecutor::new(
        mock.clone(),
        store(&temp),
        ExecutionLimits {
            maximum_source_requests: 3,
            ..Default::default()
        },
    )
    .unwrap();
    let steps = (0..5)
        .map(|i| acquisition(&format!("step-{i}"), json!(i)))
        .collect::<Vec<_>>();
    let evaluations = (0..5)
        .map(|i| evaluation(&format!("r-{i}"), &format!("step-{i}"), json!(99)))
        .collect();
    let mut plan = plan(steps, evaluations);
    plan.scheduling.maximum_concurrent_sources = 3;
    plan.scheduling.per_provider.insert("provider".into(), 2);
    let output = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(mock.calls.load(Ordering::SeqCst), 3);
    assert_eq!(mock.peak.load(Ordering::SeqCst), 2);
    assert_eq!(output.metrics.source_requests, 3);
    assert!(!output.complete);
}
#[tokio::test]
async fn historical_gaps_and_judgement_remain_explicit_without_source_calls() {
    let temp = tempfile::tempdir().unwrap();
    let mock = Arc::new(Mock::default());
    let runtime = runtime(mock.clone(), store(&temp));
    let mut step = acquisition("history", json!(0));
    step.availability = SourceAvailability::HistoricalUnavailable {
        reason: "provider retains only current values".into(),
    };
    let mut judgement = evaluation("judgement", "history", json!(0));
    judgement.inputs.clear();
    judgement.criterion = EvaluationCriterion::Judgement {
        criterion: "independent inferential review".into(),
        scope: None,
    };
    let output = runtime
        .execute(
            &plan(
                vec![step],
                vec![evaluation("history", "history", json!(0)), judgement],
            ),
            Some(NOW),
        )
        .await
        .unwrap();
    assert_eq!(mock.calls.load(Ordering::SeqCst), 0);
    assert!(
        output
            .evaluations
            .iter()
            .all(|e| e.status == EvaluationStatus::Pending)
    );
}
#[tokio::test]
async fn exact_quantities_project_without_binary64_rounding_and_units_are_checked() {
    let temp = tempfile::tempdir().unwrap();
    let mock = Arc::new(Mock::default());
    let runtime = runtime(mock, store(&temp));
    let large: Value = serde_json::from_str("90071992547409931234567891").unwrap();
    let mut plan = plan(
        vec![acquisition("measure", json!({"provider_quantity":large}))],
        vec![evaluation(
            "r",
            "logical",
            serde_json::from_str("90071992547409931234567890").unwrap(),
        )],
    );
    plan.derivations.push(DerivationStep {
        id: "logical".into(),
        inputs: vec![reference("measure", "")],
        operation: DerivationOperation::Project {
            fields: BTreeMap::from([("latency".into(), "/provider_quantity".into())]),
        },
    });
    plan.evaluations[0].inputs = vec![reference("logical", "/latency")];
    let output = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(output.evaluations[0].status, EvaluationStatus::NotSatisfied);
    if let EvaluationCriterion::Numeric { threshold_unit, .. } = &mut plan.evaluations[0].criterion
    {
        *threshold_unit = "seconds".into();
    }
    let units = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(units.evaluations[0].status, EvaluationStatus::Pending);
}
#[tokio::test]
async fn collector_projection_pins_case_plan_and_retains_false_and_context() {
    let temp = tempfile::tempdir().unwrap();
    let mock = Arc::new(Mock::default());
    let runtime = runtime(mock, store(&temp));
    let plan = plan(
        vec![acquisition("measure", json!(25))],
        vec![evaluation("r", "measure", json!(20))],
    );
    let digest = canonical_digest(&plan).unwrap();
    let mut request = AcquisitionRequest {
        evidence_id: "ev".into(),
        environment: "dev".into(),
        tool: "support_graph".into(),
        tool_version: "1".into(),
        input: json!({"schema":SUPPORT_ACQUISITION_SCHEMA,"case_digest":"f".repeat(64),"execution_plan_sha256":digest,"requirement_id":"r"}),
        context: json!({"engarg_export_digest":"f".repeat(64),"caller":"retained"}),
    };
    validate_support_request(&request, &plan).unwrap();
    let output = runtime.execute(&plan, Some(NOW)).await.unwrap();
    let envelope = project_requirement(&request, &output).unwrap();
    assert_eq!(envelope.value["collected"], true);
    assert_eq!(envelope.value["criterion_satisfied"], false);
    assert_eq!(envelope.observed_at.as_deref(), Some(AT));
    assert_eq!(envelope.context, Some(request.context.clone()));
    assert_eq!(envelope.request, Some(request.acquisition()));
    request.input["execution_plan_sha256"] = json!("a".repeat(64));
    assert!(validate_support_request(&request, &plan).is_err());
    assert!(project_requirement(&request, &output).is_err());
}

#[tokio::test]
async fn requirement_projection_executes_only_its_dependency_closure_and_keeps_full_plan_pin() {
    let temp = tempfile::tempdir().unwrap();
    let mock = Arc::new(Mock::default());
    let runtime = runtime(mock.clone(), store(&temp));
    let plan = plan(
        vec![
            acquisition("wanted", json!(25)),
            acquisition("unrelated", json!(90)),
        ],
        vec![
            evaluation("wanted", "wanted", json!(30)),
            evaluation("unrelated", "unrelated", json!(99)),
        ],
    );
    let output = runtime
        .execute_requirement(&plan, "wanted", Some(NOW))
        .await
        .unwrap();
    assert_eq!(mock.calls.load(Ordering::SeqCst), 1);
    assert_eq!(output.plan_sha256, canonical_digest(&plan).unwrap());
    assert_eq!(output.evaluations.len(), 1);
    assert!(!output.steps.contains_key("unrelated"));
}
#[tokio::test]
async fn uncertain_acquisition_requires_explicit_reconciliation_and_cache_identity_budget_precedes_io()
 {
    let temp = tempfile::tempdir().unwrap();
    let shared = SourceStore::open(&temp.path().join("sources"), 1, 1_048_576).unwrap();
    let mock = Arc::new(Mock::default());
    mock.fail.store(true, Ordering::SeqCst);
    let runtime = runtime(mock.clone(), shared.clone());
    let step = acquisition("measure", json!(25));
    let plan = plan(
        vec![step.clone()],
        vec![evaluation("r", "measure", json!(30))],
    );
    let failed = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(failed.evaluations[0].status, EvaluationStatus::Pending);
    mock.fail.store(false, Ordering::SeqCst);
    let blocked = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(blocked.metrics.source_requests, 0);
    assert_eq!(mock.calls.load(Ordering::SeqCst), 1);
    let key = canonical_digest(&(&step.source, &step.arguments)).unwrap();
    shared.reconcile_source(&key).unwrap();
    let recovered = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert!(recovered.complete);
    assert_eq!(mock.calls.load(Ordering::SeqCst), 2);
    let mut other = plan;
    other.acquisitions[0].arguments = json!({"value":26});
    let bounded = runtime.execute(&other, Some(NOW)).await.unwrap();
    assert_eq!(bounded.evaluations[0].status, EvaluationStatus::Pending);
    assert_eq!(bounded.metrics.source_requests, 0);
    assert_eq!(mock.calls.load(Ordering::SeqCst), 2);
}
#[tokio::test]
async fn cycles_are_rejected_without_source_activity() {
    let temp = tempfile::tempdir().unwrap();
    let mock = Arc::new(Mock::default());
    let runtime = runtime(mock.clone(), store(&temp));
    let mut first = acquisition("first", json!(25));
    let mut second = acquisition("second", json!(25));
    first.bindings.push(DependencyBinding {
        output: reference("second", ""),
        argument_pointer: "/value".into(),
    });
    second.bindings.push(DependencyBinding {
        output: reference("first", ""),
        argument_pointer: "/value".into(),
    });
    assert!(
        runtime
            .execute(&plan(vec![first, second], vec![]), Some(NOW))
            .await
            .unwrap_err()
            .contains("cycle")
    );
    assert_eq!(mock.calls.load(Ordering::SeqCst), 0);
}
#[test]
fn completeness_and_statistical_sample_are_retained_as_distinct_measured_properties() {
    use eal_collector_contracts::capability::{CoverageGuarantee, LogicalStrength};
    let guarantees = vec![
        CoverageGuarantee {
            strength: LogicalStrength::CompleteEnumeration,
            population: Some("requests".into()),
            completeness_pointer: Some("/complete".into()),
            sample_size_pointer: None,
        },
        CoverageGuarantee {
            strength: LogicalStrength::StatisticalEstimate,
            population: Some("requests".into()),
            completeness_pointer: None,
            sample_size_pointer: Some("/sample_size".into()),
        },
    ];
    let both = measured_coverage(&json!({"complete":true,"sample_size":120}), &guarantees);
    assert_eq!(
        both,
        CoverageStatus::CompleteEnumerationWithStatistics { sample_size: 120 }
    );
    assert!(both.is_complete());
    assert_eq!(both.sample_size(), Some(120));
    let no_sample = measured_coverage(&json!({"complete":true,"sample_size":0}), &guarantees);
    assert_eq!(no_sample, CoverageStatus::CompleteEnumeration);
    assert_eq!(no_sample.sample_size(), None);
    let incomplete = measured_coverage(&json!({"complete":false,"sample_size":120}), &guarantees);
    assert_eq!(incomplete, CoverageStatus::Statistical { sample_size: 120 });
    assert!(!incomplete.is_complete());
}

#[tokio::test]
async fn source_reuse_preserves_literal_private_number_keys_and_arbitrary_numbers() {
    let temp = tempfile::tempdir().unwrap();
    let mock = Arc::new(Mock::default());
    let runtime = runtime(mock.clone(), store(&temp));
    let literal: Value = eal_collector_contracts::strict_json_exact(
        br#"{"$serde_json::private::Number":"literal","n":90071992547409931234567891}"#,
        Default::default(),
    )
    .unwrap();
    let mut step = acquisition("literal", literal.clone());
    step.source.scope = json!({"literal":literal});
    let mut expected = evaluation("literal", "literal", json!(0));
    expected.criterion = EvaluationCriterion::Equals {
        expected: literal.clone(),
    };
    let plan = plan(vec![step], vec![expected]);
    let first = runtime.execute(&plan, Some(NOW)).await.unwrap();
    let again = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert!(first.complete && again.complete);
    assert_eq!(again.metrics.source_requests, 0);
    assert_eq!(mock.calls.load(Ordering::SeqCst), 1);
    assert_eq!(again.sources.values().next().unwrap().value, literal);
    assert_eq!(
        again.sources.values().next().unwrap().arguments,
        first.sources.values().next().unwrap().arguments
    );
}

#[tokio::test]
async fn source_admission_io_obeys_global_and_provider_concurrency_limits() {
    #[derive(Default)]
    struct Admission {
        active: AtomicUsize,
        peak: AtomicUsize,
    }
    impl SourceExecutor for Admission {
        fn bind<'a>(
            &'a self,
            step: &'a AcquisitionStep,
            arguments: Value,
        ) -> SourceFuture<'a, ResolvedAcquisition> {
            Box::pin(async move {
                let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
                self.peak.fetch_max(active, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(15)).await;
                self.active.fetch_sub(1, Ordering::SeqCst);
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
                Ok(SourceObservation {
                    value: request.arguments["value"].clone(),
                    observed_at: AT.into(),
                    provenance: json!({"kind":"bounded-admission"}),
                    coverage: CoverageStatus::CompleteEnumeration,
                    metrics: SourceMetrics::default(),
                })
            })
        }
    }
    let temp = tempfile::tempdir().unwrap();
    let admission = Arc::new(Admission::default());
    let runtime =
        EvidenceExecutor::new(admission.clone(), store(&temp), ExecutionLimits::default()).unwrap();
    let mut steps = (0..6)
        .map(|i| acquisition(&format!("step-{i}"), json!(i)))
        .collect::<Vec<_>>();
    for (index, step) in steps.iter_mut().enumerate() {
        step.source.provider_id = format!("provider-{}", index % 2);
    }
    let evaluations = (0..6)
        .map(|i| evaluation(&format!("r-{i}"), &format!("step-{i}"), json!(99)))
        .collect();
    let mut plan = plan(steps, evaluations);
    plan.scheduling.maximum_concurrent_sources = 2;
    let output = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert!(output.complete);
    assert_eq!(admission.peak.load(Ordering::SeqCst), 2);
    let mut unavailable = plan;
    unavailable.acquisitions[0].availability = SourceAvailability::Unavailable {
        reason: "source operation has no qualified producer".into(),
    };
    let missing = runtime
        .execute_requirement(&unavailable, "r-0", Some(NOW))
        .await
        .unwrap();
    assert_eq!(missing.evaluations[0].status, EvaluationStatus::Pending);
    assert!(
        missing.evaluations[0]
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("operation is unavailable")
    );
}

#[tokio::test]
async fn concurrent_first_use_store_initialization_shares_one_integrity_key() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("first-use");
    let stores = std::thread::scope(|scope| {
        let barrier = Arc::new(std::sync::Barrier::new(16));
        let workers = (0..16)
            .map(|_| {
                let barrier = barrier.clone();
                let path = path.clone();
                scope.spawn(move || {
                    barrier.wait();
                    SourceStore::open(&path, 100, 1_048_576)
                })
            })
            .collect::<Vec<_>>();
        workers
            .into_iter()
            .map(|worker| worker.join().unwrap().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(std::fs::metadata(path.join("store.key")).unwrap().len(), 32);
    let mock = Arc::new(Mock::default());
    let plan = plan(
        vec![acquisition("measurement", json!(25))],
        vec![evaluation("r", "measurement", json!(30))],
    );
    for shared in stores {
        let output = runtime(mock.clone(), shared)
            .execute(&plan, Some(NOW))
            .await
            .unwrap();
        assert!(output.complete);
    }
    assert_eq!(mock.calls.load(Ordering::SeqCst), 1);
}
#[tokio::test]
async fn count_is_dimensionless_and_cannot_be_relabelled_as_latency() {
    let temp = tempfile::tempdir().unwrap();
    let mock = Arc::new(Mock::default());
    let runtime = runtime(mock, store(&temp));
    let mut plan = plan(
        vec![acquisition("items", json!([1, 2, 3]))],
        vec![evaluation("latency", "count", json!(10))],
    );
    plan.derivations.push(DerivationStep {
        id: "count".into(),
        inputs: vec![reference("items", "")],
        operation: DerivationOperation::Count,
    });
    let invalid = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(invalid.evaluations[0].status, EvaluationStatus::Pending);
    assert!(
        invalid.evaluations[0]
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("unit")
    );
    if let EvaluationCriterion::Numeric {
        value_unit,
        threshold_unit,
        ..
    } = &mut plan.evaluations[0].criterion
    {
        *value_unit = "1".into();
        *threshold_unit = "1".into();
    }
    let valid = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(valid.evaluations[0].status, EvaluationStatus::Satisfied);
}

#[tokio::test]
async fn acquired_fanout_projects_per_record_quantities_for_numeric_and_witness_evaluations() {
    let temp = tempfile::tempdir().unwrap();
    let mock = Arc::new(Mock::default());
    let runtime = runtime(mock.clone(), store(&temp));
    let mut search = acquisition("search", Value::Null);
    search.arguments = json!({"operation":"search"});
    let mut details = acquisition("details", Value::Null);
    details.arguments = json!({"operation":"detail","id":null});
    details.for_each = Some(ForEachBinding {
        output: reference("search", "/ids"),
        argument_pointer: "/id".into(),
        maximum_items: 3,
        stop_on_witness: None,
    });
    let mut average = evaluation("mean-latency", "mean", json!(20));
    average.required_coverage = RequiredCoverage::CompleteEnumeration;
    let mut nested = evaluation("nested-values", "projected", json!(0));
    nested.inputs = vec![reference("projected", "/samples")];
    nested.criterion = EvaluationCriterion::Equals {
        expected: json!([["a"], ["b"], ["c"]]),
    };
    nested.required_coverage = RequiredCoverage::CompleteEnumeration;
    let mut count = evaluation("count", "count", json!(3));
    if let EvaluationCriterion::Numeric {
        value_unit,
        threshold_unit,
        ..
    } = &mut count.criterion
    {
        *value_unit = "1".into();
        *threshold_unit = "1".into();
    }
    count.required_coverage = RequiredCoverage::CompleteEnumeration;
    let mut witness = evaluation("witness", "projected", json!(0));
    witness.inputs = vec![reference("projected", "/match")];
    witness.criterion = EvaluationCriterion::AnyTrue;
    witness.required_coverage = RequiredCoverage::Witness;
    let mut plan = plan(vec![search, details], vec![average, nested, count, witness]);
    plan.derivations = vec![
        DerivationStep {
            id: "projected".into(),
            inputs: vec![reference("details", "")],
            operation: DerivationOperation::Project {
                fields: BTreeMap::from([
                    ("latency".into(), "/metrics/latency_ms".into()),
                    ("samples".into(), "/nested/samples".into()),
                    ("match".into(), "/witness".into()),
                ]),
            },
        },
        DerivationStep {
            id: "mean".into(),
            inputs: vec![reference("projected", "/latency")],
            operation: DerivationOperation::Mean,
        },
        DerivationStep {
            id: "count".into(),
            inputs: vec![reference("projected", "/latency")],
            operation: DerivationOperation::Count,
        },
    ];
    let output = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert!(output.complete, "{:?}", output.evaluations);
    assert_eq!(mock.calls.load(Ordering::SeqCst), 4);
    assert!(
        output
            .evaluations
            .iter()
            .all(|evaluation| evaluation.status == EvaluationStatus::Satisfied)
    );
    let projected = &output.steps["projected"];
    assert_eq!(
        projected.value,
        Some(
            json!({"latency":[10,20,30],"samples":[["a"],["b"],["c"]],"match":[false,true,false]})
        )
    );
    assert_eq!(projected.source_ids, output.steps["details"].source_ids);
    assert_eq!(projected.source_ids.len(), 4);
    assert_eq!(projected.observed_at.as_deref(), Some(AT));
    assert_eq!(projected.coverage, CoverageStatus::CompleteEnumeration);
    assert!(!projected.truncated);
    let again = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(again.metrics.source_requests, 0);
    assert_eq!(again.steps["projected"].source_ids, projected.source_ids);
    // A current, complete empty inventory has empty quantities. It establishes
    // count zero while an average over no observations remains unavailable.
    plan.acquisitions[0].arguments["ids"] = json!([]);
    let empty = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(empty.metrics.source_requests, 1);
    assert_eq!(
        empty.steps["projected"].value,
        Some(json!({"latency":[],"samples":[],"match":[]}))
    );
    assert_eq!(
        empty.steps["projected"].source_ids,
        empty.steps["search"].source_ids
    );
    assert_eq!(empty.steps["projected"].observed_at.as_deref(), Some(AT));
    assert_eq!(
        empty.steps["projected"].coverage,
        CoverageStatus::CompleteEnumeration
    );
    assert_eq!(empty.evaluations[0].status, EvaluationStatus::Pending);
    assert_eq!(empty.evaluations[2].status, EvaluationStatus::Satisfied);
    assert_eq!(empty.evaluations[3].status, EvaluationStatus::NotSatisfied);
    // Missing fields on an acquired record never silently drop that record.
    plan.acquisitions[0]
        .arguments
        .as_object_mut()
        .unwrap()
        .remove("ids");
    if let DerivationOperation::Project { fields } = &mut plan.derivations[0].operation {
        fields.insert("missing".into(), "/absent".into());
    }
    let missing = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(missing.metrics.source_requests, 0);
    assert!(missing.steps["projected"].value.is_none());
    assert_eq!(missing.steps["projected"].source_ids.len(), 4);
    assert_eq!(missing.evaluations[0].status, EvaluationStatus::Pending);
}

#[tokio::test]
async fn scalar_resource_uid_matches_a_projected_trace_uid_collection_without_coercion() {
    let temp = tempfile::tempdir().unwrap();
    let runtime = runtime(Arc::new(Mock::default()), store(&temp));
    let resource = acquisition("resource", json!({"uid":"resource-17"}));
    let mut search = acquisition("search", Value::Null);
    search.arguments = json!({"operation":"search"});
    let mut details = acquisition("details", Value::Null);
    details.arguments = json!({"operation":"detail","id":null});
    details.for_each = Some(ForEachBinding {
        output: reference("search", "/ids"),
        argument_pointer: "/id".into(),
        maximum_items: 3,
        stop_on_witness: None,
    });
    let mut matched = evaluation("correlation", "same-resource", json!(0));
    matched.criterion = EvaluationCriterion::Equals {
        expected: json!(true),
    };
    matched.required_coverage = RequiredCoverage::Witness;
    let mut plan = plan(vec![resource, search, details], vec![matched]);
    plan.derivations = vec![
        DerivationStep {
            id: "trace-uids".into(),
            inputs: vec![reference("details", "")],
            operation: DerivationOperation::Project {
                fields: BTreeMap::from([("uid".into(), "/attributes/resource_uid".into())]),
            },
        },
        DerivationStep {
            id: "same-resource".into(),
            inputs: vec![
                reference("resource", "/uid"),
                reference("trace-uids", "/uid"),
            ],
            operation: DerivationOperation::AnyIdentityEqual,
        },
    ];
    let positive = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(positive.evaluations[0].status, EvaluationStatus::Satisfied);
    assert_eq!(positive.steps["same-resource"].value, Some(json!(true)));
    assert_eq!(
        positive.steps["same-resource"].coverage,
        CoverageStatus::Witness
    );
    assert_eq!(positive.steps["same-resource"].source_ids.len(), 5);
    assert_eq!(
        positive.steps["same-resource"].observed_at.as_deref(),
        Some(AT)
    );
    plan.acquisitions[0].arguments["value"]["uid"] = json!("absent-resource");
    let negative = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(
        negative.evaluations[0].status,
        EvaluationStatus::NotSatisfied
    );
    assert_eq!(
        negative.steps["same-resource"].coverage,
        CoverageStatus::CompleteEnumeration
    );
    plan.acquisitions[0].arguments["value"]["uid"] = json!("resource-17");
    plan.acquisitions[2]
        .for_each
        .as_mut()
        .unwrap()
        .maximum_items = 1;
    let partial = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(partial.steps["same-resource"].value, Some(json!(false)));
    assert_eq!(
        partial.steps["same-resource"].coverage,
        CoverageStatus::Unspecified
    );
    assert_eq!(partial.evaluations[0].status, EvaluationStatus::Pending);
    plan.acquisitions[2]
        .for_each
        .as_mut()
        .unwrap()
        .maximum_items = 3;
    plan.acquisitions[1].arguments["ids"] = json!([]);
    let empty = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(empty.steps["same-resource"].value, Some(json!(false)));
    assert_eq!(
        empty.steps["same-resource"].coverage,
        CoverageStatus::CompleteEnumeration
    );
    assert_eq!(empty.evaluations[0].status, EvaluationStatus::NotSatisfied);
    plan.acquisitions[1]
        .arguments
        .as_object_mut()
        .unwrap()
        .remove("ids");
    plan.derivations[1].operation = DerivationOperation::IdentityEqual;
    let wrong_shape = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(wrong_shape.evaluations[0].status, EvaluationStatus::Pending);
    assert!(
        wrong_shape.steps["same-resource"]
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("mix individual and collection")
    );
}
#[tokio::test]
async fn source_contract_age_cannot_be_relaxed_by_a_looser_consumer_and_is_not_source_identity() {
    const LATE: &str = "2026-10-25T12:02:50Z"; // Original AT is exactly 200 seconds old.
    let temp = tempfile::tempdir().unwrap();
    let mock = Arc::new(Mock::default());
    let runtime = runtime(mock.clone(), store(&temp));
    let mut loose = acquisition("loose", json!(25));
    loose.max_age_seconds = Some(300);
    let mut accepted = evaluation("loose", "loose", json!(30));
    accepted.max_age_seconds = Some(300);
    let seed = plan(vec![loose.clone()], vec![accepted.clone()]);
    let original = runtime.execute(&seed, Some(LATE)).await.unwrap();
    assert!(original.complete);
    let mut strict = loose.clone();
    strict.id = "strict".into();
    strict.max_age_seconds = Some(60);
    let mut rejected = evaluation("strict", "strict", json!(30));
    rejected.max_age_seconds = Some(300);
    let checked = runtime
        .execute(
            &plan(vec![strict.clone(), loose], vec![rejected, accepted]),
            Some(LATE),
        )
        .await
        .unwrap();
    assert_eq!(checked.evaluations[0].status, EvaluationStatus::Pending);
    assert_eq!(checked.evaluations[1].status, EvaluationStatus::Satisfied);
    assert_eq!(
        checked.evaluations[0].source_ids,
        checked.evaluations[1].source_ids
    );
    assert_eq!(
        checked.evaluations[0].source_ids,
        original.evaluations[0].source_ids
    );
    assert_eq!(checked.evaluations[0].observed_at.as_deref(), Some(AT));
    assert!(checked.steps["strict"].value.is_none());
    assert!(
        checked.steps["strict"]
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("original acquisition freshness")
    );
    assert_eq!(checked.metrics.source_requests, 1);
    // A newly read retained record is still 200 seconds old; collector execution
    // time cannot satisfy the source's 60-second observation contract.
    let fresh_temp = tempfile::tempdir().unwrap();
    let fresh_runtime =
        EvidenceExecutor::new(mock, store(&fresh_temp), ExecutionLimits::default()).unwrap();
    let mut derived = evaluation("derived", "derived", json!(30));
    derived.max_age_seconds = Some(300);
    let mut new_plan = plan(vec![strict], vec![derived]);
    new_plan.derivations = vec![DerivationStep {
        id: "derived".into(),
        inputs: vec![reference("strict", "")],
        operation: DerivationOperation::Scale {
            numerator: "1".into(),
            denominator: "1".into(),
            from_unit: "ms".into(),
            to_unit: "ms".into(),
        },
    }];
    let newly_read = fresh_runtime.execute(&new_plan, Some(LATE)).await.unwrap();
    assert_eq!(newly_read.metrics.source_requests, 1);
    assert_eq!(newly_read.evaluations[0].status, EvaluationStatus::Pending);
    assert_eq!(newly_read.sources.len(), 1);
    assert_eq!(newly_read.evaluations[0].observed_at.as_deref(), Some(AT));
    assert!(newly_read.steps["derived"].value.is_none());
}
#[tokio::test]
async fn source_age_is_rechecked_at_final_assessment_and_invalidates_derived_consumers() {
    let temp = tempfile::tempdir().unwrap();
    let runtime = runtime(Arc::new(Mock::default()), store(&temp));
    let mut fast = acquisition("fast", json!(25));
    fast.arguments["current_time"] = json!(true);
    fast.max_age_seconds = Some(1);
    let mut slow = acquisition("slow", json!(25));
    slow.arguments["current_time"] = json!(true);
    slow.arguments["delay_ms"] = json!(1200);
    let mut plan = plan(
        vec![fast, slow],
        vec![
            evaluation("fast", "derived", json!(30)),
            evaluation("slow", "slow", json!(30)),
        ],
    );
    plan.derivations = vec![DerivationStep {
        id: "derived".into(),
        inputs: vec![reference("fast", "")],
        operation: DerivationOperation::Scale {
            numerator: "1".into(),
            denominator: "1".into(),
            from_unit: "ms".into(),
            to_unit: "ms".into(),
        },
    }];
    let result = runtime.execute(&plan, None).await.unwrap();
    assert_eq!(result.metrics.source_requests, 2);
    assert_eq!(result.evaluations[0].status, EvaluationStatus::Pending);
    assert_eq!(result.evaluations[1].status, EvaluationStatus::Satisfied);
    assert!(result.steps["fast"].value.is_none());
    assert!(result.steps["derived"].value.is_none());
    assert_eq!(result.evaluations[0].source_ids.len(), 1);
    assert!(result.evaluations[0].observed_at.is_some());
}
