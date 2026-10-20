//! Compare independently captured Python semantics without running Python.
use eal_core::{ExecutionLimits, composition, evaluation, methods, semantic, validation};
use serde_json::{Map, Value, json};

fn literal(value: &Value) -> Value {
    match value["type"].as_str().expect("typed lexical JSON value") {
        "null" => Value::Null,
        "boolean" | "string" => value["value"].clone(),
        "number" => serde_json::from_str(value["value"].as_str().unwrap()).unwrap(),
        "array" => Value::Array(
            value["value"]
                .as_array()
                .unwrap()
                .iter()
                .map(literal)
                .collect(),
        ),
        "object" => Value::Object(
            value["value"]
                .as_array()
                .unwrap()
                .iter()
                .map(|entry| (entry[0].as_str().unwrap().to_owned(), literal(&entry[1])))
                .collect(),
        ),
        other => panic!("Unexpected typed JSON literal {other}"),
    }
}

fn operator(value: &Value) -> &str {
    match value.as_str().unwrap() {
        "or" => "or",
        "and" => "and",
        "equal" => "==",
        "not_equal" => "!=",
        "less_equal" => "<=",
        "greater_equal" => ">=",
        "less" => "<",
        "greater" => ">",
        "add" | "positive" => "+",
        "subtract" | "negative" => "-",
        "multiply" => "*",
        "divide" => "/",
        "modulo" => "%",
        "not" => "not",
        other => panic!("Unexpected expression operator {other}"),
    }
}

fn expression(value: &Value) -> Value {
    if value.is_null() {
        return Value::Null;
    }
    if value.get("arguments").is_some() && value.get("value").is_some() {
        return value.clone();
    }
    match value["kind"].as_str().expect("typed expression node") {
        "literal" => json!({"kind":"literal","value":literal(&value["value"]),"arguments":[]}),
        "field" => json!({"kind":"field","value":value["name"],"arguments":[]}),
        "call" => {
            json!({"kind":"call","value":value["name"],"arguments":value["arguments"].as_array().unwrap().iter().map(expression).collect::<Vec<_>>()})
        }
        "unary" => {
            json!({"kind":"unary","value":operator(&value["operator"]),"arguments":[expression(&value["operand"])]})
        }
        "binary" => {
            json!({"kind":"binary","value":operator(&value["operator"]),"arguments":[expression(&value["left"]),expression(&value["right"])]})
        }
        other => panic!("Unexpected expression node {other}"),
    }
}

fn normalise_predicate(predicate: &mut Value) {
    if let Some(value) = predicate.get_mut("expression") {
        *value = expression(value);
    }
}

fn normalise_predicates(program: &mut Value) {
    for group in ["environments", "evidence", "reasoning"] {
        for entity in program[group].as_object_mut().unwrap().values_mut() {
            for predicate in entity["predicates"].as_array_mut().unwrap() {
                normalise_predicate(predicate);
            }
        }
    }
    for claim in program["claims"].as_object_mut().unwrap().values_mut() {
        if let Some(result) = claim
            .get_mut("proposition")
            .and_then(|value| value.get_mut("result"))
        {
            normalise_predicate(result);
        }
    }
}

fn compiled_view(program: &semantic::Program) -> Value {
    let full = serde_json::to_value(program).unwrap();
    let mut projection = Map::new();
    for name in [
        "language",
        "source_digest",
        "environments",
        "tools",
        "evidence",
        "assumptions",
        "reasoning",
        "claims",
        "arguments",
        "objections",
        "duplicates",
        "declaration_count",
        "generated",
        "imports",
        "source_files",
        "locations",
        "argumentation_directives",
    ] {
        projection.insert(name.to_owned(), full[name].clone());
    }
    projection.insert(
        "pattern_names".into(),
        json!(program.patterns.keys().collect::<Vec<_>>()),
    );
    projection.insert(
        "application_names".into(),
        json!(program.applications.keys().collect::<Vec<_>>()),
    );
    projection.insert(
        "lowering_diagnostic_codes".into(),
        json!(
            program
                .lowering_diagnostics
                .iter()
                .map(|item| &item.code)
                .collect::<Vec<_>>()
        ),
    );
    let mut value = Value::Object(projection);
    normalise_predicates(&mut value);
    value
}

fn normalise_backend_metadata(evaluation: &mut Value) {
    evaluation
        .as_object_mut()
        .unwrap()
        .remove("method_registry_fingerprint");
    for argument in evaluation["arguments"]
        .as_object_mut()
        .unwrap()
        .values_mut()
    {
        if let Some(contract) = argument
            .get_mut("reasoning_result")
            .and_then(|value| value.get_mut("binding"))
            .and_then(|value| value.get_mut("method_contract"))
            .and_then(Value::as_object_mut)
        {
            for name in [
                "implementation_version",
                "implementation_source_digest",
                "implementation_code_digest",
                "implementation_artifact_digest",
                "identity_basis",
            ] {
                contract.remove(name);
            }
            if matches!(
                contract.get("execution").and_then(Value::as_str),
                Some("bounded_builtin" | "bounded_native_builtin")
            ) {
                contract.insert("execution".into(), json!("builtin"));
            }
        }
    }
}

fn number_is_integer(value: &Value) -> bool {
    !value.to_string().contains(['.', 'e', 'E'])
}

// Binary64 implementations of quantiles, normalisation and standard errors may
// differ by a few final rounding bits. No integer, identity, predicate threshold
// or arbitrary JSON payload receives this tolerance.
fn floating_algorithm_path(path: &str) -> bool {
    path.contains("/reasoning_result/details/")
        && [
            "/lower",
            "/upper",
            "/estimate",
            "/standard_error",
            "/treatment_mean",
            "/control_mean",
            "/best_posterior",
        ]
        .iter()
        .any(|suffix| path.ends_with(suffix))
        || path.contains("/reasoning_result/details/posterior/")
}

fn difference(expected: &Value, actual: &Value, path: &str) -> Option<String> {
    if expected == actual {
        return None;
    }
    match (expected, actual) {
        (Value::Object(left), Value::Object(right)) => {
            if left.len() != right.len() || !left.keys().all(|key| right.contains_key(key)) {
                return Some(format!(
                    "{path}: object keys expected {:?}, observed {:?}",
                    left.keys(),
                    right.keys()
                ));
            }
            for (name, value) in left {
                if let Some(problem) = difference(value, &right[name], &format!("{path}/{name}")) {
                    return Some(problem);
                }
            }
            None
        }
        (Value::Array(left), Value::Array(right)) => {
            if left.len() != right.len() {
                return Some(format!(
                    "{path}: array lengths {} != {}",
                    left.len(),
                    right.len()
                ));
            }
            for (index, (left, right)) in left.iter().zip(right).enumerate() {
                if let Some(problem) = difference(left, right, &format!("{path}/{index}")) {
                    return Some(problem);
                }
            }
            None
        }
        (Value::Number(_), Value::Number(_))
            if !number_is_integer(expected) && !number_is_integer(actual) =>
        {
            let left = expected.as_f64().unwrap();
            let right = actual.as_f64().unwrap();
            if left.to_bits() == right.to_bits() {
                return None;
            }
            if floating_algorithm_path(path)
                && left.is_sign_negative() == right.is_sign_negative()
                && left.to_bits().abs_diff(right.to_bits()) <= 8
            {
                return None;
            }
            Some(format!("{path}: floating values {expected} != {actual}"))
        }
        _ => Some(format!("{path}: expected {expected}, observed {actual}")),
    }
}

#[test]
fn comparison_preserves_numeric_types_signed_zero_and_non_algorithm_payloads() {
    assert!(difference(&json!(1), &json!(1.0), "payload").is_some());
    assert!(difference(&json!(true), &json!(1), "payload").is_some());
    assert!(difference(&json!(-0.0), &json!(0.0), "payload").is_some());
    let rounded = json!(f64::from_bits(1.0_f64.to_bits() + 8));
    assert!(
        difference(
            &json!(1.0),
            &rounded,
            "evaluation/arguments/a/reasoning_result/details/lower"
        )
        .is_none()
    );
    assert!(difference(&json!(1.0), &rounded, "evaluation/evidence/e/value/lower").is_some());
    let outside_tolerance = json!(f64::from_bits(1.0_f64.to_bits() + 9));
    assert!(
        difference(
            &json!(1.0),
            &outside_tolerance,
            "evaluation/arguments/a/reasoning_result/details/lower"
        )
        .is_some()
    );
}

#[test]
fn compiled_tables_validation_and_evaluation_match_the_pinned_reference() {
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/semantics/reference.json")).unwrap();
    assert_eq!(
        corpus["source_revision"],
        "e2d6f346a37f92483b9f9a13923b98feb053447b"
    );
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(
        cases.len(),
        93,
        "A corpus change requires an explicit coverage review"
    );
    let mut errors = Vec::new();
    let mut compiled = 0;
    let mut evaluated = 0;
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let limits: ExecutionLimits =
            serde_json::from_value(case.get("limits").cloned().unwrap_or_else(|| json!({})))
                .unwrap();
        let resolver = |path: &str| {
            case["imports"]
                .get(path)
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| format!("No trusted source {path}"))
        };
        let input = composition::compile(
            case["source"].as_str().unwrap(),
            case.get("imports")
                .map(|_| &resolver as &composition::SourceResolver<'_>),
            case["origin"].as_str().unwrap_or(""),
            limits,
        );
        let program = match input {
            Ok(program) => {
                if case.get("error").is_some() {
                    errors.push(format!("{name}: source should fail compilation"));
                    continue;
                }
                program
            }
            Err(error) => {
                if case.get("error").is_none() {
                    errors.push(format!("{name}: unexpected compile failure {error}"));
                }
                continue;
            }
        };
        compiled += 1;
        if let Some(problem) = difference(&case["compiled"], &compiled_view(&program), "compiled") {
            errors.push(format!("{name}: {problem}"));
        }
        let diagnostics = validation::validate(&program, methods::default_registry());
        let diagnostic_view = serde_json::to_value(&diagnostics).unwrap();
        if let Some(problem) = difference(&case["diagnostics"], &diagnostic_view, "diagnostics") {
            errors.push(format!("{name}: {problem}"));
        }
        match evaluation::evaluate(
            &program,
            &case["records"],
            case["now"].as_str().unwrap(),
            &case["context"],
            methods::default_registry(),
            case.get("binding_digests"),
        ) {
            Ok(mut actual) => {
                evaluated += 1;
                let mut expected = case["evaluation"].clone();
                normalise_backend_metadata(&mut actual);
                normalise_backend_metadata(&mut expected);
                if let Some(problem) = difference(&expected, &actual, "evaluation") {
                    errors.push(format!("{name}: {problem}"));
                }
            }
            Err(error) => errors.push(format!("{name}: unexpected evaluation failure {error}")),
        }
    }
    assert_eq!(compiled, 89, "Unexpected compilation coverage");
    assert_eq!(evaluated, 89, "Unexpected evaluation coverage");
    assert!(
        errors.is_empty(),
        "{} cross-component differences:\n{}",
        errors.len(),
        errors.join("\n")
    );
}
