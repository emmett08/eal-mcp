//! Bounded command facades for discovery and ordinary EAL support collection.
use eal_collector_contracts::{
    AcquisitionRequest, JsonLimits, ObservationEnvelope, strict_json_exact,
};
use eal_evidence::{
    EvidenceConfig,
    execution::{EvidenceExecutor, ExecutionLimits},
    plan::EvidenceExecutionPlan,
    projection::{graph_request_requirements, project_graph_request},
    providers::EvidenceSources,
    storage::SourceStore,
};
use serde::de::DeserializeOwned;
use std::{
    collections::BTreeMap,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
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
        .map_err(|_| "evidence configuration or plan is unavailable")?;
    if !file
        .metadata()
        .map_err(|_| "evidence file metadata is unavailable")?
        .is_file()
    {
        return Err("evidence configuration/plan requires a regular file".into());
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
    serde_json::from_value(value)
        .map_err(|_| "evidence configuration or execution plan fields are invalid".into())
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

/// Validate the exact request/plan before constructing a bridge or opening cache.
pub async fn support_graph(
    arguments: &[String],
    request_bytes: &[u8],
) -> Result<ObservationEnvelope, String> {
    let started = Instant::now();
    let flags = flags_with_optional(
        arguments,
        &["--plan", "--config", "--store"],
        &["--timeout-ms"],
    )?;
    let mut limits = ExecutionLimits::default();
    if let Some(value) = flags.get("--timeout-ms") {
        let millis = value
            .to_str()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| (1..=86_400_000).contains(value))
            .ok_or("support timeout requires integer milliseconds from 1 to 86400000")?;
        limits.timeout = Duration::from_millis(millis);
    }
    let deadline = started
        .checked_add(limits.timeout)
        .ok_or("support timeout is not representable")?;
    if request_bytes.len() > 1_048_576 {
        return Err("support acquisition request exceeds 1 MiB".into());
    }
    let request_bytes = request_bytes.to_owned();
    let plan_path = flags["--plan"].clone();
    let config_path = flags["--config"].clone();
    let prepared = tokio::task::spawn_blocking(move || {
        support_deadline(deadline)?;
        let value = strict_json_exact(
            &request_bytes,
            JsonLimits {
                max_bytes: 1_048_576,
                ..Default::default()
            },
        )?;
        let request: AcquisitionRequest =
            serde_json::from_value(value).map_err(|_| "support request fields are invalid")?;
        let plan: EvidenceExecutionPlan = read_json(&plan_path)?;
        support_deadline(deadline)?;
        let requirements = graph_request_requirements(&request, &plan)?;
        support_deadline(deadline)?;
        let config = read_json(&config_path)?;
        support_deadline(deadline)?;
        let bridge = bridge(config)?;
        support_deadline(deadline)?;
        Ok::<_, String>((request, plan, requirements, bridge))
    });
    let (request, plan, requirements, bridge) =
        tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), prepared)
            .await
            .map_err(|_| "support preparation deadline exhausted")?
            .map_err(|_| "support preparation worker failed")??;
    support_deadline(deadline)?;
    let store =
        SourceStore::open_until(&flags["--store"], 4096, 16 * 1024 * 1024, deadline).await?;
    let executor = EvidenceExecutor::new(Arc::new(bridge), store, limits)?;
    let report = executor
        .execute_requirements_until(&plan, &requirements, None, deadline)
        .await?;
    project_graph_request(&request, &plan, &report)
}

fn support_deadline(deadline: Instant) -> Result<(), String> {
    if Instant::now() >= deadline {
        Err("support preparation deadline exhausted".into())
    } else {
        Ok(())
    }
}

/// Explicitly reconcile an uncertain read while refusing an active acquisition.
pub fn reconcile(arguments: &[String]) -> Result<serde_json::Value, String> {
    let flags = flags(arguments, &["--store", "--source-id"])?;
    let key = flags["--source-id"]
        .to_str()
        .ok_or("source identity must be UTF-8")?;
    if !eal_evidence::plan::valid_digest(key) {
        return Err("source identity requires a canonical SHA-256 digest".into());
    }
    let store = SourceStore::open(&flags["--store"], 4096, 16 * 1024 * 1024)?;
    store.reconcile_source(key)?;
    Ok(
        serde_json::json!({"schema":"eal.source-reconciliation/1","source_id":key,"reconciled":true}),
    )
}

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
