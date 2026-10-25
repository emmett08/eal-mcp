//! Deadlines stay responsive on a single-thread executor despite cache locks.
use eal_evidence::{execution::*, plan::*, storage::SourceStore};
use serde_json::{Value, json};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
struct Fixture {
    calls: AtomicUsize,
    publication: Option<(std::sync::mpsc::Sender<()>, Arc<tokio::sync::Notify>)>,
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
            let call = self.calls.fetch_add(1, Ordering::SeqCst);
            if let Some((trigger, ready)) = &self.publication {
                trigger.send(()).unwrap();
                ready.notified().await;
            }
            if request.arguments["delay"] == true {
                tokio::time::sleep(Duration::from_millis(1200)).await;
            }
            Ok(SourceObservation {
                value: if request.arguments["delay"] == true {
                    json!(0)
                } else {
                    request
                        .arguments
                        .get("value")
                        .cloned()
                        .unwrap_or_else(|| json!(call + 7))
                },
                observed_at: request
                    .arguments
                    .get("observed_at")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                    .unwrap_or_else(|| {
                        chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)
                    }),
                provenance: json!({"fixture":"deadline"}),
                coverage: CoverageStatus::CompleteEnumeration,
                metrics: SourceMetrics::default(),
            })
        })
    }
}
fn acquisition(id: &str) -> AcquisitionStep {
    AcquisitionStep {
        id: id.into(),
        source: SourceIdentity {
            provider_id: "fixture".into(),
            capability_id: if id == "delay" { "delay" } else { "data" }.into(),
            capability_version: "1".into(),
            input_schema_sha256: "a".repeat(64),
            output_schema_sha256: "b".repeat(64),
            qualified_contract_sha256: "c".repeat(64),
            configuration_sha256: "d".repeat(64),
            authentication_scope_sha256: "e".repeat(64),
            scope: json!({"cluster":"dev"}),
            historical_interval: None,
        },
        arguments: json!({"anchor":0}),
        bindings: vec![],
        for_each: None,
        availability: SourceAvailability::Configured,
        reuse: ReusePolicy::FreshOrAcquire,
        max_age_seconds: None,
    }
}
fn plan(acquisitions: Vec<AcquisitionStep>) -> EvidenceExecutionPlan {
    EvidenceExecutionPlan {
        schema: PLAN_SCHEMA.into(),
        acquisitions,
        derivations: vec![],
        evaluations: vec![RequirementEvaluation {
            id: "result".into(),
            requirement_id: "result".into(),
            inputs: vec![OutputReference {
                step_id: "data".into(),
                pointer: "".into(),
            }],
            criterion: EvaluationCriterion::Numeric {
                comparison: NumericComparison::AtMost,
                threshold: json!(99),
                value_unit: "1".into(),
                threshold_unit: "1".into(),
            },
            max_age_seconds: Some(60),
            required_coverage: RequiredCoverage::CompleteEnumeration,
            applicability: vec![],
            challenges: vec![],
        }],
        scheduling: SchedulingPolicy::default(),
    }
}
#[cfg(unix)]
fn lock(path: std::path::PathBuf) -> std::fs::File {
    use std::os::unix::fs::OpenOptionsExt;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(path)
        .unwrap();
    rustix::fs::flock(&file, rustix::fs::FlockOperation::LockExclusive).unwrap();
    file
}
#[tokio::test]
async fn dependent_cache_decisions_use_lookup_time_and_refresh_expired_records() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = Arc::new(Fixture {
        calls: AtomicUsize::new(0),
        publication: None,
    });
    let store = SourceStore::open(&temp.path().join("cache"), 10, 1_048_576).unwrap();
    let runtime =
        EvidenceExecutor::new(fixture.clone(), store, ExecutionLimits::default()).unwrap();
    let mut data = acquisition("data");
    data.max_age_seconds = Some(1);
    assert!(
        runtime
            .execute(&plan(vec![data.clone()]), None)
            .await
            .unwrap()
            .complete
    );
    let mut delay = acquisition("delay");
    delay.arguments["delay"] = json!(true);
    data.bindings = vec![DependencyBinding {
        role: Default::default(),
        output: OutputReference {
            step_id: "delay".into(),
            pointer: "".into(),
        },
        argument_pointer: "/anchor".into(),
    }];
    let report = runtime
        .execute(&plan(vec![delay, data]), None)
        .await
        .unwrap();
    assert!(report.complete, "{:?}", report.evaluations);
    assert_eq!(report.metrics.source_requests, 2);
    assert_eq!(fixture.calls.load(Ordering::SeqCst), 3);
    assert!(!report.steps["data"].reused);
}
#[cfg(unix)]
#[tokio::test]
async fn global_reservation_lock_does_not_block_the_executor_past_its_deadline() {
    let temp = tempfile::tempdir().unwrap();
    let store = SourceStore::open(&temp.path().join("cache"), 10, 1_048_576).unwrap();
    let path = store.directory().join("store.lock");
    let (ready, wait) = std::sync::mpsc::channel();
    let holder = std::thread::spawn(move || {
        let _lock = lock(path);
        ready.send(()).unwrap();
        std::thread::sleep(Duration::from_millis(700));
    });
    wait.recv().unwrap();
    let fixture = Arc::new(Fixture {
        calls: AtomicUsize::new(0),
        publication: None,
    });
    let runtime = EvidenceExecutor::new(
        fixture.clone(),
        store,
        ExecutionLimits {
            timeout: Duration::from_millis(50),
            ..Default::default()
        },
    )
    .unwrap();
    let started = Instant::now();
    let report = runtime
        .execute(&plan(vec![acquisition("data")]), None)
        .await
        .unwrap();
    let elapsed = started.elapsed();
    eprintln!("50 ms execution deadline returned after {elapsed:?}");
    // The 400 ms ceiling includes scheduler tolerance and stays below the 700 ms hold.
    assert!(elapsed < Duration::from_millis(400));
    assert!(report.budget_exhausted);
    assert!(!report.complete);
    assert_eq!(report.evaluations[0].status, EvaluationStatus::Pending);
    assert_eq!(fixture.calls.load(Ordering::SeqCst), 0);
    assert_eq!(report.metrics.source_requests, 0);
    holder.join().unwrap();
}
#[cfg(unix)]
#[tokio::test]
async fn publication_lock_timeout_never_reports_a_late_success_or_replays_the_source() {
    let temp = tempfile::tempdir().unwrap();
    let store = SourceStore::open(&temp.path().join("cache"), 10, 1_048_576).unwrap();
    let path = store.directory().join("store.lock");
    let (trigger, wait) = std::sync::mpsc::channel();
    let ready = Arc::new(tokio::sync::Notify::new());
    let notify = ready.clone();
    let holder = std::thread::spawn(move || {
        wait.recv().unwrap();
        let _lock = lock(path);
        notify.notify_one();
        std::thread::sleep(Duration::from_millis(700));
    });
    let fixture = Arc::new(Fixture {
        calls: AtomicUsize::new(0),
        publication: Some((trigger, ready)),
    });
    let runtime = EvidenceExecutor::new(
        fixture.clone(),
        store,
        ExecutionLimits {
            timeout: Duration::from_millis(50),
            ..Default::default()
        },
    )
    .unwrap();
    let plan = plan(vec![acquisition("data")]);
    let started = Instant::now();
    let report = runtime.execute(&plan, None).await.unwrap();
    let elapsed = started.elapsed();
    eprintln!("50 ms execution deadline returned after {elapsed:?}");
    // The 400 ms ceiling includes scheduler tolerance and stays below the 700 ms hold.
    assert!(elapsed < Duration::from_millis(400));
    assert!(report.budget_exhausted);
    assert!(!report.complete);
    assert_eq!(report.evaluations[0].status, EvaluationStatus::Pending);
    assert_eq!(fixture.calls.load(Ordering::SeqCst), 1);
    holder.join().unwrap();
    let retried = runtime.execute(&plan, None).await.unwrap();
    assert_eq!(retried.metrics.source_requests, 0);
    assert_eq!(fixture.calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn an_expired_initialisation_deadline_creates_no_store() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("absent");
    let result = SourceStore::open_until(&path, 10, 1_048_576, Instant::now()).await;
    assert!(result.unwrap_err().contains("deadline exhausted"));
    assert!(!path.exists());
}

#[cfg(unix)]
#[tokio::test]
async fn contended_initialisation_is_bounded_and_other_async_tasks_progress() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("cache");
    let store = SourceStore::open(&path, 10, 1_048_576).unwrap();
    let held = lock(store.directory().join("store.key"));
    let progressed = Arc::new(AtomicUsize::new(0));
    let progress = progressed.clone();
    let other_task = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(5)).await;
        progress.fetch_add(1, Ordering::SeqCst);
    });
    let started = Instant::now();
    let result =
        SourceStore::open_until(&path, 10, 1_048_576, started + Duration::from_millis(50)).await;
    let elapsed = started.elapsed();
    eprintln!("50 ms initialisation deadline returned after {elapsed:?}");
    assert!(result.unwrap_err().contains("deadline exhausted"));
    assert!(elapsed < Duration::from_millis(400));
    other_task.await.unwrap();
    assert_eq!(progressed.load(Ordering::SeqCst), 1);
    drop(held);
    SourceStore::open_until(
        &path,
        10,
        1_048_576,
        Instant::now() + Duration::from_secs(1),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn an_expired_execution_deadline_cannot_admit_acquisition_or_report_completion() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = Arc::new(Fixture {
        calls: AtomicUsize::new(0),
        publication: None,
    });
    let store = SourceStore::open(&temp.path().join("cache"), 10, 1_048_576).unwrap();
    let runtime =
        EvidenceExecutor::new(fixture.clone(), store, ExecutionLimits::default()).unwrap();
    let report = runtime
        .execute_until(&plan(vec![acquisition("data")]), None, Instant::now())
        .await
        .unwrap();
    assert!(report.budget_exhausted);
    assert!(!report.complete);
    assert_eq!(report.evaluations[0].status, EvaluationStatus::Pending);
    assert_eq!(fixture.calls.load(Ordering::SeqCst), 0);
    assert_eq!(report.metrics.source_requests, 0);
    assert!(report.sources.is_empty());
}

#[tokio::test]
async fn exact_age_limit_is_inclusive_and_future_or_fractionally_old_observations_are_pending() {
    const ASSESSMENT: &str = "2026-10-25T12:00:00Z";
    for (observed_at, expected) in [
        ("2026-10-25T11:59:00Z", EvaluationStatus::Satisfied),
        ("2026-10-25T11:58:59.999999999Z", EvaluationStatus::Pending),
        ("2026-10-25T12:00:00.000000001Z", EvaluationStatus::Pending),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let fixture = Arc::new(Fixture {
            calls: AtomicUsize::new(0),
            publication: None,
        });
        let store = SourceStore::open(&temp.path().join("cache"), 10, 1_048_576).unwrap();
        let runtime =
            EvidenceExecutor::new(fixture.clone(), store, ExecutionLimits::default()).unwrap();
        let mut step = acquisition("data");
        step.arguments["observed_at"] = json!(observed_at);
        step.max_age_seconds = Some(60);
        let plan = plan(vec![step]);
        let first = runtime.execute(&plan, Some(ASSESSMENT)).await.unwrap();
        assert_eq!(first.evaluations[0].status, expected, "{observed_at}");
        let retained_at = first
            .observations
            .values()
            .next()
            .unwrap()
            .observed_at
            .clone();
        assert_eq!(retained_at, observed_at);
        let again = runtime.execute(&plan, Some(ASSESSMENT)).await.unwrap();
        assert_eq!(again.evaluations[0].status, expected, "{observed_at}");
        assert_eq!(
            fixture.calls.load(Ordering::SeqCst),
            if expected == EvaluationStatus::Satisfied {
                1
            } else {
                2
            }
        );
    }
}

#[tokio::test]
async fn deadline_outcome_remains_incomplete_when_an_independent_criterion_is_satisfied() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = Arc::new(Fixture {
        calls: AtomicUsize::new(0),
        publication: None,
    });
    let store = SourceStore::open(&temp.path().join("cache"), 10, 1_048_576).unwrap();
    let runtime = EvidenceExecutor::new(fixture, store, ExecutionLimits::default()).unwrap();
    let mut delay = acquisition("delay");
    delay.arguments["delay"] = json!(true);
    delay.bindings.push(DependencyBinding {
        role: Default::default(),
        output: OutputReference {
            step_id: "data".into(),
            pointer: "".into(),
        },
        argument_pointer: "/anchor".into(),
    });
    let report = runtime
        .execute_until(
            &plan(vec![acquisition("data"), delay]),
            None,
            Instant::now() + Duration::from_millis(100),
        )
        .await
        .unwrap();
    assert!(report.budget_exhausted);
    assert!(!report.complete);
    assert_eq!(report.evaluations[0].status, EvaluationStatus::Satisfied);
    assert!(matches!(report.steps["data"].status, StepStatus::Measured));
    assert!(matches!(
        report.steps["delay"].status,
        StepStatus::Incomplete
    ));
}

#[tokio::test]
async fn retained_fallback_satisfies_its_criterion_without_erasing_request_budget_exhaustion() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = Arc::new(Fixture {
        calls: AtomicUsize::new(0),
        publication: None,
    });
    let store = SourceStore::open(&temp.path().join("cache"), 10, 1_048_576).unwrap();
    let prime =
        EvidenceExecutor::new(fixture.clone(), store.clone(), ExecutionLimits::default()).unwrap();
    let original = prime
        .execute(&plan(vec![acquisition("data")]), None)
        .await
        .unwrap();
    assert!(original.complete);
    let original_observation = original.evaluations[0].observation_ids.clone();
    let runtime = EvidenceExecutor::new(
        fixture.clone(),
        store,
        ExecutionLimits {
            maximum_source_requests: 1,
            ..Default::default()
        },
    )
    .unwrap();
    let mut prelude = acquisition("prelude");
    prelude.arguments["value"] = json!(0);
    let mut data = acquisition("data");
    data.reuse = ReusePolicy::Refresh;
    data.bindings.push(DependencyBinding {
        role: Default::default(),
        output: OutputReference {
            step_id: "prelude".into(),
            pointer: "".into(),
        },
        argument_pointer: "/anchor".into(),
    });
    let report = runtime
        .execute(&plan(vec![prelude, data]), None)
        .await
        .unwrap();
    assert!(report.budget_exhausted);
    assert!(!report.complete);
    assert!(report.steps["data"].reused);
    assert_eq!(report.evaluations[0].status, EvaluationStatus::Satisfied);
    assert!(
        original_observation
            .iter()
            .all(|id| report.evaluations[0].observation_ids.contains(id))
    );
    assert!(
        report.steps["prelude"]
            .observation_ids
            .iter()
            .all(|id| report.evaluations[0].observation_ids.contains(id))
    );
    assert_eq!(report.evaluations[0].observation_ids.len(), 2);
    assert_eq!(report.metrics.source_requests, 1);
    assert_eq!(fixture.calls.load(Ordering::SeqCst), 2);
    let request = serde_json::from_value(json!({
        "evidence_id":"result", "environment":"engarg_case", "tool":"support_graph", "tool_version":"1",
        "input":{"schema":"engarg.support-acquisition/1", "case_digest":"a".repeat(64),
            "execution_plan_sha256":report.plan_sha256, "requirement_id":"result"},
        "context":{"engarg_export_digest":"a".repeat(64)}
    })).unwrap();
    let envelope = eal_evidence::projection::project_requirement(&request, &report).unwrap();
    assert_eq!(envelope.value["criterion_satisfied"], true);
    assert_eq!(envelope.value["execution_complete"], false);
    assert_eq!(envelope.value["budget_exhausted"], true);
}

#[cfg(unix)]
#[tokio::test]
async fn freshness_is_recomputed_after_a_contended_source_lock() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = Arc::new(Fixture {
        calls: AtomicUsize::new(0),
        publication: None,
    });
    let store = SourceStore::open(&temp.path().join("cache"), 10, 1_048_576).unwrap();
    let runtime =
        EvidenceExecutor::new(fixture.clone(), store.clone(), ExecutionLimits::default()).unwrap();
    let mut data = acquisition("data");
    data.max_age_seconds = Some(1);
    let plan = plan(vec![data]);
    let original = runtime.execute(&plan, None).await.unwrap();
    assert!(original.complete);
    let key = original.sources.keys().next().unwrap();
    let held = lock(store.directory().join(format!("{key}.lock")));
    let holder = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(1100));
        drop(held);
    });
    let report = runtime.execute(&plan, None).await.unwrap();
    holder.join().unwrap();
    assert!(report.complete, "{:?}", report.evaluations);
    assert!(!report.steps["data"].reused);
    assert_eq!(fixture.calls.load(Ordering::SeqCst), 2);
    assert_ne!(
        original.evaluations[0].observation_ids,
        report.evaluations[0].observation_ids
    );
}
