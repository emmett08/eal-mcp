//! Compare canonical identities with the retained, exact-source Python witnesses.
use eal_core::ExecutionLimits;
use eal_core::canonical::{canonical_digest, canonical_json_bytes, same_json};
use serde_json::{Value, json};
use std::path::Path;

#[test]
fn canonical_numbers_match_python_binary64_rounding_and_exponent_rules() {
    let reference: Value =
        serde_json::from_str(include_str!("../../../fixtures/canonical/reference.json")).unwrap();
    assert_eq!(reference["cases"].as_array().unwrap().len(), 800);
    for case in reference["cases"].as_array().unwrap() {
        let value: Value = serde_json::from_str(case["source"].as_str().unwrap()).unwrap();
        let bytes = canonical_json_bytes(&value, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            String::from_utf8(bytes).unwrap(),
            case["canonical_utf8"],
            "{}",
            case["bits"]
        );
        assert_eq!(
            canonical_digest(&value, &ExecutionLimits::default()).unwrap(),
            case["sha256"]
        );
    }
}

#[test]
fn frozen_identity_witnesses_preserve_types_unicode_and_arbitrary_integers() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let reference: Value = serde_json::from_slice(
        &std::fs::read(root.join("contracts/identity-fixtures.json")).unwrap(),
    )
    .unwrap();
    for case in reference["fixtures"].as_array().unwrap() {
        let bytes = canonical_json_bytes(&case["value"], &ExecutionLimits::default()).unwrap();
        assert_eq!(
            String::from_utf8(bytes).unwrap(),
            case["canonical_utf8"],
            "{}",
            case["name"]
        );
        assert_eq!(
            canonical_digest(&case["value"], &ExecutionLimits::default()).unwrap(),
            case["sha256"]
        );
    }
    for pair in [("1", "1.0"), ("true", "1"), ("0.0", "-0.0")] {
        assert!(!same_json(
            &serde_json::from_str(pair.0).unwrap(),
            &serde_json::from_str(pair.1).unwrap(),
            &ExecutionLimits::default()
        ));
    }
    assert!(same_json(
        &json!({"b": 2, "a": 1}),
        &json!({"a": 1, "b": 2}),
        &ExecutionLimits::default()
    ));
}

#[test]
fn canonical_resources_and_non_finite_numbers_fail_explicitly() {
    assert!(
        canonical_json_bytes(
            &json!([1, 2]),
            &ExecutionLimits {
                json_nodes: 2,
                ..ExecutionLimits::default()
            }
        )
        .is_err()
    );
    assert!(
        canonical_json_bytes(
            &json!([[1]]),
            &ExecutionLimits {
                json_depth: 1,
                ..ExecutionLimits::default()
            }
        )
        .is_err()
    );
    let value: Value = serde_json::from_str("1e999").unwrap();
    assert!(canonical_json_bytes(&value, &ExecutionLimits::default()).is_err());
    for source in [
        r#"{"json_nodes":0}"#,
        r#"{"unknown":1}"#,
        r#"{"json_nodes":true}"#,
    ] {
        assert!(serde_json::from_str::<ExecutionLimits>(source).is_err());
    }
}
