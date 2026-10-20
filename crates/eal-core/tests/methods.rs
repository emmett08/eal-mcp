//! Reference agreement plus independently calculated positive and adverse cases.
use eal_core::methods::{
    MethodContract, NativeMethodIdentity, assess_mode, default_registry, engineering_registry,
    is_method_identifier, optional_registry, schema_errors, validate_mode,
};
use serde_json::{Value, json};

fn compare(actual: &Value, expected: &Value, path: &str) {
    match (actual, expected) {
        (Value::Number(_), Value::Number(_)) => {
            let integer = |v: &Value| {
                !v.to_string()
                    .bytes()
                    .any(|c| matches!(c, b'.' | b'e' | b'E'))
            };
            if integer(expected) {
                assert!(
                    integer(actual),
                    "{path}: integer output changed to float: {actual}"
                );
                assert_eq!(actual.to_string(), expected.to_string(), "{path}");
            } else {
                let a = actual.as_f64().unwrap();
                let e = expected.as_f64().unwrap();
                if e != 0.0 {
                    assert_ne!(a, 0.0, "{path}: nonzero reference erased");
                }
                assert!(
                    (a - e).abs() <= 5e-13 * a.abs().max(e.abs()).max(1e-300),
                    "{path}: {a} differs from {e}"
                );
            }
        }
        (Value::Object(a), Value::Object(e)) => {
            assert!(
                a.len() == e.len() && a.keys().all(|key| e.contains_key(key)),
                "{path}: fields"
            );
            for (key, v) in e {
                compare(&a[key], v, &format!("{path}.{key}"));
            }
        }
        (Value::Array(a), Value::Array(e)) => {
            assert_eq!(a.len(), e.len(), "{path}: array length");
            for (i, (a, e)) in a.iter().zip(e).enumerate() {
                compare(a, e, &format!("{path}[{i}]"));
            }
        }
        _ => assert_eq!(actual, expected, "{path}"),
    }
}

#[test]
fn python_reference_cases_match_without_python() {
    let reference: Value =
        serde_json::from_str(include_str!("../../../fixtures/methods/reference.json")).unwrap();
    let cases = reference["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 93);
    let optional = engineering_registry();
    for case in cases {
        let actual = assess_mode(
            case["method"].as_str().unwrap(),
            case["evidence"].as_array().unwrap(),
            case["premises"].as_array().unwrap(),
            if case["optional"] == true {
                &optional
            } else {
                default_registry()
            },
        );
        if case["optional"] == true {
            if let Some(error) = case["expected_callback"]["error"].as_str() {
                assert_eq!(actual["status"], "unsupported", "{}", case["name"]);
                assert!(
                    actual["reasons"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|r| r.as_str().unwrap().contains(error)),
                    "{}: {actual}",
                    case["name"]
                );
            } else {
                assert_eq!(actual["status"], "supported", "{}: {actual}", case["name"]);
                compare(
                    &actual["details"],
                    &case["expected_callback"]["details"],
                    case["name"].as_str().unwrap(),
                );
            }
        } else {
            compare(&actual, &case["expected"], case["name"].as_str().unwrap());
        }
    }
}

#[test]
fn optional_methods_are_explicit_and_keep_frozen_contract_semantics() {
    let frozen: Value =
        serde_json::from_str(include_str!("../../../contracts/public-contracts.json")).unwrap();
    let all = engineering_registry();
    assert_eq!(default_registry().describe().as_object().unwrap().len(), 8);
    assert_eq!(all.describe().as_object().unwrap().len(), 12);
    let one = optional_registry(&["engineering/rms/1"]).unwrap();
    assert_eq!(one.describe().as_object().unwrap().len(), 9);
    assert!(default_registry().get("engineering/rms/1").is_none());
    assert!(optional_registry(&["engineering/rms/1", "engineering/rms/1"]).is_err());
    assert!(optional_registry(&["unknown/1"]).is_err());
    for id in [
        "engineering/api-load-criteria/1",
        "engineering/reachability/1",
        "engineering/rms/1",
        "engineering/sampled-negative/1",
    ] {
        let contract = all.get(id).unwrap().describe();
        for field in [
            "identifier",
            "evidence_kind",
            "input_schema",
            "query_schema",
            "output_schema",
            "outputs",
            "quantities",
            "exact_unit",
        ] {
            assert_eq!(
                contract[field], frozen["optional_method_contracts"][id][field],
                "{id}.{field}"
            );
        }
    }
}

#[test]
fn frozen_contract_semantics_match_source_descriptors() {
    let frozen: Value =
        serde_json::from_str(include_str!("../../../contracts/public-contracts.json")).unwrap();
    let described = default_registry().describe();
    assert_eq!(described.as_object().unwrap().len(), 8);
    for (id, contract) in described.as_object().unwrap() {
        for field in [
            "identifier",
            "evidence_kind",
            "input_schema",
            "query_schema",
            "output_schema",
            "outputs",
            "quantities",
            "exact_unit",
        ] {
            assert_eq!(
                contract[field], frozen["builtin_methods"][id][field],
                "{id}.{field}"
            );
        }
    }
}

#[test]
fn identifiers_and_designated_evidence_are_exact() {
    for id in ["deductive/1", "example/queue/2", "a.b-c_1/12"] {
        assert!(is_method_identifier(id));
    }
    for id in [
        "deductive",
        "deductive/0",
        "deductive/01",
        "deductive/1/",
        "/1",
        "2bad/1",
        "a//1",
    ] {
        assert!(!is_method_identifier(id));
    }
    assert!(!validate_mode("deductive/1", &[], default_registry()).is_empty());
    assert!(
        validate_mode(
            "deductive/1",
            &["logical_case".into(), "observation".into()],
            default_registry()
        )
        .is_empty()
    );
    assert!(
        !validate_mode(
            "deductive/1",
            &["logical_case".into(), "logical_case".into()],
            default_registry()
        )
        .is_empty()
    );
}

#[test]
fn schema_subset_distinguishes_booleans_integers_floats_and_enum_types() {
    assert!(!schema_errors(&json!(true), &json!({"type":"number"}), "$").is_empty());
    assert!(!schema_errors(&json!(1.0), &json!({"type":"integer"}), "$").is_empty());
    assert!(!schema_errors(&json!(1.0), &json!({"type":"number","enum":[1]}), "$").is_empty());
    assert!(
        schema_errors(
            &json!([1, true]),
            &json!({"type":"array","items":{"anyOf":[{"type":"integer"},{"type":"boolean"}]}}),
            "$"
        )
        .is_empty()
    );
    let ordered: Value = serde_json::from_str("{\"a\":1,\"b\":2}").unwrap();
    let reversed: Value = serde_json::from_str("{\"b\":2,\"a\":1}").unwrap();
    assert!(schema_errors(&ordered, &json!({"type":"json","enum":[reversed]}), "$").is_empty());
    assert!(
        !schema_errors(
            &json!({"a":true,"b":2}),
            &json!({"type":"json","enum":[ordered]}),
            "$"
        )
        .is_empty()
    );
    let huge: Value = serde_json::from_str(&format!("1{}", "0".repeat(400))).unwrap();
    assert!(schema_errors(&huge, &json!({"type":"integer"}), "$").is_empty());
    assert_eq!(
        schema_errors(&huge, &json!({"type":"integer","maximum":1000}), "$"),
        ["$: number is outside declared bounds"]
    );
    assert_eq!(
        schema_errors(&huge, &json!({"type":"number"}), "$"),
        ["$: int too large to convert to float"]
    );
    assert!(eal_core::methods::validate_schema(&json!({"type":"integer","enum":[huge]})).is_ok());
}

#[test]
fn caller_limits_cannot_remove_schema_stack_ceiling() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let mut value = json!(0);
            for _ in 0..150 {
                value = json!([value]);
            }
            let limits = eal_core::ExecutionLimits {
                json_depth: usize::MAX,
                ..Default::default()
            };
            let errors = eal_core::methods::schema_errors_with_limits(
                &value,
                &json!({"type":"json"}),
                "$",
                &limits,
            );
            assert!(errors.iter().any(|error| error.contains("resource limit")));
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn registry_extensions_are_immutable_and_cannot_replace_builtins() {
    fn run(
        v: &Value,
        _: &eal_core::ExecutionLimits,
    ) -> Result<Value, eal_core::methods::MethodError> {
        Ok(json!({"answer":v["x"]}))
    }
    let mut contract = MethodContract::native(
        "example/1",
        "example",
        json!({"type":"object","properties":{"x":{"type":"number"}},"required":["x"],"additionalProperties":false}),
        json!({"type":"object","properties":{},"required":[],"additionalProperties":false}),
        json!({"type":"object","properties":{"answer":{"type":"number"}},"required":["answer"],"additionalProperties":false}),
        NativeMethodIdentity {
            version: "test/1".into(),
            artifact_sha256: "a".repeat(64),
        },
        run,
    );
    contract
        .outputs
        .insert("answer".into(), "dimensionless".into());
    contract.quantities.push("dimensionless".into());
    let extension = default_registry().with_method(contract.clone()).unwrap();
    assert!(default_registry().get("example/1").is_none());
    assert!(extension.get("example/1").is_some());
    assert_ne!(extension.fingerprint(), default_registry().fingerprint());
    let mut reordered = contract.clone();
    reordered.input_schema = serde_json::from_str("{\"additionalProperties\":false,\"required\":[\"x\"],\"properties\":{\"x\":{\"type\":\"number\"}},\"type\":\"object\"}").unwrap();
    assert_eq!(
        extension.fingerprint(),
        default_registry()
            .with_method(reordered)
            .unwrap()
            .fingerprint()
    );
    let mut changed = contract.clone();
    changed.implementation_artifact_digest = Some("b".repeat(64));
    let second = default_registry().with_method(changed).unwrap();
    assert_ne!(extension.fingerprint(), second.fingerprint());
    assert!(extension.describe()["example/1"]["implementation_source_digest"].is_null());
    let mut missing = contract.clone();
    missing.implementation_artifact_digest = None;
    assert!(default_registry().with_method(missing).is_err());
    assert!(extension.with_method(contract.clone()).is_err());
    contract.identifier = "deductive/1".into();
    assert!(default_registry().with_method(contract).is_err());
}

#[test]
fn native_callbacks_receive_explicit_limits_and_preserve_incomplete() {
    use eal_core::{
        ExecutionLimits,
        methods::{MethodError, assess_mode_with_limits},
    };
    fn bounded(payload: &Value, limits: &ExecutionLimits) -> Result<Value, MethodError> {
        match payload["x"].as_i64() {
            Some(0) => Err(MethodError::Incomplete(
                "test construction budget exhausted".into(),
            )),
            Some(-1) => Err(MethodError::Invalid("invalid test input".into())),
            _ => Ok(json!({"budget":limits.formal_arguments})),
        }
    }
    let contract = MethodContract::native(
        "example/bounded/1",
        "bounded",
        json!({"type":"object","properties":{"x":{"type":"integer"}},"required":["x"],"additionalProperties":false}),
        json!({"type":"object","properties":{},"required":[],"additionalProperties":false}),
        json!({"type":"object","properties":{"budget":{"type":"integer"}},"required":["budget"],"additionalProperties":false}),
        NativeMethodIdentity {
            version: "test-bounded/1".into(),
            artifact_sha256: "c".repeat(64),
        },
        bounded,
    );
    let registry = default_registry().with_method(contract).unwrap();
    let limits = ExecutionLimits {
        formal_arguments: 3,
        ..Default::default()
    };
    let run = |x| {
        assess_mode_with_limits(
            "example/bounded/1",
            &[json!({"id":"case","kind":"bounded","value":{"x":x}})],
            &[],
            &registry,
            &limits,
        )
    };
    let success = run(1);
    assert_eq!(success["details"]["budget"], 3);
    let incomplete = run(0);
    assert_eq!(incomplete["status"], "incomplete");
    assert_eq!(incomplete["details"], json!({}));
    assert_eq!(
        incomplete["method_contract"]["identifier"],
        "example/bounded/1"
    );
    assert_eq!(incomplete["evidence_id"], "case");
    let invalid = run(-1);
    assert_eq!(invalid["status"], "unsupported");
    assert_eq!(invalid["details"], json!({}));
}

#[test]
fn exact_mean_contrast_keeps_neighbouring_large_integer_effect() {
    let p:Value=serde_json::from_str("{\"assignment\":\"randomised\",\"treatment\":[1152921504606846978,1152921504606846978],\"control\":[1152921504606846977,1152921504606846977]}").unwrap();
    let output = assess_mode(
        "causal/1",
        &[json!({"id":"e","kind":"experiment","value":p})],
        &[],
        default_registry(),
    );
    assert_eq!(output["status"], "supported");
    assert_eq!(output["details"]["estimate"], 1);
    assert_eq!(
        output["details"]["treatment_mean"].to_string(),
        "1152921504606846978"
    );
}

#[test]
fn tiny_standard_error_survives_variance_underflow() {
    let output = assess_mode(
        "causal/1",
        &[
            json!({"id":"e","kind":"experiment","value":{"assignment":"randomised","treatment":[0.0,1e-200],"control":[0.0,0.0]}}),
        ],
        &[],
        default_registry(),
    );
    assert_eq!(output["status"], "supported");
    let se = output["details"]["standard_error"].as_f64().unwrap();
    assert!(se > 0.0);
    assert!((se / 5e-201 - 1.0).abs() < 1e-14);
}
