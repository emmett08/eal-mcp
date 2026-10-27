//! One-shot official-SDK clients. Targets are trusted operator configuration.
use crate::authentication::{loopback_host, read_bearer_token};
pub use crate::supervised::StdioSupervision;
use rmcp::{
    RoleClient, ServiceExt,
    model::{
        CallToolRequestParams, CallToolResponse, ClientCapabilities, ClientConfig, Implementation,
        ProtocolVersion,
    },
    service::RunningService,
    transport::{
        StreamableHttpClientTransport, streamable_http_client::StreamableHttpClientTransportConfig,
    },
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf, time::Duration};

#[derive(Clone)]
pub struct HttpTarget {
    url: String,
    token: Option<String>,
    client: reqwest::Client,
}
impl std::fmt::Debug for HttpTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpTarget")
            .field("url", &self.url)
            .finish_non_exhaustive()
    }
}
impl HttpTarget {
    pub fn url(&self) -> &str {
        &self.url
    }
}
#[derive(Clone)]
pub struct StdioTarget {
    pub command: PathBuf,
    pub arguments: Vec<String>,
    pub environment: BTreeMap<String, String>,
    pub cwd: Option<PathBuf>,
}
impl std::fmt::Debug for StdioTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StdioTarget")
            .field("command", &self.command)
            .field("cwd", &self.cwd)
            .finish_non_exhaustive()
    }
}
#[derive(Clone, Debug)]
pub enum ClientTarget {
    Stdio(StdioTarget),
    SupervisedStdio(StdioTarget, StdioSupervision),
    Http(HttpTarget),
}

/// Retain the supplied authority/path; URL credentials and nonempty query/fragment are forbidden.
pub fn validate_http_url(value: &str) -> Result<String, String> {
    if value.is_empty() || value.chars().any(|c| c.is_whitespace() || c < ' ') {
        return Err("HTTP URL must be a non-empty URL without whitespace".into());
    }
    let (without_fragment, fragment) = value
        .split_once('#')
        .map(|(a, b)| (a, Some(b)))
        .unwrap_or((value, None));
    let (base, query) = without_fragment
        .split_once('?')
        .map(|(a, b)| (a, Some(b)))
        .unwrap_or((without_fragment, None));
    let (scheme, rest) = base
        .split_once("://")
        .ok_or("HTTP URL must use http or https with a valid hostname and port")?;
    let authority = rest.split('/').next().unwrap_or("");
    if authority.contains('@')
        || query.is_some_and(|s| !s.is_empty())
        || fragment.is_some_and(|s| !s.is_empty())
    {
        return Err("HTTP URL must not contain credentials, a query or a fragment".into());
    }
    let parsed = url::Url::parse(base).map_err(|_| "HTTP URL has an invalid host or port")?;
    if !["http", "https"].contains(&scheme.to_ascii_lowercase().as_str())
        || parsed.host_str().is_none()
        || parsed.port() == Some(0)
        || authority.is_empty()
    {
        return Err("HTTP URL must use http or https with a valid hostname and port".into());
    }
    Ok(format!("{}://{}", scheme.to_ascii_lowercase(), rest))
}
fn raw_hostname(url: &str) -> &str {
    let authority = url.split_once("://").unwrap().1.split('/').next().unwrap();
    if let Some(rest) = authority.strip_prefix('[') {
        rest.split(']').next().unwrap_or("")
    } else {
        authority.split(':').next().unwrap_or("")
    }
}
pub fn create_http_target(
    url: &str,
    token_env: Option<&str>,
    environ: &BTreeMap<String, String>,
) -> Result<ClientTarget, String> {
    create_http_target_with_client(url, token_env, environ, reqwest::Client::new())
}
/// Embedders may supply their own HTTP client, for example to control proxy policy.
pub fn create_http_target_with_client(
    url: &str,
    token_env: Option<&str>,
    environ: &BTreeMap<String, String>,
    client: reqwest::Client,
) -> Result<ClientTarget, String> {
    let url = validate_http_url(url)?;
    if !loopback_host(raw_hostname(&url)) && token_env.is_none() {
        return Err("HTTP connections beyond loopback require --token-env authentication".into());
    }
    let token = read_bearer_token(token_env, environ)?;
    Ok(ClientTarget::Http(HttpTarget { url, token, client }))
}
fn configuration() -> ClientConfig {
    ClientConfig::new(
        ClientCapabilities::default(),
        Implementation::new("eal-host", env!("CARGO_PKG_VERSION")),
    )
    .with_protocol_version(ProtocolVersion::V_2025_11_25)
}
pub struct McpClient {
    service: RunningService<RoleClient, ClientConfig>,
    #[cfg(unix)]
    process: Option<eal_process::duplex::DuplexOwner>,
}
impl std::ops::Deref for McpClient {
    type Target = RunningService<RoleClient, ClientConfig>;
    fn deref(&self) -> &Self::Target {
        &self.service
    }
}
impl McpClient {
    pub async fn close(&mut self) -> Result<(), String> {
        self.close_with_timeout(Duration::from_millis(250)).await
    }
    pub async fn close_with_timeout(&mut self, timeout: Duration) -> Result<(), String> {
        self.service.cancellation_token().cancel();
        #[cfg(unix)]
        let process_result = self
            .process
            .as_mut()
            .map(|p| p.close())
            .transpose()
            .map_err(|e| e.to_string());
        let result = self
            .service
            .close_with_timeout(timeout)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string());
        #[cfg(unix)]
        process_result?;
        result
    }
}
impl Drop for McpClient {
    fn drop(&mut self) {
        self.service.cancellation_token().cancel();
    }
}
/// Explicit runtime authority and descriptor retention; no tool configuration is inferred.
pub fn supervised_stdio(
    target: StdioTarget,
    supervisor: &eal_process::Supervisor,
    lease: Option<&dyn eal_process::Retention>,
    max_input_bytes: usize,
    max_output_bytes: usize,
    max_session_duration: Duration,
) -> Result<ClientTarget, String> {
    Ok(ClientTarget::SupervisedStdio(
        target,
        StdioSupervision::new(
            supervisor,
            lease,
            max_input_bytes,
            max_output_bytes,
            max_session_duration,
        )?,
    ))
}
type Client = McpClient;
async fn invoke(client: &Client, tool: &str, arguments: &Value) -> Result<Value, String> {
    let tools = client.list_all_tools().await.map_err(|e| e.to_string())?;
    let entry = tools
        .iter()
        .find(|entry| entry.name == tool)
        .ok_or_else(|| format!("MCP server does not offer {tool}"))?;
    let schema = Value::Object((*entry.input_schema).clone());
    crate::schema::validate_schema(arguments, &schema)
        .map_err(|e| format!("Discovered input schema rejected request: {e}"))?;
    let params = CallToolRequestParams::new(tool.to_owned()).with_arguments(
        arguments
            .as_object()
            .ok_or("Tool arguments must be an object")?
            .clone(),
    );
    let result = match client
        .call_tool_once(params)
        .await
        .map_err(|e| e.to_string())?
    {
        CallToolResponse::Complete(result) => result,
        _ => {
            return Err(
                "MCP server returned an unsupported interactive or deferred tool result".into(),
            );
        }
    };
    let protocol = client
        .peer_info()
        .ok_or("MCP initialization did not return server information")?
        .protocol_version
        .to_string();
    Ok(
        json!({"operation":tool.strip_prefix("eal_").unwrap_or(tool),"protocol_version":protocol,"is_error":result.is_error.unwrap_or(false),"result":result.structured_content,"content":result.content}),
    )
}
/// A single deadline covers connection, initialization, discovery, invocation and close.
pub async fn dispatch_tool(
    tool: &str,
    arguments: &Value,
    target: ClientTarget,
    timeout_seconds: f64,
) -> Result<Value, String> {
    if !timeout_seconds.is_finite() || timeout_seconds <= 0.0 {
        return Err("Host timeout must be positive and finite".into());
    }
    let duration = Duration::try_from_secs_f64(timeout_seconds)
        .map_err(|_| "Host timeout is outside supported bounds")?;
    let deadline = std::time::Instant::now()
        .checked_add(duration)
        .ok_or("Host timeout is outside supported bounds")?;
    let mut running: Option<Client> = None;
    let outcome = tokio::time::timeout(duration, async {
        let client = connect(target, deadline).await?;
        running = Some(client);
        let client = running.as_mut().unwrap();
        let result = invoke(client, tool, arguments).await;
        client.close().await.map_err(|e| e.to_string())?;
        result
    })
    .await;
    match outcome {
        Ok(result) => result,
        Err(_) => {
            if let Some(client) = running.as_mut() {
                client.cancellation_token().cancel();
                let _ = client.close_with_timeout(Duration::from_millis(250)).await;
            }
            Err("MCP session exceeded its configured deadline".into())
        }
    }
}

/// Connect once without invocation or replay. The absolute deadline includes initialization.
pub async fn connect(
    target: ClientTarget,
    deadline: std::time::Instant,
) -> Result<McpClient, String> {
    let duration = deadline.saturating_duration_since(std::time::Instant::now());
    if duration.is_zero() {
        return Err("MCP session exceeded its configured deadline".into());
    }
    tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), async {
        Ok::<McpClient, String>(match target {
            ClientTarget::Stdio(target) => {
                connect_stdio(target, StdioSupervision::installed()?).await?
            }
            ClientTarget::SupervisedStdio(target, supervision) => {
                connect_stdio(target, supervision).await?
            }
            ClientTarget::Http(target) => {
                let mut config = StreamableHttpClientTransportConfig::with_uri(target.url)
                    .max_concurrent_requests(1)
                    .reinit_on_expired_session(false)
                    .max_sse_event_size(16 * 1024 * 1024)
                    .control_request_timeout(duration.min(Duration::from_secs(5)));
                if let Some(token) = target.token {
                    config = config.auth_header(token);
                }
                McpClient {
                    service: configuration()
                        .serve(StreamableHttpClientTransport::with_client(
                            target.client,
                            config,
                        ))
                        .await
                        .map_err(|e| e.to_string())?,
                    #[cfg(unix)]
                    process: None,
                }
            }
        })
    })
    .await
    .map_err(|_| "MCP session exceeded its configured deadline".to_owned())?
}

async fn connect_stdio(
    target: StdioTarget,
    supervision: StdioSupervision,
) -> Result<McpClient, String> {
    #[cfg(unix)]
    {
        let eal_process::duplex::DuplexProcess { stream, owner } = supervision.spawn(target)?;
        let stream = tokio::net::UnixStream::from_std(stream).map_err(|e| e.to_string())?;
        let (read, write) = stream.into_split();
        let service = configuration()
            .serve((read, write))
            .await
            .map_err(|e| e.to_string())?;
        Ok(McpClient {
            service,
            process: Some(owner),
        })
    }
    #[cfg(not(unix))]
    {
        let _ = (target, supervision);
        Err("Supervised MCP stdio currently requires a POSIX host".into())
    }
}
