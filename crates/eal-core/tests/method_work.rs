//! A finite native work allowance distinguishes unfinished reasoning from negatives.
use eal_core::{
    ExecutionLimits,
    methods::{assess_input_with_limits, default_registry},
};
use serde_json::{Value, json};

#[test]
fn native_work_admission_preserves_results_and_reports_exhaustion_separately() {
    let cases: [(&str, Value); 7] = [
        ("deductive/1", json!({"premises":["p"],"conclusion":"p"})),
        (
            "inductive/1",
            json!({"successes":90,"trials":100,"confidence":0.95}),
        ),
        (
            "abductive/1",
            json!({"observed":"alarm","candidates":[{"name":"A","prior":0.5,"likelihood":0.9},{"name":"B","prior":0.5,"likelihood":0.1}]}),
        ),
        (
            "causal/1",
            json!({"assignment":"randomised","treatment":[7,9],"control":[1,3]}),
        ),
        (
            "counterfactual/1",
            json!({"variables":{"X":{"intercept":1,"coefficients":{},"noise":0},"Y":{"intercept":0,"coefficients":{"X":2},"noise":1}},"intervention":{"variable":"X","value":4},"outcome":"Y"}),
        ),
        (
            "analogical/1",
            json!({"relevant_features":["interface"],"source":{"interface":"v1"},"target":{"interface":"v1"}}),
        ),
        (
            "temporal/1",
            json!({"start":0,"end":2,"max_gap":1,"events":[{"time":0,"value":3},{"time":1,"value":4},{"time":2,"value":5}],"property":{"operator":"ge","value":3},"semantics":"sampled"}),
        ),
    ];
    for (method, payload) in cases {
        let adequate = ExecutionLimits::default();
        let complete = assess_input_with_limits(method, &payload, default_registry(), &adequate);
        assert_eq!(complete["status"], "supported", "{method}: {complete}");
        let tight = ExecutionLimits {
            method_work: 1,
            ..adequate.clone()
        };
        let incomplete = assess_input_with_limits(method, &payload, default_registry(), &tight);
        assert_eq!(incomplete["status"], "incomplete", "{method}: {incomplete}");
        assert_eq!(incomplete["details"], json!({}));
        assert_eq!(incomplete["resource"]["kind"], "method_work");
        assert_eq!(incomplete["resource"]["allowance"], 1);
        let required = incomplete["resource"]["required"].as_u64().unwrap() as usize;
        let exact_allowance = ExecutionLimits {
            method_work: required,
            ..adequate
        };
        assert_eq!(
            assess_input_with_limits(method, &payload, default_registry(), &exact_allowance),
            complete
        );
    }
    let negative = assess_input_with_limits(
        "deductive/1",
        &json!({"premises":["p"],"conclusion":"q"}),
        default_registry(),
        &ExecutionLimits::default(),
    );
    assert_eq!(negative["status"], "supported", "{negative}");
    assert_eq!(negative["details"]["entailed"], false);
    assert!(negative["details"]["counterexample"].is_object());
}
