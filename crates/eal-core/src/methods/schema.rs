//! Validate the closed method-schema subset with explicit traversal budgets.
use super::validation::is_integer;
use crate::ExecutionLimits;
use serde_json::Value;

pub fn schema_errors(value: &Value, schema: &Value, path: &str) -> Vec<String> {
    schema_errors_with_limits(value, schema, path, &ExecutionLimits::default())
}
pub fn schema_errors_with_limits(
    value: &Value,
    schema: &Value,
    path: &str,
    limits: &ExecutionLimits,
) -> Vec<String> {
    let mut errors = Vec::new();
    let mut budget = limits.json_nodes;
    if let Err(error) = check(
        value,
        schema,
        path,
        0,
        limits.json_depth.min(128),
        &mut budget,
        &mut errors,
    ) {
        errors.push(format!("{path}: {error}"));
    }
    errors.truncate(32);
    errors
}
fn consume(depth: usize, max_depth: usize, budget: &mut usize) -> Result<(), String> {
    if depth > max_depth || *budget == 0 {
        return Err("Typed schema resource limit exceeded".into());
    }
    *budget -= 1;
    Ok(())
}
fn equal(
    left: &Value,
    right: &Value,
    depth: usize,
    max_depth: usize,
    budget: &mut usize,
) -> Result<bool, String> {
    consume(depth, max_depth, budget)?;
    Ok(match (left, right) {
        (Value::Number(_), Value::Number(_)) => {
            is_integer(left) == is_integer(right)
                && super::validation::numeric_compare(left, right)? == std::cmp::Ordering::Equal
        }
        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                false
            } else {
                let mut ok = true;
                for (x, y) in a.iter().zip(b) {
                    ok &= equal(x, y, depth + 1, max_depth, budget)?;
                }
                ok
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            if a.len() != b.len() || a.keys().any(|key| !b.contains_key(key)) {
                false
            } else {
                let mut ok = true;
                for (k, v) in a {
                    ok &= equal(v, &b[k], depth + 1, max_depth, budget)?;
                }
                ok
            }
        }
        _ => left == right,
    })
}
fn check(
    item: &Value,
    spec: &Value,
    path: &str,
    depth: usize,
    max_depth: usize,
    budget: &mut usize,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    consume(depth, max_depth, budget)?;
    if let Some(choices) = spec["anyOf"].as_array() {
        for choice in choices {
            let n = errors.len();
            check(item, choice, path, depth + 1, max_depth, budget, errors)?;
            if errors.len() == n {
                return Ok(());
            }
            errors.truncate(n);
        }
        errors.push(format!("{path}: no declared alternative type matches"));
        return Ok(());
    }
    let kind = spec["type"].as_str().unwrap_or("");
    let matches = match kind {
        "object" => item.is_object(),
        "array" => item.is_array(),
        "string" => item.is_string(),
        "boolean" => item.is_boolean(),
        "number" => item.as_f64().is_some_and(f64::is_finite),
        "integer" => is_integer(item),
        "null" => item.is_null(),
        "json" => true,
        _ => false,
    };
    if !matches {
        errors.push(format!("{path}: expected {kind}"));
        return Ok(());
    }
    if item.is_number() && !item.as_f64().is_some_and(f64::is_finite) {
        errors.push(format!("{path}: numbers must be finite"));
        return Ok(());
    }
    if let Some(values) = spec["enum"].as_array() {
        let mut matched = false;
        for value in values {
            if equal(item, value, depth, max_depth, budget)? {
                matched = true;
                break;
            }
        }
        if !matched {
            errors.push(format!("{path}: value is outside the declared enumeration"));
        }
    }
    match item {
        Value::Number(_) => {
            if spec.get("minimum").is_some_and(|n| {
                super::validation::numeric_compare(item, n).is_ok_and(|o| o.is_lt())
            }) || spec.get("maximum").is_some_and(|n| {
                super::validation::numeric_compare(item, n).is_ok_and(|o| o.is_gt())
            }) {
                errors.push(format!("{path}: number is outside declared bounds"));
            }
        }
        Value::String(s) => {
            let n = s.chars().count() as u64;
            if n < spec["minLength"].as_u64().unwrap_or(0)
                || n > spec["maxLength"].as_u64().unwrap_or(4096)
            {
                errors.push(format!("{path}: string length is outside declared bounds"));
            }
        }
        Value::Array(a) => {
            let n = a.len() as u64;
            if n < spec["minItems"].as_u64().unwrap_or(0)
                || n > spec["maxItems"].as_u64().unwrap_or(10_000)
            {
                errors.push(format!("{path}: array length is outside declared bounds"));
            }
            let default = serde_json::json!({"type":"json"});
            for (i, v) in a.iter().enumerate() {
                check(
                    v,
                    spec.get("items").unwrap_or(&default),
                    &format!("{path}[{i}]"),
                    depth + 1,
                    max_depth,
                    budget,
                    errors,
                )?;
            }
        }
        Value::Object(o) => {
            if o.len() > 10_000 {
                errors.push(format!("{path}: object requires at most 10000 string keys"));
                return Ok(());
            }
            if let Some(required) = spec["required"].as_array() {
                for key in required.iter().filter_map(Value::as_str) {
                    if !o.contains_key(key) {
                        errors.push(format!("{path}.{key}: required field is missing"));
                    }
                }
            }
            let default = if kind == "json" {
                serde_json::json!({"type":"json"})
            } else {
                Value::Bool(false)
            };
            for (key, v) in o {
                if let Some(schema) = spec["properties"].get(key) {
                    check(
                        v,
                        schema,
                        &format!("{path}.{key}"),
                        depth + 1,
                        max_depth,
                        budget,
                        errors,
                    )?;
                } else {
                    let additional = spec.get("additionalProperties").unwrap_or(&default);
                    if additional == &Value::Bool(false) {
                        errors.push(format!("{path}.{key}: undeclared field"));
                    } else {
                        check(
                            v,
                            additional,
                            &format!("{path}.{key}"),
                            depth + 1,
                            max_depth,
                            budget,
                            errors,
                        )?;
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn validate_schema(schema: &Value) -> Result<(), String> {
    validate_schema_depth(schema, 0)
}
fn validate_schema_depth(schema: &Value, depth: usize) -> Result<(), String> {
    let keys = [
        "type",
        "properties",
        "required",
        "additionalProperties",
        "items",
        "minItems",
        "maxItems",
        "minimum",
        "maximum",
        "minLength",
        "maxLength",
        "enum",
        "anyOf",
    ];
    let Some(o) = schema.as_object() else {
        return Err("Invalid or unsupported method schema".into());
    };
    if depth > 32 || o.keys().any(|key| !keys.contains(&key.as_str())) {
        return Err("Invalid or unsupported method schema".into());
    }
    if let Some(choices) = o.get("anyOf") {
        let Some(a) = choices.as_array() else {
            return Err("anyOf requires 1..16 alternatives".into());
        };
        if o.len() != 1 || !(1..=16).contains(&a.len()) {
            return Err("anyOf requires 1..16 alternatives".into());
        }
        for c in a {
            validate_schema_depth(c, depth + 1)?;
        }
        return Ok(());
    }
    if ![
        "object", "array", "string", "number", "integer", "boolean", "null", "json",
    ]
    .contains(&schema["type"].as_str().unwrap_or(""))
    {
        return Err("Method schema requires a supported explicit type".into());
    }
    if let Some(properties) = o.get("properties") {
        let Some(properties) = properties.as_object() else {
            return Err("Schema properties must map names to schemas".into());
        };
        for c in properties.values() {
            validate_schema_depth(c, depth + 1)?;
        }
    }
    if let Some(required) = o.get("required") {
        let Some(required) = required.as_array() else {
            return Err("Schema required fields must uniquely name declared properties".into());
        };
        let mut seen = std::collections::BTreeSet::new();
        for key in required {
            let Some(key) = key.as_str() else {
                return Err("Schema required fields must uniquely name declared properties".into());
            };
            if !seen.insert(key) || schema["properties"].get(key).is_none() {
                return Err("Schema required fields must uniquely name declared properties".into());
            }
        }
    }
    for key in ["additionalProperties", "items"] {
        if let Some(c) = o.get(key) {
            if key == "additionalProperties" && c == false {
                continue;
            }
            validate_schema_depth(c, depth + 1)?;
        }
    }
    for key in ["minItems", "maxItems", "minLength", "maxLength"] {
        if let Some(n) = o.get(key)
            && (!is_integer(n) || n.as_u64().is_none_or(|n| n > 100_000))
        {
            return Err("Schema lengths must be bounded nonnegative integers".into());
        }
    }
    for key in ["minimum", "maximum"] {
        if let Some(n) = o.get(key)
            && !n.as_f64().is_some_and(f64::is_finite)
        {
            return Err("Schema numerical bounds must be finite numbers".into());
        }
    }
    for (low, high) in [
        ("minItems", "maxItems"),
        ("minLength", "maxLength"),
        ("minimum", "maximum"),
    ] {
        if let (Some(a), Some(b)) = (o.get(low), o.get(high))
            && super::validation::numeric_compare(a, b)?.is_gt()
        {
            return Err("Schema lower bound cannot exceed its upper bound".into());
        }
    }
    if let Some(values) = o.get("enum") {
        let Some(values) = values.as_array() else {
            return Err("Schema enum must have 1..128 finite JSON entries".into());
        };
        if !(1..=128).contains(&values.len()) {
            return Err("Schema enum must have 1..128 finite JSON entries".into());
        }
        for v in values {
            check_finite(v, 0)?;
        }
    }
    Ok(())
}
fn check_finite(value: &Value, depth: usize) -> Result<(), String> {
    if depth > 64 {
        return Err("Schema JSON depth exceeded".into());
    }
    match value {
        Value::Number(_) => {
            if !value.as_f64().is_some_and(f64::is_finite) {
                return Err("Schema entries require finite numbers".into());
            }
        }
        Value::Array(a) => {
            for x in a {
                check_finite(x, depth + 1)?
            }
        }
        Value::Object(o) => {
            for x in o.values() {
                check_finite(x, depth + 1)?
            }
        }
        _ => {}
    }
    Ok(())
}
