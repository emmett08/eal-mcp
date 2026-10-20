//! Native persistent mailbox MCP bridge. Actor identity remains a trusted local host assertion.
use crate::{
    bus::{self, Mailbox, MailboxLimits},
    persistent_mcp::{McpError, PersistentMcp},
};
use eal_mcp::client::{ClientTarget, StdioTarget};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
pub const OPERATIONS: &[&str] = &[
    "mas_create_run",
    "mas_receive",
    "mas_ack",
    "mas_status",
    "mas_encode",
    "mas_validate",
    "mas_reserve_call",
    "mas_finish_call",
    "mas_runtime_info",
    "mas_send_envelope",
];
pub struct LabMcpTransport {
    runtime: PathBuf,
    database: PathBuf,
    session: PersistentMcp,
    operation_timeout: Duration,
}
impl LabMcpTransport {
    pub fn new(runtime: impl AsRef<Path>, database: impl AsRef<Path>) -> Result<Self, String> {
        Self::with_options(
            runtime,
            database,
            eal_mcp::application::environment()?,
            Duration::from_secs(30),
        )
    }
    pub fn with_options(
        runtime: impl AsRef<Path>,
        database: impl AsRef<Path>,
        environment: BTreeMap<String, String>,
        operation_timeout: Duration,
    ) -> Result<Self, String> {
        let runtime = runtime
            .as_ref()
            .canonicalize()
            .map_err(|_| "Native MAS runtime directory is unavailable")?;
        if !runtime.is_dir() || runtime.to_str().is_none() {
            return Err("Native MAS runtime must be a UTF-8 directory".into());
        }
        let binary = runtime.join(format!("eal-mas-server{}", std::env::consts::EXE_SUFFIX));
        if !binary.is_file() {
            return Err("Install native eal-mas-server in the configured runtime directory".into());
        }
        let database = if database.as_ref().is_absolute() {
            database.as_ref().to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|e| e.to_string())?
                .join(database)
        };
        let database_text = database
            .to_str()
            .ok_or("Mailbox database path must be UTF-8")?
            .to_owned();
        let environment = environment
            .into_iter()
            .filter(|(key, _)| {
                let upper = key.to_uppercase();
                !key.starts_with("EAL_MCP_")
                    && key != "EAL_MAS_CODEC_MODE"
                    && !["API_KEY", "TOKEN", "SECRET", "PASSWORD", "CREDENTIAL"]
                        .iter()
                        .any(|marker| upper.contains(marker))
            })
            .collect();
        let session = PersistentMcp::new(
            ClientTarget::Stdio(StdioTarget {
                command: binary,
                arguments: vec!["--database".into(), database_text],
                environment,
                cwd: Some(runtime.clone()),
            }),
            operation_timeout,
        )?;
        Ok(Self {
            runtime,
            database,
            session,
            operation_timeout,
        })
    }
    pub fn runtime(&self) -> &Path {
        &self.runtime
    }
    pub fn database(&self) -> &Path {
        &self.database
    }
    pub fn close(&self) {
        self.session.close();
    }
    pub fn diagnostics(&self) -> Value {
        let mut result = self.session.diagnostics();
        result["codec_mode"] = json!("in_process");
        result["admission_mode"] = json!("rust");
        result
    }
    pub fn mcp_call(&self, name: &str, arguments: &Value) -> Result<Value, String> {
        let deadline = Instant::now()
            .checked_add(self.operation_timeout)
            .ok_or("Mailbox MCP deadline exceeds clock resources")?;
        self.mcp_call_until(name, arguments, deadline)
    }
    pub fn mcp_call_until(
        &self,
        name: &str,
        arguments: &Value,
        deadline: Instant,
    ) -> Result<Value, String> {
        if !OPERATIONS.contains(&name) {
            return Err("Operation is not exposed by this Lab MCP transport".into());
        }
        let result = self
            .session
            .call(name, arguments, deadline)
            .map_err(|error| error.to_string())?;
        match result_value(&result) {
            Ok(value) => Ok(value),
            Err(McpError::Rejected(error)) => Err(error),
            Err(McpError::Terminal(error)) => {
                self.close();
                Err(error)
            }
        }
    }
    pub fn encode(&self, envelope: &Value) -> Result<String, String> {
        check_envelope(envelope)?;
        let value = self.mcp_call("mas_encode", &json!({"envelope":envelope}))?;
        value
            .as_str()
            .filter(|s| s.len() <= 65536)
            .map(str::to_owned)
            .ok_or_else(|| "MAS mailbox codec returned an invalid encoded message".into())
    }
    pub fn validate_wire(&self, text: &str) -> Result<Value, String> {
        self.mcp_call("mas_validate", &json!({"text":text}))
    }
}
fn result_value(result: &rmcp::model::CallToolResult) -> Result<Value, McpError> {
    let invalid =
        || McpError::Terminal("MAS mailbox MCP returned an invalid response envelope".into());
    if result.is_error.unwrap_or(false) {
        return Err(McpError::Terminal("MAS mailbox MCP tool failed".into()));
    }
    let value = if let Some(value) = &result.structured_content {
        value.clone()
    } else {
        let content = serde_json::to_value(&result.content).map_err(|_| invalid())?;
        let blocks = content.as_array().ok_or_else(invalid)?;
        if blocks.len() != 1 || blocks[0]["type"] != "text" {
            return Err(invalid());
        }
        let text = blocks[0]["text"].as_str().ok_or_else(invalid)?;
        eal_core::json::parse(
            text,
            &eal_core::ExecutionLimits {
                json_depth: 128,
                record_bytes: 16 * 1024 * 1024,
                ..Default::default()
            },
        )
        .map_err(|_| invalid())?
    };
    let object = value.as_object().ok_or_else(invalid)?;
    let success = value["ok"].as_bool().ok_or_else(invalid)?;
    if success {
        if object.len() != 2 || !object.contains_key("result") {
            return Err(invalid());
        }
        return Ok(value["result"].clone());
    }
    if object.len() != 2 || !object.contains_key("error") || !value["error"].is_object() {
        return Err(invalid());
    }
    let code = value["error"]["code"].as_str().ok_or_else(invalid)?;
    let message = value["error"]["message"].as_str().ok_or_else(invalid)?;
    if code == "CODEC_IO" {
        return Err(McpError::Terminal(
            "Persistent Rust codec connection failed; no operation was retried".into(),
        ));
    }
    Err(McpError::Rejected(format!("{code}: {message}")))
}
fn check_envelope(value: &Value) -> Result<(), String> {
    let limits = eal_core::ExecutionLimits {
        record_bytes: 65536 + 256,
        json_depth: 128,
        ..Default::default()
    };
    eal_core::canonical::canonical_json_bytes(value, &limits)
        .map(|_| ())
        .map_err(|_| "Lab codec envelope exceeds its bounded input limit".into())
}
impl Mailbox for LabMcpTransport {
    fn close(&self) -> Result<(), String> {
        LabMcpTransport::close(self);
        Ok(())
    }
    fn create(
        &self,
        run_id: &str,
        participants: &[String],
        limits: MailboxLimits,
    ) -> Result<Value, String> {
        self.mcp_call("mas_create_run",&json!({"run_id":run_id,"agents":participants,"max_messages":limits.max_messages,"max_bytes":limits.max_bytes,"max_calls":limits.max_calls}))
    }
    fn send(&self, run_id: &str, payload: &Value, actor: &str) -> Result<Value, String> {
        let envelope = bus::envelope(run_id, payload)?;
        check_envelope(&envelope)?;
        self.mcp_call(
            "mas_send_envelope",
            &json!({"envelope":envelope,"actor":actor}),
        )
    }
    fn receive(&self, run_id: &str, recipient: &str, limit: usize) -> Result<Value, String> {
        self.mcp_call(
            "mas_receive",
            &json!({"run_id":run_id,"recipient":recipient,"limit":limit}),
        )
    }
    fn acknowledge(
        &self,
        run_id: &str,
        message_id: &str,
        recipient: &str,
    ) -> Result<Value, String> {
        self.mcp_call(
            "mas_ack",
            &json!({"run_id":run_id,"message_id":message_id,"recipient":recipient}),
        )
    }
    fn reserve(&self, run_id: &str, actor: &str, call_id: &str) -> Result<Value, String> {
        self.mcp_call(
            "mas_reserve_call",
            &json!({"run_id":run_id,"actor":actor,"call_id":call_id}),
        )
    }
    fn finish(
        &self,
        run_id: &str,
        actor: &str,
        call_id: &str,
        evidence: &Value,
    ) -> Result<Value, String> {
        self.mcp_call(
            "mas_finish_call",
            &json!({"run_id":run_id,"actor":actor,"call_id":call_id,"evidence":evidence}),
        )
    }
    fn status(&self, run_id: &str) -> Result<Value, String> {
        self.mcp_call("mas_status", &json!({"run_id":run_id}))
    }
}
