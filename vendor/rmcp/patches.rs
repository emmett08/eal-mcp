// Exact pinned-SDK payload and wire-codec hunks. No protocol dispatch changes.
const PATCHES: &[Patch] = &[
    // Hunk 000: 000-module
    Patch {
        target: "src/lib.rs",
        before: r#"mod error;"#,
        after: r#"mod literal_json;
mod error;"#,
    },
    // Hunk 001: 001-model-data
    Patch {
        target: "src/model.rs",
        before: r#"    pub data: Option<Value>,
"#,
        after: r#"    #[serde(default, deserialize_with = "crate::literal_json::deserialize")]
    pub data: Option<Value>,
"#,
    },
    // Hunk 002: 002-model-params
    Patch {
        target: "src/model.rs",
        before: r#"pub struct CustomNotification {
    pub method: String,
    pub params: Option<Value>,
"#,
        after: r#"pub struct CustomNotification {
    pub method: String,
    #[serde(default, deserialize_with = "crate::literal_json::deserialize")]
    pub params: Option<Value>,
"#,
    },
    // Hunk 003: 003-model-params
    Patch {
        target: "src/model.rs",
        before: r#"pub struct CustomRequest {
    pub method: String,
    pub params: Option<Value>,
"#,
        after: r#"pub struct CustomRequest {
    pub method: String,
    #[serde(default, deserialize_with = "crate::literal_json::deserialize")]
    pub params: Option<Value>,
"#,
    },
    // Hunk 004: 004-model-arguments
    Patch {
        target: "src/model.rs",
        before: r#"    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<JsonObject>,
"#,
        after: r#"    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "crate::literal_json::deserialize")]
    pub arguments: Option<JsonObject>,
"#,
    },
    // Hunk 005: 005-model-data
    Patch {
        target: "src/model.rs",
        before: r#"    pub data: Value,
"#,
        after: r#"    #[serde(deserialize_with = "crate::literal_json::deserialize")]
    pub data: Value,
"#,
    },
    // Hunk 006: 006-model-metadata
    Patch {
        target: "src/model.rs",
        before: r#"    pub metadata: Option<Value>,
"#,
        after: r#"    #[serde(default, deserialize_with = "crate::literal_json::deserialize")]
    pub metadata: Option<Value>,
"#,
    },
    // Hunk 007: 007-model-content
    Patch {
        target: "src/model.rs",
        before: r#"    pub content: Option<Value>,
"#,
        after: r#"    #[serde(default, deserialize_with = "crate::literal_json::deserialize")]
    pub content: Option<Value>,
"#,
    },
    // Hunk 008: 008-model-structured_content
    Patch {
        target: "src/model.rs",
        before: r#"    pub structured_content: Option<Value>,
"#,
        after: r#"    #[serde(default, deserialize_with = "crate::literal_json::deserialize")]
    pub structured_content: Option<Value>,
"#,
    },
    // Hunk 009: 009-model-arguments
    Patch {
        target: "src/model.rs",
        before: r#"    /// Arguments to pass to the tool (must match the tool's input schema)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<JsonObject>,
"#,
        after: r#"    /// Arguments to pass to the tool (must match the tool's input schema)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "crate::literal_json::deserialize")]
    pub arguments: Option<JsonObject>,
"#,
    },
    // Hunk 010: 010-model-CustomResult
    Patch {
        target: "src/model.rs",
        before: r#"pub struct CustomResult(pub Value);"#,
        after: r#"pub struct CustomResult(#[serde(deserialize_with = "crate::literal_json::deserialize")] pub Value);"#,
    },
    // Hunk 011: 011-tool-input_schema
    Patch {
        target: "src/model/tool.rs",
        before: r#"    pub input_schema: Arc<JsonObject>,
"#,
        after: r#"    #[serde(deserialize_with = "crate::literal_json::deserialize")]
    pub input_schema: Arc<JsonObject>,
"#,
    },
    // Hunk 012: 012-tool-output_schema
    Patch {
        target: "src/model/tool.rs",
        before: r#"    pub output_schema: Option<Arc<JsonObject>>,
"#,
        after: r#"    #[serde(default, deserialize_with = "crate::literal_json::deserialize")]
    pub output_schema: Option<Arc<JsonObject>>,
"#,
    },
    // Hunk 013: 013-meta-MetaObject
    Patch {
        target: "src/model/meta.rs",
        before: r#"pub struct MetaObject(pub JsonObject);"#,
        after: r#"pub struct MetaObject(#[serde(deserialize_with = "crate::literal_json::deserialize")] pub JsonObject);"#,
    },
    // Hunk 014: 014-content-structured_content
    Patch {
        target: "src/model/content.rs",
        before: r#"    pub structured_content: Option<Value>,
"#,
        after: r#"    #[serde(default, deserialize_with = "crate::literal_json::deserialize")]
    pub structured_content: Option<Value>,
"#,
    },
    // Hunk 015: 015-capabilities-tools
    Patch {
        target: "src/model/capabilities.rs",
        before: r#"    pub tools: Option<JsonObject>,
"#,
        after: r#"    #[serde(default, deserialize_with = "crate::literal_json::deserialize")]
    pub tools: Option<JsonObject>,
"#,
    },
    // Hunk 016: 016-capabilities-context
    Patch {
        target: "src/model/capabilities.rs",
        before: r#"    pub context: Option<JsonObject>,
"#,
        after: r#"    #[serde(default, deserialize_with = "crate::literal_json::deserialize")]
    pub context: Option<JsonObject>,
"#,
    },
    // Hunk 017: 017-capabilities-logging
    Patch {
        target: "src/model/capabilities.rs",
        before: r#"    pub logging: Option<JsonObject>,
"#,
        after: r#"    #[serde(default, deserialize_with = "crate::literal_json::deserialize")]
    pub logging: Option<JsonObject>,
"#,
    },
    // Hunk 018: 018-capabilities-completions
    Patch {
        target: "src/model/capabilities.rs",
        before: r#"    pub completions: Option<JsonObject>,
"#,
        after: r#"    #[serde(default, deserialize_with = "crate::literal_json::deserialize")]
    pub completions: Option<JsonObject>,
"#,
    },
    // Hunk 019: 019-async_rw-decode
    Patch {
        target: "src/transport/async_rw.rs",
        before: r#"        match serde_json::from_slice(line) {
"#,
        after: r#"        match crate::literal_json::from_slice(line) {
"#,
    },
    // Hunk 020: 020-async_rw-decode
    Patch {
        target: "src/transport/async_rw.rs",
        before: r#"        serde_json::from_slice(line)
"#,
        after: r#"        crate::literal_json::from_slice(line)
"#,
    },
    // Hunk 021: 021-server_side_http-decode
    Patch {
        target: "src/transport/common/server_side_http.rs",
        before: r#"    match serde_json::from_slice::<ClientJsonRpcMessage>(&collected) {
"#,
        after: r#"    match crate::literal_json::from_slice::<ClientJsonRpcMessage>(&collected) {
"#,
    },
    // Hunk 022: 022-streamable_http_client-decode
    Patch {
        target: "src/transport/common/reqwest/streamable_http_client.rs",
        before: r#"    match serde_json::from_str::<ServerJsonRpcMessage>(body) {
"#,
        after: r#"    match crate::literal_json::from_str::<ServerJsonRpcMessage>(body) {
"#,
    },
    // Hunk 023: 023-streamable_http_client-decode
    Patch {
        target: "src/transport/common/reqwest/streamable_http_client.rs",
        before: r#"                match serde_json::from_slice::<ServerJsonRpcMessage>(&body) {
"#,
        after: r#"                match crate::literal_json::from_slice::<ServerJsonRpcMessage>(&body) {
"#,
    },
    // Hunk 024: 024-client_side_sse-decode
    Patch {
        target: "src/transport/common/client_side_sse.rs",
        before: r#"                                match serde_json::from_str::<ServerJsonRpcMessage>(&data) {
"#,
        after: r#"                                match crate::literal_json::from_str::<ServerJsonRpcMessage>(&data) {
"#,
    },
    // Hunk 025: 025-streamable_http_client-decode
    Patch {
        target: "src/transport/streamable_http_client.rs",
        before: r#"                    let message: ServerJsonRpcMessage = serde_json::from_str(&payload)?;
"#,
        after: r#"                    let message: ServerJsonRpcMessage = crate::literal_json::from_str(&payload)?;
"#,
    },
    // Hunk 026: 026-streamable_http_client-decode
    Patch {
        target: "src/transport/streamable_http_client.rs",
        before: r#"    let error = match serde_json::from_str::<ServerJsonRpcMessage>(body) {
"#,
        after: r#"    let error = match crate::literal_json::from_str::<ServerJsonRpcMessage>(body) {
"#,
    },
    // Hunk 027: 027-unix_socket-decode
    Patch {
        target: "src/transport/common/unix_socket.rs",
        before: r#"                match serde_json::from_slice::<ServerJsonRpcMessage>(&body) {
"#,
        after: r#"                match crate::literal_json::from_slice::<ServerJsonRpcMessage>(&body) {
"#,
    },
    // Hunk 028: 028-http-body-helper
    Patch {
        target: "src/transport/common/reqwest/streamable_http_client.rs",
        before: r#"impl StreamableHttpClient for reqwest::Client {"#,
        after: r#"/// Bound JSON and HTTP error bodies as well as the existing SSE event cap.
async fn bounded_response_body(
    mut response: reqwest::Response,
) -> Result<Vec<u8>, StreamableHttpError<reqwest::Error>> {
    const LIMIT: usize = 16 * 1024 * 1024;
    let over_limit = || StreamableHttpError::UnexpectedServerResponse(Cow::Borrowed(
        "MCP HTTP response body exceeds 16 MiB",
    ));
    if response.content_length().is_some_and(|length| length > LIMIT as u64) {
        return Err(over_limit());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if chunk.len() > LIMIT.saturating_sub(bytes.len()) {
            return Err(over_limit());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

impl StreamableHttpClient for reqwest::Client {"#,
    },
    // Hunk 029: 029-http-error-body
    Patch {
        target: "src/transport/common/reqwest/streamable_http_client.rs",
        before: r#"            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<failed to read response body>".to_owned());"#,
        after: r#"            let body = bounded_response_body(response).await?;
            let body = String::from_utf8_lossy(&body).into_owned();"#,
    },
    // Hunk 030: 030-http-json-body
    Patch {
        target: "src/transport/common/reqwest/streamable_http_client.rs",
        before: r#"                let body = response.bytes().await?;"#,
        after: r#"                let body = bounded_response_body(response).await?;"#,
    },
    Patch {
        target: "src/model.rs",
        before: r###"    Value::deserialize(deserializer).map(Some)"###,
        after: r###"    crate::literal_json::deserialize(deserializer).map(Some)"###,
    },
    Patch {
        target: "src/model/content.rs",
        before: r###"    pub input: super::JsonObject,"###,
        after: r###"    #[serde(deserialize_with = "crate::literal_json::deserialize")]
    pub input: super::JsonObject,"###,
    },
    Patch {
        target: "src/model/capabilities.rs",
        before: r###"pub struct ClientCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<ExperimentalCapabilities>,
"###,
        after: r###"pub struct ClientCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "crate::literal_json::deserialize")]
    pub experimental: Option<ExperimentalCapabilities>,
"###,
    },
    Patch {
        target: "src/model/capabilities.rs",
        before: r###"    /// Keys are extension identifiers (e.g., `"io.modelcontextprotocol/ui"`),
    /// values are per-extension settings objects. An empty object indicates
    /// support with no settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<ExtensionCapabilities>,
"###,
        after: r###"    /// Keys are extension identifiers (e.g., `"io.modelcontextprotocol/ui"`),
    /// values are per-extension settings objects. An empty object indicates
    /// support with no settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "crate::literal_json::deserialize")]
    pub extensions: Option<ExtensionCapabilities>,
"###,
    },
    Patch {
        target: "src/model/capabilities.rs",
        before: r###"pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<ExperimentalCapabilities>,
"###,
        after: r###"pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "crate::literal_json::deserialize")]
    pub experimental: Option<ExperimentalCapabilities>,
"###,
    },
    Patch {
        target: "src/model/capabilities.rs",
        before: r###"    /// Keys are extension identifiers (e.g., `"io.modelcontextprotocol/apps"`),
    /// values are per-extension settings objects. An empty object indicates
    /// support with no settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<ExtensionCapabilities>,
"###,
        after: r###"    /// Keys are extension identifiers (e.g., `"io.modelcontextprotocol/apps"`),
    /// values are per-extension settings objects. An empty object indicates
    /// support with no settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "crate::literal_json::deserialize")]
    pub extensions: Option<ExtensionCapabilities>,
"###,
    },
    Patch {
        target: "src/model/serde_impl.rs",
        before: r###"    _meta: Option<Cow<'a, JsonObject>>,
"###,
        after: r###"    #[serde(default, deserialize_with = "crate::literal_json::deserialize")]
    _meta: Option<Cow<'a, JsonObject>>,
"###,
    },
    Patch {
        target: "src/model/serde_impl.rs",
        before: r###"            .and_then(|v| serde_json::from_value(v).ok());"###,
        after: r###"            .and_then(|value| match value { serde_json::Value::Object(map) => Some(map), _ => None });"###,
    },
    Patch {
        target: "src/model/serde_impl.rs",
        before: r###"        Ok(CustomRequest {"###,
        after: r###"        crate::literal_json::restore_if_wire(&mut params);
        Ok(CustomRequest {"###,
    },
    Patch {
        target: "src/model/serde_impl.rs",
        before: r###"        Ok(CustomNotification {"###,
        after: r###"        crate::literal_json::restore_if_wire(&mut params);
        Ok(CustomNotification {"###,
    },
    Patch {
        target: "src/model/task.rs",
        before: r###"    result: Option<JsonObject>,
"###,
        after: r###"    #[serde(default, deserialize_with = "crate::literal_json::deserialize")]
    result: Option<JsonObject>,
"###,
    },
    Patch {
        target: "src/model/task.rs",
        before: r###"    error: Option<JsonObject>,
"###,
        after: r###"    #[serde(default, deserialize_with = "crate::literal_json::deserialize")]
    error: Option<JsonObject>,
"###,
    },
    Patch {
        target: "src/model/task.rs",
        before: r###"    input_requests: Option<InputRequests>,
"###,
        after: r###"    #[serde(default, deserialize_with = "crate::literal_json::deserialize_keys")]
    input_requests: Option<InputRequests>,
"###,
    },
    Patch {
        target: "src/model/mrtr.rs",
        before: r###"            input_requests: Option<InputRequests>,
"###,
        after: r###"            #[serde(default, deserialize_with = "crate::literal_json::deserialize_keys")]
            input_requests: Option<InputRequests>,
"###,
    },
    Patch {
        target: "src/model.rs",
        before: r###"    pub arguments: Option<std::collections::HashMap<String, String>>,
"###,
        after: r###"    #[serde(default, deserialize_with = "crate::literal_json::deserialize_keys")]
    pub arguments: Option<std::collections::HashMap<String, String>>,
"###,
    },
    Patch {
        target: "src/model/elicitation_schema.rs",
        before: r###"    properties: IndexMap<String, PrimitiveSchemaDefinition>,
"###,
        after: r###"    #[serde(deserialize_with = "crate::literal_json::deserialize_keys")]
    properties: IndexMap<String, PrimitiveSchemaDefinition>,
"###,
    },
    Patch {
        target: "src/model.rs",
        before: r###"    pub uri: String,
    /// Client responses to server-initiated input requests from a previous
    /// [`InputRequiredResult`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_responses: Option<InputResponses>,
"###,
        after: r###"    pub uri: String,
    /// Client responses to server-initiated input requests from a previous
    /// [`InputRequiredResult`].
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "crate::literal_json::deserialize")]
    pub input_responses: Option<InputResponses>,
"###,
    },
    Patch {
        target: "src/model.rs",
        before: r###"    pub arguments: Option<JsonObject>,
    /// Client responses to server-initiated input requests from a previous
    /// [`InputRequiredResult`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_responses: Option<InputResponses>,
"###,
        after: r###"    pub arguments: Option<JsonObject>,
    /// Client responses to server-initiated input requests from a previous
    /// [`InputRequiredResult`].
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "crate::literal_json::deserialize")]
    pub input_responses: Option<InputResponses>,
"###,
    },
    Patch {
        target: "src/model.rs",
        before: r###"    /// [`InputRequiredResult`]. Present only when retrying after an incomplete result.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_responses: Option<InputResponses>,
"###,
        after: r###"    /// [`InputRequiredResult`]. Present only when retrying after an incomplete result.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "crate::literal_json::deserialize")]
    pub input_responses: Option<InputResponses>,
"###,
    },
    Patch {
        target: "src/model.rs",
        before: r###"    pub input_responses: InputResponses,
"###,
        after: r###"    #[serde(deserialize_with = "crate::literal_json::deserialize")]
    pub input_responses: InputResponses,
"###,
    },
];
