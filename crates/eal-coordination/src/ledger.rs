//! Durable effect reservations and an immutable reply outbox.
use eal_runtime::private_sqlite::PrivateDatabase;
use rusqlite::{OptionalExtension, Transaction, params};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, path::Path, time::Duration};

mod records;
pub(crate) use records::{EffectRow, effect_row, snapshot};

const MAX_JSON_BYTES: usize = 64 * 1024 * 1024;

/// Only `New` permits the caller to begin an external effect. Unknown effects
/// are never reopened by expiry or replay.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectStatus {
    New,
    Committed,
    Unknown,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct EffectSnapshot {
    pub status: EffectStatus,
    pub key: String,
    pub payload_digest: String,
    pub result: Option<Value>,
    pub outbox: Vec<Value>,
    pub acknowledged: bool,
    pub diagnosis: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConfirmedSend {
    pub key: String,
    pub message_id: String,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct DrainReport {
    pub confirmed_sends: Vec<ConfirmedSend>,
    pub acknowledged_effects: Vec<String>,
}
#[derive(Clone)]
pub struct EffectLedger {
    pub(crate) database: PrivateDatabase,
}
impl EffectLedger {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, String> {
        let database = PrivateDatabase::new(path, Duration::from_secs(10))?;
        database.with_transaction(|db| {
            db.execute_batch(
                "CREATE TABLE IF NOT EXISTS effects (
effect_key TEXT PRIMARY KEY, payload TEXT NOT NULL, payload_digest TEXT NOT NULL,
state TEXT NOT NULL CHECK(state IN ('reserved', 'committed')), result TEXT,
acknowledged INTEGER NOT NULL DEFAULT 0 CHECK(acknowledged IN (0, 1)),
CHECK(state = 'reserved' OR result IS NOT NULL));
CREATE TABLE IF NOT EXISTS effect_outbox (
effect_key TEXT NOT NULL REFERENCES effects(effect_key), position INTEGER NOT NULL,
message_id TEXT NOT NULL, message TEXT NOT NULL,
sent INTEGER NOT NULL DEFAULT 0 CHECK(sent IN (0, 1)),
PRIMARY KEY(effect_key, position), UNIQUE(effect_key, message_id));",
            )
            .map_err(|e| e.to_string())
        })?;
        Ok(Self { database })
    }
    pub fn path(&self) -> &Path {
        self.database.path()
    }
    pub fn reserve_effect(&self, key: &str, payload: &Value) -> Result<EffectSnapshot, String> {
        identity(key)?;
        let encoded = canonical_json(payload)?;
        let digest = format!("{:x}", Sha256::digest(encoded.as_bytes()));
        self.database.with_transaction(|db| {
            if let Some(row) = effect_row(db, key)? {
                if row.digest != digest || row.payload != encoded {
                    return Err("Inbound identity collision: canonical payload changed".into());
                }
                return snapshot(db, &row, false);
            }
            db.execute(
                "INSERT INTO effects(effect_key, payload, payload_digest, state) VALUES (?, ?, ?, 'reserved')",
                params![key, encoded, digest],
            ).map_err(|e| e.to_string())?;
            snapshot(db, &required(db, key)?, true)
        })
    }
    pub fn complete_effect(
        &self,
        key: &str,
        result: &Value,
        outbox: &[Value],
    ) -> Result<EffectSnapshot, String> {
        identity(key)?;
        let encoded = canonical_json(result)?;
        let messages = encode_messages(outbox)?;
        self.database.with_transaction(|db| {
            let row = required(db, key)?;
            if row.state == "committed" {
                let stored = records::messages(db, key)?;
                if row.result.as_deref() != Some(&encoded) || stored != messages {
                    return Err("Committed effect result or outbox cannot change".into());
                }
                return snapshot(db, &row, false);
            }
            db.execute("UPDATE effects SET state = 'committed', result = ? WHERE effect_key = ?", params![encoded,key]).map_err(|e|e.to_string())?;
            for (position,(id,message)) in messages.iter().enumerate() {
                db.execute("INSERT INTO effect_outbox(effect_key, position, message_id, message) VALUES (?, ?, ?, ?)",params![key,position as i64,id,message]).map_err(|e|e.to_string())?;
            }
            snapshot(db, &required(db, key)?, false)
        })
    }
    /// Each callback runs after the read connection closes and outside every
    /// transaction. A callback error leaves its confirmation unrecorded.
    pub fn drain_outbox(
        &self,
        mut send: impl FnMut(&Value) -> Result<(), String>,
        mut ack: impl FnMut(&str) -> Result<(), String>,
    ) -> Result<DrainReport, String> {
        let keys = self.database.with_connection(|db| {
            let mut query = db.prepare("SELECT effect_key FROM effects WHERE state = 'committed' AND acknowledged = 0 ORDER BY rowid").map_err(|e|e.to_string())?;
            query.query_map([], |row| row.get::<_,String>(0)).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())
        })?;
        let mut report = DrainReport::default();
        for key in keys {
            let messages = self.database.with_connection(|db| {
                let mut query = db.prepare("SELECT position, message_id, message FROM effect_outbox WHERE effect_key = ? AND sent = 0 ORDER BY position").map_err(|e|e.to_string())?;
                query.query_map([&key],|row|Ok((row.get::<_,i64>(0)?,row.get::<_,String>(1)?,row.get::<_,String>(2)?))).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())
            })?;
            for (position, message_id, message) in messages {
                send(&decode(&message)?)?;
                self.database.with_transaction(|db| {
                    db.execute(
                        "UPDATE effect_outbox SET sent = 1 WHERE effect_key = ? AND position = ?",
                        params![key, position],
                    )
                    .map_err(|e| e.to_string())?;
                    Ok(())
                })?;
                report.confirmed_sends.push(ConfirmedSend {
                    key: key.clone(),
                    message_id,
                });
            }
            let unsent = self.database.with_connection(|db| {
                db.query_row(
                    "SELECT 1 FROM effect_outbox WHERE effect_key = ? AND sent = 0 LIMIT 1",
                    [&key],
                    |_| Ok(()),
                )
                .optional()
                .map_err(|e| e.to_string())
            })?;
            if unsent.is_none() {
                ack(&key)?;
                self.database.with_transaction(|db| {
                    db.execute("UPDATE effects SET acknowledged = 1 WHERE effect_key = ? AND state = 'committed'", [&key]).map_err(|e|e.to_string())?;
                    Ok(())
                })?;
                report.acknowledged_effects.push(key);
            }
        }
        Ok(report)
    }
}
fn required(db: &Transaction<'_>, key: &str) -> Result<EffectRow, String> {
    effect_row(db, key)?.ok_or("Complete requires an existing effect reservation".into())
}
fn encode_messages(outbox: &[Value]) -> Result<Vec<(String, String)>, String> {
    let mut ids = BTreeSet::new();
    outbox
        .iter()
        .map(|message| {
            let object = message
                .as_object()
                .ok_or("Outbox entries must be message objects")?;
            let id = object
                .get("message_id")
                .and_then(Value::as_str)
                .ok_or("A nonempty stable identity is required")?;
            identity(id)?;
            if !ids.insert(id) {
                return Err("Outbox message identities must be unique within an effect".into());
            }
            Ok((id.to_owned(), canonical_json(message)?))
        })
        .collect()
}
pub(crate) fn identity(value: &str) -> Result<(), String> {
    if value
        .trim_matches(|c: char| c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c))
        .is_empty()
    {
        Err("A nonempty stable identity is required".into())
    } else {
        Ok(())
    }
}
fn json_limits() -> eal_core::ExecutionLimits {
    eal_core::ExecutionLimits {
        json_depth: eal_core::json::MAX_JSON_DEPTH,
        json_nodes: MAX_JSON_BYTES,
        record_bytes: MAX_JSON_BYTES,
        ..Default::default()
    }
}
pub fn canonical_json(value: &Value) -> Result<String, String> {
    eal_core::canonical::canonical_json_bytes(value, &json_limits())
        .and_then(|bytes| String::from_utf8(bytes).map_err(|e| e.to_string()))
        .map_err(|_| "Ledger values must be finite UTF-8 JSON".into())
}
pub(crate) fn decode(text: &str) -> Result<Value, String> {
    eal_core::json::parse(text, &json_limits())
        .map_err(|e| format!("Invalid stored coordination JSON: {e}"))
}
