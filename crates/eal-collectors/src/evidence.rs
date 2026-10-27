//! Bounded optional discovery and provider preparation commands.
use eal_collector_contracts::{JsonLimits, strict_json_exact};
use eal_evidence::{EvidenceConfig, providers::EvidenceSources};
use serde::de::DeserializeOwned;
use std::{
    collections::BTreeMap,
    io::Read,
    path::{Path, PathBuf},
};

const FILE_BYTES: usize = 16 * 1024 * 1024;
fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, String> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(rustix::fs::OFlags::NONBLOCK.bits() as i32);
    }
    // Inspect the opened descriptor before reading. Nonblocking open prevents a
    // FIFO at an operator path from waiting for a writer before this check.
    let file = options
        .open(path)
        .map_err(|_| "evidence configuration is unavailable")?;
    if !file
        .metadata()
        .map_err(|_| "evidence file metadata is unavailable")?
        .is_file()
    {
        return Err("evidence configuration requires a regular file".into());
    }
    let mut bytes = Vec::new();
    file.take(FILE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "evidence file could not be read")?;
    let value = strict_json_exact(
        &bytes,
        JsonLimits {
            max_bytes: FILE_BYTES,
            ..Default::default()
        },
    )?;
    serde_json::from_value(value).map_err(|_| "evidence configuration fields are invalid".into())
}
fn flags(arguments: &[String], expected: &[&str]) -> Result<BTreeMap<String, PathBuf>, String> {
    flags_with_optional(arguments, expected, &[])
}
fn flags_with_optional(
    arguments: &[String],
    expected: &[&str],
    optional: &[&str],
) -> Result<BTreeMap<String, PathBuf>, String> {
    let mut flags = BTreeMap::new();
    for pair in arguments.chunks(2) {
        if pair.len() != 2
            || !(expected.contains(&pair[0].as_str()) || optional.contains(&pair[0].as_str()))
            || pair[1].is_empty()
            || flags
                .insert(pair[0].clone(), PathBuf::from(&pair[1]))
                .is_some()
        {
            return Err("evidence command has unknown, duplicate or incomplete options".into());
        }
    }
    if expected.iter().any(|name| !flags.contains_key(*name)) {
        return Err(format!("evidence command requires {}", expected.join(" ")));
    }
    Ok(flags)
}
fn bridge(config: EvidenceConfig) -> Result<EvidenceSources, String> {
    let environment = config
        .connections
        .iter()
        .flat_map(|connection| connection.environment_names())
        .chain(
            config
                .native_providers
                .iter()
                .flat_map(|provider| provider.environment_names()),
        )
        .filter_map(|name| {
            std::env::var(name)
                .ok()
                .map(|value| (name.to_owned(), value))
        })
        .collect();
    EvidenceSources::from_config(config, environment)
}
mod discovery;
pub use discovery::discover;

/// Normalize the native transport fingerprint without admitting a source or
/// manufacturing a scientific qualification. Schema pins remain authoritative.
pub fn prepare_native_provider(arguments: &[String]) -> Result<serde_json::Value, String> {
    let selected = flags(arguments, &["--input"])?;
    let mut provider: eal_evidence::providers::NativeProviderConfig =
        read_json(&selected["--input"])?;
    provider.capability.provider.configuration_sha256 = provider.configuration_identity()?;
    provider.validate()?;
    serde_json::to_value(provider).map_err(|_| "cannot encode prepared native provider".into())
}
