//! Read-only external observations under fixed operator selectors.
pub mod api;
pub mod catalogue;
pub mod config;
pub mod evidence;
mod github;
mod github_delivery;
mod inventory;
mod kubernetes;
mod process;
pub mod reports;
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
    let mut retained_at = None;
    let (value, credentials) = match config.selected() {
        ProviderConfig::Report(selected) => {
            let (value, observed_at) = reports::collect(selected, request)?;
            let observed = chrono::DateTime::parse_from_rfc3339(&observed_at)
                .map_err(|_| "Invalid retained report time")?;
            if observed.timestamp() > timeline.mark().timestamp().saturating_add(60) {
                return Err("Retained report measurement time is in the future".into());
            }
            retained_at = Some(observed_at);
            (value, None)
        }
        ProviderConfig::Kubernetes(selected) => (
            kubernetes::collect(config, selected, request, started, &mut timeline)?,
            None,
        ),
        ProviderConfig::Github(selected) => {
            (github::collect(config, selected, request, started)?, None)
        }
        ProviderConfig::GithubChecks(selected) => {
            let environment = process::environment(config)?;
            let guard =
                eal_adapters::http::CredentialGuard::captured(environment.values().cloned());
            (
                github_delivery::checks(config, selected, request, started, &environment)?,
                Some(Guard::Captured(guard)),
            )
        }
        ProviderConfig::GithubWorkflow(selected) => {
            let environment = process::environment(config)?;
            let guard =
                eal_adapters::http::CredentialGuard::captured(environment.values().cloned());
            (
                github_delivery::workflow(config, selected, request, started, &environment)?,
                Some(Guard::Captured(guard)),
            )
        }
        ProviderConfig::Tempo(selected) => {
            let collected =
                tempo::collect(config, selected, request, started, &mut timeline).await?;
            (collected.value, Some(Guard::Tempo(collected.credentials)))
        }
        ProviderConfig::Api(selected) => {
            let collected = api::collect(config, selected, request, started, &mut timeline).await?;
            (
                collected.value,
                Some(Guard::Captured(collected.credentials)),
            )
        }
    };
    if started.elapsed() >= config.limits().timeout() {
        return Err("Collector deadline exhausted".into());
    }
    let interval = timeline.finish();
    let envelope = serde_json::to_value(ObservationEnvelope {
        value,
        observed_at: Some(retained_at.unwrap_or_else(|| timeline.observed_at())),
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

enum Guard {
    Tempo(tempo::Credentials),
    Captured(eal_adapters::http::CredentialGuard),
}
impl Guard {
    fn inspect(&self, value: &Value) -> Result<(), String> {
        match self {
            Self::Tempo(guard) => guard.inspect(value),
            Self::Captured(guard) => guard.inspect(value).map_err(|e| e.to_string()),
        }
    }
}
