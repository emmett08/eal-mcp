use eal_collector_contracts::{
    AcquisitionRequest, Catalogue, CommandRegistry, ObservationEnvelope,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn catalogue() -> Catalogue {
    serde_json::from_str(include_str!("../../../contracts/collectors/native.json")).unwrap()
}

#[test]
fn catalogue_versions_and_template_authority_are_checked() {
    let original = catalogue();
    original.validate().unwrap();
    let mut changed = original.clone();
    changed.schema = "eal.collector-catalogue/2".into();
    assert!(changed.validate().is_err());
    let mut changed = original.clone();
    changed.collectors.push(changed.collectors[0].clone());
    assert!(changed.validate().is_err());
    for token in ["{unknown}", "--name={name}", "{operator.}", "{name}"] {
        let mut changed = original.clone();
        changed.collectors[0].command.argv[0] = token.into();
        assert!(changed.validate().is_err(), "{token}");
    }
    let mut changed = original.clone();
    changed.collectors[2].input_schema["properties"]["start_unix"]["type"] = json!("string");
    assert!(changed.validate().is_err());
    let descriptor = &original.collectors[0];
    let parameters = BTreeMap::from([
        ("inventory".into(), "/operator/inventory.json".into()),
        ("resource".into(), "applications.argoproj.io".into()),
        ("kind".into(), "Application".into()),
        ("namespace".into(), "argocd".into()),
        ("name".into(), "sync-operator".into()),
    ]);
    assert!(descriptor.binding(&parameters, &BTreeMap::new()).is_err());
    let binding = descriptor
        .binding(
            &parameters,
            &BTreeMap::from([
                ("executable".into(), "/operator/kubectl".into()),
                (
                    "supervisor".into(),
                    "/operator/eal-command-supervisor".into(),
                ),
            ]),
        )
        .unwrap();
    let registry = CommandRegistry {
        tools: BTreeMap::from([("resource_read".into(), binding)]),
    };
    let encoded = registry.to_toml().unwrap();
    let parsed: toml::Value = toml::from_str(&encoded).unwrap();
    assert_eq!(
        parsed["tools"]["resource_read"]["kind"].as_str(),
        Some("command")
    );
    assert!(
        parsed["tools"]["resource_read"]["argv"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str() == Some("/operator/kubectl"))
    );
}

#[test]
fn published_schemas_accept_current_models_and_reject_unknown_fields() {
    let cases = [
        (
            include_str!("../../../contracts/schemas/collector-catalogue.schema.json"),
            serde_json::to_value(catalogue()).unwrap(),
        ),
        (
            include_str!("../../../contracts/schemas/acquisition-request.schema.json"),
            json!({"evidence_id":"reading","environment":"lab","tool":"instrument","tool_version":"1","input":{"channels":[1,2],"voltage":1.25},"context":{"rig":"bench-1"}}),
        ),
        (
            include_str!("../../../contracts/schemas/observation-envelope.schema.json"),
            json!({"value":{"healthy":false},"observed_at":"2026-10-24T10:00:00Z","context":{"rig":"bench-1"},"request":{"tool":"instrument"}}),
        ),
    ];
    for (schema, instance) in cases {
        let schema: Value = serde_json::from_str(schema).unwrap();
        let validator = jsonschema::validator_for(&schema).unwrap();
        validator.validate(&instance).unwrap();
        let mut invalid = instance;
        invalid["unknown"] = json!(true);
        assert!(!validator.is_valid(&invalid));
    }
    let request: AcquisitionRequest = serde_json::from_value(json!({"evidence_id":"reading","environment":"lab","tool":"instrument","tool_version":"1","input":{"channels":[1,2],"voltage":1.25},"context":{"rig":"bench-1"}})).unwrap();
    request.validate().unwrap();
    let mut scalar_request = request.clone();
    scalar_request.input = json!(42);
    scalar_request.tool_version = "v".repeat(1025);
    scalar_request.validate().unwrap();
    let observation: ObservationEnvelope =
        serde_json::from_value(json!({"value":{"healthy":false}})).unwrap();
    assert_eq!(observation.value["healthy"], false);
    let mut invalid = request;
    invalid.tool = String::new();
    assert!(invalid.validate().is_err());
}
