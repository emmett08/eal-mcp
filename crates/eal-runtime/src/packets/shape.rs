use eal_core::{ExecutionLimits, canonical::canonical_json_bytes};
use serde_json::{Map, Value};

pub(super) fn mapping(value: Option<&Value>) -> Option<&Map<String, Value>> {
    value.and_then(Value::as_object)
}
pub(crate) fn identifier(value: &Value) -> Result<String, String> {
    let valid = value.as_str().filter(|s| {
        !s.is_empty()
            && s.len() <= 128
            && s.split('.').all(|segment| {
                let mut bytes = segment.bytes();
                bytes
                    .next()
                    .is_some_and(|c| c.is_ascii_alphabetic() || c == b'_')
                    && bytes.all(|c| c.is_ascii_alphanumeric() || c == b'_')
            })
    });
    valid.map(str::to_owned).ok_or_else(|| {
        "Assessment packet requires bounded EAL identifiers of at most 128 ASCII characters".into()
    })
}
pub(super) fn state(value: Option<&Value>, allowed: &[&str]) -> Result<String, String> {
    value
        .and_then(Value::as_str)
        .filter(|s| allowed.contains(s))
        .map(str::to_owned)
        .ok_or_else(|| "Assessment packet contains an unknown result status".into())
}
pub(crate) fn digest(value: &Value) -> Option<Value> {
    value
        .as_str()
        .filter(|s| {
            s.len() == 64
                && s.bytes()
                    .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        })
        .map(|s| Value::String(s.into()))
}
pub(crate) fn scalar(value: &Value) -> bool {
    value.is_boolean()
        || value.as_number().is_some_and(|n| {
            let text = n.to_string();
            !text.contains(['.', 'e', 'E']) || n.as_f64().is_some_and(f64::is_finite)
        })
}
pub(crate) fn bounded(value: Option<&Value>) -> Option<Value> {
    value
        .and_then(Value::as_str)
        .filter(|s| {
            if *s == "1" {
                return true;
            }
            let mut bytes = s.bytes();
            s.len() <= 128
                && bytes
                    .next()
                    .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
                && bytes.all(|b| b.is_ascii_alphanumeric() || b"_./:-".contains(&b))
        })
        .map(|s| Value::String(s.into()))
}
pub(crate) fn record_identifier(value: &Value) -> Option<Value> {
    value
        .as_str()
        .filter(|s| {
            let mut bytes = s.bytes();
            s.len() <= 128
                && bytes.next().is_some_and(|b| b.is_ascii_alphanumeric())
                && bytes.all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
        })
        .map(|s| Value::String(s.into()))
}
pub(super) fn ids(value: Option<&Value>, prefixed: bool) -> Result<Vec<String>, String> {
    let Some(values) = value.and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    values
        .iter()
        .filter_map(|v| {
            if prefixed {
                v.as_str()
                    .and_then(|s| s.strip_prefix("objection:"))
                    .map(|s| identifier(&Value::String(s.into())))
            } else {
                Some(identifier(v))
            }
        })
        .collect()
}
pub(crate) fn host_limits() -> ExecutionLimits {
    ExecutionLimits {
        record_bytes: 64 * 1024 * 1024,
        json_depth: 128,
        json_nodes: 64 * 1024 * 1024,
        ..ExecutionLimits::default()
    }
}

/// Stored records have a separate admission ceiling from per-observation assessment budgets.
/// The walker checks ignored payloads as well, before any recursive clone or serializer.
pub(crate) fn preflight(value: &Value) -> Result<(), String> {
    walk(value, 0, &mut 0, &mut 0)
}

fn walk(item: &Value, depth: usize, nodes: &mut usize, bytes: &mut usize) -> Result<(), String> {
    *nodes += 1;
    if depth > 128 || *nodes > 64 * 1024 * 1024 {
        return Err("Packet input JSON resource limit exceeded".into());
    }
    *bytes += match item {
        Value::Null => 4,
        Value::Bool(v) => {
            if *v {
                4
            } else {
                5
            }
        }
        Value::String(s) => string_size(s)?,
        Value::Number(n) => {
            if !scalar(item) {
                return Err("Packet input JSON numbers must be finite".into());
            }
            n.to_string().len()
        }
        Value::Array(items) => {
            for item in items {
                walk(item, depth + 1, nodes, bytes)?;
            }
            2 + items.len().saturating_sub(1)
        }
        Value::Object(items) => {
            for (key, value) in items {
                *bytes += string_size(key)? + 1;
                walk(value, depth + 1, nodes, bytes)?;
            }
            2 + items.len().saturating_sub(1)
        }
    };
    if *bytes > 64 * 1024 * 1024 {
        return Err("Packet input byte limit exceeded".into());
    }
    Ok(())
}
fn string_size(s: &str) -> Result<usize, String> {
    if s.len() > 64 * 1024 * 1024 {
        return Err("Packet input byte limit exceeded".into());
    }
    Ok(2 + s
        .bytes()
        .map(|b| match b {
            b'"' | b'\\' | b'\x08' | b'\x0c' | b'\n' | b'\r' | b'\t' => 2,
            0..=31 => 6,
            _ => 1,
        })
        .sum::<usize>())
}
pub(super) fn size(value: &Value) -> Result<usize, String> {
    Ok(canonical_json_bytes(value, &host_limits())?.len())
}
