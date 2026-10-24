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

#[test]
fn retained_review_preserves_false_and_exact_request_identity() {
    let request:AcquisitionRequest=serde_json::from_value(json!({"evidence_id":"lab","environment":"bench","tool":"lab.review","tool_version":"1","input":{"run":"A"},"context":{"scope":"B"}})).unwrap();
    let review = retained::Review {
        request: request.acquisition(),
        observed_at: "2024-01-02T00:00:00Z".into(),
        value: json!({"accepted":false,"reason":"measured negative"}),
    };
    let (value, observed) = retained::import_review(&request, vec![review]).unwrap();
    assert_eq!(value["accepted"], false);
    assert_eq!(observed, "2024-01-02T00:00:00Z");
    let other = retained::Review {
        request: json!({"tool":"lab.review","tool_version":"1","input":{"run":"different"},"context":{"scope":"B"}}),
        observed_at: "2024-01-02T00:00:00Z".into(),
        value: json!({"accepted":true}),
    };
    assert!(retained::import_review(&request, vec![other]).is_err());
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
