use super::*;

struct EntryRow {
    identifier: String,
    path: String,
    label: String,
    context: String,
    selected: Option<String>,
    current_digest: String,
    current_method: String,
    digest: String,
    source: String,
    method: String,
    claims: String,
    metadata: String,
    registered_at: String,
}
impl WorkspaceKnowledgeCatalogue {
    fn entry(
        &self,
        entry_id: &str,
        source_digest: Option<&str>,
        method: Option<&str>,
        include_source: bool,
    ) -> Result<Value, String> {
        let row=self.service.store().with_connection(|connection|{
            connection.query_row("SELECT e.entry_id, e.path, e.label, e.context_json, e.selected_claims_json, e.current_digest, e.current_method_fingerprint, r.source_digest, r.source, r.method_registry_fingerprint, r.claims_json, r.claim_metadata_json, r.registered_at FROM catalogue_entries e JOIN catalogue_revisions r ON r.entry_id = e.entry_id AND r.source_digest = COALESCE(?, e.current_digest) WHERE e.entry_id = ? AND (? IS NULL OR r.method_registry_fingerprint = ?) ORDER BY (r.source_digest = e.current_digest AND r.method_registry_fingerprint = e.current_method_fingerprint) DESC, r.registered_at DESC LIMIT 1",params![source_digest,entry_id,method,method],|row|Ok(EntryRow{identifier:row.get(0)?,path:row.get(1)?,label:row.get(2)?,context:row.get(3)?,selected:row.get(4)?,current_digest:row.get(5)?,current_method:row.get(6)?,digest:row.get(7)?,source:row.get(8)?,method:row.get(9)?,claims:row.get(10)?,metadata:row.get(11)?,registered_at:row.get(12)?})).optional().map_err(|e|e.to_string())
        })?.ok_or(format!("Unknown catalogue entry or source revision {}",repr(entry_id)?))?;
        let mut result = json!({"entry_id":row.identifier,"path":row.path,"label":row.label,"source_digest":row.digest,"current":row.digest==row.current_digest&&row.method==row.current_method,"method_registry_fingerprint":row.method,"context":stored_json(&row.context)?,"claims":stored_json(&row.claims)?,"selected_claims":row.selected.as_deref().map(stored_json).transpose()?,"claim_metadata":stored_json(&row.metadata)?,"registered_at":row.registered_at});
        if include_source {
            result["source"] = json!(row.source);
        }
        Ok(result)
    }
    pub fn get(&self, entry_id: &str, include_source: bool) -> Result<Value, String> {
        self.get_with_refresh(entry_id, include_source, true)
    }
    pub fn get_with_refresh(
        &self,
        entry_id: &str,
        include_source: bool,
        refresh: bool,
    ) -> Result<Value, String> {
        if refresh {
            let current = self.entry(entry_id, None, None, false)?;
            let claims = current["selected_claims"]
                .as_array()
                .map(|claims| {
                    claims
                        .iter()
                        .map(|claim| {
                            claim
                                .as_str()
                                .map(str::to_owned)
                                .ok_or("Invalid stored selected claim")
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?;
            self.register_revision(
                current["path"]
                    .as_str()
                    .ok_or("Invalid stored catalogue path")?,
                RegisterOptions {
                    entry_id: Some(entry_id),
                    context: Some(&current["context"]),
                    claims: claims.as_deref(),
                    label: current["label"].as_str(),
                },
                true,
            )?;
        }
        self.entry(entry_id, None, None, include_source)
    }
    pub fn get_revision(
        &self,
        entry_id: &str,
        source_digest: &str,
        method_registry_fingerprint: Option<&str>,
        include_source: bool,
    ) -> Result<Value, String> {
        if !digest(source_digest) {
            return Err("source_digest must be a lowercase SHA-256 digest".into());
        }
        if method_registry_fingerprint.is_some_and(|fingerprint| !digest(fingerprint)) {
            return Err("method_registry_fingerprint must be a lowercase SHA-256 digest".into());
        }
        self.entry(
            entry_id,
            Some(source_digest),
            method_registry_fingerprint,
            include_source,
        )
    }
    pub fn revisions(&self, entry_id: &str, limit: usize) -> Result<Vec<Value>, String> {
        check_limit(limit)?;
        let (current,rows)=self.service.store().with_connection(|connection|{
            let current:Option<(String,String)>=connection.query_row("SELECT current_digest, current_method_fingerprint FROM catalogue_entries WHERE entry_id = ?",[entry_id],|row|Ok((row.get(0)?,row.get(1)?))).optional().map_err(|e|e.to_string())?;
            let current=current.ok_or(format!("Unknown catalogue entry {}",repr(entry_id)?))?;
            let mut statement=connection.prepare("SELECT source_digest, method_registry_fingerprint, registered_at, claims_json FROM catalogue_revisions WHERE entry_id = ? ORDER BY registered_at DESC, source_digest LIMIT ?").map_err(|e|e.to_string())?;
            let rows=statement.query_map(params![entry_id,limit as i64],|row|Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,String>(2)?,row.get::<_,String>(3)?))).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;Ok((current,rows))
        })?;
        rows.into_iter().map(|(digest,method,registered_at,claims)|Ok(json!({"source_digest":digest,"method_registry_fingerprint":method,"registered_at":registered_at,"claims":stored_json(&claims)?,"current":(digest,method)==current}))).collect()
    }
    pub fn list(&self, limit: usize, offset: usize) -> Result<Vec<Value>, String> {
        check_limit(limit)?;
        if offset > 10_000 {
            return Err("offset must be an integer from 0 through 10000".into());
        }
        let names = self.service.store().with_connection(|connection| {
            let mut statement = connection
                .prepare("SELECT entry_id FROM catalogue_entries ORDER BY path LIMIT ? OFFSET ?")
                .map_err(|e| e.to_string())?;
            statement
                .query_map(params![limit as i64, offset as i64], |row| {
                    row.get::<_, String>(0)
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())
        })?;
        names
            .into_iter()
            .map(|name| self.entry(&name, None, None, false))
            .collect()
    }
    pub fn find(&self, options: FindOptions<'_>) -> Result<Vec<Value>, String> {
        check_limit(options.limit)?;
        if options
            .query
            .is_some_and(|query| query.chars().count() > 256)
        {
            return Err("query must be at most 256 characters".into());
        }
        let context = options
            .context
            .map(|context| {
                if !context.is_object() {
                    return Err("context must be a JSON object".into());
                }
                finite_json(context)
            })
            .transpose()?;
        let names=self.service.store().with_connection(|connection|{
            let mut statement=connection.prepare("SELECT e.entry_id FROM catalogue_entries e JOIN catalogue_revisions r ON r.entry_id = e.entry_id AND r.source_digest = e.current_digest AND r.method_registry_fingerprint = e.current_method_fingerprint WHERE (? IS NULL OR instr(lower(e.path || ' ' || e.label || ' ' || r.claim_metadata_json), lower(?)) > 0) AND (? IS NULL OR EXISTS (SELECT 1 FROM json_each(r.claims_json) WHERE value = ?)) AND (? IS NULL OR e.context_json = ?) AND (? IS NULL OR e.path = ?) ORDER BY e.path LIMIT ?").map_err(|e|e.to_string())?;
            statement.query_map(params![options.query,options.query,options.claim,options.claim,context,context,options.path,options.path,options.limit as i64],|row|row.get::<_,String>(0)).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())
        })?;
        names
            .into_iter()
            .map(|name| self.entry(&name, None, None, false))
            .collect()
    }
    pub fn runs(
        &self,
        entry_id: &str,
        source_digest: Option<&str>,
        limit: usize,
    ) -> Result<Value, String> {
        check_limit(limit)?;
        let entry = match source_digest {
            Some(digest) => self.get_revision(entry_id, digest, None, false)?,
            None => self.get(entry_id, false)?,
        };
        let digest = entry["source_digest"]
            .as_str()
            .ok_or("Invalid catalogue source identity")?;
        let context = finite_json(&entry["context"])?;
        let context_fingerprint =
            eal_core::canonical::canonical_digest(&entry["context"], self.service.limits())?;
        let (collections,assessments)=self.service.store().with_connection(|connection|{
            let query=|sql:&str,context:&str|->Result<Vec<(String,String,String)>,String>{let mut statement=connection.prepare(sql).map_err(|e|e.to_string())?;statement.query_map(params![digest,context,limit as i64],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?))).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())};
            Ok((query("SELECT id, created_at, payload FROM records WHERE kind = 'collection' AND json_extract(payload, '$.source_digest') = ? AND json_extract(payload, '$.context') = ? ORDER BY rowid DESC LIMIT ?",&context)?,query("SELECT id, created_at, payload FROM records WHERE kind = 'assessment' AND json_extract(payload, '$.source_digest') = ? AND json_extract(payload, '$.context_fingerprint') = ? ORDER BY rowid DESC LIMIT ?",&context_fingerprint)?))
        })?;
        let collections = collections
            .into_iter()
            .map(|(id, created_at, payload)| {
                let payload = stored_json(&payload)?;
                let count = payload
                    .get("records")
                    .and_then(|records| {
                        records
                            .as_object()
                            .map(serde_json::Map::len)
                            .or_else(|| records.as_array().map(Vec::len))
                    })
                    .unwrap_or(0);
                Ok(json!({"collection_id":id,"created_at":created_at,"record_count":count}))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let selected = entry["claims"]
            .as_array()
            .ok_or("Invalid catalogue claim selection")?;
        let assessments=assessments.into_iter().map(|(id,created_at,payload)|{let payload=stored_json(&payload)?;let mut statuses=json!({});if let Some(claims)=payload["claims"].as_object(){for (name,claim) in claims{if selected.iter().any(|selected|selected==name){statuses[name]=claim.get("status").cloned().unwrap_or(Value::Null);}}}Ok(json!({"assessment_id":id,"collection_id":payload["collection_id"],"created_at":created_at,"assessed_at":payload["assessed_at"],"method_registry_fingerprint":payload["method_registry_fingerprint"],"claim_statuses":statuses}))}).collect::<Result<Vec<_>,String>>()?;
        Ok(
            json!({"entry_id":entry_id,"source_digest":digest,"collections":collections,"assessments":assessments}),
        )
    }
}
