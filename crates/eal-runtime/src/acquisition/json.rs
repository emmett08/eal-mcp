use super::{AcquisitionError, AcquisitionFailureCategory, MAX_JSON_DEPTH, MAX_OUTPUT_BYTES};
use eal_core::{ExecutionLimits, canonical};
use serde::{
    Deserialize, Deserializer,
    de::{MapAccess, Visitor},
};
use serde_json::{Number, Value, value::RawValue};

pub fn strict_json(text: &str) -> Result<Value, AcquisitionError> {
    strict_json_with_limits(
        text,
        &ExecutionLimits {
            json_depth: MAX_JSON_DEPTH,
            record_bytes: MAX_OUTPUT_BYTES,
            ..ExecutionLimits::default()
        },
    )
}
pub fn strict_json_with_limits(
    text: &str,
    limits: &ExecutionLimits,
) -> Result<Value, AcquisitionError> {
    limits.validate()?;
    if text.len() > limits.record_bytes {
        return Err("JSON input exceeds the record byte budget".into());
    }
    nonfinite(text)?;
    let raw: &RawValue = serde_json::from_str(text).map_err(decode_error)?;
    let mut remaining = limits.json_nodes;
    decode(raw, 0, &mut remaining, limits)
}
struct Pairs<'a>(Vec<(String, &'a RawValue)>);
impl<'de> Deserialize<'de> for Pairs<'de> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct PairVisitor;
        impl<'de> Visitor<'de> for PairVisitor {
            type Value = Pairs<'de>;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a JSON object")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(value) = map.next_entry()? {
                    values.push(value)
                }
                Ok(Pairs(values))
            }
        }
        deserializer.deserialize_map(PairVisitor)
    }
}
fn decode(
    raw: &RawValue,
    depth: usize,
    remaining: &mut usize,
    limits: &ExecutionLimits,
) -> Result<Value, AcquisitionError> {
    if *remaining == 0 || depth > limits.json_depth.min(MAX_JSON_DEPTH) {
        return Err("JSON resource limit exceeded".into());
    }
    *remaining -= 1;
    let text = raw.get();
    match text.as_bytes()[0] {
        b'n' => Ok(Value::Null),
        b't' => Ok(Value::Bool(true)),
        b'f' => Ok(Value::Bool(false)),
        b'"' => serde_json::from_str::<String>(text)
            .map(Value::String)
            .map_err(|error| scalar_error(error, text)),
        b'[' | b'{' => {
            if depth >= MAX_JSON_DEPTH {
                return Err(
                    format!("JSON nesting exceeds {MAX_JSON_DEPTH} container levels").into(),
                );
            }
            if text.as_bytes()[0] == b'[' {
                let values: Vec<&RawValue> = serde_json::from_str(text).map_err(decode_error)?;
                if values.len() > *remaining {
                    return Err("JSON resource limit exceeded".into());
                }
                values
                    .into_iter()
                    .map(|v| decode(v, depth + 1, remaining, limits))
                    .collect::<Result<Vec<_>, _>>()
                    .map(Value::Array)
            } else {
                let values: Pairs<'_> =
                    serde_json::from_str(text).map_err(|error| scalar_error(error, text))?;
                if values.0.len() > *remaining {
                    return Err("JSON resource limit exceeded".into());
                }
                let mut object = serde_json::Map::new();
                for (key, value) in values.0 {
                    if object.contains_key(&key) {
                        let repr =
                            eal_core::expressions::diagnostic_repr(&Value::String(key), limits)?;
                        return Err(format!("Duplicate JSON key {repr}").into());
                    }
                    object.insert(key, decode(value, depth + 1, remaining, limits)?);
                }
                Ok(Value::Object(object))
            }
        }
        _ => {
            let value = if text.contains(['.', 'e', 'E']) {
                let parsed = text.parse::<f64>().map_err(|_| {
                    AcquisitionError::invalid(format!("Non-finite JSON value {text}"))
                })?;
                Number::from_f64(parsed).ok_or_else(|| {
                    AcquisitionError::invalid(format!("Non-finite JSON value {text}"))
                })?
            } else if text == "-0" {
                Number::from(0)
            } else {
                text.parse().map_err(decode_error)?
            };
            Ok(Value::Number(value))
        }
    }
}
fn decode_error(error: serde_json::Error) -> AcquisitionError {
    let message = error.to_string();
    if message.contains("surrogate") {
        return AcquisitionError::invalid("JSON strings must contain valid Unicode scalar values");
    }
    AcquisitionError {
        category: AcquisitionFailureCategory::JsonDecodeError,
        message,
    }
}
fn scalar_error(error: serde_json::Error, text: &str) -> AcquisitionError {
    if error.to_string().contains("unexpected end of hex escape") {
        let bytes = text.as_bytes();
        let mut i = 0;
        while i + 1 < bytes.len() {
            if bytes[i] == b'\\' {
                if bytes[i + 1] == b'u'
                    && i + 6 <= bytes.len()
                    && let Ok(hex) = std::str::from_utf8(&bytes[i + 2..i + 6])
                    && let Ok(value) = u16::from_str_radix(hex, 16)
                    && (0xd800..=0xdfff).contains(&value)
                {
                    return AcquisitionError::invalid(
                        "JSON strings must contain valid Unicode scalar values",
                    );
                }
                i += 2;
            } else {
                i += 1;
            }
        }
    }
    decode_error(error)
}
fn nonfinite(text: &str) -> Result<(), AcquisitionError> {
    let bytes = text.as_bytes();
    let mut i = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    while i < bytes.len() {
        let c = bytes[i];
        if quoted {
            if escaped {
                escaped = false
            } else if c == b'\\' {
                escaped = true
            } else if c == b'"' {
                quoted = false
            }
            i += 1;
            continue;
        }
        if c == b'"' {
            quoted = true;
            i += 1;
            continue;
        }
        if i == 0 || bytes[i - 1].is_ascii_whitespace() || b"[:,[".contains(&bytes[i - 1]) {
            for value in ["NaN", "Infinity", "-Infinity"] {
                if bytes[i..].starts_with(value.as_bytes()) {
                    return Err(format!("Non-finite JSON value {value}").into());
                }
            }
        }
        i += 1;
    }
    Ok(())
}

pub fn validate_envelope(
    stdout: &[u8],
    file_import: bool,
    context: &Value,
    acquisition: &Value,
) -> Result<Value, AcquisitionError> {
    validate_envelope_with_limits(
        stdout,
        file_import,
        context,
        acquisition,
        &ExecutionLimits {
            json_depth: MAX_JSON_DEPTH,
            record_bytes: MAX_OUTPUT_BYTES,
            ..ExecutionLimits::default()
        },
    )
}
pub fn validate_envelope_with_limits(
    stdout: &[u8],
    file_import: bool,
    context: &Value,
    acquisition: &Value,
    limits: &ExecutionLimits,
) -> Result<Value, AcquisitionError> {
    let text = std::str::from_utf8(stdout).map_err(|error| AcquisitionError {
        category: AcquisitionFailureCategory::UnicodeDecodeError,
        message: error.to_string(),
    })?;
    let envelope = strict_json_with_limits(text, limits)?;
    let object=envelope.as_object().filter(|v|v.contains_key("value")&&v.keys().all(|k|["value","observed_at","context","request","details"].contains(&k.as_str())))
        .ok_or("Tool output must be an object with value and optional observed_at, context, request, details")?;
    if file_import
        && ["observed_at", "context", "request"]
            .iter()
            .any(|k| !object.contains_key(*k))
    {
        return Err("File observations require observed_at and context and request; import must preserve age, scope and acquisition identity".into());
    }
    if let Some(request) = object.get("request")
        && (!request.is_object()
            || canonical::canonical_digest(request, limits)?
                != canonical::canonical_digest(acquisition, limits)?)
    {
        return Err("Observation request differs from the declared acquisition request".into());
    }
    if let Some(supplied) = object.get("context")
        && (!supplied.is_object()
            || canonical::canonical_digest(supplied, limits)?
                != canonical::canonical_digest(context, limits)?)
    {
        return Err("Observation context differs from the requested context".into());
    }
    Ok(envelope)
}
