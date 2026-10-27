//! Independent application checks through real collection, durable records and packets.
#![cfg(unix)]
use eal_core::{ExecutionLimits, methods};
use eal_runtime::{
    catalogue::{RegisterOptions, WorkspaceKnowledgeCatalogue},
    model_context::ModelContextBuilder,
    process::Supervisor,
    registered_assessment::{RegisteredAssessmentHost, RegisteredAssessmentOptions, ReusePolicy},
    scheduler::CollectionScheduler,
    service::{ReasoningService, ServiceConfig},
};
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    fs,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

static NEXT: AtomicUsize = AtomicUsize::new(0);
const SOURCE: &str = r#"language "EAL/3"
environment lab {
  require "site" == "bench"
}
tool reader {
  version "1"
}
evidence quick {
  tool reader
  kind test
  environment lab
  max_age 5
  require passed == true
}
evidence steady {
  tool reader
  kind test
  environment lab
  max_age 1000
  require passed == true
}
reasoning measured {
  method "structured/1"
  rationale "Synthetic predicates supply bounded support."
}
claim outcome {
  statement "Both synthetic predicates hold."
  environment lab
}
claim hidden {
  statement "This claim is not selected."
  environment lab
}
argument support = [evidence quick, steady] via measured => outcome
"#;

struct Fixture {
    path: PathBuf,
    service: Arc<ReasoningService>,
    catalogue: Arc<WorkspaceKnowledgeCatalogue>,
    host: RegisteredAssessmentHost,
}
impl Fixture {
    fn new(passed: bool) -> Self {
        let path = std::env::temp_dir().join(format!(
            "eal-app-adversarial-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        fs::write(path.join("case.eal"), SOURCE).unwrap();
        fs::write(
            path.join("observation.json"),
            json!({
                "value":{"passed":passed,"private":"raw-tool-secret"},
                "observed_at":"2040-01-01T00:00:00Z",
                "context":{"site":"bench","private":"operator-context-secret"},
                "request":{"tool":"reader","tool_version":"1","input":{},
                    "context":{"site":"bench","private":"operator-context-secret"}}
            })
            .to_string(),
        )
        .unwrap();
        fs::write(
            path.join("tools.toml"),
            "[tools.reader]\nkind='json_file'\nversion='1'\npath='observation.json'\n",
        )
        .unwrap();
        let service = Arc::new(
            ReasoningService::new(ServiceConfig {
                workspace: path.clone(),
                registry_path: Some(path.join("tools.toml")),
                database_path: Some(path.join("runs.sqlite3")),
                method_registry: methods::default_registry().clone(),
                method_executor: None,
                scheduler: CollectionScheduler::new(2).unwrap(),
                supervisor: Supervisor::new(companion::path()),
                limits: ExecutionLimits::default(),
                lease_timeout: Duration::from_secs(3),
            })
            .unwrap(),
        );
        let catalogue = Arc::new(WorkspaceKnowledgeCatalogue::new(Arc::clone(&service)).unwrap());
        catalogue
            .register(
                "case.eal",
                RegisterOptions {
                    entry_id: Some("synthetic"),
                    context: Some(&json!({"site":"bench","private":"operator-context-secret"})),
                    claims: Some(&["outcome".to_owned()]),
                    label: Some("Synthetic predicates"),
                },
            )
            .unwrap();
        let host =
            RegisteredAssessmentHost::new(Arc::clone(&service), Arc::clone(&catalogue)).unwrap();
        Self {
            path,
            service,
            catalogue,
            host,
        }
    }
    fn assess(&self, now: &str) -> Value {
        self.host
            .assess(
                "synthetic",
                "outcome",
                RegisteredAssessmentOptions {
                    now: Some(now),
                    ..Default::default()
                },
            )
            .unwrap()
    }
    fn collection(&self, assessment: &Value) -> Value {
        self.service
            .store()
            .get(
                assessment["collection_id"].as_str().unwrap(),
                Some("collection"),
            )
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn negative_reuse_rebinds_only_source_and_never_refreshes_the_observation_clock() {
    let fixture = Fixture::new(false);
    let first = fixture.assess("2040-01-01T00:00:01Z");
    assert_eq!(
        (
            first["collected_count"].as_u64(),
            first["reused_count"].as_u64()
        ),
        (Some(2), Some(0))
    );
    assert_eq!(first["status"], "unsupported");
    let original = fixture.collection(&first);
    let original_run = original["records"]["steady"]["run_id"].as_str().unwrap();
    let original_stored = fixture
        .service
        .store()
        .get(original_run, Some("observation"))
        .unwrap();

    fs::write(
        fixture.path.join("case.eal"),
        SOURCE.replace(
            "Both synthetic predicates hold.",
            "Both synthetic predicates hold in this revised synthetic case.",
        ),
    )
    .unwrap();
    let revised = fixture.assess("2040-01-01T00:00:04Z");
    assert_ne!(first["source_digest"], revised["source_digest"]);
    assert_eq!(
        (
            revised["collected_count"].as_u64(),
            revised["reused_count"].as_u64()
        ),
        (Some(0), Some(2))
    );
    assert_eq!(revised["status"], "unsupported");
    let rebound = fixture.collection(&revised);
    assert_eq!(
        rebound["records"]["steady"]["reused_from_run_id"],
        original_run
    );
    assert_eq!(
        rebound["records"]["steady"]["collected_at"],
        original["records"]["steady"]["collected_at"]
    );
    assert_eq!(
        rebound["records"]["steady"]["source_digest"],
        revised["source_digest"]
    );
    assert_eq!(
        fixture
            .service
            .store()
            .get(original_run, Some("observation"))
            .unwrap(),
        original_stored
    );
    assert_eq!(
        revised["packet"]["evidence"]["steady"]["observed_at"],
        "2040-01-01T00:00:00Z"
    );
    assert_eq!(
        revised["packet"]["evidence"]["steady"]["age_seconds"].as_f64(),
        Some(4.0)
    );
    assert_eq!(
        revised["packet"]["evidence"]["quick"]["availability_issues"],
        json!(["predicate_not_met"])
    );
    let context = ModelContextBuilder::new().build(&revised).unwrap();
    assert_eq!(context["evidence"]["quick"]["requirements"], "not_met");
    assert!(context["evidence"]["quick"].get("observation_id").is_none());
    assert!(
        context["claims"]["outcome"]
            .get("observation_ids")
            .is_none()
    );
    assert_eq!(
        context["claims"]["outcome"]["evidence_ids"],
        json!(["quick", "steady"])
    );
    for value in [&first, &revised, &context] {
        let text = value.to_string();
        assert!(!text.contains("raw-tool-secret"));
        assert!(!text.contains("operator-context-secret"));
    }

    let expired = fixture.assess("2040-01-01T00:00:07Z");
    assert_eq!(
        (
            expired["collected_count"].as_u64(),
            expired["reused_count"].as_u64()
        ),
        (Some(1), Some(1))
    );
    assert_eq!(
        expired["packet"]["evidence"]["quick"]["age_status"],
        "stale"
    );
    assert_eq!(
        expired["packet"]["evidence"]["quick"]["age_seconds"].as_f64(),
        Some(7.0)
    );
    let old_history = fixture
        .catalogue
        .runs("synthetic", first["source_digest"].as_str(), 100)
        .unwrap();
    assert_eq!(old_history["assessments"].as_array().unwrap().len(), 1);
    assert_eq!(
        old_history["assessments"][0]["assessment_id"],
        first["assessment_id"]
    );
    let current = fixture.catalogue.get("synthetic", false).unwrap();
    assert_eq!(current["source_digest"], revised["source_digest"]);
    let old = fixture
        .catalogue
        .get_revision(
            "synthetic",
            first["source_digest"].as_str().unwrap(),
            None,
            true,
        )
        .unwrap();
    assert_eq!(old["current"], false);
    assert_eq!(old["source"], SOURCE);
}

#[test]
fn current_tool_binding_and_registered_context_bound_reuse() {
    let fixture = Fixture::new(true);
    let first = fixture.assess("2040-01-01T00:00:01Z");
    let before = fixture.collection(&first);
    fs::copy(
        fixture.path.join("observation.json"),
        fixture.path.join("replacement.json"),
    )
    .unwrap();
    fs::write(
        fixture.path.join("tools.toml"),
        "[tools.reader]\nkind='json_file'\nversion='1'\npath='replacement.json'\n",
    )
    .unwrap();
    let changed = fixture.assess("2040-01-01T00:00:02Z");
    assert_eq!(
        (
            changed["collected_count"].as_u64(),
            changed["reused_count"].as_u64()
        ),
        (Some(2), Some(0))
    );
    assert_ne!(
        fixture.collection(&changed)["records"]["steady"]["tool_binding_digest"],
        before["records"]["steady"]["tool_binding_digest"]
    );
    let out_of_scope = fixture
        .host
        .assess(
            "synthetic",
            "outcome",
            RegisteredAssessmentOptions {
                now: Some("2040-01-01T00:00:02Z"),
                context: Some(&json!({"site":"elsewhere"})),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(out_of_scope["status"], "out_of_scope");
    assert_eq!(out_of_scope["reused_count"], 0);
    let fresh = fixture
        .host
        .assess(
            "synthetic",
            "outcome",
            RegisteredAssessmentOptions {
                now: Some("2040-01-01T00:00:02Z"),
                reuse: ReusePolicy::Fresh,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        (
            fresh["collected_count"].as_u64(),
            fresh["reused_count"].as_u64()
        ),
        (Some(2), Some(0))
    );
    let observation_count = fixture
        .service
        .store()
        .list(Some("observation"), 100)
        .unwrap()
        .len();
    assert!(
        fixture
            .host
            .assess(
                "synthetic",
                "hidden",
                RegisteredAssessmentOptions::default()
            )
            .is_err()
    );
    assert_eq!(
        fixture
            .service
            .store()
            .list(Some("observation"), 100)
            .unwrap()
            .len(),
        observation_count
    );
}

#[test]
fn reuse_is_rechecked_after_partial_acquisition_before_the_mixed_collection_commits() {
    let fixture = Fixture::new(true);
    fixture.assess("2040-01-01T00:00:01Z");
    let times = Arc::new(Mutex::new(VecDeque::from([
        "2040-01-01T00:00:07Z".to_owned(),
        "2040-01-01T00:16:41Z".to_owned(),
        "2040-01-01T00:16:41Z".to_owned(),
    ])));
    let clock_times = Arc::clone(&times);
    let host = RegisteredAssessmentHost::with_clock(
        Arc::clone(&fixture.service),
        Arc::clone(&fixture.catalogue),
        Arc::new(move || {
            clock_times
                .lock()
                .unwrap()
                .pop_front()
                .expect("unexpected extra clock read")
        }),
    )
    .unwrap();
    let result = host
        .assess("synthetic", "outcome", Default::default())
        .unwrap();
    assert_eq!(
        (
            result["collected_count"].as_u64(),
            result["reused_count"].as_u64()
        ),
        (Some(2), Some(0))
    );
    assert!(times.lock().unwrap().is_empty());
    assert_eq!(
        fixture.collection(&result)["records"]
            .as_object()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(result["status"], "unsupported");
    assert_eq!(
        result["packet"]["evidence"]["steady"]["age_seconds"].as_f64(),
        Some(1001.0)
    );
}

#[test]
fn a_binding_change_during_the_reuse_recheck_forces_recollection() {
    let fixture = Fixture::new(true);
    let first = fixture.assess("2040-01-01T00:00:01Z");
    let original = fixture.collection(&first);
    fs::copy(
        fixture.path.join("observation.json"),
        fixture.path.join("replacement.json"),
    )
    .unwrap();
    let registry = fixture.path.join("tools.toml");
    let clock_calls = Arc::new(AtomicUsize::new(0));
    let calls = Arc::clone(&clock_calls);
    let host = RegisteredAssessmentHost::with_clock(
        Arc::clone(&fixture.service),
        Arc::clone(&fixture.catalogue),
        Arc::new(move || {
            if calls.fetch_add(1, Ordering::SeqCst) == 1 {
                fs::write(
                    &registry,
                    "[tools.reader]\nkind='json_file'\nversion='1'\npath='replacement.json'\n",
                )
                .unwrap();
            }
            "2040-01-01T00:00:04Z".into()
        }),
    )
    .unwrap();
    let result = host
        .assess("synthetic", "outcome", Default::default())
        .unwrap();
    assert_eq!(clock_calls.load(Ordering::SeqCst), 3);
    assert_eq!(
        (
            result["collected_count"].as_u64(),
            result["reused_count"].as_u64()
        ),
        (Some(2), Some(0))
    );
    let final_collection = fixture.collection(&result);
    for name in ["quick", "steady"] {
        assert!(
            final_collection["records"][name]
                .get("reused_from_run_id")
                .is_none()
        );
        assert_ne!(
            final_collection["records"][name]["tool_binding_digest"],
            original["records"][name]["tool_binding_digest"]
        );
    }
}

#[test]
fn a_future_observation_cannot_become_reusable_by_rebinding_it_at_an_earlier_assessment() {
    let fixture = Fixture::new(true);
    assert_eq!(
        fixture.assess("2040-01-01T00:00:01Z")["status"],
        "supported"
    );
    let earlier = fixture.assess("2039-12-31T23:59:59Z");
    assert_eq!(
        (
            earlier["collected_count"].as_u64(),
            earlier["reused_count"].as_u64()
        ),
        (Some(2), Some(0))
    );
    assert_eq!(earlier["status"], "unsupported");
    assert_eq!(
        earlier["packet"]["evidence"]["steady"]["age_status"],
        "future"
    );
    assert_eq!(
        earlier["packet"]["evidence"]["steady"]["age_seconds"].as_f64(),
        Some(-1.0)
    );
    assert_eq!(
        ModelContextBuilder::new().build(&earlier).unwrap()["evidence"]["steady"]["requirements"],
        "unresolved"
    );
}

#[path = "common/companion.rs"]
mod companion;
