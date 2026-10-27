//! Operator-owned connection, qualification and bounded discovery configuration.
use eal_collector_contracts::capability::{CapabilityCatalogue, canonical_identity};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::PathBuf};

pub const CONFIG_SCHEMA: &str = "eal.evidence-config/1";
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceConfig {
    pub schema: String,
    pub connections: Vec<ConnectionConfig>,
    #[serde(default)]
    pub native_providers: Vec<crate::providers::NativeProviderConfig>,
    #[serde(default)]
    pub local_catalogues: Vec<CapabilityCatalogue>,
    #[serde(default)]
    pub approved_catalogues: Vec<ApprovedCatalogue>,
    #[serde(default)]
    pub discovery: DiscoveryLimits,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectionConfig {
    pub id: String,
    pub authentication_scope: String,
    pub transport: TransportConfig,
    pub tools: Vec<ToolMapping>,
    pub timeout_seconds: u64,
    pub output_bytes: usize,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TransportConfig {
    Stdio {
        command: PathBuf,
        arguments: Vec<String>,
        inherit_env: Vec<String>,
        cwd: Option<PathBuf>,
    },
    Http {
        url: String,
        token_env: Option<String>,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolMapping {
    pub tool: String,
    /// Pins the tool's raw MCP output schema; null requires its absence.
    pub raw_output_schema_sha256: Option<String>,
    pub response_format: ResponseFormat,
    /// JSON Pointer in decoded structured content or the selected JSON text block.
    pub value_pointer: String,
    pub observation_time: ObservationTime,
    /// Measurement output pointers for all qualified identity dimensions.
    #[serde(default)]
    pub scope_pointers: std::collections::BTreeMap<String, String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResponseFormat {
    Structured,
    JsonText { block: usize },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ObservationTime {
    Pointer { pointer: String },
    CollectionTime,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovedCatalogue {
    pub url: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoveryLimits {
    pub requests: usize,
    pub bytes: usize,
    pub candidates: usize,
    pub pages_per_connection: usize,
    pub timeout_seconds: u64,
}
impl Default for DiscoveryLimits {
    fn default() -> Self {
        Self {
            requests: 64,
            bytes: 4 * 1024 * 1024,
            candidates: 128,
            pages_per_connection: 16,
            timeout_seconds: 60,
        }
    }
}
fn text(value: &str, limit: usize) -> bool {
    !value.trim().is_empty() && value.len() <= limit && !value.chars().any(char::is_control)
}
pub(crate) fn identifier(value: &str) -> bool {
    value.len() <= 256 && eal_collector_contracts::identifier(value)
}
pub(crate) fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
pub(crate) fn pointer(value: &str) -> bool {
    if value.len() > 4096
        || (!value.is_empty() && !value.starts_with('/'))
        || value.chars().any(char::is_control)
    {
        return false;
    }
    let mut chars = value.chars();
    while let Some(character) = chars.next() {
        if character == '~' && !matches!(chars.next(), Some('0' | '1')) {
            return false;
        }
    }
    true
}
pub(crate) fn approved_http(url: &str) -> Result<(), String> {
    eal_mcp_client::client::validate_http_url(url)?;
    let parsed = reqwest::Url::parse(url).map_err(|_| "Invalid approved evidence URL")?;
    let loopback = parsed
        .host_str()
        .and_then(|host| {
            host.trim_matches(['[', ']'])
                .parse::<std::net::IpAddr>()
                .ok()
        })
        .is_some_and(|address| address.is_loopback());
    if parsed.scheme() != "https" && !(parsed.scheme() == "http" && loopback) {
        return Err(
            "Evidence HTTP connections require HTTPS; HTTP is reserved for loopback fixtures"
                .into(),
        );
    }
    Ok(())
}
impl DiscoveryLimits {
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=4096).contains(&self.requests)
            || !(1024..=16 * 1024 * 1024).contains(&self.bytes)
            || !(1..=4096).contains(&self.candidates)
            || !(1..=128).contains(&self.pages_per_connection)
            || !(1..=3600).contains(&self.timeout_seconds)
        {
            return Err("Evidence discovery limits exceed supported bounds".into());
        }
        Ok(())
    }
}
impl ConnectionConfig {
    pub fn validate(&self) -> Result<(), String> {
        if !identifier(&self.id)
            || !text(&self.authentication_scope, 1024)
            || !(1..=3600).contains(&self.timeout_seconds)
            || !(1..=16 * 1024 * 1024).contains(&self.output_bytes)
            || self.tools.len() > 4096
        {
            return Err("Invalid evidence connection identity or resource limits".into());
        }
        match &self.transport {
            TransportConfig::Http { url, token_env } => {
                approved_http(url)?;
                if token_env
                    .as_deref()
                    .is_some_and(|name| !eal_collector_contracts::environment_name(name))
                {
                    return Err("Invalid evidence credential reference".into());
                }
            }
            TransportConfig::Stdio {
                command,
                arguments,
                inherit_env,
                cwd,
            } => {
                if !command.is_absolute()
                    || command.to_str().is_none_or(|value| !text(value, 4096))
                    || arguments.len() > 128
                    || arguments
                        .iter()
                        .any(|value| value.len() > 16_384 || value.chars().any(char::is_control))
                    || inherit_env.len() > 64
                    || inherit_env
                        .iter()
                        .any(|name| !eal_collector_contracts::environment_name(name))
                    || inherit_env.iter().collect::<BTreeSet<_>>().len() != inherit_env.len()
                    || cwd.as_ref().is_some_and(|path| !path.is_absolute())
                {
                    return Err("Stdio evidence connections require an absolute operator executable and bounded arguments/environment references".into());
                }
            }
        }
        let mut names = BTreeSet::new();
        for tool in &self.tools {
            if !text(&tool.tool, 256)
                || !names.insert(&tool.tool)
                || !pointer(&tool.value_pointer)
                || tool
                    .raw_output_schema_sha256
                    .as_deref()
                    .is_some_and(|value| !digest(value))
                || tool.scope_pointers.len() > 128
                || tool
                    .scope_pointers
                    .iter()
                    .any(|(key, value)| !text(key, 1024) || !pointer(value))
                || matches!(&tool.observation_time,ObservationTime::Pointer{pointer:selected}if !pointer(selected))
                || matches!(tool.response_format,ResponseFormat::JsonText{block}if block>127)
            {
                return Err("Invalid evidence tool normalisation or schema pins".into());
            }
        }
        Ok(())
    }
    /// Static transport/normalisation identity excludes separate qualification documents.
    pub fn configuration_identity(&self) -> Result<String, String> {
        self.validate()?;
        canonical_identity(self)
    }
    pub fn environment_names(&self) -> Vec<&str> {
        match &self.transport {
            TransportConfig::Http { token_env, .. } => {
                token_env.iter().map(String::as_str).collect()
            }
            TransportConfig::Stdio { inherit_env, .. } => {
                inherit_env.iter().map(String::as_str).collect()
            }
        }
    }
}
impl EvidenceConfig {
    pub fn from_json(bytes: &[u8]) -> Result<Self, String> {
        let value = eal_collector_contracts::strict_json_exact(
            bytes,
            eal_collector_contracts::JsonLimits {
                max_bytes: 16 * 1024 * 1024,
                ..Default::default()
            },
        )?;
        let config: Self =
            serde_json::from_value(value).map_err(|_| "Invalid evidence configuration fields")?;
        config.validate()?;
        Ok(config)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != CONFIG_SCHEMA
            || self.connections.len() > 128
            || self.native_providers.len() > 128
            || self.local_catalogues.len() > 128
            || self.approved_catalogues.len() > 32
        {
            return Err("Invalid evidence configuration schema or catalogue bounds".into());
        }
        self.discovery.validate()?;
        let mut native_ids = BTreeSet::new();
        for native in &self.native_providers {
            native.validate()?;
            if !native_ids.insert(&native.capability.id) {
                return Err("Native capability identities must be unique".into());
            }
        }
        let mut ids = BTreeSet::new();
        for connection in &self.connections {
            connection.validate()?;
            if !ids.insert(&connection.id) {
                return Err("Evidence connection aliases must be unique".into());
            }
        }
        for catalogue in &self.local_catalogues {
            catalogue.validate()?;
        }
        let mut urls = BTreeSet::new();
        for catalogue in &self.approved_catalogues {
            approved_http(&catalogue.url)?;
            if !urls.insert(&catalogue.url) {
                return Err("Approved evidence catalogues must be unique".into());
            }
        }
        Ok(())
    }
}
