use super::*;

impl RunStore {
    pub(super) fn initialise_schema(&self) -> Result<(), String> {
        let mut connection = self.connect()?;
        connection
            .execute_batch("PRAGMA journal_mode=WAL")
            .map_err(|e| e.to_string())?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|e| e.to_string())?;
        transaction.execute_batch("CREATE TABLE IF NOT EXISTS records (id TEXT PRIMARY KEY, kind TEXT NOT NULL, created_at TEXT NOT NULL, payload TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS records_kind ON records(kind, created_at);
CREATE TABLE IF NOT EXISTS observation_index (sequence INTEGER PRIMARY KEY, record_id TEXT NOT NULL UNIQUE, evidence_id TEXT NOT NULL, environment TEXT NOT NULL, request_digest TEXT NOT NULL, tool_binding_digest TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS observation_identity ON observation_index (evidence_id, environment, request_digest, tool_binding_digest, sequence DESC);
CREATE TABLE IF NOT EXISTS store_metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL);").map_err(|e|e.to_string())?;
        let version: Option<String> = transaction
            .query_row(
                "SELECT value FROM store_metadata WHERE key = 'observation-index-version'",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        match version.as_deref() {
            None => {
                {
                    let mut statement=transaction.prepare("SELECT id, CASE WHEN length(CAST(payload AS BLOB)) <= ? THEN payload ELSE NULL END FROM records WHERE kind = 'observation' ORDER BY rowid").map_err(|e|e.to_string())?;
                    let mut rows = statement
                        .query([MAX_STORED_RECORD_BYTES as i64])
                        .map_err(|e| e.to_string())?;
                    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
                        let id: String = row.get(0).map_err(|e| e.to_string())?;
                        let encoded: Option<String> = row.get(1).map_err(|e| e.to_string())?;
                        index_observation(
                            &transaction,
                            &id,
                            &decode(&encoded.ok_or("Stored record exceeds the byte budget")?)?,
                        )?;
                    }
                }
                transaction.execute("INSERT INTO store_metadata(key, value) VALUES ('observation-index-version', '1')",[]).map_err(|e|e.to_string())?;
            }
            Some("1") => {}
            _ => return Err("Unsupported observation index version".into()),
        }
        self.check_files()?;
        transaction.commit().map_err(|e| e.to_string())?;
        Ok(())
    }
}
pub(super) fn index_observation(
    connection: &Connection,
    id: &str,
    payload: &Value,
) -> Result<(), String> {
    let object = payload
        .as_object()
        .ok_or("An observation record must be a JSON object")?;
    if object.get("status").and_then(Value::as_str) != Some("ok") {
        return Ok(());
    }
    let keys = [
        "evidence_id",
        "environment",
        "request_digest",
        "tool_binding_digest",
    ]
    .map(|key| {
        object
            .get(key)
            .and_then(Value::as_str)
            .filter(|key| !key.is_empty())
    });
    if let [
        Some(evidence),
        Some(environment),
        Some(request),
        Some(binding),
    ] = keys
    {
        connection.execute("INSERT INTO observation_index(record_id, evidence_id, environment, request_digest, tool_binding_digest) VALUES (?, ?, ?, ?, ?)",params![id,evidence,environment,request,binding]).map_err(|e|e.to_string())?;
    }
    Ok(())
}
