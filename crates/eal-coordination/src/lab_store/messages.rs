use super::{
    LabError, LabStore, checked_run, count_order, decode, dump, json, params, rejected, sql,
};
use crate::protocol::AdmittedMessage;
use rusqlite::OptionalExtension;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::cmp::Ordering;

impl LabStore {
    pub(super) fn send_inner(
        &self,
        admitted: &AdmittedMessage,
        actor: &str,
    ) -> Result<Value, LabError> {
        let message = admitted.message();
        let canonical = admitted.to_value();
        let text = dump(&canonical).map_err(|_| LabError::invalid_input())?;
        let fingerprint = format!("{:x}", Sha256::digest(text.as_bytes()));
        let size = text.len();
        self.transaction(|db| {
            let run = checked_run(db, &message.run_id, Some(actor), None)?;
            if message.sender != actor {
                return Err(rejected("sender differs from host worker identity"));
            }
            if !run.config["agents"]
                .as_array()
                .is_some_and(|a| a.iter().any(|v| v == &message.recipient))
            {
                return Err(rejected("unregistered recipient"));
            }
            let prior = db
                .query_row(
                    "SELECT hash,seq FROM messages WHERE run=? AND id=?",
                    params![message.run_id, message.message_id],
                    |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)),
                )
                .optional()
                .map_err(sql)?;
            if let Some((hash, sequence)) = prior {
                if hash != fingerprint {
                    return Err(rejected("message identity reused with different content"));
                }
                return Ok(json!({"accepted":true,"duplicate":true,"sequence":sequence}));
            }
            checked_run(
                db,
                &message.run_id,
                Some(actor),
                Some(self.now().map_err(|e| rejected(&e.message))?),
            )?;
            if message.version != run.active {
                return Err(rejected("inactive or incompatible protocol version"));
            }
            let next_messages = run
                .message_count
                .checked_add(1)
                .ok_or("Lab message counter overflow")?;
            let next_bytes = run
                .byte_count
                .checked_add(size.try_into().map_err(|_| "Lab message size overflow")?)
                .ok_or("Lab byte counter overflow")?;
            if count_order(next_messages, &run.config["max_messages"]) == Some(Ordering::Greater)
                || count_order(next_bytes, &run.config["max_bytes"]) == Some(Ordering::Greater)
            {
                return Err(rejected("communication budget exhausted"));
            }
            db.execute(
                "INSERT INTO messages(run,id,sender,recipient,content,hash) VALUES(?,?,?,?,?,?)",
                params![
                    message.run_id,
                    message.message_id,
                    message.sender,
                    message.recipient,
                    text,
                    fingerprint
                ],
            )
            .map_err(sql)?;
            let sequence = db.last_insert_rowid();
            db.execute(
                "UPDATE runs SET message_count=message_count+1,byte_count=byte_count+? WHERE id=?",
                params![size as i64, message.run_id],
            )
            .map_err(sql)?;
            Ok(json!({"accepted":true,"duplicate":false,"sequence":sequence}))
        })
    }
    pub(super) fn receive_inner(
        &self,
        run_id: &str,
        recipient: &str,
        limit: &Value,
    ) -> Result<Value, LabError> {
        let limit = limit
            .as_u64()
            .filter(|n| (1..=100).contains(n))
            .ok_or_else(|| LabError::rejected("limit must be 1..100"))?;
        self.transaction(|db| {
            checked_run(db,run_id,Some(recipient),None)?;
            let rows={let mut query=db.prepare("SELECT seq,content,received FROM messages WHERE run=? AND recipient=? AND acked=0 ORDER BY seq LIMIT ?").map_err(sql)?;
                query.query_map(params![run_id,recipient,limit],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)?))).map_err(sql)?.collect::<Result<Vec<_>,_>>().map_err(sql)?};
            let mut result=Vec::new();
            for(sequence,content,received)in rows {
                db.execute("UPDATE messages SET received=1 WHERE seq=?",[sequence]).map_err(sql)?;
                result.push(json!({"message":decode(&content)?,"sequence":sequence,"redelivered":received!=0}));
            }
            Ok(json!(result))
        })
    }
    pub(super) fn ack_inner(
        &self,
        run_id: &str,
        message_id: &str,
        recipient: &str,
    ) -> Result<Value, LabError> {
        self.transaction(|db| {
            checked_run(db, run_id, Some(recipient), None)?;
            let row = db
                .query_row(
                    "SELECT seq,recipient,received FROM messages WHERE run=? AND id=?",
                    params![run_id, message_id],
                    |r| {
                        Ok((
                            r.get::<_, i64>(0)?,
                            r.get::<_, String>(1)?,
                            r.get::<_, i64>(2)?,
                        ))
                    },
                )
                .optional()
                .map_err(sql)?;
            let Some((sequence, _, received)) = row.filter(|(_, owner, _)| owner == recipient)
            else {
                return Err(rejected("message not addressed to worker"));
            };
            if received == 0 {
                return Err(rejected("message has not been received"));
            }
            db.execute("UPDATE messages SET acked=1 WHERE seq=?", [sequence])
                .map_err(sql)?;
            Ok(json!({"acknowledged":true,"message_id":message_id}))
        })
    }
}
