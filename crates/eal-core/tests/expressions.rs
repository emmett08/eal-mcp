//! Differential arithmetic, unknown propagation, typing and quantity contracts.
use eal_core::{ExecutionLimits, expressions};
use serde_json::{Value, json};

fn normalise(value: &Value) -> Value {
    match value {
        Value::Null => json!({"type":"null"}),
        Value::Bool(value) => json!({"type":"boolean","value":value}),
        Value::String(value) => json!({"type":"string","value":value}),
        Value::Number(value) => {
            let text = value.to_string();
            if text.bytes().any(|byte| matches!(byte, b'.' | b'e' | b'E')) {
                json!({"type":"float","bits":format!("{:016x}",text.parse::<f64>().unwrap().to_bits())})
            } else {
                json!({"type":"integer","value":text})
            }
        }
        Value::Array(items) => {
            json!({"type":"array","value":items.iter().map(normalise).collect::<Vec<_>>()})
        }
        Value::Object(items) => {
            json!({"type":"object","value":items.iter().map(|(name,value)|(name.clone(),normalise(value))).collect::<serde_json::Map<_,_>>()})
        }
    }
}

#[test]
fn pinned_python_expression_results_types_and_diagnostics_agree() {
    let oracle: Value =
        serde_json::from_str(include_str!("../../../fixtures/expressions/reference.json")).unwrap();
    let cases = oracle["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 350);
    for case in cases {
        let source = case["source"].as_str().unwrap();
        let expression = expressions::expression_from_json(&case["expression"]).unwrap();
        assert_eq!(
            normalise(&expressions::expression_to_json(&expression).unwrap()),
            normalise(&case["expression"]),
            "{source}: expression interchange"
        );
        let result = expressions::evaluate_expression(&expression, &case["context"]);
        assert_eq!(
            result.known(),
            case["result"]["known"].as_bool().unwrap(),
            "{source}: {result:?}"
        );
        assert_eq!(
            result.issue,
            case["result"]["issue"].as_str().map(str::to_owned),
            "{source}: {result:?}"
        );
        if case["python_exception"].is_null() {
            assert_eq!(
                result.reason,
                case["result"]["reason"].as_str().unwrap(),
                "{source}"
            );
        }
        if let Some(quantity) = &result.quantity {
            let actual = json!({"type":"quantity","numerator":quantity.amount.numer().to_string(),"denominator":quantity.amount.denom().to_string(),"dimensions":quantity.dimensions});
            assert_eq!(actual, case["result"]["value"], "{source}");
        } else if let Some(value) = &result.value {
            assert_eq!(normalise(value), case["result"]["value"], "{source}");
        }
        let schema = (!case["schema"].is_null()).then_some(&case["schema"]);
        assert_eq!(
            expressions::expression_errors(&expression, schema),
            serde_json::from_value::<Vec<String>>(case["errors"].clone()).unwrap(),
            "{source}: static errors"
        );
        assert_eq!(
            expressions::format_expression(&expression).unwrap(),
            case["formatted"].as_str().unwrap(),
            "{source}: formatting"
        );
    }
}

#[test]
fn explicit_expression_limits_bound_evaluation_and_static_checks() {
    let expression=expressions::expression_from_json(&json!({"kind":"binary","value":"+","arguments":[{"kind":"literal","value":1},{"kind":"literal","value":2}]})).unwrap();
    let limits = ExecutionLimits {
        expression_nodes: 2,
        ..ExecutionLimits::default()
    };
    let result = expressions::evaluate_expression_with_limits(&expression, &json!({}), &limits);
    assert!(!result.known());
    assert_eq!(result.issue.as_deref(), Some("invalid_expression"));
    assert_eq!(
        expressions::expression_errors_with_limits(&expression, None, &limits),
        ["Expression node budget exceeded"]
    );
}

#[test]
fn invalid_interchange_shapes_are_rejected() {
    for value in [
        json!({"kind":"literal","value":1,"arguments":[{}]}),
        json!({"kind":"field","value":true}),
        json!({"kind":"unary","value":"not","arguments":[]}),
        json!({"kind":"binary","value":"eval","arguments":[{},{}]}),
        json!({"kind":"literal","value":1,"extra":2}),
    ] {
        assert!(
            expressions::expression_from_json(&value).is_err(),
            "{value}"
        );
    }
}

#[test]
fn quantity_and_json_results_have_distinct_known_values() {
    let expression=expressions::expression_from_json(&json!({"kind":"call","value":"quantity","arguments":[{"kind":"literal","value":36},{"kind":"literal","value":"km/h"}]})).unwrap();
    let result = expressions::evaluate_expression(&expression, &json!({}));
    assert!(result.known());
    assert!(result.value.is_none());
    let quantity = result.quantity.unwrap();
    assert_eq!(quantity.amount.to_string(), "10");
    assert_eq!(
        quantity.dimensions,
        std::collections::BTreeMap::from([("length".to_owned(), 1), ("time".to_owned(), -1)])
    );
}

#[test]
fn malformed_or_oversized_typed_literals_return_diagnostics() {
    use eal_core::{Expression, JsonValue, NumberLiteral};
    let malformed = Expression::Literal {
        value: JsonValue::Number(NumberLiteral("not-a-number".into())),
    };
    let result = expressions::evaluate_expression(&malformed, &json!({}));
    assert!(!result.known());
    assert_eq!(result.issue.as_deref(), Some("invalid_expression"));
    assert!(!expressions::expression_errors(&malformed, None).is_empty());
    let mut literal = JsonValue::Null;
    for _ in 0..65 {
        literal = JsonValue::Array(vec![literal]);
    }
    let result =
        expressions::evaluate_expression(&Expression::Literal { value: literal }, &json!({}));
    assert!(!result.known());
    assert!(result.reason.contains("resource limit"));
}

#[test]
fn conversion_and_formatting_hard_bounds_survive_unlimited_host_depths() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            use eal_core::{Expression, JsonValue, UnaryOperator};
            let limits = ExecutionLimits {
                expression_depth: usize::MAX,
                json_depth: usize::MAX,
                ..ExecutionLimits::default()
            };
            let mut expression = Expression::Literal {
                value: JsonValue::Boolean(true),
            };
            for _ in 0..5_000 {
                expression = Expression::Unary {
                    operator: UnaryOperator::Not,
                    operand: Box::new(expression),
                };
            }
            assert!(
                !expressions::evaluate_expression_with_limits(&expression, &json!({}), &limits)
                    .known()
            );
            assert!(expressions::format_expression_with_limits(&expression, &limits).is_err());
            assert!(expressions::expression_to_json_with_limits(&expression, &limits).is_err());
            std::mem::forget(expression); // A forged recursive value's destructor belongs to its constructing caller.

            let mut value = Value::Null;
            for _ in 0..5_000 {
                value = Value::Array(vec![value]);
            }
            assert!(expressions::try_json_to_literal(&value, &limits).is_err());
            assert!(expressions::diagnostic_repr(&value, &limits).is_err());
            let mut object = serde_json::Map::new();
            object.insert("kind".into(), Value::String("literal".into()));
            object.insert("value".into(), value);
            let input = Value::Object(object);
            assert!(expressions::expression_from_json(&input).is_err());
            std::mem::forget(input);

            let mut literal = JsonValue::Null;
            for _ in 0..5_000 {
                literal = JsonValue::Array(vec![literal]);
            }
            assert!(expressions::try_literal_value(&literal, &limits).is_err());
            std::mem::forget(literal);

            let mut schema = json!({"type":"number"});
            for _ in 0..5_000 {
                let mut object = serde_json::Map::new();
                object.insert("anyOf".into(), Value::Array(vec![schema]));
                schema = Value::Object(object);
            }
            let field = Expression::Field { name: "x".into() };
            assert!(
                !expressions::expression_errors_with_limits(&field, Some(&schema), &limits)
                    .is_empty()
            );
            std::mem::forget(schema);
            assert!(expressions::numbers::parse_decimal("1.0e-2147483648").is_err());
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn checked_diagnostic_representation_preserves_python_quoting_and_limits() {
    let limits = ExecutionLimits::default();
    assert_eq!(
        expressions::diagnostic_repr(&json!([true, null, "🫨", "\u{200b}", "can't"]), &limits)
            .unwrap(),
        "[True, None, '🫨', '\\u200b', \"can't\"]"
    );
    let tiny = ExecutionLimits {
        record_bytes: 4,
        ..limits
    };
    assert!(expressions::diagnostic_repr(&json!("long"), &tiny).is_err());
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let limits = ExecutionLimits {
                json_depth: usize::MAX,
                ..ExecutionLimits::default()
            };
            let mut value = Value::Null;
            for _ in 0..128 {
                value = Value::Array(vec![value]);
            }
            assert!(expressions::diagnostic_repr(&value, &limits).is_ok());
        })
        .unwrap()
        .join()
        .unwrap();
}
