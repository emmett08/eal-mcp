//! The standalone EAL importer preserves scoped review decisions without EngArg.
use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
};

#[test]
fn command_imports_all_verdicts_with_the_original_review_clock() {
    let directory = std::env::temp_dir().join(format!("eal-scoped-review-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("reviews.json");
    let acquisition = json!({"tool":"reviewer","tool_version":"1","input":{"criterion":"bounded adequacy","target":"claim-A","scope":{"episode":"A"}},"context":{}});
    let mut request = acquisition.clone();
    request["evidence_id"] = json!("review");
    request["environment"] = json!("lab");
    for verdict in ["supported", "rejected", "inconclusive"] {
        let value = json!({"criterion":"bounded adequacy","target":"claim-A","scope":{"episode":"A"},"reviewer":"named-reviewer",
            "verdict":verdict,"reasons":["Recorded for this specified episode only."],"reviewed_at":"2020-01-01T00:00:00Z"});
        std::fs::write(&path,serde_json::to_vec(&json!({"schema":"eal.review-inputs/2","reviews":[{"request":acquisition,"value":value}]})).unwrap()).unwrap();
        let mut child = Command::new(env!("CARGO_BIN_EXE_eal-collector"))
            .args(["review-json", "--inputs"])
            .arg(&path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(&serde_json::to_vec(&request).unwrap())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(envelope["observed_at"], "2020-01-01T00:00:00Z");
        assert_eq!(envelope["value"]["verdict"], verdict);
        assert!(envelope["value"].get("accepted").is_none());
    }
    std::fs::remove_dir_all(directory).unwrap();
}
