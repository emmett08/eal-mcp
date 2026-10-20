//! Detect accidental drift in frozen source artefacts without an upstream checkout.

use sha2::{Digest, Sha256};
use std::path::Path;

#[test]
fn frozen_grammar_and_schema_bytes_match_the_source_lock() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("contracts/source-lock.json")).unwrap())
            .unwrap();
    let grammar = std::fs::read(root.join("grammar/EAL.g4")).unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(grammar)),
        manifest["grammar"]["sha256"].as_str().unwrap()
    );
    for artifact in manifest["frozen_artifacts"].as_array().unwrap() {
        let path = artifact["local_path"].as_str().unwrap();
        let content = std::fs::read(root.join(path)).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(content)),
            artifact["sha256"].as_str().unwrap(),
            "{path} drifted"
        );
    }
}
