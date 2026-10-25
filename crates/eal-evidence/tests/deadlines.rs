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
                    json!(call + 7)
                },
                observed_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
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
    assert!(started.elapsed() < Duration::from_millis(400));
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
    assert!(started.elapsed() < Duration::from_millis(400));
    assert!(!report.complete);
    assert_eq!(report.evaluations[0].status, EvaluationStatus::Pending);
    assert_eq!(fixture.calls.load(Ordering::SeqCst), 1);
    holder.join().unwrap();
    let retried = runtime.execute(&plan, None).await.unwrap();
    assert_eq!(retried.metrics.source_requests, 0);
    assert_eq!(fixture.calls.load(Ordering::SeqCst), 1);
}
