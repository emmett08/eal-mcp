//! Independent finite definitions for source-to-check transformations.
use eal_core::{ExecutionLimits, expressions};
use serde_json::{Value, json};

fn call(name: &str, values: Vec<Value>) -> eal_core::Expression {
    expressions::expression_from_json(&json!({"kind":"call","value":name,"arguments":values
        .into_iter().map(|value| json!({"kind":"literal","value":value})).collect::<Vec<_>>()}))
    .unwrap()
}

#[test]
fn array_helpers_preserve_typed_identity_and_exact_integers() {
    for (name, values, expected) in [
        (
            "at",
            vec![json!([9007199254740993u64, 0]), json!(0)],
            json!(9007199254740993u64),
        ),
        ("flatten", vec![json!([[1, 2], [], [3]])], json!([1, 2, 3])),
        (
            "flatten",
            vec![json!([[1], 2, [3, [4]]])],
            json!([1, 2, 3, [4]]),
        ),
        ("distinct", vec![json!(["Alpha", "alpha"])], json!(true)),
        ("distinct", vec![json!(["Alpha", "Alpha"])], json!(false)),
        ("distinct", vec![json!([1, 1.0, true, "1"])], json!(true)),
        (
            "distinct",
            vec![json!([{"x":1,"y":2},{"y":2,"x":1}])],
            json!(false),
        ),
        ("distinct", vec![json!([])], json!(true)),
        ("contains", vec![json!([1, true]), json!(1.0)], json!(false)),
        (
            "contains",
            vec![json!([{ "x":1,"y":2}]), json!({"y":2,"x":1})],
            json!(true),
        ),
    ] {
        let actual = expressions::evaluate_expression(&call(name, values), &json!({}));
        assert_eq!(actual.value, Some(expected), "{name}: {}", actual.reason);
    }
}

#[test]
fn invalid_shapes_and_budgets_are_not_silently_repaired() {
    for (name, values) in [
        ("at", vec![json!([1]), json!(1)]),
        ("at", vec![json!([1]), json!(-1)]),
        ("at", vec![json!([1]), json!(0.0)]),
        ("flatten", vec![json!({"not":"array"})]),
        ("distinct", vec![json!({"x":1})]),
    ] {
        assert!(!expressions::evaluate_expression(&call(name, values), &json!({})).known());
    }
    let expression = call("distinct", vec![json!([1, 2, 3])]);
    let result = expressions::evaluate_expression_with_limits(
        &expression,
        &json!({}),
        &ExecutionLimits {
            json_nodes: 3,
            ..Default::default()
        },
    );
    assert!(!result.known());
}

#[test]
fn exact_decimal_aggregates_do_not_round_through_binary64() {
    for (name, sample, numerator, denominator, unit) in [
        ("exact_sum", json!([0.1, 0.2]), "3", "10", None),
        ("exact_mean", json!([0.1, 0.2]), "3", "20", None),
        ("exact_mean", json!([[1, 2, 2]]), "5", "3", None),
        (
            "exact_min",
            json!([9007199254740993u64, 9007199254740992u64]),
            "9007199254740992",
            "1",
            None,
        ),
        (
            "exact_max",
            json!([9007199254740993u64, 9007199254740992u64]),
            "9007199254740993",
            "1",
            None,
        ),
        ("exact_count", json!([["one", "two"]]), "2", "1", Some("1")),
    ] {
        let result = expressions::evaluate_expression(&call(name, vec![sample]), &json!({}));
        let value = result
            .value
            .unwrap_or_else(|| panic!("{name}: {}", result.reason));
        assert_eq!(value["schema"], "eal.exact-number/1");
        assert_eq!(value["numerator"], numerator);
        assert_eq!(value["denominator"], denominator);
        assert_eq!(value.get("unit").and_then(Value::as_str), unit);
    }
}

#[test]
fn exact_scale_comparison_and_identity_preserve_units_and_domains() {
    let original =
        json!({"schema":"eal.exact-number/1","numerator":"1","denominator":"3","unit":"s"});
    let scaled = expressions::evaluate_expression(
        &call(
            "exact_scale",
            vec![
                original.clone(),
                json!("1000"),
                json!("1"),
                json!("s"),
                json!("ms"),
            ],
        ),
        &json!({}),
    )
    .value
    .unwrap();
    assert_eq!(scaled["numerator"], "1000");
    assert_eq!(scaled["denominator"], "3");
    assert_eq!(scaled["unit"], "ms");
    let compared = expressions::evaluate_expression(
        &call(
            "exact_compare",
            vec![scaled.clone(), json!(333), json!(">"), json!("ms")],
        ),
        &json!({}),
    );
    assert_eq!(compared.value, Some(json!(true)));
    assert!(
        !expressions::evaluate_expression(
            &call(
                "exact_compare",
                vec![scaled, json!(333), json!(">"), json!("s")]
            ),
            &json!({})
        )
        .known()
    );
    assert!(
        !expressions::evaluate_expression(
            &call(
                "exact_scale",
                vec![original, json!("1"), json!("0"), json!("s"), json!("ms")]
            ),
            &json!({})
        )
        .known()
    );
    for (left, right, expected) in [
        (json!([1, true]), json!([1, true]), true),
        (json!([1]), json!([1.0]), false),
        (json!({"x":1,"y":2}), json!({"y":2,"x":1}), true),
    ] {
        assert_eq!(
            expressions::evaluate_expression(&call("equal", vec![left, right]), &json!({})).value,
            Some(json!(expected))
        );
    }
}

#[test]
fn exact_aggregation_rejects_unknown_units_empty_samples_and_digit_exhaustion() {
    let seconds =
        json!({"schema":"eal.exact-number/1","numerator":"1","denominator":"2","unit":"s"});
    for sample in [json!([]), json!([seconds, 1]), json!([[1], [2]])] {
        assert!(
            !expressions::evaluate_expression(&call("exact_mean", vec![sample]), &json!({}))
                .known()
        );
    }
    let huge =
        json!({"schema":"eal.exact-number/1","numerator":"9".repeat(8193),"denominator":"1"});
    assert!(
        !expressions::evaluate_expression(&call("exact_sum", vec![json!([huge])]), &json!({}))
            .known()
    );
}

#[test]
fn boolean_selection_checks_preserve_empty_and_population_boundaries() {
    for (name, values, expected) in [
        ("all_values", json!([[true, true]]), true),
        ("all_values", json!([true, false]), false),
        ("all_values", json!([[]]), true),
        ("any_values", json!([[]]), false),
        ("any_values", json!([[false, true]]), true),
    ] {
        assert_eq!(
            expressions::evaluate_expression(&call(name, vec![values]), &json!({})).value,
            Some(json!(expected))
        );
    }
    for values in [json!([[true], [true]]), json!([1]), json!({})] {
        assert!(
            !expressions::evaluate_expression(&call("all_values", vec![values]), &json!({}))
                .known()
        );
    }
}
