//! Discovery metadata has explicit scope correspondence and never selects a population.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DependencyRole {
    #[default]
    Selection,
    /// Source selector JSON pointers map to equal consumer selector values.
    /// The planner additionally checks source contract classification and lineage.
    Discovery {
        scope_bindings: BTreeMap<String, String>,
    },
}
impl DependencyRole {
    pub fn is_selection(&self) -> bool {
        matches!(self, Self::Selection)
    }

    /// Validate every source selector, including those not explicitly renamed.
    /// Shared selectors must agree even if a different mapping is supplied.
    pub fn validate_scopes(&self, source: &Value, target: &Value) -> Result<(), String> {
        let Self::Discovery { scope_bindings } = self else {
            return Ok(());
        };
        let source = source
            .as_object()
            .ok_or("discovery source scope must be an object")?;
        let target = target
            .as_object()
            .ok_or("discovery consumer scope must be an object")?;
        if scope_bindings.is_empty() || scope_bindings.len() > 128 {
            return Err("discovery requires 1–128 explicit scope correspondences".into());
        }
        let source_value = Value::Object(source.clone());
        let target_value = Value::Object(target.clone());
        for (from, to) in scope_bindings {
            if from.is_empty()
                || to.is_empty()
                || !super::valid_pointer(from)
                || !super::valid_pointer(to)
            {
                return Err(
                    "discovery scope correspondences require non-root JSON pointers".into(),
                );
            }
            let supplied = source_value
                .pointer(from)
                .filter(|value| !value.is_null())
                .ok_or_else(|| {
                    format!("discovery source scope pointer {from:?} is missing or null")
                })?;
            let expected = target_value
                .pointer(to)
                .filter(|value| !value.is_null())
                .ok_or_else(|| {
                    format!("discovery consumer scope pointer {to:?} is missing or null")
                })?;
            if supplied != expected {
                return Err(format!(
                    "discovery scope correspondence {from:?} to {to:?} has unequal values"
                ));
            }
        }
        for (key, supplied) in source {
            if supplied.is_null() {
                return Err(format!("discovery source selector {key:?} is null"));
            }
            if let Some(expected) = target.get(key) {
                if supplied != expected {
                    return Err(format!("shared discovery scope selector {key:?} differs"));
                }
            } else {
                let pointer = format!("/{}", key.replace('~', "~0").replace('/', "~1"));
                if !scope_bindings.contains_key(&pointer) {
                    return Err(format!(
                        "discovery source selector {key:?} requires an explicit consumer correspondence"
                    ));
                }
            }
        }
        Ok(())
    }
}
