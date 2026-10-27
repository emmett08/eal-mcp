//! Independent checks of public JSON shapes and native recursion boundaries.
use eal_core::{ExecutionLimits, canonical, composition, json, methods, validation};
use serde_json::{Value, json};

fn keys(value: &Value) -> Vec<&str> {
    let mut keys: Vec<_> = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    keys
}

#[test]
fn canonical_order_and_literal_reserved_keys_preserve_user_payloads() {
    let source = r#"{"𐀀":1,"":2,"span":{"start":{"line":90},"end":{"column":91}},"method_contract":{"identity_basis":"user payload","implementation_version":"literal"},"$serde_json::private::Number":"1","$serde_json::private::RawValue":"null"}"#;
    let limits = ExecutionLimits::default();
    let value = json::parse(source, &limits).unwrap();
    let canonical =
        String::from_utf8(canonical::canonical_json_bytes(&value, &limits).unwrap()).unwrap();
    assert_eq!(
        canonical,
        r#"{"$serde_json::private::Number":"1","$serde_json::private::RawValue":"null","method_contract":{"identity_basis":"user payload","implementation_version":"literal"},"span":{"end":{"column":91},"start":{"line":90}},"":2,"𐀀":1}"#
    );
    assert_eq!(json::parse(&canonical, &limits).unwrap(), value);
    assert_eq!(
        value["span"],
        json!({"start":{"line":90},"end":{"column":91}})
    );
    assert_eq!(value["method_contract"]["identity_basis"], "user payload");
}

#[test]
fn semantic_spans_are_flat_without_rewriting_nested_input_objects() {
    let source = r#"language "EAL/4"
environment lab { require true
}
tool sensor { version "1"
}
evidence observation {
  tool sensor
  environment lab
  max_age 60
  kind test
  input {"span":{"start":{"line":90},"end":{"column":91}}}
  require true
}
claim target { statement "A source location is explicit."
  environment missing
}
strict absent reviewed "synthetic-review/span-boundary"
"#;
    let program =
        composition::compile(source, None, "source.eal", ExecutionLimits::default()).unwrap();
    let encoded = serde_json::to_value(&program).unwrap();
    let expected_keys = ["column", "end_column", "end_line", "line"];
    for span in encoded["locations"].as_object().unwrap().values().chain(
        encoded["argumentation_directives"]
            .as_array()
            .unwrap()
            .iter()
            .map(|directive| &directive["span"]),
    ) {
        assert_eq!(keys(span), expected_keys);
        assert!(span.as_object().unwrap().values().all(Value::is_u64));
    }
    let diagnostics = validation::validate(&program, methods::default_registry());
    assert!(!diagnostics.is_empty());
    for diagnostic in serde_json::to_value(diagnostics)
        .unwrap()
        .as_array()
        .unwrap()
    {
        if !diagnostic["span"].is_null() {
            assert_eq!(keys(&diagnostic["span"]), expected_keys);
        }
    }
    assert_eq!(
        encoded["evidence"]["observation"]["input"]["span"],
        json!({"start":{"line":90},"end":{"column":91}})
    );
    // Authored syntax keeps byte/character positions; semantic serialization is local.
    let lexical = serde_json::to_value(eal_core::parse(source).unwrap()).unwrap();
    assert!(lexical["declarations"][0]["span"].get("start").is_some());
}

#[test]
fn raised_host_depth_still_fails_regularly_at_the_native_recursion_ceiling() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let limits = ExecutionLimits {
                json_depth: usize::MAX,
                ..ExecutionLimits::default()
            };
            for (prefix, suffix) in [("[", "]"), ("{\"x\":", "}")] {
                for depth in [64, json::MAX_JSON_DEPTH] {
                    let accepted = format!("{}0{}", prefix.repeat(depth), suffix.repeat(depth));
                    let value = json::parse(&accepted, &limits).unwrap();
                    assert_eq!(
                        canonical::canonical_json_bytes(&value, &limits).unwrap(),
                        accepted.as_bytes()
                    );
                }
            }
            for depth in [129, 5000] {
                let rejected = format!("{}0{}", "[".repeat(depth), "]".repeat(depth));
                assert!(
                    json::parse(&rejected, &limits).is_err(),
                    "Depth {depth} must fail"
                );
            }
            let mut value = Value::Null;
            for _ in 0..129 {
                value = Value::Array(vec![value]);
            }
            assert!(canonical::canonical_json_bytes(&value, &limits).is_err());
        })
        .unwrap()
        .join()
        .expect("Depth failures must return errors without panicking");
}
