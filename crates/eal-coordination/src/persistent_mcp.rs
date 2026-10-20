//! One lazy SDK connection on one owner thread. An uncertain call is never replayed.
use eal_mcp::{
    client::{self, ClientTarget, McpClient},
    guard,
};
use rmcp::model::{CallToolRequestParams, CallToolResponse, CallToolResult, Tool};
use serde_json::Value;
use std::{
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
enum Request {
    Call {
        name: String,
        arguments: Value,
        deadline: Instant,
        reply: mpsc::SyncSender<Result<CallToolResult, McpError>>,
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
        if name.is_empty() || name.len() > 256 || !arguments.is_object() {
            return Err(McpError::Rejected(
                "MCP call requires a bounded name and object arguments".into(),
            ));
        }
        let limits = eal_core::ExecutionLimits {
            record_bytes: 16 * 1024 * 1024,
            json_depth: 128,
            ..Default::default()
        };
        eal_core::canonical::canonical_json_bytes(arguments, &limits)
            .map_err(McpError::Rejected)?;
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
        let sent = state.sender.as_ref().unwrap().send(Request::Call {
            name: name.into(),
            arguments: arguments.clone(),
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

fn owner(target: ClientTarget, mut requests: tokio::sync::mpsc::UnboundedReceiver<Request>) {
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(_) => return,
    };
    runtime.block_on(async {
        let mut target = Some(target);
        let mut client: Option<McpClient> = None;
        let mut tools: Vec<Tool> = vec![];
        while let Some(request) = requests.recv().await {
            let Request::Call {
                name,
                arguments,
                deadline,
                reply,
            } = request
            else {
                break;
            };
            let work = async {
                if client.is_none() {
                    let connection = client::connect(
                        target
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
                    })?;
                    client = Some(connection);
                    tools = client
                        .as_ref()
                        .unwrap()
                        .list_all_tools()
                        .await
                        .map_err(|_| McpError::Terminal(FAILED.into()))?;
                }
                let entry = tools.iter().find(|tool| tool.name == name).ok_or_else(|| {
                    McpError::Rejected(format!("MCP server does not offer {name}"))
                })?;
                guard::validate_schema(&arguments, &Value::Object((*entry.input_schema).clone()))
                    .map_err(|error| {
                    McpError::Rejected(format!("Discovered input schema rejected request: {error}"))
                })?;
                let params = CallToolRequestParams::new(name)
                    .with_arguments(arguments.as_object().unwrap().clone());
                match client
                    .as_ref()
                    .unwrap()
                    .call_tool_once(params)
                    .await
                    .map_err(|_| McpError::Terminal(FAILED.into()))?
                {
                    CallToolResponse::Complete(result) => Ok(result),
                    _ => Err(McpError::Terminal(
                        "Persistent MCP returned an unsupported interactive or deferred result"
                            .into(),
                    )),
                }
            };
            let answer = tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), work)
                .await
                .unwrap_or_else(|_| Err(McpError::Terminal(TIMED_OUT.into())));
            let terminal = matches!(answer, Err(McpError::Terminal(_)));
            if terminal {
                shutdown(&mut client).await;
            }
            let _ = reply.send(answer);
            if terminal {
                break;
            }
        }
        shutdown(&mut client).await;
    });
}
async fn shutdown(client: &mut Option<McpClient>) {
    if let Some(mut client) = client.take() {
        client.cancellation_token().cancel();
        let _ = client.close_with_timeout(Duration::from_secs(4)).await;
    }
}
