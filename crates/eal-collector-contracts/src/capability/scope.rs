//! Derive source scope from the actual resolved request before execution or reuse.
use super::{CapabilityDescriptor, HistoricalTimeEncoding};
use serde_json::Value;
use std::collections::BTreeMap;

pub fn scope_from_input(
    capability: &CapabilityDescriptor,
    input: &Value,
) -> Result<BTreeMap<String, Value>, String> {
    capability.validate()?;
    let validator = jsonschema::validator_for(&capability.input_schema)
        .map_err(|_| "Invalid admitted capability input schema")?;
    if !validator.is_valid(input) {
        return Err("Resolved capability input differs from its schema".into());
    }
    let semantics = capability
        .semantics
        .as_ref()
        .ok_or("Source scope requires qualified measurement semantics")?;
    let mut scope = semantics.scope_constraints.clone();
    for (dimension, pointer) in &semantics.scope_bindings {
        let value = input
            .pointer(pointer)
            .filter(|value| {
                !value.is_null() && !value.as_str().is_some_and(|value| value.trim().is_empty())
            })
            .ok_or("A required source scope selector is unresolved")?;
        if scope.get(dimension).is_some_and(|fixed| fixed != value) {
            return Err(
                "Resolved input scope differs from the qualified provider constraint".into(),
            );
        }
        scope.insert(dimension.clone(), value.clone());
    }
    Ok(scope)
}

/// A separately asserted scope cannot disguise request arguments for another subject.
pub fn validate_scope_input(
    capability: &CapabilityDescriptor,
    declared_scope: &BTreeMap<String, Value>,
    input: &Value,
) -> Result<(), String> {
    let actual = scope_from_input(capability, input)?;
    if &actual != declared_scope {
        return Err(
            "Declared source scope differs from its resolved input/configuration scope".into(),
        );
    }
    Ok(())
}

/// An asserted historical window must match the actual qualified query arguments.
pub fn validate_historical_input(
    capability: &CapabilityDescriptor,
    input: &Value,
    start_rfc3339: &str,
    end_rfc3339: &str,
) -> Result<(), String> {
    capability.validate()?;
    let start = chrono::DateTime::parse_from_rfc3339(start_rfc3339)
        .map_err(|_| "Historical start is not RFC3339")?;
    let end = chrono::DateTime::parse_from_rfc3339(end_rfc3339)
        .map_err(|_| "Historical end is not RFC3339")?;
    if start >= end {
        return Err("Historical query interval must be ordered".into());
    }
    let binding = capability
        .semantics
        .as_ref()
        .and_then(|semantics| semantics.time.historical_input.as_ref())
        .ok_or("Historical query has no qualified input binding")?;
    let matches = |pointer: &str, expected: chrono::DateTime<chrono::FixedOffset>| {
        input
            .pointer(pointer)
            .is_some_and(|value| match binding.encoding {
                HistoricalTimeEncoding::UnixSeconds => {
                    value.as_i64() == Some(expected.timestamp())
                        && expected.timestamp_subsec_nanos() == 0
                }
                HistoricalTimeEncoding::UnixSecondsString => {
                    value
                        .as_str()
                        .filter(|value| {
                            !value.is_empty()
                                && value.bytes().enumerate().all(|(index, byte)| {
                                    byte.is_ascii_digit() || (index == 0 && byte == b'-')
                                })
                        })
                        .and_then(|value| value.parse::<i64>().ok())
                        == Some(expected.timestamp())
                        && expected.timestamp_subsec_nanos() == 0
                }
                HistoricalTimeEncoding::Rfc3339 => {
                    value
                        .as_str()
                        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
                        == Some(expected)
                }
            })
    };
    if !matches(&binding.start_pointer, start) || !matches(&binding.end_pointer, end) {
        return Err("Historical query arguments differ from the asserted source interval".into());
    }
    Ok(())
}
