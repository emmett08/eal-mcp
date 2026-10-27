//! A tool package is code and deployment authority, never an evidence execution plan.
use crate::{
    Error, SourceFile, ToolchainProfile, bounded_file,
    runner::{DockerRunner, ExecutionMode, FixtureRunner},
};
use eal_collector_contracts::{
    AcquisitionRequest, JsonLimits, PinnedFile, SensitiveJsonLimits, contains_sensitive_json,
    strict_json_exact,
};
use eal_process::CancellationToken;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

pub const PACKAGE_SCHEMA: &str = "EAL/tool-package/1";
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolPackage {
    pub schema: String,
    pub tool: String,
    pub version: String,
    pub image_id: String,
    pub require_observed_at: bool,
    pub profile: ToolchainProfile,
    /// Paths relative to the package's source directory; mounted as /src.
    pub source: Vec<PinnedFile>,
    pub input_schema: PinnedFile,
    pub output_schema: PinnedFile,
    /// Retained verification provenance. This does not make the new host identical.
    pub verification: PinnedFile,
}
#[derive(Debug)]
pub struct LoadedPackage {
    pub manifest: ToolPackage,
    root: PathBuf,
    source: Vec<SourceFile>,
    input: Value,
    output: Value,
}
fn invalid(message: &str) -> Error {
    Error::InvalidInput(message.into())
}
pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn digest_valid(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn relative(path: &str) -> Result<(), Error> {
    if path.is_empty()
        || path.len() > 256
        || !path
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-./".contains(&b))
        || path
            .split('/')
            .any(|p| p.is_empty() || p == "." || p == "..")
    {
        return Err(invalid(
            "tool package requires portable relative resource paths",
        ));
    }
    Ok(())
}
fn pinned(root: &Path, pin: &PinnedFile, limit: usize) -> Result<Vec<u8>, Error> {
    relative(&pin.path)?;
    if !digest_valid(&pin.sha256) {
        return Err(invalid("tool resource requires a SHA-256 pin"));
    }
    let bytes = bounded_file::read_within(root, Path::new(&pin.path), limit, "tool resource")?;
    if digest(&bytes) != pin.sha256 {
        return Err(invalid(
            "tool package resource differs from its pinned content",
        ));
    }
    Ok(bytes)
}
fn schema(bytes: &[u8]) -> Result<Value, Error> {
    let value = strict_json_exact(
        bytes,
        JsonLimits {
            max_bytes: 262_144,
            ..Default::default()
        },
    )
    .map_err(Error::InvalidInput)?;
    // Schemas cannot introduce undeclared file or network access.
    fn local_refs(value: &Value) -> bool {
        match value {
            Value::Object(map) => map.iter().all(|(k, v)| {
                if k == "$ref" || k == "$dynamicRef" {
                    v.as_str().is_some_and(|s| s.starts_with('#'))
                } else {
                    local_refs(v)
                }
            }),
            Value::Array(values) => values.iter().all(local_refs),
            _ => true,
        }
    }
    if !local_refs(&value) || jsonschema::validator_for(&value).is_err() {
        return Err(invalid(
            "tool schema must be valid and use only local references",
        ));
    }
    Ok(value)
}
impl ToolPackage {
    pub fn load(path: &Path) -> Result<LoadedPackage, Error> {
        let bytes = bounded_file::read(path, 262_144, "tool package")?;
        let text =
            std::str::from_utf8(&bytes).map_err(|_| invalid("tool package requires UTF-8 TOML"))?;
        let manifest: Self =
            toml::from_str(text).map_err(|_| invalid("tool package is invalid TOML"))?;
        if manifest.schema != PACKAGE_SCHEMA
            || !eal_collector_contracts::identifier(&manifest.tool)
            || manifest.version != "1"
            || !manifest
                .image_id
                .strip_prefix("sha256:")
                .is_some_and(digest_valid)
        {
            return Err(invalid("tool package identity is invalid"));
        }
        manifest.profile.validate()?;
        if manifest.source.is_empty() || manifest.source.len() > 16 {
            return Err(invalid("tool package requires 1 to 16 source files"));
        }
        let absolute = std::path::absolute(path)?;
        let root = absolute
            .parent()
            .ok_or_else(|| invalid("tool package has no directory"))?
            .canonicalize()?;
        let mut names = BTreeSet::new();
        let mut source = Vec::new();
        let mut total = 0usize;
        for pin in &manifest.source {
            if !names.insert(pin.path.to_ascii_lowercase()) {
                return Err(invalid("duplicate tool source path"));
            }
            relative(&pin.path)?;
            let source_pin = PinnedFile {
                path: format!("source/{}", pin.path),
                sha256: pin.sha256.clone(),
            };
            let bytes = pinned(&root, &source_pin, 131_072)?;
            total += bytes.len();
            if total > 524_288 || bytes.contains(&0) {
                return Err(invalid("tool source exceeds its content limits"));
            }
            source.push(SourceFile {
                path: pin.path.clone(),
                content: String::from_utf8(bytes)
                    .map_err(|_| invalid("tool source requires UTF-8"))?,
            });
        }
        let input = schema(&pinned(&root, &manifest.input_schema, 262_144)?)?;
        let output = schema(&pinned(&root, &manifest.output_schema, 262_144)?)?;
        let provenance = pinned(&root, &manifest.verification, 1_048_576)?;
        strict_json_exact(&provenance, JsonLimits::default()).map_err(Error::InvalidInput)?;
        Ok(LoadedPackage {
            manifest,
            root,
            source,
            input,
            output,
        })
    }
}
impl LoadedPackage {
    pub fn validate_request(&self, bytes: &[u8]) -> Result<AcquisitionRequest, Error> {
        let value = strict_json_exact(
            bytes,
            JsonLimits {
                max_bytes: 1_048_576,
                ..Default::default()
            },
        )
        .map_err(Error::InvalidInput)?;
        let request: AcquisitionRequest = serde_json::from_value(value)?;
        request.validate().map_err(Error::InvalidInput)?;
        if !eal_collector_contracts::identifier(&request.tool)
            || request.tool_version != self.manifest.version
        {
            return Err(invalid(
                "request tool identity is invalid for the deployed package",
            ));
        }
        if !jsonschema::validator_for(&self.input)
            .map_err(|_| invalid("invalid input schema"))?
            .is_valid(&request.input)
        {
            return Err(invalid("tool input differs from its pinned schema"));
        }
        Ok(request)
    }
    pub fn validate_output(
        &self,
        request: &AcquisitionRequest,
        bytes: &[u8],
    ) -> Result<Value, Error> {
        let envelope = crate::observation::validate_observation(
            &self.output,
            self.manifest.require_observed_at,
            request,
            bytes,
            self.manifest.profile.output_bytes,
        )?;
        let value = serde_json::to_value(envelope)?;
        let secrets = self
            .manifest
            .profile
            .inherit_env
            .iter()
            .map(|name| match std::env::var(name) {
                Ok(v) => Ok(v),
                Err(std::env::VarError::NotPresent) => Ok(String::new()),
                Err(_) => Err(invalid("tool environment requires UTF-8")),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let sensitive = secrets
            .iter()
            .filter(|s| !s.is_empty())
            .map(String::as_str)
            .collect::<Vec<_>>();
        if contains_sensitive_json(&value, &sensitive, SensitiveJsonLimits::default())
            .map_err(|e| Error::InvalidOutput(e.to_string()))?
        {
            return Err(Error::InvalidOutput(
                "tool output contains a forwarded credential".into(),
            ));
        }
        Ok(value)
    }
    pub fn execute(
        &self,
        bytes: &[u8],
        supervisor: &Path,
        cancellation: &CancellationToken,
    ) -> Result<Value, Error> {
        let request = self.validate_request(bytes)?;
        if cancellation.is_cancelled() {
            return Err(Error::ToolExecution("tool execution cancelled".into()));
        }
        let mut profile = self.manifest.profile.clone();
        // The portable package pins the image, even if the original authoring tag moves.
        profile.image = self.manifest.image_id.clone();
        let runner = DockerRunner::new(profile, supervisor.to_path_buf())?;
        if runner.identity().image_id != self.manifest.image_id {
            return Err(invalid("tool image differs from its immutable pin"));
        }
        let outcome = runner.execute(
            &self.root,
            &self.source,
            &[],
            bytes,
            ExecutionMode::Runtime,
            cancellation,
        )?;
        if cancellation.is_cancelled()
            || outcome.returncode != 0
            || outcome.execution_error.is_some()
        {
            return Err(Error::ToolExecution(
                "isolated tool did not complete a successful acquisition".into(),
            ));
        }
        self.validate_output(&request, &outcome.stdout)
    }
}
