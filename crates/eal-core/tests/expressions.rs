//! Differential arithmetic, unknown propagation, typing and quantity contracts.
use eal_core::{ExecutionLimits, expressions};
use serde_json::{Value, json};

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

#[test]
fn arithmetic_preserves_exact_integer_order_and_signed_float_results() {
    let literal = |value| json!({"kind":"literal","value":value,"arguments":[]});
    for (operator, left, right, expected) in [
        (
            "+",
            json!(9007199254740993u64),
            json!(1),
            json!(9007199254740994u64),
        ),
        (
            ">",
            json!(9007199254740993u64),
            json!(9007199254740992.0),
            json!(true),
        ),
        ("%", json!(-5), json!(2), json!(1)),
        ("%", json!(5), json!(-2), json!(-1)),
        ("/", json!(5), json!(-2), json!(-2.5)),
    ] {
        let expression = expressions::expression_from_json(
            &json!({"kind":"binary","value":operator,"arguments":[literal(left),literal(right)]}),
        )
        .unwrap();
        let result = expressions::evaluate_expression(&expression, &json!({}));
        assert_eq!(result.value, Some(expected), "{operator}: {result:?}");
    }
    let incompatible = expressions::expression_from_json(
        &json!({"kind":"binary","value":"==","arguments":[literal(json!(true)),literal(json!(1))]}),
    )
    .unwrap();
    assert_eq!(
        expressions::evaluate_expression(&incompatible, &json!({}))
            .issue
            .as_deref(),
        Some("invalid_expression")
    );
    let sum = expressions::expression_from_json(
        &json!({"kind":"call","value":"sum","arguments":[literal(json!([1e16,1.0,-1e16]))]}),
    )
    .unwrap();
    let result = expressions::evaluate_expression(&sum, &json!({}));
    assert_eq!(result.value, Some(json!(1.0)));
    let zero = expressions::expression_from_json(
        &json!({"kind":"binary","value":"/","arguments":[literal(json!(0)),literal(json!(-2))]}),
    )
    .unwrap();
    assert_eq!(
        expressions::evaluate_expression(&zero, &json!({}))
            .value
            .unwrap()
            .as_f64()
            .unwrap()
            .to_bits(),
        (-0.0f64).to_bits()
    );
}
