//! Operator-owned native authorities; request arguments never become argv or URLs.
use crate::config::{ObservationTime, ResponseFormat, ToolMapping, approved_http, digest, pointer};
use eal_collector_contracts::capability::{
    CapabilityDescriptor, TransportKind, canonical_identity,
};
use serde::{
    Deserialize, Serialize,
    de::{DeserializeOwned, Error},
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeProviderConfig {
    pub capability: CapabilityDescriptor,
    pub transport: NativeTransport,
    pub mapping: ToolMapping,
    /// Package manifests and other operator-owned code/configuration dependencies.
    #[serde(default)]
    pub pinned_files: Vec<NativePinnedFile>,
    /// Selected secret values are inspected before any observation is retained.
    #[serde(default)]
    pub credential_env: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativePinnedFile {
    pub path: PathBuf,
    pub sha256: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NativeTransport {
    Command {
        program: PathBuf,
        supervisor: PathBuf,
        arguments: Vec<String>,
        cwd: PathBuf,
        #[serde(default)]
        inherit_env: Vec<String>,
        protocol: CommandProtocol,
    },
    /// A fixed read-only endpoint. Only operator-declared query values vary.
    HttpGet {
        url: String,
        #[serde(default)]
        query_bindings: BTreeMap<String, String>,
        token_env: Option<String>,
    },
    RetainedArtifact {
        path: PathBuf,
        sha256: String,
    },
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CommandProtocol {
    RawJson,
    EalAcquisition {
        environment: String,
        tool: String,
        tool_version: String,
        context: Value,
    },
}

// Deserialize the whole enum before selecting its variant. Serde's internally
// tagged Content buffering cannot preserve opaque arbitrary-precision JSON fields.
fn field<T: DeserializeOwned>(
    fields: &mut serde_json::Map<String, Value>,
    key: &str,
) -> Result<T, String> {
    let value = fields
        .remove(key)
        .ok_or_else(|| format!("Native provider field {key} is required"))?;
    serde_json::from_value(value).map_err(|_| format!("Native provider field {key} is invalid"))
}
fn default_field<T: DeserializeOwned + Default>(
    fields: &mut serde_json::Map<String, Value>,
    key: &str,
) -> Result<T, String> {
    fields
        .remove(key)
        .map(|value| {
            serde_json::from_value(value)
                .map_err(|_| format!("Native provider field {key} is invalid"))
        })
        .unwrap_or_else(|| Ok(T::default()))
}
fn enum_fields<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<(String, serde_json::Map<String, Value>), D::Error> {
    let value = eal_collector_contracts::deserialize_value_exact(deserializer)?;
    let mut fields = value
        .as_object()
        .cloned()
        .ok_or_else(|| D::Error::custom("Native provider variant requires an object"))?;
    let kind = field(&mut fields, "kind").map_err(D::Error::custom)?;
    Ok((kind, fields))
}
impl<'de> Deserialize<'de> for CommandProtocol {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let (kind, mut fields) = enum_fields(deserializer)?;
        let protocol = match kind.as_str() {
            "raw_json" => Self::RawJson,
            "eal_acquisition" => Self::EalAcquisition {
                environment: field(&mut fields, "environment").map_err(D::Error::custom)?,
                tool: field(&mut fields, "tool").map_err(D::Error::custom)?,
                tool_version: field(&mut fields, "tool_version").map_err(D::Error::custom)?,
                context: fields
                    .remove("context")
                    .ok_or_else(|| D::Error::custom("Native EAL context is required"))?,
            },
            _ => return Err(D::Error::custom("Unknown native command protocol")),
        };
        if !fields.is_empty() {
            return Err(D::Error::custom("Unknown native command protocol fields"));
        }
        Ok(protocol)
    }
}
impl<'de> Deserialize<'de> for NativeTransport {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let (kind, mut fields) = enum_fields(deserializer)?;
        let transport = match kind.as_str() {
            "command" => Self::Command {
                program: field(&mut fields, "program").map_err(D::Error::custom)?,
                supervisor: field(&mut fields, "supervisor").map_err(D::Error::custom)?,
                arguments: field(&mut fields, "arguments").map_err(D::Error::custom)?,
                cwd: field(&mut fields, "cwd").map_err(D::Error::custom)?,
                inherit_env: default_field(&mut fields, "inherit_env").map_err(D::Error::custom)?,
                protocol: field(&mut fields, "protocol").map_err(D::Error::custom)?,
            },
            "http_get" => Self::HttpGet {
                url: field(&mut fields, "url").map_err(D::Error::custom)?,
                query_bindings: default_field(&mut fields, "query_bindings")
                    .map_err(D::Error::custom)?,
                token_env: default_field(&mut fields, "token_env").map_err(D::Error::custom)?,
            },
            "retained_artifact" => Self::RetainedArtifact {
                path: field(&mut fields, "path").map_err(D::Error::custom)?,
                sha256: field(&mut fields, "sha256").map_err(D::Error::custom)?,
            },
            _ => return Err(D::Error::custom("Unknown native provider transport")),
        };
        if !fields.is_empty() {
            return Err(D::Error::custom("Unknown native provider transport fields"));
        }
        Ok(transport)
    }
}

fn bounded_text(value: &str, limit: usize) -> bool {
    !value.trim().is_empty() && value.len() <= limit && !value.chars().any(char::is_control)
}
fn path_valid(path: &std::path::Path) -> bool {
    path.is_absolute() && path.to_str().is_some_and(|value| bounded_text(value, 4096))
}

impl NativeProviderConfig {
    /// Hash only the fixed execution/normalization configuration, not model prose.
    pub fn configuration_identity(&self) -> Result<String, String> {
        self.validate_execution()?;
        canonical_identity(&(
            &self.transport,
            &self.mapping,
            &self.pinned_files,
            &self.credential_env,
        ))
    }
    pub fn validate(&self) -> Result<(), String> {
        self.capability.validate()?;
        self.validate_execution()?;
        let expected = match self.transport {
            NativeTransport::Command { .. } => TransportKind::Command,
            NativeTransport::HttpGet { .. } => TransportKind::Http,
            NativeTransport::RetainedArtifact { .. } => TransportKind::RetainedArtifact,
        };
        if self.capability.provider.transport != expected
            || self.capability.provider.connection.is_some()
            || self.capability.provider.configuration_sha256 != self.configuration_identity()?
        {
            return Err(
                "Native provider identity differs from its operator-owned configuration".into(),
            );
        }
        if let Some(semantics) = &self.capability.semantics
            && (semantics
                .identity_dimensions
                .iter()
                .any(|dimension| !self.mapping.scope_pointers.contains_key(dimension))
                || (semantics.time.original_observation_time
                    || semantics.time.historical_intervals)
                    && matches!(
                        self.mapping.observation_time,
                        ObservationTime::CollectionTime
                    ))
        {
            return Err(
                "Native qualified identity or original-time extraction is unconfigured".into(),
            );
        }
        if matches!(self.transport, NativeTransport::RetainedArtifact { .. })
            && matches!(
                self.mapping.observation_time,
                ObservationTime::CollectionTime
            )
        {
            return Err("Retained artifacts must preserve their original observation time".into());
        }
        Ok(())
    }
    fn validate_execution(&self) -> Result<(), String> {
        let inherited = self.environment_names();
        if self.credential_env.len() > 64
            || self
                .credential_env
                .iter()
                .any(|name| !inherited.contains(&name.as_str()))
            || self.credential_env.iter().collect::<BTreeSet<_>>().len()
                != self.credential_env.len()
        {
            return Err(
                "Native credential references must be distinct selected environment names".into(),
            );
        }
        if !matches!(self.mapping.response_format, ResponseFormat::Structured)
            || !bounded_text(&self.mapping.tool, 256)
            || self.mapping.raw_output_schema_sha256.is_some()
            || !pointer(&self.mapping.value_pointer)
            || self.mapping.scope_pointers.len() > 128
            || self
                .mapping
                .scope_pointers
                .iter()
                .any(|(dimension, value)| !bounded_text(dimension, 1024) || !pointer(value))
            || matches!(&self.mapping.observation_time,ObservationTime::Pointer{pointer:selected} if !pointer(selected))
            || self.pinned_files.len() > 128
            || self
                .pinned_files
                .iter()
                .any(|pin| !path_valid(&pin.path) || !digest(&pin.sha256))
            || self
                .pinned_files
                .iter()
                .map(|pin| &pin.path)
                .collect::<BTreeSet<_>>()
                .len()
                != self.pinned_files.len()
        {
            return Err(
                "Native providers require bounded explicit output mappings and file pins".into(),
            );
        }
        match &self.transport {
            NativeTransport::Command {
                program,
                supervisor,
                arguments,
                cwd,
                inherit_env,
                protocol,
            } => {
                if !path_valid(program)
                    || !path_valid(supervisor)
                    || !path_valid(cwd)
                    || arguments.len() > 128
                    || arguments.iter().any(|argument| {
                        argument.len() > 16384 || argument.chars().any(char::is_control)
                    })
                    || inherit_env.len() > 64
                    || inherit_env
                        .iter()
                        .any(|name| !eal_collector_contracts::environment_name(name))
                    || inherit_env.iter().collect::<BTreeSet<_>>().len() != inherit_env.len()
                {
                    return Err("Native commands require fixed absolute executable/cwd and bounded argv/environment".into());
                }
                if let CommandProtocol::EalAcquisition {
                    environment,
                    tool,
                    tool_version,
                    context,
                } = protocol
                {
                    if !bounded_text(environment, 1024)
                        || !bounded_text(tool, 1024)
                        || !bounded_text(tool_version, 1024)
                        || !context.is_object()
                    {
                        return Err("Native EAL command protocol requires fixed request identity and object context".into());
                    }
                    canonical_identity(context)?;
                }
            }
            NativeTransport::HttpGet {
                url,
                query_bindings,
                token_env,
            } => {
                approved_http(url)?;
                if query_bindings.len() > 128
                    || query_bindings
                        .iter()
                        .any(|(name, value)| !bounded_text(name, 256) || !pointer(value))
                    || token_env
                        .as_ref()
                        .is_some_and(|name| !eal_collector_contracts::environment_name(name))
                {
                    return Err(
                        "Native HTTP query bindings or credential reference are invalid".into(),
                    );
                }
            }
            NativeTransport::RetainedArtifact { path, sha256 } => {
                if !path_valid(path) || !digest(sha256) {
                    return Err(
                        "Retained artifacts require a fixed absolute path and SHA-256".into(),
                    );
                }
            }
        }
        Ok(())
    }
    pub fn environment_names(&self) -> Vec<&str> {
        match &self.transport {
            NativeTransport::Command { inherit_env, .. } => {
                inherit_env.iter().map(String::as_str).collect()
            }
            NativeTransport::HttpGet { token_env, .. } => {
                token_env.iter().map(String::as_str).collect()
            }
            NativeTransport::RetainedArtifact { .. } => vec![],
        }
    }
}
