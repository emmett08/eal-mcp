//! Versioned descriptions of prospective measurements and operator-owned bindings.
use crate::{ACQUISITION_CONTRACT, CommandBinding, CommandTemplate};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const CATALOGUE_SCHEMA: &str = "eal.collector-catalogue/1";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Catalogue {
    pub schema: String,
    pub acquisition_contract: String,
    pub collectors: Vec<CollectorDescriptor>,
}
impl Catalogue {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != CATALOGUE_SCHEMA || self.acquisition_contract != ACQUISITION_CONTRACT {
            return Err("Unsupported collector catalogue or acquisition contract version".into());
        }
        if self.collectors.len() > 256 {
            return Err("Collector catalogue exceeds 256 entries".into());
        }
        let mut ids = BTreeSet::new();
        for collector in &self.collectors {
            collector.validate()?;
            if !ids.insert(&collector.id) {
                return Err("Collector catalogue repeats an identifier".into());
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CollectorDescriptor {
    pub id: String,
    pub version: String,
    pub purpose: String,
    pub parameters: Vec<String>,
    pub input_schema: Value,
    pub result_schema: Value,
    pub command: CommandTemplate,
    pub acquisition_predicate: String,
    #[serde(default)]
    pub constraints: Vec<InputConstraint>,
}
impl CollectorDescriptor {
    /// Unconditionally required top-level input fields and literal command parameters.
    /// Conditional schema requirements still apply when validating the complete input.
    pub fn required_parameters(&self) -> Result<Vec<String>, String> {
        self.validate()?;
        let mut required = self
            .command
            .parameter_names()?
            .into_iter()
            .collect::<BTreeSet<_>>();
        if let Some(fields) = self.input_schema.get("required").and_then(Value::as_array) {
            for field in fields {
                let name = field
                    .as_str()
                    .ok_or("Invalid required collector parameter")?;
                if !self.parameters.iter().any(|p| p == name) {
                    return Err("Required collector parameter is undeclared".into());
                }
                required.insert(name.into());
            }
        }
        Ok(required.into_iter().collect())
    }
    pub fn validate(&self) -> Result<(), String> {
        if crate::CollectorId::parse(self.id.clone()).is_err()
            || self.version.is_empty()
            || self.version.chars().count() > 1024
            || self.version.chars().any(char::is_control)
            || self.purpose.trim().is_empty()
            || self.purpose.chars().count() > 8192
            || self.parameters.len() > 256
            || self
                .parameters
                .iter()
                .any(|v| !crate::identifier(v) || v.starts_with("operator."))
            || self.parameters.iter().collect::<BTreeSet<_>>().len() != self.parameters.len()
            || self.acquisition_predicate.trim().is_empty()
            || self.acquisition_predicate.chars().count() > 8192
            || self.acquisition_predicate.chars().any(char::is_control)
        {
            return Err("Invalid collector descriptor identity or parameters".into());
        }
        let properties = self
            .input_schema
            .get("properties")
            .and_then(Value::as_object);
        if self.input_schema.get("type").and_then(Value::as_str) != Some("object")
            || self.input_schema.get("additionalProperties") != Some(&Value::Bool(false))
            || properties.is_none()
            || properties.is_some_and(|p| p.keys().any(|key| !self.parameters.contains(key)))
            || !self.result_schema.is_object()
        {
            return Err(
                "Collector schemas require explicit object inputs drawn from parameters".into(),
            );
        }
        crate::schema::validate_schema(&self.input_schema)?;
        crate::schema::validate_schema(&self.result_schema)?;
        self.command.validate(&self.parameters)?;
        for constraint in &self.constraints {
            match constraint {
                InputConstraint::OrderedWindow {
                    start,
                    end,
                    max_seconds,
                } => {
                    let properties = properties.expect("checked properties");
                    if start == end
                        || *max_seconds == 0
                        || !properties.contains_key(start)
                        || !properties.contains_key(end)
                        || properties[start].get("type").and_then(Value::as_str) != Some("integer")
                        || properties[end].get("type").and_then(Value::as_str) != Some("integer")
                    {
                        return Err("Window constraints require distinct integer input fields and a positive allowance".into());
                    }
                }
            }
        }
        Ok(())
    }
    /// Produce explicitly prospective configuration without granting execution.
    pub fn prepare_binding(
        &self,
        parameters: &BTreeMap<String, String>,
        unresolved: &[String],
    ) -> Result<CommandBinding, String> {
        self.validate_for(crate::AuthoringProfile::RuntimeExportable)?;
        if parameters
            .keys()
            .any(|name| !self.parameters.contains(name))
            || unresolved
                .iter()
                .any(|name| !self.parameters.contains(name) || parameters.contains_key(name))
            || unresolved.iter().collect::<BTreeSet<_>>().len() != unresolved.len()
        {
            return Err("EAL_PREPARATION_PARAMETER: unknown collector parameter".into());
        }
        self.command.prepare(&self.version, parameters, unresolved)
    }
    /// Bind reviewed suggestions and separately supplied operator selections.
    /// This produces configuration only and performs no acquisition.
    pub fn binding(
        &self,
        parameters: &BTreeMap<String, String>,
        operator_selections: &BTreeMap<String, String>,
    ) -> Result<CommandBinding, String> {
        self.validate()?;
        if parameters.keys().any(|key| !self.parameters.contains(key)) {
            return Err("Unknown collector parameter".into());
        }
        self.command
            .bind(&self.version, parameters, operator_selections)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum InputConstraint {
    OrderedWindow {
        start: String,
        end: String,
        max_seconds: u64,
    },
}
