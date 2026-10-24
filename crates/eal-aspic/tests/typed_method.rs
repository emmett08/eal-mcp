//! Typed native formal assessment keeps scope, query and incomplete states explicit.
#[path = "../../../tests/native_case.rs"]
mod native_case;
use eal_core::{ExecutionLimits, composition, evaluation, validation};
use serde_json::{Value, json};
fn assess(case: &Value, limits: ExecutionLimits) -> Value {
    let registry = eal_aspic::aspic_registry();
    let program = composition::compile(case["source"].as_str().unwrap(), None, "", limits).unwrap();
    assert!(validation::validate(&program, &registry).is_empty());
    assert!(!validation::validate(&program, eal_core::methods::default_registry()).is_empty());
    evaluation::evaluate(
        &program,
        &case["records"],
        case["now"].as_str().unwrap(),
        &case["context"],
        &registry,
        None,
    )
    .unwrap()
}
#[test]
fn typed_query_acceptance_is_bound_to_payload_and_original_scope() {
    let case = native_case::typed_case();
    let result = assess(&case, ExecutionLimits::default());
    assert_eq!(
        result["claims"]["accepted"]["status"], "supported",
        "{result}"
    );
    assert_eq!(
        result["arguments"]["formal_arg"]["reasoning_result"]["binding"]["formal_query"],
        case["records"]["formal"]["value"]["payload"]
    );
    let mut changed = case.clone();
    changed["context"] = json!({"site":"field"});
    assert_eq!(
        assess(&changed, ExecutionLimits::default())["claims"]["accepted"]["status"],
        "out_of_scope"
    );
    changed = case.clone();
    changed["now"] = json!("2026-10-24T12:01:01Z");
    assert_eq!(
        assess(&changed, ExecutionLimits::default())["claims"]["accepted"]["status"],
        "unsupported"
    );
    changed = case.clone();
    changed["records"]["formal"]["value"]["payload"]["theory"]["goal"] = json!("different");
    changed["records"]["formal"]["data_digest"] = json!(
        eal_core::canonical::canonical_digest(
            &changed["records"]["formal"]["value"],
            &ExecutionLimits::default()
        )
        .unwrap()
    );
    assert_eq!(
        assess(&changed, ExecutionLimits::default())["claims"]["accepted"]["status"],
        "unsupported"
    );
}
#[test]
fn exhausted_typed_formal_assessment_retains_no_partial_conclusion() {
    let result = assess(
        &native_case::typed_case(),
        ExecutionLimits {
            formal_arguments: 1,
            ..Default::default()
        },
    );
    assert_eq!(result["complete"], false);
    assert_eq!(result["claims"]["accepted"]["status"], "unsupported");
    let computation = &result["arguments"]["formal_arg"]["reasoning_result"];
    assert_eq!(computation["status"], "incomplete");
    assert_eq!(computation["details"], json!({}));
}
