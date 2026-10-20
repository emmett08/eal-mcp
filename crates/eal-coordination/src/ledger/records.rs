use super::*;
use rusqlite::Connection;

pub(crate) struct EffectRow {
    pub key: String,
    pub payload: String,
    pub digest: String,
    pub state: String,
    pub result: Option<String>,
    pub acknowledged: bool,
}
pub(crate) fn effect_row(db: &Connection, key: &str) -> Result<Option<EffectRow>, String> {
    db.query_row("SELECT effect_key,payload,payload_digest,state,result,acknowledged FROM effects WHERE effect_key = ?",[key],|row|Ok(EffectRow{key:row.get(0)?,payload:row.get(1)?,digest:row.get(2)?,state:row.get(3)?,result:row.get(4)?,acknowledged:row.get(5)?})).optional().map_err(|e|e.to_string())
}
pub(super) fn messages(db: &Connection, key: &str) -> Result<Vec<(String, String)>, String> {
    let mut query = db
        .prepare(
            "SELECT message_id,message FROM effect_outbox WHERE effect_key = ? ORDER BY position",
        )
        .map_err(|e| e.to_string())?;
    query
        .query_map([key], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}
pub(crate) fn snapshot(
    db: &Connection,
    row: &EffectRow,
    new: bool,
) -> Result<EffectSnapshot, String> {
    let committed = row.state == "committed";
    let status = if new {
        EffectStatus::New
    } else if committed {
        EffectStatus::Committed
    } else {
        EffectStatus::Unknown
    };
    Ok(EffectSnapshot {
        status,
        key: row.key.clone(),
        payload_digest: row.digest.clone(),
        result: if committed {
            Some(decode(
                row.result
                    .as_deref()
                    .ok_or("Committed effect has no result")?,
            )?)
        } else {
            None
        },
        outbox: messages(db, &row.key)?
            .into_iter()
            .map(|(_, message)| decode(&message))
            .collect::<Result<_, _>>()?,
        acknowledged: row.acknowledged,
        diagnosis: if new || committed {
            None
        } else {
            Some("effect_outcome_unknown".into())
        },
    })
}
