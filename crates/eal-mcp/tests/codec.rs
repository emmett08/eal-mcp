//! Lossless arbitrary JSON payloads through the official SDK codecs.
use eal_core::ExecutionLimits;
use rmcp::{
    ClientHandler, ServiceExt,
    model::{
        CallToolRequestParams, ClientCapabilities, ClientConfig, Implementation, ProtocolVersion,
    },
    transport::{StreamableHttpClientTransport, async_rw::AsyncRwTransport},
};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use tokio_util::sync::CancellationToken;
#[derive(Clone)]
struct LegacyClient(ProtocolVersion);
impl ClientHandler for LegacyClient {
    fn get_info(&self) -> ClientConfig {
        ClientConfig::new(
            ClientCapabilities::default(),
            Implementation::new("codec-test", "1"),
        )
        .with_protocol_version(self.0.clone())
    }
}
fn call(name: &str, arguments: &Value) -> CallToolRequestParams {
    CallToolRequestParams::new(name.to_owned())
        .with_arguments(arguments.as_object().unwrap().clone())
}

#[derive(Clone)]
struct Echo(Value);
impl rmcp::ServerHandler for Echo {
    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::ListToolsResult, rmcp::ErrorData> {
        let schema = json!({"type":"object","properties":{"payload":{"default":self.0}}});
        let mut tool = rmcp::model::Tool::new(
            "echo",
            "Codec regression",
            Arc::new(schema.as_object().unwrap().clone()),
        )
        .with_raw_output_schema(Arc::new(schema.as_object().unwrap().clone()));
        tool.meta = Some(rmcp::model::MetaObject::from(
            json!({"payload":self.0}).as_object().unwrap().clone(),
        ));
        let mut result = rmcp::model::ListToolsResult::with_all_items(vec![tool]);
        result.meta = Some(rmcp::model::MetaObject::from(
            json!({"payload":self.0}).as_object().unwrap().clone(),
        ));
        Ok(result)
    }
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::CallToolResponse, rmcp::ErrorData> {
        if request.name == "error" {
            return Err(rmcp::ErrorData::invalid_params(
                "synthetic",
                Some(self.0.clone()),
            ));
        }
        assert_eq!(request.arguments.as_ref().unwrap()["payload"], self.0);
        assert_eq!(context.meta.get("payload"), Some(&self.0));
        let mut result =
            rmcp::model::CallToolResult::structured(request.arguments.unwrap()["payload"].clone());
        result.meta = Some(rmcp::model::MetaObject::from(
            json!({"payload":self.0}).as_object().unwrap().clone(),
        ));
        Ok(result.into())
    }
}
fn literal_payload() -> Value {
    eal_core::json::parse(r#"{"$serde_json::private::Number":"1","$serde_json::private::RawValue":"raw","$eal.literal-key/plain":{"$eal.literal-key/$serde_json::private::Number":"literal"},"nested":[{"$serde_json::private::Number":"2","extra":true}],"big":12345678901234567890123456789012345678901234567890,"float":1.25,"negative_zero":-0.0}"#, &ExecutionLimits::default()).unwrap()
}
async fn verify_echo(
    client: &rmcp::service::RunningService<rmcp::RoleClient, LegacyClient>,
    payload: &Value,
) {
    let tools = client.list_tools(None).await.unwrap();
    assert_eq!(
        tools.tools[0].input_schema["properties"]["payload"]["default"],
        *payload
    );
    assert_eq!(
        tools.tools[0].output_schema.as_ref().unwrap()["properties"]["payload"]["default"],
        *payload
    );
    assert_eq!(
        tools.tools[0].meta.as_ref().unwrap().get("payload"),
        Some(payload)
    );
    assert_eq!(tools.meta.as_ref().unwrap().get("payload"), Some(payload));
    let mut request = call("echo", &json!({"payload":payload}));
    request.meta = Some(rmcp::model::RequestMetaObject::from(
        json!({"payload":payload}).as_object().unwrap().clone(),
    ));
    let result = client.call_tool(request).await.unwrap();
    assert_eq!(result.structured_content.as_ref(), Some(payload));
    assert_eq!(result.meta.as_ref().unwrap().get("payload"), Some(payload));
    match client.call_tool(call("error", &json!({}))).await {
        Err(rmcp::ServiceError::McpError(error)) => assert_eq!(error.data.as_ref(), Some(payload)),
        other => panic!("Expected protocol error with literal data, got {other:?}"),
    }
}

#[tokio::test]
async fn official_sdk_preserves_literal_keys_numbers_schemas_metadata_and_error_data_both_transports()
 {
    let payload = literal_payload();
    let server = Echo(payload.clone());
    let (client_pipe, server_pipe) = tokio::io::duplex(64 * 1024);
    let (server_reader, server_writer) = tokio::io::split(server_pipe);
    let (client_reader, client_writer) = tokio::io::split(client_pipe);
    let task = tokio::spawn(async move {
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
    tokio::time::timeout(Duration::from_secs(3), verify_echo(&client, &payload))
        .await
        .expect("byte-stream codec deadline");
    client.cancel().await.unwrap();
    tokio::time::timeout(Duration::from_secs(3), task)
        .await
        .unwrap()
        .unwrap();

    let cancellation = CancellationToken::new();
    let server = Echo(payload.clone());
    let config = rmcp::transport::StreamableHttpServerConfig::default()
        .with_legacy_session_mode(false)
        .with_json_response(true)
        .disable_allowed_hosts()
        .disable_allowed_origins();
    let service = rmcp::transport::StreamableHttpService::new(
        move || Ok(server.clone()),
        Arc::new(
            rmcp::transport::streamable_http_server::session::never::NeverSessionManager::default(),
        ),
        config,
    );
    let router = axum::Router::new().route_service("/mcp", service);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/mcp", listener.local_addr().unwrap());
    let stop = cancellation.clone();
    let task = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(stop.cancelled_owned())
            .await
            .unwrap();
    });
    let client = LegacyClient(ProtocolVersion::V_2025_11_25)
        .serve(StreamableHttpClientTransport::from_uri(url))
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(3), verify_echo(&client, &payload))
        .await
        .expect("HTTP codec deadline");
    client.cancel().await.unwrap();
    cancellation.cancel();
    tokio::time::timeout(Duration::from_secs(3), task)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn http_client_bounds_success_and_error_bodies_with_and_without_content_length() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    for status in [200, 503] {
        for chunked in [false, true] {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let url = format!("http://{}/mcp", listener.local_addr().unwrap());
            let peer = tokio::spawn(async move {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut bytes = [0u8; 4096];
                let _ = socket.read(&mut bytes).await;
                let framing = if chunked {
                    "Transfer-Encoding: chunked\r\n".to_owned()
                } else {
                    format!("Content-Length: {}\r\n", 16 * 1024 * 1024 + 1)
                };
                socket.write_all(format!("HTTP/1.1 {status} synthetic\r\nContent-Type: application/json\r\n{framing}Connection: close\r\n\r\n").as_bytes()).await.unwrap();
                if chunked {
                    let chunk = vec![b'x'; 64 * 1024];
                    for _ in 0..257 {
                        if socket.write_all(b"10000\r\n").await.is_err()
                            || socket.write_all(&chunk).await.is_err()
                            || socket.write_all(b"\r\n").await.is_err()
                        {
                            break;
                        }
                    }
                }
            });
            let result = tokio::time::timeout(
                Duration::from_secs(4),
                LegacyClient(ProtocolVersion::V_2025_11_25)
                    .serve(StreamableHttpClientTransport::from_uri(url)),
            )
            .await
            .unwrap();
            let error = match result {
                Ok(client) => {
                    client.cancel().await.unwrap();
                    panic!("Oversized peer initialized")
                }
                Err(error) => error.to_string(),
            };
            assert!(
                error.contains("exceeds 16 MiB"),
                "status={status} chunked={chunked}: {error}"
            );
            peer.await.unwrap();
        }
    }
}
