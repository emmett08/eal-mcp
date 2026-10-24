//! Bounded, duplicate-rejecting finite JSON admission for collection boundaries.
const MAX_JSON_DEPTH: usize = 128;
#[derive(Clone, Copy, Debug)]
pub struct JsonLimits {
    pub max_bytes: usize,
    pub max_depth: usize,
    pub max_nodes: usize,
}
impl Default for JsonLimits {
    fn default() -> Self {
        Self {
            max_bytes: 16 * 1024 * 1024,
            max_depth: 128,
            max_nodes: 1_000_000,
        }
    }
}
use serde::{
    Deserialize, Deserializer,
    de::{MapAccess, Visitor},
};
use serde_json::{Number, Value, value::RawValue};

/// Numeric interpretation belongs to the consuming boundary rather than JSON syntax.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JsonNumericProfile {
    Binary64,
    Exact,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JsonDiagnosticCode {
    Invalid,
    Syntax,
    Unicode,
    Resource,
    Duplicate,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JsonDiagnostic {
    pub code: JsonDiagnosticCode,
    pub message: String,
}
impl std::fmt::Display for JsonDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for JsonDiagnostic {}
impl From<String> for JsonDiagnostic {
    fn from(message: String) -> Self {
        Self {
            code: JsonDiagnosticCode::Invalid,
            message,
        }
    }
}
impl From<&str> for JsonDiagnostic {
    fn from(message: &str) -> Self {
        message.to_owned().into()
    }
}

/// String adapter for callers that do not expose acquisition error categories.
pub fn strict_json(bytes: &[u8], limits: JsonLimits) -> Result<Value, String> {
    strict_json_diagnostic(bytes, limits).map_err(|diagnostic| diagnostic.message)
}
pub fn strict_json_diagnostic(bytes: &[u8], limits: JsonLimits) -> Result<Value, JsonDiagnostic> {
    admit_json(bytes, limits, JsonNumericProfile::Binary64)
}
/// Preserve the established arbitrary-precision serde_json number representation.
pub fn strict_json_exact(bytes: &[u8], limits: JsonLimits) -> Result<Value, String> {
    strict_json_exact_diagnostic(bytes, limits).map_err(|diagnostic| diagnostic.message)
}
pub fn strict_json_exact_diagnostic(
    bytes: &[u8],
    limits: JsonLimits,
) -> Result<Value, JsonDiagnostic> {
    admit_json(bytes, limits, JsonNumericProfile::Exact)
}
fn admit_json(
    bytes: &[u8],
    limits: JsonLimits,
    numbers: JsonNumericProfile,
) -> Result<Value, JsonDiagnostic> {
    if limits.max_depth == 0
        || limits.max_depth > MAX_JSON_DEPTH
        || limits.max_nodes == 0
        || limits.max_bytes == 0
    {
        return Err("EAL_JSON_LIMIT: invalid JSON resource allowances".into());
    }
    let text = std::str::from_utf8(bytes).map_err(|_| JsonDiagnostic {
        code: JsonDiagnosticCode::Unicode,
        message: "EAL_JSON_UNICODE: JSON must contain UTF-8".into(),
    })?;
    if text.len() > limits.max_bytes {
        return Err(JsonDiagnostic {
            code: JsonDiagnosticCode::Resource,
            message: "JSON input exceeds the record byte budget".into(),
        });
    }
    nonfinite(text)?;
    let raw: &RawValue = serde_json::from_str(text).map_err(decode_error)?;
    let mut remaining = limits.max_nodes;
    decode(raw, 0, &mut remaining, &limits, numbers)
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
    limits: &JsonLimits,
    numbers: JsonNumericProfile,
) -> Result<Value, JsonDiagnostic> {
    if *remaining == 0 || depth > limits.max_depth.min(MAX_JSON_DEPTH) {
        return Err(JsonDiagnostic {
            code: JsonDiagnosticCode::Resource,
            message: "JSON resource limit exceeded".into(),
        });
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
                return Err(JsonDiagnostic {
                    code: JsonDiagnosticCode::Resource,
                    message: format!("JSON nesting exceeds {MAX_JSON_DEPTH} container levels"),
                });
            }
            if text.as_bytes()[0] == b'[' {
                let values: Vec<&RawValue> = serde_json::from_str(text).map_err(decode_error)?;
                if values.len() > *remaining {
                    return Err(JsonDiagnostic {
                        code: JsonDiagnosticCode::Resource,
                        message: "JSON resource limit exceeded".into(),
                    });
                }
                values
                    .into_iter()
                    .map(|v| decode(v, depth + 1, remaining, limits, numbers))
                    .collect::<Result<Vec<_>, _>>()
                    .map(Value::Array)
            } else {
                let values: Pairs<'_> =
                    serde_json::from_str(text).map_err(|error| scalar_error(error, text))?;
                if values.0.len() > *remaining {
                    return Err(JsonDiagnostic {
                        code: JsonDiagnosticCode::Resource,
                        message: "JSON resource limit exceeded".into(),
                    });
                }
                let mut object = serde_json::Map::new();
                for (key, value) in values.0 {
                    if object.contains_key(&key) {
                        return Err(JsonDiagnostic {
                            code: JsonDiagnosticCode::Duplicate,
                            message: "EAL_JSON_DUPLICATE: Duplicate JSON key".into(),
                        });
                    }
                    object.insert(key, decode(value, depth + 1, remaining, limits, numbers)?);
                }
                Ok(Value::Object(object))
            }
        }
        _ => {
            if numbers == JsonNumericProfile::Exact {
                return serde_json::from_str::<Value>(text).map_err(decode_error);
            }
            let value = if text.contains(['.', 'e', 'E']) {
                let parsed = text
                    .parse::<f64>()
                    .map_err(|_| format!("Non-finite JSON value {text}"))?;
                Number::from_f64(parsed).ok_or_else(|| format!("Non-finite JSON value {text}"))?
            } else if text == "-0" {
                Number::from(0)
            } else {
                text.parse().map_err(decode_error)?
            };
            Ok(Value::Number(value))
        }
    }
}
fn decode_error(error: serde_json::Error) -> JsonDiagnostic {
    let message = error.to_string();
    if message.contains("surrogate") {
        return "JSON strings must contain valid Unicode scalar values".into();
    }
    JsonDiagnostic {
        code: JsonDiagnosticCode::Syntax,
        message,
    }
}
fn scalar_error(error: serde_json::Error, text: &str) -> JsonDiagnostic {
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
                    return "JSON strings must contain valid Unicode scalar values".into();
                }
                i += 2;
            } else {
                i += 1;
            }
        }
    }
    decode_error(error)
}
fn nonfinite(text: &str) -> Result<(), JsonDiagnostic> {
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
