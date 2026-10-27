//! One lazy SDK connection on one owner thread. An uncertain call is never replayed.
use crate::{
    client::{self, ClientTarget, McpClient},
    schema,
};
use rmcp::model::{CallToolRequestParams, CallToolResponse, CallToolResult, Tool};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    sync::{Mutex, MutexGuard, mpsc},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum McpError {
    Rejected(String),
    Terminal(String),
}
impl std::fmt::Display for McpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rejected(message) | Self::Terminal(message) => f.write_str(message),
        }
    }
}
impl std::error::Error for McpError {}
const CLOSED: &str = "Persistent MCP transport is closed; reopen explicitly";
const FAILED: &str = "Persistent MCP connection failed; no operation was retried";
const TIMED_OUT: &str = "Persistent MCP operation timed out; reconcile before reopening";
const CLEANUP: Duration = Duration::from_secs(5);
/// Bounds apply to the complete paginated discovery result.
#[derive(Clone, Copy, Debug)]
pub struct ToolListLimits {
    pub max_tools: usize,
    pub max_bytes: usize,
    pub max_pages: usize,
}
impl Default for ToolListLimits {
    fn default() -> Self {
        Self {
            max_tools: 4096,
            max_bytes: 16 * 1024 * 1024,
            max_pages: 128,
        }
    }
}
#[derive(Clone, Debug)]
pub struct ToolDiscovery {
    pub tools: Vec<Tool>,
    pub server_name: String,
    pub server_version: String,
    pub protocol_version: String,
    pub requests: u64,
    pub initialization_requests: u64,
    pub response_bytes: u64,
}
#[derive(Clone, Debug)]
pub struct ToolSchemaPins {
    pub input_sha256: String,
    pub output_sha256: Option<String>,
}
#[derive(Clone, Debug)]
pub struct PinnedToolResult {
    pub result: CallToolResult,
    pub discovery_requests: u64,
    pub discovery_bytes: u64,
}
enum Operation {
    Call {
        name: String,
        arguments: Value,
        pins: Option<ToolSchemaPins>,
        limits: ToolListLimits,
        max_output_bytes: usize,
    },
    Discover(ToolListLimits),
}
enum Answer {
    Call(PinnedToolResult),
    Discovery(ToolDiscovery),
}
enum Request {
    Execute {
        operation: Box<Operation>,
        deadline: Instant,
        reply: mpsc::SyncSender<Result<Answer, McpError>>,
    },
    Close,
}
struct State {
    sender: Option<tokio::sync::mpsc::UnboundedSender<Request>>,
    thread: Option<JoinHandle<()>>,
    starts: usize,
    closed: bool,
}
pub struct PersistentMcp {
    target: ClientTarget,
    timeout: Duration,
    state: Mutex<State>,
}
impl std::fmt::Debug for PersistentMcp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PersistentMcp")
            .field("timeout", &self.timeout)
            .finish_non_exhaustive()
    }
}
impl PersistentMcp {
    pub fn new(target: ClientTarget, timeout: Duration) -> Result<Self, String> {
        if timeout.is_zero() || Instant::now().checked_add(timeout).is_none() {
            return Err("Persistent MCP timeout must be positive and representable".into());
        }
        Ok(Self {
            target,
            timeout,
            state: Mutex::new(State {
                sender: None,
                thread: None,
                starts: 0,
                closed: false,
            }),
        })
    }
    fn lock_until(&self, deadline: Instant) -> Result<MutexGuard<'_, State>, McpError> {
        loop {
            match self.state.try_lock() {
                Ok(state) => return Ok(state),
                Err(std::sync::TryLockError::Poisoned(_)) => {
                    return Err(McpError::Terminal(FAILED.into()));
                }
                Err(std::sync::TryLockError::WouldBlock) => {
                    if Instant::now() >= deadline {
                        return Err(McpError::Rejected(
                            "Persistent MCP deadline expired before admission".into(),
                        ));
                    }
                    thread::park_timeout(Duration::from_millis(2));
                }
            }
        }
    }
    pub fn call(
        &self,
        name: &str,
        arguments: &Value,
        deadline: Instant,
    ) -> Result<CallToolResult, McpError> {
        let result = self.perform(
            Operation::Call {
                name: name.into(),
                arguments: arguments.clone(),
                pins: None,
                limits: ToolListLimits::default(),
                max_output_bytes: 16 * 1024 * 1024,
            },
            deadline,
        )?;
        match result {
            Answer::Call(result) => Ok(result.result),
            _ => Err(McpError::Terminal(FAILED.into())),
        }
    }
    /// A concrete operation with bounded interface discovery and response bytes.
    pub fn call_bounded(
        &self,
        name: &str,
        arguments: &Value,
        limits: ToolListLimits,
        max_output_bytes: usize,
        deadline: Instant,
    ) -> Result<CallToolResult, McpError> {
        match self.perform(
            Operation::Call {
                name: name.into(),
                arguments: arguments.clone(),
                pins: None,
                limits,
                max_output_bytes,
            },
            deadline,
        )? {
            Answer::Call(result) => Ok(result.result),
            _ => Err(McpError::Terminal(FAILED.into())),
        }
    }
    /// Refresh discovery and check both schema identities before any tool invocation.
    pub fn call_pinned(
        &self,
        name: &str,
        arguments: &Value,
        pins: &ToolSchemaPins,
        limits: ToolListLimits,
        deadline: Instant,
    ) -> Result<PinnedToolResult, McpError> {
        let result = self.perform(
            Operation::Call {
                name: name.into(),
                arguments: arguments.clone(),
                pins: Some(pins.clone()),
                max_output_bytes: limits.max_bytes,
                limits,
            },
            deadline,
        )?;
        match result {
            Answer::Call(result) => Ok(result),
            _ => Err(McpError::Terminal(FAILED.into())),
        }
    }
    pub fn discover(&self, deadline: Instant) -> Result<ToolDiscovery, McpError> {
        self.discover_bounded(ToolListLimits::default(), deadline)
    }
    pub fn discover_bounded(
        &self,
        limits: ToolListLimits,
        deadline: Instant,
    ) -> Result<ToolDiscovery, McpError> {
        match self.perform(Operation::Discover(limits), deadline)? {
            Answer::Discovery(result) => Ok(result),
            _ => Err(McpError::Terminal(FAILED.into())),
        }
    }
    fn perform(&self, operation: Operation, deadline: Instant) -> Result<Answer, McpError> {
        let limits = match &operation {
            Operation::Call {
                name,
                arguments,
                pins,
                limits,
                max_output_bytes,
            } => {
                if name.is_empty()
                    || name.len() > 256
                    || !arguments.is_object()
                    || !(1..=16 * 1024 * 1024).contains(max_output_bytes)
                {
                    return Err(McpError::Rejected(
                        "MCP call requires a bounded name and object arguments".into(),
                    ));
                }
                let bounds = eal_core::ExecutionLimits {
                    record_bytes: 16 * 1024 * 1024,
                    json_depth: 128,
                    ..Default::default()
                };
                eal_core::canonical::canonical_json_bytes(arguments, &bounds)
                    .map_err(McpError::Rejected)?;
                if let Some(pins) = pins {
                    let valid = |value: &str| {
                        value.len() == 64
                            && value
                                .bytes()
                                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                    };
                    if !valid(&pins.input_sha256)
                        || pins
                            .output_sha256
                            .as_deref()
                            .is_some_and(|value| !valid(value))
                    {
                        return Err(McpError::Rejected(
                            "MCP schema pins require exact SHA-256 identities".into(),
                        ));
                    }
                }
                limits
            }
            Operation::Discover(limits) => limits,
        };
        if limits.max_tools == 0
            || limits.max_tools > 4096
            || limits.max_bytes == 0
            || limits.max_bytes > 16 * 1024 * 1024
            || limits.max_pages == 0
            || limits.max_pages > 128
        {
            return Err(McpError::Rejected(
                "MCP discovery limits are outside supported bounds".into(),
            ));
        }
        let deadline = deadline.min(Instant::now().checked_add(self.timeout).ok_or_else(|| {
            McpError::Rejected("Persistent MCP deadline exceeds clock resources".into())
        })?);
        let mut state = self.lock_until(deadline)?;
        if state.closed {
            return Err(McpError::Terminal(CLOSED.into()));
        }
        if Instant::now() >= deadline {
            return Err(McpError::Rejected(
                "Persistent MCP deadline expired before admission".into(),
            ));
        }
        if state.sender.is_none() {
            let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
            let target = self.target.clone();
            let owner = thread::Builder::new()
                .name("eal-persistent-mcp".into())
                .spawn(move || owner(target, receiver))
                .map_err(|_| {
                    state.closed = true;
                    McpError::Terminal(FAILED.into())
                })?;
            state.sender = Some(sender);
            state.thread = Some(owner);
            state.starts += 1;
        }
        let (reply, result) = mpsc::sync_channel(1);
        let sent = state.sender.as_ref().unwrap().send(Request::Execute {
            operation: Box::new(operation),
            deadline,
            reply,
        });
        let answer = if sent.is_ok() {
            result
                .recv_timeout(
                    deadline
                        .saturating_duration_since(Instant::now())
                        .saturating_add(CLEANUP),
                )
                .unwrap_or_else(|_| Err(McpError::Terminal(TIMED_OUT.into())))
        } else {
            Err(McpError::Terminal(FAILED.into()))
        };
        if matches!(answer, Err(McpError::Terminal(_))) {
            Self::stop(&mut state);
        }
        answer
    }
    fn stop(state: &mut State) {
        state.closed = true;
        if let Some(sender) = state.sender.take() {
            let _ = sender.send(Request::Close);
        }
        if let Some(owner) = state.thread.take() {
            let _ = owner.join();
        }
    }
    pub fn close(&self) {
        if let Ok(mut state) = self.state.lock() {
            Self::stop(&mut state);
        }
    }
    /// A busy/unstarted owner conservatively reserves initialization capacity.
    pub fn initialization_may_be_needed(&self) -> bool {
        self.state
            .try_lock()
            .map_or(true, |state| state.starts == 0)
    }
    pub fn diagnostics(&self) -> Value {
        match self.state.lock() {
            Ok(state) => serde_json::json!({"server_starts":state.starts,"closed":state.closed}),
            Err(_) => serde_json::json!({"server_starts":null,"closed":true}),
        }
    }
}
impl Drop for PersistentMcp {
    fn drop(&mut self) {
        if let Ok(state) = self.state.get_mut() {
            Self::stop(state);
        }
    }
}

struct Session {
    target: Option<ClientTarget>,
    client: Option<McpClient>,
    tools: Vec<Tool>,
}
impl Session {
    async fn connect(&mut self, deadline: Instant) -> Result<(), McpError> {
        if self.client.is_none() {
            self.client = Some(
                client::connect(
                    self.target
                        .take()
                        .ok_or_else(|| McpError::Terminal(CLOSED.into()))?,
                    deadline,
                )
                .await
                .map_err(|_| {
                    McpError::Terminal(
                        if Instant::now() >= deadline {
                            TIMED_OUT
                        } else {
                            FAILED
                        }
                        .into(),
                    )
                })?,
            );
        }
        Ok(())
    }
    async fn discover(
        &mut self,
        limits: ToolListLimits,
        deadline: Instant,
    ) -> Result<ToolDiscovery, McpError> {
        let initialization_requests = u64::from(self.client.is_none());
        self.connect(deadline).await?;
        let client = self.client.as_ref().unwrap();
        let mut tools = Vec::new();
        let mut cursor = None;
        let mut seen = BTreeSet::new();
        let mut requests = 0u64;
        let mut response_bytes = 0usize;
        loop {
            if requests as usize >= limits.max_pages {
                return Err(McpError::Rejected(
                    "MCP discovery page budget exhausted".into(),
                ));
            }
            let mut params = rmcp::model::PaginatedRequestParams::default();
            params.cursor = cursor;
            let page = client
                .list_tools(Some(params))
                .await
                .map_err(|_| McpError::Terminal(FAILED.into()))?;
            requests += 1;
            response_bytes = response_bytes
                .checked_add(
                    serde_json::to_vec(&page)
                        .map_err(|_| {
                            McpError::Rejected("MCP discovery could not be encoded".into())
                        })?
                        .len(),
                )
                .ok_or_else(|| McpError::Rejected("MCP discovery byte budget exhausted".into()))?;
            if response_bytes > limits.max_bytes
                || tools.len().saturating_add(page.tools.len()) > limits.max_tools
            {
                return Err(McpError::Rejected(
                    "MCP discovery tool or byte budget exhausted".into(),
                ));
            }
            tools.extend(page.tools);
            match page.next_cursor {
                Some(next) => {
                    if !seen.insert(next.clone()) {
                        return Err(McpError::Rejected(
                            "MCP discovery repeated a pagination cursor".into(),
                        ));
                    }
                    cursor = Some(next);
                }
                None => break,
            }
        }
        let mut names = BTreeSet::new();
        if tools
            .iter()
            .any(|tool| !names.insert(tool.name.to_string()))
        {
            return Err(McpError::Rejected(
                "MCP discovery repeated a tool identity".into(),
            ));
        }
        let server = client.peer_info().ok_or_else(|| {
            McpError::Terminal("MCP initialization omitted server identity".into())
        })?;
        let implementation = server.server_info.as_ref();
        let answer = ToolDiscovery {
            tools: tools.clone(),
            server_name: implementation.map_or_else(String::new, |info| info.name.to_string()),
            server_version: implementation
                .map_or_else(String::new, |info| info.version.to_string()),
            protocol_version: server.protocol_version.to_string(),
            requests,
            initialization_requests,
            response_bytes: response_bytes as u64,
        };
        self.tools = tools;
        Ok(answer)
    }
    async fn execute(
        &mut self,
        operation: Operation,
        deadline: Instant,
    ) -> Result<Answer, McpError> {
        match operation {
            Operation::Discover(limits) => {
                self.discover(limits, deadline).await.map(Answer::Discovery)
            }
            Operation::Call {
                name,
                arguments,
                pins,
                limits,
                max_output_bytes,
            } => {
                let discovery = if self.client.is_none() || pins.is_some() {
                    Some(self.discover(limits, deadline).await?)
                } else {
                    None
                };
                let entry = self
                    .tools
                    .iter()
                    .find(|tool| tool.name == name)
                    .ok_or_else(|| {
                        McpError::Rejected(format!("MCP server does not offer {name}"))
                    })?;
                if let Some(pins) = pins {
                    let input = eal_collector_contracts::capability::schema_identity(
                        &Value::Object((*entry.input_schema).clone()),
                    )
                    .map_err(McpError::Rejected)?;
                    let output = entry
                        .output_schema
                        .as_ref()
                        .map(|schema| {
                            eal_collector_contracts::capability::schema_identity(&Value::Object(
                                (**schema).clone(),
                            ))
                        })
                        .transpose()
                        .map_err(McpError::Rejected)?;
                    if input != pins.input_sha256 || output != pins.output_sha256 {
                        return Err(McpError::Rejected(
                            "MCP tool schema identity differs from its pinned binding".into(),
                        ));
                    }
                }
                schema::validate_schema(&arguments, &Value::Object((*entry.input_schema).clone()))
                    .map_err(|error| {
                        McpError::Rejected(format!(
                            "Discovered input schema rejected request: {error}"
                        ))
                    })?;
                let params = CallToolRequestParams::new(name)
                    .with_arguments(arguments.as_object().unwrap().clone());
                let result =
                    match self
                        .client
                        .as_ref()
                        .unwrap()
                        .call_tool_once(params)
                        .await
                        .map_err(|_| McpError::Terminal(FAILED.into()))?
                    {
                        CallToolResponse::Complete(result) => result,
                        _ => return Err(McpError::Terminal(
                            "Persistent MCP returned an unsupported interactive or deferred result"
                                .into(),
                        )),
                    };
                if serde_json::to_vec(&result)
                    .map_err(|_| McpError::Rejected("Cannot encode MCP result".into()))?
                    .len()
                    > max_output_bytes
                {
                    return Err(McpError::Rejected(
                        "MCP result exceeds output allowance".into(),
                    ));
                }
                if !result.is_error.unwrap_or(false)
                    && let Some(schema) = &entry.output_schema
                {
                    let content = result.structured_content.as_ref().ok_or_else(|| {
                        McpError::Rejected("MCP result omitted declared structured output".into())
                    })?;
                    schema::validate_schema(content, &Value::Object((**schema).clone())).map_err(
                        |_| McpError::Rejected("MCP result violates declared output schema".into()),
                    )?;
                }
                Ok(Answer::Call(PinnedToolResult {
                    result,
                    discovery_requests: discovery.as_ref().map_or(0, |d| d.requests),
                    discovery_bytes: discovery.as_ref().map_or(0, |d| d.response_bytes),
                }))
            }
        }
    }
}
fn owner(target: ClientTarget, mut requests: tokio::sync::mpsc::UnboundedReceiver<Request>) {
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(_) => return,
    };
    runtime.block_on(async {
        let mut session = Session {
            target: Some(target),
            client: None,
            tools: vec![],
        };
        while let Some(request) = requests.recv().await {
            let Request::Execute {
                operation,
                deadline,
                reply,
            } = request
            else {
                break;
            };
            let answer = tokio::time::timeout_at(
                tokio::time::Instant::from_std(deadline),
                session.execute(*operation, deadline),
            )
            .await
            .unwrap_or_else(|_| Err(McpError::Terminal(TIMED_OUT.into())));
            let terminal = matches!(answer, Err(McpError::Terminal(_)));
            if terminal {
                shutdown(&mut session.client).await;
            }
            let _ = reply.send(answer);
            if terminal {
                break;
            }
        }
        shutdown(&mut session.client).await;
    });
}
async fn shutdown(client: &mut Option<McpClient>) {
    if let Some(mut client) = client.take() {
        client.cancellation_token().cancel();
        let _ = client.close_with_timeout(Duration::from_secs(4)).await;
    }
}
