// Derived from MAS Protocol Lab Store, copyright (c) 2026 Emmett Miller.
// MIT notice and pinned source hash: protocol/LICENSE-MIT and protocol/UPSTREAM.json.
//! The pinned MAS Protocol Lab mailbox, separate from the coordinator's JSON bus.
use crate::protocol::AdmittedMessage;
use eal_core::{ExecutionLimits, canonical::canonical_json_bytes};
use eal_runtime::private_sqlite::PrivateDatabase;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde_json::{Value, json};
use std::{
    cmp::Ordering,
    collections::BTreeSet,
    path::Path,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

mod messages;
mod records;

const REJECT: &str = "__eal_lab_rejected__:";
const INVALID: &str = "__eal_lab_invalid_input__";
const EXECUTION_CLAIM: &str = "Host execution must be substantiated by external agent records; receive and ack alone do not prove cognition.";
pub type LabClock = Arc<dyn Fn() -> f64 + Send + Sync>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabError {
    pub code: String,
    pub message: String,
}
impl LabError {
    pub fn rejected(message: impl Into<String>) -> Self {
        Self {
            code: "operation_rejected".into(),
            message: message.into(),
        }
    }
    pub fn invalid_input() -> Self {
        Self {
            code: "invalid_input".into(),
            message: "Input does not match the required message or operation schema.".into(),
        }
    }
    pub fn to_value(&self) -> Value {
        json!({"code": self.code, "message": self.message})
    }
}
impl std::fmt::Display for LabError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for LabError {}

#[derive(Clone)]
pub struct LabStore {
    database: PrivateDatabase,
    clock: LabClock,
}
impl LabStore {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, LabError> {
        Self::with_clock(
            path,
            Arc::new(|| {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_or(0.0, |d| d.as_secs_f64())
            }),
        )
    }
    /// A per-instance clock supports reproducible deadlines without global state.
    pub fn with_clock(path: impl AsRef<Path>, clock: LabClock) -> Result<Self, LabError> {
        let database =
            PrivateDatabase::new(path, Duration::from_secs(10)).map_err(|_| LabError {
                code: "file_unavailable".into(),
                message: "The state or input file could not be accessed.".into(),
            })?;
        database
            .with_transaction(|db| {
                db.execute_batch(
                    "CREATE TABLE IF NOT EXISTS runs(
id TEXT PRIMARY KEY, config TEXT NOT NULL, deadline REAL NOT NULL,
message_count INTEGER NOT NULL DEFAULT 0, byte_count INTEGER NOT NULL DEFAULT 0,
call_count INTEGER NOT NULL DEFAULT 0, active TEXT NOT NULL DEFAULT '1');
CREATE TABLE IF NOT EXISTS messages(
seq INTEGER PRIMARY KEY AUTOINCREMENT, run TEXT NOT NULL, id TEXT NOT NULL,
sender TEXT NOT NULL, recipient TEXT NOT NULL, content TEXT NOT NULL,
hash TEXT NOT NULL, received INTEGER NOT NULL DEFAULT 0,
acked INTEGER NOT NULL DEFAULT 0, UNIQUE(run,id));
CREATE TABLE IF NOT EXISTS protocols(
run TEXT NOT NULL, version TEXT NOT NULL, spec TEXT NOT NULL,
certificate TEXT NOT NULL, UNIQUE(run,version));
CREATE TABLE IF NOT EXISTS calls(
run TEXT NOT NULL, id TEXT NOT NULL, actor TEXT NOT NULL,
state TEXT NOT NULL DEFAULT 'reserved', result TEXT, UNIQUE(run,id));",
                )
                .map_err(|e| e.to_string())
            })
            .map_err(state_error)?;
        Ok(Self { database, clock })
    }
    pub fn path(&self) -> &Path {
        self.database.path()
    }
    pub fn create_run(&self, arguments: &Value) -> Result<Value, LabError> {
        let run_id = arguments["run_id"]
            .as_str()
            .filter(|s| valid_id(s))
            .ok_or_else(|| LabError::rejected("invalid run id"))?;
        let agents = arguments["agents"]
            .as_array()
            .filter(|a| (2..=32).contains(&a.len()))
            .ok_or_else(|| LabError::rejected("register between 2 and 32 workers"))?;
        if agents
            .iter()
            .any(|a| a.as_str().is_none_or(|s| !valid_id(s)))
            || agents
                .iter()
                .filter_map(Value::as_str)
                .collect::<BTreeSet<_>>()
                .len()
                != agents.len()
        {
            return Err(LabError::rejected(
                "worker ids must be unique non-empty strings",
            ));
        }
        let mut config = json!({"agents": agents, "max_messages": 100, "max_bytes": 100000, "max_calls": 20, "wall_seconds": 3600});
        for key in ["max_messages", "max_bytes", "max_calls", "wall_seconds"] {
            if let Some(value) = arguments.get(key) {
                if !positive_integer(value) {
                    return Err(LabError::rejected("budgets must be positive integers"));
                }
                config[key] = value.clone();
            }
            if !positive_integer(&config[key]) {
                return Err(LabError::rejected("budgets must be positive integers"));
            }
        }
        let encoded = dump(&config).map_err(state_error)?;
        self.transaction(|db| {
            let old = db
                .query_row("SELECT config FROM runs WHERE id=?", [run_id], |r| {
                    r.get::<_, String>(0)
                })
                .optional()
                .map_err(sql)?;
            if let Some(old) = old {
                if old != encoded {
                    return Err(rejected("frozen run configuration differs"));
                }
            } else {
                let wall = config["wall_seconds"]
                    .as_f64()
                    .ok_or_else(|| rejected("wall-clock budget is not representable"))?;
                let deadline = self.now().map_err(|e| rejected(&e.message))? + wall;
                if !deadline.is_finite() {
                    return Err(rejected("wall-clock budget is not representable"));
                }
                db.execute(
                    "INSERT INTO runs(id,config,deadline) VALUES(?,?,?)",
                    params![run_id, encoded, deadline],
                )
                .map_err(sql)?;
            }
            Ok(())
        })?;
        self.inspect(run_id)
    }
    pub fn send_admitted(&self, message: &AdmittedMessage, actor: &str) -> Result<Value, LabError> {
        self.send_inner(message, actor)
    }
    pub fn receive(&self, run_id: &str, recipient: &str, limit: &Value) -> Result<Value, LabError> {
        self.receive_inner(run_id, recipient, limit)
    }
    pub fn ack(&self, run_id: &str, message_id: &str, recipient: &str) -> Result<Value, LabError> {
        self.ack_inner(run_id, message_id, recipient)
    }
    pub fn inspect(&self, run_id: &str) -> Result<Value, LabError> {
        self.connection(|db| records::inspect(db, run_id))
    }
    pub fn reserve_call(
        &self,
        run_id: &str,
        actor: &str,
        call_id: &str,
    ) -> Result<Value, LabError> {
        if call_id.is_empty() {
            return Err(LabError::rejected("call id required"));
        }
        self.transaction(|db| {
            let run = checked_run(
                db,
                run_id,
                Some(actor),
                Some(self.now().map_err(|e| rejected(&e.message))?),
            )?;
            if db
                .query_row(
                    "SELECT 1 FROM calls WHERE run=? AND id=?",
                    params![run_id, call_id],
                    |_| Ok(()),
                )
                .optional()
                .map_err(sql)?
                .is_some()
            {
                return Err(rejected(
                    "call already reserved; reconcile rather than repeat",
                ));
            }
            if !within(run.call_count, &run.config["max_calls"]) {
                return Err(rejected("call budget exhausted"));
            }
            db.execute(
                "INSERT INTO calls(run,id,actor) VALUES(?,?,?)",
                params![run_id, call_id, actor],
            )
            .map_err(sql)?;
            db.execute(
                "UPDATE runs SET call_count=call_count+1 WHERE id=?",
                [run_id],
            )
            .map_err(sql)?;
            Ok(json!({"reserved":true,"call_id":call_id}))
        })
    }
    pub fn finish_call(
        &self,
        run_id: &str,
        actor: &str,
        call_id: &str,
        evidence: &Value,
    ) -> Result<Value, LabError> {
        if !evidence.is_object() {
            return Err(LabError::rejected("evidence object required"));
        }
        self.transaction(|db| {
            checked_run(db, run_id, Some(actor), None)?;
            let prior = db
                .query_row(
                    "SELECT actor,state FROM calls WHERE run=? AND id=?",
                    params![run_id, call_id],
                    |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
                )
                .optional()
                .map_err(sql)?;
            let Some((_, state)) = prior.filter(|(owner, _)| owner == actor) else {
                return Err(rejected("call reservation belongs to another worker"));
            };
            if state != "reserved" {
                return Err(rejected("call already completed"));
            }
            let encoded = dump(evidence).map_err(|_| INVALID.to_owned())?;
            db.execute(
                "UPDATE calls SET state=?,result=? WHERE run=? AND id=?",
                params!["recorded", encoded, run_id, call_id],
            )
            .map_err(sql)?;
            Ok(json!({"recorded":true}))
        })
    }
    fn now(&self) -> Result<f64, LabError> {
        let now = (self.clock)();
        if now.is_finite() {
            Ok(now)
        } else {
            Err(LabError::rejected("wall clock is not finite"))
        }
    }
    fn transaction<T>(
        &self,
        operation: impl FnOnce(&Transaction<'_>) -> Result<T, String>,
    ) -> Result<T, LabError> {
        self.database
            .with_transaction(operation)
            .map_err(state_error)
    }
    fn connection<T>(
        &self,
        operation: impl FnOnce(&Connection) -> Result<T, String>,
    ) -> Result<T, LabError> {
        self.database
            .with_connection(operation)
            .map_err(state_error)
    }
}

pub(super) struct Run {
    config: Value,
    deadline: f64,
    message_count: i64,
    byte_count: i64,
    call_count: i64,
    active: String,
}
fn checked_run(
    db: &Connection,
    run_id: &str,
    actor: Option<&str>,
    now: Option<f64>,
) -> Result<Run, String> {
    let row = db.query_row("SELECT config,deadline,message_count,byte_count,call_count,active FROM runs WHERE id=?",[run_id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,f64>(1)?,r.get::<_,i64>(2)?,r.get::<_,i64>(3)?,r.get::<_,i64>(4)?,r.get::<_,String>(5)?))).optional().map_err(sql)?.ok_or_else(||rejected("unknown run"))?;
    let run = Run {
        config: decode(&row.0)?,
        deadline: row.1,
        message_count: row.2,
        byte_count: row.3,
        call_count: row.4,
        active: row.5,
    };
    if let Some(actor) = actor
        && !run.config["agents"]
            .as_array()
            .is_some_and(|a| a.iter().any(|v| v == actor))
    {
        return Err(rejected("unregistered worker"));
    }
    if now.is_some_and(|now| now >= run.deadline) {
        return Err(rejected("wall-clock budget exhausted"));
    }
    Ok(run)
}
fn valid_id(value: &str) -> bool {
    let mut bytes = value.bytes();
    value.len() <= 128
        && bytes.next().is_some_and(|b| b.is_ascii_alphanumeric())
        && bytes.all(|b| b.is_ascii_alphanumeric() || b"_.:/-".contains(&b))
}
fn positive_integer(value: &Value) -> bool {
    value.as_number().is_some_and(|n| {
        let s = n.to_string();
        s.bytes().all(|b| b.is_ascii_digit()) && s.bytes().any(|b| b != b'0')
    })
}
/// Counts already consumed must be strictly smaller before reserving one more.
fn within(count: i64, maximum: &Value) -> bool {
    count_order(count, maximum) == Some(Ordering::Less)
}
fn count_order(count: i64, maximum: &Value) -> Option<Ordering> {
    if count < 0 || !positive_integer(maximum) {
        return None;
    }
    let count = count.to_string();
    let maximum = maximum.as_number()?.to_string();
    Some(
        count
            .len()
            .cmp(&maximum.len())
            .then_with(|| count.cmp(&maximum)),
    )
}
fn rejected(message: &str) -> String {
    format!("{REJECT}{message}")
}
fn sql(error: rusqlite::Error) -> String {
    error.to_string()
}
fn state_error(error: String) -> LabError {
    if error == INVALID {
        LabError::invalid_input()
    } else if let Some(message) = error.strip_prefix(REJECT) {
        LabError::rejected(message)
    } else {
        LabError {
            code: "state_unavailable".into(),
            message: "The state database could not complete the operation.".into(),
        }
    }
}
fn dump(value: &Value) -> Result<String, String> {
    let limits = ExecutionLimits {
        json_depth: 128,
        json_nodes: 1_000_000,
        record_bytes: 64 * 1024 * 1024,
        ..Default::default()
    };
    let bytes = canonical_json_bytes(value, &limits)?;
    if bytes.len() > limits.record_bytes {
        return Err("Lab state JSON exceeds the native byte limit".into());
    }
    String::from_utf8(bytes).map_err(|e| e.to_string())
}
fn decode(text: &str) -> Result<Value, String> {
    eal_core::json::parse(
        text,
        &ExecutionLimits {
            json_depth: 128,
            json_nodes: 1_000_000,
            record_bytes: 64 * 1024 * 1024,
            ..Default::default()
        },
    )
    .map_err(|e| e.to_string())
}
