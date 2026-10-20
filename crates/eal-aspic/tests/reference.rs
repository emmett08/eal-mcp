//! Exact development-oracle observations; Python is never invoked by these tests.
use eal_aspic::{AspicError, solve_aspic, validate_theory};
use eal_core::ExecutionLimits;
use serde_json::Value;

fn difference(expected: &Value, actual: &Value, path: &str) -> Option<String> {
    if expected == actual {
        return None;
    }
    match (expected, actual) {
        (Value::Object(left), Value::Object(right)) => {
            if left.len() != right.len() || !left.keys().all(|key| right.contains_key(key)) {
                return Some(format!(
                    "{path}: expected keys {:?}, observed {:?}",
                    left.keys(),
                    right.keys()
                ));
            }
            left.iter()
                .find_map(|(key, value)| difference(value, &right[key], &format!("{path}/{key}")))
        }
        (Value::Array(left), Value::Array(right)) => {
            if left.len() != right.len() {
                return Some(format!(
                    "{path}: expected {} elements, observed {}",
                    left.len(),
                    right.len()
                ));
            }
            left.iter()
                .zip(right)
                .enumerate()
                .find_map(|(index, (left, right))| {
                    difference(left, right, &format!("{path}/{index}"))
                })
        }
        _ => Some(format!("{path}: expected {expected}, observed {actual}")),
    }
}

fn error_difference(expected: &Value, error: &AspicError, path: &str) -> Option<String> {
    if expected["ok"] == true {
        return Some(format!("{path}: unexpected error {error}"));
    }
    if expected["error"].as_str().unwrap() != error.to_string() {
        return Some(format!(
            "{path}: expected error {}, observed {error}",
            expected["error"]
        ));
    }
    let incomplete = matches!(error, AspicError::Incomplete(_));
    if incomplete != expected["budget_exceeded"].as_bool().unwrap() {
        return Some(format!(
            "{path}: incorrect invalid/incomplete error classification"
        ));
    }
    None
}

#[test]
fn every_solver_field_validation_message_and_budget_class_matches_pinned_python() {
    let reference: Value =
        serde_json::from_str(include_str!("../../../fixtures/aspic/reference.json")).unwrap();
    assert_eq!(
        reference["source_revision"],
        "e2d6f346a37f92483b9f9a13923b98feb053447b"
    );
    let cases = reference["cases"].as_array().unwrap();
    assert_eq!(
        cases.len(),
        149,
        "A corpus change requires an explicit coverage review"
    );
    let mut problems = Vec::new();
    let (mut valid, mut solved, mut incomplete) = (0, 0, 0);
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let limits: ExecutionLimits = serde_json::from_value(case["limits"].clone()).unwrap();
        let expected = &case["validation"];
        let problem = match validate_theory(&case["validation_target"], &limits) {
            Ok(()) => {
                valid += 1;
                (expected["ok"] != true).then(|| "validation unexpectedly succeeded".to_owned())
            }
            Err(error) => error_difference(expected, &error, "validation"),
        };
        if let Some(problem) = problem {
            problems.push(format!("{name}: {problem}"));
        }
        let expected = &case["solve"];
        let problem = match solve_aspic(&case["payload"], &limits) {
            Ok(value) => {
                solved += 1;
                if expected["ok"] != true {
                    Some("solve unexpectedly succeeded".to_owned())
                } else {
                    difference(&expected["value"], &value, "solve")
                }
            }
            Err(error) => {
                incomplete += usize::from(matches!(error, AspicError::Incomplete(_)));
                error_difference(expected, &error, "solve")
            }
        };
        if let Some(problem) = problem {
            problems.push(format!("{name}: {problem}"));
        }
    }
    assert!(
        problems.is_empty(),
        "{} exact ASPIC differences:\n{}",
        problems.len(),
        problems.join("\n")
    );
    assert_eq!(
        (valid, solved, incomplete),
        (100, 87, 13),
        "No retained cases may be silently skipped"
    );
}
