//! Real subprocess/HTTP/artifact contract tests. The integration executable also
//! exposes the unmodified eal-process supervisor entrypoint to supervised fixtures.
use eal_collector_contracts::capability::*;
use eal_evidence::{
    EvidenceConfig, ObservationTime, ResponseFormat, ToolMapping,
    execution::{CoverageStatus, SourceErrorKind, SourceExecutor},
    plan::{AcquisitionStep, SourceIdentity},
    providers::*,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    time::{Duration, Instant},
};

fn main() {
    let arguments = std::env::args().collect::<Vec<_>>();
    if arguments.len() == 2 && arguments[1].parse::<u128>().is_ok() {
        std::process::exit(eal_process::supervisor::entrypoint());
    }
    if arguments.get(1).is_some_and(|arg| arg == "native-fixture") {
        command_fixture(arguments.get(2).map(String::as_str).unwrap_or("raw"));
        return;
    }
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        retained_preserves_age_hash_and_exact_numbers().await;
        input_identity_and_contract_drift_are_rejected().await;
        command_protocol_and_failures_are_supervised().await;
        http_authority_is_fixed_and_no_redirect_is_followed().await;
        dispatcher_admits_native_catalogues_and_executes().await;
        native_rate_limits_obey_the_deadline().await;
        native_coverage_preserves_enumeration_and_statistics().await;
        native_config_preserves_opaque_context().await;
    });
    println!(
        "native provider contract scenarios passed (8 groups: retained, scope/schema, command, HTTP, dispatcher, rate limits, measured coverage, opaque context)"
    );
}

async fn native_config_preserves_opaque_context() {
    let mut config = command("eal", true);
    let opaque = json!({"nested":[{"$serde_json::private::Number":"123456789012345678901234567890"}],"huge":12345678901234567890u64});
    if let NativeTransport::Command {
        protocol: CommandProtocol::EalAcquisition { context, .. },
        ..
    } = &mut config.transport
    {
        *context = opaque.clone();
    }
    pin_config(&mut config);
    let encoded = serde_json::to_string(&config).unwrap();
    let decoded: NativeProviderConfig = serde_json::from_str(&encoded).unwrap();
    assert_eq!(
        config.configuration_identity().unwrap(),
        decoded.configuration_identity().unwrap()
    );
    let planned = step(&decoded);
    let executor = NativeSources::new(vec![decoded.clone()], BTreeMap::new()).unwrap();
    let request = executor
        .bind(&planned, planned.arguments.clone())
        .await
        .unwrap();
    executor.execute(&request, deadline()).await.unwrap();
    let decoded: NativeProviderConfig =
        serde_json::from_value(serde_json::to_value(&config).unwrap()).unwrap();
    assert_eq!(
        config.configuration_identity().unwrap(),
        decoded.configuration_identity().unwrap()
    );
    if let NativeTransport::Command {
        protocol: CommandProtocol::EalAcquisition { context, .. },
        ..
    } = decoded.transport
    {
        assert_eq!(context, opaque);
    } else {
        panic!("Lost configured EAL protocol");
    }
    let mut value = serde_json::to_value(&config).unwrap();
    value["transport"]["extra"] = json!(true);
    assert!(serde_json::from_value::<NativeProviderConfig>(value).is_err());
}

async fn native_coverage_preserves_enumeration_and_statistics() {
    let (_directory, mut config) = retained();
    config
        .capability
        .semantics
        .as_mut()
        .unwrap()
        .coverage
        .push(CoverageGuarantee {
            strength: LogicalStrength::StatisticalEstimate,
            population: Some("resources".into()),
            population_quantity: None,
            completeness_pointer: None,
            sample_size_pointer: Some("/n".into()),
        });
    let planned = step(&config);
    let executor = NativeSources::new(vec![config], BTreeMap::new()).unwrap();
    let request = executor
        .bind(&planned, planned.arguments.clone())
        .await
        .unwrap();
    let observation = executor.execute(&request, deadline()).await.unwrap();
    assert_eq!(
        observation.coverage,
        CoverageStatus::CompleteEnumerationWithStatistics {
            sample_size: 12345678901234567890u64
        }
    );
}

async fn native_rate_limits_obey_the_deadline() {
    let (_directory, mut config) = retained();
    config.capability.limits.requests_per_minute = Some(1);
    let planned = step(&config);
    let executor = NativeSources::new(vec![config], BTreeMap::new()).unwrap();
    let request = executor
        .bind(&planned, planned.arguments.clone())
        .await
        .unwrap();
    executor.execute(&request, deadline()).await.unwrap();
    let error = executor
        .execute(&request, Instant::now() + Duration::from_millis(20))
        .await
        .unwrap_err();
    assert!(matches!(error.kind, SourceErrorKind::Budget));
}
fn measurement(cluster: &str) -> Value {
    json!({"cluster":cluster,"resource_uid":"discovered-uid","n":12345678901234567890u64,"complete":true})
}
fn envelope(cluster: &str) -> Value {
    json!({"measurement":measurement(cluster),"observed_at":"2026-10-24T12:00:00Z"})
}
fn command_fixture(mode: &str) {
    let mut bytes = Vec::new();
    std::io::stdin().read_to_end(&mut bytes).unwrap();
    let input = eal_collector_contracts::strict_json_exact(&bytes, Default::default()).unwrap();
    if mode == "fail" {
        std::process::exit(7);
    }
    if mode == "oversize" {
        std::io::stdout().write_all(&vec![b'x'; 65537]).unwrap();
        return;
    }
    let mut result = envelope(if mode == "wrong_identity" {
        "prod"
    } else {
        "dev"
    });
    if mode == "eal" || mode == "bad_correspondence" {
        result["request"] = json!({"tool":input["tool"],"tool_version":input["tool_version"],"input":input["input"],"context":input["context"]});
        if mode == "bad_correspondence" {
            result["request"]["input"] = json!({"cluster":"prod"});
        }
    }
    if mode == "credential" {
        result["measurement"]["leak"] = json!(std::env::var("NATIVE_FIXTURE_TOKEN").unwrap());
    }
    println!("{}", result);
}
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
                identifier_audit: None,
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
fn step(config: &NativeProviderConfig) -> AcquisitionStep {
    let descriptor = &config.capability;
    AcquisitionStep {
        id: "source".into(),
        source: SourceIdentity {
            provider_id: descriptor.provider.id.clone(),
            capability_id: descriptor.id.clone(),
            capability_version: descriptor.version.clone(),
            input_schema_sha256: descriptor.input_schema_sha256.clone(),
            output_schema_sha256: descriptor.output_schema_sha256.clone(),
            qualified_contract_sha256: descriptor.measurement_identity().unwrap(),
            configuration_sha256: "0".repeat(64),
            authentication_scope_sha256: "0".repeat(64),
            scope: json!({"cluster":"dev"}),
            historical_interval: None,
        },
        arguments: json!({"cluster":"dev"}),
        bindings: vec![],
        for_each: None,
        availability: Default::default(),
        reuse: Default::default(),
        max_age_seconds: None,
        coverage_proof: None,
    }
}
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(5)
}
fn retained() -> (tempfile::TempDir, NativeProviderConfig) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("observation.json");
    let bytes = serde_json::to_vec(&envelope("dev")).unwrap();
    std::fs::write(&path, &bytes).unwrap();
    let config = configured(NativeTransport::RetainedArtifact {
        path,
        sha256: format!("{:x}", Sha256::digest(&bytes)),
    });
    (directory, config)
}
async fn retained_preserves_age_hash_and_exact_numbers() {
    let (_directory, config) = retained();
    let executor = NativeSources::new(vec![config.clone()], BTreeMap::new()).unwrap();
    let step = step(&config);
    let request = executor.bind(&step, step.arguments.clone()).await.unwrap();
    let observation = executor.execute(&request, deadline()).await.unwrap();
    assert_eq!(observation.observed_at, "2026-10-24T12:00:00Z");
    assert_eq!(observation.value["n"].to_string(), "12345678901234567890");
    assert_eq!(observation.coverage, CoverageStatus::CompleteEnumeration);
    assert_eq!(observation.metrics.transport_requests, Some(0));
    if let NativeTransport::RetainedArtifact { path, .. } = &config.transport {
        std::fs::write(path, b"{}").unwrap();
    }
    assert!(matches!(
        executor
            .execute(&request, deadline())
            .await
            .unwrap_err()
            .kind,
        SourceErrorKind::SchemaDrift
    ));
}
async fn input_identity_and_contract_drift_are_rejected() {
    let (_directory, mut config) = retained();
    let executor = NativeSources::new(vec![config.clone()], BTreeMap::new()).unwrap();
    let mut planned = step(&config);
    assert!(matches!(
        executor
            .bind(&planned, json!({"cluster":"prod"}))
            .await
            .unwrap_err()
            .kind,
        SourceErrorKind::Configuration
    ));
    assert!(executor.bind(&planned, json!({"cluster":3})).await.is_err());
    planned.source.output_schema_sha256 = "9".repeat(64);
    assert!(matches!(
        executor
            .bind(&planned, planned.arguments.clone())
            .await
            .unwrap_err()
            .kind,
        SourceErrorKind::SchemaDrift
    ));
    config
        .capability
        .semantics
        .as_mut()
        .unwrap()
        .scope_bindings
        .insert("resource_uid".into(), "/resource_uid".into());
    pin_config(&mut config);
    let executor = NativeSources::new(vec![config.clone()], BTreeMap::new()).unwrap();
    let planned = step(&config);
    let request = executor
        .bind(
            &planned,
            json!({"cluster":"dev","resource_uid":"discovered-uid"}),
        )
        .await
        .unwrap();
    assert_eq!(request.source.scope["resource_uid"], "discovered-uid");
    executor.execute(&request, deadline()).await.unwrap();
    let request = executor
        .bind(
            &planned,
            json!({"cluster":"dev","resource_uid":"other-uid"}),
        )
        .await
        .unwrap();
    assert!(executor.execute(&request, deadline()).await.is_err());
}
fn command(mode: &str, eal: bool) -> NativeProviderConfig {
    let executable = std::env::current_exe().unwrap();
    configured(NativeTransport::Command {
        program: executable.clone(),
        supervisor: executable,
        arguments: vec!["native-fixture".into(), mode.into()],
        cwd: std::env::temp_dir(),
        inherit_env: vec![],
        protocol: if eal {
            CommandProtocol::EalAcquisition {
                environment: "dev".into(),
                tool: "verified_generated_tool".into(),
                tool_version: "1".into(),
                context: json!({"scope":"development"}),
            }
        } else {
            CommandProtocol::RawJson
        },
    })
}
async fn command_protocol_and_failures_are_supervised() {
    for (mode, eal, success) in [
        ("raw", false, true),
        ("eal", true, true),
        ("bad_correspondence", true, false),
        ("wrong_identity", false, false),
        ("fail", false, false),
        ("oversize", false, false),
    ] {
        let config = command(mode, eal);
        let planned = step(&config);
        let executor = NativeSources::new(vec![config], BTreeMap::new()).unwrap();
        let request = executor
            .bind(&planned, planned.arguments.clone())
            .await
            .unwrap();
        assert_eq!(
            executor.execute(&request, deadline()).await.is_ok(),
            success,
            "{mode}"
        );
    }
    let mut config = command("credential", false);
    config.credential_env = vec!["NATIVE_FIXTURE_TOKEN".into()];
    if let NativeTransport::Command { inherit_env, .. } = &mut config.transport {
        inherit_env.push("NATIVE_FIXTURE_TOKEN".into());
    }
    pin_config(&mut config);
    let planned = step(&config);
    let executor = NativeSources::new(
        vec![config],
        BTreeMap::from([("NATIVE_FIXTURE_TOKEN".into(), "fixture-secret-value".into())]),
    )
    .unwrap();
    let request = executor
        .bind(&planned, planned.arguments.clone())
        .await
        .unwrap();
    assert!(executor.execute(&request, deadline()).await.is_err());
}
async fn http_authority_is_fixed_and_no_redirect_is_followed() {
    use axum::{Json, Router, response::Redirect, routing::get};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = Router::new()
        .route("/measure", get(|| async { Json(envelope("dev")) }))
        .route(
            "/redirect",
            get(|| async { Redirect::temporary("/measure") }),
        );
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    for (path, success) in [("measure", true), ("redirect", false)] {
        let config = configured(NativeTransport::HttpGet {
            url: format!("http://{address}/{path}"),
            query_bindings: BTreeMap::from([("cluster".into(), "/cluster".into())]),
            token_env: None,
        });
        let planned = step(&config);
        let executor = NativeSources::new(vec![config], BTreeMap::new()).unwrap();
        let request = executor
            .bind(&planned, planned.arguments.clone())
            .await
            .unwrap();
        assert_eq!(
            executor.execute(&request, deadline()).await.is_ok(),
            success
        );
    }
    server.abort();
}
async fn dispatcher_admits_native_catalogues_and_executes() {
    let (_directory, config) = retained();
    let planned = step(&config);
    let config = EvidenceConfig {
        schema: eal_evidence::config::CONFIG_SCHEMA.into(),
        connections: vec![],
        native_providers: vec![config],
        local_catalogues: vec![],
        approved_catalogues: vec![],
        discovery: Default::default(),
    };
    let dispatcher = EvidenceSources::from_config(config, BTreeMap::new()).unwrap();
    let report = dispatcher.discover().unwrap();
    assert_eq!(report.admitted_capabilities, ["native_measurement"]);
    assert!(report.gaps.is_empty());
    let request = dispatcher
        .bind(&planned, planned.arguments.clone())
        .await
        .unwrap();
    dispatcher.execute(&request, deadline()).await.unwrap();
}
