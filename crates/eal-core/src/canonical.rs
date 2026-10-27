//! Bounded canonical JSON identities matching the pinned Python implementation.
use crate::ExecutionLimits;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::io::Write;

mod footprint;
pub(crate) use footprint::{JsonFootprint, canonical_json_footprint};

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
    let mut maximum_depth = 0;
    encode(
        value,
        0,
        &mut remaining,
        &mut maximum_depth,
        limits,
        &mut output,
    )?;
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

fn encode<W: Write>(
    value: &Value,
    depth: usize,
    remaining: &mut usize,
    maximum_depth: &mut usize,
    limits: &ExecutionLimits,
    output: &mut W,
) -> Result<(), String> {
    if depth > limits.json_depth.min(crate::json::MAX_JSON_DEPTH) || *remaining == 0 {
        return Err("JSON resource limit exceeded".into());
    }
    *remaining -= 1;
    *maximum_depth = (*maximum_depth).max(depth);
    match value {
        Value::Null => emit(output, b"null")?,
        Value::Bool(value) => emit(output, if *value { b"true" } else { b"false" })?,
        Value::String(value) => {
            serde_json::to_writer(&mut *output, value).map_err(|error| error.to_string())?;
        }
        Value::Number(number) => {
            let raw = number.as_str();
            if raw.contains(['.', 'e', 'E']) {
                let value: f64 = raw.parse().map_err(|_| "JSON numbers must be finite")?;
                if !value.is_finite() {
                    return Err("JSON numbers must be finite".into());
                }
                emit(output, python_float_repr(value).as_bytes())?;
            } else if raw == "-0" {
                emit(output, b"0")?;
            } else {
                emit(output, raw.as_bytes())?;
            }
        }
        Value::Array(values) => {
            if values.len() > *remaining {
                return Err("JSON resource limit exceeded".into());
            }
            emit(output, b"[")?;
            for (index, item) in values.iter().enumerate() {
                if index != 0 {
                    emit(output, b",")?;
                }
                encode(item, depth + 1, remaining, maximum_depth, limits, output)?;
            }
            emit(output, b"]")?;
        }
        Value::Object(values) => {
            if values.len() > *remaining {
                return Err("JSON resource limit exceeded".into());
            }
            emit(output, b"{")?;
            let mut keys: Vec<_> = values.keys().collect();
            keys.sort();
            for (index, key) in keys.into_iter().enumerate() {
                if index != 0 {
                    emit(output, b",")?;
                }
                serde_json::to_writer(&mut *output, key).map_err(|error| error.to_string())?;
                emit(output, b":")?;
                encode(
                    &values[key],
                    depth + 1,
                    remaining,
                    maximum_depth,
                    limits,
                    output,
                )?;
            }
            emit(output, b"}")?;
        }
    }
    Ok(())
}

fn emit<W: Write>(output: &mut W, bytes: &[u8]) -> Result<(), String> {
    output.write_all(bytes).map_err(|error| error.to_string())
}
