//! Official-SDK stdio tools for the native pinned Lab mailbox and nom codec.
use crate::{
    lab_store::{LabError, LabStore},
    protocol,
};
use rmcp::{
    RoleServer, ServerHandler,
    model::{
        CallToolRequestParams, CallToolResponse, CallToolResult, Implementation, ListToolsResult,
        PaginatedRequestParams, ProtocolVersion, ServerCapabilities, ServerConfig, Tool,
    },
    service::RequestContext,
};
use serde_json::{Map, Value, json};
use std::{
    borrow::Cow,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

pub const OPERATION_NAMES: [&str; 10] = [
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

#[derive(Default)]
struct Metrics {
    encode: AtomicU64,
    parse: AtomicU64,
    admit: AtomicU64,
    codec_ns: AtomicU64,
}
enum StoreSource {
    Ready(LabStore),
    Lazy {
        path: std::path::PathBuf,
        opened: Mutex<Option<LabStore>>,
    },
}
struct Inner {
    store: StoreSource,
    tools: Vec<Tool>,
    metrics: Metrics,
}
#[derive(Clone)]
pub struct LabServer(Arc<Inner>);
impl LabServer {
    pub fn new(store: LabStore) -> Result<Self, String> {
        Ok(Self(Arc::new(Inner {
            store: StoreSource::Ready(store),
            tools: tools(),
            metrics: Metrics::default(),
        })))
    }
    /// Codec and discovery operations must not initialise mailbox state.
    pub fn for_database(path: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let path = path.as_ref();
        if !path.is_absolute() {
            return Err("The state database path must be absolute.".into());
        }
        Ok(Self(Arc::new(Inner {
            store: StoreSource::Lazy {
                path: path.to_owned(),
                opened: Mutex::new(None),
            },
            tools: tools(),
            metrics: Metrics::default(),
        })))
    }
    fn store(&self) -> Result<LabStore, LabError> {
        match &self.0.store {
            StoreSource::Ready(store) => Ok(store.clone()),
            StoreSource::Lazy { path, opened } => {
                let mut store = opened.lock().map_err(|_| LabError {
                    code: "state_unavailable".into(),
                    message: "The state database could not complete the operation.".into(),
                })?;
                if store.is_none() {
                    *store = Some(LabStore::new(path)?);
                }
                Ok(store.as_ref().expect("opened store").clone())
            }
        }
    }
    pub fn tools(&self) -> &[Tool] {
        &self.0.tools
    }
    /// Ordinary admission and mailbox failures are protocol data, not transport errors.
    pub fn call(&self, name: &str, arguments: &Value) -> Value {
        match self.dispatch(name, arguments) {
            Ok(value) => json!({"ok":true,"result":value}),
            Err(error) => json!({"ok":false,"error":error}),
        }
    }
    fn dispatch(&self, name: &str, arguments: &Value) -> Result<Value, Value> {
        let Some(tool) = self.tools().iter().find(|tool| tool.name == name) else {
            return Err(LabError::rejected("Unknown operation.").to_value());
        };
        validate_arguments(arguments, &tool.input_schema).map_err(|e| e.to_value())?;
        let string = |key: &str| arguments[key].as_str().expect("validated string");
        let result = match name {
            "mas_create_run" => self.store().and_then(|store| store.create_run(arguments)),
            "mas_receive" => self.store().and_then(|store| {
                store.receive(
                    string("run_id"),
                    string("recipient"),
                    arguments.get("limit").unwrap_or(&json!(10)),
                )
            }),
            "mas_ack" => self.store().and_then(|store| {
                store.ack(string("run_id"), string("message_id"), string("recipient"))
            }),
            "mas_status" => self
                .store()
                .and_then(|store| store.inspect(string("run_id"))),
            "mas_reserve_call" => self.store().and_then(|store| {
                store.reserve_call(string("run_id"), string("actor"), string("call_id"))
            }),
            "mas_finish_call" => self.store().and_then(|store| {
                store.finish_call(
                    string("run_id"),
                    string("actor"),
                    string("call_id"),
                    &arguments["evidence"],
                )
            }),
            "mas_runtime_info" => {
                return Ok(
                    json!({"server_pid":std::process::id(),"codec_pid":null,"codec_starts":0,"codec_mode":"in_process","admission_mode":"rust","encode_calls":self.0.metrics.encode.load(Ordering::Relaxed),"parse_calls":self.0.metrics.parse.load(Ordering::Relaxed),"admit_calls":self.0.metrics.admit.load(Ordering::Relaxed),"codec_ns":self.0.metrics.codec_ns.load(Ordering::Relaxed)}),
                );
            }
            "mas_encode" => {
                self.0.metrics.encode.fetch_add(1, Ordering::Relaxed);
                let began = Instant::now();
                let admitted = protocol::admit_envelope(
                    &arguments["envelope"],
                    protocol::MAX_BYTES,
                    protocol::MAX_DEPTH,
                );
                self.elapsed(began);
                let admitted = admitted.map_err(|e| e.to_value())?;
                if admitted.message().version != "1" {
                    return Err(version("Message version must be '1' for this encoder"));
                }
                return Ok(json!(admitted.wire()));
            }
            "mas_validate" => {
                self.0.metrics.parse.fetch_add(1, Ordering::Relaxed);
                let began = Instant::now();
                let parsed = protocol::parse_message(
                    string("text"),
                    protocol::MAX_BYTES,
                    protocol::MAX_DEPTH,
                );
                self.elapsed(began);
                let parsed = parsed.map_err(|e| e.to_value())?;
                if parsed.version != "1" {
                    return Err(version(&format!(
                        "Message version '{}' differs from decoder version '1'",
                        parsed.version
                    )));
                }
                return parsed.to_value().map_err(|e| e.to_value());
            }
            "mas_send_envelope" => {
                self.0.metrics.admit.fetch_add(1, Ordering::Relaxed);
                let began = Instant::now();
                let admitted = protocol::admit_envelope(
                    &arguments["envelope"],
                    protocol::MAX_BYTES,
                    protocol::MAX_DEPTH,
                );
                self.elapsed(began);
                let admitted = admitted.map_err(|e| e.to_value())?;
                if admitted.message().version != "1" {
                    return Err(version("Message version must be '1'"));
                }
                self.store()
                    .and_then(|store| store.send_admitted(&admitted, string("actor")))
            }
            _ => unreachable!("known tool"),
        };
        result.map_err(|e| e.to_value())
    }
    fn elapsed(&self, began: Instant) {
        let elapsed = began.elapsed().as_nanos().min(u64::MAX as u128) as u64;
        let _ = self
            .0
            .metrics
            .codec_ns
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |old| {
                Some(old.saturating_add(elapsed))
            });
    }
}
impl ServerHandler for LabServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_protocol_version(ProtocolVersion::V_2025_11_25)
            .with_server_info(Implementation::new("MAS Protocol Lab native",env!("CARGO_PKG_VERSION")))
            .with_instructions("Durable local MAS mailboxes. Actor identities and execution evidence are trusted local host assertions; delivery does not prove cognition or task correctness.")
    }
    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        Cow::Borrowed(ProtocolVersion::known_up_to(&ProtocolVersion::V_2025_11_25))
    }
    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tools().iter().find(|tool| tool.name == name).cloned()
    }
    async fn list_tools(
        &self,
        _: Option<PaginatedRequestParams>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, rmcp::ErrorData> {
        Ok(ListToolsResult::with_all_items(self.tools().to_vec()))
    }
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, rmcp::ErrorData> {
        if context.ct.is_cancelled() {
            return Err(rmcp::ErrorData::invalid_request(
                "Tool request was cancelled before execution",
                None,
            ));
        }
        let server = self.clone();
        let result = tokio::task::spawn_blocking(move || {
            server.call(
                &request.name,
                &Value::Object(request.arguments.unwrap_or_default()),
            )
        })
        .await
        .map_err(|_| rmcp::ErrorData::internal_error("Native Lab operation failed", None))?;
        Ok(CallToolResult::structured(result).into())
    }
}
type ToolSpec<'a> = (
    &'static str,
    &'a [(&'static str, &'static str)],
    &'a [(&'static str, &'static str, Value)],
);
pub fn tools() -> Vec<Tool> {
    OPERATION_NAMES.iter().map(|name| {
        let (description,required,optional):ToolSpec<'_>=match *name {
            "mas_create_run"=>("Create a run with known agent names and finite communication/call budgets.",&[("run_id","string"),("agents","array")],&[("max_messages","integer",json!(100)),("max_bytes","integer",json!(100000)),("max_calls","integer",json!(20))]),
            "mas_receive"=>("Read pending messages addressed to the host-selected recipient.",&[("run_id","string"),("recipient","string")],&[("limit","integer",json!(10))]),
            "mas_ack"=>("Acknowledge receipt as the host-selected recipient.",&[("run_id","string"),("message_id","string"),("recipient","string")],&[]),
            "mas_status"=>("Inspect persisted run state, message counts and budget use.",&[("run_id","string")],&[]),
            "mas_encode"=>("Canonical Rust encoding without routing or spending mailbox budgets.",&[("envelope","object")],&[]),
            "mas_validate"=>("Parse baseline Rust nom protocol text without routing or spending run budgets.",&[("text","string")],&[]),
            "mas_reserve_call"=>("Reserve once before external execution; uncertain calls are not repeated.",&[("run_id","string"),("actor","string"),("call_id","string")],&[]),
            "mas_finish_call"=>("Record completion using the supplied mailbox's immutable call ledger.",&[("run_id","string"),("actor","string"),("call_id","string"),("evidence","object")],&[]),
            "mas_runtime_info"=>("Inspect native in-process codec counts and timings; no provider metadata.",&[],&[]),
            "mas_send_envelope"=>("Rust admits the complete envelope before the durable Lab send rules.",&[("envelope","object"),("actor","string")],&[]),
            _=>unreachable!()
        };
        let mut properties=Map::new();
        for(key,kind)in required { properties.insert((*key).into(),json!({"type":kind})); }
        for(key,kind,default)in optional { properties.insert((*key).into(),json!({"type":kind,"default":default})); }
        if let Some(agents)=properties.get_mut("agents") { agents["items"]=json!({"type":"string"}); }
        let schema=json!({"type":"object","properties":properties,"required":required.iter().map(|(key,_)|*key).collect::<Vec<_>>(),"additionalProperties":false});
        Tool::new(*name,description,Arc::new(schema.as_object().expect("constructed schema").clone()))
    }).collect()
}
fn validate_arguments(arguments: &Value, schema: &Map<String, Value>) -> Result<(), LabError> {
    let object = arguments.as_object().ok_or_else(LabError::invalid_input)?;
    let properties = schema["properties"].as_object().expect("built schema");
    if object.keys().any(|key| !properties.contains_key(key))
        || schema["required"]
            .as_array()
            .expect("built schema")
            .iter()
            .any(|key| !object.contains_key(key.as_str().expect("schema key")))
    {
        return Err(LabError::invalid_input());
    }
    for (key, value) in object {
        let valid = match properties[key]["type"].as_str().expect("schema type") {
            "string" => value.is_string(),
            "object" => value.is_object(),
            "array" => value
                .as_array()
                .is_some_and(|a| a.iter().all(Value::is_string)),
            "integer" => value
                .as_number()
                .is_some_and(|n| !n.to_string().contains(['.', 'e', 'E'])),
            _ => false,
        };
        if !valid {
            return Err(LabError::invalid_input());
        }
    }
    Ok(())
}
fn version(message: &str) -> Value {
    json!({"code":"VERSION","message":message,"line":null,"column":null})
}
