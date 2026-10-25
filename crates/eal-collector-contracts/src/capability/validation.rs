//! Admission of qualified measurement descriptions without external IO.
use super::{CAPABILITY_CATALOGUE_SCHEMA, SUPPORT_CONTRACT_SCHEMA, types::*};
use std::collections::BTreeSet;

fn text(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}
fn identifier(value: &str) -> bool {
    value.len() <= 256 && crate::identifier(value)
}
fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
fn unique_text(values: &[String], max: usize) -> bool {
    values.len() <= max
        && values.iter().all(|value| text(value, 1024))
        && values.iter().collect::<BTreeSet<_>>().len() == values.len()
}
fn methods(values: &[String]) -> bool {
    !values.is_empty()
        && unique_text(values, 128)
        && values.iter().all(|value| {
            value.len() <= 256
                && value.split_once('/').is_some_and(|(name, version)| {
                    identifier(name)
                        && !version.is_empty()
                        && version
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
                })
        })
}
fn pointer(value: &str) -> bool {
    if value.len() > 4096
        || (!value.is_empty() && !value.starts_with('/'))
        || value.chars().any(char::is_control)
    {
        return false;
    }
    let mut bytes = value.bytes();
    while let Some(byte) = bytes.next() {
        if byte == b'~' && !matches!(bytes.next(), Some(b'0' | b'1')) {
            return false;
        }
    }
    true
}

impl SubjectScope {
    pub fn validate(&self) -> Result<(), String> {
        if !identifier(&self.kind)
            || !text(&self.subject, 8192)
            || !unique_text(&self.identity_dimensions, 128)
            || self.selectors.len() > 128
            || self.selectors.iter().any(|(key, value)| {
                !text(key, 1024)
                    || value.is_null()
                    || value.as_str().is_some_and(|value| value.trim().is_empty())
            })
        {
            return Err("Invalid support subject scope".into());
        }
        super::canonical_identity(&self.selectors)?;
        Ok(())
    }
}
impl SupportContract {
    /// Report independent defects together, so one repair need not expose the next.
    pub fn diagnostics(&self) -> Vec<ContractDiagnostic> {
        let mut errors = Vec::new();
        let mut issue = |path: &str, code: &str, message: String| {
            errors.push(ContractDiagnostic {
                path: path.into(),
                code: code.into(),
                message,
            });
        };
        if self.schema != SUPPORT_CONTRACT_SCHEMA {
            issue(
                "/schema",
                "support_contract_schema",
                format!(
                    "expected {SUPPORT_CONTRACT_SCHEMA:?}, received {:?}",
                    self.schema
                ),
            );
        }
        if let Err(error) = self.scope.validate() {
            issue("/scope", "support_contract_scope", error);
        }
        if !methods(&self.acceptable_methods) {
            issue(
                "/acceptable_methods",
                "support_contract_methods",
                format!(
                    "expected 1–128 distinct qualified method identities in name/version format, each at most 256 bytes; received {} entries",
                    self.acceptable_methods.len()
                ),
            );
        }
        if self.quantities.len() > 128
            || self
                .quantities
                .iter()
                .any(|quantity| !text(&quantity.name, 1024) || !text(&quantity.unit, 128))
            || self
                .quantities
                .iter()
                .map(|quantity| &quantity.name)
                .collect::<BTreeSet<_>>()
                .len()
                != self.quantities.len()
        {
            issue(
                "/quantities",
                "support_contract_quantities",
                "expected at most 128 distinct quantity names and explicit units".into(),
            );
        }
        if self
            .population
            .as_deref()
            .is_some_and(|value| !text(value, 4096))
            || (self.logical_strength == LogicalStrength::CompleteEnumeration
                && self.population.is_none())
        {
            issue("/population", "support_contract_population", "complete enumeration requires an explicit nonblank population of at most 4096 bytes".into());
        }
        if self
            .time
            .historical_interval
            .as_ref()
            .is_some_and(|interval| interval.end_unix <= interval.start_unix)
        {
            issue(
                "/time/historical_interval",
                "support_contract_time",
                "expected end_unix greater than start_unix".into(),
            );
        }
        if !unique_text(&self.provenance.required_authorities, 128) {
            issue(
                "/provenance/required_authorities",
                "support_contract_provenance",
                "expected at most 128 distinct nonblank authority identities".into(),
            );
        }
        if self
            .expected_output_schema_sha256
            .as_deref()
            .is_some_and(|value| !digest(value))
        {
            issue(
                "/expected_output_schema_sha256",
                "support_contract_output_schema",
                "expected null or a 64-character lowercase hexadecimal SHA-256".into(),
            );
        }
        errors
    }
    pub fn validate(&self) -> Result<(), String> {
        let errors = self.diagnostics();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; "))
        }
    }
}

impl CapabilityDescriptor {
    pub fn validate(&self) -> Result<(), String> {
        if !identifier(&self.id)
            || !text(&self.version, 1024)
            || !text(&self.description, 8192)
            || !identifier(&self.provider.id)
            || !text(&self.provider.version, 1024)
            || !text(&self.provider.authentication_scope, 1024)
            || !digest(&self.provider.configuration_sha256)
            || self
                .provider
                .connection
                .as_deref()
                .is_some_and(|value| !identifier(value))
            || self
                .provider
                .tool
                .as_deref()
                .is_some_and(|value| !text(value, 256))
        {
            return Err("Invalid capability or provider identity".into());
        }
        if matches!(
            self.provider.transport,
            TransportKind::McpStdio | TransportKind::McpHttp
        ) && (self.provider.connection.is_none() || self.provider.tool.is_none())
        {
            return Err("MCP capabilities require a connection alias and exact tool name".into());
        }
        if !digest(&self.input_schema_sha256)
            || !digest(&self.output_schema_sha256)
            || super::schema_identity(&self.input_schema)? != self.input_schema_sha256
            || super::schema_identity(&self.output_schema)? != self.output_schema_sha256
        {
            return Err("Capability schema identity differs from its schemas".into());
        }
        if self
            .input_schema
            .get("type")
            .and_then(serde_json::Value::as_str)
            != Some("object")
        {
            return Err("Capability inputs require an object schema".into());
        }
        if !(1..=3600).contains(&self.limits.timeout_seconds)
            || !(1..=16_777_216).contains(&self.limits.output_bytes)
            || !(1..=1024).contains(&self.limits.concurrency)
            || self.limits.requests_per_minute == Some(0)
        {
            return Err("Capability execution limits exceed their allowances".into());
        }
        if self.configuration_requirements.len() > 128
            || self.configuration_requirements.iter().any(|requirement| {
                !identifier(&requirement.id) || !text(&requirement.description, 8192)
            })
            || self
                .configuration_requirements
                .iter()
                .map(|requirement| &requirement.id)
                .collect::<BTreeSet<_>>()
                .len()
                != self.configuration_requirements.len()
        {
            return Err(
                "Capability configuration prerequisites require distinct bounded identities".into(),
            );
        }
        for estimate in [
            &self.costs.configuration_effort_seconds,
            &self.costs.implementation_effort_seconds,
            &self.costs.execution_requests,
            &self.costs.execution_latency_ms,
            &self.costs.monetary_microunits,
            &self.costs.transfer_bytes,
        ] {
            if !text(&estimate.reference, 4096)
                || (estimate.value.is_none() != (estimate.basis == EstimateBasis::Unknown))
            {
                return Err(
                    "Cost estimates require their basis and an explicit unknown state".into(),
                );
            }
        }
        if let Some(semantics) = &self.semantics {
            semantics.validate()?;
        }
        Ok(())
    }
}

impl MeasurementSemantics {
    pub fn validate(&self) -> Result<(), String> {
        if self.time.historical_input.as_ref().is_some_and(|binding| {
            !self.time.historical_intervals
                || binding.start_pointer == binding.end_pointer
                || !pointer(&binding.start_pointer)
                || !pointer(&binding.end_pointer)
        }) {
            return Err("Historical input bindings require distinct valid pointers and historical semantics".into());
        }
        if !text(&self.qualification_reference, 4096)
            || self.support_types.is_empty()
            || self.support_types.len() > 7
            || self.support_types.iter().collect::<BTreeSet<_>>().len() != self.support_types.len()
            || !unique_text(&self.identity_dimensions, 128)
            || self.scope_constraints.len() > 128
            || self.scope_constraints.iter().any(|(key, value)| {
                !text(key, 1024)
                    || value.is_null()
                    || value.as_str().is_some_and(|value| value.trim().is_empty())
            })
        {
            return Err("Invalid qualified measurement semantics or scope".into());
        }
        if self.subject_kinds.is_empty()
            || !unique_text(&self.subject_kinds, 128)
            || self.subject_kinds.iter().any(|kind| !identifier(kind))
            || !methods(&self.methods)
        {
            return Err(
                "Measurement semantics require subject kinds and qualified method identities"
                    .into(),
            );
        }
        if self.scope_bindings.len() > 128
            || self
                .scope_bindings
                .iter()
                .any(|(dimension, input_pointer)| !text(dimension, 1024) || !pointer(input_pointer))
        {
            return Err("Scope bindings require bounded dimensions and input JSON pointers".into());
        }
        super::canonical_identity(&self.scope_constraints)?;
        if self.quantities.len() > 128
            || self.quantities.iter().any(|quantity| {
                !text(&quantity.name, 1024)
                    || !text(&quantity.unit, 128)
                    || !pointer(&quantity.output_pointer)
            })
            || self
                .quantities
                .iter()
                .map(|quantity| &quantity.name)
                .collect::<BTreeSet<_>>()
                .len()
                != self.quantities.len()
        {
            return Err(
                "Measurement quantities require distinct names, units and JSON pointers".into(),
            );
        }
        if self.coverage.is_empty() || self.coverage.len() > 32 {
            return Err("Measurement semantics require bounded coverage guarantees".into());
        }
        let mut coverage = BTreeSet::new();
        for guarantee in &self.coverage {
            let identity =
                serde_json::to_string(guarantee).map_err(|_| "Cannot encode coverage guarantee")?;
            if !coverage.insert(identity)
                || guarantee
                    .population
                    .as_deref()
                    .is_some_and(|value| !text(value, 4096))
                || guarantee
                    .completeness_pointer
                    .as_deref()
                    .is_some_and(|value| !pointer(value))
                || guarantee
                    .sample_size_pointer
                    .as_deref()
                    .is_some_and(|value| !pointer(value))
                || (guarantee.strength == LogicalStrength::StatisticalEstimate
                    && guarantee.sample_size_pointer.is_none())
                || (guarantee.strength == LogicalStrength::CompleteEnumeration
                    && (guarantee.population.is_none() || guarantee.completeness_pointer.is_none()))
            {
                return Err(
                    "Complete enumeration requires an explicit population and completeness output"
                        .into(),
                );
            }
        }
        if !unique_text(&self.provenance.authorities, 128) || !unique_text(&self.limitations, 128) {
            return Err(
                "Measurement provenance and limitations require distinct bounded values".into(),
            );
        }
        Ok(())
    }
}

impl CapabilityCatalogue {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != CAPABILITY_CATALOGUE_SCHEMA || self.capabilities.len() > 256 {
            return Err("Unsupported capability catalogue or excessive capability count".into());
        }
        let mut ids = BTreeSet::new();
        for capability in &self.capabilities {
            capability.validate()?;
            if !ids.insert(&capability.id) {
                return Err("Capability catalogue repeats an identifier".into());
            }
        }
        super::canonical_identity(self)?;
        Ok(())
    }
}
