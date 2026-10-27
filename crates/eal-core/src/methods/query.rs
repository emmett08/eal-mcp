//! Authoring-time checks for the query contract, without acquiring empirical data.
use super::{MethodRegistry, schema_errors_with_limits};
use crate::{ExecutionLimits, canonical};
use serde_json::Value;

pub fn validate_query(method: &str, query: &Value, registry: &MethodRegistry) -> Vec<String> {
    validate_query_with_limits(method, query, registry, &ExecutionLimits::default())
}

pub fn validate_query_with_limits(
    method: &str,
    query: &Value,
    registry: &MethodRegistry,
    limits: &ExecutionLimits,
) -> Vec<String> {
    let Some(contract) = registry.get(method) else {
        return vec![format!("Unknown registered reasoning method {method:?}")];
    };
    let mut errors = schema_errors_with_limits(query, &contract.query_schema, "query", limits);
    match canonical::canonical_json_footprint(query, limits) {
        Ok(footprint) if footprint.bytes > contract.max_input_bytes => {
            errors.push("Method query exceeds its input byte limit".into());
        }
        Err(error) => errors.push(error),
        _ => {}
    }
    if !errors.is_empty() {
        return errors;
    }
    // A complete built-in query can be checked by the authoritative bounded
    // implementation. A partial query never receives invented sample data.
    // Extension callbacks may perform work outside pure validation and are not run.
    if schema_errors_with_limits(query, &contract.input_schema, "query", limits).is_empty()
        && let Some(mode) = &contract.builtin_mode
        && mode != "structured"
        && !mode.starts_with("engineering/")
        // Admission does not execute a calculation that assessment lacks budget
        // to complete. Shape checks above remain independent of calculation.
        && super::work::required(mode, query) <= limits.method_work
        && let Err(error) = super::dispatch::compute_builtin(mode, query)
    {
        errors.push(error);
    }
    errors
}
