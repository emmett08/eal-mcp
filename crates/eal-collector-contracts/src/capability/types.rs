//! Versioned measurement requirements and qualified provider descriptions.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CapabilityCatalogue {
    pub schema: String,
    pub capabilities: Vec<CapabilityDescriptor>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CapabilityDescriptor {
    pub id: String,
    pub version: String,
    pub description: String,
    pub provider: ProviderIdentity,
    #[serde(deserialize_with = "crate::deserialize_value_exact")]
    #[schemars(with = "Value")]
    pub input_schema: Value,
    pub input_schema_sha256: String,
    #[serde(deserialize_with = "crate::deserialize_value_exact")]
    #[schemars(with = "Value")]
    pub output_schema: Value,
    pub output_schema_sha256: String,
    /// Omitted qualification is an interface candidate, never an adequate route.
    pub semantics: Option<MeasurementSemantics>,
    pub configuration_requirements: Vec<ConfigurationPrerequisite>,
    pub limits: CapabilityLimits,
    pub costs: CapabilityCosts,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProviderIdentity {
    pub id: String,
    pub version: String,
    pub transport: TransportKind,
    pub connection: Option<String>,
    pub tool: Option<String>,
    pub authentication_scope: String,
    pub configuration_sha256: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TransportKind {
    Command,
    Http,
    McpStdio,
    McpHttp,
    RetainedArtifact,
    InProcess,
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum SupportType {
    Observation,
    SourceRetrieval,
    Calculation,
    Experiment,
    ProofCheck,
    Clarification,
    Judgement,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LogicalStrength {
    ExistentialWitness,
    CompleteEnumeration,
    StatisticalEstimate,
    ExactCalculation,
    CheckedProof,
    Clarification,
    Judgement,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SubjectScope {
    /// Explicit domain entity category, such as service_instance or numerical_expression.
    pub kind: String,
    pub subject: String,
    /// Exact operator/request scope. Empty values cannot stand for unknown selectors.
    #[serde(deserialize_with = "crate::deserialize_value_map_exact")]
    #[schemars(with = "BTreeMap<String, Value>")]
    pub selectors: BTreeMap<String, Value>,
    pub identity_dimensions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QuantityRequirement {
    pub name: String,
    pub unit: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MeasurementQuantity {
    pub name: String,
    pub unit: String,
    pub output_pointer: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HistoricalInterval {
    pub start_unix: i64,
    pub end_unix: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TimeRequirement {
    pub historical_interval: Option<HistoricalInterval>,
    pub max_age_seconds: Option<u64>,
    pub max_uncertainty_seconds: Option<u64>,
    pub require_original_observation_time: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TimeSemantics {
    pub historical_intervals: bool,
    /// Qualified argument bindings for the exact historical query window.
    pub historical_input: Option<HistoricalInputBinding>,
    pub original_observation_time: bool,
    /// Qualified upper bound on measurement uncertainty, not execution latency.
    pub uncertainty_seconds: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HistoricalInputBinding {
    pub start_pointer: String,
    pub end_pointer: String,
    pub encoding: HistoricalTimeEncoding,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HistoricalTimeEncoding {
    UnixSeconds,
    UnixSecondsString,
    Rfc3339,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceRequirement {
    pub required_authorities: Vec<String>,
    pub transformation_history: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceSemantics {
    pub authorities: Vec<String>,
    pub transformation_history: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SupportContract {
    #[schemars(schema_with = "support_contract_identity_schema")]
    pub schema: String,
    pub support_type: SupportType,
    pub scope: SubjectScope,
    /// Qualified protocol identifiers. At least one offered protocol must match.
    #[schemars(schema_with = "support_methods_schema")]
    pub acceptable_methods: Vec<String>,
    pub quantities: Vec<QuantityRequirement>,
    pub logical_strength: LogicalStrength,
    pub population: Option<String>,
    pub time: TimeRequirement,
    pub provenance: ProvenanceRequirement,
    pub expected_output_schema_sha256: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MeasurementSemantics {
    /// This reference identifies an operator-qualified contract, not tools/list prose.
    pub qualification_reference: String,
    pub support_types: Vec<SupportType>,
    pub subject_kinds: Vec<String>,
    pub methods: Vec<String>,
    pub identity_dimensions: Vec<String>,
    /// Fixed provider constraints, checked against exact requirement scope values.
    #[serde(deserialize_with = "crate::deserialize_value_map_exact")]
    #[schemars(with = "BTreeMap<String, Value>")]
    pub scope_constraints: BTreeMap<String, Value>,
    /// Every request selector affecting source scope maps to its input JSON pointer.
    pub scope_bindings: BTreeMap<String, String>,
    pub quantities: Vec<MeasurementQuantity>,
    pub coverage: Vec<CoverageGuarantee>,
    pub time: TimeSemantics,
    pub provenance: ProvenanceSemantics,
    pub limitations: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CoverageGuarantee {
    pub strength: LogicalStrength,
    pub population: Option<String>,
    /// Qualified quantity containing the complete enumeration of this population.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub population_quantity: Option<String>,
    /// Exhaustive routes must return the measured completeness indicator.
    pub completeness_pointer: Option<String>,
    /// A statistical route must expose the positive measured sample count.
    #[serde(default)]
    pub sample_size_pointer: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConfigurationKind {
    Connection,
    Selector,
    CredentialReference,
    SourceRecord,
    OperatorValue,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationPrerequisite {
    pub id: String,
    pub kind: ConfigurationKind,
    pub description: String,
    pub required: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CapabilityLimits {
    pub timeout_seconds: u64,
    pub output_bytes: u64,
    pub concurrency: u32,
    pub requests_per_minute: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EstimateBasis {
    Declared,
    Estimated,
    Measured,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CostEstimate {
    pub value: Option<u64>,
    pub basis: EstimateBasis,
    pub reference: String,
}

impl CostEstimate {
    pub fn unknown(reference: impl Into<String>) -> Self {
        Self {
            value: None,
            basis: EstimateBasis::Unknown,
            reference: reference.into(),
        }
    }

    pub fn declared(value: u64, reference: impl Into<String>) -> Self {
        Self {
            value: Some(value),
            basis: EstimateBasis::Declared,
            reference: reference.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CapabilityCosts {
    pub configuration_effort_seconds: CostEstimate,
    pub implementation_effort_seconds: CostEstimate,
    pub execution_requests: CostEstimate,
    pub execution_latency_ms: CostEstimate,
    pub monetary_microunits: CostEstimate,
    pub transfer_bytes: CostEstimate,
}

impl CapabilityCosts {
    pub fn unknown(reference: &str) -> Self {
        Self {
            configuration_effort_seconds: CostEstimate::unknown(reference),
            implementation_effort_seconds: CostEstimate::unknown(reference),
            execution_requests: CostEstimate::unknown(reference),
            execution_latency_ms: CostEstimate::unknown(reference),
            monetary_microunits: CostEstimate::unknown(reference),
            transfer_bytes: CostEstimate::unknown(reference),
        }
    }
}

fn support_contract_identity_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!({"type":"string","const":super::SUPPORT_CONTRACT_SCHEMA})
}

fn support_methods_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!({
        "type":"array","minItems":1,"maxItems":128,"uniqueItems":true,
        "items":{"type":"string","maxLength":256,
            "pattern":"^[A-Za-z_][A-Za-z0-9_.-]*/[A-Za-z0-9._-]+$"}
    })
}

/// One independently detected contract defect, relative to the contract root.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractDiagnostic {
    pub path: String,
    pub code: String,
    pub message: String,
}
impl std::fmt::Display for ContractDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}
