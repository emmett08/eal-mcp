//! Ordinary EAL collection crosses the command boundary and reuses one SDK measurement.
use eal_collector_contracts::{
    CommandBinding, CommandRegistry, ToolKind, capability::*, execution::*,
};
use eal_core::{ExecutionLimits, composition, evaluation, methods};
use eal_evidence::{
    ConnectionConfig, DiscoveryLimits, EvidenceConfig, ObservationTime, ResponseFormat,
    ToolMapping, TransportConfig,
};
use eal_runtime::{
    acquisition::ToolRegistry, collection::EvidenceRuntime, process::Supervisor,
    scheduler::CollectionScheduler, store::RunStore,
};
use rmcp::{ServerHandler, model::*};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;

#[derive(Default)]
struct Meter {
    calls: AtomicU64,
    lists: AtomicU64,
    http_requests: AtomicU64,
    request_bytes: AtomicU64,
    response_bytes: AtomicU64,
}
impl Meter {
    fn snapshot(&self, elapsed: Duration) -> Value {
        json!({"tool_calls":self.calls.load(Ordering::SeqCst),"tools_list_calls":self.lists.load(Ordering::SeqCst),"http_requests":self.http_requests.load(Ordering::SeqCst),"request_body_bytes":self.request_bytes.load(Ordering::SeqCst),"response_body_bytes":self.response_bytes.load(Ordering::SeqCst),"elapsed_microseconds":u64::try_from(elapsed.as_micros()).unwrap(),"generated_adapter_files":0})
    }
    fn reset(&self) {
        for counter in [
            &self.calls,
            &self.lists,
            &self.http_requests,
            &self.request_bytes,
            &self.response_bytes,
        ] {
            counter.store(0, Ordering::SeqCst);
        }
    }
}
async fn meter_http(
    axum::extract::State(meter): axum::extract::State<Arc<Meter>>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let (parts, body) = request.into_parts();
    let bytes = axum::body::to_bytes(body, 1_048_576).await.unwrap();
    meter.http_requests.fetch_add(1, Ordering::SeqCst);
    meter
        .request_bytes
        .fetch_add(bytes.len() as u64, Ordering::SeqCst);
    let response = next
        .run(axum::http::Request::from_parts(
            parts,
            axum::body::Body::from(bytes),
        ))
        .await;
    let (parts, body) = response.into_parts();
    let bytes = axum::body::to_bytes(body, 1_048_576).await.unwrap();
    meter
        .response_bytes
        .fetch_add(bytes.len() as u64, Ordering::SeqCst);
    axum::http::Response::from_parts(parts, axum::body::Body::from(bytes))
}
fn input_schema() -> Value {
    json!({"type":"object","properties":{"cluster":{"type":"string"}},"required":["cluster"],"additionalProperties":false})
}
fn output_schema() -> Value {
    json!({"type":"object","properties":{"cluster":{"type":"string"},"latency":{"type":"number"}},"required":["cluster","latency"],"additionalProperties":false})
}
#[derive(Clone)]
struct Measurement {
    meter: Arc<Meter>,
    observed_at: String,
}
impl ServerHandler for Measurement {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("acceptance-measurement", "1"))
    }
    async fn list_tools(
        &self,
        _: Option<PaginatedRequestParams>,
        _: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<ListToolsResult, rmcp::ErrorData> {
        self.meter.lists.fetch_add(1, Ordering::SeqCst);
        Ok(ListToolsResult::with_all_items(vec![Tool::new(
            "measure_latency",
            "Existing instrument",
            Arc::new(input_schema().as_object().unwrap().clone()),
        )]))
    }
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<CallToolResponse, rmcp::ErrorData> {
        self.meter.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(request.arguments.unwrap()["cluster"], "dev");
        Ok(CallToolResult::structured(
            json!({"observed_at":self.observed_at,"measurement":{"cluster":"dev","latency":25}}),
        )
        .into())
    }
}
struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("eal-shared-acceptance-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn config(url: String) -> EvidenceConfig {
    let connection = ConnectionConfig {
        id: "instrument".into(),
        authentication_scope: "development-reader".into(),
        transport: TransportConfig::Http {
            url,
            token_env: None,
        },
        tools: vec![ToolMapping {
            tool: "measure_latency".into(),
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
    let input = input_schema();
    let output = output_schema();
    let capability = CapabilityDescriptor {
        id: "latency".into(),
        version: "1".into(),
        description: "Operator-qualified latency observation".into(),
        provider: ProviderIdentity {
            id: "instrument-provider".into(),
            version: "1".into(),
            transport: TransportKind::McpHttp,
            connection: Some(connection.id.clone()),
            tool: Some("measure_latency".into()),
            authentication_scope: connection.authentication_scope.clone(),
            configuration_sha256: connection.configuration_identity().unwrap(),
        },
        input_schema_sha256: schema_identity(&input).unwrap(),
        input_schema: input,
        output_schema_sha256: schema_identity(&output).unwrap(),
        output_schema: output,
        semantics: Some(MeasurementSemantics {
            qualification_reference: "acceptance.instrument/1".into(),
            subject_kinds: vec!["service_instance".into()],
            methods: vec!["instrument.latency/1".into()],
            support_types: vec![SupportType::Observation],
            identity_dimensions: vec!["cluster".into()],
            scope_constraints: BTreeMap::new(),
            scope_bindings: BTreeMap::from([("cluster".into(), "/cluster".into())]),
            quantities: vec![MeasurementQuantity {
                name: "latency".into(),
                unit: "ms".into(),
                output_pointer: "/latency".into(),
            }],
            coverage: vec![CoverageGuarantee {
                strength: LogicalStrength::ExistentialWitness,
                population: None,
                completeness_pointer: None,
                sample_size_pointer: None,
            }],
            time: TimeSemantics {
                historical_intervals: false,
                historical_input: None,
                original_observation_time: true,
                uncertainty_seconds: Some(1),
            },
            provenance: ProvenanceSemantics {
                authorities: vec!["instrument".into()],
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
        costs: CapabilityCosts::unknown("No production cost estimate"),
    };
    EvidenceConfig {
        schema: eal_evidence::config::CONFIG_SCHEMA.into(),
        connections: vec![connection],
        local_catalogues: vec![CapabilityCatalogue {
            schema: CAPABILITY_CATALOGUE_SCHEMA.into(),
            capabilities: vec![capability],
        }],
        native_providers: vec![],
        approved_catalogues: vec![],
        discovery: DiscoveryLimits::default(),
    }
}
fn graph(config: &EvidenceConfig) -> EvidenceExecutionPlan {
    let capability = &config.local_catalogues[0].capabilities[0];
    let source = SourceIdentity {
        provider_id: capability.provider.id.clone(),
        capability_id: capability.id.clone(),
        capability_version: capability.version.clone(),
        input_schema_sha256: capability.input_schema_sha256.clone(),
        output_schema_sha256: capability.output_schema_sha256.clone(),
        qualified_contract_sha256: capability.measurement_identity().unwrap(),
        configuration_sha256: "0".repeat(64),
        authentication_scope_sha256: "0".repeat(64),
        scope: json!({"cluster":"dev"}),
        historical_interval: None,
    };
    EvidenceExecutionPlan {
        schema: PLAN_SCHEMA.into(),
        acquisitions: vec![AcquisitionStep {
            id: "source".into(),
            source,
            arguments: json!({"cluster":"dev"}),
            bindings: vec![],
            for_each: None,
            availability: SourceAvailability::Configured,
            reuse: ReusePolicy::FreshOrAcquire,
            max_age_seconds: None,
        }],
        derivations: vec![],
        evaluations: [("strict", 20), ("loose", 30)]
            .into_iter()
            .map(|(id, threshold)| RequirementEvaluation {
                id: format!("evaluation-{id}"),
                requirement_id: id.into(),
                inputs: vec![OutputReference {
                    step_id: "source".into(),
                    pointer: "/latency".into(),
                }],
                criterion: EvaluationCriterion::Numeric {
                    comparison: NumericComparison::AtMost,
                    threshold: json!(threshold),
                    value_unit: "ms".into(),
                    threshold_unit: "ms".into(),
                },
                max_age_seconds: Some(3600),
                required_coverage: RequiredCoverage::Unspecified,
                applicability: vec![],
                challenges: vec![],
            })
            .collect(),
        scheduling: SchedulingPolicy::default(),
    }
}
fn write(path: &Path, value: &impl serde::Serialize) {
    std::fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}
fn supervisor() -> PathBuf {
    let parent = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    eal_process::companion_in(&parent)
}
fn collect(
    root: &Path,
    config: &EvidenceConfig,
    plan: &EvidenceExecutionPlan,
    shared: bool,
) -> Value {
    std::fs::create_dir(root).unwrap();
    let config_path = root.join("config.json");
    let plan_path = root.join("execution.json");
    write(&config_path, config);
    write(&plan_path, plan);
    let digest = canonical_digest(plan).unwrap();
    let case_digest = "a".repeat(64);
    let mut source = String::from(
        "language \"EAL/3\"\nenvironment development {\n require cluster == \"dev\"\n}\n",
    );
    let mut tools = BTreeMap::new();
    for id in ["strict", "loose"] {
        let tool = format!("{id}_tool");
        let store = root.join(if shared { "shared-sources" } else { id });
        let argv = vec![
            env!("CARGO_BIN_EXE_eal-collector").into(),
            "support-graph".into(),
            "--plan".into(),
            plan_path.to_str().unwrap().into(),
            "--config".into(),
            config_path.to_str().unwrap().into(),
            "--store".into(),
            store.to_str().unwrap().into(),
        ];
        tools.insert(
            tool.clone(),
            CommandBinding {
                kind: ToolKind::Command,
                version: "1".into(),
                argv,
                timeout_seconds: 30,
                max_output_bytes: 65536,
                inherit_env: vec![],
                parallel_safe: false,
            },
        );
        let input = json!({"schema":"engarg.support-acquisition/1","case_digest":case_digest,"execution_plan_sha256":digest,"requirement_id":id});
        source.push_str(&format!("tool {tool} {{\n version \"1\"\n}}\nevidence {id} {{\n tool {tool}\n kind support_evaluation\n environment development\n max_age 3600\n input {input}\n require collected == true\n require criterion_satisfied == true\n}}\n"));
    }
    let tools = CommandRegistry { tools }.to_toml().unwrap();
    std::fs::write(root.join("tools.toml"), &tools).unwrap();
    std::fs::write(root.join("argument.eal"), &source).unwrap();
    let program = composition::compile(&source, None, "", ExecutionLimits::default()).unwrap();
    let runtime = EvidenceRuntime::new(
        root.to_path_buf(),
        ToolRegistry::from_toml(&tools).unwrap(),
        RunStore::new(root.join("runs.sqlite3")).unwrap(),
        methods::default_registry().clone(),
        CollectionScheduler::new(1).unwrap(),
        Supervisor::new(supervisor()),
    )
    .unwrap();
    let context = json!({"cluster":"dev","engarg_export_digest":case_digest,"engarg_execution_plan_sha256":digest,"literal":{"$serde_json::private::Number":"literal-context"}});
    let collection = runtime.collect(&program, &context, None).unwrap();
    let assessment = evaluation::evaluate_default(
        &program,
        &collection["records"],
        &chrono::Utc::now().to_rfc3339(),
        &context,
    )
    .unwrap();
    json!({"collection":collection,"assessment":assessment})
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ordinary_eal_commands_share_one_measurement_across_independent_thresholds() {
    let workspace = Workspace::new();
    let meter = Arc::new(Meter::default());
    let observed_at = (chrono::Utc::now() - chrono::Duration::seconds(30)).to_rfc3339();
    let measurement = Measurement {
        meter: meter.clone(),
        observed_at: observed_at.clone(),
    };
    let config_http = rmcp::transport::StreamableHttpServerConfig::default()
        .with_legacy_session_mode(false)
        .with_json_response(true)
        .disable_allowed_hosts()
        .disable_allowed_origins();
    let service = rmcp::transport::StreamableHttpService::new(
        move || Ok(measurement.clone()),
        Arc::new(
            rmcp::transport::streamable_http_server::session::never::NeverSessionManager::default(),
        ),
        config_http,
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/mcp", listener.local_addr().unwrap());
    let stop = CancellationToken::new();
    let cancellation = stop.clone();
    let app = axum::Router::new().route_service("/mcp", service).layer(
        axum::middleware::from_fn_with_state(meter.clone(), meter_http),
    );
    let server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(cancellation.cancelled_owned())
            .await
            .unwrap();
    });
    let config = config(url);
    let plan = graph(&config);
    let start = Instant::now();
    let baseline = collect(&workspace.0.join("baseline"), &config, &plan, false);
    let baseline_metrics = meter.snapshot(start.elapsed());
    assert_eq!(baseline_metrics["tool_calls"], 2, "{baseline}");
    meter.reset();
    let start = Instant::now();
    let shared = collect(&workspace.0.join("shared"), &config, &plan, true);
    let shared_metrics = meter.snapshot(start.elapsed());
    assert_eq!(shared_metrics["tool_calls"], 1, "{shared}");
    for result in [&baseline, &shared] {
        for id in ["strict", "loose"] {
            assert_eq!(
                result["collection"]["records"][id]["status"], "ok",
                "{result}"
            );
            assert_eq!(
                result["collection"]["records"][id]["collected_at"],
                observed_at
            );
            assert_eq!(
                result["collection"]["records"][id]["context"]["literal"],
                json!({"$serde_json::private::Number":"literal-context"})
            );
            assert_eq!(
                result["collection"]["records"][id]["value"]["criterion_satisfied"],
                id == "loose"
            );
        }
        assert_eq!(
            result["assessment"]["evidence"]["strict"]["status"],
            "unavailable"
        );
        assert_eq!(
            result["assessment"]["evidence"]["loose"]["status"],
            "available"
        );
    }
    let records = &shared["collection"]["records"];
    assert_eq!(
        records["strict"]["value"]["source_ids"],
        records["loose"]["value"]["source_ids"]
    );
    assert!(shared_metrics["request_body_bytes"].as_u64().unwrap() > 0);
    assert!(shared_metrics["response_body_bytes"].as_u64().unwrap() > 0);
    fn generated_sources(path: &Path) -> usize {
        std::fs::read_dir(path)
            .unwrap()
            .flatten()
            .map(|entry| {
                let path = entry.path();
                if path.is_dir() {
                    generated_sources(&path)
                } else {
                    usize::from(path.extension().is_some_and(|extension| extension == "rs"))
                }
            })
            .sum()
    }
    let generated = generated_sources(&workspace.0);
    assert_eq!(generated, 0);
    let report = json!({"schema":"eal.support-acceptance-metrics/1","fixture":"official SDK stateless MCP HTTP, local loopback, one 25ms instrument observation, two ordinary EAL command collectors","protocol_version":"2025-11-25","metric_scope":"HTTP request/response body bytes including MCP initialization and notifications, excludes headers/TLS; monotonic local elapsed includes collector subprocess startup and durable stores; not production performance evidence","baseline":"one isolated source store for each requirement","shared":"one persistent source store across separate requirement collector processes","baseline_metrics":baseline_metrics,"shared_metrics":shared_metrics,"original_observed_at":observed_at,"requirement_outcomes":{"strict":"contradicted","loose":"satisfied"}});
    if let Some(path) = std::env::var_os("EAL_EVIDENCE_METRICS_OUTPUT") {
        write(Path::new(&path), &report);
    }
    eprintln!("{report}");
    stop.cancel();
    tokio::time::timeout(Duration::from_secs(3), server)
        .await
        .unwrap()
        .unwrap();
}
