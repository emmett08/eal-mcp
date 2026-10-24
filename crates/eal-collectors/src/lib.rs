//! Read-only external observations under fixed operator selectors.
pub mod catalogue;
pub mod config;
mod github;
mod inventory;
mod kubernetes;
mod process;
mod request;
mod tempo;
mod timing;

use config::ProviderConfig;
pub use config::{Config, Provider};
use eal_collector_contracts::ObservationEnvelope;
pub use request::Request;
use serde_json::{Value, json};
use std::time::Instant;
pub use timing::{ObservationClock, SystemClock};

/// Execute one bounded observation and retain its exact acquisition correspondence.
pub async fn collect(config: &Config, request: &Request) -> Result<Value, String> {
    collect_with_clock(config, request, &SystemClock).await
}

/// Injectable wall time for reproducible age checks. It never controls execution deadlines.
pub async fn collect_with_clock(
    config: &Config,
    request: &Request,
    clock: &dyn ObservationClock,
) -> Result<Value, String> {
    let started = Instant::now();
    request.validate_native()?;
    if request.tool_version != config.version().name() {
        return Err(
            "Request tool version differs from the selected native collector contract".into(),
        );
    }
    let mut timeline = timing::Timeline::new(clock);
    let (value, credentials) = match config.selected() {
        ProviderConfig::Kubernetes(selected) => (
            kubernetes::collect(config, selected, request, started, &mut timeline)?,
            None,
        ),
        ProviderConfig::Github(selected) => {
            (github::collect(config, selected, request, started)?, None)
        }
        ProviderConfig::Tempo(selected) => {
            let collected =
                tempo::collect(config, selected, request, started, &mut timeline).await?;
            (collected.value, Some(collected.credentials))
        }
    };
    if started.elapsed() >= config.limits().timeout() {
        return Err("Collector deadline exhausted".into());
    }
    let interval = timeline.finish();
    let envelope = serde_json::to_value(ObservationEnvelope {
        value,
        observed_at: Some(timeline.observed_at()),
        context: Some(request.context.clone()),
        request: Some(request.acquisition()),
        details: Some(json!({
            "collector":format!("eal-collector/{}",config.version().name()),
            "provider":config.provider().name(),
            "acquisition_interval":interval
        })),
    })
    .map_err(|_| "Observation encoding failed")?;
    if serde_json::to_vec(&envelope)
        .map_err(|_| "Observation encoding failed")?
        .len()
        > config.limits().max_bytes()
    {
        return Err("Observation exceeds the configured aggregate output allowance".into());
    }
    if let Some(credentials) = credentials {
        credentials.inspect(&envelope)?;
    }
    if started.elapsed() >= config.limits().timeout() {
        return Err("Collector deadline exhausted".into());
    }
    Ok(envelope)
}
