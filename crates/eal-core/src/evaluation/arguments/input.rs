//! Bind authored query fields to explicit method input without inventing observations.
use crate::{
    ExecutionLimits, canonical,
    methods::{MethodRegistry, schema_errors_with_limits},
    propositions,
    semantic::Proposition,
};
use serde_json::{Value, json};

pub(super) fn prepare(
    proposition: &Proposition,
    method: &str,
    selected: Option<(&str, &Value)>,
    registry: &MethodRegistry,
    limits: &ExecutionLimits,
) -> (Option<Value>, Value) {
    if let Some((id, value)) = selected
        && value.get("schema").and_then(Value::as_str) == Some(propositions::SCHEMA)
    {
        return propositions::prepare_binding_with_limits(
            proposition,
            method,
            id,
            value,
            registry,
            limits,
        );
    }
    let mut trace = json!({
        "method":method,"proposition":proposition,"input_unit":proposition.unit,
        "input_origin":if selected.is_some(){"bound_observation"}else{"authored_program"},
        "authored_query":proposition.query,
        "prose_verified":false,"physical_interpretation_verified":false,
        "scope_basis":"authored proposition and explicit evidence/assumption dependencies"
    });
    let Some(contract) = registry.get(method) else {
        return rejected(trace, vec!["Unknown registered method".into()]);
    };
    trace["method_contract"] = contract.describe();
    let Some(query) = proposition.query.as_object() else {
        return rejected(
            trace,
            vec!["Authored method query must be an object".into()],
        );
    };
    let mut payload = match selected {
        Some((id, value)) => {
            trace["evidence_id"] = json!(id);
            let Some(object) = value.as_object() else {
                return rejected(trace, vec!["Bound method input must be an object".into()]);
            };
            object.clone()
        }
        None => serde_json::Map::new(),
    };
    let mut reasons = Vec::new();
    for (key, value) in query {
        if payload
            .get(key)
            .is_some_and(|actual| !canonical::same_json(actual, value, limits))
        {
            reasons.push(format!(
                "Bound input field {key:?} conflicts with the authored method query"
            ));
        } else {
            payload.insert(key.clone(), value.clone());
        }
    }
    let payload = Value::Object(payload);
    reasons.extend(schema_errors_with_limits(
        &payload,
        &contract.input_schema,
        "method input",
        limits,
    ));
    match canonical::canonical_digest(&payload, limits) {
        Ok(digest) => trace["input_payload_digest"] = json!(digest),
        Err(error) => reasons.push(error),
    }
    if !reasons.is_empty() {
        return rejected(trace, reasons);
    }
    trace["formal_query"] = payload.clone();
    trace["status"] = json!("bound");
    trace["reasons"] = json!([]);
    (Some(payload), trace)
}

fn rejected(mut trace: Value, reasons: Vec<String>) -> (Option<Value>, Value) {
    trace["status"] = json!("unsupported");
    trace["reasons"] = json!(reasons);
    (None, trace)
}
