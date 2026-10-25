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
            if request.arguments.get("stall_id") == request.arguments.get("id")
                && request.arguments.get("stall_id").is_some()
            {
                std::future::pending::<()>().await;
            }
            if request.arguments.get("fail_id") == request.arguments.get("id")
                && request.arguments.get("fail_id").is_some()
            {
                return Err(SourceError::acquisition("detail unavailable"));
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
                    json!({"id":request.arguments["id"],"witness":request.arguments["id"]=="b","metrics":{"latency_ms":latency},"nested":{"samples":[request.arguments["id"]]},"attributes":{"resource_uid":if request.arguments["id"]=="b"{request.arguments.get("expected_uid").cloned().unwrap_or(json!("resource-17"))}else{json!("other-resource")}}})
                }
                _ => request.arguments.get("value").cloned().unwrap_or(json!(25)),
            };
            Ok(SourceObservation {
                value,
                observed_at: if let Some(at) =
                    request.arguments.get("observed_at").and_then(Value::as_str)
                {
                    at.to_owned()
                } else if request.arguments.get("current_time") == Some(&Value::Bool(true)) {
                    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)
                } else {
                    AT.into()
                },
                provenance: json!({"operator":"test-provider","method":"measurement/1"}),
                coverage: if request.arguments.get("coverage_unspecified")
                    == Some(&Value::Bool(true))
                {
                    CoverageStatus::Unspecified
                } else if request.arguments.get("coverage_witness") == Some(&Value::Bool(true)) {
                    CoverageStatus::Witness
                } else {
                    CoverageStatus::CompleteEnumeration
                },
                metrics: SourceMetrics {
                    transport_requests: Some(1),
                    request_bytes: Some(10),
                    response_bytes: Some(20),
                },
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
            equals: Some(json!(true)),
            equals_output: None,
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
        role: Default::default(),
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

fn method_envelope(method: &str, quantity: &str, payload: Value) -> Value {
    json!({"schema":"EAL/typed-input/1","method":method,"subject":"catalogue-fixture",
        "quantity":quantity,"unit":"1","scope":"fixture-v1",
        "valid_from":"2026-10-25T11:00:00Z","valid_until":"2026-10-25T13:00:00Z",
        "payload":payload})
}

#[test]
fn method_input_admission_checks_each_registered_kernel_contract_without_claiming_truth() {
    for (method, quantity, payload) in [
        (
            "deductive/1",
            "proposition",
            json!({"premises":["p",{"implies":["p","q"]}],"conclusion":"q"}),
        ),
        (
            "inductive/1",
            "probability",
            json!({"successes":8,"trials":10,"confidence":0.95}),
        ),
        (
            "abductive/1",
            "probability",
            json!({"observed":"alarm","candidates":[{"name":"fault","prior":0.5,"likelihood":0.9},{"name":"noise","prior":0.5,"likelihood":0.1}]}),
        ),
        (
            "causal/1",
            "dimensionless",
            json!({"assignment":"randomised","treatment":[2,4],"control":[0,2]}),
        ),
        (
            "counterfactual/1",
            "dimensionless",
            json!({"variables":{"x":{"intercept":1,"coefficients":{},"noise":0},"y":{"intercept":0,"coefficients":{"x":2},"noise":0}},"intervention":{"variable":"x","value":3},"outcome":"y"}),
        ),
    ] {
        let envelope = method_envelope(method, quantity, payload);
        validate_reasoning_envelope(&envelope, method).unwrap();
        let mut wrong = envelope.clone();
        wrong["method"] = json!("structured/1");
        assert!(validate_reasoning_envelope(&wrong, method).is_err());
        let mut wrong = envelope.clone();
        wrong["payload"] = json!({});
        assert!(validate_reasoning_envelope(&wrong, method).is_err());
        let mut wrong = envelope.clone();
        wrong["unit"] = json!("kPa");
        assert!(validate_reasoning_envelope(&wrong, method).is_err());
        let mut wrong = envelope;
        wrong["valid_until"] = wrong["valid_from"].clone();
        assert!(validate_reasoning_envelope(&wrong, method).is_err());
    }
    assert!(validate_reasoning_envelope(&json!({}), "unknown/1").is_err());
    assert!(validate_reasoning_envelope(&json!({}), "structured/1").is_err());
}

#[tokio::test]
async fn typed_input_projection_preserves_collected_payload_and_rejects_selector_and_report_tampering()
 {
    let temp = tempfile::tempdir().unwrap();
    let runtime = runtime(Arc::new(Mock::default()), store(&temp));
    let value = method_envelope(
        "inductive/1",
        "probability",
        json!({"successes":8,"trials":10,"confidence":0.95}),
    );
    let mut plan = plan(
        vec![acquisition(
            "sample",
            json!({"input":value,"private_sibling":17}),
        )],
        vec![],
    );
    plan.derivations.push(DerivationStep {
        id: "selected".into(),
        inputs: vec![reference("sample", "")],
        operation: DerivationOperation::Extract {
            pointer: "/input".into(),
        },
    });
    let mut selected = evaluation("kernel-input", "selected", json!(0));
    selected.criterion = EvaluationCriterion::MethodInput {
        method: "inductive/1".into(),
    };
    plan.evaluations.push(selected);
    let mut observed_trials = evaluation("observed-trials", "sample", json!(10));
    observed_trials.inputs[0].pointer = "/input/payload/trials".into();
    if let EvaluationCriterion::Numeric {
        value_unit,
        threshold_unit,
        ..
    } = &mut observed_trials.criterion
    {
        *value_unit = "1".into();
        *threshold_unit = "1".into();
    }
    plan.evaluations.push(observed_trials);
    let digest = canonical_digest(&plan).unwrap();
    let mut request = AcquisitionRequest {
        evidence_id: "sample_input".into(),
        environment: "dev".into(),
        tool: "support_graph".into(),
        tool_version: "1".into(),
        input: json!({"schema":REASONING_INPUT_SCHEMA,"case_digest":"f".repeat(64),"execution_plan_sha256":digest,
            "requirement_id":"kernel-input","output":{"step_id":"selected","pointer":""},
            "premise_bindings":[{"premise_id":"g-sample","input_pointer":"/trials","requirement_id":"observed-trials","correspondence":{"kind":"identity_value","source_input_index":0,"source_pointer":""}}]}),
        context: json!({"engarg_export_digest":"f".repeat(64)}),
    };
    validate_graph_request(&request, &plan).unwrap();
    let report = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(report.evaluations[0].status, EvaluationStatus::Satisfied);
    assert_eq!(
        report.evaluations[0].value.as_ref().unwrap()["method_result_evaluated"],
        false
    );
    let envelope = project_graph_request(&request, &plan, &report).unwrap();
    assert_eq!(envelope.value, value);
    assert_eq!(envelope.observed_at.as_deref(), Some(AT));
    assert_eq!(envelope.request, Some(request.acquisition()));
    assert_eq!(
        envelope.details.as_ref().unwrap()["reasoning_input"]["premise_truth_verified"],
        false
    );
    let original = request.clone();
    request.input["output"]["step_id"] = json!("sample");
    request.input["output"]["pointer"] = json!("/input");
    assert!(validate_graph_request(&request, &plan).is_err());
    request = original.clone();
    request.input["payload"] = json!({"successes":10});
    assert!(validate_graph_request(&request, &plan).is_err());
    request = original.clone();
    request.input["premise_bindings"][0]["input_pointer"] = json!("/absent");
    assert!(project_graph_request(&request, &plan, &report).is_err());
    request = original.clone();
    request.input["premise_bindings"][0]["requirement_id"] = json!("undeclared");
    assert!(validate_graph_request(&request, &plan).is_err());
    let mut changed = report.clone();
    changed
        .steps
        .get_mut("selected")
        .unwrap()
        .value
        .as_mut()
        .unwrap()["payload"]["successes"] = json!(10);
    assert!(project_graph_request(&original, &plan, &changed).is_err());
    let mut changed = report.clone();
    changed.observations.values_mut().next().unwrap().value["input"]["payload"]["successes"] =
        json!(10);
    assert!(project_graph_request(&original, &plan, &changed).is_err());
    let mut changed = report.clone();
    changed.evaluations[0].observed_at = Some(NOW.into());
    assert!(project_graph_request(&original, &plan, &changed).is_err());
    let mut changed = report;
    changed.evaluations[0].status = EvaluationStatus::NotSatisfied;
    assert!(project_graph_request(&original, &plan, &changed).is_err());
}

#[tokio::test]
async fn missing_model_or_evidence_keeps_method_input_pending_and_unavailable() {
    for (method, value, fail) in [
        (
            "counterfactual/1",
            method_envelope("counterfactual/1", "dimensionless", json!({})),
            false,
        ),
        (
            "inductive/1",
            method_envelope(
                "inductive/1",
                "probability",
                json!({"successes":8,"trials":10,"confidence":0.95}),
            ),
            true,
        ),
        (
            "uninstalled/1",
            method_envelope("uninstalled/1", "probability", json!({})),
            false,
        ),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let mock = Arc::new(Mock::default());
        mock.fail.store(fail, Ordering::SeqCst);
        let runtime = runtime(mock, store(&temp));
        let mut plan = plan(
            vec![acquisition("input", value)],
            vec![evaluation("input", "input", json!(0))],
        );
        plan.evaluations[0].criterion = EvaluationCriterion::MethodInput {
            method: method.into(),
        };
        let request = AcquisitionRequest {
            evidence_id: "input".into(),
            environment: "dev".into(),
            tool: "support_graph".into(),
            tool_version: "1".into(),
            input: json!({"schema":REASONING_INPUT_SCHEMA,"case_digest":"f".repeat(64),"execution_plan_sha256":canonical_digest(&plan).unwrap(),"requirement_id":"input","output":{"step_id":"input","pointer":""}}),
            context: json!({"engarg_export_digest":"f".repeat(64)}),
        };
        let report = runtime.execute(&plan, Some(NOW)).await.unwrap();
        assert_eq!(report.evaluations[0].status, EvaluationStatus::Pending);
        assert!(report.evaluations[0].diagnostic.is_some());
        assert!(project_graph_request(&request, &plan, &report).is_err());
    }
}

#[tokio::test]
async fn joined_premise_acquisition_compares_actual_values_and_preserves_the_older_clock() {
    for (emitted_successes, expected_projection) in [(8, true), (9, false)] {
        let temp = tempfile::tempdir().unwrap();
        let mock = Arc::new(Mock::default());
        let runtime = runtime(mock.clone(), store(&temp));
        let input = method_envelope(
            "inductive/1",
            "probability",
            json!({"successes":emitted_successes,"trials":10,"confidence":0.95}),
        );
        let older = "2026-10-25T11:59:00Z";
        let mut empirical = acquisition("observations", json!({"successes":8,"trials":10}));
        empirical.arguments["observed_at"] = json!(older);
        let mut plan = plan(
            vec![
                acquisition("input", input),
                empirical,
                acquisition("unrelated", json!(999)),
            ],
            vec![],
        );
        let mut admission = evaluation("input", "input", json!(0));
        admission.criterion = EvaluationCriterion::MethodInput {
            method: "inductive/1".into(),
        };
        plan.evaluations.push(admission);
        for field in ["successes", "trials"] {
            let mut criterion = evaluation(field, "observations", json!(0));
            criterion.inputs[0].pointer = format!("/{field}");
            criterion.criterion = EvaluationCriterion::Numeric {
                comparison: NumericComparison::AtLeast,
                threshold: json!(0),
                value_unit: "1".into(),
                threshold_unit: "1".into(),
            };
            plan.evaluations.push(criterion);
        }
        plan.evaluations
            .push(evaluation("unrelated", "unrelated", json!(1000)));
        let request = AcquisitionRequest {
            evidence_id: "input".into(),
            environment: "dev".into(),
            tool: "support_graph".into(),
            tool_version: "1".into(),
            input: json!({"schema":REASONING_INPUT_SCHEMA,"case_digest":"f".repeat(64),"execution_plan_sha256":canonical_digest(&plan).unwrap(),"requirement_id":"input","output":{"step_id":"input","pointer":""},
                "premise_bindings":[
                    {"premise_id":"g-sample","input_pointer":"/successes","requirement_id":"successes","correspondence":{"kind":"identity_value","source_input_index":0,"source_pointer":""}},
                    {"premise_id":"g-sample","input_pointer":"/trials","requirement_id":"trials","correspondence":{"kind":"identity_value","source_input_index":0,"source_pointer":""}}]}),
            context: json!({"engarg_export_digest":"f".repeat(64)}),
        };
        let ids = graph_request_requirements(&request, &plan).unwrap();
        let mut duplicate = request.clone();
        let copied = duplicate.input["premise_bindings"][0].clone();
        duplicate.input["premise_bindings"]
            .as_array_mut()
            .unwrap()
            .push(copied);
        assert!(validate_graph_request(&duplicate, &plan).is_err());
        assert_eq!(ids, ["input", "successes", "trials"]);
        let report = runtime
            .execute_requirements_until(
                &plan,
                &ids,
                Some(NOW),
                Instant::now() + Duration::from_secs(30),
            )
            .await
            .unwrap();
        assert_eq!(mock.calls.load(Ordering::SeqCst), 2);
        assert!(!report.steps.contains_key("unrelated"));
        assert_eq!(report.evaluations.len(), 3);
        assert!(
            report
                .evaluations
                .iter()
                .all(|evaluation| evaluation.status == EvaluationStatus::Satisfied)
        );
        let projected = project_graph_request(&request, &plan, &report);
        assert_eq!(projected.is_ok(), expected_projection);
        if let Ok(envelope) = projected {
            assert_eq!(envelope.observed_at.as_deref(), Some(older));
            assert_eq!(
                envelope.details.as_ref().unwrap()["reasoning_input"]["observation_ids"]
                    .as_array()
                    .unwrap()
                    .len(),
                2
            );
            assert_eq!(envelope.value["payload"]["successes"], 8);
            let mut rejected = report.clone();
            rejected
                .evaluations
                .iter_mut()
                .find(|evaluation| evaluation.requirement_id == "successes")
                .unwrap()
                .status = EvaluationStatus::NotSatisfied;
            assert!(project_graph_request(&request, &plan, &rejected).is_err());
            let mut absent = report;
            absent
                .evaluations
                .retain(|evaluation| evaluation.requirement_id != "successes");
            assert!(project_graph_request(&request, &plan, &absent).is_err());
        }
    }
}

#[tokio::test]
async fn accepted_deductive_premise_requires_a_collected_satisfied_independent_criterion() {
    for observed in [true, false] {
        let temp = tempfile::tempdir().unwrap();
        let runtime = runtime(Arc::new(Mock::default()), store(&temp));
        let mut plan = plan(
            vec![
                acquisition(
                    "input",
                    method_envelope(
                        "deductive/1",
                        "proposition",
                        json!({"premises":["p"],"conclusion":"p"}),
                    ),
                ),
                acquisition("observed", json!(observed)),
            ],
            vec![],
        );
        let mut admission = evaluation("input", "input", json!(0));
        admission.criterion = EvaluationCriterion::MethodInput {
            method: "deductive/1".into(),
        };
        let mut empirical = evaluation("premise", "observed", json!(0));
        empirical.criterion = EvaluationCriterion::AllTrue;
        plan.evaluations = vec![admission, empirical];
        let request = AcquisitionRequest {
            evidence_id: "input".into(),
            environment: "dev".into(),
            tool: "support_graph".into(),
            tool_version: "1".into(),
            input: json!({"schema":REASONING_INPUT_SCHEMA,"case_digest":"f".repeat(64),"execution_plan_sha256":canonical_digest(&plan).unwrap(),"requirement_id":"input","output":{"step_id":"input","pointer":""},"premise_bindings":[{"premise_id":"g-observed","input_pointer":"/premises/0","requirement_id":"premise","correspondence":{"kind":"accepted_premise"}}]}),
            context: json!({"engarg_export_digest":"f".repeat(64)}),
        };
        let ids = graph_request_requirements(&request, &plan).unwrap();
        let report = runtime
            .execute_requirements_until(
                &plan,
                &ids,
                Some(NOW),
                Instant::now() + Duration::from_secs(30),
            )
            .await
            .unwrap();
        assert_eq!(
            project_graph_request(&request, &plan, &report).is_ok(),
            observed
        );
        let mut wrong = request.clone();
        wrong.input["premise_bindings"][0]["input_pointer"] = json!("/conclusion");
        assert!(validate_graph_request(&wrong, &plan).is_err());
        let mut wrong = request;
        wrong.input["premise_bindings"][0]["requirement_id"] = json!("input");
        assert!(validate_graph_request(&wrong, &plan).is_err());
    }
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
        role: Default::default(),
        output: reference("second", ""),
        argument_pointer: "/value".into(),
    });
    second.bindings.push(DependencyBinding {
        role: Default::default(),
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
async fn whole_value_projection_addresses_root_arrays_without_implicit_fanout() {
    let temp = tempfile::tempdir().unwrap();
    let mock = Arc::new(Mock::default());
    let runtime = runtime(mock.clone(), store(&temp));
    let raw = json!([
        {"uid":"resource-17","nested":{"samples":[["a"],["b"]]}},
        {"uid":"resource-18"}
    ]);
    let mut check = evaluation("first-uid", "projected", json!(0));
    check.inputs = vec![reference("projected", "/first_uid")];
    check.criterion = EvaluationCriterion::Equals {
        expected: json!("resource-17"),
    };
    check.required_coverage = RequiredCoverage::CompleteEnumeration;
    let mut plan = plan(vec![acquisition("batch", raw.clone())], vec![check]);
    plan.derivations.push(DerivationStep {
        id: "projected".into(),
        inputs: vec![reference("batch", "")],
        operation: DerivationOperation::ProjectValue {
            fields: BTreeMap::from([
                ("first_uid".into(), "/0/uid".into()),
                ("samples".into(), "/0/nested/samples".into()),
                ("whole".into(), String::new()),
            ]),
        },
    });
    let output = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(output.evaluations[0].status, EvaluationStatus::Satisfied);
    assert_eq!(
        output.steps["projected"].value,
        Some(json!({"first_uid":"resource-17","samples":[["a"],["b"]],"whole":raw}))
    );
    assert_eq!(
        output.steps["projected"].source_ids,
        output.steps["batch"].source_ids
    );
    assert_eq!(
        output.steps["projected"].observation_ids,
        output.steps["batch"].observation_ids
    );
    assert_eq!(output.steps["projected"].observed_at.as_deref(), Some(AT));
    assert_eq!(
        output.steps["projected"].coverage,
        CoverageStatus::CompleteEnumeration
    );
    assert!(!output.steps["projected"].truncated);
    assert_eq!(mock.calls.load(Ordering::SeqCst), 1);
    // An absent whole-response pointer cannot silently become a per-record
    // lookup, a null measurement or a successful negative.
    if let DerivationOperation::ProjectValue { fields } = &mut plan.derivations[0].operation {
        fields.insert("first_uid".into(), "/2/uid".into());
    }
    let missing = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(missing.evaluations[0].status, EvaluationStatus::Pending);
    assert!(missing.steps["projected"].value.is_none());
    assert_eq!(missing.metrics.source_requests, 0);
    // Invalid pointers are rejected before source activity.
    if let DerivationOperation::ProjectValue { fields } = &mut plan.derivations[0].operation {
        fields.insert("first_uid".into(), "0/uid".into());
    }
    assert!(runtime.execute(&plan, Some(NOW)).await.is_err());
    assert_eq!(mock.calls.load(Ordering::SeqCst), 1);
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

#[tokio::test]
async fn dynamic_witness_stopping_uses_upstream_uid_without_skipping_challenges() {
    let temp = tempfile::tempdir().unwrap();
    let mock = Arc::new(Mock::default());
    let runtime = runtime(mock.clone(), store(&temp));
    let uid = acquisition("resource", json!({"uid":"resource-17"}));
    let mut search = acquisition("search", Value::Null);
    search.arguments = json!({"operation":"search"});
    let mut details = acquisition("details", Value::Null);
    details.arguments = json!({"operation":"detail","id":null});
    details.for_each = Some(ForEachBinding {
        output: reference("search", "/ids"),
        argument_pointer: "/id".into(),
        maximum_items: 3,
        stop_on_witness: Some(WitnessPredicate {
            pointer: "/attributes/resource_uid".into(),
            equals: None,
            equals_output: Some(reference("resource", "/uid")),
        }),
    });
    let challenge = acquisition("challenge", json!(false));
    let mut criterion = evaluation("correlation", "matches", json!(0));
    criterion.criterion = EvaluationCriterion::Equals {
        expected: json!(true),
    };
    criterion.required_coverage = RequiredCoverage::Witness;
    criterion.challenges = vec![reference("challenge", "")];
    let mut plan = plan(vec![uid, search, details, challenge], vec![criterion]);
    plan.derivations = vec![
        DerivationStep {
            id: "uids".into(),
            inputs: vec![reference("details", "")],
            operation: DerivationOperation::Project {
                fields: BTreeMap::from([("uid".into(), "/attributes/resource_uid".into())]),
            },
        },
        DerivationStep {
            id: "matches".into(),
            inputs: vec![reference("resource", "/uid"), reference("uids", "/uid")],
            operation: DerivationOperation::AnyIdentityEqual,
        },
    ];
    let output = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(mock.calls.load(Ordering::SeqCst), 5);
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
    assert!(output.steps["details"].truncated);
    assert_eq!(output.steps["matches"].value, Some(json!(true)));
    assert_eq!(output.evaluations[0].status, EvaluationStatus::Pending);
    assert!(
        output.evaluations[0]
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("challenge")
    );
    assert!(
        output.steps["details"]
            .source_ids
            .iter()
            .any(|id| output.steps["resource"].source_ids.contains(id))
    );
    plan.evaluations[0].challenges.clear();
    let reused = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(reused.evaluations[0].status, EvaluationStatus::Satisfied);
    assert_eq!(reused.metrics.source_requests, 0);
}

fn mixed_witness_plan(ids: Value, uid_value: Value) -> EvidenceExecutionPlan {
    let uid = acquisition("resource", json!({"uid": uid_value}));
    let mut search = acquisition("search", Value::Null);
    search.arguments = json!({"operation":"search", "ids": ids});
    let mut details = acquisition("details", Value::Null);
    details.arguments = json!({"operation":"detail","id":null,"expected_uid":uid_value});
    details.for_each = Some(ForEachBinding {
        output: reference("search", "/ids"),
        argument_pointer: "/id".into(),
        maximum_items: 3,
        stop_on_witness: Some(WitnessPredicate {
            pointer: "/attributes/resource_uid".into(),
            equals: None,
            equals_output: Some(reference("resource", "/uid")),
        }),
    });
    let mut total = evaluation("total", "count", json!(ids.as_array().unwrap().len()));
    total.criterion = EvaluationCriterion::Numeric {
        comparison: NumericComparison::Equal,
        threshold: json!(ids.as_array().unwrap().len()),
        value_unit: "1".into(),
        threshold_unit: "1".into(),
    };
    total.required_coverage = RequiredCoverage::CompleteEnumeration;
    let mut witness = evaluation("witness", "matches", json!(0));
    witness.criterion = EvaluationCriterion::Equals {
        expected: json!(true),
    };
    witness.required_coverage = RequiredCoverage::Witness;
    let mut plan = plan(vec![uid, search, details], vec![total, witness]);
    plan.derivations = vec![
        DerivationStep {
            id: "count".into(),
            inputs: vec![reference("details", "")],
            operation: DerivationOperation::Count,
        },
        DerivationStep {
            id: "uids".into(),
            inputs: vec![reference("details", "")],
            operation: DerivationOperation::Project {
                fields: BTreeMap::from([("uid".into(), "/attributes/resource_uid".into())]),
            },
        },
        DerivationStep {
            id: "matches".into(),
            inputs: vec![reference("resource", "/uid"), reference("uids", "/uid")],
            operation: DerivationOperation::AnyIdentityEqual,
        },
    ];
    plan
}

#[tokio::test]
async fn shared_witness_and_population_consumers_finish_for_first_last_and_absent_matches() {
    for (ids, witness_status) in [
        (json!(["b", "a", "c"]), EvaluationStatus::Satisfied),
        (json!(["a", "c", "b"]), EvaluationStatus::Satisfied),
        (json!(["a", "c"]), EvaluationStatus::NotSatisfied),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let mock = Arc::new(Mock::default());
        let runtime = runtime(mock.clone(), store(&temp));
        let plan = mixed_witness_plan(ids.clone(), json!("resource-17"));
        let report = runtime.execute(&plan, Some(NOW)).await.unwrap();
        assert!(report.complete);
        assert!(!report.budget_exhausted);
        assert!(!report.steps["count"].truncated);
        assert_eq!(
            report.steps["count"].value.as_ref().unwrap()["numerator"],
            ids.as_array().unwrap().len().to_string()
        );
        assert_eq!(report.evaluations[0].status, EvaluationStatus::Satisfied);
        assert_eq!(report.evaluations[1].status, witness_status);
        assert_eq!(
            mock.calls.load(Ordering::SeqCst),
            ids.as_array().unwrap().len() + 2
        );
    }
}

#[tokio::test]
async fn selecting_only_the_witness_requirement_keeps_early_stopping() {
    let temp = tempfile::tempdir().unwrap();
    let mock = Arc::new(Mock::default());
    let runtime = runtime(mock.clone(), store(&temp));
    let plan = mixed_witness_plan(json!(["b", "a", "c"]), json!("resource-17"));
    let report = runtime
        .execute_requirement(&plan, "witness", Some(NOW))
        .await
        .unwrap();
    assert!(report.complete);
    assert!(report.steps["details"].truncated);
    assert_eq!(report.evaluations[0].status, EvaluationStatus::Satisfied);
    assert_eq!(mock.calls.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn incomplete_enumeration_retains_positive_witness_and_keeps_absence_pending() {
    for (ids, witness_status) in [
        (json!(["b", "a", "c"]), EvaluationStatus::Satisfied),
        (json!(["a", "c", "b"]), EvaluationStatus::Pending),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let mock = Arc::new(Mock::default());
        let runtime = EvidenceExecutor::new(
            mock.clone(),
            store(&temp),
            ExecutionLimits {
                maximum_source_requests: 3,
                ..Default::default()
            },
        )
        .unwrap();
        let plan = mixed_witness_plan(ids, json!("resource-17"));
        let report = runtime.execute(&plan, Some(NOW)).await.unwrap();
        assert!(!report.complete);
        assert!(report.budget_exhausted);
        assert!(report.steps["details"].truncated);
        assert_eq!(report.evaluations[0].status, EvaluationStatus::Pending);
        assert_eq!(report.evaluations[1].status, witness_status);
        assert_eq!(mock.calls.load(Ordering::SeqCst), 3);
    }
}

#[tokio::test]
async fn failed_continuation_retains_witness_with_exact_large_identity() {
    let temp = tempfile::tempdir().unwrap();
    let mock = Arc::new(Mock::default());
    let runtime = runtime(mock.clone(), store(&temp));
    let identity: Value = serde_json::from_str("90071992547409931234567891").unwrap();
    let mut plan = mixed_witness_plan(json!(["b", "a", "c"]), identity.clone());
    plan.acquisitions[2].arguments["fail_id"] = json!("a");
    let report = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert!(!report.complete);
    assert!(!report.budget_exhausted);
    assert_eq!(report.evaluations[0].status, EvaluationStatus::Pending);
    assert_eq!(report.evaluations[1].status, EvaluationStatus::Satisfied);
    assert_eq!(
        report.steps["uids"].value.as_ref().unwrap()["uid"][0],
        identity
    );
    assert!(matches!(report.steps["details"].status, StepStatus::Failed));
    assert_eq!(mock.calls.load(Ordering::SeqCst), 4);
}

#[tokio::test]
async fn deadline_during_continuation_retains_witness_and_each_completed_response() {
    let temp = tempfile::tempdir().unwrap();
    let mock = Arc::new(Mock::default());
    let runtime = EvidenceExecutor::new(
        mock.clone(),
        store(&temp),
        ExecutionLimits {
            timeout: Duration::from_millis(500),
            ..Default::default()
        },
    )
    .unwrap();
    let mut plan = mixed_witness_plan(json!(["b", "a", "c"]), json!("resource-17"));
    plan.acquisitions[2].arguments["stall_id"] = json!("c");
    let report = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert!(!report.complete);
    assert!(report.budget_exhausted);
    assert_eq!(report.evaluations[0].status, EvaluationStatus::Pending);
    assert_eq!(report.evaluations[1].status, EvaluationStatus::Satisfied);
    assert_eq!(
        report.steps["details"]
            .value
            .as_ref()
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(mock.calls.load(Ordering::SeqCst), 5);
    assert_eq!(report.metrics.source_requests, 5);
    assert_eq!(report.metrics.reported_provider_requests, 4);
    assert_eq!(report.metrics.reported_request_bytes, 40);
    assert_eq!(report.metrics.reported_response_bytes, 80);
    assert_eq!(report.observations.len(), 4);
    assert!(!report.evaluations[1].observation_ids.is_empty());
}

#[tokio::test]
async fn mixed_continuation_preserves_stale_comparisons_and_material_challenges() {
    for stale in [true, false] {
        let temp = tempfile::tempdir().unwrap();
        let mock = Arc::new(Mock::default());
        let runtime = runtime(mock, store(&temp));
        let mut plan = mixed_witness_plan(json!(["b", "a", "c"]), json!("resource-17"));
        if stale {
            plan.acquisitions[0].max_age_seconds = Some(1);
        } else {
            plan.acquisitions
                .push(acquisition("challenge", json!(false)));
            plan.evaluations[1]
                .challenges
                .push(reference("challenge", ""));
        }
        let report = runtime.execute(&plan, Some(NOW)).await.unwrap();
        assert_eq!(report.evaluations[1].status, EvaluationStatus::Pending);
        assert!(!report.complete);
    }
}

#[tokio::test]
async fn a_future_comparison_cannot_hide_behind_an_older_search_timestamp() {
    let temp = tempfile::tempdir().unwrap();
    let runtime = runtime(Arc::new(Mock::default()), store(&temp));
    let mut plan = mixed_witness_plan(json!(["b", "a", "c"]), json!("resource-17"));
    plan.acquisitions[0].arguments["observed_at"] = json!("2026-10-25T12:00:01Z");
    let report = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(report.evaluations[1].status, EvaluationStatus::Pending);
    assert!(report.steps["resource"].value.is_none());
    assert!(!report.complete);
}

#[tokio::test]
async fn postdeadline_finalisation_bounds_expanding_collect_before_cloning() {
    let temp = tempfile::tempdir().unwrap();
    let runtime = EvidenceExecutor::new(
        Arc::new(Mock::default()),
        store(&temp),
        ExecutionLimits {
            timeout: Duration::from_millis(500),
            ..Default::default()
        },
    )
    .unwrap();
    let mut plan = mixed_witness_plan(json!(["b", "a", "c"]), json!("resource-17"));
    plan.acquisitions[2].arguments["stall_id"] = json!("c");
    let mut previous = "details".to_owned();
    for index in 0..40 {
        let id = format!("expand-{index:02}");
        plan.derivations.push(DerivationStep {
            id: id.clone(),
            inputs: vec![reference(&previous, ""), reference(&previous, "")],
            operation: DerivationOperation::Collect,
        });
        previous = id;
    }
    let report = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert!(report.budget_exhausted);
    assert!(!report.complete);
    assert_eq!(report.evaluations[1].status, EvaluationStatus::Satisfied);
    assert!(report.steps[&previous].value.is_none());
    assert!(matches!(
        report.steps[&previous].status,
        StepStatus::Incomplete
    ));
    assert!(report.steps.values().any(|step| {
        step.diagnostic
            .as_deref()
            .is_some_and(|diagnostic| diagnostic.contains("bounded final assessment"))
    }));
}

#[tokio::test]
async fn discovery_binding_executes_projection_but_rejects_scope_change_before_io() {
    let temp = tempfile::tempdir().unwrap();
    let mock = Arc::new(Mock::default());
    let runtime = runtime(mock.clone(), store(&temp));
    let mut discovery = acquisition("discovery", json!({"api_version":25}));
    discovery.arguments["coverage_unspecified"] = json!(true);
    let mut read = acquisition("read", Value::Null);
    read.arguments["coverage_witness"] = json!(true);
    read.source.scope = json!({"cluster":"dev","resource":"widgets"});
    read.bindings.push(DependencyBinding {
        output: reference("discovery_projection", "/api_version"),
        argument_pointer: "/value".into(),
        role: DependencyRole::Discovery {
            scope_bindings: BTreeMap::from([("/cluster".into(), "/cluster".into())]),
        },
    });
    let mut graph = plan(
        vec![discovery, read],
        vec![evaluation("named_object", "read", json!(25))],
    );
    graph.evaluations[0].required_coverage = RequiredCoverage::Witness;
    graph.derivations.push(DerivationStep {
        id: "discovery_projection".into(),
        inputs: vec![reference("discovery", "")],
        operation: DerivationOperation::Project {
            fields: BTreeMap::from([("api_version".into(), "/api_version".into())]),
        },
    });
    let result = runtime.execute(&graph, Some(NOW)).await.unwrap();
    assert_eq!(result.evaluations[0].status, EvaluationStatus::Satisfied);
    assert_eq!(result.steps["read"].value, Some(json!(25)));
    assert!(matches!(
        result.steps["discovery"].coverage,
        CoverageStatus::Unspecified
    ));
    assert!(matches!(
        result.steps["read"].coverage,
        CoverageStatus::Witness
    ));
    assert_eq!(result.steps["read"].source_ids.len(), 2);
    assert_eq!(result.steps["read"].observation_ids.len(), 2);
    assert_eq!(result.steps["read"].observed_at.as_deref(), Some(AT));
    assert_eq!(mock.calls.load(Ordering::SeqCst), 2);
    graph.acquisitions[0].source.scope["cluster"] = json!("prod");
    assert!(runtime.execute(&graph, Some(NOW)).await.is_err());
    assert_eq!(mock.calls.load(Ordering::SeqCst), 2);
}
