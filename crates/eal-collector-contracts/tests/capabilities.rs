//! Independent evidence-contract cases exercise adequacy before cost or transport.
use eal_collector_contracts::capability::*;
use serde_json::json;
use std::collections::BTreeMap;

fn requirement() -> SupportContract {
    SupportContract {
        schema: SUPPORT_CONTRACT_SCHEMA.into(),
        support_type: SupportType::Observation,
        scope: SubjectScope {
            kind: "service_instance".into(),
            subject: "Release service instances in the development cluster".into(),
            selectors: BTreeMap::from([("cluster".into(), json!("dev"))]),
            identity_dimensions: vec!["cluster".into(), "resource_uid".into()],
        },
        acceptable_methods: vec!["instrument.latency/1".into()],
        quantities: vec![QuantityRequirement {
            name: "latency_p99".into(),
            unit: "ms".into(),
        }],
        logical_strength: LogicalStrength::ExistentialWitness,
        population: None,
        time: TimeRequirement {
            historical_interval: None,
            max_age_seconds: Some(300),
            max_uncertainty_seconds: Some(2),
            require_original_observation_time: true,
        },
        provenance: ProvenanceRequirement {
            required_authorities: vec!["instrument-run".into()],
            transformation_history: true,
        },
        expected_output_schema_sha256: None,
    }
}

fn capability(id: &str) -> CapabilityDescriptor {
    let input_schema = json!({"type":"object","properties":{"cluster":{"type":"string"}},"required":["cluster"],"additionalProperties":false});
    let output_schema = json!({"type":"object","properties":{"latency_p99":{"type":"number"},"complete":{"type":"boolean"}},"required":["latency_p99","complete"],"additionalProperties":false});
    CapabilityDescriptor {
        id: id.into(),
        version: "1".into(),
        description: "Read a qualified latency instrument".into(),
        provider: ProviderIdentity {
            id: "instrument".into(),
            version: "1".into(),
            transport: TransportKind::Command,
            connection: None,
            tool: None,
            authentication_scope: "development-reader".into(),
            configuration_sha256: "1".repeat(64),
        },
        input_schema_sha256: schema_identity(&input_schema).unwrap(),
        input_schema,
        output_schema_sha256: schema_identity(&output_schema).unwrap(),
        output_schema,
        semantics: Some(MeasurementSemantics {
            qualification_reference: "instrument-protocol/1".into(),
            support_types: vec![SupportType::Observation],
            subject_kinds: vec!["service_instance".into()],
            methods: vec!["instrument.latency/1".into()],
            identity_dimensions: vec!["cluster".into(), "resource_uid".into()],
            scope_constraints: BTreeMap::new(),
            scope_bindings: BTreeMap::from([("cluster".into(), "/cluster".into())]),
            quantities: vec![MeasurementQuantity {
                name: "latency_p99".into(),
                unit: "ms".into(),
                output_pointer: "/latency_p99".into(),
            }],
            coverage: vec![CoverageGuarantee {
                strength: LogicalStrength::ExistentialWitness,
                population: None,
                completeness_pointer: None,
                sample_size_pointer: None,
            }],
            time: TimeSemantics {
                historical_intervals: true,
                historical_input: None,
                original_observation_time: true,
                uncertainty_seconds: Some(1),
            },
            provenance: ProvenanceSemantics {
                authorities: vec!["instrument-run".into()],
                transformation_history: true,
            },
            limitations: vec!["A witness does not establish absence across the population".into()],
        }),
        configuration_requirements: vec![],
        limits: CapabilityLimits {
            timeout_seconds: 30,
            output_bytes: 1_048_576,
            concurrency: 2,
            requests_per_minute: Some(60),
        },
        costs: CapabilityCosts::unknown("not measured"),
    }
}

#[test]
fn declared_interface_and_description_do_not_qualify_measurement_semantics() {
    let mut candidate = capability("search");
    candidate.description = "Proves every claim with complete authoritative results".into();
    candidate.semantics = None;
    candidate.validate().unwrap();
    let result = compatible(&requirement(), &candidate);
    assert!(!result.adequate);
    assert_eq!(
        result.issues,
        [CompatibilityIssue::MissingQualifiedSemantics]
    );
}

#[test]
fn actual_arguments_determine_scope_before_execution_or_reuse() {
    let candidate = capability("instrument");
    let requested = BTreeMap::from([("cluster".into(), json!("dev"))]);
    validate_scope_input(&candidate, &requested, &json!({"cluster":"dev"})).unwrap();
    assert!(validate_scope_input(&candidate, &requested, &json!({"cluster":"prod"})).is_err());
    assert!(scope_from_input(&candidate, &json!({})).is_err());
    assert!(scope_from_input(&candidate, &json!({"cluster":""})).is_err());
    assert!(scope_from_input(&candidate, &json!({"cluster":null})).is_err());
    let mut fixed = candidate.clone();
    fixed
        .semantics
        .as_mut()
        .unwrap()
        .scope_constraints
        .insert("cluster".into(), json!("dev"));
    assert!(scope_from_input(&fixed, &json!({"cluster":"prod"})).is_err());
    assert!(validate_scope_input(&candidate, &BTreeMap::new(), &json!({"cluster":"dev"})).is_err());
}

#[test]
fn missing_scope_bindings_are_qualification_gaps_and_invalid_pointers_are_rejected() {
    let mut candidate = capability("instrument");
    candidate.semantics.as_mut().unwrap().scope_bindings.clear();
    let result = compatible(&requirement(), &candidate);
    assert!(!result.adequate);
    assert!(result.issues.iter().any(|issue| matches!(issue, CompatibilityIssue::ScopeBindingUnavailable { dimension } if dimension == "cluster")));
    candidate
        .semantics
        .as_mut()
        .unwrap()
        .scope_constraints
        .insert("cluster".into(), json!("dev"));
    assert!(compatible(&requirement(), &candidate).adequate);
    candidate
        .semantics
        .as_mut()
        .unwrap()
        .scope_bindings
        .insert("cluster".into(), "/bad~9pointer".into());
    assert!(candidate.validate().is_err());
}

#[test]
fn measurement_identity_tracks_qualification_and_provider_not_estimate_or_description_edits() {
    let mut candidate = capability("instrument");
    let original = candidate.measurement_identity().unwrap();
    candidate.costs.execution_requests = CostEstimate::declared(5, "updated measurement");
    candidate.description = "Updated operator help".into();
    candidate.limits.concurrency = 3;
    assert_eq!(original, candidate.measurement_identity().unwrap());
    candidate.semantics.as_mut().unwrap().methods = vec!["instrument.latency/2".into()];
    assert_ne!(original, candidate.measurement_identity().unwrap());
    candidate.semantics.as_mut().unwrap().methods = vec!["instrument.latency/1".into()];
    candidate.provider.authentication_scope = "another-reader".into();
    assert_ne!(original, candidate.measurement_identity().unwrap());
    candidate.semantics = None;
    assert!(candidate.measurement_identity().is_err());
}

#[test]
fn equal_numeric_outputs_cannot_substitute_a_different_subject_or_measurement_protocol() {
    let mut required = requirement();
    required.support_type = SupportType::Experiment;
    required.acceptable_methods = vec!["benchmark.controlled-load/1".into()];
    let mut candidate = capability("passive_timing");
    let semantics = candidate.semantics.as_mut().unwrap();
    semantics.support_types = vec![SupportType::Experiment];
    semantics.methods = vec!["measurement.passive-timing/1".into()];
    semantics.subject_kinds = vec!["disk_device".into()];
    let compared = compatible(&required, &candidate);
    assert!(!compared.adequate);
    assert!(
        compared
            .issues
            .iter()
            .any(|issue| matches!(issue, CompatibilityIssue::SubjectKindMismatch { .. }))
    );
    assert!(
        compared
            .issues
            .iter()
            .any(|issue| matches!(issue, CompatibilityIssue::MeasurementMethodMismatch { .. }))
    );
    candidate.semantics.as_mut().unwrap().subject_kinds = vec!["service_instance".into()];
    candidate.semantics.as_mut().unwrap().methods = vec!["benchmark.controlled-load/1".into()];
    assert!(compatible(&required, &candidate).adequate);
    required.acceptable_methods = vec!["arbitrary descriptive prose".into()];
    assert!(required.validate().is_err());
}

#[test]
fn a_bounded_witness_search_cannot_supply_population_absence() {
    let mut required = requirement();
    required.logical_strength = LogicalStrength::CompleteEnumeration;
    required.population = Some("all-service-instances".into());
    let mut search = capability("cheap_search");
    search.description = "Complete inventory of all service instances".into();
    search.costs.execution_requests = CostEstimate::declared(0, "cached search");
    let mut inventory = capability("inventory");
    inventory.semantics.as_mut().unwrap().coverage = vec![CoverageGuarantee {
        strength: LogicalStrength::CompleteEnumeration,
        population: Some("all-service-instances".into()),
        completeness_pointer: Some("/complete".into()),
        sample_size_pointer: None,
    }];
    inventory.costs.execution_requests = CostEstimate::declared(100, "complete enumeration");
    let result = compatible(&required, &search);
    assert!(
        result
            .issues
            .iter()
            .any(|issue| matches!(issue, CompatibilityIssue::CoverageMismatch { .. }))
    );
    let ranked = rank_adequate(
        &required,
        &[search, inventory],
        &RankingPolicy {
            dimensions: vec![CostDimension::ExecutionRequests],
        },
    )
    .unwrap();
    assert_eq!(
        ranked
            .iter()
            .map(|candidate| candidate.capability_id.as_str())
            .collect::<Vec<_>>(),
        ["inventory"]
    );
}

#[test]
fn complete_enumeration_requires_population_and_a_measured_completeness_field() {
    let mut candidate = capability("inventory");
    let coverage = &mut candidate.semantics.as_mut().unwrap().coverage[0];
    coverage.strength = LogicalStrength::CompleteEnumeration;
    assert!(candidate.validate().is_err());
    candidate.semantics.as_mut().unwrap().coverage[0].population = Some("instances".into());
    assert!(candidate.validate().is_err());
    candidate.semantics.as_mut().unwrap().coverage[0].completeness_pointer =
        Some("/complete".into());
    candidate.validate().unwrap();
    assert!(compatible(&requirement(), &candidate).adequate);
    let mut required = requirement();
    required.population = Some("another-population".into());
    assert!(!compatible(&required, &candidate).adequate);
}

#[test]
fn statistical_guarantees_require_a_qualified_measured_sample_size_pointer() {
    let mut candidate = capability("sample");
    let guarantee = &mut candidate.semantics.as_mut().unwrap().coverage[0];
    guarantee.strength = LogicalStrength::StatisticalEstimate;
    assert!(candidate.validate().is_err());
    candidate.semantics.as_mut().unwrap().coverage[0].sample_size_pointer =
        Some("not-a-pointer".into());
    assert!(candidate.validate().is_err());
    candidate.semantics.as_mut().unwrap().coverage[0].sample_size_pointer =
        Some("/sample_size".into());
    candidate.validate().unwrap();
    let mut required = requirement();
    required.logical_strength = LogicalStrength::StatisticalEstimate;
    assert!(compatible(&required, &candidate).adequate);
}

#[test]
fn units_scope_identity_time_authority_and_transformation_are_hard_checks() {
    let mut candidate = capability("instrument");
    let semantics = candidate.semantics.as_mut().unwrap();
    semantics.quantities[0].unit = "s".into();
    semantics.identity_dimensions.pop();
    semantics
        .scope_constraints
        .insert("cluster".into(), json!("prod"));
    semantics.time.original_observation_time = false;
    semantics.time.uncertainty_seconds = Some(9);
    semantics.provenance.authorities.clear();
    semantics.provenance.transformation_history = false;
    let result = compatible(&requirement(), &candidate);
    assert!(!result.adequate);
    assert!(result.issues.iter().any(|issue|matches!(issue,CompatibilityIssue::UnitMismatch{required,offered,..} if required=="ms" && offered=="s")));
    assert!(
        result
            .issues
            .iter()
            .any(|issue| matches!(issue, CompatibilityIssue::ScopeMismatch { .. }))
    );
    assert!(
        result
            .issues
            .iter()
            .any(|issue| matches!(issue, CompatibilityIssue::MissingIdentityDimension { .. }))
    );
    assert!(
        result
            .issues
            .contains(&CompatibilityIssue::OriginalObservationTimeUnavailable)
    );
    assert!(result.issues.iter().any(|issue| matches!(
        issue,
        CompatibilityIssue::TemporalUncertaintyExceeded { .. }
    )));
    assert!(
        result
            .issues
            .iter()
            .any(|issue| matches!(issue, CompatibilityIssue::MissingSourceAuthority { .. }))
    );
    assert!(
        result
            .issues
            .contains(&CompatibilityIssue::TransformationHistoryUnavailable)
    );
}

#[test]
fn unknown_scope_and_uncertainty_retain_their_own_gap_classes() {
    let mut candidate = capability("scoped");
    candidate
        .semantics
        .as_mut()
        .unwrap()
        .scope_constraints
        .insert("region".into(), json!("west"));
    candidate
        .semantics
        .as_mut()
        .unwrap()
        .time
        .uncertainty_seconds = None;
    let result = compatible(&requirement(), &candidate);
    assert!(
        result
            .issues
            .iter()
            .any(CompatibilityIssue::is_configuration_gap)
    );
    assert!(
        result
            .issues
            .iter()
            .any(CompatibilityIssue::requires_qualification)
    );
    assert!(!result.adequate);
}

#[test]
fn historical_sources_and_schema_versions_cannot_be_silently_substituted() {
    let mut required = requirement();
    required.time.historical_interval = Some(HistoricalInterval {
        start_unix: 1000,
        end_unix: 2000,
    });
    let mut candidate = capability("live_only");
    let mut early = required.clone();
    early.time.historical_interval = Some(HistoricalInterval {
        start_unix: -631_152_000,
        end_unix: -315_619_200,
    });
    early.validate().unwrap();
    candidate
        .semantics
        .as_mut()
        .unwrap()
        .time
        .historical_intervals = false;
    assert!(
        compatible(&required, &candidate)
            .issues
            .contains(&CompatibilityIssue::HistoricalIntervalUnsupported)
    );
    candidate = capability("drifted");
    required.expected_output_schema_sha256 = Some(candidate.output_schema_sha256.clone());
    candidate.output_schema["properties"]["latency_p99"]["type"] = json!("string");
    assert!(candidate.validate().is_err());
    candidate.output_schema_sha256 = schema_identity(&candidate.output_schema).unwrap();
    assert!(
        compatible(&required, &candidate)
            .issues
            .contains(&CompatibilityIssue::OutputSchemaMismatch)
    );
}

#[test]
fn historical_query_arguments_are_pinned_without_rounding_or_timezone_confusion() {
    let mut candidate = capability("history");
    candidate.semantics.as_mut().unwrap().time.historical_input = Some(HistoricalInputBinding {
        start_pointer: "/start".into(),
        end_pointer: "/end".into(),
        encoding: HistoricalTimeEncoding::UnixSeconds,
    });
    validate_historical_input(
        &candidate,
        &json!({"start":-1,"end":1}),
        "1969-12-31T23:59:59Z",
        "1970-01-01T00:00:01Z",
    )
    .unwrap();
    assert!(
        validate_historical_input(
            &candidate,
            &json!({"start":0,"end":1}),
            "1969-12-31T23:59:59Z",
            "1970-01-01T00:00:01Z"
        )
        .is_err()
    );
    candidate
        .semantics
        .as_mut()
        .unwrap()
        .time
        .historical_input
        .as_mut()
        .unwrap()
        .encoding = HistoricalTimeEncoding::UnixSecondsString;
    validate_historical_input(
        &candidate,
        &json!({"start":"-1","end":"1"}),
        "1969-12-31T23:59:59Z",
        "1970-01-01T00:00:01Z",
    )
    .unwrap();
    for start in [
        json!(-1),
        json!("-1.0"),
        json!("-1e0"),
        json!("+0"),
        json!("9223372036854775808"),
    ] {
        assert!(
            validate_historical_input(
                &candidate,
                &json!({"start":start,"end":"1"}),
                "1969-12-31T23:59:59Z",
                "1970-01-01T00:00:01Z"
            )
            .is_err()
        );
    }
    assert!(
        validate_historical_input(
            &candidate,
            &json!({"start":0,"end":1}),
            "1970-01-01T00:00:00.5Z",
            "1970-01-01T00:00:01Z"
        )
        .is_err()
    );
    candidate
        .semantics
        .as_mut()
        .unwrap()
        .time
        .historical_input
        .as_mut()
        .unwrap()
        .encoding = HistoricalTimeEncoding::Rfc3339;
    validate_historical_input(
        &candidate,
        &json!({"start":"1970-01-01T01:00:00+01:00","end":"1970-01-01T00:00:01Z"}),
        "1970-01-01T00:00:00Z",
        "1970-01-01T00:00:01Z",
    )
    .unwrap();
    candidate
        .semantics
        .as_mut()
        .unwrap()
        .time
        .historical_input
        .as_mut()
        .unwrap()
        .end_pointer = "/start".into();
    assert!(candidate.validate().is_err());
    candidate.semantics.as_mut().unwrap().time.historical_input = None;
    assert!(
        validate_historical_input(
            &candidate,
            &json!({}),
            "1970-01-01T00:00:00Z",
            "1970-01-01T00:00:01Z"
        )
        .is_err()
    );
    let mut required = requirement();
    required.time.historical_interval = Some(HistoricalInterval {
        start_unix: 0,
        end_unix: 1,
    });
    assert!(
        compatible(&required, &candidate)
            .issues
            .contains(&CompatibilityIssue::HistoricalInputBindingUnavailable)
    );
}

#[test]
fn ranking_has_explicit_cost_precedence_unknowns_and_stable_ties() {
    let required = requirement();
    let mut costly = capability("a_costly");
    costly.costs.execution_requests = CostEstimate::declared(9, "measured request count");
    let mut cheap = capability("z_cheap");
    cheap.costs.execution_requests = CostEstimate::declared(1, "measured request count");
    cheap.provider.transport = TransportKind::McpHttp;
    cheap.provider.connection = Some("server".into());
    cheap.provider.tool = Some("read".into());
    let unknown = capability("unknown");
    let policy = RankingPolicy {
        dimensions: vec![CostDimension::ExecutionRequests],
    };
    let ranked = rank_adequate(
        &required,
        &[unknown.clone(), costly, cheap.clone()],
        &policy,
    )
    .unwrap();
    assert_eq!(
        ranked
            .iter()
            .map(|candidate| candidate.capability_id.as_str())
            .collect::<Vec<_>>(),
        ["z_cheap", "a_costly", "unknown"]
    );
    assert_eq!(
        ranked[2].unknown_dimensions,
        [CostDimension::ExecutionRequests]
    );
    let mut tie = cheap.clone();
    tie.id = "a_tie".into();
    tie.provider.transport = TransportKind::Command;
    tie.provider.connection = None;
    tie.provider.tool = None;
    let ranked = rank_adequate(&required, &[cheap, tie], &policy).unwrap();
    assert_eq!(ranked[0].capability_id, "a_tie");
    assert!(rank_adequate(&required, &[unknown.clone(), unknown], &policy).is_err());
    assert!(RankingPolicy { dimensions: vec![] }.validate().is_err());
}

#[test]
fn unknown_costs_cannot_be_encoded_as_zero_or_an_unlabelled_estimate() {
    let mut candidate = capability("bad_cost");
    candidate.costs.execution_requests.value = Some(0);
    assert!(candidate.validate().is_err());
    candidate.costs.execution_requests.basis = EstimateBasis::Estimated;
    candidate.validate().unwrap();
    candidate.costs.execution_requests.reference.clear();
    assert!(candidate.validate().is_err());
}

#[test]
fn exact_identity_is_order_independent_and_distinguishes_large_integers_and_decimals() {
    let left: serde_json::Value = serde_json::from_str(
        r#"{"scope":{"id":90071992547409931234567890,"time":1.234567890123456789},"name":"run"}"#,
    )
    .unwrap();
    let right: serde_json::Value = serde_json::from_str(
        r#"{"name":"run","scope":{"time":1.234567890123456789,"id":90071992547409931234567890}}"#,
    )
    .unwrap();
    assert_eq!(
        canonical_identity(&left).unwrap(),
        canonical_identity(&right).unwrap()
    );
    let changed: serde_json::Value = serde_json::from_str(
        r#"{"scope":{"id":90071992547409931234567891,"time":1.234567890123456788},"name":"run"}"#,
    )
    .unwrap();
    assert_ne!(
        canonical_identity(&left).unwrap(),
        canonical_identity(&changed).unwrap()
    );
    assert!(schema_identity(&json!({"$ref":"https://outside.invalid/schema"})).is_err());
}

#[test]
fn typed_capability_and_support_wire_roundtrips_preserve_opaque_private_number_keys() {
    let opaque =
        json!({"outer":[{"$serde_json::private::Number":"123456789012345678901234567890"}]});
    let mut candidate = capability("literal_payload");
    candidate.input_schema["default"] = opaque.clone();
    candidate.input_schema_sha256 = schema_identity(&candidate.input_schema).unwrap();
    candidate.output_schema["default"] = opaque.clone();
    candidate.output_schema_sha256 = schema_identity(&candidate.output_schema).unwrap();
    candidate
        .semantics
        .as_mut()
        .unwrap()
        .scope_constraints
        .insert("literal".into(), opaque.clone());
    let encoded = serde_json::to_string(&candidate).unwrap();
    let decoded: CapabilityDescriptor = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded.input_schema["default"], opaque);
    assert_eq!(decoded.output_schema["default"], opaque);
    assert_eq!(
        decoded.semantics.as_ref().unwrap().scope_constraints["literal"],
        opaque
    );
    assert_eq!(
        candidate.measurement_identity().unwrap(),
        decoded.measurement_identity().unwrap()
    );
    let decoded: CapabilityDescriptor =
        serde_json::from_value(serde_json::to_value(&candidate).unwrap()).unwrap();
    assert_eq!(decoded.input_schema["default"], opaque);
    let mut required = requirement();
    required
        .scope
        .selectors
        .insert("literal".into(), opaque.clone());
    let decoded: SupportContract =
        serde_json::from_str(&serde_json::to_string(&required).unwrap()).unwrap();
    assert_eq!(decoded.scope.selectors["literal"], opaque);
}

#[test]
fn catalogue_versions_duplicate_ids_and_nonliteral_pointers_are_rejected() {
    let mut catalogue = CapabilityCatalogue {
        schema: CAPABILITY_CATALOGUE_SCHEMA.into(),
        capabilities: vec![capability("read")],
    };
    catalogue.validate().unwrap();
    catalogue.capabilities.push(capability("read"));
    assert!(catalogue.validate().is_err());
    catalogue.capabilities.pop();
    catalogue.schema = "eal.capability-catalogue/2".into();
    assert!(catalogue.validate().is_err());
    let mut candidate = capability("bad_pointer");
    candidate.semantics.as_mut().unwrap().quantities[0].output_pointer = "/name~2escape".into();
    assert!(candidate.validate().is_err());
    candidate = capability("mcp");
    candidate.provider.transport = TransportKind::McpStdio;
    assert!(candidate.validate().is_err());
}
