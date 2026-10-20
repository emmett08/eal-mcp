//! Frozen single-host coordination with durable effect reservations.
use crate::{
    bus::{JsonBus, Mailbox},
    contracts::{ServerSpec, TaskSpec},
    host::HostPaths,
    state::State,
};
use eal_runtime::private_sqlite::PrivateLock;
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

mod admission;
mod lifecycle;
mod queries;
mod record;
mod views;

pub const MESSAGE_SCHEMA: &str = "EAL/MAS-coordination/3";
pub const MAX_RAW_RESPONSE_BYTES: usize = 16 * 1024 * 1024;

pub struct Coordinator {
    pub directory: PathBuf,
    pub config: Value,
    pub server: ServerSpec,
    pub anchor: Value,
    pub task: TaskSpec,
    pub state: State,
    pub host: HostPaths,
    bus: Box<dyn Mailbox>,
    lifecycle: PrivateLock,
}
impl Coordinator {
    pub fn start(
        directory: impl AsRef<Path>,
        server: ServerSpec,
        task: TaskSpec,
        budgets: Option<&Value>,
        transport: Option<&Value>,
    ) -> Result<Self, String> {
        Self::start_with_host(
            directory,
            server,
            task,
            budgets,
            transport,
            HostPaths::current()?,
        )
    }
    pub fn start_with_host(
        directory: impl AsRef<Path>,
        server: ServerSpec,
        task: TaskSpec,
        budgets: Option<&Value>,
        transport: Option<&Value>,
        host: HostPaths,
    ) -> Result<Self, String> {
        host.validate()?;
        server.validate()?;
        task.validate()?;
        let directory = absolute(directory.as_ref())?;
        if directory.exists()
            && std::fs::read_dir(&directory)
                .map_err(|e| e.to_string())?
                .next()
                .is_some()
        {
            return Err("Use an empty directory; existing runs are immutable".into());
        }
        let budgets = budgets_value(budgets)?;
        let transport = transport_value(transport)?;
        let anchor = crate::profiles::pin_task_with_host(&server, &task, &host)?;
        let state = State::new(directory.join("coordination.sqlite"))?;
        let deadline = unix_seconds()?
            .checked_add(number(&budgets, "wall_seconds")?)
            .ok_or("Run deadline overflow")?;
        let config = json!({"schema":"EAL/coordination-run/1","run_id":format!("coord-{}",uuid::Uuid::new_v4().simple()),"server":server.to_value()?,"anchor":anchor,"budgets":budgets,"transport":transport,"created_at":eal_runtime::store::utc_now(),"deadline":deadline});
        state.initialize(&config)?;
        let value = Self::open_with_host(&directory, host)?;
        value.state.event(
            "started",
            &json!({"task_digest":value.anchor["task_digest"]}),
        )?;
        value.resume()?;
        Ok(value)
    }
    pub fn open(directory: impl AsRef<Path>) -> Result<Self, String> {
        Self::open_with_host(directory, HostPaths::current()?)
    }
    pub fn open_with_host(directory: impl AsRef<Path>, host: HostPaths) -> Result<Self, String> {
        host.validate()?;
        let directory = directory
            .as_ref()
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if !directory.join("coordination.sqlite").is_file() {
            return Err("No coordinator state; start a new run first".into());
        }
        let state = State::new(directory.join("coordination.sqlite"))?;
        let config = state.config()?;
        let fields = [
            "schema",
            "run_id",
            "server",
            "anchor",
            "budgets",
            "transport",
            "created_at",
            "deadline",
        ];
        closed(&config, &fields, "Frozen coordination configuration")?;
        if config["schema"] != "EAL/coordination-run/1" {
            return Err("Unsupported coordination run schema".into());
        }
        string(&config, "run_id")?;
        string(&config, "created_at")?;
        number(&config, "deadline")?;
        if budgets_value(Some(&config["budgets"]))? != config["budgets"] {
            return Err("Frozen run requires every budget".into());
        }
        let transport = transport_value(Some(&config["transport"]))?;
        let server = ServerSpec::from_value(&config["server"], &directory)?;
        let anchor = config["anchor"].clone();
        let task = crate::contracts::validate_anchor(&anchor)?;
        eal_runtime::private_sqlite::ensure_immutable_file(
            directory.join("response-schema.json"),
            crate::RESPONSE_SCHEMA.as_bytes(),
        )?;
        let bus: Box<dyn Mailbox> = match transport["kind"].as_str() {
            Some("json") => Box::new(JsonBus::new(directory.join("mailbox.sqlite"))?),
            Some("lab") => Box::new(crate::lab_mcp::LabMcpTransport::new(
                Path::new(string(&transport, "runtime")?),
                directory.join("mailbox.sqlite"),
            )?),
            _ => return Err("Unknown frozen coordination transport".into()),
        };
        let lifecycle = PrivateLock::new(directory.join("lifecycle.lock"))?;
        Ok(Self {
            directory,
            config,
            server,
            anchor,
            task,
            state,
            host,
            bus,
            lifecycle,
        })
    }
    pub fn budget(&self, name: &str) -> Result<usize, String> {
        usize::try_from(number(&self.config["budgets"], name)?).map_err(|e| e.to_string())
    }
    pub fn close(&self) -> Result<(), String> {
        self.bus.close()
    }
    fn active(&self) -> Result<(), String> {
        if self.state.cancelled()? {
            return Err("Run was cancelled".into());
        }
        if unix_seconds()? >= number(&self.config, "deadline")? {
            return Err("Run wall-clock budget exhausted".into());
        }
        Ok(())
    }
    fn wall_remaining(&self) -> Result<Duration, String> {
        let deadline = UNIX_EPOCH
            .checked_add(Duration::from_secs(number(&self.config, "deadline")?))
            .ok_or("Run deadline is not representable")?;
        Ok(deadline
            .duration_since(SystemTime::now())
            .unwrap_or(Duration::ZERO))
    }
    fn request(&self) -> Value {
        json!({"schema":MESSAGE_SCHEMA,"kind":"request","message_id":"request","task_id":self.task.task_id,"task_digest":self.anchor["task_digest"],"sender":"reviewer","recipient":"assessor","reply_to":null,"attempt":0})
    }
    fn worker(&self, call_id: &str, required: Option<&str>) -> Result<(String, Value), String> {
        for actor in ["assessor", "reviewer"] {
            if let Some(effect) = self.state.effect(&format!("worker:{actor}"))?
                && effect["payload"]["call_id"].as_str() == Some(call_id)
            {
                if required.is_some_and(|required| actor != required) {
                    return Err("Operation is unavailable to this worker role".into());
                }
                return Ok((actor.to_owned(), effect));
            }
        }
        Err("No prelaunch worker reservation matches this call".into())
    }
}
impl Drop for Coordinator {
    fn drop(&mut self) {
        let _ = self.bus.close();
    }
}

pub(crate) fn string<'a>(value: &'a Value, name: &str) -> Result<&'a str, String> {
    value
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("Missing or invalid {name} string"))
}
pub(crate) fn number(value: &Value, name: &str) -> Result<u64, String> {
    value
        .get(name)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("Missing or invalid {name} integer"))
}
pub(crate) fn array<'a>(value: &'a Value, name: &str) -> Result<&'a Vec<Value>, String> {
    value
        .get(name)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("Missing or invalid {name} array"))
}
pub(crate) fn closed(value: &Value, names: &[&str], label: &str) -> Result<(), String> {
    if !value.as_object().is_some_and(|object| {
        object.len() == names.len() && names.iter().all(|name| object.contains_key(*name))
    }) {
        return Err(format!("{label} requires exactly its declared fields"));
    }
    Ok(())
}
pub(crate) fn digest(value: &Value) -> Result<String, String> {
    eal_core::canonical::canonical_digest(
        value,
        &eal_core::ExecutionLimits {
            record_bytes: 64 * 1024 * 1024,
            json_nodes: 64 * 1024 * 1024,
            json_depth: 128,
            ..Default::default()
        },
    )
}
fn unix_seconds() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|v| v.as_secs())
        .map_err(|e| e.to_string())
}
fn absolute(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        Ok(path.to_owned())
    } else {
        Ok(std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path))
    }
}
fn budgets_value(supplied: Option<&Value>) -> Result<Value, String> {
    let mut value = json!({"max_worker_calls":2,"max_gateway_calls_per_role":4,"max_mcp_calls_per_role":64,"max_messages":8,"max_bytes":65536,"max_worker_output_bytes":65536,"wall_seconds":900,"worker_seconds":180});
    if let Some(supplied) = supplied {
        let supplied = supplied.as_object().ok_or("Unknown coordination budget")?;
        for (name, selected) in supplied {
            if !value.as_object().unwrap().contains_key(name) {
                return Err("Unknown coordination budget".into());
            }
            let selected = selected
                .as_u64()
                .ok_or_else(|| format!("Missing or invalid {name} integer"))?;
            value[name] = json!(selected);
        }
    }
    for (name, max) in [
        ("max_worker_calls", 2),
        ("max_gateway_calls_per_role", 64),
        ("max_mcp_calls_per_role", 256),
        ("max_messages", 100),
        ("max_bytes", 1048576),
        ("max_worker_output_bytes", 1048576),
        ("wall_seconds", 3600),
        ("worker_seconds", 3600),
    ] {
        let selected = number(&value, name)?;
        if selected == 0 || selected > max {
            return Err("Budgets exceed the single-host coordinator profile".into());
        }
    }
    if number(&value, "worker_seconds")? > number(&value, "wall_seconds")? {
        return Err("Worker budget exceeds run wall budget".into());
    }
    Ok(value)
}
fn transport_value(supplied: Option<&Value>) -> Result<Value, String> {
    let default = json!({"kind":"json"});
    let value = supplied.unwrap_or(&default);
    if value.as_object().is_some_and(|object| {
        object.len() == 1 && object.get("kind").and_then(Value::as_str) == Some("json")
    }) {
        return Ok(default);
    }
    closed(value, &["kind", "runtime"], "Transport")?;
    if value["kind"] != "lab" {
        return Err("Transport must be JSON or an explicitly installed native Lab runtime".into());
    }
    let runtime = absolute(Path::new(string(value, "runtime")?))?;
    Ok(json!({"kind":"lab","runtime":runtime}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configuration_rejects_nested_values_before_cloning() {
        std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(|| {
                let mut nested = Value::Null;
                for _ in 0..20_000 {
                    nested = Value::Array(vec![nested]);
                }
                let mut budget = serde_json::Map::new();
                budget.insert("max_messages".into(), nested);
                let budget = Value::Object(budget);
                assert!(budgets_value(Some(&budget)).is_err());
                let mut transport = serde_json::Map::new();
                transport.insert("kind".into(), Value::String("lab".into()));
                transport.insert("runtime".into(), budget);
                let mut transport = Value::Object(transport);
                assert!(transport_value(Some(&transport)).is_err());
                // Drop hostile caller-owned input iteratively too.
                let mut pending = vec![transport.take()];
                while let Some(value) = pending.pop() {
                    match value {
                        Value::Array(values) => pending.extend(values),
                        Value::Object(values) => pending.extend(values.into_values()),
                        _ => {}
                    }
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
