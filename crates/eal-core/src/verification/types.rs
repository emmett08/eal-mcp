use crate::{Expression, semantic};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const CONTRACT_SCHEMA: &str = "eal.verification-contract/1";
pub const REPORT_SCHEMA: &str = "eal.verification-report/1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Mathematical,
    Empirical,
    Defeasible,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Premise,
    Grounding,
    Applicability,
    Warrant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Pass,
    Fail,
    Unknown,
    NotRequired,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationStatus {
    Available,
    #[default]
    Unavailable,
    Failed,
}

/// A closed serialization adapter for EAL's existing pure predicate AST.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Predicate {
    pub path: String,
    pub operator: String,
    #[serde(deserialize_with = "eal_collector_contracts::deserialize_value_exact")]
    pub expected: Value,
    #[serde(default, with = "crate::expressions::optional_serde")]
    pub expression: Option<Expression>,
}
impl From<Predicate> for semantic::Predicate {
    fn from(value: Predicate) -> Self {
        Self {
            path: value.path,
            operator: value.operator,
            expected: value.expected,
            expression: value.expression,
        }
    }
}
impl From<semantic::Predicate> for Predicate {
    fn from(value: semantic::Predicate) -> Self {
        Self {
            path: value.path,
            operator: value.operator,
            expected: value.expected,
            expression: value.expression,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Formula {
    pub id: String,
    #[serde(deserialize_with = "eal_collector_contracts::deserialize_value_exact")]
    pub formula: Value,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredObligation {
    pub id: String,
    pub role: Role,
}

/// Consumer-derived coverage is independent of the submitted contract's subset.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Coverage {
    pub target_id: String,
    pub mode: Mode,
    pub formulas: Vec<Formula>,
    pub required_obligations: Vec<RequiredObligation>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AtomBinding {
    pub symbol: String,
    pub observation_id: String,
    pub predicate: Predicate,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Obligation {
    pub id: String,
    pub role: Role,
    pub observation_id: String,
    pub predicate: Predicate,
}

/// Expected applicability facets are selected from an actual source value.
/// The observed timestamp is always Observation.observed_at, never a fresh query clock.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Applicability {
    pub observation_id: String,
    pub identity_pointer: String,
    #[serde(deserialize_with = "eal_collector_contracts::deserialize_value_exact")]
    pub identity: Value,
    pub unit_pointer: String,
    pub unit: String,
    pub scope_pointer: String,
    #[serde(deserialize_with = "eal_collector_contracts::deserialize_value_exact")]
    pub scope: Value,
    pub valid_from_pointer: String,
    pub valid_until_pointer: String,
    pub valid_from: String,
    pub valid_until: String,
    pub max_age_seconds: u64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contract {
    pub schema: String,
    pub target_id: String,
    pub mode: Mode,
    pub atoms: Vec<AtomBinding>,
    pub obligations: Vec<Obligation>,
    pub applicability: Vec<Applicability>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    #[serde(default)]
    pub status: ObservationStatus,
    #[serde(
        default,
        deserialize_with = "eal_collector_contracts::deserialize_value_exact"
    )]
    pub value: Value,
    pub observed_at: Option<String>,
    pub source_digest: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    pub observations: BTreeMap<String, Observation>,
    pub assessed_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObligationResult {
    pub id: String,
    pub role: Role,
    pub status: Status,
    pub reasons: Vec<String>,
    #[serde(deserialize_with = "eal_collector_contracts::deserialize_value_exact")]
    pub details: Value,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AtomResult {
    pub symbol: String,
    pub observation_id: Option<String>,
    pub status: Status,
    pub truth: Option<bool>,
    pub reasons: Vec<String>,
    #[serde(deserialize_with = "eal_collector_contracts::deserialize_value_exact")]
    pub details: Value,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormulaResult {
    pub id: String,
    pub status: Status,
    pub truth: Option<bool>,
    pub reasons: Vec<String>,
    #[serde(deserialize_with = "eal_collector_contracts::deserialize_value_exact")]
    pub details: Value,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub schema: String,
    pub target_id: String,
    pub mode: Mode,
    pub status: Status,
    pub obligations: Vec<ObligationResult>,
    pub atoms: Vec<AtomResult>,
    pub formulas: Vec<FormulaResult>,
}

/// Native method input keeps the consumer coverage separate from the contract.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Query {
    pub coverage: Coverage,
    pub contract: Contract,
    pub observations: BTreeMap<String, Observation>,
    pub assessed_at: String,
}
