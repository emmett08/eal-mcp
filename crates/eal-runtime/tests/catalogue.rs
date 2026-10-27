#![cfg(unix)]
use eal_core::{ExecutionLimits, canonical, methods};
use eal_runtime::{
    catalogue::{FindOptions, RegisterOptions, WorkspaceKnowledgeCatalogue},
    methods::{MethodExecutor, WorkerCommand},
    process::Supervisor,
    registered_assessment::{RegisteredAssessmentHost, RegisteredAssessmentOptions},
    scheduler::CollectionScheduler,
    service::{ReasoningService, ServiceConfig},
};
use serde_json::{Value, json};
use std::{path::PathBuf, sync::Arc, time::Duration};

const SOURCE: &str = "language \"EAL/3\"\nenvironment lab {\nrequire site == \"bench\"\n}\ntool reader {\nversion \"1\"\n}\nevidence observation {\ntool reader\nkind test\nenvironment lab\nmax_age 60\nrequire ok == true\n}\nreasoning authored {\nmethod \"structured/1\"\nrationale \"The measurement supports this claim.\"\n}\nclaim ready {\nstatement \"The service is ready.\"\nenvironment lab\n}\nargument route = [evidence observation] via authored => ready\n";
struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("eal-catalogue-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn write(&self, path: &str, source: &str) {
        let path = self.0.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, source).unwrap();
    }
    fn service(&self) -> Arc<ReasoningService> {
        Arc::new(
            ReasoningService::new(ServiceConfig {
                workspace: self.0.clone(),
                registry_path: None,
                database_path: None,
                method_registry: methods::default_registry().clone(),
                method_executor: None,
                scheduler: CollectionScheduler::default(),
                supervisor: Supervisor::new(self.0.join("unused-supervisor")),
                limits: ExecutionLimits::default(),
                lease_timeout: Duration::from_secs(1),
            })
            .unwrap(),
        )
    }
    fn catalogue(&self) -> WorkspaceKnowledgeCatalogue {
        WorkspaceKnowledgeCatalogue::new(self.service()).unwrap()
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn options<'a>(context: &'a Value, claims: &'a [String]) -> RegisterOptions<'a> {
    RegisterOptions {
        context: Some(context),
        claims: Some(claims),
        label: Some("Readiness"),
        ..Default::default()
    }
}

#[test]
fn named_sources_metadata_search_and_reload_preserve_exact_identities() {
    let workspace = Workspace::new();
    workspace.write("source.eal", SOURCE);
    let catalogue = workspace.catalogue();
    let context = json!({"site":"bench"});
    let claims = vec!["ready".into()];
    let entry = catalogue
        .register("source.eal", options(&context, &claims))
        .unwrap();
    assert_eq!(entry["entry_id"], "source.eal");
    assert_eq!(entry["context"], context);
    assert_eq!(entry["claims"], json!(claims));
    assert!(entry.get("source").is_none());
    assert_eq!(
        entry["source_digest"],
        catalogue.service().parse(SOURCE).unwrap().source_digest
    );
    assert_eq!(
        entry["method_registry_fingerprint"],
        catalogue.service().method_registry().fingerprint()
    );
    assert_eq!(
        catalogue
            .register("source.eal", options(&context, &claims))
            .unwrap(),
        entry
    );
    let second = workspace.catalogue();
    assert_eq!(second.get("source.eal", true).unwrap()["source"], SOURCE);
    assert_eq!(second.read("source.eal").unwrap(), SOURCE);
    assert_eq!(
        second.list(50, 0).unwrap().as_slice(),
        std::slice::from_ref(&entry)
    );
    assert_eq!(
        second
            .find(FindOptions {
                query: Some("readiness"),
                claim: Some("ready"),
                context: Some(&context),
                ..Default::default()
            })
            .unwrap()
            .as_slice(),
        std::slice::from_ref(&entry)
    );
    assert_eq!(
        second
            .find(FindOptions {
                query: Some("service is ready"),
                ..Default::default()
            })
            .unwrap()
            .as_slice(),
        std::slice::from_ref(&entry)
    );
    assert!(
        second
            .find(FindOptions {
                claim: Some("missing"),
                ..Default::default()
            })
            .unwrap()
            .is_empty()
    );
    assert!(
        second
            .find(FindOptions {
                context: Some(&json!({"site":"elsewhere"})),
                ..Default::default()
            })
            .unwrap()
            .is_empty()
    );
    let mut detached = second.get_with_refresh("source.eal", true, false).unwrap();
    detached["claim_metadata"]["ready"]["statement"] = json!("Changed");
    assert_eq!(
        second.get_with_refresh("source.eal", false, false).unwrap(),
        entry
    );
}

#[test]
fn source_revisions_are_immutable_and_invalid_edits_do_not_replace_them() {
    let workspace = Workspace::new();
    workspace.write("source.eal", SOURCE);
    let catalogue = workspace.catalogue();
    let claims = vec!["ready".into()];
    let context = json!({"site":"bench"});
    let original = catalogue
        .register("source.eal", options(&context, &claims))
        .unwrap();
    let changed_source = SOURCE.replace(
        "The service is ready.",
        "The service is ready after the second check.",
    );
    workspace.write("source.eal", &changed_source);
    let changed = catalogue.get("source.eal", true).unwrap();
    assert_ne!(changed["source_digest"], original["source_digest"]);
    assert_eq!(changed["source"], changed_source);
    let digest = original["source_digest"].as_str().unwrap();
    let previous = catalogue
        .get_revision("source.eal", digest, None, true)
        .unwrap();
    assert_eq!(previous["current"], false);
    assert_eq!(previous["source"], SOURCE);
    assert_eq!(catalogue.revisions("source.eal", 50).unwrap().len(), 2);
    workspace.write(
        "source.eal",
        &changed_source
            .replace("claim ready", "claim gone")
            .replace("=> ready", "=> gone"),
    );
    assert!(
        catalogue
            .get("source.eal", false)
            .unwrap_err()
            .contains("Selected claims are absent")
    );
    assert_eq!(
        catalogue
            .get_revision(
                "source.eal",
                changed["source_digest"].as_str().unwrap(),
                None,
                false
            )
            .unwrap()["current"],
        true
    );
    workspace.write("source.eal", "invalid source");
    assert!(
        catalogue
            .get("source.eal", false)
            .unwrap_err()
            .contains("EAL validation")
    );
    std::fs::remove_file(workspace.0.join("source.eal")).unwrap();
    assert!(catalogue.get("source.eal", false).is_err());
    assert_eq!(
        catalogue
            .get_revision("source.eal", digest, None, true)
            .unwrap()["source"],
        SOURCE
    );
}

#[test]
fn changed_import_bundles_advance_revision_even_when_root_text_is_identical() {
    let workspace = Workspace::new();
    let source = "language \"EAL/3\"\nimport \"support.eal\" as support\nclaim ready {\nstatement \"The bounded condition holds.\"\nenvironment support.lab\n}\n";
    let support = "language \"EAL/3\"\nenvironment lab {\nrequire site == \"bench\"\n}\n";
    workspace.write("source.eal", source);
    workspace.write("support.eal", support);
    let catalogue = workspace.catalogue();
    let original = catalogue
        .register("source.eal", Default::default())
        .unwrap();
    workspace.write("support.eal", &support.replace("bench", "field"));
    let changed = catalogue.get("source.eal", true).unwrap();
    assert_ne!(original["source_digest"], changed["source_digest"]);
    assert_eq!(changed["source"], source);
    assert_eq!(catalogue.revisions("source.eal", 50).unwrap().len(), 2);
}

#[test]
fn entry_metadata_is_fixed_and_path_ownership_is_unique() {
    let workspace = Workspace::new();
    workspace.write("source.eal", SOURCE);
    workspace.write("other.eal", SOURCE);
    let catalogue = workspace.catalogue();
    catalogue
        .register(
            "source.eal",
            RegisterOptions {
                entry_id: Some("stable"),
                label: Some("Readiness"),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(
        catalogue
            .register(
                "source.eal",
                RegisterOptions {
                    entry_id: Some("another"),
                    ..Default::default()
                }
            )
            .unwrap_err()
            .contains("already registered")
    );
    assert!(
        catalogue
            .register(
                "other.eal",
                RegisterOptions {
                    entry_id: Some("stable"),
                    label: Some("Readiness"),
                    ..Default::default()
                }
            )
            .unwrap_err()
            .contains("metadata differs")
    );
    assert!(
        catalogue
            .register(
                "source.eal",
                RegisterOptions {
                    entry_id: Some("stable"),
                    label: Some("Changed"),
                    ..Default::default()
                }
            )
            .is_err()
    );
    assert!(
        catalogue
            .get_revision("stable", "A".repeat(64).as_str(), None, false)
            .unwrap_err()
            .contains("lowercase SHA-256")
    );
    assert!(
        catalogue
            .get_revision("stable", &"0".repeat(64), Some("bad"), false)
            .unwrap_err()
            .contains("method_registry_fingerprint")
    );
    for limit in [0, 101, usize::MAX] {
        assert!(catalogue.list(limit, 0).is_err());
        assert!(catalogue.revisions("stable", limit).is_err());
        assert!(catalogue.runs("stable", None, limit).is_err());
    }
    assert!(catalogue.list(1, 10001).is_err());
}

#[test]
fn historical_runs_are_exact_source_and_context_metadata_only() {
    let workspace = Workspace::new();
    workspace.write("source.eal", SOURCE);
    let catalogue = workspace.catalogue();
    let context = json!({"site":"bench"});
    let entry = catalogue
        .register(
            "source.eal",
            RegisterOptions {
                context: Some(&context),
                ..Default::default()
            },
        )
        .unwrap();
    let digest = &entry["source_digest"];
    let store = catalogue.service().store();
    store.put("collection",&json!({"source_digest":digest,"context":context,"records":{"observation":{"value":{"secret":"Not catalogue metadata"}}}}),Some("collection-good")).unwrap();
    store
        .put(
            "collection",
            &json!({"source_digest":digest,"context":{"site":"elsewhere"},"records":{}}),
            Some("collection-other"),
        )
        .unwrap();
    store.put("assessment",&json!({"source_digest":digest,"context_fingerprint":canonical::canonical_digest(&context,catalogue.service().limits()).unwrap(),"method_registry_fingerprint":entry["method_registry_fingerprint"],"collection_id":"collection-good","assessed_at":"2040-01-01T00:00:00Z","claims":{"ready":{"status":"supported"},"unselected":{"status":"contested"}}}),Some("assessment-good")).unwrap();
    store.put("assessment",&json!({"source_digest":digest,"context_fingerprint":"other","claims":{"ready":{"status":"unsupported"}}}),Some("assessment-other")).unwrap();
    let history = workspace.catalogue().runs("source.eal", None, 50).unwrap();
    assert_eq!(history["collections"].as_array().unwrap().len(), 1);
    assert_eq!(
        history["collections"][0]["collection_id"],
        "collection-good"
    );
    assert_eq!(history["collections"][0]["record_count"], 1);
    assert_eq!(history["assessments"].as_array().unwrap().len(), 1);
    assert_eq!(
        history["assessments"][0]["claim_statuses"],
        json!({"ready":"supported"})
    );
    assert!(!history.to_string().contains("secret"));
}

#[test]
fn discovery_is_bounded_sorted_and_reports_invalid_files() {
    let workspace = Workspace::new();
    workspace.write("nested/good.eal", SOURCE);
    workspace.write("nested/bad.eal", "invalid source");
    workspace.write("nested/ignored.txt", SOURCE);
    let catalogue = workspace.catalogue();
    let result = catalogue.register_tree("nested", None, 512).unwrap();
    assert_eq!(result["registered"][0]["path"], "nested/good.eal");
    assert_eq!(result["rejected"][0]["path"], "nested/bad.eal");
    assert!(
        catalogue
            .register_tree("nested", None, 1)
            .unwrap_err()
            .contains("discovery limit")
    );
    // Existing generated IDs are refreshed in their trusted namespace; public
    // explicit developer IDs still use their separate simple-name admission.
    assert_eq!(
        catalogue.get("nested/good.eal", true).unwrap()["source"],
        SOURCE
    );
    assert!(
        catalogue
            .register("../outside.eal", Default::default())
            .unwrap_err()
            .contains("outside the workspace")
    );
    assert!(
        catalogue
            .register("source.txt", Default::default())
            .is_err()
    );
    for (index, options) in [
        RegisterOptions {
            entry_id: Some("a/b"),
            ..Default::default()
        },
        RegisterOptions {
            label: Some("\u{1c}\u{1d}\u{1e}\u{1f}"),
            ..Default::default()
        },
        RegisterOptions {
            context: Some(&json!([])),
            ..Default::default()
        },
        RegisterOptions {
            context: Some(&json!({"x":"x".repeat(8192)})),
            ..Default::default()
        },
    ]
    .into_iter()
    .enumerate()
    {
        assert!(
            catalogue.register("nested/good.eal", options).is_err(),
            "{index}"
        );
    }
}

#[test]
fn tree_does_not_follow_directory_symlinks_or_read_nonregular_sources() {
    let workspace = Workspace::new();
    workspace.write("real/good.eal", SOURCE);
    std::os::unix::fs::symlink(workspace.0.join("real"), workspace.0.join("alias")).unwrap();
    rustix::fs::mkfifoat(
        rustix::fs::CWD,
        workspace.0.join("pipe.eal"),
        rustix::fs::Mode::from_raw_mode(0o600),
    )
    .unwrap();
    let catalogue = workspace.catalogue();
    let result = catalogue.register_tree(".", None, 512).unwrap();
    assert_eq!(result["registered"].as_array().unwrap().len(), 1);
    assert_eq!(result["rejected"][0]["path"], "pipe.eal");
    assert!(
        result["rejected"][0]["reason"]
            .as_str()
            .unwrap()
            .contains("regular EAL file")
    );
}

#[test]
fn unchanged_source_keeps_distinct_immutable_method_registry_revisions() {
    let workspace = Workspace::new();
    workspace.write("source.eal", SOURCE);
    let first = workspace
        .catalogue()
        .register("source.eal", Default::default())
        .unwrap();
    let supervisor = Supervisor::new(companion::path());
    let worker = WorkerCommand::new(
        vec![env!("CARGO_BIN_EXE_eal-method-worker").into()],
        Default::default(),
        workspace.0.clone(),
    )
    .unwrap();
    let executor = MethodExecutor::new(supervisor.clone(), worker).unwrap();
    let service = Arc::new(
        ReasoningService::new(ServiceConfig {
            workspace: workspace.0.clone(),
            registry_path: None,
            database_path: None,
            method_registry: methods::engineering_registry(),
            method_executor: Some(executor),
            scheduler: CollectionScheduler::default(),
            supervisor,
            limits: ExecutionLimits::default(),
            lease_timeout: Duration::from_secs(1),
        })
        .unwrap(),
    );
    let catalogue = WorkspaceKnowledgeCatalogue::new(service).unwrap();
    let current = catalogue.get("source.eal", true).unwrap();
    assert_eq!(current["source_digest"], first["source_digest"]);
    assert_ne!(
        current["method_registry_fingerprint"],
        first["method_registry_fingerprint"]
    );
    let previous = catalogue
        .get_revision(
            "source.eal",
            first["source_digest"].as_str().unwrap(),
            first["method_registry_fingerprint"].as_str(),
            true,
        )
        .unwrap();
    assert_eq!(previous["current"], false);
    assert_eq!(previous["source"], SOURCE);
    let revisions = catalogue.revisions("source.eal", 50).unwrap();
    assert_eq!(revisions.len(), 2);
    assert_eq!(
        revisions
            .iter()
            .filter(|item| item["current"] == true)
            .count(),
        1
    );
}

#[test]
fn nested_default_entry_ids_can_refresh_and_assess_registered_claims() {
    let workspace = Workspace::new();
    workspace.write("nested/source.eal", SOURCE);
    workspace.write(
        "tools.toml",
        "[tools.reader]\nkind='json_file'\nversion='1'\npath='observation.json'\n",
    );
    workspace.write(
        "observation.json",
        &json!({"observed_at":"2040-01-01T00:00:00Z","context":{"site":"bench"},
            "request":{"tool":"reader","tool_version":"1","input":{},"context":{"site":"bench"}},
            "value":{"ok":true}})
        .to_string(),
    );
    let service = Arc::new(
        ReasoningService::new(ServiceConfig {
            workspace: workspace.0.clone(),
            registry_path: Some(workspace.0.join("tools.toml")),
            database_path: None,
            method_registry: methods::default_registry().clone(),
            method_executor: None,
            scheduler: CollectionScheduler::default(),
            supervisor: Supervisor::new(companion::path()),
            limits: ExecutionLimits::default(),
            lease_timeout: Duration::from_secs(1),
        })
        .unwrap(),
    );
    let catalogue = Arc::new(WorkspaceKnowledgeCatalogue::new(Arc::clone(&service)).unwrap());
    let entry = catalogue
        .register(
            "nested/source.eal",
            RegisterOptions {
                context: Some(&json!({"site":"bench"})),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(entry["entry_id"], "nested/source.eal");
    assert_eq!(catalogue.read("nested/source.eal").unwrap(), SOURCE);
    let host = RegisteredAssessmentHost::new(service, catalogue).unwrap();
    let result = host
        .assess(
            "nested/source.eal",
            "ready",
            RegisteredAssessmentOptions {
                now: Some("2040-01-01T00:00:01Z"),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(result["status"], "supported");
    assert_eq!(result["source_digest"], entry["source_digest"]);
    assert_eq!(result["collected_count"], 1);
}

#[path = "common/companion.rs"]
mod companion;
