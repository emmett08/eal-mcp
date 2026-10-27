//! Authored formulae are checked program data, never manufactured observations.
use eal_core::{ExecutionLimits, methods};
use serde_json::json;

#[test]
fn authored_entailment_has_no_observation_identity() {
    let result = methods::assess_input_with_limits(
        "deductive/1",
        &json!({"premises":["a","b"],"conclusion":{"and":["a","b"]}}),
        methods::default_registry(),
        &ExecutionLimits::default(),
    );
    assert_eq!(result["status"], "supported");
    assert_eq!(result["details"]["entailed"], true);
    assert_eq!(result["input_origin"], "authored_program");
    assert!(result.get("evidence_id").is_none());
    assert!(result.get("run_id").is_none());
}

#[test]
fn query_validation_rejects_semantic_errors_but_accepts_negative_findings() {
    let registry = methods::default_registry();
    assert!(
        methods::validate_query("inductive/1", &json!({"confidence":0.95}), registry).is_empty()
    );
    assert!(
        methods::validate_query("causal/1", &json!({"assignment":"randomised"}), registry)
            .is_empty()
    );
    assert!(
        methods::validate_query(
            "deductive/1",
            &json!({"premises":["a"],"conclusion":"b"}),
            registry
        )
        .is_empty()
    );
    assert!(
        !methods::validate_query(
            "deductive/1",
            &json!({"premises":[true],"conclusion":"b"}),
            registry
        )
        .is_empty()
    );
    assert!(!methods::validate_query("abductive/1", &json!({"observed":"x","candidates":[{"name":"a","prior":0.6,"likelihood":0.5},{"name":"b","prior":0.6,"likelihood":0.5}]}), registry).is_empty());
    assert!(!methods::validate_query("counterfactual/1", &json!({"variables":{"a":{"intercept":0,"noise":0,"coefficients":{"a":0}}},"intervention":{"variable":"a","value":1},"outcome":"a"}), registry).is_empty());
}
