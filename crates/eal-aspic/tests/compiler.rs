//! Portable compiler correspondence and independently recomputed native identities.
use eal_aspic::{AspicError, CompileOptions, CompiledTheory, compile_eal_aspic};
use eal_core::{ExecutionLimits, canonical, composition, methods};
use serde_json::{Value, json};

fn corpus() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/aspic/compiler-reference.json"
    ))
    .unwrap()
}
fn compiled(case: &Value) -> Result<CompiledTheory, AspicError> {
    let resolver = |path: &str| {
        case["imports"]
            .get(path)
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| format!("No trusted source {path}"))
    };
    let mut options = CompileOptions::new(
        case["goal"].as_str().unwrap(),
        case["now"].as_str().unwrap(),
        &case["context"],
    );
    options.limits =
        serde_json::from_value(case.get("limits").cloned().unwrap_or_else(|| json!({}))).unwrap();
    options.binding_digests = case.get("binding_digests");
    options.resolver = case
        .get("imports")
        .map(|_| &resolver as &composition::SourceResolver<'_>);
    options.semantics = case["semantics"].as_str().unwrap_or("grounded");
    options.query_mode = case["query_mode"].as_str().unwrap_or("sceptical");
    options.preference = case["preference"].as_str();
    compile_eal_aspic(case["source"].as_str().unwrap(), &case["records"], options)
}
fn assessment_view(mut value: Value) -> Value {
    value
        .as_object_mut()
        .unwrap()
        .remove("method_registry_fingerprint");
    for argument in value["arguments"].as_object_mut().unwrap().values_mut() {
        if let Some(contract) = argument
            .get_mut("reasoning_result")
            .and_then(|v| v.get_mut("binding"))
            .and_then(|v| v.get_mut("method_contract"))
            .and_then(Value::as_object_mut)
        {
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
                Some("bounded_builtin" | "bounded_native_builtin")
            ) {
                contract.insert("execution".into(), json!("builtin"));
            }
        }
    }
    value
}
fn mapping_view(mut value: Value) -> Value {
    for key in [
        "method_registry_fingerprint",
        "assessment_digest",
        "snapshot_digest",
    ] {
        value.as_object_mut().unwrap().remove(key);
    }
    for key in ["contract_digest", "module_digest"] {
        value["backend"].as_object_mut().unwrap().remove(key);
    }
    value
}
fn projection_view(mut value: Value) -> Value {
    value.as_object_mut().unwrap().remove("snapshot_digest");
    value["source_map"] = mapping_view(value["source_map"].take());
    value
}
fn first_difference(expected: &Value, actual: &Value, path: &str) -> Option<String> {
    match (expected, actual) {
        (Value::Object(a), Value::Object(b)) => {
            if a.len() != b.len() || a.keys().any(|key| !b.contains_key(key)) {
                return Some(format!("{path}: differing object keys"));
            }
            a.iter()
                .find_map(|(key, value)| first_difference(value, &b[key], &format!("{path}/{key}")))
        }
        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                return Some(format!("{path}: array lengths {} != {}", a.len(), b.len()));
            }
            a.iter()
                .zip(b)
                .enumerate()
                .find_map(|(index, (a, b))| first_difference(a, b, &format!("{path}/{index}")))
        }
        _ if expected == actual => None,
        _ => Some(format!("{path}: expected {expected}, observed {actual}")),
    }
}
fn check_identities(snapshot: &CompiledTheory) {
    let theory = snapshot.theory().unwrap();
    let mapping = snapshot.source_map().unwrap();
    let assessment = snapshot.assessment().unwrap();
    let limits = snapshot.limits().unwrap();
    assert_eq!(
        mapping["theory_digest"],
        canonical::canonical_digest(&theory, &limits).unwrap()
    );
    assert_eq!(
        mapping["assessment_digest"],
        canonical::canonical_digest(&assessment, &limits).unwrap()
    );
    assert_eq!(
        mapping["method_registry_fingerprint"],
        methods::default_registry().fingerprint()
    );
    assert_eq!(
        mapping["backend"]["module_digest"],
        eal_aspic::registry::module_identity()
    );
    assert_eq!(
        mapping["backend"]["contract_digest"],
        canonical::canonical_digest(&eal_aspic::aspic_contract().describe(), &limits).unwrap()
    );
    let mut identity = json!({"evidence_record_digests":{}});
    for key in [
        "source_digest",
        "assessed_at",
        "context_fingerprint",
        "method_registry_fingerprint",
        "backend",
        "assessment_digest",
        "theory_digest",
        "execution_limits",
    ] {
        identity[key] = mapping[key].clone();
    }
    for (name, record) in mapping["evidence"].as_object().unwrap() {
        identity["evidence_record_digests"][name] = record["record_digest"].clone();
    }
    assert_eq!(
        mapping["snapshot_digest"],
        canonical::canonical_digest(&identity, &limits).unwrap()
    );
    assert_eq!(mapping["execution_limits"], limits.describe());
}

#[test]
fn compiler_matches_every_pinned_observation_snapshot() {
    let corpus = corpus();
    assert_eq!(
        corpus["source_revision"],
        "e2d6f346a37f92483b9f9a13923b98feb053447b"
    );
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 76);
    let mut errors = Vec::new();
    let mut compiled_count = 0;
    let mut solved_count = 0;
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let snapshot = match compiled(case) {
            Ok(snapshot) => {
                if case["error"]["stage"] == "compile" {
                    errors.push(format!("{name}: invalid compilation succeeded"));
                    continue;
                }
                snapshot
            }
            Err(error) => {
                if case["error"]["stage"] != "compile" {
                    errors.push(format!("{name}: unexpected compilation failure: {error}"));
                } else if error.to_string() != case["error"]["message"] {
                    errors.push(format!("{name}: compile error differs: {error}"));
                }
                continue;
            }
        };
        compiled_count += 1;
        check_identities(&snapshot);
        for (field, expected, actual) in [
            ("theory", case["theory"].clone(), snapshot.theory().unwrap()),
            (
                "source_map",
                mapping_view(case["source_map"].clone()),
                mapping_view(snapshot.source_map().unwrap()),
            ),
            (
                "assessment",
                assessment_view(case["assessment"].clone()),
                assessment_view(snapshot.assessment().unwrap()),
            ),
        ] {
            if let Some(problem) = first_difference(&expected, &actual, field) {
                errors.push(format!("{name}: {problem}"));
            }
        }
        match snapshot.to_dict() {
            Ok(actual) => {
                if case.get("projection").is_none() {
                    errors.push(format!("{name}: invalid solve succeeded"));
                    continue;
                }
                solved_count += 1;
                if let Some(problem) = first_difference(
                    &projection_view(case["projection"].clone()),
                    &projection_view(actual),
                    "projection",
                ) {
                    errors.push(format!("{name}: {problem}"));
                }
            }
            Err(error) => {
                if case["error"]["stage"] != "solve" {
                    errors.push(format!("{name}: unexpected solve failure: {error}"));
                } else if error.to_string() != case["error"]["message"] {
                    errors.push(format!("{name}: solve error differs: {error}"));
                }
            }
        }
    }
    assert!(
        errors.is_empty(),
        "{} differences:\n{}",
        errors.len(),
        errors.join("\n")
    );
    assert_eq!((compiled_count, solved_count), (64, 61));
}

#[test]
fn detached_getters_and_exports_cannot_modify_the_checked_snapshot() {
    let corpus = corpus();
    let case = &corpus["cases"][0];
    let snapshot = compiled(case).unwrap();
    let baseline = snapshot.to_dict().unwrap();
    let mut theory = snapshot.theory().unwrap();
    theory["goal"] = json!("unrelated");
    let mut mapping = snapshot.source_map().unwrap();
    mapping["goal"]["claim"] = json!("reportable");
    let mut assessment = snapshot.assessment().unwrap();
    assessment["claims"]["run_passes"]["status"] = json!("out_of_scope");
    let mut output = snapshot.solve().unwrap();
    output["source_map"]["arguments"]["primary_route"]["emitted"] = json!(false);
    let mut limits = snapshot.limits().unwrap();
    limits.formal_arguments = 1;
    assert_eq!(snapshot.to_dict().unwrap(), baseline);
    check_identities(&snapshot);
}

#[test]
fn unusable_supplied_records_remain_bound_into_snapshot_identity() {
    let corpus = corpus();
    let cases = corpus["cases"].as_array().unwrap();
    let a = compiled(cases.iter().find(|c| c["name"] == "stale-primary").unwrap()).unwrap();
    let b = compiled(
        cases
            .iter()
            .find(|c| c["name"] == "stale-primary-other-run")
            .unwrap(),
    )
    .unwrap();
    assert_eq!(a.theory().unwrap(), b.theory().unwrap());
    let ma = a.source_map().unwrap();
    let mb = b.source_map().unwrap();
    assert_eq!(ma["evidence"]["primary_data"]["identity"], Value::Null);
    assert_ne!(
        ma["evidence"]["primary_data"]["record_digest"],
        mb["evidence"]["primary_data"]["record_digest"]
    );
    assert_ne!(ma["snapshot_digest"], mb["snapshot_digest"]);
}

#[test]
fn captured_formal_budgets_fail_without_an_acceptance_result() {
    let corpus = corpus();
    for name in [
        "premise-budget",
        "rule-budget",
        "antecedent-budget",
        "construction-budget",
        "extension-budget",
        "local-incomplete",
    ] {
        let case = corpus["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["name"] == name)
            .unwrap();
        let error = match compiled(case) {
            Err(error) => error,
            Ok(snapshot) => snapshot.solve().unwrap_err(),
        };
        assert!(
            matches!(error, AspicError::Incomplete(_)),
            "{name}: {error:?}"
        );
    }
    let baseline = &corpus["cases"][0];
    let context = json!({"site":"bench"});
    let mut options =
        CompileOptions::new("run_passes", baseline["now"].as_str().unwrap(), &context);
    options.limits = ExecutionLimits {
        formal_premises: 0,
        ..ExecutionLimits::default()
    };
    assert!(
        compile_eal_aspic(
            baseline["source"].as_str().unwrap(),
            &baseline["records"],
            options
        )
        .is_err()
    );
}
