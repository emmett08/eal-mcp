//! Bounded inspection of known sensitive values in JSON and encoded JSON text.

use crate::{JsonDiagnosticCode, JsonLimits, strict_json_exact_diagnostic};
use serde_json::Value;

/// Independent work allowances for inspecting one complete response.
#[derive(Clone, Copy, Debug)]
pub struct SensitiveJsonLimits {
    /// Combined depth through the response and successfully decoded JSON strings.
    pub max_depth: usize,
    /// Visited values and object keys, including decoded values.
    pub max_nodes: usize,
    /// Aggregate bytes scanned for each secret, plus nested JSON decoding input.
    pub max_bytes: usize,
}

impl Default for SensitiveJsonLimits {
    fn default() -> Self {
        Self {
            max_depth: 128,
            max_nodes: 100_000,
            max_bytes: 128 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SensitiveJsonError {
    InvalidLimits,
    ResourceLimit,
    InvalidEncodedJson,
}

impl std::fmt::Display for SensitiveJsonError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidLimits => "invalid sensitive JSON inspection allowances",
            Self::ResourceLimit => "sensitive JSON inspection allowance exhausted",
            Self::InvalidEncodedJson => "ambiguous encoded JSON in sensitive response",
        })
    }
}

impl std::error::Error for SensitiveJsonError {}

/// Inspect immutable semantic data for known values, including JSON Unicode
/// escapes inside repeatedly encoded strings and object keys. Invalid JSON-like
/// prose remains ordinary text. Resource exhaustion is an error, never clearance.
/// This detects known values rather than arbitrary transformed exfiltration.
pub fn contains_sensitive_json(
    value: &Value,
    secrets: &[&str],
    limits: SensitiveJsonLimits,
) -> Result<bool, SensitiveJsonError> {
    if limits.max_depth == 0
        || limits.max_depth > 128
        || limits.max_nodes == 0
        || limits.max_bytes == 0
    {
        return Err(SensitiveJsonError::InvalidLimits);
    }
    if secrets.len() > limits.max_nodes {
        return Err(SensitiveJsonError::ResourceLimit);
    }
    let count = secrets.iter().filter(|secret| !secret.is_empty()).count();
    if count == 0 {
        return Ok(false);
    }
    let mut inspection = Inspection {
        secrets,
        count,
        depth: limits.max_depth,
        nodes: limits.max_nodes,
        bytes: limits.max_bytes,
    };
    for secret in secrets {
        inspection.charge_bytes(secret.len())?;
    }
    inspection.value(value, 0)
}

struct Inspection<'a> {
    secrets: &'a [&'a str],
    count: usize,
    depth: usize,
    nodes: usize,
    bytes: usize,
}

impl Inspection<'_> {
    fn visit(&mut self, depth: usize) -> Result<(), SensitiveJsonError> {
        if depth > self.depth || self.nodes == 0 {
            return Err(SensitiveJsonError::ResourceLimit);
        }
        self.nodes -= 1;
        Ok(())
    }

    fn charge_bytes(&mut self, bytes: usize) -> Result<(), SensitiveJsonError> {
        self.bytes = self
            .bytes
            .checked_sub(bytes)
            .ok_or(SensitiveJsonError::ResourceLimit)?;
        Ok(())
    }

    fn text(&mut self, text: &str, depth: usize) -> Result<bool, SensitiveJsonError> {
        self.charge_bytes(
            text.len()
                .checked_mul(self.count)
                .ok_or(SensitiveJsonError::ResourceLimit)?,
        )?;
        if self
            .secrets
            .iter()
            .any(|secret| !secret.is_empty() && text.contains(secret))
        {
            return Ok(true);
        }
        if !text.trim_start().starts_with(['{', '[', '"']) {
            return Ok(false);
        }
        self.charge_bytes(text.len())?;
        if depth >= self.depth || self.nodes == 0 {
            return Err(SensitiveJsonError::ResourceLimit);
        }
        let decoded = strict_json_exact_diagnostic(
            text.as_bytes(),
            JsonLimits {
                max_bytes: text.len(),
                max_depth: self.depth - depth,
                max_nodes: self.nodes,
            },
        );
        match decoded {
            Ok(value) => self.value(&value, depth + 1),
            Err(error) if error.code == JsonDiagnosticCode::Resource => {
                Err(SensitiveJsonError::ResourceLimit)
            }
            Err(error) if error.code == JsonDiagnosticCode::Duplicate => {
                Err(SensitiveJsonError::InvalidEncodedJson)
            }
            Err(_) => Ok(false),
        }
    }

    fn value(&mut self, value: &Value, depth: usize) -> Result<bool, SensitiveJsonError> {
        self.visit(depth)?;
        match value {
            Value::String(text) => self.text(text, depth),
            Value::Array(values) => {
                for value in values {
                    if self.value(value, depth + 1)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Value::Object(values) => {
                for (key, value) in values {
                    self.visit(depth + 1)?;
                    if self.text(key, depth + 1)? || self.value(value, depth + 1)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Value::Number(number) => self.text(number.as_str(), depth),
            Value::Bool(value) => self.text(if *value { "true" } else { "false" }, depth),
            Value::Null => self.text("null", depth),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn encoded_values_keys_and_scalar_spellings_are_inspected_without_modification() {
        for (value, secret) in [
            (
                json!({"result":"{\"evidence\":\"\\u0073ecret\"}"}),
                "secret",
            ),
            (json!({"{\"\\u0073ecret\":true}":"public"}), "secret"),
            (json!({"result":"\"\\u0073ecret\""}), "secret"),
            (json!({"result":12345}), "234"),
            (json!({"result":false}), "false"),
            (json!({"result":null}), "null"),
        ] {
            let original = value.clone();
            assert!(
                contains_sensitive_json(&value, &[secret], SensitiveJsonLimits::default()).unwrap()
            );
            assert_eq!(value, original);
        }
    }

    #[test]
    fn repeated_encoding_and_all_selected_secrets_are_checked() {
        let mut value = Value::String("{\"evidence\":\"\\u0073ecret\"}".into());
        for _ in 0..4 {
            value = Value::String(serde_json::to_string(&value).unwrap());
        }
        assert!(
            contains_sensitive_json(
                &value,
                &["", "other", "secret"],
                SensitiveJsonLimits::default()
            )
            .unwrap()
        );
        assert!(!contains_sensitive_json(&value, &[""], SensitiveJsonLimits::default()).unwrap());
    }

    #[test]
    fn invalid_json_like_prose_and_unrelated_results_are_unchanged() {
        for value in [
            json!({"result":"[Results] calibration passed"}),
            json!({"result":"{proposal} awaits measurement"}),
            json!({"result":"\"Unclosed quotation"}),
            json!({"result":"{\"public\":true}"}),
        ] {
            let original = value.clone();
            assert!(
                !contains_sensitive_json(&value, &["secret"], SensitiveJsonLimits::default())
                    .unwrap()
            );
            assert_eq!(value, original);
        }
    }

    #[test]
    fn depth_node_and_byte_exhaustion_never_clear_a_response() {
        let value = json!({"result":"{\"nested\":\"\\u0073ecret\"}"});
        for limits in [
            SensitiveJsonLimits {
                max_depth: 1,
                ..Default::default()
            },
            SensitiveJsonLimits {
                max_nodes: 1,
                ..Default::default()
            },
            SensitiveJsonLimits {
                max_bytes: 10,
                ..Default::default()
            },
        ] {
            assert_eq!(
                contains_sensitive_json(&value, &["secret"], limits),
                Err(SensitiveJsonError::ResourceLimit)
            );
        }
        for limits in [
            SensitiveJsonLimits {
                max_depth: 0,
                ..Default::default()
            },
            SensitiveJsonLimits {
                max_depth: 129,
                ..Default::default()
            },
            SensitiveJsonLimits {
                max_nodes: 0,
                ..Default::default()
            },
            SensitiveJsonLimits {
                max_bytes: 0,
                ..Default::default()
            },
        ] {
            assert_eq!(
                contains_sensitive_json(&value, &["secret"], limits),
                Err(SensitiveJsonError::InvalidLimits)
            );
        }
    }

    #[test]
    fn default_scan_allowance_accepts_an_unrelated_eight_mib_response() {
        assert!(
            !contains_sensitive_json(
                &json!({"result":"x".repeat(8 * 1024 * 1024)}),
                &["secret"],
                SensitiveJsonLimits::default()
            )
            .unwrap()
        );
    }

    #[test]
    fn ambiguous_duplicate_key_json_cannot_clear_an_encoded_response() {
        let value = json!({"result":"{\"evidence\":\"public\",\"evidence\":\"\\u0073ecret\"}"});
        assert_eq!(
            contains_sensitive_json(&value, &["secret"], SensitiveJsonLimits::default()),
            Err(SensitiveJsonError::InvalidEncodedJson)
        );
    }

    #[test]
    fn deeply_encoded_json_cannot_hide_a_value_beyond_the_scan_allowance() {
        let encoded = format!("{}\"\\u0073ecret\"{}", "[".repeat(150), "]".repeat(150));
        assert_eq!(
            contains_sensitive_json(
                &json!({"encoded":encoded}),
                &["secret"],
                SensitiveJsonLimits::default()
            ),
            Err(SensitiveJsonError::ResourceLimit)
        );
    }
}
