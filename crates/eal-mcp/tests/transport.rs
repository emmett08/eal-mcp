#![cfg(unix)]
//! Actual native SDK sessions over byte streams and TCP.
mod common;
use eal_core::{ExecutionLimits, methods::default_registry};
use eal_mcp::{
    auth::HttpPolicy,
    server::{Server, ServerOptions},
    settings::ServerSettings,
    transport,
};
use eal_runtime::{
    catalogue::{RegisterOptions, WorkspaceKnowledgeCatalogue},
    process::Supervisor,
    scheduler::CollectionScheduler,
    service::{ReasoningService, ServiceConfig},
};
use rmcp::{
    ClientHandler, ServiceExt,
    model::{
        CallToolRequestParams, ClientCapabilities, ClientConfig, Implementation, ProtocolVersion,
    },
    transport::{StreamableHttpClientTransport, async_rw::AsyncRwTransport},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio_util::sync::CancellationToken;

static SEQUENCE: AtomicUsize = AtomicUsize::new(0);
struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "eal-mcp-transport-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn service(&self) -> Arc<ReasoningService> {
        Arc::new(
            ReasoningService::new(ServiceConfig {
                workspace: self.0.clone(),
                registry_path: None,
                database_path: None,
                method_registry: default_registry().clone(),
                method_executor: None,
                scheduler: CollectionScheduler::default(),
                supervisor: Supervisor::new(PathBuf::from("/unused-supervisor")),
                limits: ExecutionLimits::default(),
                lease_timeout: Duration::from_secs(3),
            })
            .unwrap(),
        )
    }
    fn registered(&self) -> Server {
        let service = self.service();
        std::fs::write(
            self.0.join("synthetic.eal"),
            "language \"EAL/3\"\nenvironment lab {\n require true\n}\nclaim outcome {\n statement \"Synthetic outcome.\"\n environment lab\n}\n",
        )
        .unwrap();
        WorkspaceKnowledgeCatalogue::new(Arc::clone(&service))
            .unwrap()
            .register(
                "synthetic.eal",
                RegisterOptions {
                    entry_id: Some("synthetic"),
                    ..Default::default()
                },
            )
            .unwrap();
        Server::new(
            service,
            ServerOptions {
                known_entries: vec!["synthetic".into()],
                ..Default::default()
            },
        )
        .unwrap()
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[derive(Clone)]
struct LegacyClient(ProtocolVersion);
impl ClientHandler for LegacyClient {
    fn get_info(&self) -> ClientConfig {
        ClientConfig::new(
            ClientCapabilities::default(),
            Implementation::new("native-transport-test", "1"),
        )
        .with_protocol_version(self.0.clone())
    }
}
fn call(name: &str, arguments: &Value) -> CallToolRequestParams {
    CallToolRequestParams::new(name.to_owned())
        .with_arguments(arguments.as_object().unwrap().clone())
}

#[tokio::test]
async fn sdk_byte_stream_roundtrip_preserves_inventory_and_actual_framework_errors() {
    for registered in [false, true] {
        let workspace = Workspace::new();
        let server = if registered {
            workspace.registered()
        } else {
            Server::new(workspace.service(), ServerOptions::default()).unwrap()
        };
        let (client_pipe, server_pipe) = tokio::io::duplex(64 * 1024);
        let (server_reader, server_writer) = tokio::io::split(server_pipe);
        let (client_reader, client_writer) = tokio::io::split(client_pipe);
        let server_task = tokio::spawn(async move {
            server
                .serve(AsyncRwTransport::new(server_reader, server_writer))
                .await
                .unwrap()
                .waiting()
                .await
                .unwrap()
        });
        let client = LegacyClient(ProtocolVersion::V_2025_11_25)
            .serve(AsyncRwTransport::new(client_reader, client_writer))
            .await
            .unwrap();
        assert_eq!(
            client.peer_info().unwrap().protocol_version,
            ProtocolVersion::V_2025_11_25
        );
        let tools = client.list_tools(None).await.unwrap().tools;
        common::assert_inventory(serde_json::to_value(tools).unwrap(), registered);
        for (tool, arguments) in common::invalid_calls(registered) {
            let result = client.call_tool(call(tool, &arguments)).await.unwrap();
            assert_eq!(result.is_error, Some(true), "{tool}: {arguments}");
            assert_eq!(result.structured_content, None);
            assert!(!result.content.is_empty());
        }
        if registered {
            let result = client
                .call_tool(call(
                    "eal_sources",
                    &json!({"limit":{"$serde_json::private::Number":"1"}}),
                ))
                .await
                .unwrap();
            assert_eq!(
                result.is_error,
                Some(true),
                "ordinary private object key changed type across SDK wire codec: {result:?}"
            );
            assert!(
                result.content[0]
                    .as_text()
                    .unwrap()
                    .text
                    .contains("EAL_OPERATION_INTEGER")
            );
        }
        if !registered {
            let result = client
                .call_tool(call(
                    "eal_grounded",
                    &json!({"arguments":["a","b"],"attacks":[["a","b"]]}),
                ))
                .await
                .unwrap();
            assert_eq!(result.is_error, Some(false));
            assert_eq!(result.structured_content.unwrap()["accepted"], json!(["a"]));
        }
        client.cancel().await.unwrap();
        tokio::time::timeout(Duration::from_secs(3), server_task)
            .await
            .unwrap()
            .unwrap();
    }
}

#[tokio::test]
async fn stateless_http_official_client_negotiates_published_and_legacy_protocols() {
    let workspace = Workspace::new();
    let server = Server::new(workspace.service(), ServerOptions::default()).unwrap();
    let settings = ServerSettings {
        workspace: workspace.0.clone(),
        transport: "http".into(),
        ..Default::default()
    };
    let policy = HttpPolicy::from_settings(&settings, &BTreeMap::new()).unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/mcp", listener.local_addr().unwrap());
    let cancellation = CancellationToken::new();
    let stop = cancellation.clone();
    let task = tokio::spawn(async move {
        transport::serve_http(server, listener, "/mcp", policy, stop)
            .await
            .unwrap()
    });
    for version in [ProtocolVersion::V_2025_11_25, ProtocolVersion::V_2025_03_26] {
        let client = LegacyClient(version.clone())
            .serve(StreamableHttpClientTransport::from_uri(url.clone()))
            .await
            .unwrap();
        assert_eq!(client.peer_info().unwrap().protocol_version, version);
        assert_eq!(client.list_tools(None).await.unwrap().tools.len(), 11);
        let result = client
            .call_tool(call("eal_validate", &json!({"source":""})))
            .await
            .unwrap();
        assert_eq!(result.is_error, Some(false));
        assert_eq!(result.structured_content.unwrap()["valid"], false);
        client.cancel().await.unwrap();
    }
    let raw = reqwest::Client::new();
    let duplicate = raw.post(&url).header("Content-Type", "application/json").header("Accept", "application/json, text/event-stream").header("MCP-Protocol-Version", "2025-11-25")
        .body(r#"{"jsonrpc":"2.0","id":99,"method":"tools/call","params":{"name":"eal_validate","arguments":{"source":7,"source":""}}}"#)
        .send().await.unwrap();
    assert_eq!(duplicate.status(), http::StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert!(
        duplicate
            .text()
            .await
            .unwrap()
            .contains("Duplicate JSON object key")
    );
    let draft = raw
        .post(&url)
        .header("MCP-Protocol-Version", "2026-07-28")
        .send()
        .await
        .unwrap();
    assert_eq!(draft.status(), http::StatusCode::BAD_REQUEST);
    cancellation.cancel();
    tokio::time::timeout(Duration::from_secs(3), task)
        .await
        .unwrap()
        .unwrap();
}

#[test]
fn deeply_nested_direct_calls_reject_and_destroy_values_on_a_small_stack() {
    let workspace = Workspace::new();
    let server = Server::new(workspace.service(), ServerOptions::default()).unwrap();
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            for name in ["eal_validate", "eal_unknown"] {
                let mut value = Value::Null;
                for _ in 0..10_000 {
                    value = Value::Array(vec![value]);
                }
                let error = runtime
                    .block_on(server.call(name, value, CancellationToken::new()))
                    .unwrap_err();
                assert!(error.contains("resource limit") || error.contains("Unknown tool"));
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[tokio::test]
async fn configuration_and_direct_schema_validation_fail_closed() {
    let workspace = Workspace::new();
    let policy = HttpPolicy::from_settings(
        &ServerSettings {
            workspace: workspace.0.clone(),
            transport: "http".into(),
            ..Default::default()
        },
        &BTreeMap::new(),
    )
    .unwrap();
    for path in ["/:literal", "/*literal"] {
        let _ = transport::http_router(
            Server::new(workspace.service(), ServerOptions::default()).unwrap(),
            path,
            policy.clone(),
            CancellationToken::new(),
        )
        .unwrap();
    }
    assert!(
        Server::new(
            workspace.service(),
            ServerOptions {
                max_in_flight: 0,
                ..Default::default()
            }
        )
        .is_err()
    );
    assert!(
        Server::new(
            workspace.service(),
            ServerOptions {
                exposure: Some("registered".into()),
                ..Default::default()
            }
        )
        .is_err()
    );
    let server = workspace.registered();
    assert!(
        server
            .call(
                "eal_validate",
                json!({"source":""}),
                CancellationToken::new()
            )
            .await
            .unwrap_err()
            .starts_with("Unknown tool:")
    );
    assert!(
        eal_mcp::guard::validate_schema(
            &json!({}),
            &json!({"$ref":"https://example.invalid/schema"})
        )
        .unwrap_err()
        .contains("External schema")
    );
    assert!(
        eal_mcp::guard::validate_schema(&json!({}), &json!({"$ref":"file:///etc/passwd"})).is_err()
    );
    eal_mcp::guard::validate_schema(
        &json!({"$ref":"https://example.invalid/literal"}),
        &json!({"enum":[{"$ref":"https://example.invalid/literal"}]}),
    )
    .unwrap();
}
