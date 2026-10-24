//! Native, provider-independent EAL-CONTEXT-001 diagnostics.
pub mod history;
pub mod planning;
#[path = "probe/mod.rs"]
pub mod probe;
pub mod process;
pub mod rehearse;

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// New evidence is written only to an explicitly selected, previously absent path.
pub fn save_report(path: &Path, report: &Value) -> Result<(), String> {
    use std::io::Write;
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("Cannot create new report {}: {e}", path.display()))?;
    serde_json::to_writer_pretty(&mut file, report).map_err(|e| e.to_string())?;
    file.write_all(b"\n").map_err(|e| e.to_string())?;
    Ok(())
}
