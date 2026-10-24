//! Pinned operator configuration for read-only API measurements.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct ApiConfig {
    pub provider: super::ApiProvider,
    pub measurement: String,
    pub path: PathBuf,
    pub sha256: String,
}
impl ApiConfig {
    pub(crate) fn validate(&self) -> Result<(), String> {
        super::super::config::absolute(&self.path)?;
        if !digest(&self.sha256) {
            return Err("API configuration requires a lowercase SHA-256 pin".into());
        }
        self.load().map(|_| ())
    }
    pub(crate) fn load(&self) -> Result<ApiSettings, String> {
        let bytes = pinned(&self.path, &self.sha256)?;
        let settings: ApiSettings = serde_json::from_value(
            eal_adapters::parse_json(&bytes)
                .map_err(|_| "API configuration requires bounded strict JSON")?,
        )
        .map_err(|_| "Invalid API configuration shape")?;
        if settings.schema != "eal.api-source/1"
            || settings.source.provider() != self.provider
            || settings.source.id() != self.measurement
        {
            return Err(
                "API configuration differs from its selected provider or measurement".into(),
            );
        }
        settings.validate()?;
        Ok(settings)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ApiSettings {
    pub schema: String,
    pub origin: String,
    #[serde(default)]
    pub headers_env: BTreeMap<String, String>,
    #[serde(default)]
    pub bearer_env: Option<String>,
    #[serde(default = "window")]
    pub max_window_seconds: u64,
    #[serde(default = "points")]
    pub max_points: u64,
    pub source: ApiSource,
}
fn window() -> u64 {
    86400
}
fn points() -> u64 {
    10000
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ApiSource {
    Prometheus {
        query: String,
        step_seconds: u64,
    },
    Loki {
        query: String,
        limit: u64,
    },
    DatadogMetrics {
        query: String,
        interval_seconds: u64,
    },
    DatadogLogs {
        query: String,
        limit: u64,
    },
    DatadogSpans {
        query: String,
        limit: u64,
    },
    DatadogMonitor {
        id: u64,
    },
    DatadogSlo {
        id: String,
    },
    Openapi {
        #[serde(default)]
        base_path: Option<String>,
        specification: PathBuf,
        specification_sha256: String,
        operation_id: String,
        read_only: bool,
    },
}
impl ApiSource {
    pub fn provider(&self) -> super::ApiProvider {
        match self {
            Self::Prometheus { .. } => super::ApiProvider::Prometheus,
            Self::Loki { .. } => super::ApiProvider::Loki,
            Self::Openapi { .. } => super::ApiProvider::Openapi,
            _ => super::ApiProvider::Datadog,
        }
    }
    pub fn id(&self) -> &'static str {
        match self {
            Self::Prometheus { .. } => "prometheus_range",
            Self::Loki { .. } => "loki_search",
            Self::DatadogMetrics { .. } => "datadog_metrics",
            Self::DatadogLogs { .. } => "datadog_logs",
            Self::DatadogSpans { .. } => "datadog_spans",
            Self::DatadogMonitor { .. } => "datadog_monitor",
            Self::DatadogSlo { .. } => "datadog_slo",
            Self::Openapi { .. } => "openapi_operation",
        }
    }
    pub fn query(&self) -> Option<&str> {
        match self {
            Self::Prometheus { query, .. }
            | Self::Loki { query, .. }
            | Self::DatadogMetrics { query, .. }
            | Self::DatadogLogs { query, .. }
            | Self::DatadogSpans { query, .. } => Some(query),
            _ => None,
        }
    }
}
impl ApiSettings {
    fn validate(&self) -> Result<(), String> {
        eal_adapters::http::validate_origin(&self.origin).map_err(|e| e.to_string())?;
        eal_adapters::http::validate_headers(&self.headers_env).map_err(|e| e.to_string())?;
        if self
            .bearer_env
            .as_ref()
            .is_some_and(|name| !crate::config::environment_name(name))
            || self.bearer_env.is_some()
                && self
                    .headers_env
                    .keys()
                    .any(|name| name.eq_ignore_ascii_case("authorization"))
            || !(1..=2_592_000).contains(&self.max_window_seconds)
            || !(1..=100_000).contains(&self.max_points)
        {
            return Err("Invalid API credential or query allowances".into());
        }
        if self.source.query().is_some_and(|query| {
            query.trim().is_empty() || query.len() > 8192 || query.chars().any(char::is_control)
        }) {
            return Err("API query requires a bounded literal".into());
        }
        match &self.source {
            ApiSource::Prometheus { step_seconds, .. }
            | ApiSource::DatadogMetrics {
                interval_seconds: step_seconds,
                ..
            } if !(1..=86400).contains(step_seconds) => {
                Err("Invalid metric query resolution".into())
            }
            ApiSource::Loki { limit, .. }
            | ApiSource::DatadogLogs { limit, .. }
            | ApiSource::DatadogSpans { limit, .. }
                if !(1..=1000).contains(limit) =>
            {
                Err("Invalid API search limit".into())
            }
            ApiSource::DatadogMonitor { id } if *id == 0 => {
                Err("Monitor identity must be positive".into())
            }
            ApiSource::DatadogSlo { id }
                if id.is_empty()
                    || id.len() > 128
                    || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') =>
            {
                Err("Invalid SLO identity".into())
            }
            ApiSource::Openapi {
                base_path,
                specification,
                specification_sha256,
                operation_id,
                read_only,
            } => {
                if !*read_only
                    || operation_id.trim().is_empty()
                    || operation_id.len() > 128
                    || operation_id.chars().any(char::is_control)
                {
                    return Err(
                        "OpenAPI requires an explicitly approved read-only operation".into(),
                    );
                }
                crate::config::absolute(specification)?;
                super::openapi::Operation::load(
                    specification,
                    specification_sha256,
                    operation_id,
                    base_path.as_deref(),
                )
                .map(|_| ())
            }
            _ => Ok(()),
        }
    }
}
pub(crate) fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
pub(crate) fn pinned(path: &Path, expected: &str) -> Result<Vec<u8>, String> {
    if !digest(expected) {
        return Err("Pinned input requires a lowercase SHA-256 digest".into());
    }
    let bytes = eal_adapters::files::read(path, 1_048_576, "API input")
        .map_err(|_| "Cannot read bounded regular API input")?;
    let hash = Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    if hash != expected {
        return Err("API input differs from its SHA-256 pin".into());
    }
    Ok(bytes)
}
