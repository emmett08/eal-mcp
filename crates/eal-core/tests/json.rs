use eal_core::{ExecutionLimits, json::parse};
use serde_json::json;

#[test]
fn strict_json_rejects_duplicates_and_preserves_reserved_literal_keys() {
    for source in [
        r#"{"a":1,"a":2}"#,
        r#"{"a":1,"\u0061":2}"#,
        r#"[ {"x":0,"x":1} ]"#,
    ] {
        assert!(
            parse(source, &ExecutionLimits::default())
                .unwrap_err()
                .contains("Duplicate")
        );
    }
    let value = parse(
        r#"{"$serde_json::private::Number":"1","$serde_json::private::RawValue":[true]}"#,
        &ExecutionLimits::default(),
    )
    .unwrap();
    assert!(value.is_object());
    assert_eq!(value["$serde_json::private::Number"], "1");
    assert_eq!(value["$serde_json::private::RawValue"], json!([true]));
}

#[test]
fn strict_json_preserves_numeric_types_and_binary64_semantics() {
    let value = parse(
        "[123456789012345678901234567890123456789,1.00000000000000000000000001,-0.0,-0]",
        &ExecutionLimits::default(),
    )
    .unwrap();
    assert_eq!(
        value[0].to_string(),
        "123456789012345678901234567890123456789"
    );
    assert_eq!(value[1].to_string(), "1.0");
    assert_eq!(value[2].to_string(), "-0.0");
    assert_eq!(value[3].to_string(), "0");
    for invalid in ["1e999", "NaN", "Infinity", r#""\ud800""#, "true false"] {
        assert!(
            parse(invalid, &ExecutionLimits::default()).is_err(),
            "{invalid}"
        );
    }
}

#[test]
fn strict_json_enforces_explicit_input_depth_and_node_budgets() {
    for limits in [
        ExecutionLimits {
            json_depth: 1,
            ..ExecutionLimits::default()
        },
        ExecutionLimits {
            json_nodes: 2,
            ..ExecutionLimits::default()
        },
        ExecutionLimits {
            record_bytes: 3,
            ..ExecutionLimits::default()
        },
    ] {
        assert!(parse("[[1]]", &limits).is_err());
    }
}

#[test]
fn raised_host_depth_cannot_overflow_a_small_calling_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let limits = ExecutionLimits {
                json_depth: usize::MAX,
                ..ExecutionLimits::default()
            };
            let source = format!("{}0{}", "[".repeat(5000), "]".repeat(5000));
            assert!(parse(&source, &limits).is_err());
            let mut value = json!(0);
            for _ in 0..256 {
                value = serde_json::Value::Array(vec![value]);
            }
            assert!(eal_core::canonical::canonical_json_bytes(&value, &limits).is_err());
        })
        .unwrap()
        .join()
        .unwrap();
}
