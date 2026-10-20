//! Independent structural, witness-count and public resource-bound checks.
use eal_aspic::{AspicError, solve_aspic, validate_theory};
use eal_core::ExecutionLimits;
use serde_json::{Value, json};

fn pair() -> Value {
    json!({"premises":[{"atom":"p","kind":"ordinary","rank":500},{"atom":"q","kind":"ordinary","rank":500}],"rules":[],"contraries":[{"attacker":"p","target":"q"},{"attacker":"q","target":"p"}],"goal":"p","options":{"semantics":"preferred"}})
}

#[test]
fn extension_masks_retain_arguments_beyond_128() {
    let premises: Vec<_> = (0..129)
        .map(|index| json!({"atom":format!("p{index}"),"kind":"ordinary","rank":500}))
        .collect();
    let theory = json!({"premises":premises,"rules":[],"contraries":[],"goal":"p128","options":{"semantics":"preferred"}});
    let limits = ExecutionLimits {
        formal_premises: 200,
        formal_arguments: 200,
        formal_atoms: 200,
        ..ExecutionLimits::default()
    };
    let result = solve_aspic(&json!({"theory":theory}), &limits).unwrap();
    assert_eq!(result["argument_count"], 129);
    assert_eq!(result["extensions"][0].as_array().unwrap().len(), 129);
    assert_eq!(result["extensions"][0][128], "A128");
    assert_eq!(result["query_status"], "accepted");
}

#[test]
fn duplicate_edge_witnesses_each_consume_the_defeat_budget() {
    let theory = json!({"premises":[{"atom":"x","kind":"ordinary","rank":1},{"atom":"p","kind":"ordinary","rank":9},{"atom":"q","kind":"ordinary","rank":9}],"rules":[{"id":"join","kind":"strict","antecedents":["p","q"],"consequent":"goal"}],"contraries":[{"attacker":"x","target":"p"},{"attacker":"x","target":"q"}],"goal":"goal"});
    let limits = ExecutionLimits {
        formal_defeats: 4,
        ..ExecutionLimits::default()
    };
    let payload = json!({"theory":theory});
    let result = solve_aspic(&payload, &limits).unwrap();
    assert_eq!(result["defeat_count"], 3);
    assert_eq!(result["defeats"].as_array().unwrap().len(), 4);
    let exhausted = ExecutionLimits {
        formal_defeats: 3,
        ..limits
    };
    assert_eq!(
        solve_aspic(&payload, &exhausted),
        Err(AspicError::Incomplete(
            "Theory exceeds the defeat-witness budget".into()
        ))
    );
}

#[test]
fn extension_exhaustion_is_typed_and_has_no_partial_result() {
    let limits = ExecutionLimits {
        extension_search: 1,
        ..ExecutionLimits::default()
    };
    assert_eq!(
        solve_aspic(&json!({"theory":pair()}), &limits),
        Err(AspicError::Incomplete(
            "Extension enumeration budget exceeded".into()
        ))
    );
}

#[test]
fn finite_cycles_cannot_supply_their_own_missing_premise() {
    let theory = json!({"premises":[{"atom":"p","kind":"ordinary","rank":500}],"rules":[{"id":"r_q","kind":"strict","antecedents":["p"],"consequent":"q"},{"id":"r_p","kind":"strict","antecedents":["q"],"consequent":"p"},{"id":"r_y","kind":"strict","antecedents":["x"],"consequent":"y"},{"id":"r_x","kind":"strict","antecedents":["y"],"consequent":"x"}],"contraries":[],"goal":"x"});
    let result = solve_aspic(&json!({"theory":theory}), &ExecutionLimits::default()).unwrap();
    assert_eq!(result["argument_count"], 2);
    assert_eq!(result["query_status"], "unconstructed");
    assert_eq!(result["arguments"][1]["conclusion"], "q");
}

#[test]
fn captured_json_limits_apply_before_the_fixed_theory_schema() {
    let limits = ExecutionLimits {
        json_depth: 1,
        ..ExecutionLimits::default()
    };
    assert_eq!(
        validate_theory(&pair(), &limits),
        Err(AspicError::Incomplete(
            "ASPIC JSON resource limit exceeded".into()
        ))
    );
}

#[test]
fn raised_host_depth_cannot_overflow_a_small_callers_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let limits = ExecutionLimits {
                json_depth: usize::MAX,
                ..ExecutionLimits::default()
            };
            let mut nested = Value::Null;
            for _ in 0..5000 {
                nested = Value::Array(vec![nested]);
            }
            let mut object = serde_json::Map::new();
            object.insert("theory".into(), nested);
            let payload = Value::Object(object);
            assert!(matches!(
                validate_theory(&payload["theory"], &limits),
                Err(AspicError::Incomplete(_))
            ));
            assert!(matches!(
                solve_aspic(&payload, &limits),
                Err(AspicError::Incomplete(_))
            ));
            // Destruction of a forged recursive Value belongs to its constructing
            // caller and is independent of the solver's bounded traversal.
            std::mem::forget(payload);
        })
        .unwrap()
        .join()
        .unwrap();
}
