use super::{AcquisitionError, AcquisitionResult, ToolBinding};
use std::{collections::BTreeMap, path::Path};

#[derive(Clone, Debug)]
pub struct ToolRegistry {
    pub context: serde_json::Value,
    pub bindings: BTreeMap<String, ToolBinding>,
}
impl Default for ToolRegistry {
    fn default() -> Self {
        Self {
            context: serde_json::json!({}),
            bindings: BTreeMap::new(),
        }
    }
}
impl ToolRegistry {
    pub fn new(bindings: BTreeMap<String, ToolBinding>) -> Result<Self, AcquisitionError> {
        for (name, binding) in &bindings {
            if name != &binding.name {
                return Err("Registry keys must match their binding names".into());
            }
            validate_binding(binding)?;
        }
        Ok(Self {
            bindings,
            context: serde_json::json!({}),
        })
    }
    pub fn load(path: impl AsRef<Path>) -> Result<Self, AcquisitionError> {
        Self::load_snapshot(path).map(|(registry, _)| registry)
    }
    /// Registry and digest describe exactly the same bounded byte snapshot.
    pub(crate) fn load_snapshot(
        path: impl AsRef<Path>,
    ) -> Result<(Self, String), AcquisitionError> {
        use sha2::{Digest, Sha256};
        let path = super::files::resolve(path.as_ref())?;
        let contents =
            super::files::read_regular(&path, 1024 * 1024, "The registry must be a regular file")?;
        let text = std::str::from_utf8(&contents)
            .map_err(|_| AcquisitionError::invalid("The registry must contain UTF-8 TOML"))?;
        let mut registry = Self::from_toml(text)?;
        for binding in registry.bindings.values_mut() {
            binding.package_root = path.parent().map(Path::to_path_buf);
        }
        Ok((registry, format!("{:x}", Sha256::digest(&contents))))
    }
    pub fn from_toml(text: &str) -> Result<Self, AcquisitionError> {
        let document = eal_collector_contracts::RegistryDocument::from_toml(text)
            .map_err(AcquisitionError::invalid)?;
        let context = serde_json::to_value(&document.context)
            .map_err(|_| AcquisitionError::invalid("Invalid manifest context"))?;
        let bindings = document
            .tools
            .into_iter()
            .map(|(name, raw)| {
                (
                    name.clone(),
                    ToolBinding {
                        name,
                        kind: raw.kind,
                        version: raw.version,
                        argv: raw.argv,
                        path: raw.path,
                        timeout_seconds: raw.timeout_seconds,
                        max_output_bytes: raw.max_output_bytes,
                        env: raw.env,
                        pinned_files: raw.pinned_files,
                        parallel_safe: raw.parallel_safe,
                        inherit_env: raw.inherit_env,
                        direct: raw.direct,
                        package_root: None,
                    },
                )
            })
            .collect();
        let mut registry = Self::new(bindings)?;
        registry.context = context;
        Ok(registry)
    }

    pub fn binding_for(&self, name: &str, version: &str) -> Result<&ToolBinding, AcquisitionError> {
        let binding = self.bindings.get(name).ok_or_else(|| {
            let repr = eal_core::expressions::diagnostic_repr(
                &serde_json::json!(name),
                &eal_core::ExecutionLimits::default(),
            )
            .unwrap_or_else(|_| format!("{name:?}"));
            AcquisitionError::invalid(format!("Tool {repr} is not in the operator registry"))
        })?;
        if binding.version != version {
            return Err("Declared tool version differs from the operator registry".into());
        }
        Ok(binding)
    }
    /// Admit operator configuration without invoking a command or remote source.
    /// The digest checks pinned implementation files; credentials stay private.
    pub(crate) fn admit_binding(
        &self,
        name: &str,
        version: &str,
        workspace: &Path,
        secret: &[u8],
        limits: &eal_core::ExecutionLimits,
    ) -> Result<String, AcquisitionError> {
        let binding = self.binding_for(name, version)?;
        binding.preflight(workspace)?;
        binding.binding_digest_with_limits(secret, Some(workspace), limits)
    }

    pub fn acquire(
        &self,
        binding: &ToolBinding,
        request: &serde_json::Value,
        workspace: &Path,
        secret: &[u8],
        supervisor: &crate::process::Supervisor,
        lease: Option<&crate::lease::AcquisitionLease>,
    ) -> Result<AcquisitionResult, AcquisitionError> {
        super::acquire(binding, request, workspace, secret, supervisor, lease)
    }
    // Keep the explicit policy, lease and supervisor inputs alongside the source-compatible adapter call.
    #[allow(clippy::too_many_arguments)]
    pub fn acquire_with_limits(
        &self,
        binding: &ToolBinding,
        request: &serde_json::Value,
        workspace: &Path,
        secret: &[u8],
        supervisor: &crate::process::Supervisor,
        lease: Option<&crate::lease::AcquisitionLease>,
        limits: &eal_core::ExecutionLimits,
    ) -> Result<AcquisitionResult, AcquisitionError> {
        super::acquire_with_limits(
            binding, request, workspace, secret, supervisor, lease, limits,
        )
    }
}

pub(super) fn validate_binding(binding: &ToolBinding) -> Result<(), AcquisitionError> {
    eal_collector_contracts::BindingSpec {
        name: binding.name.clone(),
        kind: binding.kind,
        version: binding.version.clone(),
        argv: binding.argv.clone(),
        path: binding.path.clone(),
        timeout_seconds: binding.timeout_seconds,
        max_output_bytes: binding.max_output_bytes,
        env: binding.env.clone(),
        pinned_files: binding.pinned_files.clone(),
        parallel_safe: binding.parallel_safe,
        inherit_env: binding.inherit_env.clone(),
        direct: binding.direct.clone(),
    }
    .validate()
    .map_err(AcquisitionError::invalid)
}
