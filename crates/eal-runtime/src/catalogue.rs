//! Persistent named sources and immutable source/method revisions. Historical
//! run metadata never stands in for a fresh observation or assessment.
use crate::{service::ReasoningService, store::utc_now};
use rusqlite::{OptionalExtension, params};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::Arc,
};

mod discovery;
mod queries;

pub const MAX_CLAIMS: usize = 256;
pub const MAX_TREE_FILES: usize = 512;
const MAX_CONTEXT_BYTES: usize = 8192;

#[derive(Clone)]
pub struct WorkspaceKnowledgeCatalogue {
    service: Arc<ReasoningService>,
}
#[derive(Clone, Copy, Default)]
pub struct RegisterOptions<'a> {
    pub entry_id: Option<&'a str>,
    pub context: Option<&'a Value>,
    pub claims: Option<&'a [String]>,
    pub label: Option<&'a str>,
}
#[derive(Clone, Copy)]
pub struct FindOptions<'a> {
    pub query: Option<&'a str>,
    pub claim: Option<&'a str>,
    pub context: Option<&'a Value>,
    pub path: Option<&'a str>,
    pub limit: usize,
}
impl Default for FindOptions<'_> {
    fn default() -> Self {
        Self {
            query: None,
            claim: None,
            context: None,
            path: None,
            limit: 50,
        }
    }
}

impl WorkspaceKnowledgeCatalogue {
    pub fn new(service: Arc<ReasoningService>) -> Result<Self, String> {
        service.store().with_transaction(|connection|{
            connection.execute_batch("CREATE TABLE IF NOT EXISTS catalogue_entries (entry_id TEXT PRIMARY KEY, path TEXT NOT NULL UNIQUE, label TEXT NOT NULL, context_json TEXT NOT NULL, selected_claims_json TEXT, current_digest TEXT NOT NULL, current_method_fingerprint TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS catalogue_revisions (entry_id TEXT NOT NULL, source_digest TEXT NOT NULL, source TEXT NOT NULL, method_registry_fingerprint TEXT NOT NULL, claims_json TEXT NOT NULL, claim_metadata_json TEXT NOT NULL, registered_at TEXT NOT NULL, PRIMARY KEY(entry_id, source_digest, method_registry_fingerprint), FOREIGN KEY(entry_id) REFERENCES catalogue_entries(entry_id));
CREATE INDEX IF NOT EXISTS catalogue_run_source ON records (kind, json_extract(payload, '$.source_digest'));").map_err(|e|e.to_string())?;Ok(())
        })?;
        Ok(Self { service })
    }
    pub fn service(&self) -> &Arc<ReasoningService> {
        &self.service
    }
    pub fn register(&self, path: &str, options: RegisterOptions<'_>) -> Result<Value, String> {
        self.register_revision(path, options, false)
    }
    fn register_revision(
        &self,
        path: &str,
        options: RegisterOptions<'_>,
        refresh_existing: bool,
    ) -> Result<Value, String> {
        let relative = self.normal_path(path)?;
        let identifier = options.entry_id.unwrap_or(&relative);
        if identifier.is_empty() || identifier.chars().count() > 256 {
            return Err("entry_id must be a bounded nonempty string".into());
        }
        if !refresh_existing && options.entry_id.is_some() && !entry_name(identifier) {
            return Err("Explicit entry_id must be a simple developer name".into());
        }
        let stem = Path::new(&relative)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or("label must be valid UTF-8")?;
        let label = options.label.unwrap_or(stem);
        if label.is_empty()
            || label.chars().count() > 128
            || label
                .trim_matches(|character: char| {
                    character.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&character)
                })
                .is_empty()
        {
            return Err("label must contain one to 128 characters".into());
        }
        let empty = json!({});
        let context = options.context.unwrap_or(&empty);
        if !context.is_object() {
            return Err("context must be a JSON object".into());
        }
        let context_json = finite_json(context)?;
        if context_json.len() > MAX_CONTEXT_BYTES {
            return Err("context exceeds 8192 UTF-8 bytes".into());
        }
        if let Some(claims) = options.claims
            && (claims.is_empty()
                || claims.len() > MAX_CLAIMS
                || claims.iter().collect::<BTreeSet<_>>().len() != claims.len())
        {
            return Err("claims must contain unique claim identifiers".into());
        }
        let selected_json = options
            .claims
            .map(|claims| finite_json(&json!(claims)))
            .transpose()?;
        let source = self.read_source(&relative)?;
        let (digest, fingerprint, metadata) = self.validate_revision(&source, options.claims)?;
        let claims = metadata
            .as_object()
            .ok_or("Invalid catalogue claim metadata")?
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        let claims_json = finite_json(&json!(claims))?;
        let metadata_json = finite_json(&metadata)?;
        let now = utc_now();
        self.service.store().with_transaction(|connection|{
            let row:Option<(String,String,String,Option<String>)>=connection.query_row("SELECT path, label, context_json, selected_claims_json FROM catalogue_entries WHERE entry_id = ?",[identifier],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?))).optional().map_err(|e|e.to_string())?;
            let expected=(relative.clone(),label.to_owned(),context_json.clone(),selected_json.clone());
            if row.as_ref().is_some_and(|row|row!=&expected){return Err("Registered entry metadata differs; use another developer entry_id".into());}
            let owner:Option<String>=connection.query_row("SELECT entry_id FROM catalogue_entries WHERE path = ?",[&relative],|row|row.get(0)).optional().map_err(|e|e.to_string())?;
            if owner.as_deref().is_some_and(|owner|owner!=identifier){return Err("Source path is already registered under another entry_id".into());}
            if row.is_none(){connection.execute("INSERT INTO catalogue_entries VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",params![identifier,relative,label,context_json,selected_json,digest,fingerprint,now,now]).map_err(|e|e.to_string())?;}else{connection.execute("UPDATE catalogue_entries SET current_digest = ?, current_method_fingerprint = ?, updated_at = ? WHERE entry_id = ? AND (current_digest != ? OR current_method_fingerprint != ?)",params![digest,fingerprint,now,identifier,digest,fingerprint]).map_err(|e|e.to_string())?;}
            connection.execute("INSERT OR IGNORE INTO catalogue_revisions VALUES (?, ?, ?, ?, ?, ?, ?)",params![identifier,digest,source,fingerprint,claims_json,metadata_json,now]).map_err(|e|e.to_string())?;Ok(())
        })?;
        self.get_with_refresh(identifier, false, false)
    }
    fn validate_revision(
        &self,
        source: &str,
        selected: Option<&[String]>,
    ) -> Result<(String, String, Value), String> {
        let validation = self.service.validate(source);
        if validation["valid"] != true {
            let messages = validation["diagnostics"]
                .as_array()
                .ok_or("Invalid EAL validation response")?
                .iter()
                .take(5)
                .map(|item| item["message"].as_str().unwrap_or("Invalid EAL source"))
                .collect::<Vec<_>>();
            return Err(format!(
                "Registered source failed EAL validation: {}",
                messages.join("; ")
            ));
        }
        let program = self.service.parse(source)?;
        let names = selected.map_or_else(
            || program.claims.keys().cloned().collect::<Vec<_>>(),
            |selected| selected.to_vec(),
        );
        let missing = names
            .iter()
            .filter(|name| !program.claims.contains_key(*name))
            .collect::<BTreeSet<_>>();
        if !missing.is_empty() {
            return Err(format!(
                "Selected claims are absent from the source: {}",
                missing
                    .into_iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if names.is_empty() || names.len() > MAX_CLAIMS {
            return Err(format!("Register one to {MAX_CLAIMS} selected claims"));
        }
        let mut metadata = json!({});
        for name in names {
            let claim = &program.claims[&name];
            metadata[&name] = json!({"statement":claim.statement,"environment":claim.environment});
        }
        if validation["source_digest"] != program.source_digest {
            return Err("Source changed during validation".into());
        }
        Ok((
            program.source_digest,
            validation["method_registry_fingerprint"]
                .as_str()
                .ok_or("Missing validated method registry fingerprint")?
                .to_owned(),
            metadata,
        ))
    }
    pub fn read(&self, entry_id: &str) -> Result<String, String> {
        self.get(entry_id, true)?["source"]
            .as_str()
            .map(str::to_owned)
            .ok_or("Missing catalogue source".into())
    }
}
fn entry_name(value: &str) -> bool {
    let bytes = value.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= 128
        && (bytes[0].is_ascii_alphabetic() || bytes[0] == b'_')
        && bytes
            .iter()
            .skip(1)
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-'))
}
fn check_limit(limit: usize) -> Result<(), String> {
    if (1..=100).contains(&limit) {
        Ok(())
    } else {
        Err("limit must be an integer from 1 through 100".into())
    }
}
fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
fn finite_json(value: &Value) -> Result<String, String> {
    let limits = eal_core::ExecutionLimits {
        json_depth: eal_core::json::MAX_JSON_DEPTH,
        json_nodes: crate::store::MAX_STORED_RECORD_BYTES,
        record_bytes: crate::store::MAX_STORED_RECORD_BYTES,
        ..eal_core::ExecutionLimits::default()
    };
    eal_core::canonical::canonical_json_bytes(value, &limits)
        .and_then(|bytes| String::from_utf8(bytes).map_err(|e| e.to_string()))
        .map_err(|_| "Catalogue values must be finite UTF-8 JSON".into())
}
fn stored_json(value: &str) -> Result<Value, String> {
    eal_core::json::parse(
        value,
        &eal_core::ExecutionLimits {
            record_bytes: crate::store::MAX_STORED_RECORD_BYTES,
            json_nodes: crate::store::MAX_STORED_RECORD_BYTES,
            json_depth: eal_core::json::MAX_JSON_DEPTH,
            ..eal_core::ExecutionLimits::default()
        },
    )
    .map_err(|e| format!("Invalid catalogue JSON: {e}"))
}
fn repr(value: &str) -> Result<String, String> {
    eal_core::expressions::diagnostic_repr(&json!(value), &eal_core::ExecutionLimits::default())
}
