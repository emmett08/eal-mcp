//! Independent configuration checks and Unix native-store MCP roundtrips.
#[cfg(unix)]
mod common;
use eal_mcp::settings::ServerSettings;
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);

#[test]
fn invalid_http_configuration_and_credentials_fail_before_any_state_is_created() {
    let root = std::env::temp_dir().join(format!(
        "eal-mcp-uncreated-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let settings = ServerSettings {
        workspace: root.join("workspace"),
        database: Some(root.join("database/runs.sqlite3")),
        transport: "http".into(),
        token_env: Some("EAL_PRIVATE_HTTP_TOKEN".into()),
        ..Default::default()
    };
    for secret in [
        None,
        Some(""),
        Some("invalid credential"),
        Some("invalid\tcredential"),
        Some("invalid\ncredential"),
        Some("invalid-é"),
        Some("invalid-\x7f"),
    ] {
        let environment = secret
            .map(|secret| {
                BTreeMap::from([("EAL_PRIVATE_HTTP_TOKEN".to_owned(), secret.to_owned())])
            })
            .unwrap_or_default();
        let error = match eal_mcp::application::build(settings.clone(), &environment) {
            Ok(_) => panic!("Invalid bearer configuration was admitted"),
            Err(error) => error,
        };
        assert!(error.contains("EAL_PRIVATE_HTTP_TOKEN"), "{error}");
        if let Some(secret) = secret.filter(|secret| !secret.is_empty()) {
            assert!(!error.contains(secret));
        }
        assert!(
            !root.exists(),
            "Credential rejection must precede workspace/database creation"
        );
    }
    let settings = ServerSettings {
        host: "0.0.0.0".into(),
        token_env: None,
        ..settings
    };
    assert!(eal_mcp::application::build(settings, &BTreeMap::new()).is_err());
    assert!(!root.exists());
    for path in ["/mcp/{tenant}", "/mcp/}"] {
        let settings = ServerSettings {
            workspace: root.join("workspace"),
            database: Some(root.join("database/runs.sqlite3")),
            transport: "http".into(),
            path: path.into(),
            ..Default::default()
        };
        let error = match eal_mcp::application::build(settings, &BTreeMap::new()) {
            Ok(_) => panic!("Unsupported route syntax was admitted"),
            Err(error) => error,
        };
        assert!(
            error.contains("path"),
            "Route validation must precede workspace/companion resolution: {error}"
        );
        assert!(!root.exists());
    }
}

// Initialization settings are ordinary JSON too. This test uses no native
// store or processes and remains enabled on Windows along with pure settings.
mod capability_codec_review {
    use eal_core::ExecutionLimits;
    use rmcp::{
        ServerHandler, ServiceExt,
        model::{
            ClientCapabilities, ClientConfig, Implementation, ProtocolVersion, ServerCapabilities,
            ServerConfig,
        },
        transport::async_rw::AsyncRwTransport,
    };
    use serde_json::Value;
    use std::collections::BTreeMap;

    #[derive(Clone)]
    struct CapabilityServer(Value);
    impl ServerHandler for CapabilityServer {
        fn get_info(&self) -> ServerConfig {
            let mut capabilities = ServerCapabilities::default();
            let settings = BTreeMap::from([(
                "literal/review".to_owned(),
                self.0.as_object().unwrap().clone(),
            )]);
            capabilities.experimental = Some(settings.clone());
            capabilities.extensions = Some(settings);
            ServerConfig::new(capabilities).with_protocol_version(ProtocolVersion::V_2025_11_25)
        }
    }

    #[tokio::test]
    async fn initialization_capability_settings_preserve_literal_keys_and_exact_numbers_in_both_directions()
     {
        let huge = "1234567890".repeat(40);
        let payload=eal_core::json::parse(&format!(r#"{{
            "$serde_json::private::Number":"literal-number-key",
            "$eal.literal-key/$serde_json::private::Number":"literal-prefix-key",
            "$eal.literal-key/$eal.literal-key/$serde_json::private::Number":"literal-double-prefix-key",
            "raw":{{"$serde_json::private::RawValue":"literal-raw-key"}},
            "huge":{huge},"decimal":0.1000000000000000000000000000000000000001,"tiny":1e-400,"negative_zero":-0.0
        }}"#),&ExecutionLimits::default()).unwrap();
        let settings = BTreeMap::from([(
            "literal/review".to_owned(),
            payload.as_object().unwrap().clone(),
        )]);
        let mut capabilities = ClientCapabilities::default();
        capabilities.experimental = Some(settings.clone());
        capabilities.extensions = Some(settings.clone());
        let (client_pipe, server_pipe) = tokio::io::duplex(64 * 1024);
        let (server_reader, server_writer) = tokio::io::split(server_pipe);
        let (client_reader, client_writer) = tokio::io::split(client_pipe);
        let server = CapabilityServer(payload);
        let server_task = tokio::spawn(async move {
            server
                .serve(AsyncRwTransport::new(server_reader, server_writer))
                .await
                .unwrap()
        });
        let client = ClientConfig::new(
            capabilities,
            Implementation::new("independent-capability-codec-review", "1"),
        )
        .with_protocol_version(ProtocolVersion::V_2025_11_25)
        .serve(AsyncRwTransport::new(client_reader, client_writer))
        .await
        .unwrap();
        let server = server_task.await.unwrap();
        let client_observed = &client.peer_info().unwrap().capabilities;
        assert_eq!(
            client_observed.experimental,
            Some(settings.clone()),
            "Server experimental settings changed on the wire"
        );
        assert_eq!(
            client_observed.extensions,
            Some(settings.clone()),
            "Server extension settings changed on the wire"
        );
        let server_observed = &server.peer_info().unwrap().capabilities;
        assert_eq!(
            server_observed.experimental,
            Some(settings.clone()),
            "Client experimental settings changed on the wire"
        );
        assert_eq!(
            server_observed.extensions,
            Some(settings),
            "Client extension settings changed on the wire"
        );
        client.cancel().await.unwrap();
        server.cancel().await.unwrap();
    }
}

#[cfg(unix)]
mod unix {
    use eal_core::{ExecutionLimits, methods};
    use eal_mcp::{
        auth::HttpPolicy,
        server::{Server, ServerOptions},
        settings::ServerSettings,
        transport::http_router,
    };
    use eal_runtime::{
        catalogue::{RegisterOptions, WorkspaceKnowledgeCatalogue},
        lease::WorkspaceAcquisition,
        process::Supervisor,
        scheduler::CollectionScheduler,
        service::{ReasoningService, ServiceConfig, utc_now},
    };
    use rmcp::{
        ServiceExt,
        model::{CallToolRequestParams, ClientConfig, ProtocolVersion},
        transport::{
            StreamableHttpClientTransport,
            streamable_http_client::StreamableHttpClientTransportConfig,
        },
    };
    use serde_json::{Value, json};
    use std::{
        collections::BTreeMap,
        fs,
        future::{Future, poll_fn},
        path::PathBuf,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        task::Poll,
        time::{Duration, Instant},
    };
    use tokio_util::sync::CancellationToken;

    static NEXT: AtomicUsize = AtomicUsize::new(0);
    const TOKEN: &str = "synthetic-http-token.ABC_123~";
    const SOURCE: &str = r#"language "EAL/3"
environment lab {
  require "site" == "bench"
}
tool reader {
  version "1"
}
evidence observed {
  tool reader
  kind test
  environment lab
  max_age 3600
  require passed == true
}
reasoning measured {
  method "structured/1"
  rationale "A synthetic predicate supplies bounded support."
}
claim outcome {
  statement "The synthetic predicate holds."
  environment lab
}
argument support = [evidence observed] via measured => outcome
"#;

    struct Fixture {
        path: PathBuf,
        service: Arc<ReasoningService>,
    }
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "eal-mcp-adversarial-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            fs::write(path.join("case.eal"), SOURCE).unwrap();
            let context = json!({"site":"bench","private":"private-operator-context"});
            fs::write(
                path.join("observation.json"),
                json!({"value":{"passed":true,"secret":"private-tool-observation"},
            "observed_at":utc_now(),"context":context,
            "request":{"tool":"reader","tool_version":"1","input":{},"context":context}})
                .to_string(),
            )
            .unwrap();
            fs::write(
                path.join("tools.toml"),
                "[tools.reader]\nkind='json_file'\nversion='1'\npath='observation.json'\n",
            )
            .unwrap();
            let service = Arc::new(
                ReasoningService::new(ServiceConfig {
                    workspace: path.clone(),
                    registry_path: Some(path.join("tools.toml")),
                    database_path: Some(path.join("runs.sqlite3")),
                    method_registry: methods::default_registry().clone(),
                    method_executor: None,
                    scheduler: CollectionScheduler::default(),
                    // These tests import JSON files and run only the built-in structured method.
                    supervisor: Supervisor::new(PathBuf::from("/bin/false")),
                    limits: ExecutionLimits::default(),
                    lease_timeout: Duration::from_secs(3),
                })
                .unwrap(),
            );
            let catalogue = WorkspaceKnowledgeCatalogue::new(Arc::clone(&service)).unwrap();
            catalogue
                .register(
                    "case.eal",
                    RegisterOptions {
                        entry_id: Some("visible"),
                        context: Some(&context),
                        claims: Some(&["outcome".into()]),
                        label: None,
                    },
                )
                .unwrap();
            fs::write(path.join("hidden.eal"), SOURCE).unwrap();
            catalogue
                .register(
                    "hidden.eal",
                    RegisterOptions {
                        entry_id: Some("hidden"),
                        context: Some(&context),
                        claims: Some(&["outcome".into()]),
                        label: None,
                    },
                )
                .unwrap();
            Self { path, service }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    struct Running {
        url: String,
        cancel: CancellationToken,
        task: tokio::task::JoinHandle<()>,
    }
    impl Drop for Running {
        fn drop(&mut self) {
            self.cancel.cancel();
            self.task.abort();
        }
    }
    async fn start(fixture: &Fixture, registered: bool) -> Running {
        let server = Server::new(
            Arc::clone(&fixture.service),
            ServerOptions {
                known_entries: if registered {
                    vec!["visible".into()]
                } else {
                    vec![]
                },
                exposure: Some(if registered { "registered" } else { "operator" }.into()),
                max_in_flight: 2,
            },
        )
        .unwrap();
        let settings = ServerSettings {
            workspace: fixture.path.clone(),
            transport: "http".into(),
            token_env: Some("EAL_TEST_TOKEN".into()),
            allowed_hosts: vec!["engineering.example".into()],
            allowed_origins: vec!["https://engineering.example".into()],
            ..Default::default()
        };
        let policy = HttpPolicy::from_settings(
            &settings,
            &BTreeMap::from([("EAL_TEST_TOKEN".into(), TOKEN.into())]),
        )
        .unwrap();
        let cancel = CancellationToken::new();
        let router = http_router(server, "/mcp", policy, cancel.clone()).unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/mcp", listener.local_addr().unwrap());
        let shutdown = cancel.clone();
        let task = tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(async move { shutdown.cancelled().await })
                .await
                .unwrap();
        });
        Running { url, cancel, task }
    }
    fn transport(url: &str) -> StreamableHttpClientTransport<reqwest::Client> {
        StreamableHttpClientTransport::from_config(
            StreamableHttpClientTransportConfig::with_uri(url.to_owned()).auth_header(TOKEN),
        )
    }
    fn params(tool: &str, arguments: &Value) -> CallToolRequestParams {
        CallToolRequestParams::new(tool.to_owned())
            .with_arguments(arguments.as_object().unwrap().clone())
    }
    fn reference() -> Value {
        serde_json::from_str(include_str!("../../../fixtures/mcp/reference.json")).unwrap()
    }

    #[tokio::test]
    async fn actual_http_preserves_the_pinned_loopback_and_same_origin_admission_profile() {
        let fixture = Fixture::new();
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let cases = reference();
        assert_eq!(cases["http_admission"].as_array().unwrap().len(), 10);
        for case in cases["http_admission"].as_array().unwrap() {
            let settings = ServerSettings {
                workspace: fixture.path.clone(),
                transport: "http".into(),
                host: case["bind_host"].as_str().unwrap().into(),
                token_env: Some("EAL_TEST_TOKEN".into()),
                allowed_hosts: serde_json::from_value(case["allowed_hosts"].clone()).unwrap(),
                allowed_origins: serde_json::from_value(case["allowed_origins"].clone()).unwrap(),
                ..Default::default()
            };
            let policy = HttpPolicy::from_settings(
                &settings,
                &BTreeMap::from([("EAL_TEST_TOKEN".into(), TOKEN.into())]),
            )
            .unwrap();
            let cancellation = CancellationToken::new();
            let server =
                Server::new(Arc::clone(&fixture.service), ServerOptions::default()).unwrap();
            let router = http_router(server, "/mcp", policy, cancellation.clone()).unwrap();
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let url = format!("http://{}/mcp", listener.local_addr().unwrap());
            let shutdown = cancellation.clone();
            let task = tokio::spawn(async move {
                axum::serve(listener, router)
                    .with_graceful_shutdown(shutdown.cancelled_owned())
                    .await
                    .unwrap();
            });
            let running = Running {
                url,
                cancel: cancellation,
                task,
            };
            let mut request=client.post(&running.url).header("Host",case["host_header"].as_str().unwrap())
            .header("Authorization",format!("Bearer {TOKEN}")).header("Accept","application/json, text/event-stream")
            .json(&json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"independent-origin-probe","version":"1"}}}));
            if let Some(origin) = case["origin"].as_str() {
                request = request.header("Origin", origin);
            }
            let status = request.send().await.unwrap().status().as_u16();
            let expected = if case["status"] == 204 {
                200
            } else {
                case["status"].as_u64().unwrap() as u16
            };
            assert_eq!(status, expected, "{}", case["name"]);
        }
    }

    #[derive(Clone)]
    struct SlowServer {
        listed: Arc<AtomicUsize>,
        called: Arc<AtomicUsize>,
    }
    impl rmcp::ServerHandler for SlowServer {
        fn get_info(&self) -> rmcp::model::ServerConfig {
            rmcp::model::ServerConfig::new(
                rmcp::model::ServerCapabilities::builder()
                    .enable_tools()
                    .build(),
            )
        }
        async fn list_tools(
            &self,
            _request: Option<rmcp::model::PaginatedRequestParams>,
            _context: rmcp::service::RequestContext<rmcp::RoleServer>,
        ) -> Result<rmcp::model::ListToolsResult, rmcp::ErrorData> {
            self.listed.fetch_add(1, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(600)).await;
            Ok(rmcp::model::ListToolsResult::with_all_items(vec![
                serde_json::from_value(reference()["operator_tools"][0].clone()).unwrap(),
            ]))
        }
        async fn call_tool(
            &self,
            _request: CallToolRequestParams,
            _context: rmcp::service::RequestContext<rmcp::RoleServer>,
        ) -> Result<rmcp::model::CallToolResponse, rmcp::ErrorData> {
            self.called.fetch_add(1, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(600)).await;
            Ok(rmcp::model::CallToolResult::structured(json!({"synthetic":true})).into())
        }
    }

    #[tokio::test]
    async fn text_client_uses_one_deadline_across_discovery_and_invocation() {
        use rmcp::transport::streamable_http_server::{
            StreamableHttpServerConfig, StreamableHttpService, session::never::NeverSessionManager,
        };
        let listed = Arc::new(AtomicUsize::new(0));
        let called = Arc::new(AtomicUsize::new(0));
        let server = SlowServer {
            listed: Arc::clone(&listed),
            called: Arc::clone(&called),
        };
        let cancellation = CancellationToken::new();
        let service = StreamableHttpService::new(
            move || Ok(server.clone()),
            Arc::new(NeverSessionManager::default()),
            StreamableHttpServerConfig::default()
                .with_legacy_session_mode(false)
                .with_json_response(true)
                .with_stateless_protocol_metadata_required(false)
                .with_cancellation_token(cancellation.clone()),
        );
        let router = axum::Router::new().route_service("/mcp", service);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/mcp", listener.local_addr().unwrap());
        let shutdown = cancellation.clone();
        let task = tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(shutdown.cancelled_owned())
                .await
                .unwrap();
        });
        let running = Running {
            url,
            cancel: cancellation,
            task,
        };
        let target = eal_mcp::client::create_http_target_with_client(
            &running.url,
            None,
            &BTreeMap::new(),
            reqwest::Client::builder().no_proxy().build().unwrap(),
        )
        .unwrap();
        let start = Instant::now();
        let error = eal_mcp::host::dispatch_request(r#"{"operation":"describe"}"#, target, 1.0)
            .await
            .unwrap_err();
        assert_eq!(error, "MCP session exceeded its configured deadline");
        assert_eq!(listed.load(Ordering::SeqCst), 1);
        assert_eq!(
            called.load(Ordering::SeqCst),
            1,
            "The deadline must expire after discovery, during the separate invocation"
        );
        assert!(
            start.elapsed() < Duration::from_secs(3),
            "A timed-out session must finish bounded cleanup promptly"
        );
    }

    #[tokio::test]
    async fn dropping_an_admitted_caller_keeps_its_permit_until_blocking_collection_finishes() {
        let fixture = Fixture::new();
        let external = WorkspaceAcquisition::new(&fixture.path)
            .unwrap()
            .hold(Duration::ZERO)
            .unwrap();
        let server = Server::new(
            Arc::clone(&fixture.service),
            ServerOptions {
                max_in_flight: 1,
                ..Default::default()
            },
        )
        .unwrap();
        let mut first = Box::pin(server.call(
        "eal_collect",
        json!({"source":SOURCE,"context":{"site":"bench","private":"private-operator-context"}}),
        CancellationToken::new(),
    ));
        // A first poll acquires the available permit synchronously and queues the
        // blocking operation, which must then wait for the external workspace lease.
        poll_fn(|context| {
            assert!(first.as_mut().poll(context).is_pending());
            Poll::Ready(())
        })
        .await;
        drop(first);
        let second = server.call(
            "eal_validate",
            json!({"source":SOURCE}),
            CancellationToken::new(),
        );
        assert!(
            tokio::time::timeout(Duration::from_millis(100), second)
                .await
                .is_err(),
            "Dropping the call future must not release the active operation's admission permit"
        );
        assert!(
            fixture
                .service
                .store()
                .list(Some("collection"), 100)
                .unwrap()
                .is_empty()
        );
        drop(external);
        tokio::time::timeout(Duration::from_secs(3), async {
            while fixture
                .service
                .store()
                .list(Some("collection"), 100)
                .unwrap()
                .is_empty()
            {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("Detached admitted collection must retain ownership through its durable commit");
        let valid = tokio::time::timeout(
            Duration::from_secs(2),
            server.call(
                "eal_validate",
                json!({"source":SOURCE}),
                CancellationToken::new(),
            ),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(valid["valid"], true);
    }

    #[tokio::test]
    async fn actual_http_rejects_foreign_host_origin_and_credentials_before_operations() {
        let fixture = Fixture::new();
        let server = start(&fixture, false).await;
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let initialize = json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
        "protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"independent-admission-probe","version":"1"}}});
        for (host, origin, authorization, status) in [
            (
                "foreign.example",
                None,
                Some(format!("Bearer {TOKEN}")),
                421,
            ),
            (
                "127.0.0.1",
                Some("https://foreign.example"),
                Some(format!("Bearer {TOKEN}")),
                403,
            ),
            (
                "engineering.example",
                Some("https://engineering.example"),
                None,
                401,
            ),
            (
                "engineering.example",
                Some("https://engineering.example"),
                Some(format!("Bearer {TOKEN}-wrong")),
                401,
            ),
        ] {
            let mut request = client
                .post(&server.url)
                .header("Host", host)
                .header("Accept", "application/json, text/event-stream")
                .json(&initialize);
            if let Some(origin) = origin {
                request = request.header("Origin", origin);
            }
            if let Some(authorization) = authorization {
                request = request.header("Authorization", authorization);
            }
            let response = request.send().await.unwrap();
            assert_eq!(
                response.status().as_u16(),
                status,
                "host={host}, origin={origin:?}"
            );
            assert!(!response.text().await.unwrap().contains(TOKEN));
        }
        assert!(
            fixture
                .service
                .store()
                .list(Some("observation"), 100)
                .unwrap()
                .is_empty()
        );
        let peer = ClientConfig::default()
            .with_protocol_version(ProtocolVersion::LATEST_WITH_INITIALIZE)
            .serve(transport(&server.url))
            .await
            .unwrap();
        let result = peer
            .call_tool(params("eal_validate", &json!({"source":SOURCE})))
            .await
            .unwrap();
        let result = serde_json::to_value(result).unwrap();
        assert_eq!(result["structuredContent"]["valid"], true);
        peer.cancel().await.unwrap();
    }

    #[tokio::test]
    async fn real_tools_list_and_schema_failures_match_the_pinned_framework() {
        let fixture = Fixture::new();
        let reference = reference();
        for (registered, prefix, count, error_count) in
            [(false, "operator", 11, 6), (true, "registered", 4, 5)]
        {
            let server = start(&fixture, registered).await;
            let peer = ClientConfig::default()
                .with_protocol_version(ProtocolVersion::LATEST_WITH_INITIALIZE)
                .serve(transport(&server.url))
                .await
                .unwrap();
            let tools = serde_json::to_value(peer.list_all_tools().await.unwrap()).unwrap();
            assert_eq!(tools.as_array().unwrap().len(), count);
            crate::common::assert_inventory_compatibility(
                tools,
                &reference[format!("{prefix}_tools")],
                registered,
            );
            let errors = reference[format!("{prefix}_errors")].as_array().unwrap();
            assert_eq!(errors.len(), error_count);
            for case in errors {
                let result = serde_json::to_value(
                    peer.call_tool(params(case["tool"].as_str().unwrap(), &case["arguments"]))
                        .await
                        .unwrap(),
                )
                .unwrap();
                assert_eq!(
                    result["isError"], case["result"]["is_error"],
                    "{}",
                    case["name"]
                );
                assert_eq!(
                    result.get("structuredContent").unwrap_or(&Value::Null),
                    &case["result"]["structured_content"],
                    "{}",
                    case["name"]
                );
                assert_eq!(
                    result["content"], case["result"]["content"],
                    "{}",
                    case["name"]
                );
            }
            if registered {
                let strict_calls = reference["registered_calls"].as_array().unwrap();
                assert_eq!(strict_calls.len(), 2);
                for case in strict_calls {
                    assert_eq!(case["result"]["is_error"], true);
                    assert!(
                        case["result"]["content"][0]["text"]
                            .as_str()
                            .unwrap()
                            .contains("[type=int_type,")
                    );
                    let result = serde_json::to_value(
                        peer.call_tool(params(case["tool"].as_str().unwrap(), &case["arguments"]))
                            .await
                            .unwrap(),
                    )
                    .unwrap();
                    assert_eq!(result["isError"], true, "{}", case["name"]);
                    // Source Pydantic exposes its Python version/URL here. Native
                    // execution retains its own explicit integer-type diagnostic.
                    let field = if case["arguments"].get("limit").is_some() {
                        "limit"
                    } else {
                        "offset"
                    };
                    assert_eq!(
                        result["content"],
                        json!([{"type":"text","text":format!("Tool input requires a strict integer for {field}")}])
                    );
                }
            }
            peer.cancel().await.unwrap();
        }
    }

    #[tokio::test]
    async fn raw_http_integer_arguments_reject_decimal_and_exponent_json_numbers() {
        let fixture = Fixture::new();
        let server = start(&fixture, true).await;
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        for number in ["1.0", "1e0", "1E+0", "0.0", "-0.0"] {
            let response=client.post(&server.url).header("Authorization",format!("Bearer {TOKEN}"))
            .header("MCP-Protocol-Version","2025-11-25").header("Accept","application/json, text/event-stream")
            .header("Content-Type","application/json")
            .body(format!(r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"eal_sources","arguments":{{"limit":{number}}}}}}}"#))
            .send().await.unwrap();
            assert_eq!(response.status().as_u16(), 200, "{number}");
            let response: Value = response.json().await.unwrap();
            assert_eq!(response["result"]["isError"], true, "{number}: {response}");
            assert_eq!(
                response["result"]["content"],
                json!([{"type":"text","text":"Tool input requires a strict integer for limit"}])
            );
        }
    }

    #[tokio::test]
    async fn registered_transport_exposes_only_selected_metadata_and_checked_packets() {
        let fixture = Fixture::new();
        let server = start(&fixture, true).await;
        let peer = ClientConfig::default()
            .with_protocol_version(ProtocolVersion::LATEST_WITH_INITIALIZE)
            .serve(transport(&server.url))
            .await
            .unwrap();
        let sources = serde_json::to_value(
            peer.call_tool(params("eal_sources", &json!({})))
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            sources["structuredContent"]["sources"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        let entry = &sources["structuredContent"]["sources"][0];
        assert_eq!(entry["entry_id"], "visible");
        assert_eq!(entry["claims"], json!(["outcome"]));
        assert!(entry.get("context").is_none());
        assert!(entry.get("source").is_none());
        assert!(!sources.to_string().contains("private-operator-context"));
        for (tool, arguments) in [
            ("eal_collect", json!({"source":SOURCE,"context":{}})),
            (
                "eal_assess_known",
                json!({"entry_id":"hidden","claim":"outcome"}),
            ),
            (
                "eal_assess_known",
                json!({"entry_id":"visible","claim":"outcome","context":{"site":"elsewhere"}}),
            ),
        ] {
            let result =
                serde_json::to_value(peer.call_tool(params(tool, &arguments)).await.unwrap())
                    .unwrap();
            assert_eq!(result["isError"], true);
        }
        assert!(
            fixture
                .service
                .store()
                .list(Some("observation"), 100)
                .unwrap()
                .is_empty()
        );
        for (collected, reused) in [(1, 0), (0, 1)] {
            let result = serde_json::to_value(
                peer.call_tool(params(
                    "eal_assess_known",
                    &json!({"entry_id":"visible","claim":"outcome"}),
                ))
                .await
                .unwrap(),
            )
            .unwrap();
            assert_ne!(result["isError"], true, "{result}");
            let result = &result["structuredContent"];
            assert_eq!(result["status"], "supported");
            assert_eq!(result["collected_count"], collected);
            assert_eq!(result["reused_count"], reused);
            assert_eq!(
                result["packet"]["claims"]["outcome"]["prose_verified"],
                false
            );
            assert_eq!(
                result["full_explanation"]["assessment_id"],
                result["assessment_id"]
            );
            assert!(!result.to_string().contains("private-tool-observation"));
            assert!(!result.to_string().contains("private-operator-context"));
        }
        peer.cancel().await.unwrap();
    }

    #[tokio::test]
    async fn bounded_packet_retrieval_checks_allowlist_without_acquisition_or_refresh() {
        let fixture = Fixture::new();
        let server = start(&fixture, true).await;
        let peer = ClientConfig::default()
            .with_protocol_version(ProtocolVersion::V_2025_11_25)
            .serve(transport(&server.url))
            .await
            .unwrap();
        // All malformed budgets are rejected before even a registered assessment
        // can create observations. Decimal/exponent integers must remain distinct.
        for budget in [
            json!({"max_bytes":4095}),
            json!({"max_bytes":65537}),
            json!({"max_bytes":4096.0}),
            json!({"max_statement_bytes":1.0}),
            json!({"max_statement_bytes":0}),
            json!({"max_statement_bytes":4097}),
            json!({"max_outputs":1}),
            json!(null),
            json!([]),
        ] {
            let rejected = peer
                .call_tool(params(
                    "eal_assess_known",
                    &json!({
                        "entry_id":"visible", "claim":"outcome", "packet_limits":budget
                    }),
                ))
                .await
                .unwrap();
            assert_eq!(rejected.is_error, Some(true), "{budget}");
        }
        assert!(
            fixture
                .service
                .store()
                .list(Some("observation"), 100)
                .unwrap()
                .is_empty()
        );
        let assessed = peer
            .call_tool(params(
                "eal_assess_known",
                &json!({
                    "entry_id":"visible", "claim":"outcome", "packet_limits":{}
                }),
            ))
            .await
            .unwrap();
        assert_eq!(assessed.is_error, Some(false), "{assessed:?}");
        let assessed = assessed.structured_content.unwrap();
        let id = assessed["assessment_id"].as_str().unwrap();
        let original_packet = fixture.service.packet(id, Some("outcome")).unwrap();
        assert_eq!(assessed["packet"], original_packet);
        let observations = fixture
            .service
            .store()
            .list(Some("observation"), 100)
            .unwrap();
        let assessments = fixture
            .service
            .store()
            .list(Some("assessment"), 100)
            .unwrap();

        // Read an identified historical record while the live file is invalid
        // and another caller owns the acquisition lease.
        fs::write(fixture.path.join("case.eal"), "invalid live source").unwrap();
        let _lease = fixture.service.acquisition_lease().unwrap();
        let request = json!({"entry_id":"visible", "assessment_id":id, "claim":"outcome"});
        let retrieved = peer
            .call_tool(params("eal_get_known_packet", &request))
            .await
            .unwrap();
        assert_eq!(retrieved.is_error, Some(false), "{retrieved:?}");
        let retrieved = retrieved.structured_content.unwrap();
        assert_eq!(retrieved["schema"], "EAL/registered-packet/1");
        assert_eq!(retrieved["packet"], original_packet);
        assert_eq!(retrieved["assessed_at"], assessed["assessed_at"]);
        assert!(!retrieved.to_string().contains("private-tool-observation"));
        assert!(!retrieved.to_string().contains("private-operator-context"));
        let mut compact_request = request.clone();
        compact_request["packet_limits"] = json!({"max_bytes":4096,"max_statement_bytes":3});
        let compact = peer
            .call_tool(params("eal_get_known_packet", &compact_request))
            .await
            .unwrap();
        assert_eq!(compact.is_error, Some(false), "{compact:?}");
        let compact = compact.structured_content.unwrap();
        assert!(serde_json::to_vec(&compact["packet"]).unwrap().len() <= 4096);
        assert_eq!(compact["packet"]["claims"]["outcome"]["statement"], "The");
        assert_eq!(
            compact["packet"]["claims"]["outcome"]["statement_truncated"],
            true
        );
        assert_eq!(compact["packet"]["summary_complete"], false);
        for replacement in [json!({"entry_id":"hidden"}), json!({"claim":"other"})] {
            let mut denied = request.clone();
            for (key, value) in replacement.as_object().unwrap() {
                denied[key] = value.clone();
            }
            let denied = peer
                .call_tool(params("eal_get_known_packet", &denied))
                .await
                .unwrap();
            assert_eq!(denied.is_error, Some(true), "{denied:?}");
        }
        // Exercise the text host's discovery and invocation route too.
        let target = eal_mcp::client::create_http_target_with_client(
            &server.url,
            Some("TOKEN"),
            &BTreeMap::from([("TOKEN".into(), TOKEN.into())]),
            reqwest::Client::builder().no_proxy().build().unwrap(),
        )
        .unwrap();
        let mut host_request = compact_request;
        host_request["operation"] = json!("get_known_packet");
        let via_host = eal_mcp::host::dispatch_request(&host_request.to_string(), target, 10.0)
            .await
            .unwrap();
        assert_eq!(via_host["is_error"], false, "{via_host}");
        assert_eq!(via_host["result"], compact);
        assert_eq!(
            fixture
                .service
                .store()
                .list(Some("observation"), 100)
                .unwrap(),
            observations
        );
        assert_eq!(
            fixture
                .service
                .store()
                .list(Some("assessment"), 100)
                .unwrap(),
            assessments
        );
        peer.cancel().await.unwrap();
    }
}
