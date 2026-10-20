#![cfg(unix)]
use eal_core::{ExecutionLimits, methods::default_registry};
use eal_runtime::{
    catalogue::{RegisterOptions, WorkspaceKnowledgeCatalogue},
    knowledge::{EALKnowledgeBase, ModelContextAdapter},
    process::Supervisor,
    registered_assessment::{RegisteredAssessmentHost, RegisteredAssessmentOptions, ReusePolicy},
    scheduler::CollectionScheduler,
    service::{ReasoningService, ServiceConfig},
};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

const SOURCE: &str = "language \"EAL/3\"\nenvironment lab {\n require site == \"bench\"\n}\ntool reader {\n version \"1\"\n}\nevidence quick {\n tool reader\n kind test\n environment lab\n max_age 5\n require ok == true\n}\nevidence steady {\n tool reader\n kind test\n environment lab\n max_age 1000\n require ok == true\n}\nreasoning measured {\n method \"structured/1\"\n rationale \"Both observations support this bounded claim.\"\n}\nclaim works {\n statement \"Both checks passed.\"\n environment lab\n}\nclaim other {\n statement \"An unrelated claim.\"\n environment lab\n}\nargument result = [evidence quick, steady] via measured => works\n";
const FIRST: &str = "2040-01-01T00:00:01Z";
const SECOND: &str = "2040-01-01T00:00:07Z";

struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("eal-registered-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        let out = Self(path);
        out.write("case.eal", SOURCE);
        out.write(
            "tools.toml",
            "[tools.reader]\nkind='json_file'\nversion='1'\npath='observation.json'\n",
        );
        out.observation("2040-01-01T00:00:00Z", true);
        out
    }
    fn write(&self, path: &str, text: &str) {
        std::fs::write(self.0.join(path), text).unwrap()
    }
    fn observation(&self, time: &str, passed: bool) {
        self.write("observation.json",&json!({"observed_at":time,"context":{"site":"bench"},
        "request":{"tool":"reader","tool_version":"1","input":{},"context":{"site":"bench"}},"value":{"ok":passed,"secret":"collector-secret"}}).to_string())
    }
    fn service(&self) -> Arc<ReasoningService> {
        Arc::new(
            ReasoningService::new(ServiceConfig {
                workspace: self.0.clone(),
                registry_path: Some(self.0.join("tools.toml")),
                database_path: None,
                method_registry: default_registry().clone(),
                method_executor: None,
                scheduler: CollectionScheduler::default(),
                supervisor: Supervisor::new(PathBuf::from(env!(
                    "CARGO_BIN_EXE_eal-command-supervisor"
                ))),
                limits: ExecutionLimits::default(),
                lease_timeout: Duration::from_secs(1),
            })
            .unwrap(),
        )
    }
    fn knowledge(&self) -> Arc<EALKnowledgeBase> {
        let knowledge = Arc::new(EALKnowledgeBase::new(self.service()).unwrap());
        knowledge
            .register(
                "case.eal",
                RegisterOptions {
                    entry_id: Some("demo"),
                    context: Some(&json!({"site":"bench"})),
                    claims: Some(&["works".into()]),
                    label: None,
                },
            )
            .unwrap();
        knowledge
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn at(now: &str) -> RegisteredAssessmentOptions<'_> {
    RegisteredAssessmentOptions {
        now: Some(now),
        ..Default::default()
    }
}
fn records(knowledge: &EALKnowledgeBase, result: &Value) -> Value {
    knowledge
        .service
        .store()
        .get(
            result["collection_id"].as_str().unwrap(),
            Some("collection"),
        )
        .unwrap()["records"]
        .clone()
}

#[test]
fn registered_assessment_reuses_individual_readings_and_preserves_original_time_across_revisions() {
    let workspace = Workspace::new();
    let knowledge = workspace.knowledge();
    let first = knowledge.assess("demo", "works", at(FIRST)).unwrap();
    assert_eq!(first["status"], "supported");
    assert_eq!(first["collected_count"], 2);
    assert_eq!(first["reused_count"], 0);
    assert!(!first.to_string().contains("collector-secret"));
    workspace.observation("2040-01-01T00:00:06Z", true);
    let second = knowledge.assess("demo", "works", at(SECOND)).unwrap();
    assert_eq!(second["status"], "supported");
    assert_eq!(second["reused_count"], 1);
    assert_eq!(second["collected_count"], 1);
    let original = records(&knowledge, &first);
    let current = records(&knowledge, &second);
    assert_eq!(current["steady"]["collected_at"], "2040-01-01T00:00:00Z");
    assert_eq!(
        current["steady"]["reused_from_run_id"],
        original["steady"]["run_id"]
    );
    assert!(current["quick"].get("reused_from_run_id").is_none());
    workspace.write(
        "case.eal",
        &SOURCE.replace("Both checks passed.", "Both checks passed at this bench."),
    );
    let revised = knowledge.assess("demo", "works", at(SECOND)).unwrap();
    assert_ne!(revised["source_digest"], second["source_digest"]);
    assert_eq!(revised["reused_count"], 2);
    assert_eq!(revised["collected_count"], 0);
    assert_ne!(revised["collection_id"], second["collection_id"]);
    assert_eq!(
        knowledge.history("demo", None, 50).unwrap()["assessments"][0]["assessment_id"],
        revised["assessment_id"]
    );
    let fresh = knowledge
        .assess(
            "demo",
            "works",
            RegisteredAssessmentOptions {
                reuse: ReusePolicy::Fresh,
                ..at(SECOND)
            },
        )
        .unwrap();
    assert_eq!(fresh["reused_count"], 0);
    assert_eq!(fresh["collected_count"], 2);
}

#[test]
fn comparable_negative_measurements_reuse_but_failed_imports_and_context_changes_do_not() {
    let workspace = Workspace::new();
    workspace.observation("2040-01-01T00:00:00Z", false);
    let knowledge = workspace.knowledge();
    let negative = knowledge.assess("demo", "works", at(FIRST)).unwrap();
    assert_eq!(negative["status"], "unsupported");
    let reused = knowledge.assess("demo", "works", at(FIRST)).unwrap();
    assert_eq!(reused["reused_count"], 2);
    assert_eq!(reused["collected_count"], 0);
    assert_eq!(
        reused["packet"]["evidence"]["quick"]["availability_issues"],
        json!(["predicate_not_met"])
    );
    let outside = knowledge
        .assess(
            "demo",
            "works",
            RegisteredAssessmentOptions {
                context: Some(&json!({"site":"field"})),
                ..at(FIRST)
            },
        )
        .unwrap();
    assert_eq!(outside["status"], "out_of_scope");
    assert_eq!(outside["reused_count"], 0);
    assert_eq!(outside["collected_count"], 2);
    let empty = Workspace::new();
    let failed = empty.knowledge();
    empty.write("observation.json", "not JSON");
    assert_eq!(
        failed.assess("demo", "works", at(FIRST)).unwrap()["status"],
        "unsupported"
    );
    empty.observation("2040-01-01T00:00:00Z", true);
    let retried = failed.assess("demo", "works", at(FIRST)).unwrap();
    assert_eq!(retried["status"], "supported");
    assert_eq!(retried["collected_count"], 2);
}

#[test]
fn selected_claim_and_question_byte_admission_precede_every_acquisition() {
    let workspace = Workspace::new();
    let knowledge = workspace.knowledge();
    let adapter = ModelContextAdapter::new(Arc::clone(&knowledge));
    assert!(
        knowledge
            .assess("demo", "other", at(FIRST))
            .unwrap_err()
            .contains("not selected")
    );
    for question in [" ", "\u{1c}", &"é".repeat(4097)] {
        assert_eq!(
            adapter
                .prepare(question, "absent", "works", at(FIRST))
                .unwrap_err(),
            "question must contain one to 8192 UTF-8 bytes"
        );
    }
    assert!(
        knowledge
            .service
            .store()
            .list(Some("observation"), 100)
            .unwrap()
            .is_empty()
    );
    let question = "é".repeat(4096);
    let prepared = adapter
        .prepare(&question, "demo", "works", at(FIRST))
        .unwrap();
    assert_eq!(
        prepared["messages"][1],
        json!({"role":"user","content":question})
    );
    assert!(
        !prepared["messages"][0]["content"]
            .as_str()
            .unwrap()
            .contains("collector-secret")
    );
    assert_eq!(prepared["assessment"]["status"], "supported");
    assert!(prepared["context"]["claims"]["works"]["prose_verified"] == false);
}

#[test]
fn expiry_during_other_acquisition_is_rechecked_without_refreshing_measurement_time() {
    let workspace = Workspace::new();
    let knowledge = workspace.knowledge();
    knowledge.assess("demo", "works", at(FIRST)).unwrap();
    workspace.observation("2040-01-01T00:00:06Z", true);
    let calls = Arc::new(AtomicUsize::new(0));
    let captured = Arc::clone(&calls);
    let clock = Arc::new(move || {
        let n = captured.fetch_add(1, Ordering::SeqCst);
        if n == 0 {
            SECOND.into()
        } else {
            "2040-01-01T00:16:41Z".into()
        }
    });
    let host = RegisteredAssessmentHost::with_clock(
        Arc::clone(&knowledge.service),
        Arc::clone(&knowledge.catalogue),
        clock,
    )
    .unwrap();
    let result = host
        .assess("demo", "works", RegisteredAssessmentOptions::default())
        .unwrap();
    assert_eq!(result["status"], "unsupported");
    assert_eq!(result["reused_count"], 0);
    assert_eq!(result["collected_count"], 2);
    let stored = records(&knowledge, &result);
    assert_eq!(stored["quick"]["collected_at"], "2040-01-01T00:00:06Z");
    assert_eq!(stored["steady"]["collected_at"], "2040-01-01T00:00:06Z");
}

#[test]
fn mixed_reuse_commit_rolls_back_derived_observations_when_collection_insert_fails() {
    let workspace = Workspace::new();
    let knowledge = workspace.knowledge();
    knowledge.assess("demo", "works", at(FIRST)).unwrap();
    let before = knowledge
        .service
        .store()
        .list(Some("observation"), 100)
        .unwrap()
        .len();
    let connection = rusqlite::Connection::open(workspace.0.join(".eal/runs.sqlite3")).unwrap();
    connection.execute_batch("CREATE TRIGGER reject_test_collection BEFORE INSERT ON records WHEN NEW.kind='collection' BEGIN SELECT RAISE(ABORT,'test collection failure'); END;").unwrap();
    drop(connection);
    assert!(
        knowledge
            .assess("demo", "works", at(FIRST))
            .unwrap_err()
            .contains("test collection failure")
    );
    assert_eq!(
        knowledge
            .service
            .store()
            .list(Some("observation"), 100)
            .unwrap()
            .len(),
        before
    );
    assert_eq!(
        knowledge
            .service
            .store()
            .list(Some("assessment"), 100)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn source_changed_by_a_collector_is_rejected_before_assessment_return() {
    let workspace = Workspace::new();
    workspace.write(
        "changed.eal",
        &SOURCE.replace("Both checks passed.", "Changed during collection."),
    );
    // Use JSON quoting for the trusted shell argument in TOML to preserve apostrophes.
    let argv = json!([
        "/bin/sh",
        "-c",
        "/bin/cp changed.eal case.eal; printf '%s' '{\"value\":{\"ok\":true}}'"
    ]);
    workspace.write(
        "tools.toml",
        &format!("[tools.reader]\nkind='command'\nversion='1'\ninherit_env=[]\nargv={argv}\n"),
    );
    let knowledge = workspace.knowledge();
    let result = knowledge.assess("demo", "works", at(FIRST));
    assert!(
        result
            .unwrap_err()
            .contains("source changed during assessment")
    );
    assert!(
        knowledge
            .service
            .store()
            .list(Some("assessment"), 100)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_catalogue_from_another_service_cannot_be_used_as_authority() {
    let workspace = Workspace::new();
    let service = workspace.service();
    let other = workspace.service();
    let catalogue = Arc::new(WorkspaceKnowledgeCatalogue::new(other).unwrap());
    assert!(
        RegisteredAssessmentHost::new(service, catalogue)
            .err()
            .unwrap()
            .contains("same service")
    );
}
