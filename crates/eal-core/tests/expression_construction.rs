//! Pure construction must reject oversized runtime values under the selected policy.
use eal_core::{ExecutionLimits, Expression, JsonValue, canonical, expressions};
use serde_json::{Value, json};

fn field(name: &str) -> Expression {
    Expression::Field { name: name.into() }
}

fn object(fields: Vec<(&str, Expression)>) -> Expression {
    Expression::Call {
        name: "object".into(),
        arguments: fields
            .into_iter()
            .flat_map(|(key, value)| {
                [
                    Expression::Literal {
                        value: JsonValue::String(key.into()),
                    },
                    value,
                ]
            })
            .collect(),
    }
}

fn rejects_construction(expression: &Expression, input: &Value, limits: &ExecutionLimits) {
    assert!(
        expressions::derivation_errors_with_limits(expression, None, limits).is_empty(),
        "the data-dependent expression is valid under the selected policy"
    );
    let input_bytes = canonical::canonical_json_bytes(input, limits).unwrap();
    assert!(input_bytes.len() <= limits.record_bytes);
    let control = expressions::evaluate_expression(expression, input);
    assert!(control.known(), "{}", control.reason);
    let constrained = expressions::evaluate_expression_with_limits(expression, input, limits);
    assert!(!constrained.known(), "constructed an oversized JSON value");
    assert_eq!(constrained.issue.as_deref(), Some("invalid_expression"));
}

#[test]
fn repeated_bound_values_cannot_expand_an_object_past_its_node_budget() {
    rejects_construction(
        &object(vec![("a", field("items")), ("b", field("items"))]),
        &json!({"items":(0..6).collect::<Vec<_>>()}),
        &ExecutionLimits {
            json_nodes: 10,
            ..ExecutionLimits::default()
        },
    );
}

#[test]
fn repeated_bound_values_cannot_expand_an_object_past_its_byte_budget() {
    rejects_construction(
        &object(vec![("a", field("text")), ("b", field("text"))]),
        &json!({"text":"abcd"}),
        &ExecutionLimits {
            record_bytes: 18,
            ..ExecutionLimits::default()
        },
    );
}

#[test]
fn nested_construction_cannot_add_depth_beyond_the_json_budget() {
    rejects_construction(
        &object(vec![("outer", object(vec![("inner", field("items"))]))]),
        &json!({"items":[0]}),
        &ExecutionLimits {
            json_depth: 2,
            ..ExecutionLimits::default()
        },
    );
}

#[test]
fn separately_admitted_comparison_operands_do_not_share_an_output_budget() {
    let expression = Expression::Call {
        name: "equal".into(),
        arguments: vec![field("items"), field("items")],
    };
    let input = json!({"items":(0..6).collect::<Vec<_>>()});
    let limits = ExecutionLimits {
        json_nodes: 10,
        ..ExecutionLimits::default()
    };
    let result = expressions::evaluate_expression_with_limits(&expression, &input, &limits);
    assert_eq!(result.value, Some(json!(true)), "{}", result.reason);
}

#[test]
fn object_construction_admits_exact_node_byte_and_depth_boundaries() {
    let expression = object(vec![("a", field("items")), ("b", field("items"))]);
    let input = json!({"items":(0..6).collect::<Vec<_>>()});
    let limits = ExecutionLimits {
        json_nodes: 15,
        record_bytes: 37,
        json_depth: 2,
        ..ExecutionLimits::default()
    };
    let result = expressions::evaluate_expression_with_limits(&expression, &input, &limits);
    assert_eq!(
        result.value,
        Some(json!({"a":[0,1,2,3,4,5],"b":[0,1,2,3,4,5]})),
        "{}",
        result.reason
    );
}

#[test]
fn escaped_unicode_keys_use_canonical_byte_boundaries() {
    let key = "🫨\n\"\\\u{0001}";
    let expression = object(vec![(key, field("text"))]);
    for (bytes, known) in [(24, true), (23, false)] {
        let result = expressions::evaluate_expression_with_limits(
            &expression,
            &json!({"text":"x"}),
            &ExecutionLimits {
                record_bytes: bytes,
                ..ExecutionLimits::default()
            },
        );
        assert_eq!(result.known(), known, "{}", result.reason);
        if known {
            assert_eq!(result.value, Some(json!({key:"x"})));
        }
    }
}
