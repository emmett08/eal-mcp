//! Explicit consumer restrictions layered over the published wire contract.
use crate::{Catalogue, CollectorDescriptor, RUNTIME_EXPORTABLE_PROFILE};
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthoringProfile {
    RuntimeExportable,
}
impl AuthoringProfile {
    pub fn id(self) -> &'static str {
        match self {
            Self::RuntimeExportable => RUNTIME_EXPORTABLE_PROFILE,
        }
    }
}
impl Catalogue {
    pub fn validate_for(&self, profile: AuthoringProfile) -> Result<(), String> {
        self.validate()?;
        for collector in &self.collectors {
            collector.validate_profile(profile)?;
        }
        Ok(())
    }
}
impl CollectorDescriptor {
    pub fn validate_for(&self, profile: AuthoringProfile) -> Result<(), String> {
        self.validate()?;
        self.validate_profile(profile)
    }
    fn validate_profile(&self, _profile: AuthoringProfile) -> Result<(), String> {
        crate::validate_environment_names(&self.command.inherit_env, 64)?;
        if [
            "custom",
            "artifact_json",
            "http_json_get",
            "scoped_json",
            "kubectl_get",
            "gh_api_get",
        ]
        .contains(&self.id.as_str())
        {
            return Err(
                "EAL_PROFILE_RESERVED_ID: collector ID collides with a legacy adapter".into(),
            );
        }
        checked_predicate(&self.acquisition_predicate)
    }
    pub fn validate_input(&self, input: &Value) -> Result<(), String> {
        self.validate_prepared_input(input, &[])
    }
    /// Unresolved properties are explicitly prospective. A supplied value must
    /// never be listed as unresolved by the authoring caller.
    pub fn validate_prepared_input(
        &self,
        input: &Value,
        unresolved: &[String],
    ) -> Result<(), String> {
        self.validate()?;
        if unresolved
            .iter()
            .any(|name| !self.parameters.contains(name))
            || unresolved
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != unresolved.len()
        {
            return Err(
                "EAL_PREPARATION_PARAMETER: unresolved parameters must be distinct declared names"
                    .into(),
            );
        }
        let mut schema = self.input_schema.clone();
        for name in unresolved {
            if input.get(name).is_some_and(|value| !value.is_null()) {
                return Err(
                    "EAL_PREPARATION_PARAMETER: resolved values cannot be exempted from validation"
                        .into(),
                );
            }
            schema["properties"][name] = serde_json::json!({});
        }
        let validator = jsonschema::validator_for(&schema)
            .map_err(|_| "EAL_SCHEMA_INPUT: invalid collector input schema")?;
        if !validator.is_valid(input) {
            return Err("EAL_INPUT_SCHEMA: input differs from collector schema".into());
        }
        for constraint in &self.constraints {
            match constraint {
                crate::InputConstraint::OrderedWindow {
                    start,
                    end,
                    max_seconds,
                } => {
                    let endpoint = |name: &str| -> Result<Option<u64>, String> {
                        if unresolved.iter().any(|parameter| parameter == name) {
                            Ok(None)
                        } else {
                            input[name].as_u64().map(Some).ok_or_else(|| {
                                "EAL_INPUT_WINDOW: endpoints require unsigned 64-bit seconds".into()
                            })
                        }
                    };
                    if let (Some(start), Some(end)) = (endpoint(start)?, endpoint(end)?)
                        && (end <= start || end - start > *max_seconds)
                    {
                        return Err("EAL_INPUT_WINDOW: invalid ordered measurement window".into());
                    }
                }
            }
        }
        Ok(())
    }
}
/// The runtime-exportable authoring profile admits one scalar comparison.
/// Descriptor text never supplies executable EAL syntax beyond this subset.
pub fn checked_predicate(text: &str) -> Result<(), String> {
    let parts: Vec<_> = text.splitn(3, ' ').collect();
    let path_component = |component: &str| crate::environment_name(component);
    if parts.len() != 3
        || !parts[0].split('.').all(path_component)
        || !["==", "!=", ">=", "<=", ">", "<"].contains(&parts[1])
        || text.chars().any(char::is_control)
    {
        return Err("EAL_PROFILE_PREDICATE: one scalar comparison is required".into());
    }
    let value = crate::strict_json(
        parts[2].as_bytes(),
        crate::JsonLimits {
            max_bytes: 8192,
            max_depth: 4,
            max_nodes: 4,
        },
    )?;
    if value.is_array() || value.is_object() || value.is_null() {
        return Err("EAL_PROFILE_PREDICATE: a finite scalar comparison value is required".into());
    }
    Ok(())
}
