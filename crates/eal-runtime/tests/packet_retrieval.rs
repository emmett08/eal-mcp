#![cfg(unix)]

use eal_core::{
    ExecutionLimits,
    methods::{MethodRegistry, default_registry},
};
use eal_runtime::{
    catalogue::RegisterOptions,
    knowledge::EALKnowledgeBase,
    packets::PacketLimits,
    process::Supervisor,
    registered_assessment::RegisteredAssessmentOptions,
    scheduler::CollectionScheduler,
    service::{ReasoningService, ServiceConfig},
};
use serde_json::{Value, json};
use std::{path::PathBuf, sync::Arc, time::Duration};

const OBSERVED_AT: &str = "2040-01-01T00:00:00Z";
const ASSESSED_AT: &str = "2040-01-01T00:00:01Z";
const SOURCE: &str = r#"language "EAL/3"
environment lab {
 require site == "bench"
}
tool reader {
 version "1"
}
evidence reading {
 tool reader
 kind test
 environment lab
 max_age 5
 require ok == true
}
evidence private_reading {
 tool reader
 kind test
 environment lab
 max_age 5
 require ok == true
}
reasoning measured {
 method "structured/1"
 rationale "The identified observation supports the bounded proposition."
}
claim premise {
 statement "The identified reading satisfies the declared predicate."
 environment lab
}
claim works {
 statement "The declared check passed at this bench."
 environment lab
}
claim private_claim {
 statement "PRIVATE_SIBLING_STATEMENT"
 environment lab
}
argument observation = [evidence reading] via measured => premise
argument result = [premises premise] via measured => works
argument private_argument = [evidence private_reading] via measured => private_claim
"#;

struct Workspace(PathBuf);

impl Workspace {
    fn new(source: &str) -> Self {
        let path = std::env::temp_dir().join(format!("eal-packet-read-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        let workspace = Self(path);
        workspace.write("case.eal", source);
        workspace.write(
            "tools.toml",
            "[tools.reader]\nkind='json_file'\nversion='1'\npath='observation.json'\n",
        );
        workspace.write(
            "observation.json",
            &json!({
                "observed_at": OBSERVED_AT,
                "context": {"site": "bench"},
                "request": {"tool": "reader", "tool_version": "1", "input": {}, "context": {"site": "bench"}},
                "value": {"ok": true, "raw_secret": "PRIVATE_COLLECTOR_PAYLOAD"}
            })
            .to_string(),
        );
        workspace
    }

    fn write(&self, path: &str, contents: &str) {
        std::fs::write(self.0.join(path), contents).unwrap();
    }

    fn knowledge(&self, methods: MethodRegistry) -> EALKnowledgeBase {
        let service = Arc::new(
            ReasoningService::new(ServiceConfig {
                workspace: self.0.clone(),
                registry_path: Some(self.0.join("tools.toml")),
                database_path: None,
                method_registry: methods,
                method_executor: None,
                scheduler: CollectionScheduler::default(),
                supervisor: Supervisor::new(PathBuf::from(env!(
                    "CARGO_BIN_EXE_eal-command-supervisor"
                ))),
                limits: ExecutionLimits::default(),
                lease_timeout: Duration::from_secs(1),
            })
            .unwrap(),
        );
        let knowledge = EALKnowledgeBase::new(service).unwrap();
        knowledge
            .register(
                "case.eal",
                RegisterOptions {
                    entry_id: Some("demo"),
                    context: Some(&json!({"site": "bench"})),
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

fn assess(knowledge: &EALKnowledgeBase) -> Value {
    knowledge
        .assess(
            "demo",
            "works",
            RegisteredAssessmentOptions {
                now: Some(ASSESSED_AT),
                ..Default::default()
            },
        )
        .unwrap()
}

fn id(assessment: &Value) -> &str {
    assessment["assessment_id"].as_str().unwrap()
}

#[test]
fn default_retrieval_preserves_the_packet_and_qualifications_without_private_siblings() {
    let workspace = Workspace::new(SOURCE);
    let knowledge = workspace.knowledge(default_registry().clone());
    let assessment = assess(&knowledge);
    let retrieved = knowledge
        .host
        .retrieve_packet("demo", id(&assessment), "works", PacketLimits::default())
        .unwrap();

    assert_eq!(retrieved["schema"], "EAL/registered-packet/1");
    assert_eq!(retrieved["entry_id"], "demo");
    assert_eq!(retrieved["claim"], "works");
    assert_eq!(retrieved["packet"], assessment["packet"]);
    for key in [
        "assessment_id",
        "assessed_at",
        "status",
        "source_digest",
        "context_fingerprint",
    ] {
        assert_eq!(retrieved[key], assessment[key], "{key}");
    }
    assert_eq!(
        retrieved["method_registry_fingerprint"],
        assessment["packet"]["method_registry_fingerprint"]
    );
    assert_eq!(retrieved["status"], "supported");
    assert_eq!(
        retrieved["packet"]["premise_claims"]["premise"]["status"],
        "supported"
    );
    assert_eq!(
        retrieved["packet"]["claims"]["works"]["prose_verified"],
        false
    );
    assert_eq!(
        retrieved["packet"]["evidence"]["reading"]["observed_at"],
        OBSERVED_AT
    );
    assert_eq!(
        retrieved["packet"]["evidence"]["reading"]["age_seconds"].as_f64(),
        Some(1.0)
    );
    for secret in [
        "PRIVATE_COLLECTOR_PAYLOAD",
        "PRIVATE_SIBLING_STATEMENT",
        "private_claim",
        "private_argument",
        "private_reading",
    ] {
        assert!(!retrieved.to_string().contains(secret), "{secret}");
    }
    assert!(
        knowledge
            .host
            .retrieve_packet("demo", id(&assessment), "premise", PacketLimits::default())
            .is_err()
    );
    assert!(
        knowledge
            .host
            .retrieve_packet(
                "demo",
                id(&assessment),
                "private_claim",
                PacketLimits::default()
            )
            .is_err()
    );
}

#[test]
fn retrieval_requires_a_registered_source_revision_and_the_fixed_context() {
    let workspace = Workspace::new(SOURCE);
    let knowledge = workspace.knowledge(default_registry().clone());
    let assessment = assess(&knowledge);
    assert!(
        knowledge
            .host
            .retrieve_packet("absent", id(&assessment), "works", PacketLimits::default())
            .is_err()
    );
    let unrelated_source =
        SOURCE.replace("The declared check passed", "An unregistered check passed");
    let unrelated = knowledge
        .service
        .reason(
            &unrelated_source,
            &json!({"site": "bench"}),
            None,
            Some(ASSESSED_AT),
        )
        .unwrap();
    assert_ne!(unrelated["source_digest"], assessment["source_digest"]);
    assert!(
        knowledge
            .host
            .retrieve_packet("demo", id(&unrelated), "works", PacketLimits::default())
            .is_err()
    );
    let other_context = knowledge
        .assess(
            "demo",
            "works",
            RegisteredAssessmentOptions {
                context: Some(&json!({"site": "field"})),
                now: Some(ASSESSED_AT),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(other_context["source_digest"], assessment["source_digest"]);
    assert_ne!(
        other_context["context_fingerprint"],
        assessment["context_fingerprint"]
    );
    assert!(
        knowledge
            .host
            .retrieve_packet("demo", id(&other_context), "works", PacketLimits::default())
            .is_err()
    );
}

#[test]
fn a_different_current_method_registry_cannot_reinterpret_an_old_assessment() {
    let workspace = Workspace::new(SOURCE);
    let original = workspace.knowledge(default_registry().clone());
    let assessment = assess(&original);
    // Both registries can evaluate this source, but their contracts have different identities.
    let methods =
        MethodRegistry::new([default_registry().get("structured/1").unwrap().clone()]).unwrap();
    let changed = workspace.knowledge(methods);
    assert_ne!(
        changed.service.method_registry().fingerprint(),
        assessment["packet"]["method_registry_fingerprint"]
    );
    assert!(
        changed
            .host
            .retrieve_packet("demo", id(&assessment), "works", PacketLimits::default())
            .is_err()
    );
    // Retaining a newer catalogue revision must not hide the exact older registered revision.
    assert_eq!(
        original
            .host
            .retrieve_packet("demo", id(&assessment), "works", PacketLimits::default())
            .unwrap()["packet"],
        assessment["packet"]
    );
}

#[test]
fn historical_retrieval_survives_invalid_source_edits_without_acquisition_or_new_records() {
    let workspace = Workspace::new(SOURCE);
    let knowledge = workspace.knowledge(default_registry().clone());
    let original = assess(&knowledge);
    workspace.write(
        "case.eal",
        &SOURCE.replace("The declared check passed", "The revised check passed"),
    );
    let revised = assess(&knowledge);
    assert_ne!(original["source_digest"], revised["source_digest"]);
    workspace.write("case.eal", "This is no longer valid EAL.");
    std::fs::remove_file(workspace.0.join("observation.json")).unwrap();
    let before = knowledge.service.store().list(None, 100).unwrap().len();
    let revisions = knowledge.catalogue.revisions("demo", 100).unwrap();

    let retrieved = knowledge
        .host
        .retrieve_packet("demo", id(&original), "works", PacketLimits::default())
        .unwrap();

    assert_eq!(retrieved["packet"], original["packet"]);
    assert_eq!(retrieved["assessed_at"], ASSESSED_AT);
    assert_eq!(
        retrieved["packet"]["evidence"]["reading"]["observed_at"],
        OBSERVED_AT
    );
    assert_eq!(
        knowledge.service.store().list(None, 100).unwrap().len(),
        before
    );
    assert_eq!(
        knowledge.catalogue.revisions("demo", 100).unwrap(),
        revisions
    );
}

#[test]
fn larger_budgets_recover_statement_and_dependencies_from_the_same_assessment() {
    let statement = format!("The declared check passed. {}", "é".repeat(1900));
    let source = SOURCE.replace("The declared check passed at this bench.", &statement);
    let workspace = Workspace::new(&source);
    let knowledge = workspace.knowledge(default_registry().clone());
    let assessment = assess(&knowledge);
    assert_eq!(
        assessment["packet"]["claims"]["works"]["statement_truncated"],
        true
    );
    assert_eq!(assessment["packet"]["summary_complete"], false);
    let detailed_limits = PacketLimits {
        max_bytes: 64 * 1024,
        max_statement_bytes: 4096,
        ..Default::default()
    };
    let small = knowledge
        .host
        .retrieve_packet(
            "demo",
            id(&assessment),
            "works",
            PacketLimits {
                max_bytes: 4096,
                ..detailed_limits.clone()
            },
        )
        .unwrap();
    assert_eq!(small["packet"]["omitted"]["details"], "packet_byte_limit");
    assert_eq!(small["packet"]["summary_complete"], false);
    assert!(serde_json::to_vec(&small["packet"]).unwrap().len() <= 4096);

    let expanded = knowledge
        .host
        .retrieve_packet("demo", id(&assessment), "works", detailed_limits)
        .unwrap();
    assert_eq!(
        expanded["packet"]["claims"]["works"]["statement"],
        statement
    );
    assert_eq!(expanded["packet"]["summary_complete"], true);
    assert_eq!(
        expanded["packet"]["claims"]["works"]["prose_verified"],
        false
    );
    assert!(expanded["packet"]["arguments"].get("observation").is_some());
    assert!(expanded["packet"]["arguments"].get("result").is_some());
    assert_eq!(expanded["assessment_id"], small["assessment_id"]);
    assert_eq!(expanded["status"], small["status"]);
    assert!(serde_json::to_vec(&expanded["packet"]).unwrap().len() <= 64 * 1024);
}

#[test]
fn invalid_assessment_packet_limits_are_rejected_before_acquisition() {
    let workspace = Workspace::new(SOURCE);
    let knowledge = workspace.knowledge(default_registry().clone());
    let limits = PacketLimits {
        max_bytes: 4095,
        ..Default::default()
    };
    assert!(
        knowledge
            .assess(
                "demo",
                "works",
                RegisteredAssessmentOptions {
                    packet_limits: Some(&limits),
                    now: Some(ASSESSED_AT),
                    ..Default::default()
                },
            )
            .is_err()
    );
    assert!(
        knowledge
            .service
            .store()
            .list(None, 100)
            .unwrap()
            .is_empty()
    );
}
