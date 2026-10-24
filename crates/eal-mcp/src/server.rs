//! MCP handlers using the official SDK and the frozen public operation contracts.
use crate::{
    guard::{self, Admission},
    operations::Operations,
};
use eal_runtime::service::ReasoningService;
use rmcp::{
    RoleServer, ServerHandler,
    model::{
        CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
        ListToolsResult, PaginatedRequestParams, ProtocolVersion, ServerCapabilities, ServerConfig,
        Tool,
    },
    service::RequestContext,
};
use serde_json::Value;
use std::{borrow::Cow, sync::Arc};
use tokio_util::sync::CancellationToken;

pub const INSTRUCTIONS: &str = "Validate explicit engineering arguments, collect configured observations, reason over their declared scope, and explain results. Support is relative to declared inference rationales, not a proof of prose truth.";

#[derive(Clone, Debug)]
pub struct ServerOptions {
    pub known_entries: Vec<String>,
    pub exposure: Option<String>,
    pub max_in_flight: usize,
}
impl Default for ServerOptions {
    fn default() -> Self {
        Self {
            known_entries: Vec::new(),
            exposure: None,
            max_in_flight: 1,
        }
    }
}

#[derive(Clone)]
pub struct Server {
    operations: Arc<Operations>,
    admission: Admission,
}
impl Server {
    pub fn new(service: Arc<ReasoningService>, options: ServerOptions) -> Result<Self, String> {
        // Admission configuration is rejected before catalogue construction.
        let admission = Admission::new(options.max_in_flight)?;
        let operations = Arc::new(Operations::new(
            service,
            options.known_entries,
            options.exposure.as_deref(),
        )?);
        Ok(Self {
            operations,
            admission,
        })
    }
    pub fn tools(&self) -> &[Tool] {
        &self.operations.tools
    }

    /// The same validated dispatcher is available to embedding applications.
    /// Once blocking work starts, it owns the permit and any acquisition lease
    /// until completion, even if the request future or connection is dropped.
    pub async fn call(
        &self,
        name: &str,
        arguments: Value,
        cancellation: CancellationToken,
    ) -> Result<Value, String> {
        let request = match self.operations.decode(name, &arguments) {
            Ok(request) => request,
            Err(error) => {
                guard::discard(arguments);
                return Err(error);
            }
        };
        let permit = self.admission.acquire(&cancellation).await?;
        if cancellation.is_cancelled() {
            return Err("Tool request cancelled before execution".into());
        }
        let operations = Arc::clone(&self.operations);
        let name = name.to_owned();
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            operations
                .dispatch(request)
                .map_err(|error| format!("Error calling tool '{name}': {error}"))
        })
        .await
        .map_err(|error| format!("Tool execution failed: {error}"))?
    }
}
impl ServerHandler for Server {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_protocol_version(ProtocolVersion::V_2025_11_25)
            .with_server_info(Implementation::new(
                "EAL engineering reasoning",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(INSTRUCTIONS)
    }
    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        Cow::Borrowed(ProtocolVersion::known_up_to(&ProtocolVersion::V_2025_11_25))
    }
    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tools().iter().find(|tool| tool.name == name).cloned()
    }
    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, rmcp::ErrorData> {
        published_protocol(&context)?;
        Ok(ListToolsResult::with_all_items(self.tools().to_vec()))
    }
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, rmcp::ErrorData> {
        published_protocol(&context)?;
        let arguments = Value::Object(request.arguments.unwrap_or_default());
        let result = match self.call(&request.name, arguments, context.ct).await {
            Ok(value) => CallToolResult::structured(value),
            Err(message) => CallToolResult::error(vec![ContentBlock::text(message)]),
        };
        Ok(result.into())
    }
}

fn published_protocol(context: &RequestContext<RoleServer>) -> Result<(), rmcp::ErrorData> {
    if let Some(version) = context.protocol_version()
        && !ProtocolVersion::known_up_to(&ProtocolVersion::V_2025_11_25).contains(&version)
    {
        return Err(rmcp::ErrorData::invalid_request(
            "Unsupported MCP protocol version; this server supports published revisions through 2025-11-25",
            None,
        ));
    }
    Ok(())
}
