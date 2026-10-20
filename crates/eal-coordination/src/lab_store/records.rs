use super::{EXECUTION_CLAIM, checked_run, decode, json, sql};
use rusqlite::Connection;
use serde_json::Value;

pub(super) fn inspect(db: &Connection, run_id: &str) -> Result<Value, String> {
    let run = checked_run(db, run_id, None, None)?;
    let mut query = db
        .prepare(
            "SELECT id,sender,recipient,received,acked,hash FROM messages WHERE run=? ORDER BY seq",
        )
        .map_err(sql)?;
    let messages=query.query_map([run_id],|r|Ok(json!({"id":r.get::<_,String>(0)?,"sender":r.get::<_,String>(1)?,"recipient":r.get::<_,String>(2)?,"received":r.get::<_,i64>(3)?,"acked":r.get::<_,i64>(4)?,"hash":r.get::<_,String>(5)?}))).map_err(sql)?.collect::<Result<Vec<_>,_>>().map_err(sql)?;
    let mut query = db
        .prepare("SELECT version,certificate FROM protocols WHERE run=?")
        .map_err(sql)?;
    let protocols = query
        .query_map([run_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })
        .map_err(sql)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(sql)?
        .into_iter()
        .map(|(version, certificate)| {
            Ok(json!({"version":version,"certificate":decode(&certificate)?}))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut query = db
        .prepare("SELECT id,actor,state,result FROM calls WHERE run=?")
        .map_err(sql)?;
    let calls=query.query_map([run_id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,Option<String>>(3)?))).map_err(sql)?.collect::<Result<Vec<_>,_>>().map_err(sql)?.into_iter().map(|(id,actor,state,result)|Ok(json!({"id":id,"actor":actor,"state":state,"evidence":result.as_deref().map(decode).transpose()?}))).collect::<Result<Vec<_>,String>>()?;
    Ok(
        json!({"run_id":run_id,"config":run.config,"active_protocol":run.active,"deadline":run.deadline,"usage":{"messages":run.message_count,"bytes":run.byte_count,"calls":run.call_count},"messages":messages,"protocols":protocols,"calls":calls,"execution_claim":EXECUTION_CLAIM}),
    )
}
