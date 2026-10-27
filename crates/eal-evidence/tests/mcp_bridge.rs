//! Real SDK acquisition, adversarial interface changes, and normalisation boundaries.
use eal_collector_contracts::{AcquisitionRequest, capability::*};
use eal_evidence::*;
use rmcp::{ServerHandler, model::*};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;
fn input_schema() -> Value {
    json!({"type":"object","properties":{"cluster":{"type":"string"},"payload":{}},"required":["cluster","payload"],"additionalProperties":false})
}
fn output_schema() -> Value {
    json!({"type":"object","properties":{"cluster":{"type":"string"},"present":{"type":"boolean"},"complete":{"type":"boolean"},"payload":{}},"required":["cluster","present","complete","payload"],"additionalProperties":false})
}
#[derive(Clone)]
struct Server {
    calls: Arc<AtomicUsize>,
    delay_ms: Arc<AtomicUsize>,
    active: Arc<AtomicUsize>,
    peak: Arc<AtomicUsize>,
    lists: Arc<AtomicUsize>,
    initializations: Arc<AtomicUsize>,
    list_error: Arc<AtomicBool>,
    drift: Arc<AtomicBool>,
    result: Arc<Mutex<Value>>,
    tool_error: Arc<AtomicBool>,
    text: Arc<Mutex<Option<String>>>,
}
impl ServerHandler for Server {
    async fn initialize(
        &self,
        request: InitializeRequestParams,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<InitializeResult, rmcp::ErrorData> {
        self.initializations.fetch_add(1, Ordering::SeqCst);
        context.peer.set_peer_info(request.clone());
        self.negotiate_initialize(&request)
    }
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("measurement-server", "1"))
    }
    async fn list_tools(
        &self,
        _: Option<PaginatedRequestParams>,
        _: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<ListToolsResult, rmcp::ErrorData> {
        self.lists.fetch_add(1, Ordering::SeqCst);
        if self.list_error.load(Ordering::SeqCst) {
            return Err(rmcp::ErrorData::internal_error(
                "independent discovery failure",
                None,
            ));
        }
        let schema = if self.drift.load(Ordering::SeqCst) {
            json!({"type":"object","properties":{"different":{"type":"string"}},"required":["different"]})
        } else {
            input_schema()
        };
        Ok(ListToolsResult::with_all_items(vec![Tool::new(
            "read_measurement",
            "Descriptive prose is not qualification",
            Arc::new(schema.as_object().unwrap().clone()),
        )]))
    }
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<CallToolResponse, rmcp::ErrorData> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.peak.fetch_max(active, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(
            self.delay_ms.load(Ordering::SeqCst) as u64
        ))
        .await;
        self.active.fetch_sub(1, Ordering::SeqCst);
        assert_eq!(request.name, "read_measurement");
        let mut result = if let Some(text) = self.text.lock().unwrap().as_ref() {
            CallToolResult::success(vec![ContentBlock::text(text.clone())])
        } else {
            CallToolResult::structured(self.result.lock().unwrap().clone())
        };
        result.is_error = Some(self.tool_error.load(Ordering::SeqCst));
        Ok(result.into())
    }
}
struct Fixture {
    server: Server,
    url: String,
    stop: CancellationToken,
    task: tokio::task::JoinHandle<()>,
}
impl Fixture {
    async fn start() -> Self {
        let server = Server {
            calls: Arc::new(AtomicUsize::new(0)),
            delay_ms: Arc::new(AtomicUsize::new(0)),
            active: Arc::new(AtomicUsize::new(0)),
            peak: Arc::new(AtomicUsize::new(0)),
            lists: Arc::new(AtomicUsize::new(0)),
            initializations: Arc::new(AtomicUsize::new(0)),
            list_error: Arc::new(AtomicBool::new(false)),
            drift: Arc::new(AtomicBool::new(false)),
            result: Arc::new(Mutex::new(
                json!({"observed_at":"2026-10-24T12:00:00Z","measurement":{"cluster":"dev","present":false,"complete":false,"payload":{"literal":12345678901234567890u64}}}),
            )),
            tool_error: Arc::new(AtomicBool::new(false)),
            text: Arc::new(Mutex::new(None)),
        };
        let selected = server.clone();
        let stop = CancellationToken::new();
        let config = rmcp::transport::StreamableHttpServerConfig::default()
            .with_legacy_session_mode(false)
            .with_json_response(true)
            .disable_allowed_hosts()
            .disable_allowed_origins();
        let service=rmcp::transport::StreamableHttpService::new(move||Ok(selected.clone()),Arc::new(rmcp::transport::streamable_http_server::session::never::NeverSessionManager::default()),config);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/mcp", listener.local_addr().unwrap());
        let cancellation = stop.clone();
        let task = tokio::spawn(async move {
            axum::serve(listener, axum::Router::new().route_service("/mcp", service))
                .with_graceful_shutdown(cancellation.cancelled_owned())
                .await
                .unwrap();
        });
        Self {
            server,
            url,
            stop,
            task,
        }
    }
    fn config(&self) -> EvidenceConfig {
        let connection = ConnectionConfig {
            id: "fixture".into(),
            authentication_scope: "dev-reader".into(),
            transport: TransportConfig::Http {
                url: self.url.clone(),
                token_env: None,
            },
            tools: vec![ToolMapping {
                tool: "read_measurement".into(),
                raw_output_schema_sha256: None,
                response_format: ResponseFormat::Structured,
                value_pointer: "/measurement".into(),
                observation_time: ObservationTime::Pointer {
                    pointer: "/observed_at".into(),
                },
                scope_pointers: BTreeMap::from([("cluster".into(), "/cluster".into())]),
            }],
            timeout_seconds: 10,
            output_bytes: 65536,
        };
        let input_schema = input_schema();
        let output_schema = output_schema();
        let descriptor = CapabilityDescriptor {
            id: "measurement".into(),
            version: "1".into(),
            description: "Operator-qualified resource witness".into(),
            provider: ProviderIdentity {
                id: "fixture-provider".into(),
                version: "1".into(),
                transport: TransportKind::McpHttp,
                connection: Some("fixture".into()),
                tool: Some("read_measurement".into()),
                authentication_scope: "dev-reader".into(),
                configuration_sha256: connection.configuration_identity().unwrap(),
            },
            input_schema_sha256: schema_identity(&input_schema).unwrap(),
            input_schema,
            output_schema_sha256: schema_identity(&output_schema).unwrap(),
            output_schema,
            semantics: Some(MeasurementSemantics {
                qualification_reference: "protocol.resource-witness/1".into(),
                subject_kinds: vec!["service_instance".into()],
                methods: vec!["instrument.resource/1".into()],
                support_types: vec![SupportType::Observation],
                identity_dimensions: vec!["cluster".into()],
                scope_constraints: BTreeMap::new(),
                scope_bindings: BTreeMap::from([("cluster".into(), "/cluster".into())]),
                quantities: vec![],
                coverage: vec![
                    CoverageGuarantee {
                        strength: LogicalStrength::ExistentialWitness,
                        population: None,
                        population_quantity: None,
                        completeness_pointer: None,
                        sample_size_pointer: None,
                    },
                    CoverageGuarantee {
                        strength: LogicalStrength::CompleteEnumeration,
                        population: Some("instances".into()),
                        population_quantity: None,
                        completeness_pointer: Some("/complete".into()),
                        sample_size_pointer: None,
                    },
                ],
                time: TimeSemantics {
                    historical_intervals: false,
                    historical_input: None,
                    original_observation_time: true,
                    uncertainty_seconds: Some(1),
                },
                provenance: ProvenanceSemantics {
                    authorities: vec!["fixture-server".into()],
                    transformation_history: true,
                },
                limitations: vec![],
            }),
            configuration_requirements: vec![],
            limits: CapabilityLimits {
                timeout_seconds: 10,
                output_bytes: 65536,
                concurrency: 1,
                requests_per_minute: None,
            },
            costs: CapabilityCosts::unknown("unmeasured"),
        };
        EvidenceConfig {
            native_providers: vec![],
            schema: eal_evidence::config::CONFIG_SCHEMA.into(),
            connections: vec![connection],
            local_catalogues: vec![CapabilityCatalogue {
                schema: CAPABILITY_CATALOGUE_SCHEMA.into(),
                capabilities: vec![descriptor],
            }],
            approved_catalogues: vec![],
            discovery: DiscoveryLimits::default(),
        }
    }
    fn bridge(&self) -> EvidenceBridge {
        EvidenceBridge::new(self.config(), BTreeMap::new()).unwrap()
    }
    async fn close(self) {
        self.stop.cancel();
        tokio::time::timeout(Duration::from_secs(3), self.task)
            .await
            .unwrap()
            .unwrap();
    }
}
fn args() -> Value {
    json!({"cluster":"dev","payload":{"$serde_json::private::Number":"literal","n":12345678901234567890u64}})
}
fn scope() -> Value {
    json!({"cluster":"dev"})
}
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn existing_sdk_tools_produce_evidence_without_generated_adapters_preserving_age_request_and_negatives()
 {
    let fixture = Fixture::start().await;
    let bridge = fixture.bridge();
    let report = bridge.discover().unwrap();
    assert_eq!(report.admitted_capabilities, ["measurement"]);
    assert_eq!(report.metrics.requests, 2);
    assert_eq!(report.metrics.server_starts, 1);
    assert!(report.metrics.response_bytes > 0);
    let request = AcquisitionRequest {
        evidence_id: "E1".into(),
        environment: "development".into(),
        tool: "mcp_bridge".into(),
        tool_version: "1".into(),
        input: args(),
        context: json!({"argument":"digest","need":"N1","literal":{"$serde_json::private::Number":"not-number"}}),
    };
    let observation = bridge
        .observe("measurement", &request, scope(), deadline())
        .unwrap();
    assert_eq!(observation.value["present"], false);
    assert_eq!(
        observation.observed_at.as_deref(),
        Some("2026-10-24T12:00:00Z")
    );
    assert_eq!(observation.context, Some(request.context.clone()));
    assert_eq!(observation.request, Some(request.acquisition()));
    let again = bridge
        .call("measurement", args(), scope(), deadline())
        .unwrap();
    assert_eq!(again.observed_at, "2026-10-24T12:00:00Z");
    assert_eq!(again.coverage.len(), 1);
    assert_eq!(
        again.coverage[0].strength,
        LogicalStrength::ExistentialWitness
    );
    assert_eq!(again.metrics.requests, 3);
    assert!(again.metrics.request_bytes > 0);
    assert_eq!(fixture.server.calls.load(Ordering::SeqCst), 2);
    assert_eq!(bridge.diagnostics()["fixture"]["server_starts"], 1);
    drop(bridge);
    fixture.close().await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn changed_schema_is_rejected_before_binding_cache_or_tool_execution() {
    let fixture = Fixture::start().await;
    let bridge = fixture.bridge();
    let binding = bridge
        .bind_acquisition("measurement", args(), scope())
        .unwrap();
    fixture.server.drift.store(true, Ordering::SeqCst);
    assert_eq!(
        bridge
            .bind_acquisition("measurement", args(), scope())
            .unwrap_err()
            .kind,
        BridgeErrorKind::SchemaDrift
    );
    assert_eq!(
        bridge.execute_bound(&binding, deadline()).unwrap_err().kind,
        BridgeErrorKind::SchemaDrift
    );
    assert_eq!(fixture.server.calls.load(Ordering::SeqCst), 0);
    drop(bridge);
    fixture.close().await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tool_metadata_does_not_supply_semantics_and_missing_auth_is_a_configuration_gap() {
    let fixture = Fixture::start().await;
    let mut config = fixture.config();
    config.local_catalogues[0].capabilities[0].semantics = None;
    let bridge = EvidenceBridge::new(config, BTreeMap::new()).unwrap();
    assert!(bridge.discover().unwrap().admitted_capabilities.is_empty());
    assert!(
        bridge
            .bind_acquisition("measurement", args(), scope())
            .is_err()
    );
    assert_eq!(fixture.server.calls.load(Ordering::SeqCst), 0);
    drop(bridge);
    let mut config = fixture.config();
    if let TransportConfig::Http { token_env, .. } = &mut config.connections[0].transport {
        *token_env = Some("EAL_TEST_TOKEN".into());
    }
    config.local_catalogues[0].capabilities[0]
        .provider
        .configuration_sha256 = config.connections[0].configuration_identity().unwrap();
    let bridge = EvidenceBridge::new(config, BTreeMap::new()).unwrap();
    let report = bridge.discover().unwrap();
    assert!(report.incomplete);
    assert!(report.admitted_capabilities.is_empty());
    assert!(!serde_json::to_string(&report).unwrap().contains("Bearer "));
    drop(bridge);
    fixture.close().await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn identity_timestamp_errors_and_tool_failures_do_not_become_false_measurements() {
    let fixture = Fixture::start().await;
    let bridge = fixture.bridge();
    fixture.server.result.lock().unwrap()["measurement"]["cluster"] = json!("production");
    assert_eq!(
        bridge
            .call("measurement", args(), scope(), deadline())
            .unwrap_err()
            .kind,
        BridgeErrorKind::Protocol
    );
    fixture.server.result.lock().unwrap()["measurement"]["cluster"] = json!("dev");
    fixture
        .server
        .result
        .lock()
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("observed_at");
    assert_eq!(
        bridge
            .call("measurement", args(), scope(), deadline())
            .unwrap_err()
            .kind,
        BridgeErrorKind::Protocol
    );
    fixture.server.tool_error.store(true, Ordering::SeqCst);
    assert_eq!(
        bridge
            .call("measurement", args(), scope(), deadline())
            .unwrap_err()
            .kind,
        BridgeErrorKind::Acquisition
    );
    drop(bridge);
    fixture.close().await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn forged_scope_or_public_binding_is_not_executable() {
    let fixture = Fixture::start().await;
    let bridge = fixture.bridge();
    assert!(
        bridge
            .bind_acquisition("measurement", args(), json!({"cluster":"production"}))
            .is_err()
    );
    let mut binding = bridge
        .bind_acquisition("measurement", args(), scope())
        .unwrap();
    binding
        .descriptor
        .semantics
        .as_mut()
        .unwrap()
        .qualification_reference = "forged/1".into();
    binding.identity.configuration_sha256 = canonical_identity(&(
        fixture.config().connections[0]
            .configuration_identity()
            .unwrap(),
        canonical_identity(&binding.descriptor).unwrap(),
    ))
    .unwrap();
    assert!(bridge.execute_bound(&binding, deadline()).is_err());
    assert_eq!(fixture.server.calls.load(Ordering::SeqCst), 0);
    drop(bridge);
    fixture.close().await;
}
#[test]
fn operator_boundaries_reject_unapproved_endpoints_and_ambiguous_json() {
    let bytes =
        br#"{"schema":"eal.evidence-config/1","schema":"eal.evidence-config/1","connections":[]}"#;
    assert!(EvidenceConfig::from_json(bytes).is_err());
    for url in [
        "http://example.com/mcp",
        "https://user:pass@example.com/mcp",
        "https://example.com/mcp?token=secret",
        "file:///etc/passwd",
    ] {
        let config = EvidenceConfig {
            native_providers: vec![],
            schema: eal_evidence::config::CONFIG_SCHEMA.into(),
            connections: vec![],
            local_catalogues: vec![],
            approved_catalogues: vec![ApprovedCatalogue { url: url.into() }],
            discovery: DiscoveryLimits::default(),
        };
        assert!(config.validate().is_err(), "{url}");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn approved_catalogue_discovery_follows_configured_connections_and_never_redirects() {
    let fixture = Fixture::start().await;
    let mut config = fixture.config();
    let catalogue = config.local_catalogues.remove(0);
    let body = serde_json::to_string(&catalogue).unwrap();
    let hits = Arc::new(AtomicUsize::new(0));
    let selected_hits = hits.clone();
    let lists = fixture.server.lists.clone();
    let selected_body = body.clone();
    let router = axum::Router::new()
        .route(
            "/catalogue",
            axum::routing::get(move || {
                let body = selected_body.clone();
                let hits = selected_hits.clone();
                let lists = lists.clone();
                async move {
                    assert!(
                        lists.load(Ordering::SeqCst) > 0,
                        "configured MCP discovery must run first"
                    );
                    hits.fetch_add(1, Ordering::SeqCst);
                    body
                }
            }),
        )
        .route(
            "/redirect",
            axum::routing::get(|| async { axum::response::Redirect::temporary("/catalogue") }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    config.approved_catalogues = vec![ApprovedCatalogue {
        url: format!("{origin}/catalogue"),
    }];
    let bridge = EvidenceBridge::new(config.clone(), BTreeMap::new()).unwrap();
    let report = bridge.discover().unwrap();
    assert_eq!(report.admitted_capabilities, ["measurement"]);
    assert_eq!(report.metrics.requests, 3);
    assert_eq!(hits.load(Ordering::SeqCst), 1);
    drop(bridge);
    config.approved_catalogues[0].url = format!("{origin}/redirect");
    let bridge = EvidenceBridge::new(config.clone(), BTreeMap::new()).unwrap();
    let report = bridge.discover().unwrap();
    assert!(report.incomplete);
    assert!(report.admitted_capabilities.is_empty());
    assert_eq!(
        hits.load(Ordering::SeqCst),
        1,
        "catalogue redirect is not approved"
    );
    drop(bridge);
    config.approved_catalogues[0].url = format!("{origin}/catalogue");
    config.discovery.bytes = 1024;
    let bridge = EvidenceBridge::new(config, BTreeMap::new()).unwrap();
    let report = bridge.discover().unwrap();
    assert!(report.incomplete);
    assert!(report.admitted_capabilities.is_empty());
    assert!(body.len() > 1024);
    drop(bridge);
    task.abort();
    fixture.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bounded_json_text_can_discover_output_identity_unknown_before_acquisition() {
    let fixture = Fixture::start().await;
    let mut config = fixture.config();
    config.connections[0].tools[0].response_format = ResponseFormat::JsonText { block: 0 };
    config.connections[0].tools[0]
        .scope_pointers
        .insert("resource_uid".into(), "/payload/resource_uid".into());
    let config_sha = config.connections[0].configuration_identity().unwrap();
    let descriptor = &mut config.local_catalogues[0].capabilities[0];
    descriptor.provider.configuration_sha256 = config_sha;
    descriptor
        .semantics
        .as_mut()
        .unwrap()
        .identity_dimensions
        .push("resource_uid".into());
    fixture.server.result.lock().unwrap()["measurement"]["payload"]["resource_uid"] =
        json!("new-resource-uid");
    *fixture.server.text.lock().unwrap() =
        Some(serde_json::to_string(&*fixture.server.result.lock().unwrap()).unwrap());
    let bridge = EvidenceBridge::new(config, BTreeMap::new()).unwrap();
    let observation = bridge
        .call("measurement", args(), scope(), deadline())
        .unwrap();
    assert_eq!(
        observation.value["payload"]["resource_uid"],
        "new-resource-uid"
    );
    assert_eq!(observation.observed_at, "2026-10-24T12:00:00Z");
    *fixture.server.text.lock().unwrap() =
        Some("{\"observed_at\":\"first\",\"observed_at\":\"second\"}".into());
    assert_eq!(
        bridge
            .call("measurement", args(), scope(), deadline())
            .unwrap_err()
            .kind,
        BridgeErrorKind::Protocol
    );
    drop(bridge);
    fixture.close().await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn actual_credential_scope_invalidates_reuse_without_exposing_values() {
    let fixture = Fixture::start().await;
    let mut config = fixture.config();
    if let TransportConfig::Http { token_env, .. } = &mut config.connections[0].transport {
        *token_env = Some("EAL_TEST_TOKEN".into());
    }
    config.local_catalogues[0].capabilities[0]
        .provider
        .configuration_sha256 = config.connections[0].configuration_identity().unwrap();
    let first = EvidenceBridge::new(
        config.clone(),
        BTreeMap::from([("EAL_TEST_TOKEN".into(), "first-private-credential".into())]),
    )
    .unwrap();
    let second = EvidenceBridge::new(
        config,
        BTreeMap::from([("EAL_TEST_TOKEN".into(), "second-private-credential".into())]),
    )
    .unwrap();
    let left = first
        .bind_acquisition("measurement", args(), scope())
        .unwrap();
    let right = second
        .bind_acquisition("measurement", args(), scope())
        .unwrap();
    assert_ne!(
        left.identity.authentication_scope_sha256,
        right.identity.authentication_scope_sha256
    );
    assert_eq!(
        left.identity.configuration_sha256,
        right.identity.configuration_sha256
    );
    let displayed = serde_json::to_string(&left.identity).unwrap();
    assert!(!displayed.contains("private-credential"));
    assert!(
        !serde_json::to_string(&first.discover().unwrap())
            .unwrap()
            .contains("private-credential")
    );
    assert_eq!(fixture.server.calls.load(Ordering::SeqCst), 0);
    drop(first);
    drop(second);
    fixture.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn capability_rate_limit_is_shared_by_bridge_clones_and_rejects_before_another_sdk_call() {
    let fixture = Fixture::start().await;
    let mut config = fixture.config();
    config.local_catalogues[0].capabilities[0]
        .limits
        .requests_per_minute = Some(1);
    fixture.server.result.lock().unwrap()["measurement"]["present"] = json!(true);
    let bridge = EvidenceBridge::new(config, BTreeMap::new()).unwrap();
    let binding = bridge
        .bind_acquisition("measurement", args(), scope())
        .unwrap();
    assert_eq!(
        bridge.execute_bound(&binding, deadline()).unwrap().value["present"],
        true
    );
    let lists = fixture.server.lists.load(Ordering::SeqCst);
    let start = Instant::now();
    let failure = bridge
        .clone()
        .execute_bound(&binding, start + Duration::from_millis(100))
        .unwrap_err();
    assert_eq!(failure.kind, BridgeErrorKind::Budget);
    assert!(failure.message.contains("request rate"));
    assert!(
        start.elapsed() < Duration::from_secs(1),
        "a minute-long rate wait must be rejected before waiting past its deadline"
    );
    assert_eq!(fixture.server.calls.load(Ordering::SeqCst), 1);
    assert_eq!(fixture.server.lists.load(Ordering::SeqCst), lists);
    assert_eq!(bridge.diagnostics()["fixture"]["server_starts"], 1);
    drop(bridge);
    fixture.close().await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn capability_concurrency_deadline_does_not_cancel_or_replay_an_admitted_measurement() {
    let fixture = Fixture::start().await;
    fixture.server.delay_ms.store(200, Ordering::SeqCst);
    let bridge = fixture.bridge();
    let binding = bridge
        .bind_acquisition("measurement", args(), scope())
        .unwrap();
    let selected = bridge.clone();
    let first_binding = binding.clone();
    let first =
        tokio::task::spawn_blocking(move || selected.execute_bound(&first_binding, deadline()));
    tokio::time::timeout(Duration::from_secs(2), async {
        while fixture.server.calls.load(Ordering::SeqCst) == 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let selected = bridge.clone();
    let next_binding = binding.clone();
    let rejected = tokio::task::spawn_blocking(move || {
        selected.execute_bound(&next_binding, Instant::now() + Duration::from_millis(20))
    })
    .await
    .unwrap()
    .unwrap_err();
    assert_eq!(rejected.kind, BridgeErrorKind::Budget);
    assert_eq!(fixture.server.calls.load(Ordering::SeqCst), 1);
    first.await.unwrap().unwrap();
    assert_eq!(fixture.server.peak.load(Ordering::SeqCst), 1);
    fixture.server.delay_ms.store(0, Ordering::SeqCst);
    bridge.execute_bound(&binding, deadline()).unwrap();
    assert_eq!(
        fixture.server.calls.load(Ordering::SeqCst),
        2,
        "the released permit admits a subsequent operation exactly once"
    );
    assert_eq!(bridge.diagnostics()["fixture"]["server_starts"], 1);
    drop(bridge);
    fixture.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn failed_configured_discovery_consumes_reserved_capacity_before_other_connections() {
    let fixture = Fixture::start().await;
    fixture.server.list_error.store(true, Ordering::SeqCst);
    for maximum in [1, 2, 4] {
        fixture.server.initializations.store(0, Ordering::SeqCst);
        fixture.server.lists.store(0, Ordering::SeqCst);
        let mut config = fixture.config();
        let connection = config.connections.remove(0);
        config.connections = (0..12)
            .map(|index| {
                let mut connection = connection.clone();
                connection.id = format!("connection-{index}");
                connection
            })
            .collect();
        config.local_catalogues.clear();
        config.discovery.requests = maximum;
        config.discovery.pages_per_connection = 4;
        config.discovery.bytes = 4096;
        let bridge = EvidenceBridge::new(config, BTreeMap::new()).unwrap();
        let report = bridge.discover().unwrap();
        let actual = fixture.server.initializations.load(Ordering::SeqCst)
            + fixture.server.lists.load(Ordering::SeqCst);
        assert!(
            actual <= maximum,
            "observed {actual} SDK requests for a budget of {maximum}"
        );
        assert!(report.incomplete);
        assert!(report.admitted_capabilities.is_empty());
        assert!(report.budget_usage.requests_upper_bound <= maximum as u64);
        if maximum == 1 {
            assert_eq!(actual, 0);
            assert_eq!(report.budget_usage.requests_upper_bound, 0);
        } else {
            assert_eq!(actual, 2);
            assert_eq!(report.budget_usage.requests_upper_bound, maximum as u64);
            assert_eq!(report.budget_usage.bytes_upper_bound, 4096);
            assert_eq!(report.budget_usage.pages_by_connection.len(), 1);
            assert_eq!(
                report.budget_usage.pages_by_connection["connection-0"],
                maximum as u64 - 1
            );
            assert_eq!(
                report.metrics.requests, 0,
                "unknown failed requests must not be reported as measured successes"
            );
        }
        drop(bridge);
    }
    fixture.close().await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn discovery_reports_each_connections_pages_instead_of_summing_initializations_into_page_usage()
 {
    let fixture = Fixture::start().await;
    let mut config = fixture.config();
    let connection = config.connections.remove(0);
    config.connections = (0..12)
        .map(|index| {
            let mut connection = connection.clone();
            connection.id = format!("connection-{index}");
            connection
        })
        .collect();
    config.local_catalogues.clear();
    let bridge = EvidenceBridge::new(config, BTreeMap::new()).unwrap();
    let report = bridge.discover().unwrap();
    assert_eq!(fixture.server.initializations.load(Ordering::SeqCst), 12);
    assert_eq!(fixture.server.lists.load(Ordering::SeqCst), 12);
    assert_eq!(report.budget_usage.requests_upper_bound, 24);
    assert_eq!(report.metrics.requests, 24);
    assert_eq!(
        report.budget_usage.bytes_upper_bound,
        report.metrics.response_bytes
    );
    assert_eq!(report.budget_usage.pages_by_connection.len(), 12);
    assert!(
        report
            .budget_usage
            .pages_by_connection
            .values()
            .all(|pages| *pages == 1)
    );
    assert_eq!(report.limits.pages_per_connection, 16);
    drop(bridge);
    fixture.close().await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn failed_registry_discovery_cannot_reuse_its_unknown_byte_allowance() {
    let fixture = Fixture::start().await;
    let calls = Arc::new(AtomicUsize::new(0));
    let selected = calls.clone();
    let router = axum::Router::new().fallback(move || {
        let selected = selected.clone();
        async move {
            selected.fetch_add(1, Ordering::SeqCst);
            axum::http::StatusCode::SERVICE_UNAVAILABLE
        }
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let mut config = fixture.config();
    config.connections.clear();
    config.local_catalogues.clear();
    config.approved_catalogues = (0..12)
        .map(|index| ApprovedCatalogue {
            url: format!("{origin}/catalogue-{index}"),
        })
        .collect();
    config.discovery.requests = 2;
    config.discovery.bytes = 1024;
    let bridge = EvidenceBridge::new(config, BTreeMap::new()).unwrap();
    let report = bridge.discover().unwrap();
    assert!(report.incomplete);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(report.budget_usage.requests_upper_bound, 1);
    assert_eq!(report.budget_usage.bytes_upper_bound, 1024);
    assert!(report.budget_usage.pages_by_connection.is_empty());
    drop(bridge);
    task.abort();
    fixture.close().await;
}
