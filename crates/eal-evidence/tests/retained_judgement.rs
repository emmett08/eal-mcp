//! A qualified source can retain a reviewer verdict; the runtime checks correspondence only.
use eal_collector_contracts::capability::SubjectScope;
use eal_evidence::{execution::*, plan::*, storage::SourceStore};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Instant,
};
const AT: &str = "2026-10-25T11:59:30Z";
const NOW: &str = "2026-10-25T12:00:00Z";
const CRITERION: &str = "Independent design review accepts the stated failure controls";
#[derive(Default)]
struct ReviewerSource {
    calls: AtomicUsize,
    change_on_refresh: bool,
}
impl SourceExecutor for ReviewerSource {
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
            let call = self.calls.fetch_add(1, Ordering::SeqCst);
            let mut value = request.arguments["record"].clone();
            let revised = self.change_on_refresh && call > 0;
            if revised {
                value["review"]["verdict"] = json!("rejected");
                value["review"]["reviewed_at"] = json!("2026-10-25T12:02:30Z");
            }
            Ok(SourceObservation {
                value,
                observed_at: if revised {
                    "2026-10-25T12:02:30Z".into()
                } else {
                    request.arguments["source_time"]
                        .as_str()
                        .unwrap_or(AT)
                        .into()
                },
                provenance: json!({"authority":"independent-review-board","method":"retained-review/1","qualified_support_types":[request.arguments["source_qualification"].as_str().unwrap_or("judgement")],"qualified_logical_strengths":["judgement"]}),
                coverage: CoverageStatus::Unspecified,
                metrics: SourceMetrics::default(),
            })
        })
    }
}
fn scope() -> SubjectScope {
    SubjectScope {
        kind: "service_design".into(),
        subject: "Catalog sync controller, revision abc123".into(),
        selectors: BTreeMap::from([
            ("cluster".into(), json!("dev")),
            ("revision".into(), json!("abc123")),
        ]),
        identity_dimensions: vec!["cluster".into(), "revision".into()],
    }
}
fn record(verdict: JudgementVerdict) -> RetainedJudgement {
    RetainedJudgement {
        schema: RETAINED_JUDGEMENT_SCHEMA.into(),
        criterion: CRITERION.into(),
        scope: scope(),
        reviewer_identity: "independent-review-board/reviewer-17".into(),
        reviewed_at: AT.into(),
        verdict,
        reasons: vec![
            "The retained review describes tested failure controls and records its limitations"
                .into(),
        ],
        provenance: json!({"review_reference":"review-board/design-17","revision":"abc123"}),
    }
}
fn plan(value: Value) -> EvidenceExecutionPlan {
    EvidenceExecutionPlan {
        schema: PLAN_SCHEMA.into(),
        acquisitions: vec![AcquisitionStep {
            id: "review-source".into(),
            source: SourceIdentity {
                provider_id: "review-board".into(),
                capability_id: "retained-review".into(),
                capability_version: "1".into(),
                input_schema_sha256: "a".repeat(64),
                output_schema_sha256: "b".repeat(64),
                qualified_contract_sha256: "c".repeat(64),
                configuration_sha256: "d".repeat(64),
                authentication_scope_sha256: "e".repeat(64),
                scope: json!({"cluster":"dev","revision":"abc123"}),
                historical_interval: None,
            },
            arguments: json!({"record":value}),
            bindings: vec![],
            for_each: None,
            availability: SourceAvailability::Configured,
            reuse: ReusePolicy::FreshOrAcquire,
            max_age_seconds: None,
            coverage_proof: None,
        }],
        derivations: vec![],
        evaluations: vec![RequirementEvaluation {
            id: "review-evaluation".into(),
            requirement_id: "design-review".into(),
            inputs: vec![OutputReference {
                step_id: "review-source".into(),
                pointer: "/review".into(),
            }],
            criterion: EvaluationCriterion::Judgement {
                criterion: CRITERION.into(),
                scope: Some(scope()),
            },
            max_age_seconds: Some(60),
            required_coverage: RequiredCoverage::Unspecified,
            applicability: vec![],
            challenges: vec![],
        }],
        scheduling: SchedulingPolicy::default(),
    }
}
fn runtime(
    source: Arc<ReviewerSource>,
    temp: &tempfile::TempDir,
) -> EvidenceExecutor<ReviewerSource> {
    EvidenceExecutor::new(
        source,
        SourceStore::open(&temp.path().join("sources"), 100, 1_048_576).unwrap(),
        ExecutionLimits::default(),
    )
    .unwrap()
}
#[tokio::test]
async fn supported_rejected_and_inconclusive_records_preserve_reviewer_reasons_age_and_lineage() {
    let temp = tempfile::tempdir().unwrap();
    let source = Arc::new(ReviewerSource::default());
    let runtime = runtime(source.clone(), &temp);
    for (verdict, status) in [
        (JudgementVerdict::Supported, EvaluationStatus::Satisfied),
        (JudgementVerdict::Rejected, EvaluationStatus::NotSatisfied),
        (JudgementVerdict::Inconclusive, EvaluationStatus::Pending),
    ] {
        let retained = record(verdict);
        let plan = plan(json!({"review":retained}));
        let report = runtime.execute(&plan, Some(NOW)).await.unwrap();
        let evaluation = &report.evaluations[0];
        assert_eq!(evaluation.status, status);
        assert_eq!(evaluation.observed_at.as_deref(), Some(AT));
        assert_eq!(evaluation.source_ids.len(), 1);
        let value = evaluation.value.as_ref().unwrap();
        assert_eq!(value["retained_judgement"], json!(retained));
        assert_eq!(value["judgement_record_correspondence_only"], true);
        let again = runtime.execute(&plan, Some(NOW)).await.unwrap();
        assert_eq!(again.metrics.source_requests, 0);
        assert_eq!(again.evaluations[0].source_ids, evaluation.source_ids);
        assert_eq!(again.evaluations[0].observed_at, evaluation.observed_at);
    }
    assert_eq!(source.calls.load(Ordering::SeqCst), 3);
}
#[tokio::test]
async fn wrong_scope_criterion_time_or_incomplete_reviewer_records_remain_pending() {
    let temp = tempfile::tempdir().unwrap();
    let runtime = runtime(Arc::new(ReviewerSource::default()), &temp);
    let original = json!(record(JudgementVerdict::Supported));
    for mutate in [
        |r: &mut Value| r["criterion"] = json!("different criterion"),
        |r: &mut Value| r["scope"]["selectors"]["revision"] = json!("other"),
        |r: &mut Value| r["reviewer_identity"] = json!(""),
        |r: &mut Value| r["reasons"] = json!([]),
        |r: &mut Value| r["provenance"] = json!({}),
        |r: &mut Value| r["reviewed_at"] = json!("2026-10-25T11:59:31Z"),
        |r: &mut Value| r["verdict"] = json!("unrecognized"),
    ] {
        let mut value = original.clone();
        mutate(&mut value);
        let report = runtime
            .execute(&plan(json!({"review":value})), Some(NOW))
            .await
            .unwrap();
        assert_eq!(report.evaluations[0].status, EvaluationStatus::Pending);
    }
    let mut other = plan(json!({"review":original}));
    other.acquisitions[0].source.scope["revision"] = json!("other");
    let result = runtime.execute(&other, Some(NOW)).await.unwrap();
    assert_eq!(result.evaluations[0].status, EvaluationStatus::Pending);
    assert!(
        result.evaluations[0]
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("source identity")
    );
}
#[tokio::test]
async fn legacy_missing_records_stale_records_and_unresolved_challenges_do_not_become_judgements() {
    let temp = tempfile::tempdir().unwrap();
    let source = Arc::new(ReviewerSource::default());
    let runtime = runtime(source.clone(), &temp);
    let mut plan = plan(json!({"review":record(JudgementVerdict::Supported)}));
    let stale = runtime
        .execute(&plan, Some("2026-10-25T12:01:00Z"))
        .await
        .unwrap();
    assert_eq!(stale.evaluations[0].status, EvaluationStatus::Pending);
    if let EvaluationCriterion::Judgement { scope, .. } = &mut plan.evaluations[0].criterion {
        *scope = None;
    }
    let legacy = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(legacy.evaluations[0].status, EvaluationStatus::Pending);
    plan.evaluations[0].criterion = EvaluationCriterion::Judgement {
        criterion: CRITERION.into(),
        scope: Some(scope()),
    };
    plan.evaluations[0].challenges = vec![OutputReference {
        step_id: "review-source".into(),
        pointer: "/challenge_passed".into(),
    }];
    plan.acquisitions[0].arguments["record"]["challenge_passed"] = json!(false);
    let challenged = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(challenged.evaluations[0].status, EvaluationStatus::Pending);
    plan.acquisitions[0].availability = SourceAvailability::Unavailable {
        reason: "review record has not been retained".into(),
    };
    let missing = runtime.execute(&plan, Some(NOW)).await.unwrap();
    assert_eq!(missing.evaluations[0].status, EvaluationStatus::Pending);
    assert_eq!(missing.metrics.source_requests, 0);
}

#[tokio::test]
async fn an_observation_source_or_assembled_record_cannot_supply_retained_reviewer_authority() {
    let temp = tempfile::tempdir().unwrap();
    let runtime = runtime(Arc::new(ReviewerSource::default()), &temp);
    let mut unqualified = plan(json!({"review":record(JudgementVerdict::Supported)}));
    unqualified.acquisitions[0].arguments["source_qualification"] = json!("observation");
    let result = runtime.execute(&unqualified, Some(NOW)).await.unwrap();
    assert_eq!(result.evaluations[0].status, EvaluationStatus::Pending);
    assert!(
        result.evaluations[0]
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("not qualified")
    );
    let retained = json!(record(JudgementVerdict::Supported));
    let parts = retained
        .as_object()
        .unwrap()
        .iter()
        .map(|(key, value)| (format!("raw_{key}"), value.clone()))
        .collect::<serde_json::Map<_, _>>();
    let mut assembled = plan(Value::Object(parts));
    assembled.derivations = vec![DerivationStep {
        id: "assembled".into(),
        inputs: vec![OutputReference {
            step_id: "review-source".into(),
            pointer: "".into(),
        }],
        operation: DerivationOperation::Project {
            fields: retained
                .as_object()
                .unwrap()
                .keys()
                .map(|key| (key.clone(), format!("/raw_{key}")))
                .collect(),
        },
    }];
    assembled.evaluations[0].inputs = vec![OutputReference {
        step_id: "assembled".into(),
        pointer: "".into(),
    }];
    let result = runtime.execute(&assembled, Some(NOW)).await.unwrap();
    assert_eq!(result.evaluations[0].status, EvaluationStatus::Pending);
    assert!(
        result.evaluations[0]
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("absent from its qualified source records")
    );
}

#[tokio::test]
async fn revised_judgement_on_the_same_source_does_not_replace_the_old_evaluations_record() {
    const LATE: &str = "2026-10-25T12:02:50Z";
    let temp = tempfile::tempdir().unwrap();
    let runtime = runtime(
        Arc::new(ReviewerSource {
            change_on_refresh: true,
            ..Default::default()
        }),
        &temp,
    );
    let mut seed = plan(json!({"review":record(JudgementVerdict::Supported)}));
    seed.evaluations[0].max_age_seconds = Some(300);
    let first = runtime.execute(&seed, Some(LATE)).await.unwrap();
    assert_eq!(first.evaluations[0].status, EvaluationStatus::Satisfied);
    let mut old = seed.acquisitions[0].clone();
    old.id = "old-review".into();
    old.max_age_seconds = Some(300);
    old.reuse = ReusePolicy::RetainedOnly;
    let mut new = seed.acquisitions[0].clone();
    new.id = "new-review".into();
    new.max_age_seconds = Some(60);
    new.arguments["record"] = Value::Null;
    new.bindings = vec![DependencyBinding {
        role: Default::default(),
        output: OutputReference {
            step_id: "old-review".into(),
            pointer: "".into(),
        },
        argument_pointer: "/record".into(),
    }];
    let mut old_eval = seed.evaluations[0].clone();
    old_eval.id = "old-evaluation".into();
    old_eval.requirement_id = "old-review".into();
    old_eval.inputs[0].step_id = "old-review".into();
    let mut new_eval = old_eval.clone();
    new_eval.id = "new-evaluation".into();
    new_eval.requirement_id = "new-review".into();
    new_eval.inputs[0].step_id = "new-review".into();
    seed.acquisitions = vec![old, new];
    seed.evaluations = vec![old_eval, new_eval];
    let report = runtime.execute(&seed, Some(LATE)).await.unwrap();
    assert!(report.complete, "{:?}", report.evaluations);
    assert_eq!(report.sources.len(), 1);
    assert_eq!(report.observations.len(), 2);
    assert_eq!(report.evaluations[0].status, EvaluationStatus::Satisfied);
    assert_eq!(report.evaluations[1].status, EvaluationStatus::NotSatisfied);
    assert_eq!(
        report.evaluations[0].value.as_ref().unwrap()["retained_judgement"]["verdict"],
        "supported"
    );
    assert_eq!(
        report.evaluations[1].value.as_ref().unwrap()["retained_judgement"]["verdict"],
        "rejected"
    );
    assert_eq!(report.evaluations[0].observation_ids.len(), 1);
    assert_eq!(report.evaluations[1].observation_ids.len(), 2);
    assert_eq!(
        report.evaluations[0].source_ids,
        report.evaluations[1].source_ids
    );
}
