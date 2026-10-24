use eal_core::ExecutionLimits;
use eal_runtime::identity::CollectionIdentityValidator;
use serde_json::{Value, json};

#[test]
fn exact_source_context_types_and_record_shape_are_required() {
    let limits = ExecutionLimits::default();
    let validator = CollectionIdentityValidator;
    let original = json!({"source_digest":"source","context":{"site":"bench","number":1},"records":{"a":{"status":"ok"}}});
    let expected = json!({"site":"bench","number":1});
    assert_eq!(
        validator
            .validate(&original, "source", &expected, &limits)
            .unwrap(),
        original["records"]
    );
    assert!(
        validator
            .validate(&json!([]), "source", &expected, &limits)
            .unwrap_err()
            .contains("JSON object")
    );
    assert!(
        validator
            .validate(&original, "revision", &expected, &limits)
            .unwrap_err()
            .contains("assessment source")
    );
    for context in [
        json!({"site":"bench","number":true}),
        json!({"site":"bench","number":1.0}),
        json!({"site":"field","number":1}),
    ] {
        assert!(
            validator
                .validate(&original, "source", &context, &limits)
                .unwrap_err()
                .contains("assessment context")
        );
    }
    for records in [json!([]), Value::Null, json!({"a":"not an observation"})] {
        let mut changed = original.clone();
        changed["records"] = records;
        assert!(
            validator
                .validate(&changed, "source", &expected, &limits)
                .unwrap_err()
                .contains("Collection records")
        );
    }
    let mut detached = validator
        .validate(&original, "source", &expected, &limits)
        .unwrap();
    detached["a"]["status"] = json!("edited");
    assert_eq!(original["records"]["a"]["status"], "ok");
}

#[test]
fn forged_deep_records_are_refused_before_clone_even_with_raised_limits() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let limits = ExecutionLimits {
                json_depth: usize::MAX,
                json_nodes: usize::MAX,
                ..ExecutionLimits::default()
            };
            let mut nested = Value::Null;
            for _ in 0..5000 {
                nested = Value::Array(vec![nested]);
            }
            let mut fields = serde_json::Map::new();
            fields.insert("deep".into(), nested);
            let mut records = serde_json::Map::new();
            records.insert("a".into(), Value::Object(fields));
            let mut collection = json!({"source_digest":"source","context":{}});
            collection["records"] = Value::Object(records);
            assert!(
                CollectionIdentityValidator
                    .validate(&collection, "source", &json!({}), &limits)
                    .is_err()
            );
            std::mem::forget(collection);
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn independent_records_do_not_share_one_observation_json_node_allowance() {
    let limits = ExecutionLimits {
        json_nodes: 1,
        ..ExecutionLimits::default()
    };
    let collection = json!({"source_digest":"source","context":{},"records":{"a":{},"b":{}}});
    assert_eq!(
        CollectionIdentityValidator
            .validate(&collection, "source", &json!({}), &limits)
            .unwrap(),
        collection["records"]
    );
}
