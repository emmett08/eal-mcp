use super::{AcquisitionError, AcquisitionResult, ToolBinding};
use std::{collections::BTreeMap, path::Path};

#[derive(Clone, Debug, Default)]
pub struct ToolRegistry {
    pub bindings: BTreeMap<String, ToolBinding>,
}
impl ToolRegistry {
    pub fn new(bindings: BTreeMap<String, ToolBinding>) -> Result<Self, AcquisitionError> {
        for (name, binding) in &bindings {
            if name != &binding.name {
                return Err("Registry keys must match their binding names".into());
            }
            validate_binding(binding)?;
        }
        Ok(Self { bindings })
    }
    pub fn load(path: impl AsRef<Path>) -> Result<Self, AcquisitionError> {
        let path = super::files::resolve(path.as_ref())?;
        let contents =
            super::files::read_regular(&path, 1024 * 1024, "The registry must be a regular file")?;
        let text = std::str::from_utf8(&contents)
            .map_err(|_| AcquisitionError::invalid("The registry must contain UTF-8 TOML"))?;
        Self::from_toml(text)
    }
    pub fn from_toml(text: &str) -> Result<Self, AcquisitionError> {
        let document = eal_collector_contracts::RegistryDocument::from_toml(text)
            .map_err(AcquisitionError::invalid)?;
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
                    },
                )
            })
            .collect();
        Self::new(bindings)
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
    }
    .validate()
    .map_err(AcquisitionError::invalid)
}
