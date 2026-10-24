//! Independent boundary cases for the authoritative contract and named profile.
use eal_collector_contracts::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;
fn lab() -> Catalogue {
    serde_json::from_value(json!({"schema":CATALOGUE_SCHEMA,"acquisition_contract":ACQUISITION_CONTRACT,"collectors":[{
        "id":"lab.calibration","version":"1","purpose":"Measure calibration error for a laboratory instrument.",
        "parameters":["channel","start","end"],"input_schema":{"type":"object","additionalProperties":false,"properties":{"channel":{"type":"integer"},"start":{"type":"integer"},"end":{"type":"integer"}},"required":["channel","start","end"]},
        "result_schema":{"type":"object"},"command":{"argv":["{operator.instrument}","{channel}"],"inherit_env":["LAB_TOKEN"],"timeout_seconds":30,"max_output_bytes":4096,"parallel_safe":false},
        "acquisition_predicate":"collected == true","constraints":[{"kind":"ordered_window","start":"start","end":"end","max_seconds":30}]
    }]})).unwrap()
}
#[test]
fn independent_registry_corpus_admits_exactly_declared_cases() {
    for case in serde_json::from_str::<Vec<Value>>(REGISTRY_CORPUS).unwrap() {
        assert_eq!(
            RegistryDocument::from_toml(case["registry"].as_str().unwrap()).is_ok(),
            case["accepted"].as_bool().unwrap(),
            "{}",
            case["name"]
        );
    }
}
#[test]
fn wire_allowance_and_execution_profile_remain_distinct() {
    let mut catalogue = lab();
    catalogue
        .validate_for(AuthoringProfile::RuntimeExportable)
        .unwrap();
    catalogue.collectors[0].command.inherit_env = (0..65).map(|i| format!("LAB_{i}")).collect();
    catalogue.validate().unwrap();
    assert!(
        catalogue
            .validate_for(AuthoringProfile::RuntimeExportable)
            .is_err()
    );
    catalogue.collectors[0].command.inherit_env = vec!["LAB_TOKEN".into(), "LAB_TOKEN".into()];
    assert!(catalogue.validate().is_err());
}
#[test]
fn executable_bindings_require_resolved_operator_authority() {
    let descriptor = &lab().collectors[0];
    let parameters = BTreeMap::from([("channel".into(), "3".into())]);
    let prepared = descriptor
        .prepare_binding(&parameters, &["start".into(), "end".into()])
        .unwrap();
    let prepared = PreparedCommandRegistry {
        tools: BTreeMap::from([("measurement".into(), prepared)]),
    }
    .to_toml()
    .unwrap();
    assert!(RegistryDocument::from_toml(&prepared).is_err());
    let binding = descriptor
        .binding(
            &parameters,
            &BTreeMap::from([("instrument".into(), "/usr/bin/instrument".into())]),
        )
        .unwrap();
    let executable = CommandRegistry {
        tools: BTreeMap::from([("measurement".into(), binding)]),
    }
    .to_toml()
    .unwrap();
    RegistryDocument::from_toml(&executable).unwrap();
}
#[test]
fn schema_and_predicate_admission_precede_authoring() {
    for reference in [
        "https://untrusted.invalid/schema",
        "file:///secret",
        "other.json#/$defs/value",
    ] {
        let mut catalogue = lab();
        catalogue.collectors[0].result_schema = json!({"$ref":reference});
        assert!(catalogue.validate().is_err());
    }
    let mut catalogue = lab();
    catalogue.collectors[0].input_schema["properties"]["channel"] = json!({"type":"invented"});
    assert!(catalogue.validate().is_err());
    for predicate in [
        "healthy == true and complete == true",
        "healthy == true\nclaim injected {}",
        "healthy == null",
        "healthy == []",
        "healthy == 1e9999",
    ] {
        assert!(checked_predicate(predicate).is_err(), "{predicate}");
    }
    for predicate in [
        "healthy == true",
        "measurement.error <= 0.02",
        "label == \"calibrated\"",
    ] {
        checked_predicate(predicate).unwrap();
    }
    let descriptor = &lab().collectors[0];
    descriptor
        .validate_input(&json!({"channel":3,"start":10,"end":30}))
        .unwrap();
    for input in [
        json!({"channel":null,"start":10,"end":30}),
        json!({"channel":3,"start":10,"end":10}),
        json!({"channel":3,"start":10,"end":41}),
        json!({"channel":3,"start":-1,"end":5}),
    ] {
        assert!(descriptor.validate_input(&input).is_err());
    }
    descriptor
        .validate_prepared_input(
            &json!({"channel":3,"start":null,"end":30}),
            &["start".into()],
        )
        .unwrap();
    assert!(
        descriptor
            .validate_prepared_input(&json!({"channel":null,"start":10,"end":30}), &[])
            .is_err()
    );
}
#[test]
fn strict_json_retains_exact_integers_and_rejects_ambiguous_or_excessive_values() {
    let limits = JsonLimits {
        max_bytes: 1024,
        max_depth: 8,
        max_nodes: 20,
    };
    let exact = b"{\"n\":184467440737095516161234}";
    assert_eq!(
        strict_json(exact, limits).unwrap()["n"].to_string(),
        "184467440737095516161234"
    );
    for bytes in [
        b"{\"status\":false,\"status\":true}".as_slice(),
        b"[NaN]",
        b"[1e9999]",
        b"\"\\uD800\"",
        b"[[[[[[[[[[0]]]]]]]]]]",
    ] {
        assert!(strict_json(bytes, limits).is_err());
    }
    assert!(
        strict_json(
            b"[1,2,3]",
            JsonLimits {
                max_nodes: 3,
                ..limits
            }
        )
        .is_err()
    );
    assert!(
        strict_json(
            b"1234",
            JsonLimits {
                max_bytes: 3,
                ..limits
            }
        )
        .is_err()
    );
}
#[test]
fn operator_classification_cannot_be_supplied_by_a_catalogue_or_downgrade_credentials() {
    let mut policy = OperatorEnvironmentPolicy {
        schema: OPERATOR_ENVIRONMENT_POLICY.into(),
        forward: vec!["LAB_MODE".into(), "LAB_TOKEN".into()],
        public: vec!["LAB_MODE".into()],
    };
    policy.validate().unwrap();
    assert!(!policy.sensitive("LAB_MODE"));
    assert!(policy.sensitive("LAB_TOKEN"));
    policy.public.push("LAB_TOKEN".into());
    assert!(policy.validate().is_err());
    policy.public = vec!["NOT_FORWARDED".into()];
    assert!(policy.validate().is_err());
    policy.public = vec!["LAB_MODE".into(), "LAB_MODE".into()];
    assert!(policy.validate().is_err());
    let mut catalogue = serde_json::to_value(lab()).unwrap();
    catalogue["public_env"] = json!(["LAB_TOKEN"]);
    assert!(serde_json::from_value::<Catalogue>(catalogue).is_err());
}
#[test]
fn runtime_allowance_edges_and_pinned_identity_are_directly_validated() {
    let command = |extra: &str| {
        format!(
            "[tools.instrument]\nkind='command'\nversion='1'\nargv=['/usr/bin/instrument']\n{extra}\n"
        )
    };
    for (limit, accepted) in [(64, true), (65, false)] {
        let names = (0..limit)
            .map(|i| format!("'LAB_{i}'"))
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(
            RegistryDocument::from_toml(&command(&format!("inherit_env=[{names}]"))).is_ok(),
            accepted
        );
    }
    for extra in [
        "timeout_seconds=3600\nmax_output_bytes=16777216",
        "timeout_seconds=0.0001\nmax_output_bytes=1",
    ] {
        RegistryDocument::from_toml(&command(extra)).unwrap();
    }
    for extra in [
        "timeout_seconds=3600.1",
        "max_output_bytes=16777217",
        "pinned_files=[{path='x',sha256='wrong'}]",
        "pinned_files=[{path='x',sha256='0000000000000000000000000000000000000000000000000000000000000000'},{path='x',sha256='0000000000000000000000000000000000000000000000000000000000000000'}]",
    ] {
        assert!(
            RegistryDocument::from_toml(&command(extra)).is_err(),
            "{extra}"
        );
    }
    for token in ["{channel}", "prefix{channel}", "{unknown}"] {
        let mut catalogue = lab();
        catalogue.collectors[0].command.argv[0] = token.into();
        assert!(catalogue.validate().is_err());
    }
    let id = CollectorId::parse("lab.calibration-v2").unwrap();
    assert_eq!(
        serde_json::to_value(id).unwrap(),
        json!("lab.calibration-v2")
    );
    assert!(EnvironmentName::parse("LAB.TOKEN").is_err());
}
#[test]
fn prepared_binding_rejects_duplicate_or_conflicting_unresolved_names() {
    let catalogue = lab();
    let descriptor = &catalogue.collectors[0];
    let parameters = BTreeMap::from([("channel".into(), "3".into())]);
    assert!(
        descriptor
            .prepare_binding(&parameters, &["start".into(), "start".into()])
            .is_err()
    );
    assert!(
        descriptor
            .prepare_binding(&parameters, &["channel".into()])
            .is_err()
    );
    assert!(
        descriptor
            .prepare_binding(&parameters, &["unknown".into()])
            .is_err()
    );
}

#[test]
fn exact_and_binary64_profiles_share_admission_and_preserve_their_number_contracts() {
    for source in [
        "0.12345678901234567890123456789",
        "1.230000000000000000000e+30",
        "-0",
        "-0.0",
        "1e9999",
        "184467440737095516161234",
    ] {
        let expected: Value = serde_json::from_str(source).unwrap();
        let actual = strict_json_exact(source.as_bytes(), Default::default()).unwrap();
        assert_eq!(actual.to_string(), expected.to_string(), "{source}");
    }
    assert_ne!(
        strict_json_exact(b"0.12345678901234567890123456789", Default::default())
            .unwrap()
            .to_string(),
        strict_json(b"0.12345678901234567890123456789", Default::default())
            .unwrap()
            .to_string()
    );
    assert_eq!(
        strict_json(b"-0", Default::default()).unwrap().to_string(),
        "0"
    );
    assert_eq!(
        strict_json(b"1.25", Default::default())
            .unwrap()
            .to_string(),
        "1.25"
    );
    assert!(strict_json(b"1e9999", Default::default()).is_err());
    for source in [
        b"NaN".as_slice(),
        b"Infinity",
        b"-Infinity",
        b"{\"x\":1,\"x\":2}",
        b"\"\\uD800\"",
    ] {
        assert!(strict_json(source, Default::default()).is_err());
        assert!(strict_json_exact(source, Default::default()).is_err());
    }
    let limits = JsonLimits {
        max_bytes: 4,
        max_depth: 2,
        max_nodes: 2,
    };
    for source in [b"12345".as_slice(), b"[[[]]]", b"[1,2]"] {
        assert!(strict_json_exact(source, limits).is_err());
        assert!(strict_json(source, limits).is_err());
    }
}

#[test]
fn catalogue_corpus_aligns_schema_expressible_checks_and_names_semantic_profiles() {
    let schema: Value = serde_json::from_str(include_str!(
        "../../../contracts/schemas/collector-catalogue.schema.json"
    ))
    .unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    for case in serde_json::from_str::<Vec<Value>>(CATALOGUE_CORPUS).unwrap() {
        let decoded = serde_json::from_value::<Catalogue>(case["catalogue"].clone());
        let wire = decoded
            .as_ref()
            .is_ok_and(|catalogue| catalogue.validate().is_ok());
        let profile = decoded.as_ref().is_ok_and(|catalogue| {
            catalogue
                .validate_for(AuthoringProfile::RuntimeExportable)
                .is_ok()
        });
        assert_eq!(
            wire,
            case["wire_accepted"].as_bool().unwrap(),
            "wire {}",
            case["name"]
        );
        assert_eq!(
            validator.is_valid(&case["catalogue"]),
            case["schema_accepted"].as_bool().unwrap(),
            "schema {}",
            case["name"]
        );
        assert_eq!(
            profile,
            case["profile_accepted"].as_bool().unwrap(),
            "profile {}",
            case["name"]
        );
        if case["scope"] == "schema-and-semantic" {
            assert_eq!(
                wire,
                validator.is_valid(&case["catalogue"]),
                "{}",
                case["name"]
            );
        }
    }
}
