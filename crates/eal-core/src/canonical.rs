//! Bounded canonical JSON identities matching the pinned Python implementation.
use crate::ExecutionLimits;
use serde_json::Value;
use sha2::{Digest, Sha256};

/// Python's shortest binary64 representation, including its exponent convention.
/// Non-finite spellings are useful in diagnostics; canonical JSON rejects them.
pub fn python_float_repr(value: f64) -> String {
    if value.is_nan() {
        return "nan".into();
    }
    if value.is_infinite() {
        return if value.is_sign_negative() {
            "-inf"
        } else {
            "inf"
        }
        .into();
    }
    if value == 0.0 {
        return if value.is_sign_negative() {
            "-0.0"
        } else {
            "0.0"
        }
        .into();
    }
    let mut buffer = ryu::Buffer::new();
    let raw = buffer.format_finite(value.abs());
    let (mantissa, exponent) = raw
        .split_once('e')
        .map(|(m, e)| (m, e.parse::<i32>().expect("Ryu emits a decimal exponent")))
        .unwrap_or((raw, 0));
    let point = mantissa.find('.').unwrap_or(mantissa.len()) as i32;
    let all_digits: String = mantissa.chars().filter(|&c| c != '.').collect();
    let leading = all_digits.bytes().take_while(|&c| c == b'0').count();
    let digits = all_digits[leading..].trim_end_matches('0');
    let decimal_position = point + exponent - leading as i32;
    let scientific_exponent = decimal_position - 1;
    let sign = if value.is_sign_negative() { "-" } else { "" };
    if !(-4..16).contains(&scientific_exponent) {
        let coefficient = if digits.len() == 1 {
            digits.to_owned()
        } else {
            format!("{}.{}", &digits[..1], &digits[1..])
        };
        let exponent_sign = if scientific_exponent < 0 { '-' } else { '+' };
        return format!(
            "{sign}{coefficient}e{exponent_sign}{:02}",
            scientific_exponent.unsigned_abs()
        );
    }
    if decimal_position <= 0 {
        format!(
            "{sign}0.{}{digits}",
            "0".repeat((-decimal_position) as usize)
        )
    } else if decimal_position as usize >= digits.len() {
        format!(
            "{sign}{digits}{}.0",
            "0".repeat(decimal_position as usize - digits.len())
        )
    } else {
        let split = decimal_position as usize;
        format!("{sign}{}.{}", &digits[..split], &digits[split..])
    }
}

pub fn canonical_json_bytes(value: &Value, limits: &ExecutionLimits) -> Result<Vec<u8>, String> {
    limits.validate()?;
    let mut output = Vec::new();
    let mut remaining = limits.json_nodes;
    encode(value, 0, &mut remaining, limits, &mut output)?;
    Ok(output)
}

pub fn canonical_digest(value: &Value, limits: &ExecutionLimits) -> Result<String, String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(canonical_json_bytes(value, limits)?)
    ))
}

/// Typed identity distinguishes integers, floats, booleans and signed floating zero.
pub fn same_json(left: &Value, right: &Value, limits: &ExecutionLimits) -> bool {
    match (
        canonical_json_bytes(left, limits),
        canonical_json_bytes(right, limits),
    ) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

fn encode(
    value: &Value,
    depth: usize,
    remaining: &mut usize,
    limits: &ExecutionLimits,
    output: &mut Vec<u8>,
) -> Result<(), String> {
    if depth > limits.json_depth.min(crate::json::MAX_JSON_DEPTH) || *remaining == 0 {
        return Err("JSON resource limit exceeded".into());
    }
    *remaining -= 1;
    match value {
        Value::Null => output.extend_from_slice(b"null"),
        Value::Bool(value) => output.extend_from_slice(if *value { b"true" } else { b"false" }),
        Value::String(value) => output.extend_from_slice(
            serde_json::to_string(value)
                .expect("Rust strings serialise as JSON")
                .as_bytes(),
        ),
        Value::Number(number) => {
            let raw = number.to_string();
            if raw.contains(['.', 'e', 'E']) {
                let value: f64 = raw.parse().map_err(|_| "JSON numbers must be finite")?;
                if !value.is_finite() {
                    return Err("JSON numbers must be finite".into());
                }
                output.extend_from_slice(python_float_repr(value).as_bytes());
            } else if raw == "-0" {
                output.push(b'0');
            } else {
                output.extend_from_slice(raw.as_bytes());
            }
        }
        Value::Array(values) => {
            output.push(b'[');
            for (index, item) in values.iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                encode(item, depth + 1, remaining, limits, output)?;
            }
            output.push(b']');
        }
        Value::Object(values) => {
            output.push(b'{');
            let mut keys: Vec<_> = values.keys().collect();
            keys.sort();
            for (index, key) in keys.into_iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                output.extend_from_slice(serde_json::to_string(key).expect("JSON key").as_bytes());
                output.push(b':');
                encode(&values[key], depth + 1, remaining, limits, output)?;
            }
            output.push(b'}');
        }
    }
    Ok(())
}
