//! Direct invariants for executable bindings, independent of any serialiser.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const RUNTIME_EXPORTABLE_PROFILE: &str = "eal.runtime-exportable/1";
pub const OPERATOR_ENVIRONMENT_POLICY: &str = "eal.operator-environment/1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolKind {
    Command,
    JsonFile,
    McpStdio,
    McpHttp,
    HttpGet,
}
impl ToolKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Command => "command",
            Self::JsonFile => "json_file",
            Self::McpStdio => "mcp_stdio",
            Self::McpHttp => "mcp_http",
            Self::HttpGet => "http_get",
        }
    }
}
impl PartialEq<str> for ToolKind {
    fn eq(&self, value: &str) -> bool {
        self.as_str() == value
    }
}
impl PartialEq<&str> for ToolKind {
    fn eq(&self, value: &&str) -> bool {
        self.as_str() == *value
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PinnedFile {
    pub path: String,
    pub sha256: String,
}

/// A prospective executable specification. Call `validate` at admission and
/// immediately before execution, since callers may change these public fields.
#[derive(Clone)]
pub struct BindingSpec {
    pub name: String,
    pub kind: ToolKind,
    pub version: String,
    pub argv: Vec<String>,
    pub path: Option<String>,
    pub timeout_seconds: f64,
    pub max_output_bytes: usize,
    pub env: BTreeMap<String, String>,
    pub pinned_files: Vec<PinnedFile>,
    pub parallel_safe: bool,
    pub inherit_env: Option<Vec<String>>,
    pub direct: Option<crate::DirectBinding>,
}
impl BindingSpec {
    pub fn validate(&self) -> Result<(), String> {
        self.validate_readiness(false)
    }
    pub(crate) fn validate_readiness(&self, prepared: bool) -> Result<(), String> {
        if crate::ToolId::parse(self.name.clone()).is_err() || !literal(&self.version, 8192) {
            return Err("EAL_BINDING_IDENTITY: invalid binding identity".into());
        }
        if !self.timeout_seconds.is_finite()
            || self.timeout_seconds <= 0.0
            || self.timeout_seconds > 3600.0
        {
            return Err(
                "EAL_BINDING_LIMIT: timeout_seconds must be finite, positive and at most 3600"
                    .into(),
            );
        }
        if !(1..=16_777_216).contains(&self.max_output_bytes) {
            return Err(
                "EAL_BINDING_LIMIT: max_output_bytes must be from 1 through 16777216".into(),
            );
        }
        match self.kind {
            ToolKind::Command | ToolKind::McpStdio => {
                if self.path.is_some()
                    || self.argv.is_empty()
                    || self.argv.len() > 256
                    || self.argv.iter().any(|value| {
                        value.is_empty() || value.contains('\0') || (!prepared && unresolved(value))
                    })
                {
                    return Err("EAL_BINDING_COMMAND: command requires resolved bounded argv and does not use path".into());
                }
            }
            ToolKind::McpHttp | ToolKind::HttpGet => {
                if self.path.is_some()
                    || !self.argv.is_empty()
                    || !self.env.is_empty()
                    || self.inherit_env.is_some()
                    || !self.pinned_files.is_empty()
                {
                    return Err(
                        "EAL_BINDING_HTTP: network bindings have no command or file settings"
                            .into(),
                    );
                }
            }
            ToolKind::JsonFile => {
                if self
                    .path
                    .as_deref()
                    .is_none_or(|value| !literal(value, 8192) || (!prepared && unresolved(value)))
                    || !self.argv.is_empty()
                    || self.inherit_env.is_some()
                    || !self.pinned_files.is_empty()
                {
                    return Err("EAL_BINDING_FILE: json_file requires a resolved path and has no command settings".into());
                }
            }
        }
        let direct = matches!(
            self.kind,
            ToolKind::McpStdio | ToolKind::McpHttp | ToolKind::HttpGet
        );
        if direct != self.direct.is_some() {
            return Err(
                "EAL_DIRECT_BINDING: transport settings required only for direct operations".into(),
            );
        }
        if let Some(settings) = &self.direct {
            settings.validate(self.kind)?;
        }
        if self.env.keys().any(|name| !crate::environment_name(name))
            || self.env.values().any(|value| value.contains('\0'))
        {
            return Err("EAL_BINDING_ENVIRONMENT: invalid explicit environment".into());
        }
        if let Some(names) = &self.inherit_env {
            validate_environment_names(names, 64)?;
        }
        if self.pinned_files.len() > 32
            || self
                .pinned_files
                .iter()
                .map(|value| &value.path)
                .collect::<BTreeSet<_>>()
                .len()
                != self.pinned_files.len()
            || self
                .pinned_files
                .iter()
                .any(|pin| !literal(&pin.path, 8192) || !valid_digest(&pin.sha256))
        {
            return Err("EAL_BINDING_PIN: pinned_files requires distinct paths and lowercase SHA-256 digests".into());
        }
        Ok(())
    }
}
fn literal(value: &str, maximum: usize) -> bool {
    !value.is_empty() && value.len() <= maximum && !value.chars().any(char::is_control)
}
fn unresolved(value: &str) -> bool {
    value.contains("{operator.")
        || value.starts_with("__ENGARG_")
        || value.starts_with("REPLACE_")
        || value.starts_with("/REPLACE_")
        || value.starts_with("<operator")
}
fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub fn validate_environment_names(names: &[String], maximum: usize) -> Result<(), String> {
    if names.len() > maximum
        || names
            .iter()
            .any(|name| crate::EnvironmentName::parse(name.clone()).is_err())
        || names.iter().collect::<BTreeSet<_>>().len() != names.len()
    {
        return Err(
            "EAL_ENVIRONMENT_NAMES: distinct bounded environment names are required".into(),
        );
    }
    Ok(())
}

/// Operator-owned classification is supplied separately from untrusted
/// catalogues and the established registry wire format. Omitted names are secret.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperatorEnvironmentPolicy {
    pub schema: String,
    pub forward: Vec<String>,
    #[serde(default)]
    pub public: Vec<String>,
}
impl OperatorEnvironmentPolicy {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != OPERATOR_ENVIRONMENT_POLICY {
            return Err(
                "EAL_ENVIRONMENT_POLICY_VERSION: unsupported operator environment policy".into(),
            );
        }
        validate_environment_names(&self.forward, 64)?;
        validate_environment_names(&self.public, 64)?;
        if self
            .public
            .iter()
            .any(|name| !self.forward.contains(name) || credential_name(name))
        {
            return Err("EAL_ENVIRONMENT_POLICY_AUTHORITY: public names must be forwarded configuration rather than credentials".into());
        }
        Ok(())
    }
    pub fn sensitive(&self, name: &str) -> bool {
        credential_name(name) || !self.public.iter().any(|public| public == name)
    }
}
fn credential_name(name: &str) -> bool {
    let name = name.to_ascii_uppercase();
    [
        "TOKEN",
        "SECRET",
        "PASSWORD",
        "CREDENTIAL",
        "API_KEY",
        "AUTHORIZATION",
        "BEARER",
    ]
    .iter()
    .any(|part| name.contains(part))
}

/// The registry wire representation. Defaults preserve the established command
/// and retained-file format. All semantic validation delegates to BindingSpec.
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryBinding {
    pub kind: ToolKind,
    pub version: String,
    #[serde(default)]
    pub argv: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: f64,
    #[serde(default = "default_output")]
    pub max_output_bytes: usize,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub pinned_files: Vec<PinnedFile>,
    #[serde(default)]
    pub parallel_safe: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inherit_env: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct: Option<crate::DirectBinding>,
}
fn default_timeout() -> f64 {
    30.0
}
fn default_output() -> usize {
    1024 * 1024
}
impl RegistryBinding {
    pub fn specification(&self, name: &str) -> BindingSpec {
        BindingSpec {
            name: name.into(),
            kind: self.kind,
            version: self.version.clone(),
            argv: self.argv.clone(),
            path: self.path.clone(),
            timeout_seconds: self.timeout_seconds,
            max_output_bytes: self.max_output_bytes,
            env: self.env.clone(),
            pinned_files: self.pinned_files.clone(),
            parallel_safe: self.parallel_safe,
            inherit_env: self.inherit_env.clone(),
            direct: self.direct.clone(),
        }
    }
    pub fn validate(&self, name: &str) -> Result<(), String> {
        self.specification(name).validate()
    }
}
/// Complete executable registry admission, shared by producers and consumers.
#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryDocument {
    #[serde(default)]
    pub context: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub tools: BTreeMap<String, RegistryBinding>,
}
impl RegistryDocument {
    pub fn from_toml(text: &str) -> Result<Self, String> {
        if text.len() > 1024 * 1024 {
            return Err("EAL_REGISTRY_LIMIT: registry exceeds 1 MiB".into());
        }
        let table: toml::Table = toml::from_str(text).map_err(|error| {
            crate::toml_diagnostic(crate::TomlDocumentKind::Registry, text, &error)
        })?;
        let document: Self =
            toml::Value::Table(table)
                .try_into()
                .map_err(|error: toml::de::Error| {
                    if error.message().starts_with("unknown field") {
                        "EAL_REGISTRY_FIELDS: unknown registry settings"
                    } else {
                        "EAL_REGISTRY_SHAPE: invalid registry field types or required fields"
                    }
                })?;
        document.validate()?;
        Ok(document)
    }
    pub fn validate(&self) -> Result<(), String> {
        for (name, binding) in &self.tools {
            binding.validate(name)?;
        }
        Ok(())
    }
}
