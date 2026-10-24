//! Reject known credential leakage in semantic results without changing results.
use eal_collector_contracts::{SensitiveJsonLimits, contains_sensitive_json};
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
        return match contains_sensitive_json(
            &Value::String(diagnostic.into()),
            &[token],
            SensitiveJsonLimits::default(),
        ) {
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
    if contains_sensitive_json(response, &[token], SensitiveJsonLimits::default())
        .map_err(|_| "EAL_HOST_CREDENTIAL_INSPECTION: response inspection failed")?
    {
        return Err("EAL_HOST_CREDENTIAL_LEAK: semantic response contains sensitive data".into());
    }
    Ok(response.clone())
}
