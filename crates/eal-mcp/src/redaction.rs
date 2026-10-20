//! Redact remote application keys and encoded JSON while retaining fixed host wrappers.
use serde_json::{Map, Value};
const HOST: &[&str] = &[
    "operation",
    "protocol_version",
    "is_error",
    "result",
    "content",
    "error",
];
const METADATA: &[&str] = &["operation", "protocol_version", "is_error"];
fn replacement(token: &str) -> &'static str {
    if "[redacted]".contains(token) {
        ""
    } else {
        "[redacted]"
    }
}
fn text(value: &str, token: &str) -> String {
    let result = value.replace(token, replacement(token));
    if result.contains(token) {
        String::new()
    } else {
        result
    }
}
fn object(
    value: &Map<String, Value>,
    token: &str,
    public_keys: &[&str],
    public_values: &[&str],
    depth: usize,
    budget: &mut usize,
) -> Result<Value, String> {
    let mut result = Map::new();
    for (key, value) in value {
        let field = if public_keys.contains(&key.as_str()) {
            key.clone()
        } else {
            text(key, token)
        };
        if result.contains_key(&field) {
            return Err("Credential redaction would merge distinct JSON object keys".into());
        }
        result.insert(
            field,
            if public_values.contains(&key.as_str()) {
                value.clone()
            } else {
                redact(value, token, depth + 1, budget)?
            },
        );
    }
    Ok(Value::Object(result))
}
fn redact(value: &Value, token: &str, depth: usize, budget: &mut usize) -> Result<Value, String> {
    if depth > 128 || *budget == 0 {
        return Err("Credential redaction resource limit exceeded".into());
    }
    *budget -= 1;
    match value {
        Value::String(value) => {
            let limits = eal_core::ExecutionLimits {
                json_depth: 128,
                json_nodes: 100_000,
                record_bytes: 16 * 1024 * 1024,
                ..Default::default()
            };
            match eal_core::json::parse(value, &limits) {
                Ok(decoded) if decoded.is_object() || decoded.is_array() || decoded.is_string() => {
                    let sanitised = redact(&decoded, token, depth + 1, budget)?;
                    if sanitised == decoded {
                        Ok(Value::String(value.clone()))
                    } else {
                        Ok(Value::String(python_json(&sanitised)?))
                    }
                }
                Err(_) if value.trim_start().starts_with(['{', '[', '"']) => {
                    Ok(Value::String(replacement(token).into()))
                }
                _ => Ok(Value::String(text(value, token))),
            }
        }
        Value::Array(values) => values
            .iter()
            .map(|v| redact(v, token, depth + 1, budget))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        Value::Object(values) => object(values, token, &[], &[], depth, budget),
        other => Ok(other.clone()),
    }
}
fn content_fields(kind: &str) -> Option<&'static [&'static str]> {
    match kind {
        "text" => Some(&["type", "text", "annotations", "meta", "_meta"]),
        "image" | "audio" => Some(&[
            "type",
            "data",
            "mimeType",
            "mime_type",
            "annotations",
            "meta",
            "_meta",
        ]),
        "resource_link" => Some(&[
            "type",
            "uri",
            "name",
            "title",
            "description",
            "mimeType",
            "mime_type",
            "size",
            "icons",
            "annotations",
            "meta",
            "_meta",
        ]),
        "resource" => Some(&["type", "resource", "annotations", "meta", "_meta"]),
        _ => None,
    }
}
pub fn sanitise_response(response: &Value, token: Option<&str>) -> Result<Value, String> {
    let limits = eal_core::ExecutionLimits {
        json_depth: 128,
        json_nodes: 100_000,
        record_bytes: 16 * 1024 * 1024,
        ..Default::default()
    };
    eal_core::canonical::canonical_json_bytes(response, &limits)?;
    let Some(token) = token.filter(|s| !s.is_empty()) else {
        return Ok(response.clone());
    };
    let values = response
        .as_object()
        .ok_or("Host response must be an object")?;
    let mut budget = 100_000;
    let content = response.get("content").and_then(Value::as_array);
    let protected = if content.is_some() {
        [METADATA, &["content"]].concat()
    } else {
        METADATA.to_vec()
    };
    let mut result = object(values, token, HOST, &protected, 0, &mut budget)?;
    if let Some(content) = content {
        let mut sanitised = Vec::new();
        for block in content {
            let value = if let Some(value) = block.as_object() {
                let fields = value
                    .get("type")
                    .and_then(Value::as_str)
                    .and_then(content_fields);
                object(
                    value,
                    token,
                    fields.unwrap_or(&["type"]),
                    if fields.is_some() { &["type"] } else { &[] },
                    1,
                    &mut budget,
                )?
            } else {
                redact(block, token, 1, &mut budget)?
            };
            sanitised.push(value);
        }
        result["content"] = Value::Array(sanitised);
    }
    Ok(result)
}

// Python json.dumps defaults retain insertion order and put a space after separators.
fn python_json(value: &Value) -> Result<String, String> {
    fn write(value: &Value, out: &mut String) -> Result<(), String> {
        match value {
            Value::Array(values) => {
                out.push('[');
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        out.push_str(", ");
                    }
                    write(value, out)?;
                }
                out.push(']');
            }
            Value::Object(values) => {
                out.push('{');
                for (index, (key, value)) in values.iter().enumerate() {
                    if index > 0 {
                        out.push_str(", ");
                    }
                    out.push_str(&serde_json::to_string(key).map_err(|e| e.to_string())?);
                    out.push_str(": ");
                    write(value, out)?;
                }
                out.push('}');
            }
            _ => {
                let limits = eal_core::ExecutionLimits::default();
                let bytes = eal_core::canonical::canonical_json_bytes(value, &limits)?;
                out.push_str(std::str::from_utf8(&bytes).map_err(|e| e.to_string())?);
            }
        }
        Ok(())
    }
    let mut result = String::new();
    write(value, &mut result)?;
    Ok(result)
}
