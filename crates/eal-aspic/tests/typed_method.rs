//! Typed EAL observations, registry dispatch and authored evaluation agree with Python.
use eal_aspic::registry::aspic_registry;
use eal_core::{ExecutionLimits, composition, evaluation, methods::default_registry, validation};
use serde_json::{Value, json};

fn normalise_contract(contract: &mut Value) {
    if let Some(contract) = contract.as_object_mut() {
        for key in [
            "implementation_version",
            "implementation_source_digest",
            "implementation_code_digest",
            "implementation_artifact_digest",
            "identity_basis",
        ] {
            contract.remove(key);
        }
        if matches!(
            contract.get("execution").and_then(Value::as_str),
            Some("bounded_process" | "trusted_native_callback")
        ) {
            contract.insert("execution".into(), json!("registered_callback"));
        }
    }
}

// Only host-owned descriptors at these two known result locations are backend metadata.
fn normalise_backend_metadata(result: &mut Value) {
    result
        .as_object_mut()
        .unwrap()
        .remove("method_registry_fingerprint");
    for argument in result["arguments"].as_object_mut().unwrap().values_mut() {
        if let Some(computation) = argument.get_mut("reasoning_result") {
            if let Some(contract) = computation.get_mut("method_contract") {
                normalise_contract(contract);
            }
            if let Some(contract) = computation
                .get_mut("binding")
                .and_then(|binding| binding.get_mut("method_contract"))
            {
                normalise_contract(contract);
            }
        }
    }
}

fn difference(expected: &Value, actual: &Value, path: &str) -> Option<String> {
    if expected == actual {
        return None;
    }
    match (expected, actual) {
        (Value::Object(expected), Value::Object(actual)) => {
            if expected.len() != actual.len()
                || expected.keys().any(|key| !actual.contains_key(key))
            {
                return Some(format!("{path}: object fields differ"));
            }
            for (key, expected) in expected {
                if let Some(error) = difference(expected, &actual[key], &format!("{path}/{key}")) {
                    return Some(error);
                }
            }
            None
        }
        (Value::Array(expected), Value::Array(actual)) => {
            if expected.len() != actual.len() {
                return Some(format!("{path}: array lengths differ"));
            }
            for (index, (expected, actual)) in expected.iter().zip(actual).enumerate() {
                if let Some(error) = difference(expected, actual, &format!("{path}/{index}")) {
                    return Some(error);
                }
            }
            None
        }
        _ => Some(format!("{path}: expected {expected}, observed {actual}")),
    }
}

fn checked_budget_diagnostic(result: &mut Value) {
    const MESSAGE: &str = "Theory exceeds the constructed-argument budget";
    assert_eq!(result["complete"], false);
    let argument = &mut result["arguments"]["formal_arg"];
    assert_eq!(argument["status"], "unsupported");
    let computation = &mut argument["reasoning_result"];
    assert_eq!(computation["status"], "incomplete");
    assert_eq!(computation["details"], json!({}));
    let reasons = computation["reasons"].as_array().unwrap();
    assert_eq!(reasons.len(), 1);
    let diagnostic = reasons[0].as_str().unwrap().to_owned();
    assert!(diagnostic.ends_with(MESSAGE), "{diagnostic}");
    assert!(
        diagnostic.starts_with("Method argumentation/aspic/2: BudgetExceeded: ")
            || diagnostic
                .starts_with("Method argumentation/aspic/2: Method execution incomplete: ")
    );
    // The retained Python worker class prefix and native error presentation differ;
    // validate that single diagnostic before comparing every remaining result field.
    computation.as_object_mut().unwrap().remove("reasons");
    let reasons = argument["reasons"].as_array_mut().unwrap();
    assert_eq!(
        reasons
            .iter()
            .filter(|reason| reason.as_str() == Some(&diagnostic))
            .count(),
        1
    );
    reasons.retain(|reason| reason.as_str() != Some(&diagnostic));
}

#[test]
fn typed_source_query_scope_age_and_incomplete_match_retained_python() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/aspic/method-reference.json"
    ))
    .unwrap();
    assert_eq!(
        fixture["source_revision"],
        "e2d6f346a37f92483b9f9a13923b98feb053447b"
    );
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 5);
    let registry = aspic_registry();
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let limits: ExecutionLimits = serde_json::from_value(case["limits"].clone()).unwrap();
        let program =
            composition::compile(case["source"].as_str().unwrap(), None, "", limits).unwrap();
        assert_eq!(program.source_digest, case["source_sha256"]);
        let diagnostics = validation::validate(&program, &registry);
        assert_eq!(
            serde_json::to_value(diagnostics).unwrap(),
            case["diagnostics"],
            "{name}"
        );
        assert!(!validation::validate(&program, default_registry()).is_empty());
        let mut actual = evaluation::evaluate(
            &program,
            &case["records"],
            case["now"].as_str().unwrap(),
            &case["context"],
            &registry,
            None,
        )
        .unwrap();
        assert_eq!(
            actual["method_registry_fingerprint"],
            registry.fingerprint()
        );
        let mut expected = case["evaluation"].clone();
        if name == "typed-construction-incomplete" {
            checked_budget_diagnostic(&mut actual);
            checked_budget_diagnostic(&mut expected);
        }
        normalise_backend_metadata(&mut actual);
        normalise_backend_metadata(&mut expected);
        assert!(
            difference(&expected, &actual, "").is_none(),
            "{name}: {}",
            difference(&expected, &actual, "").unwrap_or_default()
        );
        if name == "typed-theory-accepted" {
            let computation = &actual["arguments"]["formal_arg"]["reasoning_result"];
            assert_eq!(actual["claims"]["run_passes"]["status"], "supported");
            assert_eq!(
                computation["binding"]["formal_query"],
                case["records"]["formal"]["value"]["payload"]
            );
        } else {
            assert_eq!(actual["claims"]["run_passes"]["status"], "unsupported");
        }
    }
}

#[test]
fn identity_normalisation_does_not_strip_literal_payload_members() {
    let literal = json!({"implementation_version":"user value","identity_basis":"literal payload"});
    let mut result = json!({"arguments":{"a":{"reasoning_result":{"details":{"method_contract":literal.clone()},"binding":{"proposition":{"method_contract":literal.clone()}}}}},"evidence":{"method_contract":literal.clone()}});
    normalise_backend_metadata(&mut result);
    assert_eq!(
        result["arguments"]["a"]["reasoning_result"]["details"]["method_contract"],
        literal
    );
    assert_eq!(
        result["arguments"]["a"]["reasoning_result"]["binding"]["proposition"]["method_contract"],
        literal
    );
    assert_eq!(result["evidence"]["method_contract"], literal);
}
