//! Deadline and byte ownership for fixed read-only external commands.
use eal_runtime::process::{CommandRequest, Supervisor};
use std::{collections::BTreeMap, time::Instant};

pub(crate) fn run(
    config: &crate::Config,
    arguments: Vec<String>,
    started: Instant,
) -> Result<Vec<u8>, String> {
    let remaining = config
        .timeout
        .checked_sub(started.elapsed())
        .filter(|v| !v.is_zero())
        .ok_or("Collector deadline exhausted")?;
    let mut environment = BTreeMap::new();
    for name in &config.inherit_env {
        if let Some(value) = std::env::var_os(name) {
            environment.insert(
                name.clone(),
                value
                    .into_string()
                    .map_err(|_| "Inherited environment must be Unicode")?,
            );
        }
    }
    let mut argv = vec![config.path("--executable")?.to_string_lossy().into_owned()];
    argv.extend(arguments);
    let result = Supervisor::new(config.path("--supervisor")?)
        .run(
            CommandRequest {
                argv,
                environment,
                cwd: std::env::current_dir().map_err(|_| "Cannot resolve collector workspace")?,
                input: Vec::new(),
                timeout: remaining,
                max_output_bytes: config.max_bytes,
            },
            None,
        )
        .map_err(|_| "External read supervisor failed")?;
    if result.timed_out {
        return Err("External read failed with timeout".into());
    }
    if let Some(error) = result.execution_error {
        return Err(format!("External read failed with {}", error.label()));
    }
    if result.returncode != 0 {
        return Err(format!(
            "External read exited with status {}",
            result.returncode
        ));
    }
    Ok(result.stdout)
}
pub(crate) fn object(bytes: &[u8]) -> Result<serde_json::Value, String> {
    let value = eal_runtime::acquisition::strict_json(
        std::str::from_utf8(bytes).map_err(|_| "External response requires UTF-8")?,
    )
    .map_err(|_| "External response must be strict JSON")?;
    if !value.is_object() {
        return Err("External response must be a JSON object".into());
    }
    Ok(value)
}
