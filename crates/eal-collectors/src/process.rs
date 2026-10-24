//! Deadline and byte ownership for fixed read-only external commands.
use eal_process::process::{CommandRequest, Supervisor};
use std::{collections::BTreeMap, time::Instant};

pub(crate) fn run(
    config: &crate::Config,
    arguments: Vec<String>,
    started: Instant,
) -> Result<Vec<u8>, String> {
    let remaining = config
        .limits()
        .timeout()
        .checked_sub(started.elapsed())
        .filter(|v| !v.is_zero())
        .ok_or("Collector deadline exhausted")?;
    let mut environment = BTreeMap::new();
    let command = config
        .selected()
        .command()
        .ok_or("The selected provider does not execute commands")?;
    for name in &command.inherit_env {
        if let Some(value) = std::env::var_os(name) {
            environment.insert(
                name.clone(),
                value
                    .into_string()
                    .map_err(|_| "Inherited environment must be Unicode")?,
            );
        }
    }
    let mut argv = vec![command.executable.clone().to_string_lossy().into_owned()];
    argv.extend(arguments);
    let result = Supervisor::new(command.supervisor.clone())
        .run(
            CommandRequest {
                argv,
                environment,
                cwd: std::env::current_dir().map_err(|_| "Cannot resolve collector workspace")?,
                input: Vec::new(),
                timeout: remaining,
                max_output_bytes: config.limits().max_bytes(),
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
    let value =
        eal_collector_contracts::strict_json(bytes, eal_collector_contracts::JsonLimits::default())
            .map_err(|_| "External response must be strict JSON")?;
    if !value.is_object() {
        return Err("External response must be a JSON object".into());
    }
    Ok(value)
}
