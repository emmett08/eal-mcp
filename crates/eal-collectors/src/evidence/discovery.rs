//! Discovery-only reductions preserve operator maxima across staged processes.
use super::{bridge, read_json};
use eal_evidence::{DiscoveryLimits, EvidenceConfig, discovery::DiscoveryReport};
use std::{collections::BTreeMap, path::PathBuf};

struct Options {
    config: PathBuf,
    configured_only: bool,
    budgets: BTreeMap<String, String>,
}
fn options(arguments: &[String]) -> Result<Options, String> {
    let mut arguments = arguments.iter();
    let mut config = None;
    let mut configured_only = false;
    let mut budgets = BTreeMap::new();
    while let Some(name) = arguments.next() {
        if name == "--configured-only" {
            if configured_only {
                return Err("evidence discovery accepts --configured-only at most once".into());
            }
            configured_only = true;
            continue;
        }
        if !matches!(
            name.as_str(),
            "--config"
                | "--request-budget"
                | "--byte-budget"
                | "--timeout-secs"
                | "--pages-per-connection"
        ) {
            return Err("evidence discovery has an unknown option".into());
        }
        let value = arguments
            .next()
            .filter(|value| !value.is_empty() && !value.starts_with("--"))
            .ok_or("evidence discovery has an incomplete option")?;
        if name == "--config" {
            if config.replace(PathBuf::from(value)).is_some() {
                return Err("evidence discovery requires one --config".into());
            }
        } else if budgets.insert(name.clone(), value.clone()).is_some() {
            return Err("evidence discovery has a duplicate budget option".into());
        }
    }
    Ok(Options {
        config: config.ok_or("evidence discovery requires --config")?,
        configured_only,
        budgets,
    })
}
fn reduced(budgets: &BTreeMap<String, String>, name: &str, maximum: u64) -> Result<u64, String> {
    let Some(value) = budgets.get(name) else {
        return Ok(maximum);
    };
    if !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!(
            "evidence discovery {name} requires a nonnegative integer"
        ));
    }
    let value = value
        .parse::<u64>()
        .map_err(|_| format!("evidence discovery {name} exceeds supported integer bounds"))?;
    if value > maximum {
        return Err(format!(
            "evidence discovery {name} cannot exceed the operator-configured bound"
        ));
    }
    Ok(value)
}
/// Read-only source discovery. Remaining budgets can only reduce operator bounds;
/// exhausted/subminimum transport allowances return candidates without source IO.
pub async fn discover(arguments: &[String]) -> Result<DiscoveryReport, String> {
    let options = options(arguments)?;
    let mut selected: EvidenceConfig = read_json(&options.config)?;
    selected.validate()?;
    let original = &selected.discovery;
    let limits = DiscoveryLimits {
        requests: reduced(
            &options.budgets,
            "--request-budget",
            original.requests as u64,
        )? as usize,
        bytes: reduced(&options.budgets, "--byte-budget", original.bytes as u64)? as usize,
        candidates: original.candidates,
        pages_per_connection: reduced(
            &options.budgets,
            "--pages-per-connection",
            original.pages_per_connection as u64,
        )? as usize,
        timeout_seconds: reduced(&options.budgets, "--timeout-secs", original.timeout_seconds)?,
    };
    if options.configured_only {
        selected.approved_catalogues.clear();
    }
    if limits.validate().is_err() {
        return DiscoveryReport::exhausted(&selected, limits);
    }
    selected.discovery = limits;
    let bridge = bridge(selected)?;
    tokio::task::spawn_blocking(move || bridge.discover())
        .await
        .map_err(|_| "evidence discovery worker failed")?
        .map_err(|error| error.message)
}
