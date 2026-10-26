//! Frozen ticket configuration and first-response receipts for ordered stages.
use super::artifacts::{self, digest};
use crate::state::State;
use eal_coordination_contracts::ticket::{Stage, StageResult, TicketSpec};
use eal_runtime::private_sqlite::ensure_immutable_file;
use serde_json::{Value, json};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

pub(super) const SCHEMA: &str = "EAL/ticket-run/1";
pub(super) struct Journal {
    pub directory: PathBuf,
    pub state: State,
    pub spec: TicketSpec,
}

impl Journal {
    pub fn create(directory: &Path, spec: TicketSpec) -> Result<Self, String> {
        if std::fs::symlink_metadata(directory).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err("ticket run directory must not be a symlink".into());
        }
        std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
        let directory = directory.canonicalize().map_err(|e| e.to_string())?;
        if directory == spec.workspace
            || std::fs::read_dir(&directory)
                .map_err(|e| e.to_string())?
                .next()
                .is_some()
        {
            return Err(
                "new ticket journal requires an empty directory distinct from its workspace".into(),
            );
        }
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut owner = options
            .open(directory.join("ticket.owner"))
            .map_err(|_| "ticket journal already has an owner")?;
        owner
            .write_all(uuid::Uuid::new_v4().to_string().as_bytes())
            .and_then(|()| owner.sync_all())
            .map_err(|e| e.to_string())?;
        let encoded = serde_json::to_vec(&spec).map_err(|e| e.to_string())?;
        ensure_immutable_file(directory.join("ticket-spec.json"), &encoded)?;
        let (program_bytes, program_digest) =
            artifacts::executable_identity(Path::new(&spec.worker_argv[0]))?;
        let state = State::new(directory.join("ticket.sqlite"))?;
        let deadline = unix_ms()?
            .checked_add(spec.timeout_seconds * 1000)
            .ok_or("ticket deadline overflow")?;
        state.initialize(&json!({"schema":SCHEMA,"spec_sha256":digest(&encoded),"worker_sha256":program_digest,"worker_bytes":program_bytes,"created_at":eal_runtime::store::utc_now(),"deadline_unix_ms":deadline}))?;
        Ok(Self {
            directory,
            state,
            spec,
        })
    }
    pub fn open(directory: &Path) -> Result<Self, String> {
        let directory = directory.canonicalize().map_err(|e| e.to_string())?;
        let database = directory.join("ticket.sqlite");
        let metadata = std::fs::symlink_metadata(&database).map_err(|e| e.to_string())?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.len() > 16 * 1024 * 1024
        {
            return Err("ticket index must be a bounded regular file".into());
        }
        let state = State::new(database)?;
        let config = state.config()?;
        let bytes = artifacts::read(&directory.join("ticket-spec.json"), 4 * 1024 * 1024)?;
        if config["schema"] != SCHEMA
            || config["spec_sha256"].as_str() != Some(digest(&bytes).as_str())
        {
            return Err("frozen ticket specification identity differs".into());
        }
        let spec = serde_json::from_value(artifacts::json(&bytes)?)
            .map_err(|e| format!("invalid frozen ticket specification: {e}"))?;
        Ok(Self {
            directory,
            state,
            spec,
        })
    }
    pub fn check_worker(&self) -> Result<(), String> {
        let config = self.state.config()?;
        let (bytes, digest) = artifacts::executable_identity(Path::new(&self.spec.worker_argv[0]))?;
        if config["worker_sha256"].as_str() != Some(digest.as_str())
            || config["worker_bytes"].as_u64() != Some(bytes)
        {
            return Err("ticket worker executable changed".into());
        }
        Ok(())
    }
    pub fn request(&self, stage: Stage, request: &Value) -> Result<Value, String> {
        let bytes = serde_json::to_vec(request).map_err(|e| e.to_string())?;
        let name = format!("{}-request.json", stage.as_str());
        ensure_immutable_file(self.directory.join(&name), &bytes)?;
        Ok(
            serde_json::json!({"ticket_id":request["ticket_id"],"stage":request["stage"],"previous":request["previous"],"context_sha256":digest(&serde_json::to_vec(&self.spec.context).map_err(|e|e.to_string())?),"request":{"path":name,"sha256":digest(&bytes),"bytes":bytes.len()}}),
        )
    }
    pub fn raw_response(&self, stage: Stage, bytes: &[u8]) -> Result<Value, String> {
        let name = format!("{}-response.bin", stage.as_str());
        ensure_immutable_file(self.directory.join(&name), bytes)?;
        Ok(json!({"path":name,"bytes":bytes.len(),"sha256":digest(bytes)}))
    }
    pub fn response(&self, stage: Stage, effect: &Value) -> Result<Option<StageResult>, String> {
        if effect["key"] != format!("stage:{}", stage.as_str()) {
            return Err("ticket stage effect identity differs".into());
        }
        if effect["status"] != "committed" {
            return Ok(None);
        }
        let call_id = format!("ticket:{}:{}", self.spec.ticket_id, stage.as_str());
        let payload = &effect["payload"];
        let reference = &payload["request"];
        let expected = format!("{}-request.json", stage.as_str());
        if reference["path"] != expected {
            return Err("ticket request reference differs".into());
        }
        let request_bytes = artifacts::read(&self.directory.join(expected), 4 * 1024 * 1024)?;
        if reference["sha256"].as_str() != Some(digest(&request_bytes).as_str())
            || reference["bytes"].as_u64() != Some(request_bytes.len() as u64)
        {
            return Err("ticket request identity differs".into());
        }
        let request: eal_coordination_contracts::ticket::StageRequest =
            serde_json::from_value(artifacts::json(&request_bytes)?).map_err(|e| e.to_string())?;
        if payload["ticket_id"] != self.spec.ticket_id
            || payload["stage"] != stage.as_str()
            || request.ticket_id != self.spec.ticket_id
            || request.stage != stage
            || request.call_id != call_id
            || request.workspace != self.spec.workspace
            || request.schema_version != eal_coordination_contracts::ticket::STAGE_REQUEST_SCHEMA
            || request.context != self.spec.context
            || serde_json::to_value(&request.previous).map_err(|e| e.to_string())?
                != payload["previous"]
            || payload["context_sha256"].as_str()
                != Some(
                    digest(&serde_json::to_vec(&self.spec.context).map_err(|e| e.to_string())?)
                        .as_str(),
                )
        {
            return Err("ticket stage reservation differs from its frozen specification".into());
        }
        let result = &effect["result"];
        let receipt = &result["raw"];
        let expected = format!("{}-response.bin", stage.as_str());
        if receipt["path"] != expected {
            return Err("ticket response frame reference differs".into());
        }
        let bytes = artifacts::read(&self.directory.join(expected), 4 * 1024 * 1024)?;
        if receipt["bytes"].as_u64() != Some(bytes.len() as u64)
            || receipt["sha256"].as_str() != Some(digest(&bytes).as_str())
        {
            return Err("ticket response frame identity differs".into());
        }
        if result["interrupted"] == true || result["rejected"] == true {
            return Ok(None);
        }
        let raw = artifacts::json(&bytes)?;
        let call_id = format!("ticket:{}:{}", self.spec.ticket_id, stage.as_str());
        let (_, execution) = crate::workers::live_response(&raw, &call_id)?;
        if execution["execution_id"] != result["execution_id"] {
            return Err("ticket execution identity differs from exact retained receipt".into());
        }
        let response: StageResult =
            serde_json::from_value(raw["response"].clone()).map_err(|e| e.to_string())?;
        if result["response_sha256"].as_str()
            != Some(digest(&serde_json::to_vec(&response).map_err(|e| e.to_string())?).as_str())
        {
            return Err("ticket receipt differs from exact retained response".into());
        }
        super::run::validate_result(&self.spec, stage, &response, &request.previous)?;
        Ok(Some(response))
    }
}

pub(super) fn key(stage: Stage) -> String {
    format!("stage:{}", stage.as_str())
}
pub(super) fn unix_ms() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "system clock precedes Unix epoch".into())
        .and_then(|d| {
            u64::try_from(d.as_millis()).map_err(|_| "system clock is not representable".into())
        })
}
