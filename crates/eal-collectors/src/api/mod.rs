//! Dispatch of admitted API measurements over the common bounded JSON transport.
pub mod config;
mod descriptor;
mod openapi;
mod telemetry;

pub use config::{ApiConfig, ApiSettings, ApiSource};
use eal_adapters::http::JsonClient;
use serde_json::{Value, json};
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApiProvider {
    Openapi,
    Datadog,
    Prometheus,
    Loki,
}
impl ApiProvider {
    pub fn name(self) -> &'static str {
        match self {
            Self::Openapi => "openapi",
            Self::Datadog => "datadog",
            Self::Prometheus => "prometheus",
            Self::Loki => "loki",
        }
    }
}
pub(crate) struct Collected {
    pub value: Value,
    pub credentials: eal_adapters::http::CredentialGuard,
}

/// Describe the exact admitted operation without loading credentials or making network calls.
pub fn describe(selected: &ApiConfig) -> Result<eal_collector_contracts::Catalogue, String> {
    selected.validate()?;
    let descriptor = descriptor::selected(selected, &selected.load()?)?;
    let catalogue = eal_collector_contracts::Catalogue {
        schema: eal_collector_contracts::CATALOGUE_SCHEMA.into(),
        acquisition_contract: eal_collector_contracts::ACQUISITION_CONTRACT.into(),
        collectors: vec![descriptor],
    };
    catalogue.validate_for(eal_collector_contracts::AuthoringProfile::RuntimeExportable)?;
    Ok(catalogue)
}

pub(crate) async fn collect(
    config: &crate::Config,
    selected: &ApiConfig,
    request: &crate::Request,
    started: Instant,
    timeline: &mut crate::timing::Timeline<'_>,
) -> Result<Collected, String> {
    let settings = selected.load()?;
    let descriptor = descriptor::selected(selected, &settings)?;
    jsonschema::validator_for(&descriptor.input_schema)
        .map_err(|_| "Invalid API input contract")?
        .validate(&request.input)
        .map_err(|_| "API input differs from its selected contract")?;
    let mut client = JsonClient::new(
        &settings.origin,
        &settings.headers_env,
        settings.bearer_env.as_deref(),
        started,
        config.limits().timeout(),
        config.limits().max_bytes(),
    )
    .map_err(|e| e.to_string())?;
    let mut value = match &settings.source {
        ApiSource::Openapi {
            specification,
            specification_sha256,
            operation_id,
            ..
        } => {
            openapi::Operation::load(specification, specification_sha256, operation_id)?
                .collect(&mut client, &request.input)
                .await?
        }
        _ => telemetry::collect(&settings, &request.input, &mut client, timeline).await?,
    };
    value["config_sha256"] = json!(selected.sha256);
    value["measurement_only"] = json!(true);
    Ok(Collected {
        value,
        credentials: client.into_credentials(),
    })
}
