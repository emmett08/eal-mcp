//! A measured SDK comparison of the same requirement evaluations with/without sharing.
use super::*;
use eal_evidence::{
    execution::{EvaluationStatus, EvidenceExecutor, ExecutionLimits},
    plan::*,
    storage::SourceStore,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn graph_vs_flat_retains_observed_request_byte_and_elapsed_measurements() {
    let fixture = Fixture::start().await;
    fixture.server.result.lock().unwrap()["measurement"]["payload"] = json!({"latency":55});
    let thresholds = [20_u64, 60, 80];
    let flat = fixture.bridge();
    let before_calls = fixture.server.calls.load(Ordering::SeqCst);
    let flat_started = Instant::now();
    let mut flat_requests = 0;
    let mut flat_request_bytes = 0;
    let mut flat_response_bytes = 0;
    let mut flat_results = Vec::new();
    for threshold in thresholds {
        let observed = flat
            .call("measurement", args(), scope(), deadline())
            .unwrap();
        flat_requests += observed.metrics.requests;
        flat_request_bytes += observed.metrics.request_bytes;
        flat_response_bytes += observed.metrics.response_bytes;
        flat_results.push(observed.value["payload"]["latency"].as_u64().unwrap() <= threshold);
    }
    let flat_elapsed = flat_started.elapsed().as_micros();
    let flat_calls = fixture.server.calls.load(Ordering::SeqCst) - before_calls;
    let config = fixture.config();
    let descriptor = &config.local_catalogues[0].capabilities[0];
    let source = SourceIdentity {
        provider_id: descriptor.provider.id.clone(),
        capability_id: descriptor.id.clone(),
        capability_version: descriptor.version.clone(),
        input_schema_sha256: descriptor.input_schema_sha256.clone(),
        output_schema_sha256: descriptor.output_schema_sha256.clone(),
        qualified_contract_sha256: descriptor.measurement_identity().unwrap(),
        configuration_sha256: "0".repeat(64),
        authentication_scope_sha256: "0".repeat(64),
        scope: scope(),
        historical_interval: None,
    };
    let plan = EvidenceExecutionPlan {
        schema: PLAN_SCHEMA.into(),
        acquisitions: vec![AcquisitionStep {
            id: "shared-latency".into(),
            source,
            arguments: args(),
            bindings: vec![],
            for_each: None,
            availability: SourceAvailability::Configured,
            reuse: ReusePolicy::FreshOrAcquire,
            max_age_seconds: None,
        }],
        derivations: vec![],
        evaluations: thresholds
            .iter()
            .enumerate()
            .map(|(index, threshold)| RequirementEvaluation {
                id: format!("evaluate-{index}"),
                requirement_id: format!("latency-{index}"),
                inputs: vec![OutputReference {
                    step_id: "shared-latency".into(),
                    pointer: "/payload/latency".into(),
                }],
                criterion: EvaluationCriterion::Numeric {
                    comparison: NumericComparison::AtMost,
                    threshold: json!(threshold),
                    value_unit: "ms".into(),
                    threshold_unit: "ms".into(),
                },
                max_age_seconds: Some(60),
                required_coverage: RequiredCoverage::Witness,
                applicability: vec![],
                challenges: vec![],
            })
            .collect(),
        scheduling: SchedulingPolicy::default(),
    };
    let graph_bridge = Arc::new(EvidenceBridge::new(config, BTreeMap::new()).unwrap());
    let temp = tempfile::tempdir().unwrap();
    let store = SourceStore::open(&temp.path().join("sources"), 10, 1_048_576).unwrap();
    let executor =
        EvidenceExecutor::new(graph_bridge.clone(), store, ExecutionLimits::default()).unwrap();
    let before_graph = fixture.server.calls.load(Ordering::SeqCst);
    let graph_started = Instant::now();
    let graph = executor
        .execute(&plan, Some("2026-10-24T12:00:10Z"))
        .await
        .unwrap();
    let graph_elapsed = graph_started.elapsed().as_micros();
    let graph_calls = fixture.server.calls.load(Ordering::SeqCst) - before_graph;
    let graph_results = graph
        .evaluations
        .iter()
        .map(|result| result.status == EvaluationStatus::Satisfied)
        .collect::<Vec<_>>();
    assert_eq!(flat_results, graph_results);
    assert_eq!(flat_calls, 3);
    assert_eq!(graph_calls, 1);
    assert_eq!(graph.metrics.source_requests, 1);
    assert!(graph.metrics.reported_provider_requests < flat_requests);
    assert!(
        graph
            .evaluations
            .windows(2)
            .all(|pair| pair[0].source_ids == pair[1].source_ids)
    );
    let measurement = json!({"schema":"eal.source-sharing-measurement/1","fixture":"loopback HTTP MCP SDK server; three thresholds over the same observed latency","requirements":thresholds.len(),"flat":{"source_calls":flat_calls,"reported_provider_requests":flat_requests,"logical_request_bytes":flat_request_bytes,"logical_response_bytes":flat_response_bytes,"elapsed_us":flat_elapsed,"generated_source_count":0},"graph":{"source_calls":graph_calls,"reported_provider_requests":graph.metrics.reported_provider_requests,"logical_request_bytes":graph.metrics.reported_request_bytes,"logical_response_bytes":graph.metrics.reported_response_bytes,"elapsed_us":graph_elapsed,"generated_source_count":graph.metrics.generated_code_count},"results":graph_results,"metric_scope":"source tool calls and reported logical MCP JSON payloads including initialization and schema admission; excludes HTTP/TLS framing and unreported list-request bytes; elapsed is one loopback test run, not a general speed claim","live_private_evidence_verified":false});
    println!("EAL_SOURCE_SHARING_MEASUREMENT {measurement}");
    drop(executor);
    drop(graph_bridge);
    drop(flat);
    fixture.close().await;
}
