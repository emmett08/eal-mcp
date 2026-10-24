//! Generate checked views from explicit theory and authored EAL scenarios.
use super::Result;
use super::input::required_string;
use super::process::Runner;
use serde_json::{Value, json};
use std::{
    fs,
    path::{Component, Path},
};
pub(super) fn put(work: &Path, name: &str, value: &Value) -> Result<()> {
    let relative = Path::new(name);
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(format!(
            "Fixture path escapes temporary workspace: {name:?}"
        ));
    }
    let path = work.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let bytes = match value.as_str() {
        Some(text) => text.as_bytes().to_vec(),
        None => serde_json::to_vec(value).map_err(|e| e.to_string())?,
    };
    fs::write(path, bytes).map_err(|e| e.to_string())
}
fn flags(work: &Path, case: &Value) -> Result<Vec<String>> {
    let Some(limits) = case.get("limits") else {
        return Ok(Vec::new());
    };
    let limits = limits
        .as_object()
        .ok_or("Fixture limits must be an object")?;
    if limits.is_empty() {
        return Ok(Vec::new());
    }
    let mut text = "[limits]\n".to_owned();
    for (key, value) in limits {
        if !key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') || value.as_u64().is_none()
        {
            return Err("Fixture limits require integer values and identifier keys".into());
        }
        text.push_str(&format!("{key} = {value}\n"));
    }
    put(work, "limits.toml", &Value::String(text))?;
    Ok(vec!["--limits".into(), "limits.toml".into()])
}
fn export(
    runner: &Runner,
    binary: &Path,
    work: &Path,
    name: String,
    result: Value,
    selected: &[String],
) -> Result<Value> {
    put(work, "result.json", &result)?;
    let args = selected
        .iter()
        .cloned()
        .chain(["export-aspic".into(), "result.json".into()])
        .collect::<Vec<_>>();
    let view = runner.core(binary, work, &args)?;
    Ok(json!({"name":name,"view":view}))
}
pub(super) fn generated_views(
    runner: &Runner,
    binary: &Path,
    work: &Path,
    explicit: &[Value],
    compiled: &[Value],
) -> Result<Vec<Value>> {
    fs::create_dir(work).map_err(|e| e.to_string())?;
    let mut views = Vec::with_capacity(explicit.len() + compiled.len());
    for case in explicit {
        let selected = flags(work, case)?;
        put(
            work,
            "theory.json",
            case.get("payload").ok_or("Fixture payload is missing")?,
        )?;
        let args = selected
            .iter()
            .cloned()
            .chain(["aspic".into(), "theory.json".into()])
            .collect::<Vec<_>>();
        let result = runner.core(binary, work, &args)?;
        views.push(export(
            runner,
            binary,
            work,
            format!("explicit-{}", required_string(case, "name")?),
            result,
            &selected,
        )?);
    }
    for case in compiled {
        let selected = flags(work, case)?;
        put(
            work,
            "source.eal",
            case.get("source").ok_or("Fixture source is missing")?,
        )?;
        put(
            work,
            "records.json",
            case.get("records").ok_or("Fixture records are missing")?,
        )?;
        if let Some(imports) = case.get("imports") {
            for (name, source) in imports
                .as_object()
                .ok_or("Fixture imports must be an object")?
            {
                put(work, name, source)?;
            }
        }
        let mut args = selected.clone();
        args.extend([
            "compile-aspic".into(),
            "source.eal".into(),
            "--claim".into(),
            required_string(case, "goal")?.into(),
            "--now".into(),
            required_string(case, "now")?.into(),
            "--context".into(),
            serde_json::to_string(case.get("context").ok_or("Fixture context is missing")?)
                .map_err(|e| e.to_string())?,
            "--records".into(),
            "records.json".into(),
        ]);
        for (key, option) in [
            ("semantics", "--semantics"),
            ("query_mode", "--query-mode"),
            ("preference", "--preference"),
        ] {
            if let Some(value) = case.get(key).filter(|v| !v.is_null()) {
                args.extend([
                    option.into(),
                    value.as_str().ok_or("Invalid solver option")?.into(),
                ]);
            }
        }
        let result = runner.core(binary, work, &args)?;
        views.push(export(
            runner,
            binary,
            work,
            format!("compiled-{}", required_string(case, "name")?),
            result,
            &selected,
        )?);
    }
    Ok(views)
}
