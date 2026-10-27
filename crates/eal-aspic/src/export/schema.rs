//! Complete validation of the closed keyword vocabulary in the frozen view schema.
//!
//! This is deliberately not a general JSON Schema implementation. Every keyword
//! in the published contract is checked; a changed, unsupported contract fails
//! closed. No references, formats or external schema resources are resolved.
use eal_core::{ExecutionLimits, expressions::numbers};
use serde_json::{Value, json};
use std::{cmp::Ordering, sync::OnceLock};

pub fn view_schema() -> &'static Value {
    static SCHEMA: OnceLock<Value> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../contracts/schemas/aspic-view.schema.json"
        )))
        .expect("frozen aspic-view schema")
    })
}

pub fn source_schema() -> &'static Value {
    static SCHEMA: OnceLock<Value> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        let view = &view_schema()["properties"];
        let origin = &view["arguments"]["items"]["properties"]["origin"]["properties"];
        let node = &view["arguments"]["items"]["properties"];
        let review = &view["formal_directives"]["items"];
        let nullable_name = view["goal_claim"].clone();
        let name = nullable_name["anyOf"][0].clone();
        let text = node["authored_statement"].clone();
        let nullable_review = json!({"anyOf":[review,{"type":"null"}]});
        let entry = json!({"type":"object","properties":{
            "atom":name,"rule_id":name,"conclusion_atom":name,
            "applicability_atom":nullable_name,
            "rule_kind":{"enum":["strict","defeasible"]},
            "statement":text,"reasoning":text,"rationale":text,
            "emitted":{"type":"boolean"},"available":{"type":"boolean"},
            "span":origin["span"],"rank_annotation":nullable_review,
            "strict_annotation":nullable_review,"identity":origin["observation"],
            "reasons":view["unavailable_evidence"]["items"]["properties"]["reasons"],
            "availability_issues":{
                "type":"array","uniqueItems":true,
                "items":view["unavailable_evidence"]["items"]["properties"]["availability_issues"]["items"]
            },"source_file":name,"scope_transfer":origin["scope_transfer"]
        }});
        let mut properties = serde_json::Map::new();
        properties.insert("formal_directives".into(), view["formal_directives"].clone());
        for kind in ["claims", "arguments", "assumptions", "objections", "evidence"] {
            properties.insert(kind.into(), json!({"type":"object","additionalProperties":entry}));
        }
        properties.insert("contraries".into(), json!({"type":"array","items":{
            "type":"object","properties":{
                "attacker":name,"target":name,"objection":name,"annotation":nullable_review
            },"required":["attacker","target"]
        }}));
        properties.insert("structural_basis".into(), json!({
            "type":"object","properties":{"atom":name,"meaning":text},
            "required":["atom","meaning"],"additionalProperties":false
        }));
        json!({"type":"object","properties":properties})
    })
}

#[derive(Debug)]
pub struct Error {
    pub path: String,
    pub message: String,
}

impl Error {
    fn new(path: &str, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

pub fn validate(value: &Value, schema: &Value, limits: &ExecutionLimits) -> Result<(), Error> {
    supported(schema).map_err(|message| Error::new("", message))?;
    let mut remaining = limits.json_nodes;
    check(value, schema, "", 0, &mut remaining, limits)
}

fn supported(schema: &Value) -> Result<(), String> {
    if schema.is_boolean() {
        return Ok(());
    }
    let object = schema.as_object().ok_or("Invalid retained view schema")?;
    for (key, value) in object {
        match key.as_str() {
            "$schema" | "$id" | "title" | "description" | "type" | "const" | "enum" | "minimum"
            | "maximum" | "minLength" | "maxLength" | "minItems" | "maxItems" | "uniqueItems"
            | "required" => {}
            "pattern" if value == "^[0-9a-f]{64}$" => {}
            "properties" => {
                for value in value
                    .as_object()
                    .ok_or("Invalid schema properties")?
                    .values()
                {
                    supported(value)?;
                }
            }
            "items" | "additionalProperties" => supported(value)?,
            "anyOf" => {
                for value in value.as_array().ok_or("Invalid schema alternatives")? {
                    supported(value)?;
                }
            }
            _ => return Err(format!("Unsupported retained view schema keyword {key:?}")),
        }
    }
    Ok(())
}

fn child(path: &str, key: impl std::fmt::Display) -> String {
    if path.is_empty() {
        key.to_string()
    } else {
        format!("{path}.{key}")
    }
}

fn check(
    value: &Value,
    schema: &Value,
    path: &str,
    depth: usize,
    remaining: &mut usize,
    limits: &ExecutionLimits,
) -> Result<(), Error> {
    if depth > limits.json_depth.min(128) || *remaining == 0 {
        return Err(Error::new(path, "View schema resource limit exceeded"));
    }
    *remaining -= 1;
    if schema == &Value::Bool(true) {
        return Ok(());
    }
    if schema == &Value::Bool(false) {
        return Err(Error::new(path, "Additional properties are not allowed"));
    }
    if let Some(options) = schema["anyOf"].as_array()
        && !options
            .iter()
            .any(|option| check(value, option, path, depth + 1, remaining, limits).is_ok())
    {
        return Err(Error::new(
            path,
            "Value is not valid under any of the given schemas",
        ));
    }
    if let Some(kind) = schema["type"].as_str() {
        let matches = match kind {
            "null" => value.is_null(),
            "boolean" => value.is_boolean(),
            "object" => value.is_object(),
            "array" => value.is_array(),
            "string" => value.is_string(),
            "number" => value.is_number(),
            "integer" => {
                value.is_number()
                    && (value
                        .as_number()
                        .is_some_and(|v| !v.to_string().contains(['.', 'e', 'E']))
                        || value
                            .as_f64()
                            .is_some_and(|v| v.is_finite() && v.fract() == 0.0))
            }
            _ => return Err(Error::new(path, "Unsupported retained schema type")),
        };
        if !matches {
            return Err(Error::new(path, format!("Value is not of type {kind:?}")));
        }
    }
    if schema.get("const").is_some_and(|v| !equal(value, v)) {
        return Err(Error::new(
            path,
            "Value does not match the required constant",
        ));
    }
    if let Some(options) = schema["enum"].as_array()
        && !options.iter().any(|option| equal(value, option))
    {
        return Err(Error::new(path, "Value is not one of the permitted values"));
    }
    for (key, ordering) in [("minimum", Ordering::Less), ("maximum", Ordering::Greater)] {
        if let Some(bound) = schema.get(key)
            && numbers::compare(value, bound) == Some(ordering)
        {
            return Err(Error::new(path, format!("Value exceeds its {key}")));
        }
    }
    if let Some(text) = value.as_str() {
        let length = text.chars().count();
        length_check(length, schema, "minLength", "maxLength", path)?;
        if schema.get("pattern").is_some()
            && (text.len() != 64
                || !text
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)))
        {
            return Err(Error::new(path, "Value does not match '^[0-9a-f]{64}$'"));
        }
    }
    if let Some(values) = value.as_array() {
        length_check(values.len(), schema, "minItems", "maxItems", path)?;
        if schema["uniqueItems"] == true {
            for (i, value) in values.iter().enumerate() {
                for previous in &values[..i] {
                    if *remaining == 0 {
                        return Err(Error::new(path, "View schema resource limit exceeded"));
                    }
                    *remaining -= 1;
                    if equal(value, previous) {
                        return Err(Error::new(path, "Array has non-unique elements"));
                    }
                }
            }
        }
        if let Some(items) = schema.get("items") {
            for (i, value) in values.iter().enumerate() {
                check(value, items, &child(path, i), depth + 1, remaining, limits)?;
            }
        }
    }
    if let Some(values) = value.as_object() {
        if let Some(required) = schema["required"].as_array() {
            for key in required.iter().filter_map(Value::as_str) {
                if !values.contains_key(key) {
                    return Err(Error::new(path, format!("{key:?} is a required property")));
                }
            }
        }
        for (key, value) in values {
            if let Some(spec) = schema["properties"]
                .get(key)
                .or_else(|| schema.get("additionalProperties"))
            {
                check(value, spec, &child(path, key), depth + 1, remaining, limits)?;
            }
        }
    }
    Ok(())
}

fn length_check(
    length: usize,
    schema: &Value,
    min: &str,
    max: &str,
    path: &str,
) -> Result<(), Error> {
    if schema[min].as_u64().is_some_and(|v| (length as u64) < v)
        || schema[max].as_u64().is_some_and(|v| length as u64 > v)
    {
        return Err(Error::new(path, format!("Value violates {min}/{max}")));
    }
    Ok(())
}

/// JSON Schema treats numerically equal integers/floats alike, but not booleans.
fn equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(_), Value::Number(_)) => {
            numbers::compare(left, right) == Some(Ordering::Equal)
        }
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| equal(a, b))
        }
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len() && a.iter().all(|(k, a)| b.get(k).is_some_and(|b| equal(a, b)))
        }
        _ => left == right,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_frozen_schema_and_json_schema_scalar_semantics() {
        supported(view_schema()).unwrap();
        supported(source_schema()).unwrap();
        let limits = ExecutionLimits::default();
        let budget_schema = &view_schema()["properties"]["execution_limits"];
        let budgets = limits.describe();
        validate(&budgets, budget_schema, &limits).unwrap();
        assert_eq!(
            budget_schema["properties"]
                .as_object()
                .unwrap()
                .keys()
                .collect::<Vec<_>>(),
            budgets.as_object().unwrap().keys().collect::<Vec<_>>(),
            "The frozen view must capture every executable budget"
        );
        for (value, spec, valid) in [
            (json!(1.0), json!({"type":"integer"}), true),
            (json!(true), json!({"type":"integer"}), false),
            (
                json!("é🧪"),
                json!({"type":"string","minLength":2,"maxLength":2}),
                true,
            ),
            (json!([1, 1.0]), json!({"uniqueItems":true}), false),
            (json!([true, 1]), json!({"uniqueItems":true}), true),
            (
                json!({"extra":1}),
                json!({"additionalProperties":false}),
                false,
            ),
            (
                json!({"extra":1}),
                json!({"additionalProperties":{"type":"integer"}}),
                true,
            ),
            (
                json!("a".repeat(64)),
                json!({"pattern":"^[0-9a-f]{64}$"}),
                true,
            ),
            (
                json!("A".repeat(64)),
                json!({"pattern":"^[0-9a-f]{64}$"}),
                false,
            ),
        ] {
            assert_eq!(
                validate(&value, &spec, &limits).is_ok(),
                valid,
                "{value} {spec}"
            );
        }
        assert!(validate(&Value::Null, &json!({"$ref":"remote"}), &limits).is_err());
    }
}
