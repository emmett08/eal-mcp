//! Bounded native CLI/MCP access and operator-only record inspection.
use super::{SECRET, ensure};
use crate::{process::execute, sha256};
use serde::Deserialize;
use serde_json::{Value, json, value::RawValue};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    path::{Path, PathBuf},
};
pub(super) struct Runtime {
    bin: PathBuf,
    pub(super) workspace: PathBuf,
}
impl Runtime {
    pub(super) fn new(root: &Path, workspace: PathBuf) -> Self {
        Self {
            bin: root.join("target/debug"),
            workspace,
        }
    }
    pub(super) fn cli(&self, args: &[&str]) -> Result<Value, String> {
        let mut argv = vec![
            OsString::from("--workspace"),
            self.workspace.clone().into_os_string(),
            "--registry".into(),
            self.workspace.join("tools.toml").into_os_string(),
        ];
        argv.extend(args.iter().map(OsString::from));
        let result = execute(
            &self.bin.join("eal"),
            &argv,
            None,
            Some(&self.workspace),
            true,
            0,
        )?;
        serde_json::from_slice(&result.stdout).map_err(|e| format!("Invalid native CLI JSON: {e}"))
    }
    pub(super) fn register(&self) -> Result<Value, String> {
        self.cli(&[
            "register",
            "case.eal",
            "--entry-id",
            "demo",
            "--claim",
            "ready",
            "--context",
            "{\"site\":\"bench\"}",
        ])
    }
    pub(super) fn assess(&self, now: &str) -> Result<Value, String> {
        self.cli(&["assess-known", "demo", "--claim", "ready", "--now", now])
    }
    pub(super) fn host(&self, request: &Value, expected: i32) -> Result<(Value, usize), String> {
        let argv = vec![
            "--workspace".into(),
            self.workspace.clone().into_os_string(),
            "--registry".into(),
            self.workspace.join("tools.toml").into_os_string(),
            "--exposure".into(),
            "registered".into(),
            "--known-entry".into(),
            "demo".into(),
        ];
        let result = execute(
            &self.bin.join("eal-host"),
            &argv,
            Some(&serde_json::to_vec(request).map_err(|e| e.to_string())?),
            Some(&self.workspace),
            true,
            expected,
        )?;
        let value = serde_json::from_slice(&result.stdout)
            .map_err(|e| format!("Invalid native host JSON: {e}"))?;
        Ok((value, result.stdout.len()))
    }
    pub(super) fn retrieve(
        &self,
        id: &str,
        max_bytes: usize,
        max_statement_bytes: usize,
    ) -> Result<(Value, Value), String> {
        let (response,host_bytes)=self.host(&json!({"operation":"get_known_packet","entry_id":"demo","assessment_id":id,"claim":"ready","packet_limits":{"max_bytes":max_bytes,"max_statement_bytes":max_statement_bytes}}),0)?;
        let registered = response.get("result").ok_or("registered result absent")?;
        let raw = response["content"][0]["text"]
            .as_str()
            .ok_or("raw MCP content absent")?;
        #[derive(Deserialize)]
        struct PacketEnvelope {
            packet: Box<RawValue>,
        }
        let envelope: PacketEnvelope = serde_json::from_str(raw).map_err(|e| e.to_string())?;
        let packet: Value =
            serde_json::from_str(envelope.packet.get()).map_err(|e| e.to_string())?;
        ensure(
            packet == registered["packet"],
            "raw packet differs from registered result",
        )?;
        ensure(
            envelope.packet.get().len() <= max_bytes,
            "packet exceeds requested byte limit",
        )?;
        ensure(
            !raw.contains(SECRET),
            "private observation appears in MCP packet",
        )?;
        let reading = json!({"max_bytes":max_bytes,"max_statement_bytes":max_statement_bytes,"packet_utf8_bytes":envelope.packet.get().len(),"registered_envelope_utf8_bytes":raw.len(),"host_inspection_envelope_utf8_bytes":host_bytes,"packet_sha256":sha256(envelope.packet.get().as_bytes()),"summary_complete":packet["summary_complete"],"omitted":packet["omitted"],"claims":object_size(&packet["claims"]),"premise_claims":object_size(&packet["premise_claims"]),"arguments":object_size(&packet["arguments"])});
        Ok((registered.clone(), reading))
    }
}
fn object_size(value: &Value) -> usize {
    value.as_object().map(|map| map.len()).unwrap_or(0)
}
pub(super) fn records(workspace: &Path) -> Result<Value, String> {
    let database = rusqlite::Connection::open_with_flags(
        workspace.join(".eal/runs.sqlite3"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .map_err(|e| e.to_string())?;
    let mut statement = database
        .prepare("SELECT kind,count(*) FROM records GROUP BY kind")
        .map_err(|e| e.to_string())?;
    let pairs = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<BTreeMap<_, _>, _>>()
        .map_err(|e| e.to_string())?;
    serde_json::to_value(pairs).map_err(|e| e.to_string())
}
pub(super) fn normalise_packet(packet: &Value) -> Value {
    let mut replacements = BTreeMap::new();
    for (key, replacement) in [
        ("assessment_id", "<ASSESSMENT>"),
        ("collection_id", "<COLLECTION>"),
    ] {
        if let Some(id) = packet[key].as_str() {
            replacements.insert(id.to_owned(), replacement.to_owned());
        }
    }
    if let Some(evidence) = packet["evidence"].as_object() {
        for (name, item) in evidence {
            if let Some(id) = item["observation_id"].as_str() {
                replacements.insert(id.to_owned(), format!("<OBSERVATION:{name}>"));
            }
        }
    }
    pub(super) fn visit(value: &Value, replacements: &BTreeMap<String, String>) -> Value {
        match value {
            Value::String(text) => Value::String(replacements.get(text).unwrap_or(text).clone()),
            Value::Array(items) => {
                Value::Array(items.iter().map(|item| visit(item, replacements)).collect())
            }
            Value::Object(items) => Value::Object(
                items
                    .iter()
                    .map(|(key, item)| (key.clone(), visit(item, replacements)))
                    .collect(),
            ),
            value => value.clone(),
        }
    }
    visit(packet, &replacements)
}
