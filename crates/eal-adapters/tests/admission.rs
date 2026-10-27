//! Independently authored adapter resource and correspondence cases.
use eal_adapters::{files, retained, trace};
use eal_collector_contracts::AcquisitionRequest;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "eal-adapters-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn bounded_files_accept_equality_and_reject_growth_and_special_files() {
    let directory = Directory::new();
    let path = directory.0.join("input");
    std::fs::write(&path, b"1234").unwrap();
    assert_eq!(files::read(&path, 4, "input").unwrap(), b"1234");
    assert!(files::read(&path, 3, "input").is_err());
    assert!(files::read(&directory.0, 4, "input").is_err());
    #[cfg(unix)]
    {
        let alias = directory.0.join("alias");
        std::os::unix::fs::symlink(&path, &alias).unwrap();
        assert!(files::read(&alias, 4, "input").is_err());
        let fifo = directory.0.join("fifo");
        assert!(
            std::process::Command::new("mkfifo")
                .arg(&fifo)
                .status()
                .unwrap()
                .success()
        );
        let started = std::time::Instant::now();
        assert!(files::read(&fifo, 4, "input").is_err());
        assert!(started.elapsed() < std::time::Duration::from_secs(1));
    }
}

#[test]
fn retained_artifacts_keep_the_oldest_observation_and_check_pins() {
    let directory = Directory::new();
    let bytes = b"{\"voltage\":2}";
    std::fs::write(directory.0.join("a.json"), bytes).unwrap();
    let pin = format!("{:x}", Sha256::digest(bytes));
    let records = vec![retained::Artifact {
        path: "a.json".into(),
        sha256: pin.clone(),
        observed_at: "2024-01-02T00:00:00Z".into(),
        source: "meter".into(),
    }];
    let (value, observed) = retained::import_artifacts(&records, &directory.0).unwrap();
    assert_eq!(observed, "2024-01-02T00:00:00Z");
    assert_eq!(value["artifacts"][0]["observation"]["voltage"], 2);
    assert!(value.get("accepted").is_none());
    std::fs::write(directory.0.join("a.json"), b"{\"voltage\":3}").unwrap();
    assert!(retained::import_artifacts(&records, &directory.0).is_err());
}

fn artifact(path: PathBuf, bytes: &[u8]) -> retained::Artifact {
    retained::Artifact {
        path,
        sha256: format!("{:x}", Sha256::digest(bytes)),
        observed_at: "2024-01-02T00:00:00Z".into(),
        source: "meter".into(),
    }
}

#[test]
fn retained_artifacts_accept_relative_bases_and_keep_absolute_grants_independent() {
    // Avoid changing the process-wide current directory in parallel tests.
    let directory = Directory(PathBuf::from(format!(
        "eal-adapters-relative-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    std::fs::create_dir(&directory.0).unwrap();
    let bytes = b"{\"voltage\":2}";
    std::fs::write(directory.0.join("sample.json"), bytes).unwrap();
    let records = [artifact("sample.json".into(), bytes)];
    let (value, _) = retained::import_artifacts(&records, &directory.0).unwrap();
    assert_eq!(value["artifacts"][0]["path"], "sample.json");
    assert_eq!(value["artifacts"][0]["observation"]["voltage"], 2);
    assert!(
        retained::import_artifacts(
            &[artifact(directory.0.join("sample.json"), bytes)],
            std::path::Path::new(".")
        )
        .is_ok()
    );

    let absolute = directory.0.canonicalize().unwrap().join("sample.json");
    let unused_base = directory.0.join("not-created");
    let (value, _) =
        retained::import_artifacts(&[artifact(absolute.clone(), bytes)], &unused_base).unwrap();
    assert_eq!(value["artifacts"][0]["path"], absolute.to_str().unwrap());
    assert!(retained::import_artifacts(&records, &unused_base).is_err());
    assert!(retained::import_artifacts(&records, &absolute).is_err());
    assert!(
        retained::import_artifacts(&[artifact("../sample.json".into(), bytes)], &directory.0)
            .is_err()
    );
}

#[cfg(unix)]
#[test]
fn retained_artifacts_accept_directory_aliases_but_reject_relative_and_leaf_escape() {
    use std::os::unix::fs::symlink;

    let directory = Directory::new();
    let outside = Directory::new();
    let bytes = b"{\"voltage\":2}";
    std::fs::write(directory.0.join("sample.json"), bytes).unwrap();
    std::fs::write(outside.0.join("sample.json"), bytes).unwrap();
    let alias = outside.0.join("inputs");
    symlink(&directory.0, &alias).unwrap();
    let (value, _) =
        retained::import_artifacts(&[artifact("sample.json".into(), bytes)], &alias).unwrap();
    assert_eq!(value["artifacts"][0]["observation"]["voltage"], 2);

    symlink(&outside.0, directory.0.join("escape")).unwrap();
    assert!(
        retained::import_artifacts(&[artifact("escape/sample.json".into(), bytes)], &alias)
            .is_err()
    );
    symlink(outside.0.join("sample.json"), directory.0.join("leaf.json")).unwrap();
    assert!(retained::import_artifacts(&[artifact("leaf.json".into(), bytes)], &alias).is_err());
}

fn review_request() -> AcquisitionRequest {
    serde_json::from_value(json!({"evidence_id":"lab","environment":"bench","tool":"lab.review","tool_version":"1",
        "input":{"criterion":"calibration applicable","target":"meter-A","scope":{"run":"A"}},"context":{"scope":"B"}})).unwrap()
}
fn review_value(verdict: &str) -> serde_json::Value {
    json!({"criterion":"calibration applicable","target":"meter-A","scope":{"run":"A"},
        "reviewer":"lab-reviewer","verdict":verdict,"reasons":["The retained calibration episode has the stated limit."],
        "reviewed_at":"2024-01-02T00:00:00Z","evidence_refs":["calibration-record-A"]})
}
#[test]
fn scoped_reviews_preserve_every_verdict_and_original_review_time() {
    let request = review_request();
    for verdict in ["supported", "rejected", "inconclusive"] {
        let review = retained::Review {
            request: request.acquisition(),
            value: review_value(verdict),
        };
        let (value, observed) = retained::import_review(&request, vec![review]).unwrap();
        assert_eq!(value["verdict"], verdict);
        assert_eq!(observed, "2024-01-02T00:00:00Z");
        assert!(value.get("accepted").is_none());
    }
}
#[test]
fn scoped_reviews_reject_missing_identity_reasons_scope_and_duplicate_requests() {
    let request = review_request();
    for (field, replacement) in [
        ("scope", json!({"run":"other"})),
        ("target", json!("other")),
        ("criterion", json!("other")),
        ("reviewer", json!("")),
        ("verdict", json!("true")),
        ("reasons", json!([])),
        ("reviewed_at", json!("2024-02-30T00:00:00Z")),
    ] {
        let mut value = review_value("supported");
        value[field] = replacement;
        assert!(
            retained::import_review(
                &request,
                vec![retained::Review {
                    request: request.acquisition(),
                    value
                }]
            )
            .is_err(),
            "{field}"
        );
    }
    assert!(
        retained::import_review(
            &request,
            vec![retained::Review {
                request: request.acquisition(),
                value: json!({"accepted":true})
            }]
        )
        .is_err()
    );
    let records = (0..2)
        .map(|_| retained::Review {
            request: request.acquisition(),
            value: review_value("supported"),
        })
        .collect();
    assert!(retained::import_review(&request, records).is_err());
    let mut different = request.acquisition();
    different["context"] = json!({"scope":"different"});
    assert!(
        retained::import_review(
            &request,
            vec![retained::Review {
                request: different,
                value: review_value("supported")
            }]
        )
        .is_err()
    );
    let mut unscoped = request;
    unscoped.input["scope"] = json!(null);
    assert!(retained::import_review(&unscoped, vec![]).is_err());
}

#[test]
fn trace_attribute_keys_are_configurable_without_strengthening_conclusions() {
    let data = json!({"resourceSpans":[{"scopeSpans":[{"spans":[{"traceId":"11111111111111111111111111111111","spanId":"2222222222222222","startTimeUnixNano":"1","endTimeUnixNano":"2","attributes":[{"key":"lab.kind","value":{"stringValue":"meter"}},{"key":"lab.name","value":{"stringValue":"A"}}]}]}]}]});
    let result = trace::inspect_trace_with_keys(
        &data,
        "meter",
        "A",
        trace::AttributeKeys {
            kind: "lab.kind",
            name: "lab.name",
        },
    )
    .unwrap();
    assert_eq!(result["candidate_matches"], 1);
    for field in [
        "provenance_verified",
        "current_state_verified",
        "fleet_coverage_verified",
        "completeness_verified",
    ] {
        assert_eq!(result[field], false);
    }
    assert_eq!(
        trace::inspect_trace(&data, "meter", "A").unwrap()["candidate_matches"],
        0
    );
}
