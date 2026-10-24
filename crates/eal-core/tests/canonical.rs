//! Canonical identity preserves scalar distinctions within explicit resource bounds.
use eal_core::ExecutionLimits;
use eal_core::canonical::{canonical_digest, canonical_json_bytes, same_json};
use serde_json::{Value, json};

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

#[test]
fn canonical_identity_distinguishes_scalar_types_and_orders_object_keys() {
    for (left, right) in [("1", "1.0"), ("true", "1"), ("0.0", "-0.0")] {
        assert!(!same_json(
            &serde_json::from_str(left).unwrap(),
            &serde_json::from_str(right).unwrap(),
            &ExecutionLimits::default()
        ));
    }
    assert!(same_json(
        &json!({"b":2,"a":1}),
        &json!({"a":1,"b":2}),
        &ExecutionLimits::default()
    ));
    assert_ne!(
        canonical_digest(&json!(1), &ExecutionLimits::default()).unwrap(),
        canonical_digest(&json!(1.0), &ExecutionLimits::default()).unwrap()
    );
}
