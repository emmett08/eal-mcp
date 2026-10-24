//! Shared bounded domain checks for finite JSON method inputs.
use crate::expressions::numbers;
use serde_json::{Map, Value, json};
use std::cmp::Ordering;

pub fn object<'a>(
    value: &'a Value,
    fields: &[&str],
    label: &str,
) -> Result<&'a Map<String, Value>, String> {
    let Some(object) = value.as_object() else {
        return Err(format!(
            "{label} requires exactly these fields: {}",
            fields.join(", ")
        ));
    };
    if object.len() != fields.len() || fields.iter().any(|key| !object.contains_key(*key)) {
        return Err(format!(
            "{label} requires exactly these fields: {}",
            fields.join(", ")
        ));
    }
    Ok(object)
}
pub fn list<'a>(
    value: &'a Value,
    label: &str,
    min: usize,
    max: usize,
) -> Result<&'a [Value], String> {
    value
        .as_array()
        .filter(|a| (min..=max).contains(&a.len()))
        .map(Vec::as_slice)
        .ok_or_else(|| format!("{label} must be a list with {min}..{max} entries"))
}
pub fn number(value: &Value, label: &str) -> Result<f64, String> {
    if !value.is_number() {
        return Err(format!("{label} must be a finite number"));
    }
    let n = value
        .as_f64()
        .filter(|n| n.is_finite())
        .ok_or_else(|| format!("{label} must be finite and within [-1e100, 1e100]"))?;
    if numbers::compare(value, &json!(-1e100)).is_none_or(|o| o.is_lt())
        || numbers::compare(value, &json!(1e100)).is_none_or(|o| o.is_gt())
    {
        return Err(format!("{label} must be finite and within [-1e100, 1e100]"));
    }
    Ok(n)
}
pub fn is_integer(value: &Value) -> bool {
    value
        .as_number()
        .is_some_and(|n| !n.as_str().bytes().any(|b| matches!(b, b'.' | b'e' | b'E')))
}
pub fn integer(value: &Value, label: &str, min: u64, max: u64) -> Result<u64, String> {
    if !is_integer(value) {
        return Err(format!("{label} must be an integer in [{min}, {max}]"));
    }
    value
        .as_u64()
        .filter(|n| (min..=max).contains(n))
        .ok_or_else(|| format!("{label} must be an integer in [{min}, {max}]"))
}
pub fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.bytes().enumerate().all(|(i, c)| {
            if i == 0 {
                c.is_ascii_alphabetic() || c == b'_'
            } else {
                c.is_ascii_alphanumeric() || c == b'_'
            }
        })
}
pub fn name<'a>(value: &'a Value, label: &str) -> Result<&'a str, String> {
    value
        .as_str()
        .filter(|v| identifier(v))
        .ok_or_else(|| format!("{label} must be an identifier of at most 64 characters"))
}
pub fn text<'a>(value: &'a Value, label: &str) -> Result<&'a str, String> {
    value
        .as_str()
        .filter(|v| !v.trim().is_empty() && v.chars().count() <= 4096)
        .ok_or_else(|| format!("{label} must contain 1..4096 characters and be nonblank"))
}
pub fn probability(value: &Value, label: &str) -> Result<f64, String> {
    let n = number(value, label)?;
    if !(0.0..=1.0).contains(&n) {
        return Err(format!("{label} must be in [0, 1]"));
    }
    Ok(n)
}

pub fn result(ok: bool, reason: &str, details: Value) -> Value {
    json!({"status":if ok{super::MethodResultStatus::Supported}else{super::MethodResultStatus::Unsupported},"reasons":[reason],"details":details})
}

pub fn check_json(value: &Value) -> Result<(), String> {
    let mut pending = vec![(value, 0)];
    let mut count = 0;
    while let Some((item, depth)) = pending.pop() {
        count += 1;
        if count > 100_000 || depth > 64 {
            return Err("Evidence exceeds JSON node or depth limits".into());
        }
        match item {
            Value::Null | Value::Bool(_) => {}
            Value::String(s) => {
                if s.chars().count() > 4096 {
                    return Err("Evidence strings are limited to 4096 characters".into());
                }
            }
            Value::Number(_) => {
                number(item, "Evidence number")?;
            }
            Value::Array(a) => {
                if a.len() > 10_000 {
                    return Err("Evidence list exceeds entry limit".into());
                }
                pending.extend(a.iter().map(|v| (v, depth + 1)));
            }
            Value::Object(o) => {
                if o.len() > 10_000 {
                    return Err("Evidence object exceeds member limit".into());
                }
                for (k, v) in o {
                    if k.chars().count() > 4096 {
                        return Err("Evidence object keys must be bounded strings".into());
                    }
                    pending.push((v, depth + 1));
                }
            }
        }
    }
    Ok(())
}
pub fn numeric_compare(left: &Value, right: &Value) -> Result<Ordering, String> {
    numbers::compare(left, right).ok_or_else(|| "Incomparable numerical values".into())
}
pub fn scalar_equal(left: &Value, right: &Value) -> Result<bool, String> {
    if left.is_array() || left.is_object() || right.is_array() || right.is_object() {
        return Err("Mapped feature values must be JSON scalars".into());
    }
    Ok(if left.is_number() && right.is_number() {
        numeric_compare(left, right)? == Ordering::Equal
    } else {
        left == right
    })
}
