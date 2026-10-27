//! Select registered computational evidence and bind results to its exact payload.
use super::validation::{check_json, result};
use super::{MethodRegistry, schema_errors_with_limits};
use crate::{ExecutionLimits, canonical};
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub fn evidence_kind(method: &str, registry: &MethodRegistry) -> Option<String> {
    registry.get(method).and_then(|c| c.evidence_kind.clone())
}

pub fn validate_mode(method: &str, kinds: &[String], registry: &MethodRegistry) -> Vec<String> {
    validate_mode_with_limits(method, kinds, registry, &ExecutionLimits::default())
}
pub fn validate_mode_with_limits(
    method: &str,
    kinds: &[String],
    registry: &MethodRegistry,
    limits: &ExecutionLimits,
) -> Vec<String> {
    let Some(contract) = registry.get(method) else {
        return vec![format!(
            "Unknown registered reasoning method {}; use an installed versioned identifier",
            crate::expressions::python_repr(&Value::String(method.to_owned()))
        )];
    };
    if kinds.len() > limits.declarations {
        return vec!["At most 4096 evidence entries are allowed".into()];
    }
    if let Some(kind) = &contract.evidence_kind
        && kinds.iter().filter(|k| *k == kind).count() != 1
    {
        return vec![format!(
            "Method {} requires exactly one evidence entry of kind {}",
            crate::expressions::python_repr(&Value::String(method.to_owned())),
            crate::expressions::python_repr(&Value::String(kind.to_owned()))
        )];
    }
    Vec::new()
}

pub fn assess_mode(
    method: &str,
    evidence: &[Value],
    premises: &[Value],
    registry: &MethodRegistry,
) -> Value {
    assess_mode_with_limits(
        method,
        evidence,
        premises,
        registry,
        &ExecutionLimits::default(),
    )
}
pub fn assess_mode_with_limits(
    method: &str,
    evidence: &[Value],
    premises: &[Value],
    registry: &MethodRegistry,
    limits: &ExecutionLimits,
) -> Value {
    let mut output = assess(method, evidence, premises, registry, limits).unwrap_or_else(|error| {
        result(
            false,
            &format!("Method evaluation failed: {error}"),
            json!({}),
        )
    });
    output["method"] = Value::String(method.to_owned());
    if let Some(reasons) = output["reasons"].as_array_mut() {
        for reason in reasons {
            let text = reason.as_str().unwrap_or("");
            *reason = Value::String(format!("Method {method}: {text}"));
        }
    }
    output
}
fn assess(
    method: &str,
    evidence: &[Value],
    premises: &[Value],
    registry: &MethodRegistry,
    limits: &ExecutionLimits,
) -> Result<Value, String> {
    if evidence.len() > limits.declarations {
        return Err("Evidence must be a list with within the host declaration budget".into());
    }
    if premises.len() > limits.declarations {
        return Err("Premises must be a list with within the host declaration budget".into());
    }
    if evidence.iter().any(|e| {
        !e.is_object() || !e["kind"].is_string() || e.get("value").is_none() || !e["id"].is_string()
    }) {
        return Err("Each evidence entry requires id, kind and value".into());
    }
    if evidence
        .iter()
        .map(|e| e["id"].as_str().expect("checked id"))
        .collect::<BTreeSet<_>>()
        .len()
        != evidence.len()
    {
        return Err("Evidence identifiers must be unique".into());
    }
    if premises.iter().any(|p| !p.is_object()) {
        return Err("Premise entries must be objects".into());
    }
    for entry in evidence {
        check_json(&entry["value"])?;
    }
    let kinds: Vec<_> = evidence
        .iter()
        .map(|e| e["kind"].as_str().expect("checked kind").to_owned())
        .collect();
    let errors = validate_mode_with_limits(method, &kinds, registry, limits);
    if !errors.is_empty() {
        return Ok(json!({"status":"unsupported","reasons":errors,"details":{}}));
    }
    let contract = registry.get(method).expect("validated method");
    let selected = if contract.builtin_mode.as_deref() == Some("structured") {
        None
    } else {
        evidence
            .iter()
            .find(|e| e["kind"].as_str() == contract.evidence_kind.as_deref())
    };
    let empty = json!({});
    let payload = selected.map(|e| &e["value"]).unwrap_or(&empty);
    let available = !evidence.is_empty()
        || premises
            .iter()
            .any(|p| matches!(p["status"].as_str(), Some("supported" | "contested")));
    let mut output = evaluate_payload(contract, payload, available, limits)?;
    if let Some(entry) = selected {
        output["evidence_id"] = entry["id"].clone();
        output["input_digest"] = Value::String(canonical::canonical_digest(payload, limits)?);
    }
    Ok(output)
}

/// Evaluate explicit program data without pretending that it was acquired evidence.
/// Empirical support and applicability are checked by the argument evaluator.
pub fn assess_input_with_limits(
    method: &str,
    payload: &Value,
    registry: &MethodRegistry,
    limits: &ExecutionLimits,
) -> Value {
    let assessed = (|| {
        let contract = registry.get(method).ok_or("Unknown registered method")?;
        if contract.evidence_kind.is_none() {
            return Err("An authored method input requires a computational method".into());
        }
        let mut output = evaluate_payload(contract, payload, false, limits)?;
        output["input_digest"] = Value::String(canonical::canonical_digest(payload, limits)?);
        Ok(output)
    })();
    let mut output = assessed.unwrap_or_else(|error: String| {
        result(
            false,
            &format!("Method evaluation failed: {error}"),
            json!({}),
        )
    });
    output["method"] = json!(method);
    output["input_origin"] = json!("authored_program");
    if let Some(reasons) = output["reasons"].as_array_mut() {
        for reason in reasons {
            let text = reason.as_str().unwrap_or("");
            *reason = json!(format!("Method {method}: {text}"));
        }
    }
    output
}

fn evaluate_payload(
    contract: &super::MethodContract,
    payload: &Value,
    support_available: bool,
    limits: &ExecutionLimits,
) -> Result<Value, String> {
    let encoded = canonical::canonical_json_bytes(payload, limits)?;
    if encoded.len() > contract.max_input_bytes {
        return Err("Method input exceeds byte limit".into());
    }
    let errors = schema_errors_with_limits(payload, &contract.input_schema, "$", limits);
    if !errors.is_empty() {
        return Err(format!(
            "Method input contract violation: {}",
            errors.join("; ")
        ));
    }
    let output = if contract.builtin_mode.as_deref() == Some("structured") {
        let available = support_available;
        result(
            available,
            if available {
                "Authored support is available; prose sufficiency is not mechanically established"
            } else {
                "Structured reasoning requires evidence or a supported premise"
            },
            json!({"authored":true,"mechanically_proved":false}),
        )
    } else if let Some(mode) = &contract.builtin_mode
        && contract.implementation.is_none()
    {
        if mode.starts_with("engineering/") {
            let mut output = match super::optional::compute(mode, payload) {
                Ok(details) => result(
                    true,
                    &format!("Computed registered method {}", contract.identifier),
                    details,
                ),
                Err(error) => result(
                    false,
                    &format!("Method execution failed: {error}"),
                    json!({}),
                ),
            };
            output["method_contract"] = contract.describe();
            output
        } else {
            let work = super::work::required(mode, payload);
            if work > limits.method_work {
                json!({
                    "status":super::MethodResultStatus::Incomplete,
                    "reasons":[format!("Native calculation requires {work} work units; method_work allowance is {}", limits.method_work)],
                    "details":{},
                    "resource":{"kind":"method_work","required":work,"allowance":limits.method_work}
                })
            } else {
                compute_builtin(mode, payload)?
            }
        }
    } else if let Some(callback) = &contract.implementation {
        let mut output = match callback.call(payload, limits) {
            Ok(details) => result(
                true,
                &format!("Computed registered method {}", contract.identifier),
                details,
            ),
            Err(super::MethodError::Invalid(error)) => result(
                false,
                &format!("Method execution failed: {error}"),
                json!({}),
            ),
            Err(super::MethodError::Incomplete(error)) => {
                json!({"status":super::MethodResultStatus::Incomplete,"reasons":[format!("Method execution incomplete: {error}")],"details":{}})
            }
        };
        output["method_contract"] = contract.describe();
        output
    } else {
        return Err(
            "A registered method requires a native implementation or a runtime executor".into(),
        );
    };
    if output["status"] == "supported" {
        let errors =
            schema_errors_with_limits(&output["details"], &contract.output_schema, "$", limits);
        if !errors.is_empty() {
            return Err(format!(
                "Method output contract violation: {}",
                errors.join("; ")
            ));
        }
        if canonical::canonical_json_bytes(&output["details"], limits)?.len()
            > contract.max_output_bytes
        {
            return Err("Method output exceeds byte limit".into());
        }
    }
    Ok(output)
}

pub(crate) fn compute_builtin(mode: &str, payload: &Value) -> Result<Value, String> {
    match mode {
        "deductive" => super::logical::deductive(payload),
        "inductive" => super::probabilistic::inductive(payload),
        "abductive" => super::probabilistic::abductive(payload),
        "causal" => super::numerical::causal(payload),
        "counterfactual" => super::structural::counterfactual(payload),
        "analogical" => super::structural::analogical(payload),
        "temporal" => super::structural::temporal(payload),
        _ => Err("Unknown built-in calculation".into()),
    }
}
