//! Reject known credential leakage in semantic results without changing results.
use serde_json::Value;

const MAX_DEPTH: usize = 128;
const MAX_NODES: usize = 100_000;
fn limits() -> eal_core::ExecutionLimits {
    eal_core::ExecutionLimits {
        json_depth: MAX_DEPTH,
        json_nodes: MAX_NODES,
        record_bytes: 16 * 1024 * 1024,
        ..Default::default()
    }
}
fn replacement(token: &str) -> &'static str {
    if "[redacted]".contains(token) {
        ""
    } else {
        "[redacted]"
    }
}

/// Look inside repeatedly encoded JSON strings as well as decoded JSON. This
/// inspects known values and is not a guarantee against arbitrary encodings.
fn text_contains_secret(
    text: &str,
    token: &str,
    depth: usize,
    budget: &mut usize,
) -> Result<bool, String> {
    if text.contains(token) {
        return Ok(true);
    }
    if text.trim_start().starts_with(['{', '[', '"'])
        && let Ok(decoded) = eal_core::json::parse(text, &limits())
    {
        return contains_secret(&decoded, token, depth + 1, budget);
    }
    Ok(false)
}
fn contains_secret(
    value: &Value,
    token: &str,
    depth: usize,
    budget: &mut usize,
) -> Result<bool, String> {
    if depth > MAX_DEPTH || *budget == 0 {
        return Err(
            "EAL_HOST_CREDENTIAL_INSPECTION: response inspection allowance exhausted".into(),
        );
    }
    *budget -= 1;
    match value {
        Value::String(text) => text_contains_secret(text, token, depth, budget),
        Value::Array(values) => {
            for value in values {
                if contains_secret(value, token, depth + 1, budget)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Value::Object(values) => {
            for (key, value) in values {
                if text_contains_secret(key, token, depth + 1, budget)?
                    || contains_secret(value, token, depth + 1, budget)?
                {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Value::Number(number) => Ok(number.to_string().contains(token)),
        Value::Bool(value) => Ok(value.to_string().contains(token)),
        Value::Null => Ok("null".contains(token)),
    }
}

/// Semantic data is immutable. A known credential in any semantic field, key,
/// numeric value, evidence string or encoded result rejects the entire response.
/// Only a local display-error envelope may redact its explicitly diagnostic text.
/// Fixed error-envelope keys are public protocol literals and remain unchanged.
pub fn sanitise_response(response: &Value, token: Option<&str>) -> Result<Value, String> {
    eal_core::canonical::canonical_json_bytes(response, &limits())?;
    let Some(token) = token.filter(|token| !token.is_empty()) else {
        return Ok(response.clone());
    };
    let object = response
        .as_object()
        .ok_or("Host response must be an object")?;
    if object.get("is_error") == Some(&Value::Bool(true))
        && object
            .keys()
            .all(|key| ["is_error", "error"].contains(&key.as_str()))
    {
        let diagnostic = object
            .get("error")
            .and_then(Value::as_str)
            .ok_or("Host diagnostic must be text")?;
        let mut budget = MAX_NODES;
        return match text_contains_secret(diagnostic, token, 0, &mut budget) {
            Ok(false) => Ok(response.clone()),
            Ok(true) | Err(_) => {
                let display = if diagnostic.contains(token) {
                    diagnostic.replace(token, replacement(token))
                } else {
                    replacement(token).into()
                };
                let display = if display.contains(token) {
                    String::new()
                } else {
                    display
                };
                Ok(serde_json::json!({"is_error":true,"error":display}))
            }
        };
    }
    let mut budget = MAX_NODES;
    if contains_secret(response, token, 0, &mut budget)? {
        return Err("EAL_HOST_CREDENTIAL_LEAK: semantic response contains sensitive data".into());
    }
    Ok(response.clone())
}
