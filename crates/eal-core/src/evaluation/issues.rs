//! Closed evidence diagnostics shared by evaluation and its transport projections.
use serde::{Deserialize, Serialize};

// Declaration order follows wire-code order so retained sets remain canonical.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AvailabilityIssue {
    DependencyUnavailable,
    InvalidObservation,
    MissingObservation,
    OutOfScope,
    PredicateNotMet,
    PredicateUnresolved,
    StaleObservation,
    ToolError,
}
impl AvailabilityIssue {
    pub fn parse(code: &str) -> Option<Self> {
        if code.len() > 32 {
            return None;
        }
        Self::deserialize(serde::de::value::StrDeserializer::<serde::de::value::Error>::new(code))
            .ok()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PredicateIssue {
    Incomparable,
    IncompatibleType,
    InvalidExpression,
    MissingField,
    NonFinite,
    NotMet,
}
impl PredicateIssue {
    pub fn parse(code: &str) -> Option<Self> {
        if code.len() > 32 {
            return None;
        }
        Self::deserialize(serde::de::value::StrDeserializer::<serde::de::value::Error>::new(code))
            .ok()
    }
}
