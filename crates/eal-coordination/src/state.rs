//! Authoritative local coordination state; JSON exports remain detached views.
use crate::ledger::{self, DrainReport, EffectLedger, EffectSnapshot, canonical_json, decode};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};
use std::{path::Path, sync::Arc};

#[derive(Clone)]
pub struct State {
    ledger: EffectLedger,
    clock: Arc<dyn Fn() -> String + Send + Sync>,
}
impl State {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, String> {
        Self::with_clock(path, Arc::new(eal_runtime::store::utc_now))
    }
    pub fn with_clock(
        path: impl AsRef<Path>,
        clock: Arc<dyn Fn() -> String + Send + Sync>,
    ) -> Result<Self, String> {
        let ledger = EffectLedger::new(path)?;
        ledger.database.with_transaction(|db|{
            db.execute_batch("CREATE TABLE IF NOT EXISTS run_config(id INTEGER PRIMARY KEY CHECK(id=1), content TEXT NOT NULL, cancelled INTEGER NOT NULL DEFAULT 0);
CREATE TABLE IF NOT EXISTS events(seq INTEGER PRIMARY KEY, at TEXT NOT NULL, kind TEXT NOT NULL, content TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS findings(id TEXT PRIMARY KEY, content TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS calls(seq INTEGER PRIMARY KEY, actor TEXT NOT NULL, category TEXT NOT NULL, operation TEXT NOT NULL, at TEXT NOT NULL);").map_err(|e|e.to_string())
        })?;
        Ok(Self { ledger, clock })
    }
    pub fn path(&self) -> &Path {
        self.ledger.path()
    }
    pub fn initialize(&self, config: &Value) -> Result<(), String> {
        self.ledger.database.with_transaction(|db| {
            if db
                .query_row("SELECT 1 FROM run_config", [], |_| Ok(()))
                .optional()
                .map_err(|e| e.to_string())?
                .is_some()
            {
                return Err("A run already exists; resume its frozen configuration".into());
            }
            let encoded = canonical_json(config)?;
            db.execute("INSERT INTO run_config(id,content) VALUES(1,?)", [encoded])
                .map_err(|e| e.to_string())?;
            Ok(())
        })
    }
    pub fn config(&self) -> Result<Value, String> {
        self.ledger.database.with_transaction(|db| {
            let text: Option<String> = db
                .query_row("SELECT content FROM run_config WHERE id=1", [], |row| {
                    row.get(0)
                })
                .optional()
                .map_err(|e| e.to_string())?;
            decode(&text.ok_or("No coordination run exists in this directory")?)
        })
    }
    pub fn event(&self, kind: &str, content: &Value) -> Result<(), String> {
        let encoded = canonical_json(content)?;
        self.ledger.database.with_transaction(|db| {
            db.execute(
                "INSERT INTO events(at,kind,content) VALUES(?,?,?)",
                params![(self.clock)(), kind, encoded],
            )
            .map_err(|e| e.to_string())?;
            Ok(())
        })
    }
    pub fn events(&self) -> Result<Vec<Value>, String> {
        self.ledger.database.with_transaction(|db| {
            let mut query = db
                .prepare("SELECT seq,at,kind,content FROM events ORDER BY seq")
                .map_err(|e| e.to_string())?;
            let rows = query
                .query_map([], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;
            rows.into_iter()
                .map(|(sequence, at, kind, content)| {
                    Ok(json!({"sequence":sequence,"at":at,"kind":kind,"data":decode(&content)?}))
                })
                .collect()
        })
    }
    pub fn reserve_effect(&self, key: &str, payload: &Value) -> Result<EffectSnapshot, String> {
        self.ledger.reserve_effect(key, payload)
    }
    pub fn complete_effect(
        &self,
        key: &str,
        result: &Value,
        outbox: &[Value],
    ) -> Result<EffectSnapshot, String> {
        self.ledger.complete_effect(key, result, outbox)
    }
    pub fn drain_outbox(
        &self,
        send: impl FnMut(&Value) -> Result<(), String>,
        ack: impl FnMut(&str) -> Result<(), String>,
    ) -> Result<DrainReport, String> {
        self.ledger.drain_outbox(send, ack)
    }
    pub fn effect(&self, key: &str) -> Result<Option<Value>, String> {
        self.ledger.database.with_transaction(|db| effect(db, key))
    }
    pub fn effects(&self) -> Result<Vec<Value>, String> {
        self.ledger.database.with_transaction(|db| {
            let mut query = db
                .prepare("SELECT effect_key FROM effects ORDER BY rowid")
                .map_err(|e| e.to_string())?;
            let keys = query
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;
            keys.iter()
                .map(|key| effect(db, key)?.ok_or("Effect disappeared during snapshot".into()))
                .collect()
        })
    }
    pub fn issue(&self, finding: &Value) -> Result<(), String> {
        let encoded = canonical_json(finding)?;
        let id = finding
            .get("finding_id")
            .and_then(Value::as_str)
            .ok_or("Finding requires a finding_id string")?;
        self.ledger.database.with_transaction(|db| {
            let prior: Option<String> = db
                .query_row("SELECT content FROM findings WHERE id=?", [id], |row| {
                    row.get(0)
                })
                .optional()
                .map_err(|e| e.to_string())?;
            if prior.as_deref().is_some_and(|prior| prior != encoded) {
                return Err("Finding identity collision".into());
            }
            db.execute(
                "INSERT OR IGNORE INTO findings(id,content) VALUES(?,?)",
                params![id, encoded],
            )
            .map_err(|e| e.to_string())?;
            Ok(())
        })
    }
    pub fn finding(&self, id: &str) -> Result<Value, String> {
        self.ledger.database.with_transaction(|db| {
            let text: Option<String> = db
                .query_row("SELECT content FROM findings WHERE id=?", [id], |row| {
                    row.get(0)
                })
                .optional()
                .map_err(|e| e.to_string())?;
            decode(&text.ok_or("Finding was not issued by this run")?)
        })
    }
    pub fn reserve_call(
        &self,
        actor: &str,
        category: &str,
        operation: &str,
        maximum: usize,
    ) -> Result<(), String> {
        self.ledger.database.with_transaction(|db| {
            let count: i64 = db
                .query_row(
                    "SELECT count(*) FROM calls WHERE actor=? AND category=?",
                    params![actor, category],
                    |row| row.get(0),
                )
                .map_err(|e| e.to_string())?;
            if count as u64 >= maximum as u64 {
                return Err(format!("{category} call budget exhausted for {actor}"));
            }
            db.execute(
                "INSERT INTO calls(actor,category,operation,at) VALUES(?,?,?,?)",
                params![actor, category, operation, (self.clock)()],
            )
            .map_err(|e| e.to_string())?;
            Ok(())
        })
    }
    pub fn calls(&self) -> Result<Vec<Value>, String> {
        self.ledger.database.with_transaction(|db|{
            let mut query=db.prepare("SELECT seq,actor,category,operation,at FROM calls ORDER BY seq").map_err(|e|e.to_string())?;
            query.query_map([],|row|Ok(json!({"seq":row.get::<_,i64>(0)?,"actor":row.get::<_,String>(1)?,"category":row.get::<_,String>(2)?,"operation":row.get::<_,String>(3)?,"at":row.get::<_,String>(4)?}))).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())
        })
    }
    pub fn cancel(&self) -> Result<(), String> {
        self.ledger.database.with_transaction(|db| {
            db.execute("UPDATE run_config SET cancelled=1 WHERE id=1", [])
                .map_err(|e| e.to_string())?;
            Ok(())
        })?;
        self.event("cancelled", &json!({}))
    }
    pub fn cancelled(&self) -> Result<bool, String> {
        self.ledger.database.with_transaction(|db| {
            db.query_row("SELECT cancelled FROM run_config WHERE id=1", [], |row| {
                row.get(0)
            })
            .optional()
            .map_err(|e| e.to_string())?
            .ok_or("No coordination run exists in this directory".into())
        })
    }
}
fn effect(db: &Connection, key: &str) -> Result<Option<Value>, String> {
    ledger::effect_row(db, key)?
        .map(|row| {
            let mut value = serde_json::to_value(ledger::snapshot(db, &row, false)?)
                .map_err(|e| e.to_string())?;
            value["payload"] = decode(&row.payload)?;
            Ok(value)
        })
        .transpose()
}
