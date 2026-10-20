use super::{AcquisitionError, MAX_PINNED_FILE_BYTES, files};
use eal_core::{ExecutionLimits, canonical};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolKind {
    Command,
    JsonFile,
}
impl ToolKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Command => "command",
            Self::JsonFile => "json_file",
        }
    }
}
impl PartialEq<str> for ToolKind {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}
impl PartialEq<&str> for ToolKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PinnedFile {
    pub path: String,
    pub sha256: String,
}

#[derive(Clone)]
pub struct ToolBinding {
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
}
impl std::fmt::Debug for ToolBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolBinding")
            .field("name", &self.name)
            .field("kind", &self.kind)
            .field("version", &self.version)
            .field("argument_count", &self.argv.len())
            .field("path", &self.path)
            .field("environment_entries", &self.env.len())
            .finish_non_exhaustive()
    }
}
impl ToolBinding {
    pub fn command(name: impl Into<String>, version: impl Into<String>, argv: Vec<String>) -> Self {
        Self {
            name: name.into(),
            kind: ToolKind::Command,
            version: version.into(),
            argv,
            path: None,
            timeout_seconds: 30.0,
            max_output_bytes: 1024 * 1024,
            env: BTreeMap::new(),
            pinned_files: Vec::new(),
            parallel_safe: false,
            inherit_env: None,
        }
    }
    pub fn json_file(
        name: impl Into<String>,
        version: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self {
            kind: ToolKind::JsonFile,
            path: Some(path.into()),
            ..Self::command(name, version, Vec::new())
        }
    }
    pub fn effective_environment(&self) -> Result<BTreeMap<String, String>, AcquisitionError> {
        let mut current = BTreeMap::new();
        if let Some(inherited) = &self.inherit_env {
            for name in inherited {
                if let Some(value) = std::env::var_os(name) {
                    current.insert(
                        name.clone(),
                        value.into_string().map_err(|_| {
                            AcquisitionError::invalid(
                                "Process environment must contain valid Unicode scalar values",
                            )
                        })?,
                    );
                }
            }
        } else {
            for (name, value) in std::env::vars_os() {
                let name = name.into_string().map_err(|_| {
                    AcquisitionError::invalid(
                        "Process environment must contain valid Unicode scalar values",
                    )
                })?;
                let value = value.into_string().map_err(|_| {
                    AcquisitionError::invalid(
                        "Process environment must contain valid Unicode scalar values",
                    )
                })?;
                current.insert(name, value);
            }
        }
        current.extend(self.env.clone());
        Ok(current)
    }
    pub fn process_environment_digest(
        &self,
        secret: &[u8],
        effective_env: Option<&BTreeMap<String, String>>,
    ) -> Result<Option<String>, AcquisitionError> {
        self.process_environment_digest_with_limits(
            secret,
            effective_env,
            &ExecutionLimits::default(),
        )
    }
    pub fn process_environment_digest_with_limits(
        &self,
        secret: &[u8],
        effective_env: Option<&BTreeMap<String, String>>,
        limits: &ExecutionLimits,
    ) -> Result<Option<String>, AcquisitionError> {
        if self.kind != ToolKind::Command {
            return Ok(None);
        }
        let environment = match effective_env {
            Some(v) => v.clone(),
            None => self.effective_environment()?,
        };
        Ok(Some(keyed_digest_with_limits(
            secret,
            b"process-environment",
            &json!(environment),
            limits,
        )?))
    }
    pub fn binding_digest(
        &self,
        secret: &[u8],
        workspace: Option<&Path>,
    ) -> Result<String, AcquisitionError> {
        self.binding_digest_with_limits(secret, workspace, &ExecutionLimits::default())
    }
    pub fn binding_digest_with_limits(
        &self,
        secret: &[u8],
        workspace: Option<&Path>,
        limits: &ExecutionLimits,
    ) -> Result<String, AcquisitionError> {
        super::registry::validate_binding(self)?;
        if !self.pinned_files.is_empty() && workspace.is_none() {
            return Err("Pinned collector files require a workspace".into());
        }
        for pinned in &self.pinned_files {
            let original = Path::new(&pinned.path);
            let path: PathBuf = if original.is_absolute() {
                files::resolve(original)?
            } else {
                files::bounded_path(workspace.expect("checked workspace"), &pinned.path)?
            };
            let bytes = files::read_pinned(&path, MAX_PINNED_FILE_BYTES)?;
            if format!("{:x}", Sha256::digest(bytes)) != pinned.sha256 {
                return Err(
                    "Pinned collector file identity differs from operator configuration".into(),
                );
            }
        }
        let mut identity = json!({"name":self.name,"kind":self.kind.as_str(),"version":self.version,
            "argv":self.argv,"path":self.path,"timeout_seconds":self.timeout_seconds,
            "max_output_bytes":self.max_output_bytes,"env":self.env,"pinned_files":self.pinned_files});
        if let Some(inherited) = &self.inherit_env {
            let mut sorted = inherited.clone();
            sorted.sort();
            identity["inherit_env"] = json!(sorted);
        }
        keyed_digest_with_limits(secret, b"tool-binding", &identity, limits)
    }
}

pub fn keyed_digest(
    secret: &[u8],
    domain: &[u8],
    value: &Value,
) -> Result<String, AcquisitionError> {
    keyed_digest_with_limits(secret, domain, value, &ExecutionLimits::default())
}
pub fn keyed_digest_with_limits(
    secret: &[u8],
    domain: &[u8],
    value: &Value,
    limits: &ExecutionLimits,
) -> Result<String, AcquisitionError> {
    if secret.len() != 32 {
        return Err("A private 32-byte store key is required".into());
    }
    if domain.is_empty() {
        return Err("A domain separator is required".into());
    }
    let digest = Sha256::digest(canonical::canonical_json_bytes(value, limits)?);
    let mut hmac = Hmac::<Sha256>::new_from_slice(secret)
        .map_err(|_| AcquisitionError::invalid("A private 32-byte store key is required"))?;
    hmac.update(b"EAL/2\0");
    hmac.update(domain);
    hmac.update(b"\0");
    hmac.update(&digest);
    Ok(format!("{:x}", hmac.finalize().into_bytes()))
}
