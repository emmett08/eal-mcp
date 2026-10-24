//! Read-only external observations under fixed operator selectors.
pub mod config;
mod github;
mod inventory;
mod kubernetes;
mod process;
mod request;
mod tempo;

use chrono::{SecondsFormat, Utc};
pub use config::{Config, Provider};
pub use request::Request;
use serde_json::{Value, json};

/// Execute one bounded observation and retain its exact acquisition correspondence.
pub async fn collect(config: &Config, request: &Request) -> Result<Value, String> {
    config.validate()?;
    let value = match config.provider {
        Provider::Kubernetes => kubernetes::collect(config, request)?,
        Provider::Github => github::collect(config, request)?,
        Provider::Tempo => tempo::collect(config, request).await?,
    };
    let envelope = json!({"value":value,
        "observed_at":Utc::now().to_rfc3339_opts(SecondsFormat::Secs,true),
        "context":request.context,"request":request.acquisition(),
        "details":{"collector":"eal-collector/1","provider":config.provider.name()}});
    if serde_json::to_vec(&envelope)
        .map_err(|_| "Observation encoding failed")?
        .len()
        > config.max_bytes
    {
        return Err("Observation exceeds the configured aggregate output allowance".into());
    }
    Ok(envelope)
}
