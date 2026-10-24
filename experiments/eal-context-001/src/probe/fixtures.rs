//! Authored synthetic pressure scenarios and private observation setup.
use super::{HEADER, SECRET};
use serde_json::json;
use std::{fs, path::Path};
pub(super) fn source(kind: &str) -> Result<String, String> {
    let root = "claim ready {\n  statement \"The calibrated service can proceed in the bench environment.\"\n  environment lab\n}\n";
    let mut text = format!("{HEADER}{root}");
    match kind {
        "simple" => {
            text.push_str("argument route = [evidence observation] via authored => ready\n")
        }
        "count_pressure" => {
            for i in 0..40 {
                text.push_str(&format!(
                    "argument route{i:02} = [evidence observation] via authored => ready\n"
                ));
            }
        }
        "byte_pressure" => {
            for i in 0..6 {
                let sentence = format!(
                    "Calibration premise {i} applies to release {i} in the bench environment and requires the recorded acceptance predicate. "
                );
                let statement = sentence.repeat(15).chars().take(1800).collect::<String>();
                text.push_str(&format!("claim premise{i} {{\n  statement {}\n  environment lab\n}}\nargument premise_route{i} = [evidence observation] via authored => premise{i}\n",json!(statement)));
            }
            text.push_str("argument combined = [premises premise0, premise1, premise2, premise3, premise4, premise5] via authored => ready\n");
        }
        _ => return Err("Unknown diagnostic fixture".into()),
    }
    Ok(text)
}
pub(super) fn setup(workspace: &Path, kind: &str) -> Result<(), String> {
    fs::create_dir(workspace).map_err(|e| e.to_string())?;
    fs::write(workspace.join("case.eal"), source(kind)?).map_err(|e| e.to_string())?;
    fs::write(
        workspace.join("tools.toml"),
        "[tools.reader]\nkind='json_file'\nversion='1'\npath='observation.json'\n",
    )
    .map_err(|e| e.to_string())?;
    fs::write(workspace.join("observation.json"),json!({"value":{"ok":true,"private":SECRET},"observed_at":"2040-01-01T00:00:00Z","context":{"site":"bench"},"request":{"tool":"reader","tool_version":"1","input":{},"context":{"site":"bench"}}}).to_string()).map_err(|e|e.to_string())?;
    Ok(())
}
