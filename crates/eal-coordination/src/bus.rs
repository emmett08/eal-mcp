//! Stable-ID addressed delivery over the local durable JSON mailbox.
use crate::ledger::{canonical_json, decode};
use eal_runtime::private_sqlite::PrivateDatabase;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{path::Path, time::Duration};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MailboxLimits {
    pub max_messages: usize,
    pub max_bytes: usize,
    pub max_calls: usize,
}
impl Default for MailboxLimits {
    fn default() -> Self {
        Self {
            max_messages: 8,
            max_bytes: 65536,
            max_calls: 2,
        }
    }
}
pub trait Mailbox: Send + Sync {
    fn close(&self) -> Result<(), String> {
        Ok(())
    }
    fn create(
        &self,
        run_id: &str,
        participants: &[String],
        limits: MailboxLimits,
    ) -> Result<Value, String>;
    fn send(&self, run_id: &str, payload: &Value, actor: &str) -> Result<Value, String>;
    fn receive(&self, run_id: &str, recipient: &str, limit: usize) -> Result<Value, String>;
    fn acknowledge(&self, run_id: &str, message_id: &str, recipient: &str)
    -> Result<Value, String>;
    fn reserve(&self, run_id: &str, actor: &str, call_id: &str) -> Result<Value, String>;
    fn finish(
        &self,
        run_id: &str,
        actor: &str,
        call_id: &str,
        evidence: &Value,
    ) -> Result<Value, String>;
    fn status(&self, run_id: &str) -> Result<Value, String>;
}
pub fn envelope(run_id: &str, message: &Value) -> Result<Value, String> {
    let field = |name: &str| {
        message
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Message requires {name}"))
    };
    Ok(
        json!({"message_id":field("message_id")?,"run_id":run_id,"sender":field("sender")?,"recipient":field("recipient")?,"version":"1","kind":field("kind")?,"payload":message}),
    )
}
#[derive(Clone)]
pub struct JsonBus {
    database: PrivateDatabase,
}
impl JsonBus {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, String> {
        let database = PrivateDatabase::new(path, Duration::from_secs(30))?;
        database.with_transaction(|db| {
            db.execute_batch("CREATE TABLE IF NOT EXISTS bus_config(id INTEGER PRIMARY KEY CHECK(id=1), content TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS bus_messages(seq INTEGER PRIMARY KEY, id TEXT UNIQUE, content TEXT, received INTEGER DEFAULT 0, acked INTEGER DEFAULT 0);
CREATE TABLE IF NOT EXISTS bus_calls(id TEXT PRIMARY KEY, actor TEXT, evidence TEXT);").map_err(|e|e.to_string())
        })?;
        Ok(Self { database })
    }
    pub fn path(&self) -> &Path {
        self.database.path()
    }
}
impl Mailbox for JsonBus {
    fn create(
        &self,
        run_id: &str,
        participants: &[String],
        limits: MailboxLimits,
    ) -> Result<Value, String> {
        let encoded = canonical_json(
            &json!({"run_id":run_id,"participants":participants,"max_messages":limits.max_messages,"max_bytes":limits.max_bytes,"max_calls":limits.max_calls}),
        )?;
        self.database.with_transaction(|db| {
            let prior: Option<String> = db
                .query_row("SELECT content FROM bus_config", [], |row| row.get(0))
                .optional()
                .map_err(|e| e.to_string())?;
            if prior.as_deref().is_some_and(|prior| prior != encoded) {
                return Err("Frozen mailbox configuration differs".into());
            }
            db.execute(
                "INSERT OR IGNORE INTO bus_config(id,content) VALUES(1,?)",
                [encoded],
            )
            .map_err(|e| e.to_string())?;
            Ok(Value::Null)
        })
    }
    fn send(&self, run_id: &str, payload: &Value, actor: &str) -> Result<Value, String> {
        let encoded = canonical_json(&envelope(run_id, payload)?)?;
        self.database.with_transaction(|db| {
            let config=config(db,run_id,Some(actor))?;
            let recipient=payload.get("recipient").and_then(Value::as_str);
            if payload.get("sender").and_then(Value::as_str)!=Some(actor) || !config["participants"].as_array().is_some_and(|values|values.iter().any(|value|value.as_str()==recipient)) {
                return Err("Message has invalid addressed identities".into());
            }
            let id=payload.get("message_id").and_then(Value::as_str).ok_or("Message requires message_id string")?;
            let prior:Option<(String,i64)>=db.query_row("SELECT content,seq FROM bus_messages WHERE id=?",[id],|row|Ok((row.get(0)?,row.get(1)?))).optional().map_err(|e|e.to_string())?;
            if let Some((content,sequence))=prior {
                if content!=encoded {return Err("Changed content under existing message identity".into());}
                return Ok(json!({"accepted":true,"duplicate":true,"sequence":sequence}));
            }
            let (count,bytes):(i64,i64)=db.query_row("SELECT count(*), COALESCE(sum(length(CAST(content AS BLOB))),0) FROM bus_messages",[],|row|Ok((row.get(0)?,row.get(1)?))).map_err(|e|e.to_string())?;
            let max_bytes=number(&config,"max_bytes")?;
            if count as u64>=number(&config,"max_messages")? || (bytes as u64).checked_add(encoded.len() as u64).is_none_or(|bytes|bytes>max_bytes) {
                return Err("Communication budget exhausted".into());
            }
            db.execute("INSERT INTO bus_messages(id,content) VALUES(?,?)",params![id,encoded]).map_err(|e|e.to_string())?;
            Ok(json!({"accepted":true,"duplicate":false,"sequence":db.last_insert_rowid()}))
        })
    }
    fn receive(&self, run_id: &str, recipient: &str, limit: usize) -> Result<Value, String> {
        self.database.with_transaction(|db| {
            config(db, run_id, Some(recipient))?;
            let mut query = db
                .prepare("SELECT seq,content,received FROM bus_messages WHERE acked=0 ORDER BY seq")
                .map_err(|e| e.to_string())?;
            let rows = query
                .query_map([], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, bool>(2)?,
                    ))
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;
            let mut results = Vec::new();
            for (sequence, content, redelivered) in rows {
                let message = decode(&content)?;
                if message["recipient"] == recipient && results.len() < limit {
                    results.push(
                        json!({"message":message,"sequence":sequence,"redelivered":redelivered}),
                    );
                    db.execute("UPDATE bus_messages SET received=1 WHERE seq=?", [sequence])
                        .map_err(|e| e.to_string())?;
                }
            }
            Ok(Value::Array(results))
        })
    }
    fn acknowledge(
        &self,
        run_id: &str,
        message_id: &str,
        recipient: &str,
    ) -> Result<Value, String> {
        self.database.with_transaction(|db| {
            config(db, run_id, Some(recipient))?;
            let row: Option<(String, bool)> = db
                .query_row(
                    "SELECT content,received FROM bus_messages WHERE id=?",
                    [message_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()
                .map_err(|e| e.to_string())?;
            let Some((content, true)) = row else {
                return Err("ACK requires the addressed received message".into());
            };
            if decode(&content)?["recipient"] != recipient {
                return Err("ACK requires the addressed received message".into());
            }
            db.execute("UPDATE bus_messages SET acked=1 WHERE id=?", [message_id])
                .map_err(|e| e.to_string())?;
            Ok(json!({"acknowledged":true,"message_id":message_id}))
        })
    }
    fn reserve(&self, run_id: &str, actor: &str, call_id: &str) -> Result<Value, String> {
        self.database.with_transaction(|db| {
            let config = config(db, run_id, Some(actor))?;
            let count: i64 = db
                .query_row("SELECT count(*) FROM bus_calls", [], |row| row.get(0))
                .map_err(|e| e.to_string())?;
            if count as u64 >= number(&config, "max_calls")? {
                return Err("Worker call budget exhausted".into());
            }
            db.execute(
                "INSERT INTO bus_calls(id,actor) VALUES(?,?)",
                params![call_id, actor],
            )
            .map_err(|e| e.to_string())?;
            Ok(json!({"reserved":true,"call_id":call_id}))
        })
    }
    fn finish(
        &self,
        run_id: &str,
        actor: &str,
        call_id: &str,
        evidence: &Value,
    ) -> Result<Value, String> {
        let encoded = canonical_json(evidence)?;
        self.database.with_transaction(|db| {
            config(db, run_id, Some(actor))?;
            let row: Option<(String, Option<String>)> = db
                .query_row(
                    "SELECT actor,evidence FROM bus_calls WHERE id=?",
                    [call_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()
                .map_err(|e| e.to_string())?;
            if row.as_ref().is_none_or(|(prior_actor, prior_evidence)| {
                prior_actor != actor
                    || prior_evidence
                        .as_deref()
                        .is_some_and(|prior| prior != encoded)
            }) {
                return Err("Completion identity or immutable evidence differs".into());
            }
            db.execute(
                "UPDATE bus_calls SET evidence=? WHERE id=?",
                params![encoded, call_id],
            )
            .map_err(|e| e.to_string())?;
            Ok(json!({"recorded":true}))
        })
    }
    fn status(&self, run_id: &str) -> Result<Value, String> {
        self.database.with_connection(|db| {
            let config=config(db,run_id,None)?;
            let mut query=db.prepare("SELECT seq,id,content,received,acked FROM bus_messages ORDER BY seq").map_err(|e|e.to_string())?;
            let rows=query.query_map([],|row|Ok((row.get::<_,i64>(0)?,row.get::<_,String>(1)?,row.get::<_,String>(2)?,row.get::<_,bool>(3)?,row.get::<_,bool>(4)?))).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
            let bytes:usize=rows.iter().map(|(_,_,content,_,_)|content.len()).sum();
            let messages=rows.iter().map(|(sequence,id,_,received,acked)|json!({"sequence":sequence,"message_id":id,"received":received,"acked":acked})).collect::<Vec<_>>();
            let mut query=db.prepare("SELECT id,actor,evidence FROM bus_calls").map_err(|e|e.to_string())?;
            let rows=query.query_map([],|row|Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,Option<String>>(2)?))).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
            let calls=rows.into_iter().map(|(id,actor,evidence)|Ok(json!({"id":id,"actor":actor,"state":if evidence.is_some(){"recorded"}else{"reserved"},"evidence":evidence.as_deref().map(decode).transpose()?}))).collect::<Result<Vec<_>,String>>()?;
            Ok(json!({"run_id":run_id,"config":config,"usage":{"messages":messages.len(),"bytes":bytes,"calls":calls.len()},"messages":messages,"calls":calls}))
        })
    }
}
fn config(db: &Connection, run_id: &str, actor: Option<&str>) -> Result<Value, String> {
    let text: Option<String> = db
        .query_row("SELECT content FROM bus_config WHERE id=1", [], |row| {
            row.get(0)
        })
        .optional()
        .map_err(|e| e.to_string())?;
    let value = decode(&text.ok_or("Mailbox is not initialized")?)?;
    if value["run_id"] != run_id
        || actor.is_some_and(|actor| {
            !actor.is_empty()
                && !value["participants"]
                    .as_array()
                    .is_some_and(|members| members.iter().any(|member| member == actor))
        })
    {
        return Err("Unknown run or actor".into());
    }
    Ok(value)
}
fn number(value: &Value, name: &str) -> Result<u64, String> {
    value
        .get(name)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("Invalid mailbox configuration {name}"))
}
