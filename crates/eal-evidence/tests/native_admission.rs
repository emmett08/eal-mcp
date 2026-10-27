//! Candidate discovery validates pinned deployment identities without acquiring evidence.
use eal_collector_contracts::capability::*;
use eal_evidence::{ObservationTime, ResponseFormat, ToolMapping, providers::*};
use serde_json::json;
use std::collections::BTreeMap;

fn configured(transport: NativeTransport) -> NativeProviderConfig {
    let input_schema = json!({"type":"object","properties":{"cluster":{"type":"string"},"resource_uid":{"type":"string"}},"required":["cluster"],"additionalProperties":false});
    let output_schema = json!({"type":"object","properties":{"cluster":{"type":"string"},"resource_uid":{"type":"string"},"n":{"type":"integer"},"complete":{"type":"boolean"},"leak":{"type":"string"}},"required":["cluster","resource_uid","n","complete"],"additionalProperties":false});
    let kind = match transport {
        NativeTransport::Command { .. } => TransportKind::Command,
        NativeTransport::HttpGet { .. } => TransportKind::Http,
        NativeTransport::RetainedArtifact { .. } => TransportKind::RetainedArtifact,
    };
    let mut config = NativeProviderConfig {
        capability: CapabilityDescriptor {
            id: "native_measurement".into(),
            version: "1".into(),
            description: "Qualified native inventory".into(),
            provider: ProviderIdentity {
                id: "native-provider".into(),
                version: "1".into(),
                transport: kind,
                connection: None,
                tool: None,
                authentication_scope: "development-reader".into(),
                configuration_sha256: "0".repeat(64),
            },
            input_schema_sha256: schema_identity(&input_schema).unwrap(),
            input_schema,
            output_schema_sha256: schema_identity(&output_schema).unwrap(),
            output_schema,
            semantics: Some(MeasurementSemantics {
                qualification_reference: "inventory-protocol/1".into(),
                subject_kinds: vec!["resource".into()],
                methods: vec!["inventory.authoritative/1".into()],
                support_types: vec![SupportType::Observation],
                identity_dimensions: vec!["cluster".into(), "resource_uid".into()],
                scope_constraints: BTreeMap::new(),
                scope_bindings: BTreeMap::from([("cluster".into(), "/cluster".into())]),
                quantities: vec![MeasurementQuantity {
                    name: "count".into(),
                    unit: "1".into(),
                    output_pointer: "/n".into(),
                }],
                coverage: vec![CoverageGuarantee {
                    strength: LogicalStrength::CompleteEnumeration,
                    population: Some("resources".into()),
                    population_quantity: None,
                    completeness_pointer: Some("/complete".into()),
                    sample_size_pointer: None,
                }],
                time: TimeSemantics {
                    historical_intervals: false,
                    historical_input: None,
                    original_observation_time: true,
                    uncertainty_seconds: Some(1),
                },
                provenance: ProvenanceSemantics {
                    authorities: vec!["native-instrument".into()],
                    transformation_history: true,
                },
                limitations: vec![],
            }),
            configuration_requirements: vec![],
            limits: CapabilityLimits {
                timeout_seconds: 3,
                output_bytes: 65536,
                concurrency: 1,
                requests_per_minute: None,
            },
            costs: CapabilityCosts::unknown("not measured"),
        },
        transport,
        mapping: ToolMapping {
            tool: "native_measurement".into(),
            raw_output_schema_sha256: None,
            response_format: ResponseFormat::Structured,
            value_pointer: "/measurement".into(),
            observation_time: ObservationTime::Pointer {
                pointer: "/observed_at".into(),
            },
            scope_pointers: BTreeMap::from([
                ("cluster".into(), "/cluster".into()),
                ("resource_uid".into(), "/resource_uid".into()),
            ]),
        },
        pinned_files: vec![],
        credential_env: vec![],
    };
    pin_config(&mut config);
    config
}
fn pin_config(config: &mut NativeProviderConfig) {
    config.capability.provider.configuration_sha256 = config.configuration_identity().unwrap();
    config.validate().unwrap();
}

#[test]
fn optional_native_discovery_does_not_invoke_the_selected_endpoint() {
    let config = configured(NativeTransport::HttpGet {
        url: "http://127.0.0.1:9/never-called".into(),
        query_bindings: BTreeMap::new(),
        token_env: None,
    });
    let source = NativeSources::new(vec![config], BTreeMap::new()).unwrap();
    source.admit("native_measurement").unwrap();
    assert_eq!(source.configurations().count(), 1);
    assert!(source.admit("unknown").is_err());
}

#[test]
fn retained_candidate_checks_pinned_bytes_without_creating_an_observation() {
    use sha2::{Digest, Sha256};
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("snapshot.json");
    let bytes = b"{}";
    std::fs::write(&path, bytes).unwrap();
    let config = configured(NativeTransport::RetainedArtifact {
        path: path.clone(),
        sha256: format!("{:x}", Sha256::digest(bytes)),
    });
    let source = NativeSources::new(vec![config], BTreeMap::new()).unwrap();
    source.admit("native_measurement").unwrap();
    std::fs::write(&path, b"{\"changed\":true}").unwrap();
    assert!(source.admit("native_measurement").is_err());
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
}
