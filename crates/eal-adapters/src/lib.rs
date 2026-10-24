//! Optional acquisition mechanics independent of the pure EAL language core.
#![forbid(unsafe_code)]
pub mod command;
pub mod files;
pub mod generic;
pub mod http;
pub mod profiles;
pub mod retained;
pub mod scoped;
pub mod trace;

#[derive(Debug)]
pub enum Error {
    InvalidInput(String),
    Provider(String),
    Budget(String),
    Io(std::io::Error),
    Json(serde_json::Error),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(s) | Self::Provider(s) | Self::Budget(s) => f.write_str(s),
            Self::Io(e) => e.fmt(f),
            Self::Json(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

/// Strict JSON admission shared with collector contracts.
pub fn parse_json(bytes: &[u8]) -> Result<serde_json::Value, Error> {
    eal_collector_contracts::strict_json_exact(
        bytes,
        eal_collector_contracts::JsonLimits {
            max_bytes: 16 * 1024 * 1024,
            max_depth: 64,
            max_nodes: 1_000_000,
        },
    )
    .map_err(Error::InvalidInput)
}

/// Serialise one authoritative observation envelope without changing semantic data.
pub fn observation(
    request: &eal_collector_contracts::AcquisitionRequest,
    value: serde_json::Value,
    observed_at: Option<String>,
    details: Option<serde_json::Value>,
) -> Result<serde_json::Value, Error> {
    request.validate().map_err(Error::InvalidInput)?;
    Ok(serde_json::to_value(
        eal_collector_contracts::ObservationEnvelope {
            value,
            observed_at,
            context: Some(request.context.clone()),
            request: Some(request.acquisition()),
            details,
        },
    )?)
}
